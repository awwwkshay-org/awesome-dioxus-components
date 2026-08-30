//! A shadcn-style Light/Dark/System theme toggle, backed by
//! `adico-primitives::theme_mode`.
//!
//! Composes the installed [`super::dropdown_menu`] registry item, matching
//! shadcn's own documented dark-mode toggle pattern (a small icon trigger
//! opening a three-item menu) and this repo's dialog-composes-button
//! composition convention (reuse a sibling styled facade, not the raw
//! primitive, wherever one already exists). The selected mode persists
//! across reloads and its resolved appearance is applied to the document
//! root automatically — see
//! `adico_primitives::theme_mode::use_persisted_theme_mode`.

use dioxus::prelude::*;

use adico_primitives::icons::{Monitor, Moon, Sun};
use adico_primitives::theme_mode::{ThemeMode, use_persisted_theme_mode};

use super::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use crate::adico_lib::cn::cn;

fn mode_label(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "Light",
        ThemeMode::Dark => "Dark",
        ThemeMode::System => "System",
    }
}

/// A self-contained Light/Dark/System toggle. Renders its own trigger and
/// menu, so it can be dropped into a layout with no other props required.
#[component]
pub fn ModeToggle(class: Option<String>) -> Element {
    let (mode, set_mode) = use_persisted_theme_mode();
    let mut open = use_signal(|| false);

    rsx! {
        DropdownMenu {
            open: Some(open()),
            on_open_change: move |value| open.set(value),
            class: class.unwrap_or_default(),
            DropdownMenuTrigger {
                class: cn(&["h-9 w-9 justify-center px-0"]),
                match mode() {
                    ThemeMode::Light => rsx! {
                        Sun { class: "h-4 w-4" }
                    },
                    ThemeMode::Dark => rsx! {
                        Moon { class: "h-4 w-4" }
                    },
                    ThemeMode::System => rsx! {
                        Monitor { class: "h-4 w-4" }
                    },
                }
                // `DropdownMenuTrigger` doesn't extend `GlobalAttributes` (it
                // wraps a primitive component, not a native element -- the
                // same constraint recorded for other registry facades in
                // docs/adico/m3-wave2-migration.md), so the accessible name
                // comes from visually-hidden text content instead of an
                // `aria-label` attribute.
                span { class: "sr-only", "Toggle theme (current: {mode_label(mode())})" }
            }
            DropdownMenuContent {
                for (index , option) in
                    [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System].into_iter().enumerate()
                {
                    DropdownMenuItem::<ThemeMode> {
                        key: "{index}",
                        value: option,
                        index,
                        on_select: move |value| set_mode.call(value),
                        class: if mode() == option { "font-semibold" } else { "" },
                        {mode_label(option)}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mode_has_a_distinct_label() {
        assert_eq!(mode_label(ThemeMode::Light), "Light");
        assert_eq!(mode_label(ThemeMode::Dark), "Dark");
        assert_eq!(mode_label(ThemeMode::System), "System");
    }
}
