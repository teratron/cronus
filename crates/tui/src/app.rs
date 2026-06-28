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

use std::io::{self, Stdout};
use std::time::Duration;

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend as RatatuiCrosstermBackend;
use ratatui::widgets::{Paragraph, Widget};

use crate::command::{self, CommandOutcome};
use crate::terminal::{CrosstermBackend, Key, TermEvent, TerminalBackend, Tui};
use crate::view::{self, BoardView, Focus, OfficeView, SessionsView};
use cronus::{Capabilities, Engine};

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

/// The render loop's state: a view-model snapshot plus a quit flag. Nothing else.
#[derive(Debug, Clone, Default)]
pub struct App {
    view: ViewModel,
    should_quit: bool,
}

impl App {
    /// Construct an app seeded with an initial view-model (typically just the
    /// terminal size; the first snapshot arrives on the first tick).
    pub fn new(initial: ViewModel) -> Self {
        Self {
            view: initial,
            should_quit: false,
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
    fn handle_key(&mut self, key: Key) -> bool {
        match key {
            Key::Tab => {
                self.view.focus = self.view.focus.next();
                true
            }
            Key::BackTab => {
                self.view.focus = self.view.focus.prev();
                true
            }
            _ if self.view.focus == Focus::CommandBar => self.handle_command_key(key),
            Key::Esc => {
                self.should_quit = true;
                false
            }
            _ => false,
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

    /// Classify the current command line and record inline feedback. Recognized
    /// commands are only acknowledged here — dispatch to the core is the next
    /// step's job.
    fn submit_command(&mut self) {
        let line = format!("/{}", self.view.command_input);
        self.view.command_feedback = Some(match command::classify(&line) {
            CommandOutcome::Help => {
                format!(
                    "commands: {}",
                    command::names().collect::<Vec<_>>().join(" ")
                )
            }
            CommandOutcome::Run(cmd) => format!("ok: /{}", cmd.verb),
            CommandOutcome::Error(message) => message,
        });
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
        self.terminal.draw(|frame| {
            let areas = view::layout(frame.area());
            let snapshot = &view.snapshot;
            let buf = frame.buffer_mut();

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
        })?;
        Ok(())
    }
}

/// Run the TUI against the real terminal and the live engine.
pub fn run() -> io::Result<()> {
    let engine = Engine::new();
    run_with(
        CrosstermBackend::new(),
        CapabilitySource::new(engine),
        RatatuiRenderer::new()?,
    )
}

/// Run the loop against injected backend / source / renderer.
///
/// The terminal lifecycle is RAII-guarded by [`Tui`], so any exit path — normal,
/// `?` error, or panic — restores the terminal.
pub fn run_with<B, S, R>(backend: B, mut source: S, mut renderer: R) -> io::Result<()>
where
    B: TerminalBackend,
    S: SnapshotSource,
    R: Renderer,
{
    let mut tui = Tui::new(backend)?;
    let (cols, rows) = tui.size()?;
    let mut app = App::new(ViewModel {
        size: (cols, rows),
        ..Default::default()
    });

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

    #[test]
    fn render_loop_state_change_schedules_exactly_one_redraw() {
        let mut app = App::new(ViewModel::default());
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
        let mut app = App::new(ViewModel::default());
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
        let mut app = App::new(ViewModel::default());
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

    #[test]
    fn render_loop_resize_updates_view_and_redraws_without_snapshot() {
        let mut app = App::new(ViewModel::default());
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
        let mut app = App::new(ViewModel::default());
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

    #[test]
    fn command_parse_bar_typing_then_enter_acknowledges_known_command() {
        let mut app = App::new(ViewModel {
            focus: Focus::CommandBar,
            ..Default::default()
        });
        let mut source = ScriptedSource::new(vec![]);
        let mut renderer = RecordingRenderer::default();

        for c in "status".chars() {
            app.tick(&[TermEvent::Key(Key::Char(c))], &mut source, &mut renderer)
                .unwrap();
        }
        assert_eq!(app.view().command_input, "status");

        app.tick(&[TermEvent::Key(Key::Enter)], &mut source, &mut renderer)
            .unwrap();
        assert_eq!(app.view().command_input, "", "input clears on submit");
        assert_eq!(app.view().command_feedback.as_deref(), Some("ok: /status"));
    }

    #[test]
    fn command_parse_bar_unknown_errors_and_esc_cancels_without_quitting() {
        let mut app = App::new(ViewModel {
            focus: Focus::CommandBar,
            ..Default::default()
        });
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

        let mut app1 = App::new(ViewModel::default());
        let mut r1 = RecordingRenderer::default();
        let mut s1 = script();
        app1.tick(&[], &mut s1, &mut r1).unwrap();
        app1.tick(&[], &mut s1, &mut r1).unwrap();

        let mut app2 = App::new(ViewModel::default());
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
}
