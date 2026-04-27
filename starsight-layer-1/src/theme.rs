//! Theme types wrapping chromata for use in starsight.
//!
//! Provides a unified interface for applying editor/terminal themes to figures.
//! The theme colors are used for background, foreground, grid lines, axis labels, etc.

use crate::primitives::Color;

// ── Theme ───────────────────────────────────────────────────────────────────────────────────────

/// A theme wrapping chromata's Theme with mapped semantic roles for visualization.
///
/// This provides colors for:
/// - `background`: plot background
/// - `foreground`: default text/label color
/// - `grid`: grid line color
/// - `accent`: primary accent color from the theme
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// Background color for the plot area.
    pub background: Color,
    /// Default foreground color for text and labels.
    pub foreground: Color,
    /// Color for grid lines.
    pub grid: Color,
    /// Primary accent color.
    pub accent: Color,
    /// Color for axis lines and ticks.
    pub axis: Color,
    /// Color for tick labels.
    pub tick_label: Color,
    /// Color for title.
    pub title: Color,
    /// Whether this is a dark theme.
    pub is_dark: bool,
}

impl Theme {
    /// Create a theme from a chromata theme.
    ///
    /// Maps chromata's semantic colors to visualization roles.
    /// Falls back to sensible defaults if optional colors are not defined.
    #[must_use]
    pub fn from_chromata(theme: &chromata::Theme) -> Self {
        let background = Color::from(theme.bg);
        let foreground = Color::from(theme.fg);
        let accent = Color::from(theme.accent());

        let grid = theme
            .line_highlight
            .or(theme.selection)
            .map(Color::from)
            .unwrap_or_else(|| foreground.lerp(background, 0.8));

        let axis = foreground.lerp(background, 0.7);

        let tick_label = theme
            .comment
            .map(Color::from)
            .unwrap_or_else(|| foreground.lerp(background, 0.6));

        let title = foreground;

        Self {
            background,
            foreground,
            grid,
            accent,
            axis,
            tick_label,
            title,
            is_dark: theme.is_dark(),
        }
    }

    /// Get a contrasting text color for the given background.
    #[must_use]
    pub fn contrast_text(&self, bg: Color) -> Color {
        if bg.contrast_ratio(self.background) > 7.0 {
            self.background
        } else {
            self.foreground
        }
    }
}

impl From<chromata::Theme> for Theme {
    fn from(theme: chromata::Theme) -> Self {
        Self::from_chromata(&theme)
    }
}

impl From<&chromata::Theme> for Theme {
    fn from(theme: &chromata::Theme) -> Self {
        Self::from_chromata(theme)
    }
}

// ── Default Theme ─────────────────────────────────────────────────────────────────────────────

/// Default light theme (white background, dark text).
pub const DEFAULT_LIGHT: Theme = Theme {
    background: Color::WHITE,
    foreground: Color::from_hex(0x333333),
    grid: Color::from_hex(0xDDDDDD),
    accent: Color::from_hex(0x2196F3),
    axis: Color::from_hex(0x666666),
    tick_label: Color::from_hex(0x555555),
    title: Color::from_hex(0x222222),
    is_dark: false,
};

/// Default dark theme (dark background, light text).
pub const DEFAULT_DARK: Theme = Theme {
    background: Color::from_hex(0x1E1E1E),
    foreground: Color::from_hex(0xE0E0E0),
    grid: Color::from_hex(0x3A3A3A),
    accent: Color::from_hex(0x64B5F6),
    axis: Color::from_hex(0xAAAAAA),
    tick_label: Color::from_hex(0xBBBBBB),
    title: Color::from_hex(0xFFFFFF),
    is_dark: true,
};

impl Default for Theme {
    fn default() -> Self {
        DEFAULT_LIGHT
    }
}

#[cfg(test)]
mod tests {
    use super::{Color, DEFAULT_DARK, DEFAULT_LIGHT, Theme};

    fn build_theme(line_highlight: Option<chromata::Color>, comment: Option<chromata::Color>) -> chromata::Theme {
        let mut builder = chromata::ThemeBuilder::new(
            "Test",
            "tester",
            chromata::Color::new(0, 0, 0),
            chromata::Color::new(255, 255, 255),
        );
        if let Some(c) = line_highlight {
            builder = builder.line_highlight(c);
        }
        if let Some(c) = comment {
            builder = builder.comment(c);
        }
        builder.build()
    }

    #[test]
    fn from_chromata_with_optional_colors() {
        let t = build_theme(
            Some(chromata::Color::new(50, 50, 50)),
            Some(chromata::Color::new(100, 100, 100)),
        );
        let theme = Theme::from_chromata(&t);
        assert_eq!(theme.background, Color::new(0, 0, 0));
        assert_eq!(theme.foreground, Color::new(255, 255, 255));
        assert_eq!(theme.grid, Color::new(50, 50, 50));
        assert_eq!(theme.tick_label, Color::new(100, 100, 100));
        assert!(theme.is_dark);
    }

    #[test]
    fn from_chromata_falls_back_when_optional_missing() {
        let t = build_theme(None, None);
        let theme = Theme::from_chromata(&t);
        assert!(theme.grid.r > 0); // lerp fallback should produce non-zero
        assert!(theme.tick_label.r > 0);
    }

    #[test]
    fn from_chromata_value() {
        let t = build_theme(None, None);
        let theme: Theme = t.into();
        assert_eq!(theme.background.r, 0);
    }

    #[test]
    fn from_chromata_ref() {
        let t = build_theme(None, None);
        let theme: Theme = (&t).into();
        assert_eq!(theme.background.r, 0);
    }

    #[test]
    fn contrast_text_picks_background_on_high_contrast() {
        // Light theme background is white; black has high contrast against it
        let t = DEFAULT_LIGHT;
        let chosen = t.contrast_text(Color::BLACK);
        assert_eq!(chosen, t.background);
    }

    #[test]
    fn contrast_text_picks_foreground_on_low_contrast() {
        // Foreground when contrast is low against background
        let t = DEFAULT_LIGHT;
        let chosen = t.contrast_text(Color::new(250, 250, 250));
        assert_eq!(chosen, t.foreground);
    }

    #[test]
    fn default_is_light() {
        let t = Theme::default();
        assert!(!t.is_dark);
        assert_eq!(t.background, DEFAULT_LIGHT.background);
    }

    #[test]
    fn default_dark_constant() {
        assert!(DEFAULT_DARK.is_dark);
        assert_ne!(DEFAULT_DARK.background, DEFAULT_LIGHT.background);
    }
}
