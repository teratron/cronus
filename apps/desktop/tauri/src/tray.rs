//! Tray icon state machine.
//!
//! The tray reflects operation state × active theme. All nine icon variants
//! are resolved up front so a state transition never does file I/O, and the
//! context menu is rebuilt per state so destructive items (Cancel) exist only
//! while an operation is actually running.

/// What the application is currently doing, as the tray reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationState {
    Idle,
    Active,
    Processing,
}

/// The visual theme the tray icon matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppTheme {
    Dark,
    Light,
    Colored,
}

impl OperationState {
    pub const ALL: [Self; 3] = [Self::Idle, Self::Active, Self::Processing];

    fn key(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Active => "active",
            Self::Processing => "processing",
        }
    }
}

impl AppTheme {
    pub const ALL: [Self; 3] = [Self::Dark, Self::Light, Self::Colored];

    fn key(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Colored => "colored",
        }
    }
}

/// The pre-loaded icon matrix: one asset identifier per State × Theme.
#[derive(Debug)]
pub struct TrayIcons {
    variants: [(OperationState, AppTheme, String); 9],
}

impl TrayIcons {
    /// Resolve all nine variants at startup (no I/O on transitions).
    pub fn preload() -> Self {
        let mut variants: Vec<(OperationState, AppTheme, String)> = Vec::with_capacity(9);
        for state in OperationState::ALL {
            for theme in AppTheme::ALL {
                variants.push((
                    state,
                    theme,
                    format!("tray-{}-{}", state.key(), theme.key()),
                ));
            }
        }
        // Vec is exactly 9 by construction of the two ALL arrays.
        let variants: [(OperationState, AppTheme, String); 9] = match variants.try_into() {
            Ok(v) => v,
            Err(_) => unreachable!("3 states x 3 themes always yields 9 variants"),
        };
        Self { variants }
    }

    /// The icon asset for a state/theme pair.
    pub fn icon(&self, state: OperationState, theme: AppTheme) -> &str {
        // The matrix is total, so the lookup always finds a variant.
        self.variants
            .iter()
            .find(|(s, t, _)| *s == state && *t == theme)
            .map(|(_, _, id)| id.as_str())
            .unwrap_or("tray-idle-dark")
    }
}

/// One entry of the state-dependent context menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItem {
    ShowMainWindow,
    CopyLastResult,
    Cancel,
    Quit,
}

/// Rebuild the tray menu for a state: Cancel exists only while an operation
/// is in flight — a no-op destructive item is never shown.
pub fn menu_for(state: OperationState) -> Vec<MenuItem> {
    let mut items = vec![MenuItem::ShowMainWindow, MenuItem::CopyLastResult];
    if state != OperationState::Idle {
        items.push(MenuItem::Cancel);
    }
    items.push(MenuItem::Quit);
    items
}

/// Copy-last-result fallback chain: prefer the post-processed text, fall back
/// to the raw output when post-processing did not run.
pub fn copy_last_result(post_processed: Option<&str>, raw_output: Option<&str>) -> Option<String> {
    post_processed.or(raw_output).map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preloads_all_nine_state_theme_variants_distinctly() {
        let icons = TrayIcons::preload();
        let mut seen = std::collections::BTreeSet::new();
        for state in OperationState::ALL {
            for theme in AppTheme::ALL {
                seen.insert(icons.icon(state, theme).to_string());
            }
        }
        assert_eq!(seen.len(), 9, "every State x Theme pair has its own icon");
    }

    #[test]
    fn cancel_is_present_only_while_an_operation_runs() {
        assert!(!menu_for(OperationState::Idle).contains(&MenuItem::Cancel));
        assert!(menu_for(OperationState::Active).contains(&MenuItem::Cancel));
        assert!(menu_for(OperationState::Processing).contains(&MenuItem::Cancel));
    }

    #[test]
    fn copy_last_result_prefers_post_processed_and_falls_back_to_raw() {
        assert_eq!(
            copy_last_result(Some("refined"), Some("raw")).as_deref(),
            Some("refined")
        );
        assert_eq!(copy_last_result(None, Some("raw")).as_deref(), Some("raw"));
        assert_eq!(copy_last_result(None, None), None);
    }
}
