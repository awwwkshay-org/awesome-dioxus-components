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
//! **The selected preset is persisted**, through
//! [`adico_primitives::persisted_state::use_persisted_global`] under the
//! `adico-theme-palette` key (`localStorage` on `web`, a small preferences
//! file on `native`) -- the same generic primitive
//! `adico_primitives::theme_mode::use_persisted_theme_mode` itself is built
//! on, so the choice survives a reload the same way the light/dark/system
//! mode already does. The persisted store, not the DOM, is this component's
//! source of truth; every mounted instance (a persistent sidebar picker and
//! this component's own demo page, for example) shares one live selection
//! through the same module-level [`PALETTE`] `GlobalSignal`
//! `use_persisted_global` drives, so changing it in one place updates every
//! other mounted instance immediately, with no reload needed to notice.
//!
//! An earlier version instead read the live `--primary`/`--secondary`/
//! `--accent` values back off the document root once on mount and
//! reverse-matched them against the preset tables. That only ever recovered
//! a value already applied *in the same page session* -- a reload discards
//! it -- so it bought nothing a persisted store doesn't do strictly better,
//! and it's gone now.
//!
//! **A protection was deliberately dropped along with it.** The old
//! read-back was gated behind a one-shot `hydrated` flag specifically so
//! this component would *adopt* a still-mounted sibling editor's live colors
//! rather than stomp them. The apply effect below now writes the persisted
//! preset's inline properties immediately on every mount, so mounting a
//! `ThemeSwitcher` while a `theme-builder` has live per-token edits applied
//! overwrites the primary/secondary/accent subset of those edits. This is
//! the accepted trade, not a new regression: the direction that actually
//! matters -- `theme-builder` reading back whatever `theme-switcher` last
//! applied -- is unaffected and still works via `theme-builder`'s own,
//! separate DOM read-back; and a `theme-builder` edit is explicitly
//! transient (it has its own `use_drop` cleanup on unmount) while a
//! `theme-switcher` preset is a persisted, durable user setting.
//!
//! Accepted limitation, inherited from `use_persisted_global`: on `web` the
//! stored preset loads asynchronously after first mount, so a reload
//! briefly applies the default Slate before the persisted preset lands --
//! strictly better than the previous behavior, which lost the selection
//! entirely on every reload.
//!
//! Writes only the raw `--foo` custom properties, never the `--color-foo`
//! Tailwind aliases the installed `@theme` block already derives from them
//! live via `var()`. An earlier version also wrote `--color-primary`
//! inline, which freezes that alias at a static value and stops it tracking
//! further `--primary` changes -- the same defect fixed in `theme-builder`;
//! see that file's module doc comment and `theme_mode.rs`'s
//! `read_root_properties` doc comment for the full story.

use dioxus::prelude::*;

use adico_primitives::persisted_state::use_persisted_global;
use adico_primitives::theme_mode::{
    ResolvedTheme, apply_root_properties, use_persisted_theme_mode,
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

    /// The short, lowercase persistence token for this preset, matching
    /// `ThemeMode`'s own `"light"`/`"dark"`/`"system"` token convention --
    /// deliberately not derived from [`Self::label`] (whose casing is a
    /// display concern), so a future label-text change can never silently
    /// change the persisted storage format.
    const fn token(self) -> &'static str {
        match self {
            Self::Slate => "slate",
            Self::Blue => "blue",
            Self::Violet => "violet",
            Self::Emerald => "emerald",
            Self::Rose => "rose",
            Self::Amber => "amber",
        }
    }

    fn from_token(token: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.token() == token)
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

/// Distinct from `theme-mode`'s own `adico-theme-mode` key: the appearance
/// (light/dark/system) and the palette preset are two independent settings a
/// consumer sets separately, and both need to survive a reload on their own.
const PALETTE_STORAGE_KEY: &str = "adico-theme-palette";

/// The single coordinated preset every mounted `ThemeSwitcher` shares. A
/// `GlobalSignal` (matching `theme_mode.rs`'s own `MODE` convention) rather
/// than a per-mount `use_signal`, so a persistent sidebar instance and this
/// component's own demo-page instance -- or any two simultaneously-mounted
/// instances -- read and drive the same live selection instead of silently
/// diverging. Driven exclusively through
/// `adico_primitives::persisted_state::use_persisted_global` inside
/// [`ThemeSwitcher`] -- never write it directly (`*PALETTE.write() = ...`)
/// anywhere else, or the UI updates but the choice silently stops
/// persisting.
static PALETTE: GlobalSignal<ThemePalette> = Global::new(ThemePalette::default);

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
    let (palette, set_palette) = use_persisted_global(
        &PALETTE,
        PALETTE_STORAGE_KEY,
        ThemePalette::token,
        ThemePalette::from_token,
    );

    // Reads `mode()` and `palette()` *inside* the effect body (not as
    // captured plain values) so Dioxus's reactive tracking re-runs this on
    // either a mode change or a palette change -- the exact convention
    // `theme_mode.rs`'s own `apply_resolved_class` doc comment warns about:
    // an earlier version that passed the resolved value in as a plain
    // argument compiled and rendered fine but silently stopped re-applying
    // after first mount.
    use_effect(move || {
        let dark = mode().resolve() == ResolvedTheme::Dark;
        let current = palette();
        let mut pairs = Vec::new();
        pairs.extend(current.primary_pairs(dark));
        pairs.extend(current.secondary_pairs(dark));
        pairs.extend(current.accent_pairs(dark));
        apply_root_properties(&pairs);
    });

    let dark = mode().resolve() == ResolvedTheme::Dark;
    let value = use_memo(move || Some(palette()));

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
                    set_palette.call(next.unwrap_or_default())
                },
                SelectTrigger { class: "w-full", aria_label: "Theme palette",
                    div { class: "flex flex-1 items-center gap-2",
                        PaletteSwatch { palette: palette(), dark }
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
    fn palette_tokens_round_trip_every_preset() {
        for palette in ThemePalette::ALL {
            assert_eq!(ThemePalette::from_token(palette.token()), Some(palette));
        }
    }

    #[test]
    fn palette_tokens_are_distinct_lowercase_ascii() {
        let tokens: Vec<_> = ThemePalette::ALL.iter().map(|p| p.token()).collect();
        for (index, token) in tokens.iter().enumerate() {
            assert!(!tokens[index + 1..].contains(token));
            assert!(token.chars().all(|c| c.is_ascii_lowercase()));
        }
    }

    #[test]
    fn from_token_rejects_an_unrecognized_token() {
        // Locks in that persisted tokens are the lowercase form (`token()`,
        // not `label()`), so a future label-text change can't silently
        // change the persisted storage format.
        assert_eq!(ThemePalette::from_token(""), None);
        assert_eq!(ThemePalette::from_token("slat"), None);
        assert_eq!(ThemePalette::from_token("Slate"), None);
    }

    #[test]
    fn the_three_role_groups_apply_fourteen_distinct_properties() {
        let dark = false;
        let palette = ThemePalette::default();
        let mut names: Vec<&str> = Vec::new();
        names.extend(palette.primary_pairs(dark).map(|(name, _)| name));
        names.extend(palette.secondary_pairs(dark).map(|(name, _)| name));
        names.extend(palette.accent_pairs(dark).map(|(name, _)| name));
        assert_eq!(names.len(), 14);
        for (index, name) in names.iter().enumerate() {
            assert!(
                !names[index + 1..].contains(name),
                "{name} written by more than one role group"
            );
        }
    }
}
