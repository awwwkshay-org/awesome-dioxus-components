//! Cross-platform light/dark/system theme-mode primitive.
//!
//! `ThemeMode::System` resolves to a concrete [`ResolvedTheme`] using the OS
//! or browser color-scheme preference on the `web`/`desktop` features (via
//! the `dark-light` crate, which covers macOS/Windows/Linux+BSD/WASM from one
//! API); native SSR/server builds with neither feature enabled deterministic-
//! ally resolve `System` to [`ResolvedTheme::Light`] rather than attempting
//! platform detection, matching this crate's established SSR-safety
//! convention for other target-gated behavior (see `dialog`/`portal`). See
//! `design.md` §7b for the full rationale, including the accepted v1 gap:
//! resolution is a one-shot read on each call, not a live subscription to
//! OS-theme-change notifications (`dark-light`'s `subscribe()`/`stream()`
//! APIs are not wired up here).

use dioxus::prelude::*;

/// The user-selectable theme mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    Light,
    Dark,
    #[default]
    System,
}

/// A concrete appearance — what [`ThemeMode::System`] resolves to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedTheme {
    Light,
    Dark,
}

impl ThemeMode {
    /// Resolve this mode to a concrete [`ResolvedTheme`], detecting the
    /// OS/browser preference for [`ThemeMode::System`] where a client
    /// feature (`web` or `desktop`) is enabled.
    pub fn resolve(self) -> ResolvedTheme {
        match self {
            ThemeMode::Light => ResolvedTheme::Light,
            ThemeMode::Dark => ResolvedTheme::Dark,
            ThemeMode::System => detect_system_theme(),
        }
    }
}

impl ResolvedTheme {
    /// The `class="dark"`-convention token this crate's installed CSS/theme
    /// pipeline (`adico-cli`'s CSS installer) expects on the document root.
    pub fn as_class(self) -> Option<&'static str> {
        match self {
            ResolvedTheme::Light => None,
            ResolvedTheme::Dark => Some("dark"),
        }
    }
}

#[cfg(any(feature = "web", feature = "desktop"))]
fn detect_system_theme() -> ResolvedTheme {
    match dark_light::detect() {
        Ok(dark_light::Mode::Dark) => ResolvedTheme::Dark,
        // `Mode::Light`, `Mode::Unspecified`, and any detection error all
        // fall back to Light, matching this crate's SSR-safe-by-default
        // convention: an unknown preference never silently renders dark.
        _ => ResolvedTheme::Light,
    }
}

#[cfg(not(any(feature = "web", feature = "desktop")))]
fn detect_system_theme() -> ResolvedTheme {
    ResolvedTheme::Light
}

/// Controlled/uncontrolled `ThemeMode` state, following this crate's
/// [`crate::use_controlled`] convention used throughout (see `toggle`,
/// `switch`).
pub fn use_theme_mode(
    mode: ReadSignal<Option<ThemeMode>>,
    default_mode: ThemeMode,
    on_change: Callback<ThemeMode>,
) -> (Memo<ThemeMode>, Callback<ThemeMode>) {
    crate::use_controlled(mode, default_mode, on_change)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_and_dark_resolve_to_themselves() {
        assert_eq!(ThemeMode::Light.resolve(), ResolvedTheme::Light);
        assert_eq!(ThemeMode::Dark.resolve(), ResolvedTheme::Dark);
    }

    #[test]
    fn system_resolves_to_light_without_a_client_feature() {
        // Neither `web` nor `desktop` is enabled for this crate's own test
        // profile (see Cargo.toml `[features] default = []`), so this
        // exercises the deterministic SSR-safe fallback path.
        assert_eq!(ThemeMode::System.resolve(), ResolvedTheme::Light);
    }

    #[test]
    fn resolved_theme_class_matches_installed_css_dark_mode_convention() {
        assert_eq!(ResolvedTheme::Light.as_class(), None);
        assert_eq!(ResolvedTheme::Dark.as_class(), Some("dark"));
    }

    #[test]
    fn theme_mode_default_is_system() {
        assert_eq!(ThemeMode::default(), ThemeMode::System);
    }
}
