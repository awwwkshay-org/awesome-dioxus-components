//! A source-owned Dioxus-only palette switcher (no shadcn equivalent).
//!
//! Productizes the primary-color preset concept prototyped in
//! `apps/playground/src/theme.rs`'s advanced customization tray, distilled
//! to just the palette-selection experience (a preset changes `--primary`/
//! `--primary-foreground` and their Tailwind color aliases) rather than that
//! tray's full 28-token editor, which stays playground-only "parity
//! inspection" tooling per design.md §7a. Classified `EXISTING_DIOXUS_EXTRA`
//! per design.md §7b — this component has no shadcn/dioxus-components
//! catalog counterpart in `statics/component_compatibility.json`.

use dioxus::prelude::*;

use adico_primitives::theme_mode::{
    ResolvedTheme, apply_root_properties, use_persisted_theme_mode,
};

use crate::adico_lib::cn::cn;

/// A primary-color preset. Each variant is a light/dark HSL pair, matching
/// the values the playground's own palette presets already use.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ThemePalette {
    #[default]
    Slate,
    Blue,
    Violet,
    Emerald,
    Rose,
    Amber,
}

impl ThemePalette {
    const ALL: [Self; 6] = [
        Self::Slate,
        Self::Blue,
        Self::Violet,
        Self::Emerald,
        Self::Rose,
        Self::Amber,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Slate => "Slate",
            Self::Blue => "Blue",
            Self::Violet => "Violet",
            Self::Emerald => "Emerald",
            Self::Rose => "Rose",
            Self::Amber => "Amber",
        }
    }

    /// A representative swatch color for the picker UI itself (independent
    /// of the active light/dark mode, so the picker's own swatches stay
    /// legible in both appearances).
    const fn swatch_class(self) -> &'static str {
        match self {
            Self::Slate => "bg-slate-500",
            Self::Blue => "bg-blue-500",
            Self::Violet => "bg-violet-500",
            Self::Emerald => "bg-emerald-500",
            Self::Rose => "bg-rose-500",
            Self::Amber => "bg-amber-500",
        }
    }

    const fn primary_hsl(self, dark: bool) -> (&'static str, &'static str) {
        match (self, dark) {
            (Self::Slate, false) => ("222.2 47.4% 11.2%", "210 40% 98%"),
            (Self::Slate, true) => ("210 40% 98%", "222.2 47.4% 11.2%"),
            (Self::Blue, false) => ("221.2 83.2% 53.3%", "210 40% 98%"),
            (Self::Blue, true) => ("213.1 93.9% 67.8%", "222.2 47.4% 11.2%"),
            (Self::Violet, false) => ("262.1 83.3% 57.8%", "210 40% 98%"),
            (Self::Violet, true) => ("263.4 70% 50.4%", "0 0% 100%"),
            (Self::Emerald, false) => ("160.1 84.1% 39.4%", "210 40% 98%"),
            (Self::Emerald, true) => ("158.1 64.4% 51.6%", "2.7 19.3% 10.2%"),
            (Self::Rose, false) => ("346.8 77.2% 49.8%", "210 40% 98%"),
            (Self::Rose, true) => ("349.7 89.2% 60.2%", "0 0% 100%"),
            (Self::Amber, false) => ("37.7 92.1% 50.2%", "26 83.3% 14.1%"),
            (Self::Amber, true) => ("47.9 95.8% 53.1%", "26 83.3% 14.1%"),
        }
    }

    fn apply(self, dark: bool) {
        let (foreground_on, background) = self.primary_hsl(dark);
        apply_root_properties(&[
            ("--primary", foreground_on.to_string()),
            ("--primary-foreground", background.to_string()),
            ("--color-primary", format!("hsl({foreground_on})")),
            ("--color-primary-foreground", format!("hsl({background})")),
        ]);
    }
}

/// A self-contained palette picker. Applies the selected preset's primary
/// role variables immediately and re-applies them whenever the resolved
/// light/dark appearance changes (via the shared `theme_mode` signal), so a
/// mode switch and a palette switch compose correctly.
#[component]
pub fn ThemeSwitcher(class: Option<String>) -> Element {
    let (mode, _) = use_persisted_theme_mode();
    let mut palette = use_signal(ThemePalette::default);

    use_effect(move || {
        let dark = mode().resolve() == ResolvedTheme::Dark;
        palette().apply(dark);
    });

    rsx! {
        div {
            class: cn(&["flex items-center gap-2", class.as_deref().unwrap_or_default()]),
            role: "radiogroup",
            "aria-label": "Theme palette",
            for option in ThemePalette::ALL {
                button {
                    r#type: "button",
                    role: "radio",
                    "aria-checked": palette() == option,
                    "aria-label": "{option.label()} palette",
                    class: cn(&[
                        "h-6 w-6 rounded-full border-2 transition-colors",
                        option.swatch_class(),
                        if palette() == option { "border-ring" } else { "border-transparent" },
                    ]),
                    onclick: move |_| palette.set(option),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_palette_has_a_distinct_label_and_swatch() {
        let labels: Vec<_> = ThemePalette::ALL.iter().map(|p| p.label()).collect();
        let swatches: Vec<_> = ThemePalette::ALL.iter().map(|p| p.swatch_class()).collect();
        for (index, label) in labels.iter().enumerate() {
            assert!(!labels[index + 1..].contains(label));
            assert!(!swatches[index + 1..].contains(&swatches[index]));
        }
    }

    #[test]
    fn light_and_dark_primary_pairs_differ() {
        for palette in ThemePalette::ALL {
            assert_ne!(palette.primary_hsl(false), palette.primary_hsl(true));
        }
    }
}
