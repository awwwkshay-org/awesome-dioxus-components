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

const STORAGE_KEY: &str = "adico-theme-mode";

fn mode_token(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
        ThemeMode::System => "system",
    }
}

fn mode_from_token(token: &str) -> Option<ThemeMode> {
    match token {
        "light" => Some(ThemeMode::Light),
        "dark" => Some(ThemeMode::Dark),
        "system" => Some(ThemeMode::System),
        _ => None,
    }
}

/// Shared app-wide theme mode. A `GlobalSignal` (matching this crate's
/// existing cross-component-shared-state convention, see `pointer.rs`'s
/// `POINTERS`) rather than a per-call-site `use_signal`, so every
/// `use_persisted_theme_mode()` caller in the same app — a `mode-toggle` and
/// a `theme-switcher` both mounted on one page, for example — reads and
/// drives the same live value instead of silently diverging until the next
/// reload re-reads persisted storage.
static MODE: GlobalSignal<ThemeMode> = Global::new(ThemeMode::default);

/// An uncontrolled `ThemeMode` signal, shared app-wide, that persists the
/// user's selection across reloads (`localStorage` on `web`; a small JSON
/// preferences file on `desktop`) and applies the resolved appearance's class
/// to the document root on `web`. Builds with neither feature enabled have no
/// persistence or DOM target and behave like a plain in-memory shared signal
/// defaulting to [`ThemeMode::System`] every render — the same accepted
/// limitation this crate's other stateful client primitives have for native
/// SSR/server builds.
///
/// Accepted v1 limitation (see `design.md` §7b): the persisted value is read
/// asynchronously after first mount, so a `web` render briefly shows the
/// default `System` resolution before the stored preference applies. This is
/// a named gap, not a silent one — a synchronous, hydration-matching read
/// would need an inline pre-hydration script, which is out of scope here.
pub fn use_persisted_theme_mode() -> (Memo<ThemeMode>, Callback<ThemeMode>) {
    let value = use_memo(|| *MODE.read());

    let on_loaded = use_callback(|loaded: ThemeMode| *MODE.write() = loaded);
    load_persisted_mode(on_loaded);

    let set_mode = use_callback(|next: ThemeMode| {
        *MODE.write() = next;
        persist_mode(next);
    });

    apply_resolved_class();

    (value, set_mode)
}

#[cfg(feature = "web")]
fn load_persisted_mode(on_loaded: Callback<ThemeMode>) {
    use_effect(move || {
        let mut eval = dioxus_document::eval(
            "const key = await dioxus.recv();
            dioxus.send(window.localStorage.getItem(key));",
        );
        let _ = eval.send(STORAGE_KEY);
        spawn(async move {
            if let Ok(Some(token)) = eval.recv::<Option<String>>().await
                && let Some(mode) = mode_from_token(&token)
            {
                on_loaded.call(mode);
            }
        });
    });
}

#[cfg(feature = "desktop")]
fn load_persisted_mode(on_loaded: Callback<ThemeMode>) {
    use_effect(move || {
        if let Some(token) = read_desktop_preferences()
            && let Some(mode) = mode_from_token(&token)
        {
            on_loaded.call(mode);
        }
    });
}

#[cfg(not(any(feature = "web", feature = "desktop")))]
fn load_persisted_mode(_on_loaded: Callback<ThemeMode>) {}

#[cfg(feature = "web")]
fn persist_mode(mode: ThemeMode) {
    let eval = dioxus_document::eval(
        "const [key, value] = await dioxus.recv();
        window.localStorage.setItem(key, value);",
    );
    let _ = eval.send((STORAGE_KEY, mode_token(mode)));
}

#[cfg(feature = "desktop")]
fn persist_mode(mode: ThemeMode) {
    write_desktop_preferences(mode_token(mode));
}

#[cfg(not(any(feature = "web", feature = "desktop")))]
fn persist_mode(_mode: ThemeMode) {}

/// Applies `MODE`'s resolved appearance's CSS class to the document root on
/// `web`, matching the `class`-driven dark-mode convention `adico-cli`'s CSS
/// installer sets up. `desktop`'s native window has no shared document root
/// for installed component CSS to react to, so this is a `web`-only effect;
/// desktop consumers apply the resolved theme through their own window/webview
/// styling, same as every other web-only visual primitive in this crate.
///
/// Reads `MODE` directly inside the effect body (rather than taking it as a
/// parameter captured by value) so Dioxus's automatic reactive-dependency
/// tracking actually re-runs this effect on every mode change -- an earlier
/// version passed the resolved value in as a plain argument, which compiled
/// and rendered fine but silently never re-applied the class after the first
/// mount (found only by a real, live `mode-toggle` Playwright run: `<html>`
/// never gained a `dark` class after clicking "Dark", not by any compile or
/// unit-test check).
#[cfg(feature = "web")]
fn apply_resolved_class() {
    use_effect(move || {
        let class = MODE.read().resolve().as_class().unwrap_or("");
        let eval = dioxus_document::eval(
            "const cls = await dioxus.recv();
            const root = document.documentElement;
            root.classList.remove('dark');
            if (cls) root.classList.add(cls);",
        );
        let _ = eval.send(class);
    });
}

#[cfg(not(feature = "web"))]
fn apply_resolved_class() {}

/// Sets a list of `--custom-property: value` pairs on the document root
/// (`:root`) on `web`, so a consumer's `theme-switcher`-style palette picker
/// can recolor semantic tokens live without becoming a second DOM-eval call
/// site outside this crate. No-op on every other target — `desktop`'s native
/// window has no shared CSS custom-property root for installed component
/// styling to react to, matching `apply_resolved_class`'s own scoping.
#[cfg(feature = "web")]
pub fn apply_root_properties(pairs: &[(&str, String)]) {
    let script = pairs
        .iter()
        .map(|(name, value)| format!("root.style.setProperty('{name}', {value:?});"))
        .collect::<String>();
    let _ = dioxus_document::eval(&format!("const root = document.documentElement; {script}"));
}

#[cfg(not(feature = "web"))]
pub fn apply_root_properties(_pairs: &[(&str, String)]) {}

/// A deliberately simple location: the OS temp directory rather than a real
/// per-app data directory (which would need an additional dependency this
/// crate doesn't otherwise need, e.g. `dirs`). Named, accepted v1 limitation
/// — a production consumer wanting a proper app-data location can override
/// this by writing their own persistence around `use_theme_mode` instead of
/// `use_persisted_theme_mode`.
#[cfg(feature = "desktop")]
fn desktop_preferences_path() -> std::path::PathBuf {
    std::env::temp_dir().join("adico-theme-mode.json")
}

#[cfg(feature = "desktop")]
fn read_desktop_preferences() -> Option<String> {
    let contents = std::fs::read_to_string(desktop_preferences_path()).ok()?;
    contents
        .split_once("\"mode\":\"")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map(|(mode, _)| mode.to_string())
}

#[cfg(feature = "desktop")]
fn write_desktop_preferences(mode: &str) {
    let _ = std::fs::write(
        desktop_preferences_path(),
        format!("{{\"mode\":\"{mode}\"}}"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_and_dark_resolve_to_themselves() {
        assert_eq!(ThemeMode::Light.resolve(), ResolvedTheme::Light);
        assert_eq!(ThemeMode::Dark.resolve(), ResolvedTheme::Dark);
    }

    // This crate's own `default = []` means `cargo test -p adico-primitives`
    // in isolation exercises the deterministic SSR-safe fallback below. But
    // `cargo test --workspace` unifies Cargo features across every workspace
    // member building against the same target -- since other members (the
    // examples, the playground) depend on `adico-primitives` with
    // `features = ["web"]`, that feature ends up enabled for this crate's
    // own unit tests too under a workspace-wide test run, and `System` then
    // genuinely calls `dark_light::detect()` and reflects this machine's
    // real OS appearance (which is why an earlier version of this test
    // flaked: it assumed feature isolation `cargo test --workspace` does not
    // provide). These two tests are written to hold under both profiles.
    #[test]
    #[cfg(not(any(feature = "web", feature = "desktop")))]
    fn system_resolves_to_light_without_a_client_feature() {
        assert_eq!(ThemeMode::System.resolve(), ResolvedTheme::Light);
    }

    #[test]
    #[cfg(any(feature = "web", feature = "desktop"))]
    fn system_resolves_to_a_real_detected_appearance_with_a_client_feature() {
        // No specific variant is asserted -- the real OS/browser preference
        // is environment-dependent -- only that resolution is deterministic
        // (repeated calls agree) and produces a valid `ResolvedTheme`.
        let first = ThemeMode::System.resolve();
        let second = ThemeMode::System.resolve();
        assert_eq!(first, second);
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
