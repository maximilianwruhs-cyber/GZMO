//! Shared TUI palette — sovereign console, not companion chic.

use ratatui::style::Color;

use super::accessibility::AccessibilityFlags;

pub const RUBY: Color = Color::Rgb(155, 17, 30);
pub const RITUAL: Color = Color::Rgb(137, 0, 0);
pub const GOLD: Color = Color::Rgb(212, 175, 55);
pub const COPPER: Color = Color::Rgb(184, 115, 51);
pub const PARCHMENT: Color = Color::Rgb(220, 210, 190);
pub const CYAN: Color = Color::Rgb(0, 200, 210);
pub const STEEL: Color = Color::Rgb(120, 130, 140);
pub const DIM: Color = Color::Rgb(60, 60, 70);
pub const TEXT: Color = Color::Rgb(201, 209, 217);
pub const MUTED: Color = Color::Rgb(100, 100, 110);

/// Stale pulse threshold — ~10 missed 200 ms heartbeats.
pub const PULSE_STALE_SECS: u64 = 2;

pub fn tension_color(tension: f64) -> Color {
    if tension > 80.0 {
        RITUAL
    } else if tension > 50.0 {
        COPPER
    } else {
        CYAN
    }
}

pub fn chrome_border(tension: f64, speaking: bool, drop: bool) -> Color {
    chrome_border_with_flags(tension, speaking, drop, &AccessibilityFlags::default())
}

pub fn chrome_border_with_flags(
    tension: f64,
    speaking: bool,
    drop: bool,
    flags: &AccessibilityFlags,
) -> Color {
    if flags.high_contrast {
        if drop || tension > 85.0 {
            return Color::White;
        }
        if speaking && !flags.reduced_motion {
            return GOLD;
        }
        return Color::White;
    }
    if drop || tension > 85.0 {
        RITUAL
    } else if speaking && !flags.reduced_motion {
        GOLD
    } else if tension > 60.0 {
        COPPER
    } else {
        DIM
    }
}
