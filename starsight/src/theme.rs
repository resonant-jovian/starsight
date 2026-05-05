pub use starsight_layer_1::theme::{DEFAULT_DARK, DEFAULT_LIGHT, Theme};

/// Read `STARSIGHT_THEME` and return the corresponding default theme.
///
/// `dark` → [`DEFAULT_DARK`]; anything else (or unset) → [`DEFAULT_LIGHT`].
/// Used by the chrome examples so a single example binary can render both
/// theme variants based on its environment.
#[must_use]
pub fn theme_from_env() -> Theme {
    match std::env::var("STARSIGHT_THEME").as_deref() {
        Ok("dark") => DEFAULT_DARK,
        _ => DEFAULT_LIGHT,
    }
}

/// Filename suffix paired with [`theme_from_env`]. `"_dark"` for dark mode,
/// `""` for light. Lets examples write `<name>.png` and `<name>_dark.png`
/// from the same code path with no other branching.
#[must_use]
pub fn theme_suffix_from_env() -> &'static str {
    match std::env::var("STARSIGHT_THEME").as_deref() {
        Ok("dark") => "_dark",
        _ => "",
    }
}
