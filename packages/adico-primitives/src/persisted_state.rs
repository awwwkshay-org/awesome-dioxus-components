//! A generic, reusable "persisted `GlobalSignal`" primitive.
//!
//! [`theme_mode`](crate::theme_mode)'s `use_persisted_theme_mode` was the
//! first (and, before this module existed, the only) example in this crate
//! of a shared app-wide setting that survives a reload. This module
//! generalizes that exact pattern -- a module-level `GlobalSignal` plus
//! `localStorage`-on-`web`/preferences-file-on-`native` persistence -- into
//! one reusable hook, so a second persisted setting (for example
//! `theme-switcher`'s palette preset) doesn't need to hand-roll its own copy
//! of the same three-target-gated load/persist machinery.
//!
//! Deliberately a hook backed by a `GlobalSignal`, not a Context/Provider
//! component: every registry item in this ecosystem is a drop-in component a
//! consumer mounts anywhere with zero setup (`ModeToggle {}`,
//! `ThemeSwitcher {}`, ...); a Provider would force every consumer app to
//! wrap its root in a new required component just to use one of these, which
//! this shadcn-style, copy-paste-source ecosystem does not otherwise ask of
//! anyone. `GlobalSignal` already gives "read and write from anywhere,
//! observed everywhere" for free, matching this crate's own established
//! shared-state idiom (`theme_mode.rs`'s `MODE`, `pointer.rs`'s `POINTERS`).
//!
//! Accepted v1 limitation, inherited from `use_persisted_theme_mode`'s own:
//! the persisted value loads asynchronously after first mount on `web` (a
//! `dioxus_document::eval` round trip), so a render can briefly show the
//! default value before the stored one lands. A synchronous,
//! hydration-matching read would need an inline pre-hydration script, out of
//! scope here.
//!
//! The native store is `std::env::temp_dir()`, a deliberately simple
//! location rather than a real per-app data directory (which would need an
//! additional dependency this crate doesn't otherwise carry, e.g. `dirs`).
//! Named, accepted limitation, not a durable store.
//!
//! Persisted values round-trip as a short string token (`to_token`/
//! `from_token`), not real JSON -- this crate carries **zero** serde
//! dependency (by design, kept lean), so the native preferences file's
//! `{"value":"<token>"}` is a hand-rolled single-field record, parsed by
//! plain string splitting, the same technique `theme_mode.rs` already used
//! for its own `{"mode":"<token>"}` file before this module existed.
//!
//! `storage_key` names both a `localStorage` entry and (via
//! `{storage_key}.json`) a native file name, so it's restricted to
//! non-empty ASCII `[A-Za-z0-9_-]` -- enforced with a `debug_assert!`, not a
//! runtime error, since it's always a caller-supplied constant, never
//! user input.

use dioxus::prelude::*;

/// An uncontrolled, persisted `T` signal, shared app-wide through `global`.
/// Persists to `localStorage[storage_key]` on `web`, to a
/// `{storage_key}.json` preferences file under the OS temp directory on
/// `native`, and behaves like a plain in-memory shared signal (no
/// persistence) when neither client feature is enabled.
///
/// `to_token`/`from_token` are the same fn-pointer-based token conversion
/// convention `theme_mode.rs`'s own `mode_token`/`mode_from_token` already
/// use, so an existing enum's token functions plug straight into this hook
/// without restructuring (see `use_persisted_theme_mode`'s own
/// implementation, which is this hook plus `apply_resolved_class()`).
///
/// Preserves `use_persisted_theme_mode`'s exact hook order -- `use_memo` →
/// `use_callback` (load) → load-effect → `use_callback` (set) -- so
/// refactoring an existing persisted signal onto this primitive is a
/// zero-behavior-change move, not a rewrite.
pub fn use_persisted_global<T: Copy + PartialEq + 'static>(
    global: &'static GlobalSignal<T>,
    storage_key: &'static str,
    to_token: fn(T) -> &'static str,
    from_token: fn(&str) -> Option<T>,
) -> (Memo<T>, Callback<T>) {
    debug_assert!(
        is_valid_storage_key(storage_key),
        "storage_key names both a localStorage entry and a native preferences \
         file name, so it must be non-empty ASCII [A-Za-z0-9_-]"
    );

    let value = use_memo(move || *global.read());

    let on_loaded = use_callback(move |loaded: T| *global.write() = loaded);
    load_persisted(storage_key, from_token, on_loaded);

    let set_value = use_callback(move |next: T| {
        *global.write() = next;
        persist(storage_key, to_token(next));
    });

    (value, set_value)
}

fn is_valid_storage_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(feature = "web")]
fn load_persisted<T: 'static>(
    storage_key: &'static str,
    from_token: fn(&str) -> Option<T>,
    on_loaded: Callback<T>,
) {
    use_effect(move || {
        let mut eval = dioxus_document::eval(
            "const key = await dioxus.recv();
            dioxus.send(window.localStorage.getItem(key));",
        );
        let _ = eval.send(storage_key);
        spawn(async move {
            if let Ok(Some(token)) = eval.recv::<Option<String>>().await
                && let Some(value) = from_token(&token)
            {
                on_loaded.call(value);
            }
        });
    });
}

#[cfg(feature = "native")]
fn load_persisted<T: 'static>(
    storage_key: &'static str,
    from_token: fn(&str) -> Option<T>,
    on_loaded: Callback<T>,
) {
    use_effect(move || {
        if let Some(token) = read_preferences_file(storage_key)
            && let Some(value) = from_token(&token)
        {
            on_loaded.call(value);
        }
    });
}

#[cfg(not(any(feature = "web", feature = "native")))]
fn load_persisted<T: 'static>(
    _storage_key: &'static str,
    _from_token: fn(&str) -> Option<T>,
    _on_loaded: Callback<T>,
) {
}

#[cfg(feature = "web")]
fn persist(storage_key: &'static str, token: &'static str) {
    let eval = dioxus_document::eval(
        "const [key, value] = await dioxus.recv();
        window.localStorage.setItem(key, value);",
    );
    let _ = eval.send((storage_key, token));
}

#[cfg(feature = "native")]
fn persist(storage_key: &'static str, token: &'static str) {
    write_preferences_file(storage_key, token);
}

#[cfg(not(any(feature = "web", feature = "native")))]
fn persist(_storage_key: &'static str, _token: &'static str) {}

#[cfg(any(feature = "native", test))]
fn preferences_file_name(storage_key: &str) -> String {
    format!("{storage_key}.json")
}

#[cfg(any(feature = "native", test))]
fn preferences_contents(token: &str) -> String {
    format!("{{\"value\":\"{token}\"}}")
}

#[cfg(any(feature = "native", test))]
fn parse_preferences_contents(contents: &str) -> Option<&str> {
    contents
        .split_once("\"value\":\"")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map(|(token, _)| token)
}

#[cfg(feature = "native")]
fn preferences_path(storage_key: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(preferences_file_name(storage_key))
}

#[cfg(feature = "native")]
fn read_preferences_file(storage_key: &str) -> Option<String> {
    let contents = std::fs::read_to_string(preferences_path(storage_key)).ok()?;
    parse_preferences_contents(&contents).map(str::to_string)
}

#[cfg(feature = "native")]
fn write_preferences_file(storage_key: &str, token: &str) {
    let _ = std::fs::write(preferences_path(storage_key), preferences_contents(token));
}

#[cfg(test)]
mod tests {
    use super::*;

    // A throwaway stand-in for a real persisted setting, written exactly the
    // way a real call site writes one (a `#[default]`-having enum with a
    // `const fn` token getter and a plain-`fn` reverse lookup), so this
    // proves the `to_token`/`from_token` fn-pointer contract actually
    // coerces -- not just that a hand-written match arm agrees with itself.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    enum TestSetting {
        #[default]
        One,
        Two,
    }

    impl TestSetting {
        const ALL: [Self; 2] = [Self::One, Self::Two];

        const fn token(self) -> &'static str {
            match self {
                Self::One => "one",
                Self::Two => "two",
            }
        }

        fn from_token(token: &str) -> Option<Self> {
            Self::ALL
                .into_iter()
                .find(|candidate| candidate.token() == token)
        }
    }

    #[test]
    fn a_token_pair_coerces_to_the_hooks_fn_pointer_parameters() {
        let to_token: fn(TestSetting) -> &'static str = TestSetting::token;
        let from_token: fn(&str) -> Option<TestSetting> = TestSetting::from_token;
        for setting in TestSetting::ALL {
            assert_eq!(from_token(to_token(setting)), Some(setting));
        }
    }

    #[test]
    fn from_token_rejects_an_unrecognized_or_empty_token() {
        assert_eq!(TestSetting::from_token(""), None);
        assert_eq!(TestSetting::from_token("three"), None);
    }

    #[test]
    fn every_token_is_distinct() {
        let tokens: Vec<_> = TestSetting::ALL.iter().map(|s| s.token()).collect();
        for (index, token) in tokens.iter().enumerate() {
            assert!(!tokens[index + 1..].contains(token));
        }
    }

    #[test]
    fn preferences_file_name_reproduces_the_shipped_theme_mode_file() {
        assert_eq!(
            preferences_file_name("adico-theme-mode"),
            "adico-theme-mode.json"
        );
    }

    #[test]
    fn preferences_contents_round_trip() {
        let contents = preferences_contents("dark");
        assert_eq!(parse_preferences_contents(&contents), Some("dark"));
    }

    #[test]
    fn parse_preferences_contents_rejects_garbage() {
        assert_eq!(parse_preferences_contents(""), None);
        assert_eq!(parse_preferences_contents("{}"), None);
        assert_eq!(
            parse_preferences_contents("{\"value\":\"unterminated"),
            None
        );
    }

    #[test]
    fn storage_key_validation_rejects_path_and_whitespace_characters() {
        assert!(is_valid_storage_key("adico-theme-mode"));
        assert!(is_valid_storage_key("adico_theme_palette"));
        assert!(!is_valid_storage_key(""));
        assert!(!is_valid_storage_key("../escape"));
        assert!(!is_valid_storage_key("a b"));
        assert!(!is_valid_storage_key("a/b"));
    }
}
