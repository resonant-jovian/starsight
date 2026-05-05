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
}

pub const LIGHT: Palette = Palette {
    bg: "#ffffff",
    card: "#ffffff",
    text: "#1a1a1a",
    subtext: "#555555",
    muted: "#888888",
    border: "#cccccc",
    rule: "#e5e5e5",
};

pub const DARK: Palette = Palette {
    bg: "#0e0e10",
    card: "#16161a",
    text: "#f4f4f6",
    subtext: "#a8a8b0",
    muted: "#6b6b75",
    border: "#2c2c33",
    rule: "#232328",
};

pub fn palette(theme: Theme) -> &'static Palette {
    match theme {
        Theme::Light => &LIGHT,
        Theme::Dark => &DARK,
    }
}

/// `(R, G, B, A)` for tiny-skia raster compositing.
pub mod rgba {
    use super::Theme;

    pub fn bg(theme: Theme) -> (u8, u8, u8, u8) {
        match theme {
            Theme::Light => (0xff, 0xff, 0xff, 0xff),
            Theme::Dark => (0x0e, 0x0e, 0x10, 0xff),
        }
    }
    pub fn card(theme: Theme) -> (u8, u8, u8, u8) {
        match theme {
            Theme::Light => (0xff, 0xff, 0xff, 0xff),
            Theme::Dark => (0x16, 0x16, 0x1a, 0xff),
        }
    }
    pub fn border(theme: Theme) -> (u8, u8, u8, u8) {
        match theme {
            Theme::Light => (0xcc, 0xcc, 0xcc, 0xff),
            Theme::Dark => (0x2c, 0x2c, 0x33, 0xff),
        }
    }
}

pub const SANS: &str =
    "-apple-system, BlinkMacSystemFont, &quot;Segoe UI&quot;, Roboto, Helvetica, Arial, sans-serif";
pub const MONO_FAMILY: &str =
    "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace";
