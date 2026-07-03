//! Single-instance enforcement and runtime-only CLI flags.
//!
//! Exactly one process may run: a second launch forwards its parsed CLI
//! arguments to the primary over the instance channel and exits. Runtime-only
//! flags steer the running instance and are never written to settings. Every
//! external trigger — shortcut, OS signal, forwarded CLI — funnels through
//! one dispatch entry point so state logic exists once.

/// Runtime-only flags parsed from CLI arguments. Never persisted.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RuntimeArgs {
    pub toggles: Vec<String>,
    pub cancel: bool,
    pub start_hidden: bool,
    pub debug: bool,
    pub no_tray: bool,
}

impl RuntimeArgs {
    /// Parse recognized runtime-only flags; unknown arguments are ignored
    /// (they belong to the platform or future versions, not to settings).
    pub fn parse<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut parsed = Self::default();
        for arg in args {
            let arg = arg.as_ref();
            if let Some(feature) = arg.strip_prefix("--toggle-") {
                if !feature.is_empty() {
                    parsed.toggles.push(feature.to_string());
                }
                continue;
            }
            match arg {
                "--cancel" => parsed.cancel = true,
                "--start-hidden" => parsed.start_hidden = true,
                "--debug" => parsed.debug = true,
                "--no-tray" => parsed.no_tray = true,
                _ => {}
            }
        }
        parsed
    }
}

/// The OS lock / named IPC channel used to detect a running instance.
/// Production adapts the single-instance plugin; tests use a mock.
pub trait InstanceChannel {
    /// Try to become the primary instance.
    fn try_acquire(&mut self) -> bool;
    /// Forward CLI arguments to the running primary.
    fn forward(&mut self, args: &RuntimeArgs) -> Result<(), String>;
}

/// What a process launch resolved to.
#[derive(Debug, PartialEq, Eq)]
pub enum LaunchOutcome {
    /// This process is the primary; proceed with startup.
    Primary,
    /// A primary already runs; arguments were handed off — exit now.
    ForwardedToPrimary,
    /// A primary runs but the hand-off failed; exit with the reason.
    HandoffFailed(String),
}

/// Enforce single instance: acquire or forward-and-exit.
pub fn launch(channel: &mut dyn InstanceChannel, args: &RuntimeArgs) -> LaunchOutcome {
    if channel.try_acquire() {
        return LaunchOutcome::Primary;
    }
    match channel.forward(args) {
        Ok(()) => LaunchOutcome::ForwardedToPrimary,
        Err(reason) => LaunchOutcome::HandoffFailed(reason),
    }
}

/// Every external trigger source, normalized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trigger {
    /// A global shortcut fired (by binding id).
    Shortcut(String),
    /// A Unix signal handler fired (by signal name).
    Signal(String),
    /// CLI arguments arrived — locally or forwarded by a second instance.
    Cli(RuntimeArgs),
}

/// The actions the application state coordinator understands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    ToggleFeature(String),
    CancelOperation,
    ShowMainWindow,
}

/// The single dispatch entry point: normalize any trigger into coordinator
/// actions. No trigger source carries its own state logic.
pub fn dispatch(trigger: &Trigger) -> Vec<Action> {
    match trigger {
        Trigger::Shortcut(id) => match id.as_str() {
            "cancel" => vec![Action::CancelOperation],
            "show-main-window" => vec![Action::ShowMainWindow],
            other => vec![Action::ToggleFeature(other.to_string())],
        },
        Trigger::Signal(name) => match name.as_str() {
            "SIGUSR2" => vec![Action::CancelOperation],
            _ => vec![Action::ToggleFeature("primary".into())],
        },
        Trigger::Cli(args) => {
            let mut actions: Vec<Action> = args
                .toggles
                .iter()
                .map(|feature| Action::ToggleFeature(feature.clone()))
                .collect();
            if args.cancel {
                actions.push(Action::CancelOperation);
            }
            actions
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockChannel {
        primary_running: bool,
        forwarded: Vec<RuntimeArgs>,
        forward_fails: bool,
    }

    impl InstanceChannel for MockChannel {
        fn try_acquire(&mut self) -> bool {
            !self.primary_running
        }
        fn forward(&mut self, args: &RuntimeArgs) -> Result<(), String> {
            if self.forward_fails {
                return Err("channel closed".into());
            }
            self.forwarded.push(args.clone());
            Ok(())
        }
    }

    #[test]
    fn runtime_flags_parse_and_unknown_arguments_are_ignored() {
        let args = RuntimeArgs::parse([
            "--toggle-recording",
            "--cancel",
            "--start-hidden",
            "--debug",
            "--no-tray",
            "--future-flag",
        ]);
        assert_eq!(args.toggles, vec!["recording".to_string()]);
        assert!(args.cancel && args.start_hidden && args.debug && args.no_tray);
    }

    #[test]
    fn first_launch_becomes_primary() {
        let mut channel = MockChannel {
            primary_running: false,
            forwarded: Vec::new(),
            forward_fails: false,
        };
        assert_eq!(
            launch(&mut channel, &RuntimeArgs::default()),
            LaunchOutcome::Primary
        );
    }

    #[test]
    fn second_instance_forwards_its_arguments_and_exits() {
        let mut channel = MockChannel {
            primary_running: true,
            forwarded: Vec::new(),
            forward_fails: false,
        };
        let args = RuntimeArgs::parse(["--toggle-recording", "--cancel"]);

        assert_eq!(
            launch(&mut channel, &args),
            LaunchOutcome::ForwardedToPrimary
        );
        assert_eq!(channel.forwarded, vec![args.clone()]);

        // The primary processes the forwarded arguments exactly as if typed.
        let actions = dispatch(&Trigger::Cli(channel.forwarded[0].clone()));
        assert_eq!(
            actions,
            vec![
                Action::ToggleFeature("recording".into()),
                Action::CancelOperation
            ]
        );
    }

    #[test]
    fn failed_handoff_is_surfaced_not_swallowed() {
        let mut channel = MockChannel {
            primary_running: true,
            forwarded: Vec::new(),
            forward_fails: true,
        };
        assert_eq!(
            launch(&mut channel, &RuntimeArgs::default()),
            LaunchOutcome::HandoffFailed("channel closed".into())
        );
    }

    #[test]
    fn all_trigger_sources_funnel_through_one_dispatch() {
        assert_eq!(
            dispatch(&Trigger::Shortcut("cancel".into())),
            vec![Action::CancelOperation]
        );
        assert_eq!(
            dispatch(&Trigger::Shortcut("show-main-window".into())),
            vec![Action::ShowMainWindow]
        );
        assert_eq!(
            dispatch(&Trigger::Signal("SIGUSR2".into())),
            vec![Action::CancelOperation]
        );
    }
}
