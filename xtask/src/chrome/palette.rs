//! Two monochrome palettes (light + dark), threaded through every chrome
//! generator so each asset emits paired `<name>-light.<ext>` and
//! `<name>-dark.<ext>` files for `<picture>` auto-selection.

#[derive(Copy, Clone)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub const ALL: [Theme; 2] = [Theme::Light, Theme::Dark];

    pub fn suffix(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    pub fn example_suffix(self) -> &'static str {
        match self {
            Theme::Light => "",
            Theme::Dark => "_dark",
        }
    }
}

pub struct Palette {
    pub bg: &'static str,
    pub card: &'static str,
    pub text: &'static str,
    pub subtext: &'static str,
    pub muted: &'static str,
    pub border: &'static str,
    pub rule: &'static str,
    /// Stroke colour for the eclipse mark's corona rays. Has to be visibly
    /// brighter than the surrounding background; dark mode bumps higher than
    /// `muted` would suggest to compensate for human perception of low-contrast
    /// shapes against near-black canvases.
    pub ray: &'static str,
}

pub const LIGHT: Palette = Palette {
    bg: "#ffffff",
    card: "#ffffff",
    text: "#1a1a1a",
    subtext: "#555555",
    muted: "#888888",
    border: "#cccccc",
    rule: "#e5e5e5",
    ray: "#888888",
};

pub const DARK: Palette = Palette {
    bg: "#0e0e10",
    card: "#16161a",
    text: "#f4f4f6",
    subtext: "#a8a8b0",
    muted: "#6b6b75",
    border: "#2c2c33",
    rule: "#232328",
    ray: "#c8c8cc",
};

pub fn palette(theme: Theme) -> &'static Palette {
    match theme {
        Theme::Light => &LIGHT,
        Theme::Dark => &DARK,
    }
}

pub const SANS: &str =
    "-apple-system, BlinkMacSystemFont, &quot;Segoe UI&quot;, Roboto, Helvetica, Arial, sans-serif";
pub const MONO_FAMILY: &str = "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace";
