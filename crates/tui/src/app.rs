//! The event-driven render loop and the view-model it drives.
//!
//! The loop is split into a pure step function ([`App::tick`]) and a thin driver
//! ([`run_with`]) that feeds it terminal events. This keeps the loop's logic
//! testable without a TTY: tests call `tick` directly with scripted events and a
//! stub state source.
//!
//! INV-5 (view-only): [`App`] holds nothing but a [`ViewModel`] snapshot and a
//! view-local quit flag. All durable state lives in the core and is read through
//! the [`SnapshotSource`] seam — the loop never mutates domain state.
//!
//! Command dispatch (`crate::dispatch`) goes through the real, shared
//! `cronus_core::invocable::Dispatcher` — the same one every surface uses —
//! never a local re-implementation. `App` holds the registry and dispatcher
//! directly rather than behind a trait object: there is exactly one dispatch
//! mechanism now, not a swappable one, so the indirection a trait bought
//! before this migration buys nothing here.

use std::io::{self, Stdout};
use std::time::Duration;

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend as RatatuiCrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Widget};

use crate::command::{self, CommandOutcome, CommandSpec, SlashCommand};
use crate::dispatch;
use crate::pane_actions::{self, PaneAction};
use crate::terminal::{CrosstermBackend, Key, TermEvent, TerminalBackend, Tui};
use crate::view::{self, BoardView, Focus, OfficeView, SessionsView};
use cronus_contract::{ArgValues, Dispatched, Invocable, Invocation, Surface};
use cronus_core::invocable::{Dispatcher, InvocableRegistry};
use cronus_domain::{Capabilities, Engine};

/// How long the loop blocks for input before ticking again. Bounds redraw
/// latency for state changes that arrive without a terminal event.
const TICK: Duration = Duration::from_millis(50);

/// An immutable projection of durable core state at one instant.
///
/// The TUI renders from this and never mutates it (INV-5). The version/status
/// fields come from the core capability surface; the board/office projections are
/// populated once the core exposes a cheap board snapshot — until then they stay
/// empty and the panels render empty columns.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreSnapshot {
    /// Engine/product version string.
    pub version: String,
    /// Human-readable status line.
    pub status: String,
    /// Kanban board projection (cards by column).
    pub board: BoardView,
    /// Office projection (agents and their current tasks).
    pub office: OfficeView,
    /// Sessions/log projection (bounded tail of recent activity).
    pub sessions: SessionsView,
}

/// View-only state the panels render from.
///
/// A pure function of the last [`CoreSnapshot`] plus terminal-local view state
/// (currently the size). It accumulates no domain state, so feeding the same
/// snapshot repeatedly is idempotent.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ViewModel {
    /// The latest core snapshot the view reflects.
    pub snapshot: CoreSnapshot,
    /// Terminal size as `(cols, rows)`.
    pub size: (u16, u16),
    /// Which panel currently has keyboard focus. View state, not domain state.
    pub focus: Focus,
    /// In-progress command-bar text (the part after the `/` prompt). View state.
    pub command_input: String,
    /// The result of the last submitted command (help summary / error / ack),
    /// shown beside the prompt until the next keystroke.
    pub command_feedback: Option<String>,
}

/// The seam the loop reads core state through.
///
/// The core exposes no event/observe bus yet, so the production source
/// ([`CapabilitySource`]) *polls* a fresh snapshot each tick — the poll-snapshot
/// fallback the spec mandates. An event-driven source can implement the same
/// trait unchanged if the core later grows a subscription.
pub trait SnapshotSource {
    /// Return the latest snapshot if one is available, else `None`.
    ///
    /// `None` models a snapshot still being produced (a slow core call in
    /// flight). The loop must stay responsive to input when this happens.
    fn poll_snapshot(&mut self) -> Option<CoreSnapshot>;
}

/// Draws a single frame from the immutable view-model.
///
/// Taking `&ViewModel` (never `&mut`) enforces render-from-state: a renderer
/// cannot smuggle mutable state back into the loop.
pub trait Renderer {
    /// Render one frame. Called at most once per [`App::tick`].
    fn draw(&mut self, view: &ViewModel) -> io::Result<()>;
}

/// Outcome of a single [`App::tick`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickResult {
    /// Whether a frame was drawn this tick.
    pub redrawn: bool,
    /// Whether the loop should exit after this tick.
    pub quit: bool,
}

/// The render loop's state: the view-model, a quit flag, and the command-dispatch
/// machinery. Still no domain state — the registry/dispatcher pair reaches the
/// core only through the same public composition door every surface uses, and
/// redaction happens once, inside the shared dispatcher (INV-7), never here.
pub struct App {
    view: ViewModel,
    should_quit: bool,
    /// A command submitted this tick, awaiting dispatch by the loop.
    pending_dispatch: Option<SlashCommand>,
    /// The shared invocable catalog — resolves a slash command's candidate
    /// identity and its declared binders before dispatch.
    registry: InvocableRegistry,
    /// Runs a resolved invocation for real, masking secrets at the boundary.
    dispatcher: Dispatcher,
    /// The slash-command catalog, built once at construction from the core's
    /// invocable registry (`command::build_catalog`) — never a hand-maintained
    /// list. Held on `App` rather than re-derived per keystroke, since it
    /// changes only if the registry itself does.
    catalog: Vec<CommandSpec>,
}

impl App {
    /// Construct an app over a real (or test-constructed) registry/dispatcher
    /// pair and catalog. An empty registry with nothing attached is a valid,
    /// inert choice for scenarios where dispatch behavior is irrelevant —
    /// every slash command then resolves to `Dispatched::Unknown`, exactly
    /// as a genuinely unbound verb would.
    pub fn new(
        initial: ViewModel,
        registry: InvocableRegistry,
        dispatcher: Dispatcher,
        catalog: Vec<CommandSpec>,
    ) -> Self {
        Self {
            view: initial,
            should_quit: false,
            pending_dispatch: None,
            registry,
            dispatcher,
            catalog,
        }
    }

    /// The current view-model the panels render from.
    pub fn view(&self) -> &ViewModel {
        &self.view
    }

    /// Process one loop iteration.
    ///
    /// Order matters: input is handled first so a pending snapshot (a slow core
    /// call) can never starve the keyboard. The snapshot is then pulled cheaply;
    /// if it changed, the view-model updates. Exactly one redraw is issued when
    /// anything changed this tick.
    pub fn tick<S: SnapshotSource, R: Renderer>(
        &mut self,
        events: &[TermEvent],
        source: &mut S,
        renderer: &mut R,
    ) -> io::Result<TickResult> {
        let mut needs_redraw = false;

        // 1) Input first — never blocked by snapshot work.
        for event in events {
            match event {
                TermEvent::Resize(cols, rows) => {
                    self.view.size = (*cols, *rows);
                    needs_redraw = true;
                }
                TermEvent::Key(key) => {
                    if self.handle_key(*key) {
                        needs_redraw = true;
                    }
                }
            }
        }

        // 1b) Dispatch a submitted command through the shared registry/dispatcher.
        //     Resolution, binding, dispatch, and INV-7 masking all happen inside
        //     `dispatch::dispatch_command` — no raw secret value reaches the
        //     view-model or the screen buffer. `None` means the line resolved
        //     to no invocable (`Dispatched::Unknown`) — ordinary input, not a
        //     rendered failure (l2-tui v1.2.0), so feedback is cleared exactly
        //     as it would be for any other line with nothing to report.
        if let Some(command) = self.pending_dispatch.take() {
            self.view.command_feedback =
                dispatch::dispatch_command(&self.registry, &self.dispatcher, &command);
            needs_redraw = true;
        }

        // 2) Snapshot poll — may be `None` (slow call in flight); the loop has
        //    already handled input above, so it stays responsive regardless.
        if let Some(snapshot) = source.poll_snapshot()
            && snapshot != self.view.snapshot
        {
            self.view.snapshot = snapshot;
            needs_redraw = true;
        }

        // 3) Coalesce to exactly one redraw per changed tick.
        if needs_redraw {
            renderer.draw(&self.view)?;
        }

        Ok(TickResult {
            redrawn: needs_redraw,
            quit: self.should_quit,
        })
    }

    /// Route a key press by the focused region; returns whether a redraw is
    /// needed. Tab / Shift+Tab cycle focus from anywhere. Outside the command
    /// bar, Esc quits; inside it, keys edit the command line (Esc cancels).
    ///
    /// Which key triggers which action is decided right here — key bindings
    /// stay local presentation (l2-tui.md §4.4 v1.2.0 Notes). What does NOT
    /// stay local is what each action *is*: every one of these is a real
    /// `ClientLocal` invocable ([`pane_actions`]), and this match only ever
    /// names which one a key requests, never what running it does.
    fn handle_key(&mut self, key: Key) -> bool {
        match key {
            Key::Tab => self.dispatch_pane_action(PaneAction::FocusNext),
            Key::BackTab => self.dispatch_pane_action(PaneAction::FocusPrev),
            _ if self.view.focus == Focus::CommandBar => self.handle_command_key(key),
            Key::Esc => self.dispatch_pane_action(PaneAction::Quit),
            _ => false,
        }
    }

    /// Dispatch one of this surface's own actions through the shared
    /// registry/dispatcher — the only path from a key press to a view
    /// mutation. `action` is applied only once the real dispatch answers
    /// `Dispatched::Ran`; an action that never registered (or lost its
    /// handler) answers `Dispatched::Unknown`, exactly like a genuinely
    /// unbound verb, and nothing happens here — there is no second,
    /// key-code-to-behaviour path that could run anyway.
    fn dispatch_pane_action(&mut self, action: PaneAction) -> bool {
        let invocation = Invocation {
            id: action.id(),
            args: ArgValues::new(),
            caller: Surface::Tui,
        };
        match self.dispatcher.dispatch(&self.registry, &invocation) {
            Dispatched::Unknown => false,
            Dispatched::Ran(_outcome) => match action {
                PaneAction::FocusNext => {
                    self.view.focus = self.view.focus.next();
                    true
                }
                PaneAction::FocusPrev => {
                    self.view.focus = self.view.focus.prev();
                    true
                }
                PaneAction::Quit => {
                    self.should_quit = true;
                    false
                }
            },
        }
    }

    /// Command-bar text entry. Esc cancels the in-progress line rather than
    /// quitting the app.
    fn handle_command_key(&mut self, key: Key) -> bool {
        match key {
            Key::Char(c) => {
                self.view.command_input.push(c);
                self.view.command_feedback = None;
                true
            }
            Key::Backspace => {
                self.view.command_input.pop();
                self.view.command_feedback = None;
                true
            }
            Key::Enter => {
                self.submit_command();
                true
            }
            Key::Esc => {
                self.view.command_input.clear();
                self.view.command_feedback = None;
                true
            }
            _ => false,
        }
    }

    /// Classify the current command line. Help and errors resolve inline; a
    /// recognized command is queued for the loop to dispatch (the loop holds the
    /// registry/dispatcher and performs the real call + masking).
    fn submit_command(&mut self) {
        let line = format!("/{}", self.view.command_input);
        match command::classify(&line, &self.catalog) {
            CommandOutcome::Help => {
                self.view.command_feedback = Some(format!(
                    "commands: {}",
                    command::names(&self.catalog).collect::<Vec<_>>().join(" ")
                ));
            }
            CommandOutcome::Run(command) => self.pending_dispatch = Some(command),
            CommandOutcome::Error(message) => self.view.command_feedback = Some(message),
        }
        self.view.command_input.clear();
    }
}

/// Production [`SnapshotSource`] over the core capability surface.
///
/// Reads `version`/`status` each tick — a cheap call — and hands the loop a fresh
/// snapshot every time; [`App::tick`] dedupes by equality so identical reads do
/// not trigger redraws. Heavier future snapshots can be produced off-thread and
/// delivered through this same poll seam without changing the loop.
pub struct CapabilitySource<C: Capabilities> {
    core: C,
}

impl<C: Capabilities> CapabilitySource<C> {
    /// Wrap a core handle as a snapshot source.
    pub fn new(core: C) -> Self {
        Self { core }
    }
}

impl<C: Capabilities> SnapshotSource for CapabilitySource<C> {
    fn poll_snapshot(&mut self) -> Option<CoreSnapshot> {
        Some(CoreSnapshot {
            version: self.core.version().to_string(),
            status: self.core.status(),
            // Board/office stay empty until the core exposes a cheap snapshot;
            // the panels render empty columns in the meantime.
            ..CoreSnapshot::default()
        })
    }
}

/// Production [`Renderer`] drawing the panel layout via ratatui.
///
/// Holds a ratatui terminal over the same crossterm version the [`Tui`] guard
/// drives, so the two share one alternate screen: the guard owns the raw-mode
/// lifecycle while ratatui owns frame diffing. Panels render purely from the
/// view-model (INV-5); content for each panel arrives in the panel tracks — this
/// renderer establishes the bordered skeleton and the focus highlight.
pub struct RatatuiRenderer {
    terminal: Terminal<RatatuiCrosstermBackend<Stdout>>,
}

impl RatatuiRenderer {
    /// Construct a renderer drawing to the process stdout.
    pub fn new() -> io::Result<Self> {
        let backend = RatatuiCrosstermBackend::new(io::stdout());
        Ok(Self {
            terminal: Terminal::new(backend)?,
        })
    }
}

impl Renderer for RatatuiRenderer {
    fn draw(&mut self, view: &ViewModel) -> io::Result<()> {
        self.terminal
            .draw(|frame| render_view(frame.area(), frame.buffer_mut(), view))?;
        Ok(())
    }
}

/// Render the whole TUI frame into `buf` for `area`, purely from the view-model.
///
/// A pure function of `view`: the same view-model always produces the same buffer
/// (INV-5 render-from-state). Extracted from the renderer so the render path can
/// be exercised against an off-screen [`Buffer`] without a terminal.
pub fn render_view(area: Rect, buf: &mut Buffer, view: &ViewModel) {
    let areas = view::layout(area);
    let snapshot = &view.snapshot;

    view::render_board(
        areas.board,
        buf,
        &snapshot.board,
        view.focus == Focus::Board,
    );
    view::render_office(
        areas.office,
        buf,
        &snapshot.office,
        view.focus == Focus::Office,
    );
    view::render_status(
        areas.status,
        buf,
        &snapshot.version,
        &snapshot.status,
        view.focus == Focus::Status,
    );
    view::render_sessions(
        areas.sessions,
        buf,
        &snapshot.sessions,
        view.focus == Focus::Sessions,
    );

    let bar_text = match &view.command_feedback {
        Some(feedback) if view.command_input.is_empty() => format!("/  {feedback}"),
        _ => format!("/{}", view.command_input),
    };
    Paragraph::new(bar_text)
        .style(view::focus_border_style(view.focus == Focus::CommandBar))
        .render(areas.command_bar, buf);
}

/// Run the TUI against the real terminal and the live engine.
pub fn run() -> io::Result<()> {
    // The same public composition door every surface uses
    // (`invocable_bootstrap::bootstrap`) — no private construction path.
    // Secret values to mask are loaded from the core secrets store once that
    // binding lands; the dispatcher is left with none for now, the same
    // disclosed residual the sibling CLI frontend's own composition carries.
    let (mut registry, mut dispatcher) =
        cronus_core::invocable_bootstrap::bootstrap(cronus_core::Engine::new());
    // This surface's own actions register through the same door, before the
    // catalog is built from the result — so `/pane` is a real discoverable
    // group, not a parallel mechanism the catalog never learns about.
    pane_actions::register(&mut registry, &mut dispatcher);
    let invocables: Vec<&Invocable> = registry.all().collect();
    let catalog = command::build_catalog(&invocables);

    run_with(
        CrosstermBackend::new(),
        CapabilitySource::new(Engine::new()),
        RatatuiRenderer::new()?,
        registry,
        dispatcher,
        catalog,
    )
}

/// Run the loop against injected backend / source / renderer / registry /
/// dispatcher.
///
/// The terminal lifecycle is RAII-guarded by [`Tui`], so any exit path — normal,
/// `?` error, or panic — restores the terminal.
pub fn run_with<B, S, R>(
    backend: B,
    mut source: S,
    mut renderer: R,
    registry: InvocableRegistry,
    dispatcher: Dispatcher,
    catalog: Vec<CommandSpec>,
) -> io::Result<()>
where
    B: TerminalBackend,
    S: SnapshotSource,
    R: Renderer,
{
    let mut tui = Tui::new(backend)?;
    let (cols, rows) = tui.size()?;
    let mut app = App::new(
        ViewModel {
            size: (cols, rows),
            ..Default::default()
        },
        registry,
        dispatcher,
        catalog,
    );

    // Initial frame so the screen is populated before the first event.
    renderer.draw(app.view())?;

    loop {
        let mut events = Vec::new();
        if let Some(event) = tui.poll_event(TICK)? {
            events.push(event);
        }
        let result = app.tick(&events, &mut source, &mut renderer)?;
        if result.quit {
            break;
        }
    }

    tui.restore()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Arc;

    use cronus_contract::{
        ArgValue, Binder, BinderKind, InvocableId, Locus, Outcome, OutcomeValue, Stability,
    };
    use cronus_core::invocable::Registrant;

    /// A scripted [`SnapshotSource`]: returns each queued value in turn, so tests
    /// can model "new snapshot ready" (`Some`) and "slow call in flight" (`None`).
    struct ScriptedSource {
        queue: VecDeque<Option<CoreSnapshot>>,
    }

    impl ScriptedSource {
        fn new(script: Vec<Option<CoreSnapshot>>) -> Self {
            Self {
                queue: script.into(),
            }
        }
    }

    impl SnapshotSource for ScriptedSource {
        fn poll_snapshot(&mut self) -> Option<CoreSnapshot> {
            self.queue.pop_front().flatten()
        }
    }

    /// Records every frame drawn so tests can count redraws and inspect content.
    #[derive(Default)]
    struct RecordingRenderer {
        frames: Vec<ViewModel>,
    }

    impl Renderer for RecordingRenderer {
        fn draw(&mut self, view: &ViewModel) -> io::Result<()> {
            self.frames.push(view.clone());
            Ok(())
        }
    }

    fn snap(status: &str) -> CoreSnapshot {
        CoreSnapshot {
            version: "0.1.0".to_string(),
            status: status.to_string(),
            ..CoreSnapshot::default()
        }
    }

    /// An empty registry/dispatcher pair — a valid, inert choice for tests
    /// where dispatch behavior is irrelevant: any slash command resolves to
    /// `Dispatched::Unknown`, exactly as a genuinely unbound verb would.
    fn empty_registry_and_dispatcher() -> (InvocableRegistry, Dispatcher) {
        (InvocableRegistry::new(), Dispatcher::new())
    }

    /// The real pane-action registration production `run()` performs — used
    /// by every test that drives Tab/BackTab/Esc and needs the real dispatch
    /// path behind them to actually resolve, not merely the key press.
    fn registry_with_pane_actions() -> (InvocableRegistry, Dispatcher) {
        let mut registry = InvocableRegistry::new();
        let mut dispatcher = Dispatcher::new();
        pane_actions::register(&mut registry, &mut dispatcher);
        (registry, dispatcher)
    }

    /// A minimal catalog naming `test` as a known verb — used by tests
    /// exercising dispatch *wiring* end to end (does a recognized command
    /// reach the registry, bind its args, and dispatch for real), not by
    /// anything asserting on catalog derivation itself (see `command.rs`'s
    /// own `build_catalog` tests for that).
    fn test_catalog() -> Vec<CommandSpec> {
        vec![CommandSpec {
            name: "test",
            summary: "test-only catalog entry".to_string(),
        }]
    }

    /// A real, test-registered `core:test.probe` invocable, one required
    /// `Text` binder, echoing it back as `probe: {value}` — proves dispatch
    /// wiring end to end without depending on any real domain subsystem.
    fn registry_with_probe() -> (InvocableRegistry, Dispatcher) {
        let mut registry = InvocableRegistry::new();
        let mut dispatcher = Dispatcher::new();
        let id = InvocableId::new("core:test.probe").expect("well-formed test id");
        registry
            .register(
                &Registrant::core(),
                Invocable {
                    id: id.clone(),
                    name: "Probe",
                    summary: "Test-only probe invocable.",
                    group: "test",
                    locus: Locus::Semantic,
                    binders: vec![Binder {
                        name: "value",
                        kind: BinderKind::Text,
                        optional: false,
                    }],
                    stability: Stability::Shipped,
                    journal_raw_input: true,
                },
            )
            .expect("test fixture registers cleanly");
        dispatcher.attach(
            id,
            Arc::new(|args| {
                let echoed = match args.get("value") {
                    Some(ArgValue::Text(s)) => s.clone(),
                    _ => String::new(),
                };
                Outcome::Value(OutcomeValue::Text(format!("probe: {echoed}")))
            }),
        );
        (registry, dispatcher)
    }

    /// The same probe invocable, taking no arguments and echoing back a
    /// value containing `secret` in its raw, unmasked output — the
    /// dispatcher's own secret list (set via `set_secrets`) is what should
    /// mask it, never anything in this crate.
    fn registry_with_secret_probe(secret: &str) -> (InvocableRegistry, Dispatcher) {
        let mut registry = InvocableRegistry::new();
        let mut dispatcher = Dispatcher::new();
        let id = InvocableId::new("core:test.probe").expect("well-formed test id");
        registry
            .register(
                &Registrant::core(),
                Invocable {
                    id: id.clone(),
                    name: "Probe",
                    summary: "Test-only probe invocable.",
                    group: "test",
                    locus: Locus::Semantic,
                    binders: Vec::new(),
                    stability: Stability::Shipped,
                    journal_raw_input: true,
                },
            )
            .expect("test fixture registers cleanly");
        let secret_owned = secret.to_string();
        dispatcher.attach(
            id,
            Arc::new(move |_args| {
                Outcome::Value(OutcomeValue::Text(format!("key={secret_owned}")))
            }),
        );
        dispatcher.set_secrets(vec![secret.to_string()]);
        (registry, dispatcher)
    }

    /// The same probe identity, declaring one required binder but never
    /// attaching a handler for it — dispatch always rejects before any
    /// handler could run, letting a test drive a real `Rejected` outcome
    /// end to end through the app's tick loop.
    fn registry_with_required_binder() -> (InvocableRegistry, Dispatcher) {
        let mut registry = InvocableRegistry::new();
        let dispatcher = Dispatcher::new();
        let id = InvocableId::new("core:test.probe").expect("well-formed test id");
        registry
            .register(
                &Registrant::core(),
                Invocable {
                    id,
                    name: "Probe",
                    summary: "Test-only probe invocable.",
                    group: "test",
                    locus: Locus::Semantic,
                    binders: vec![Binder {
                        name: "value",
                        kind: BinderKind::Text,
                        optional: false,
                    }],
                    stability: Stability::Shipped,
                    journal_raw_input: true,
                },
            )
            .expect("test fixture registers cleanly");
        (registry, dispatcher)
    }

    #[test]
    fn render_loop_state_change_schedules_exactly_one_redraw() {
        let (registry, dispatcher) = empty_registry_and_dispatcher();
        let mut app = App::new(ViewModel::default(), registry, dispatcher, Vec::new());
        let mut source = ScriptedSource::new(vec![Some(snap("running"))]);
        let mut renderer = RecordingRenderer::default();

        let result = app
            .tick(&[], &mut source, &mut renderer)
            .expect("tick succeeds");

        assert!(result.redrawn, "a state change must redraw");
        assert_eq!(renderer.frames.len(), 1, "exactly one redraw per change");
        assert_eq!(app.view().snapshot, snap("running"), "view-model updated");
    }

    #[test]
    fn render_loop_unchanged_snapshot_does_not_redraw() {
        let (registry, dispatcher) = empty_registry_and_dispatcher();
        let mut app = App::new(ViewModel::default(), registry, dispatcher, Vec::new());
        // Same snapshot delivered twice across two ticks.
        let mut source = ScriptedSource::new(vec![Some(snap("idle")), Some(snap("idle"))]);
        let mut renderer = RecordingRenderer::default();

        let first = app.tick(&[], &mut source, &mut renderer).unwrap();
        let second = app.tick(&[], &mut source, &mut renderer).unwrap();

        assert!(first.redrawn, "first snapshot is a change");
        assert!(!second.redrawn, "identical snapshot must not redraw");
        assert_eq!(renderer.frames.len(), 1, "no redundant frame");
    }

    #[test]
    fn render_loop_stays_responsive_while_core_call_is_slow() {
        let (registry, dispatcher) = registry_with_pane_actions();
        let mut app = App::new(ViewModel::default(), registry, dispatcher, Vec::new());
        // `None` models a snapshot still being produced — a slow core call.
        let mut source = ScriptedSource::new(vec![None]);
        let mut renderer = RecordingRenderer::default();

        // An Esc keypress arrives during the slow call.
        let result = app
            .tick(&[TermEvent::Key(Key::Esc)], &mut source, &mut renderer)
            .expect("tick succeeds");

        assert!(
            result.quit,
            "input must be handled even when no snapshot is ready"
        );
    }

    /// The other half of the same criterion pane dispatch introduces: with
    /// no pane actions registered at all, Esc must NOT quit — there is no
    /// second, direct key-code-to-behaviour path that could run once the
    /// real dispatch answers `Dispatched::Unknown`.
    #[test]
    fn an_unregistered_quit_action_never_quits_the_app() {
        let (registry, dispatcher) = empty_registry_and_dispatcher();
        let mut app = App::new(ViewModel::default(), registry, dispatcher, Vec::new());
        let mut source = ScriptedSource::new(vec![None]);
        let mut renderer = RecordingRenderer::default();

        let result = app
            .tick(&[TermEvent::Key(Key::Esc)], &mut source, &mut renderer)
            .expect("tick succeeds");

        assert!(
            !result.quit,
            "quit must go through a real dispatch, never a hardcoded key match"
        );
    }

    #[test]
    fn render_loop_resize_updates_view_and_redraws_without_snapshot() {
        let (registry, dispatcher) = empty_registry_and_dispatcher();
        let mut app = App::new(ViewModel::default(), registry, dispatcher, Vec::new());
        let mut source = ScriptedSource::new(vec![None]);
        let mut renderer = RecordingRenderer::default();

        let result = app
            .tick(&[TermEvent::Resize(120, 40)], &mut source, &mut renderer)
            .unwrap();

        assert!(result.redrawn, "resize requests a redraw");
        assert_eq!(app.view().size, (120, 40), "view-model tracks new size");
        assert_eq!(renderer.frames.len(), 1);
    }

    #[test]
    fn layout_focus_tab_key_advances_focus_and_redraws() {
        let (registry, dispatcher) = registry_with_pane_actions();
        let mut app = App::new(ViewModel::default(), registry, dispatcher, Vec::new());
        let mut source = ScriptedSource::new(vec![None]);
        let mut renderer = RecordingRenderer::default();

        assert_eq!(app.view().focus, Focus::Board, "default focus is the board");

        let forward = app
            .tick(&[TermEvent::Key(Key::Tab)], &mut source, &mut renderer)
            .unwrap();
        assert!(forward.redrawn, "focus change requests a redraw");
        assert_eq!(app.view().focus, Focus::Office, "Tab advances focus");

        let mut source2 = ScriptedSource::new(vec![None]);
        app.tick(&[TermEvent::Key(Key::BackTab)], &mut source2, &mut renderer)
            .unwrap();
        assert_eq!(app.view().focus, Focus::Board, "Shift+Tab steps focus back");
    }

    /// The registry, not the key match, decides whether focus moves at all:
    /// with no pane actions registered, Tab must leave focus exactly where
    /// it was and request no redraw — proving `handle_key` really resolves
    /// through dispatch rather than mutating `view.focus` unconditionally.
    #[test]
    fn an_unregistered_focus_action_never_moves_focus() {
        let (registry, dispatcher) = empty_registry_and_dispatcher();
        let mut app = App::new(ViewModel::default(), registry, dispatcher, Vec::new());
        let mut source = ScriptedSource::new(vec![None]);
        let mut renderer = RecordingRenderer::default();

        let result = app
            .tick(&[TermEvent::Key(Key::Tab)], &mut source, &mut renderer)
            .unwrap();

        assert!(!result.redrawn, "no dispatch ran, so nothing changed");
        assert_eq!(app.view().focus, Focus::Board, "focus must not move");
    }

    /// Types `/test probe hello`, submits it, and asserts the feedback is
    /// exactly the real, registered invocable's own `Outcome` rendered —
    /// end to end through the app's tick loop, the real registry, and the
    /// real dispatcher, not a stub.
    #[test]
    fn command_parse_bar_typing_then_enter_dispatches_a_real_registered_invocable() {
        let (registry, dispatcher) = registry_with_probe();
        let mut app = App::new(
            ViewModel {
                focus: Focus::CommandBar,
                ..Default::default()
            },
            registry,
            dispatcher,
            test_catalog(),
        );
        let mut source = ScriptedSource::new(vec![]);
        let mut renderer = RecordingRenderer::default();

        for c in "test probe hello".chars() {
            app.tick(&[TermEvent::Key(Key::Char(c))], &mut source, &mut renderer)
                .unwrap();
        }
        assert_eq!(app.view().command_input, "test probe hello");

        app.tick(&[TermEvent::Key(Key::Enter)], &mut source, &mut renderer)
            .unwrap();
        assert_eq!(app.view().command_input, "", "input clears on submit");
        assert_eq!(
            app.view().command_feedback.as_deref(),
            Some("probe: hello"),
            "feedback is exactly the dispatched invocable's own rendered Outcome"
        );
    }

    #[test]
    fn command_parse_bar_unknown_errors_and_esc_cancels_without_quitting() {
        let (registry, dispatcher) = empty_registry_and_dispatcher();
        let mut app = App::new(
            ViewModel {
                focus: Focus::CommandBar,
                ..Default::default()
            },
            registry,
            dispatcher,
            Vec::new(),
        );
        let mut source = ScriptedSource::new(vec![]);
        let mut renderer = RecordingRenderer::default();

        for c in "xyz".chars() {
            app.tick(&[TermEvent::Key(Key::Char(c))], &mut source, &mut renderer)
                .unwrap();
        }
        app.tick(&[TermEvent::Key(Key::Enter)], &mut source, &mut renderer)
            .unwrap();
        assert!(
            app.view()
                .command_feedback
                .as_deref()
                .unwrap()
                .contains("unknown command"),
            "an unrecognized verb yields an inline error"
        );

        let result = app
            .tick(&[TermEvent::Key(Key::Esc)], &mut source, &mut renderer)
            .unwrap();
        assert!(!result.quit, "Esc cancels in the command bar, never quits");
        assert_eq!(app.view().command_feedback, None);
    }

    #[test]
    fn render_loop_holds_no_state_beyond_the_snapshot() {
        // Determinism proves the absence of hidden domain state: two apps fed the
        // same event/snapshot script end in identical view-models.
        let script = || ScriptedSource::new(vec![Some(snap("a")), Some(snap("b"))]);

        let (r1, d1) = empty_registry_and_dispatcher();
        let mut app1 = App::new(ViewModel::default(), r1, d1, Vec::new());
        let mut r1 = RecordingRenderer::default();
        let mut s1 = script();
        app1.tick(&[], &mut s1, &mut r1).unwrap();
        app1.tick(&[], &mut s1, &mut r1).unwrap();

        let (r2, d2) = empty_registry_and_dispatcher();
        let mut app2 = App::new(ViewModel::default(), r2, d2, Vec::new());
        let mut r2 = RecordingRenderer::default();
        let mut s2 = script();
        app2.tick(&[], &mut s2, &mut r2).unwrap();
        app2.tick(&[], &mut s2, &mut r2).unwrap();

        assert_eq!(
            app1.view(),
            app2.view(),
            "loop is a pure function of its inputs"
        );
        // The view-model is exactly the last snapshot — nothing accumulated.
        assert_eq!(app1.view().snapshot, snap("b"));
    }

    /// Both halves of the same criterion, driven end to end through the
    /// app's tick loop over one real, registered invocable: an unresolved
    /// slash line renders **no** feedback at all — ordinary input, not a
    /// rendered failure (l2-tui v1.2.0) — and a genuinely `Rejected` outcome
    /// (the identity resolves; its one required binder is never supplied)
    /// still renders as a legible refusal. Proven together so the two stay
    /// distinguishable rather than both silently swallowed.
    #[test]
    fn command_dispatch_distinguishes_an_unresolved_line_from_a_genuine_rejection() {
        let (registry, dispatcher) = registry_with_required_binder();
        let mut app = App::new(
            ViewModel {
                focus: Focus::CommandBar,
                ..Default::default()
            },
            registry,
            dispatcher,
            test_catalog(),
        );
        let mut source = ScriptedSource::new(vec![]);
        let mut renderer = RecordingRenderer::default();

        // "test" is a known catalog group, but "frobnicate" resolves to no
        // registered invocable — an unresolved candidate identity, not a
        // failure to render anything about.
        for c in "test frobnicate".chars() {
            app.tick(&[TermEvent::Key(Key::Char(c))], &mut source, &mut renderer)
                .unwrap();
        }
        app.tick(&[TermEvent::Key(Key::Enter)], &mut source, &mut renderer)
            .unwrap();
        assert_eq!(
            app.view().command_feedback,
            None,
            "an unresolved identity must render no feedback at all, not an error string"
        );

        // "test probe" resolves to a real invocable whose one required
        // binder is never supplied — a genuine Rejected outcome, which must
        // still render, distinguishing it from the unresolved case above.
        for c in "test probe".chars() {
            app.tick(&[TermEvent::Key(Key::Char(c))], &mut source, &mut renderer)
                .unwrap();
        }
        app.tick(&[TermEvent::Key(Key::Enter)], &mut source, &mut renderer)
            .unwrap();

        let feedback = app.view().command_feedback.as_deref().unwrap();
        assert!(
            feedback.contains("value"),
            "the offending binder's own name must reach the view: {feedback:?}"
        );
        assert!(
            feedback.contains("Absent"),
            "the rejection mode must reach the view, not be flattened away: {feedback:?}"
        );
    }

    #[test]
    fn command_dispatch_through_the_bar_never_leaks_a_secret_to_the_view() {
        let (registry, dispatcher) = registry_with_secret_probe("sk-LIVE-7");
        let mut app = App::new(
            ViewModel {
                focus: Focus::CommandBar,
                ..Default::default()
            },
            registry,
            dispatcher,
            test_catalog(),
        );
        let mut source = ScriptedSource::new(vec![]);
        let mut renderer = RecordingRenderer::default();

        for c in "test probe".chars() {
            app.tick(&[TermEvent::Key(Key::Char(c))], &mut source, &mut renderer)
                .unwrap();
        }
        app.tick(&[TermEvent::Key(Key::Enter)], &mut source, &mut renderer)
            .unwrap();

        let feedback = app.view().command_feedback.as_deref().unwrap();
        assert!(
            !feedback.contains("sk-LIVE-7"),
            "the secret never reaches the view-model or the screen buffer"
        );
        assert!(feedback.contains("***"), "the leaked value renders masked");
    }

    /// A representative populated view-model for render-from-state tests.
    fn populated_view() -> ViewModel {
        let mut sessions = SessionsView::default();
        sessions.push("agent started");
        ViewModel {
            snapshot: CoreSnapshot {
                version: "1.2.3".to_string(),
                status: "running | 80%".to_string(),
                board: BoardView {
                    cards: vec![view::BoardCard {
                        id: "k1".to_string(),
                        title: String::new(),
                        column: view::BoardColumn::Running,
                    }],
                },
                office: OfficeView {
                    agents: vec![view::AgentActivity {
                        agent: "orchestrator".to_string(),
                        task: "planning".to_string(),
                    }],
                },
                sessions,
            },
            size: (80, 24),
            focus: Focus::Board,
            command_input: String::new(),
            command_feedback: None,
        }
    }

    fn render_to_buffer(view: &ViewModel, area: Rect) -> Buffer {
        let mut buf = Buffer::empty(area);
        render_view(area, &mut buf, view);
        buf
    }

    fn buffer_text(buf: &Buffer, area: Rect) -> String {
        let mut text = String::new();
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                if let Some(cell) = buf.cell((x, y)) {
                    text.push_str(cell.symbol());
                }
            }
        }
        text
    }

    #[test]
    fn render_state_is_a_pure_function_of_the_view_model() {
        let view = populated_view();
        let area = Rect::new(0, 0, 80, 24);

        let first = render_to_buffer(&view, area);
        let second = render_to_buffer(&view, area);

        assert_eq!(
            first, second,
            "the same view-model must render an identical frame (INV-5)"
        );
    }

    #[test]
    fn render_state_differs_when_the_snapshot_changes() {
        let area = Rect::new(0, 0, 80, 24);
        let base = render_to_buffer(&populated_view(), area);

        let mut changed = populated_view();
        changed.snapshot.status = "running | 100%".to_string();
        let after = render_to_buffer(&changed, area);

        assert_ne!(base, after, "a changed snapshot must change the frame");
    }

    #[test]
    fn mask_secrets_dispatched_value_never_reaches_the_buffer() {
        let (registry, dispatcher) = registry_with_secret_probe("sk-LIVE-42");
        let mut app = App::new(
            ViewModel {
                focus: Focus::CommandBar,
                size: (80, 24),
                ..Default::default()
            },
            registry,
            dispatcher,
            test_catalog(),
        );
        let mut source = ScriptedSource::new(vec![]);
        let mut renderer = RecordingRenderer::default();

        for c in "test probe".chars() {
            app.tick(&[TermEvent::Key(Key::Char(c))], &mut source, &mut renderer)
                .unwrap();
        }
        app.tick(&[TermEvent::Key(Key::Enter)], &mut source, &mut renderer)
            .unwrap();

        let area = Rect::new(0, 0, 80, 24);
        let buf = render_to_buffer(app.view(), area);
        let rendered = buffer_text(&buf, area);

        assert!(
            !rendered.contains("sk-LIVE-42"),
            "no secret value reaches the rendered screen buffer (INV-7)"
        );
        assert!(rendered.contains("***"), "the secret renders masked");
    }
}
