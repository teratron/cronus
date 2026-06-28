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

use std::io::{self, Write};
use std::time::Duration;

use crate::terminal::{CrosstermBackend, Key, TermEvent, TerminalBackend, Tui};
use cronus::{Capabilities, Engine};

/// How long the loop blocks for input before ticking again. Bounds redraw
/// latency for state changes that arrive without a terminal event.
const TICK: Duration = Duration::from_millis(50);

/// An immutable projection of durable core state at one instant.
///
/// The TUI renders from this and never mutates it (INV-5). Today the core's
/// capability surface is version + status; richer projections (board, sessions)
/// extend this struct as those capabilities land.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreSnapshot {
    /// Engine/product version string.
    pub version: String,
    /// Human-readable status line.
    pub status: String,
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
                // Provisional quit key. Input routing (a focused command bar that
                // reclaims Esc as cancel) is refined when the command bar lands.
                TermEvent::Key(Key::Esc) => {
                    self.should_quit = true;
                }
                TermEvent::Key(_) => {}
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
        })
    }
}

/// Minimal production [`Renderer`]: clears the screen and prints the status line
/// and size. The full panel layout replaces this in the view-panel tracks; this
/// keeps the binary a working interactive surface in the meantime.
pub struct PlainRenderer<W: Write> {
    out: W,
}

impl<W: Write> PlainRenderer<W> {
    /// Construct a renderer writing frames to `out`.
    pub fn new(out: W) -> Self {
        Self { out }
    }
}

impl<W: Write> Renderer for PlainRenderer<W> {
    fn draw(&mut self, view: &ViewModel) -> io::Result<()> {
        use crossterm::{cursor, queue, terminal};

        queue!(
            self.out,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        write!(
            self.out,
            "Cronus TUI {} — {}",
            view.snapshot.version, view.snapshot.status
        )?;
        queue!(self.out, cursor::MoveTo(0, 1))?;
        write!(self.out, "[{}x{}]  Esc to quit", view.size.0, view.size.1)?;
        self.out.flush()
    }
}

/// Run the TUI against the real terminal and the live engine.
pub fn run() -> io::Result<()> {
    let engine = Engine::new();
    run_with(
        CrosstermBackend::new(),
        CapabilitySource::new(engine),
        PlainRenderer::new(io::stdout()),
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
