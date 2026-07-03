//! Overlay window geometry.
//!
//! The overlay is a small always-on-top strip that must not steal keyboard
//! focus. This module owns the pure geometry: fixed size, per-OS vertical
//! clearance (menu bar / taskbar), and the Linux escape hatch that disables
//! the GTK layer shell. The window backend itself is per-OS (NSPanel /
//! layer shell / borderless topmost) and binds where the window is created.

use crate::settings::OverlayPosition;

/// Fixed overlay size — constants in code by design.
pub const OVERLAY_WIDTH: u32 = 420;
pub const OVERLAY_HEIGHT: u32 = 56;

/// Vertical clearance from the docked screen edge, per OS: macOS menu bar,
/// Windows taskbar, Linux compositor panels.
pub fn vertical_offset() -> u32 {
    #[cfg(target_os = "macos")]
    {
        28
    }
    #[cfg(target_os = "windows")]
    {
        48
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        32
    }
}

/// Computed overlay placement on a screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlayRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Map a position to concrete geometry; `None` means no overlay window.
pub fn overlay_rect(
    position: OverlayPosition,
    screen_width: u32,
    screen_height: u32,
) -> Option<OverlayRect> {
    let x = screen_width.saturating_sub(OVERLAY_WIDTH) / 2;
    match position {
        OverlayPosition::None => None,
        OverlayPosition::Top => Some(OverlayRect {
            x,
            y: vertical_offset(),
            width: OVERLAY_WIDTH,
            height: OVERLAY_HEIGHT,
        }),
        OverlayPosition::Bottom => Some(OverlayRect {
            x,
            y: screen_height
                .saturating_sub(OVERLAY_HEIGHT)
                .saturating_sub(vertical_offset()),
            width: OVERLAY_WIDTH,
            height: OVERLAY_HEIGHT,
        }),
    }
}

/// Linux escape hatch: `APP_NO_GTK_LAYER_SHELL=1` falls back to a borderless
/// normal window on compositors where the layer shell misbehaves.
pub fn gtk_layer_shell_disabled(env_value: Option<&str>) -> bool {
    matches!(env_value, Some("1"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_position_creates_no_overlay() {
        assert_eq!(overlay_rect(OverlayPosition::None, 1920, 1080), None);
    }

    #[test]
    fn top_and_bottom_positions_dock_with_per_os_clearance() {
        let top = overlay_rect(OverlayPosition::Top, 1920, 1080).expect("top rect");
        assert_eq!(top.y, vertical_offset());
        assert_eq!(top.x, (1920 - OVERLAY_WIDTH) / 2, "horizontally centered");

        let bottom = overlay_rect(OverlayPosition::Bottom, 1920, 1080).expect("bottom rect");
        assert_eq!(bottom.y, 1080 - OVERLAY_HEIGHT - vertical_offset());
    }

    #[test]
    fn tiny_screens_never_underflow() {
        let rect = overlay_rect(OverlayPosition::Bottom, 100, 40).expect("rect");
        assert_eq!(rect.y, 0, "saturating math instead of a panic");
    }

    #[test]
    fn escape_hatch_only_engages_on_exact_flag() {
        assert!(gtk_layer_shell_disabled(Some("1")));
        assert!(!gtk_layer_shell_disabled(Some("0")));
        assert!(!gtk_layer_shell_disabled(None));
    }
}
