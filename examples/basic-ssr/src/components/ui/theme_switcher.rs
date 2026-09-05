//! A source-owned Dioxus-only palette switcher (no shadcn equivalent).
//!
//! Productizes the palette preset concept prototyped in
//! `apps/playground/src/theme.rs`'s advanced customization tray, distilled to
//! just the palette-selection experience: **one** [`super::select::Select`]
//! whose options each set the primary, secondary, and accent semantic role
//! groups together as one coordinated preset -- rather than that tray's full
//! 28-token editor (`theme-builder`'s job), and rather than three
//! independent per-role pickers (an earlier version of this component; a
//! single combined choice is what was actually wanted -- one theme,
//! consistently applied to every role at once). `theme-switcher` still never
//! exposes the individual surface/structural tokens (`--background`,
//! `--card`, `--border`, `--radius`, ...) `theme-builder` covers; only the
//! three palette-derived role groups, and only as one linked choice.
//! Classified `EXISTING_DIOXUS_EXTRA` per design.md §7b -- this component
//! has no shadcn/dioxus-components catalog counterpart in
//! `statics/component_compatibility.json`.
//!
//! Each option previews its preset as three overlapping circles -- primary,
//! secondary, and accent's own real colors for that preset -- the same
//! "overlapping swatches" convention modern theme pickers use, sourced from
//! the exact HSL values [`ThemePalette::primary_pairs`]/
//! [`ThemePalette::secondary_pairs`]/[`ThemePalette::accent_pairs`] actually
//! apply, recomputed for whichever appearance (light/dark) is currently
//! resolved so the preview always matches what turning dark mode on or off
//! would actually show. Declares `select` as a `registryDependencies` entry
//! in `registry.json` so installing `theme-switcher` also installs it,
//! matching the existing `mode-toggle -> dropdown-menu` / `date-picker ->
//! calendar, popover` cross-`registry:ui` dependency precedent.
//!
//! The selected preset lives in a module-level [`PALETTE`] `GlobalSignal`
//! (matching `theme_mode.rs`'s own `MODE` convention), not a per-mount
//! `use_signal`: the playground mounts both a persistent sidebar instance
//! and, on this component's own demo page, a second instance at the same
//! time, and both need to show and drive the same live selection rather
//! than each keeping its own independent copy that silently drifts from the
//! other's.
//!
//! **Hydrates from the live theme on mount**, rather than always resetting
//! to hardcoded Slate: it reads back the primary/secondary/accent role
//! groups' effective raw values with
//! [`adico_primitives::theme_mode::read_root_properties`], once, on mount,
//! and adopts a preset only if all three roles independently match the
//! *same* preset in [`ThemePalette`]'s HSL tables -- if a consumer used
//! `theme-builder` to give the roles inconsistent values, or nothing
//! matches, this component falls back to showing Slate selected without
//! touching the live color itself; it has no notion of "custom", only which
//! of its 6 coordinated presets (if any) is currently in effect. Without
//! this, mounting a second `ThemeSwitcher` (for example the playground's own
//! persistent sidebar instance plus this component's own demo page) would
//! silently reset every role back to Slate the moment either one mounted.
//!
//! The read is deliberately one-shot (gated by a `hydrated` flag), not
//! repeated on every later mode change -- re-reading was tried first and
//! reverted after it was found to silently wipe out a *different*,
//! still-mounted editor's own applied colors (a `theme-builder` instance
//! opened in a dialog, say) the moment this component's own hydration effect
//! re-ran. See `theme-builder`'s own module doc comment and
//! `theme_mode.rs`'s `read_root_properties` doc comment for the full story;
//! the short version is that a mode change alone needs no fresh DOM read
//! here anyway -- the "apply" effect below already recomputes each role's
//! actual colors purely from [`PALETTE`] and the freshly resolved
//! appearance.
//!
//! Writes only the raw `--foo` custom properties, never the `--color-foo`
//! Tailwind aliases the installed `@theme` block already derives from them
//! live via `var()`. An earlier version also wrote `--color-primary`
//! inline, which freezes that alias at a static value and stops it tracking
//! further `--primary` changes -- the same defect fixed in `theme-builder`;
//! see that file's module doc comment and `theme_mode.rs`'s
//! `read_root_properties` doc comment for the full story.

use dioxus::prelude::*;

use adico_primitives::theme_mode::{
    ResolvedTheme, apply_root_properties, read_root_properties, use_persisted_theme_mode,
};

use super::select::{Select, SelectList, SelectOption, SelectTrigger, SelectValue};
use crate::adico_lib::cn::cn;

/// A coordinated palette preset applied to the primary, secondary, and
/// accent semantic role groups together.
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

    /// `(--primary, --primary-foreground)` HSL pair, matching
    /// `theme-builder`'s `Palette::primary_tokens` table exactly.
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

    /// `(background, foreground)` HSL pair for a pastel "surface" role --
    /// shared by the semantic `--secondary`/`--muted` role group and the
    /// `--accent`/`--sidebar-accent` role group, matching `theme-builder`'s
    /// `Palette::surface_tokens` table exactly.
    const fn surface_hsl(self, dark: bool) -> (&'static str, &'static str) {
        match (self, dark) {
            (Self::Slate, false) => ("210 40% 96.1%", "222.2 47.4% 11.2%"),
            (Self::Slate, true) => ("217.2 32.6% 17.5%", "210 40% 98%"),
            (Self::Blue, false) => ("214.3 94.6% 92.7%", "221.2 83.2% 29.4%"),
            (Self::Blue, true) => ("217.2 32.6% 17.5%", "219.4 100% 92%"),
            (Self::Violet, false) => ("250 100% 95.3%", "262.1 83.3% 30%"),
            (Self::Violet, true) => ("263.4 38.6% 17.8%", "250 100% 92%"),
            (Self::Emerald, false) => ("152.4 76% 92.2%", "161.4 93.5% 16.9%"),
            (Self::Emerald, true) => ("163.1 36.7% 16.1%", "149.3 80.4% 90%"),
            (Self::Rose, false) => ("355.6 100% 94.7%", "343.4 79.7% 25.7%"),
            (Self::Rose, true) => ("343.4 43.8% 16.1%", "355.6 100% 94.7%"),
            (Self::Amber, false) => ("48 96.5% 88.8%", "26 83.3% 14.1%"),
            (Self::Amber, true) => ("30 47.8% 16.1%", "48 96.5% 88.8%"),
        }
    }

    /// The preset (if any) whose [`Self::primary_hsl`] exactly matches
    /// `(background, foreground)` for the given appearance.
    fn matching_primary(background: &str, foreground: &str, dark: bool) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|palette| palette.primary_hsl(dark) == (background, foreground))
    }

    /// The preset (if any) whose [`Self::surface_hsl`] exactly matches
    /// `(background, foreground)` for the given appearance.
    fn matching_surface(background: &str, foreground: &str, dark: bool) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|palette| palette.surface_hsl(dark) == (background, foreground))
    }

    /// The raw property/value pairs this preset contributes when applied to
    /// the `--primary` role group (including `--ring` and the sidebar
    /// counterparts, matching `theme-builder`'s `set_primary_palette`).
    fn primary_pairs(self, dark: bool) -> [(&'static str, String); 6] {
        let (primary, primary_foreground) = self.primary_hsl(dark);
        [
            ("--primary", primary.to_string()),
            ("--primary-foreground", primary_foreground.to_string()),
            ("--ring", primary.to_string()),
            ("--sidebar-primary", primary.to_string()),
            (
                "--sidebar-primary-foreground",
                primary_foreground.to_string(),
            ),
            ("--sidebar-ring", primary.to_string()),
        ]
    }

    /// The raw property/value pairs this preset contributes when applied to
    /// the `--secondary`/`--muted` role group (matching `theme-builder`'s
    /// `set_secondary_palette`).
    fn secondary_pairs(self, dark: bool) -> [(&'static str, String); 4] {
        let (background, foreground) = self.surface_hsl(dark);
        [
            ("--secondary", background.to_string()),
            ("--secondary-foreground", foreground.to_string()),
            ("--muted", background.to_string()),
            ("--muted-foreground", foreground.to_string()),
        ]
    }

    /// The raw property/value pairs this preset contributes when applied to
    /// the `--accent`/`--sidebar-accent` role group (matching
    /// `theme-builder`'s `set_accent_palette`).
    fn accent_pairs(self, dark: bool) -> [(&'static str, String); 4] {
        let (background, foreground) = self.surface_hsl(dark);
        [
            ("--accent", background.to_string()),
            ("--accent-foreground", foreground.to_string()),
            ("--sidebar-accent", background.to_string()),
            ("--sidebar-accent-foreground", foreground.to_string()),
        ]
    }
}

/// Every raw custom property this component writes, across all three role
/// groups -- the hydration read-back request on mount asks for exactly
/// these.
const ALL_PROPERTY_NAMES: [&str; 14] = [
    "--primary",
    "--primary-foreground",
    "--ring",
    "--sidebar-primary",
    "--sidebar-primary-foreground",
    "--sidebar-ring",
    "--secondary",
    "--secondary-foreground",
    "--muted",
    "--muted-foreground",
    "--accent",
    "--accent-foreground",
    "--sidebar-accent",
    "--sidebar-accent-foreground",
];

/// The single coordinated preset every mounted `ThemeSwitcher` shares. A
/// `GlobalSignal` (matching `theme_mode.rs`'s own `MODE` convention) rather
/// than a per-mount `use_signal`, so a persistent sidebar instance and this
/// component's own demo-page instance -- or any two simultaneously-mounted
/// instances -- read and drive the same live selection instead of silently
/// diverging until one of them happens to re-hydrate from the DOM.
static PALETTE: GlobalSignal<ThemePalette> = Global::new(ThemePalette::default);

/// Looks up `name`'s freshly read-back value out of a `values` slice
/// positioned the same as [`ALL_PROPERTY_NAMES`].
fn read_value<'a>(values: &'a [String], name: &str) -> Option<&'a str> {
    ALL_PROPERTY_NAMES
        .iter()
        .position(|candidate| *candidate == name)
        .map(|index| values[index].as_str())
}

/// The single coordinated preset currently in effect across all three role
/// groups, read back from `values` (positioned like [`ALL_PROPERTY_NAMES`]),
/// or `None` if the roles don't all agree on the same preset (an
/// unavailable read, or a `theme-builder` edit that gave the roles
/// inconsistent values).
fn matching_combined_palette(values: &[String], dark: bool) -> Option<ThemePalette> {
    if values.len() != ALL_PROPERTY_NAMES.len() {
        return None;
    }
    let primary = read_value(values, "--primary")
        .zip(read_value(values, "--primary-foreground"))
        .and_then(|(background, foreground)| {
            ThemePalette::matching_primary(background, foreground, dark)
        })?;
    let secondary = read_value(values, "--secondary")
        .zip(read_value(values, "--secondary-foreground"))
        .and_then(|(background, foreground)| {
            ThemePalette::matching_surface(background, foreground, dark)
        })?;
    let accent = read_value(values, "--accent")
        .zip(read_value(values, "--accent-foreground"))
        .and_then(|(background, foreground)| {
            ThemePalette::matching_surface(background, foreground, dark)
        })?;
    (primary == secondary && secondary == accent).then_some(primary)
}

/// A single combined palette picker: one [`Select`] whose 6 options each set
/// the primary, secondary, and accent role groups together as one
/// coordinated preset. Applies the selected preset's role variables
/// immediately and re-applies them whenever the resolved light/dark
/// appearance changes (via the shared `theme_mode` signal), so a mode switch
/// and a palette switch compose correctly.
///
/// Deliberately has no `radius` prop: the `rounded-full` swatches are
/// per-preset color-swatch dots rendered in a loop -- circular is the
/// established convention for a selectable color dot (same reasoning as
/// `Skeleton`'s `Circle` variant), not a cosmetic choice.
#[component]
pub fn ThemeSwitcher(class: Option<String>) -> Element {
    let (mode, _set_mode) = use_persisted_theme_mode();
    let mut hydrated = use_signal(|| false);

    // One-shot hydration: reads the DOM exactly once, on mount, not on every
    // later mode change -- a re-read on every mode change was tried first
    // and reverted (see `read_root_properties`'s and `theme-builder`'s own
    // doc comments): with `read_root_properties` no longer clearing inline
    // values before reading, a later re-read would just keep echoing
    // whatever this component itself already applied, and it isn't needed
    // anyway -- the "apply" effect below already recomputes pure colors for
    // whichever appearance is now resolved from `PALETTE` alone, no fresh
    // DOM read required.
    use_effect(move || {
        if hydrated() {
            return;
        }
        let dark = mode().resolve() == ResolvedTheme::Dark;
        spawn(async move {
            let values = read_root_properties(&ALL_PROPERTY_NAMES).await;
            if let Some(found) = matching_combined_palette(&values, dark) {
                *PALETTE.write() = found;
            }
            hydrated.set(true);
        });
    });

    use_effect(move || {
        if !hydrated() {
            return;
        }
        let dark = mode().resolve() == ResolvedTheme::Dark;
        let current = *PALETTE.read();
        let mut pairs = Vec::with_capacity(ALL_PROPERTY_NAMES.len());
        pairs.extend(current.primary_pairs(dark));
        pairs.extend(current.secondary_pairs(dark));
        pairs.extend(current.accent_pairs(dark));
        apply_root_properties(&pairs);
    });

    let dark = mode().resolve() == ResolvedTheme::Dark;
    let value = use_memo(|| Some(*PALETTE.read()));

    rsx! {
        label {
            class: cn(&[
                "grid gap-1 text-xs font-medium text-muted-foreground",
                class.as_deref().unwrap_or_default(),
            ]),
            "Theme"
            Select::<ThemePalette> {
                value: ReadSignal::from(value),
                on_value_change: move |next: Option<ThemePalette>| {
                    *PALETTE.write() = next.unwrap_or_default()
                },
                SelectTrigger { class: "w-full", aria_label: "Theme palette",
                    div { class: "flex flex-1 items-center gap-2",
                        PaletteSwatch { palette: *PALETTE.read(), dark }
                        SelectValue { placeholder: "Choose a theme" }
                    }
                }
                SelectList { class: "w-full", aria_label: "Theme palette options",
                    for (index , option) in ThemePalette::ALL.into_iter().enumerate() {
                        SelectOption::<ThemePalette> {
                            index,
                            value: option,
                            text_value: option.label(),
                            div { class: "flex items-center gap-2",
                                PaletteSwatch { palette: option, dark }
                                span { "{option.label()}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Three small overlapping circles previewing one preset's primary,
/// secondary, and accent colors together -- the "overlapping swatches"
/// convention modern theme pickers use for a coordinated preview. The
/// secondary and accent circles both source from [`ThemePalette::surface_hsl`]
/// (that table's own background and foreground halves, respectively) since
/// `theme-builder`'s underlying data model gives those two role groups the
/// same pastel pair rather than two independent lookup tables -- this is a
/// faithful preview of what actually gets applied, not an approximation.
#[component]
fn PaletteSwatch(palette: ThemePalette, dark: bool) -> Element {
    let (primary, _) = palette.primary_hsl(dark);
    let (secondary, accent) = palette.surface_hsl(dark);
    let primary_style = format!("background-color: hsl({primary});");
    let secondary_style = format!("background-color: hsl({secondary});");
    let accent_style = format!("background-color: hsl({accent});");
    rsx! {
        span {
            class: "relative inline-flex h-5 w-10 shrink-0 items-center",
            "aria-hidden": "true",
            span {
                class: "absolute left-0 size-4 rounded-full ring-2 ring-background",
                style: "{primary_style}",
            }
            span {
                class: "absolute left-3 size-4 rounded-full ring-2 ring-background",
                style: "{secondary_style}",
            }
            span {
                class: "absolute left-6 size-4 rounded-full ring-2 ring-background",
                style: "{accent_style}",
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn combined_values(palette: ThemePalette, dark: bool) -> Vec<String> {
        let mut pairs = Vec::new();
        pairs.extend(palette.primary_pairs(dark));
        pairs.extend(palette.secondary_pairs(dark));
        pairs.extend(palette.accent_pairs(dark));
        ALL_PROPERTY_NAMES
            .iter()
            .map(|name| {
                pairs
                    .iter()
                    .find(|(candidate, _)| candidate == name)
                    .unwrap()
                    .1
                    .clone()
            })
            .collect()
    }

    #[test]
    fn every_palette_has_a_distinct_label() {
        let labels: Vec<_> = ThemePalette::ALL.iter().map(|p| p.label()).collect();
        for (index, label) in labels.iter().enumerate() {
            assert!(!labels[index + 1..].contains(label));
        }
    }

    #[test]
    fn light_and_dark_primary_pairs_differ() {
        for palette in ThemePalette::ALL {
            assert_ne!(palette.primary_hsl(false), palette.primary_hsl(true));
        }
    }

    #[test]
    fn light_and_dark_surface_pairs_differ() {
        for palette in ThemePalette::ALL {
            assert_ne!(palette.surface_hsl(false), palette.surface_hsl(true));
        }
    }

    #[test]
    fn every_palette_has_a_distinct_primary_and_surface_pair_per_appearance() {
        for dark in [false, true] {
            let primaries: Vec<_> = ThemePalette::ALL
                .iter()
                .map(|p| p.primary_hsl(dark))
                .collect();
            let surfaces: Vec<_> = ThemePalette::ALL
                .iter()
                .map(|p| p.surface_hsl(dark))
                .collect();
            for index in 0..primaries.len() {
                assert!(!primaries[index + 1..].contains(&primaries[index]));
                assert!(!surfaces[index + 1..].contains(&surfaces[index]));
            }
        }
    }

    #[test]
    fn matching_primary_and_surface_round_trip_every_preset() {
        for dark in [false, true] {
            for palette in ThemePalette::ALL {
                let (background, foreground) = palette.primary_hsl(dark);
                assert_eq!(
                    ThemePalette::matching_primary(background, foreground, dark),
                    Some(palette)
                );
                let (background, foreground) = palette.surface_hsl(dark);
                assert_eq!(
                    ThemePalette::matching_surface(background, foreground, dark),
                    Some(palette)
                );
            }
        }
    }

    #[test]
    fn matching_primary_is_none_for_an_unrecognized_value() {
        assert_eq!(
            ThemePalette::matching_primary("1 2% 3%", "4 5% 6%", false),
            None
        );
    }

    #[test]
    fn read_value_looks_up_by_name_regardless_of_position() {
        let values: Vec<String> = ALL_PROPERTY_NAMES
            .iter()
            .enumerate()
            .map(|(index, _)| index.to_string())
            .collect();
        assert_eq!(read_value(&values, "--sidebar-ring"), Some("5"));
        assert_eq!(read_value(&values, "--nonexistent"), None);
    }

    #[test]
    fn matching_combined_palette_adopts_a_fully_consistent_live_theme() {
        for dark in [false, true] {
            for palette in ThemePalette::ALL {
                let values = combined_values(palette, dark);
                assert_eq!(matching_combined_palette(&values, dark), Some(palette));
            }
        }
    }

    #[test]
    fn matching_combined_palette_is_none_when_roles_disagree() {
        let dark = false;
        let mut pairs = Vec::new();
        pairs.extend(ThemePalette::Slate.primary_pairs(dark));
        pairs.extend(ThemePalette::Blue.secondary_pairs(dark));
        pairs.extend(ThemePalette::Slate.accent_pairs(dark));
        let values: Vec<String> = ALL_PROPERTY_NAMES
            .iter()
            .map(|name| {
                pairs
                    .iter()
                    .find(|(candidate, _)| candidate == name)
                    .unwrap()
                    .1
                    .clone()
            })
            .collect();
        assert_eq!(matching_combined_palette(&values, dark), None);
    }

    #[test]
    fn matching_combined_palette_is_none_on_a_length_mismatch() {
        assert_eq!(matching_combined_palette(&[], false), None);
    }
}
