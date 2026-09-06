//! Source-owned Dioxus-only theme customization builder for Dioxus (no
//! shadcn equivalent), backed by the owned adico primitive layer.
//!
//! Productizes the full 28-semantic-token editor, independent light/dark
//! values, deterministic "generate theme," and CSS export prototyped in
//! `apps/playground/src/theme.rs`'s advanced customization tray. Unlike
//! `theme-switcher` (which only distills the primary-color preset concept),
//! `theme-builder` ports that tray's complete token coverage, since none of
//! it is actually playground-specific in nature -- see design.md §7d.
//!
//! Applies its edited tokens live via
//! [`adico_primitives::theme_mode::apply_root_properties`], the same
//! document-root mechanism `theme-switcher` already uses for its 4
//! properties, so `ThemeBuilder`, `ThemeSwitcher`, and `ModeToggle` can be
//! mounted together and compose correctly. Its CSS export renders into a
//! read-only, selectable `<textarea>` rather than calling the browser
//! clipboard API directly -- a real one-click clipboard write would leak a
//! browser-interop detail into this registry item's source and require new
//! `web-sys`/`wasm-bindgen-futures` cargo dependencies no other registry
//! item needs; see design.md §7d.
//!
//! Unlike `theme-switcher` (which only ever touches 4 properties, so
//! whichever component last wrote them simply wins, an ordinary CSS
//! custom-property cascade), `ThemeBuilder` covers the *entire* semantic
//! token set, including `--background`/`--foreground` -- the same
//! properties `mode-toggle`'s `.dark` class selector defines. An inline
//! style always beats a class selector for the same property, so if
//! `ThemeBuilder` left its properties in place after unmounting, opening it
//! even once would permanently override `mode-toggle`'s dark-mode toggle
//! for the rest of the session. A `use_drop` cleanup below removes every
//! property this component applies as soon as it unmounts, handing control
//! back to `mode-toggle`/`theme-switcher`'s class-based mechanism.
//!
//! **Deliberately excluded (task 5.3d, 2026-09-01):** `--chart-1`..
//! `--chart-5`, added to the installed CSS contract's token set alongside
//! this component's original 28. Chart colors are a data-visualization
//! palette, not a UI-surface color in the same sense as the 28 tokens this
//! editor covers, and shadcn's own reference theme customizer does not
//! expose them for interactive editing either -- recorded as a deliberate
//! scope decision, not an oversight.
//!
//! **Hydrates from the live theme on mount**, rather than always starting
//! from a hardcoded Slate/Light default: it reads
//! [`use_persisted_theme_mode`] to learn which appearance is actually
//! resolved (so it opens already showing "Dark" if the app is dark) and
//! reads back each token's effective value with
//! [`adico_primitives::theme_mode::read_root_properties`], once, on mount.
//! This makes `ThemeBuilder` a *reader* of the persisted mode signal, and --
//! unlike the fully one-way relationship this component has with a
//! `theme-switcher`'s palette state -- also a *writer* of it: its own
//! `ThemeAppearanceControl` dropdown calls the persisted mode's setter too
//! (mapped straight to `ThemeMode::Light`/`Dark`, skipping `System`, which
//! this dropdown has no representation for), not just `selection`'s local
//! `appearance` field. Without this, picking "Light" here would only ever
//! change `ThemeBuilder`'s own inline overrides -- which its `use_drop`
//! cleanup removes the instant it unmounts -- while the real, persisted
//! mode stayed on whatever it was before, so closing the dialog (or
//! navigating away) would silently snap the appearance back to what
//! `mode-toggle` had it set to, discarding the choice the instant the
//! editor closed. A *separate*, ordinary effect keeps `appearance` synced
//! to the persisted mode on every *later* change too (so a `mode-toggle`
//! flip while `ThemeBuilder` stays mounted still switches which of
//! `light`/`dark` is active), but that resync never re-reads the DOM --
//! seeing why requires understanding why the read is one-shot in the first
//! place:
//!
//! Reading on every mode change (not just once) was tried first and
//! reverted. `read_root_properties` used to clear its own previously-applied
//! inline values before reading, since without that clear, a component that
//! re-reads after having already applied its own edits would just keep
//! echoing its own prior inline values forever (`getComputedStyle` prefers
//! an inline value over any class selector or `:root` rule for the same
//! property) -- a self-referential loop a mode change alone could never
//! break through. But *with* the clear, a later re-read was found to just as
//! readily wipe out a *different*, still-mounted editor's legitimate inline
//! value out from under it -- for example a persistent `theme-switcher` in
//! the playground's sidebar, whose applied palette would otherwise get
//! silently reset back to Slate the moment `ThemeBuilder` reads the DOM a
//! second time. Reading once, on mount, sidesteps both problems: nothing has
//! been applied by this instance yet, so there's no self-echo to guard
//! against, and no in-mount re-read at all means no chance to clobber a
//! sibling later. See `theme_mode.rs`'s doc comment on
//! `read_root_properties` for the full story. Applying the freshly read
//! values back to the document root before this one-shot hydration
//! completes is still guarded against, so an early render never briefly
//! stomps the live theme with hardcoded defaults. Inherits
//! `use_persisted_theme_mode`'s own accepted limitation (see
//! `theme_mode.rs`): a stored preference loads asynchronously, so the very
//! first paint can briefly resolve to the system default before the stored
//! choice lands -- and since hydration is one-shot, if that stored
//! preference lands *after* hydration already ran, only `appearance` (via
//! the separate resync effect above) catches up; the newly-active
//! appearance's token values and palette matches keep whatever this
//! component's own hardcoded defaults or an earlier edit already gave them.
//!
//! Hydration also reverse-matches the hydrated `--primary`/`--secondary`/
//! `--accent` (and their `-foreground`) values against [`Palette`]'s own
//! tables (`Palette::matching_primary`/`matching_surface`), so the
//! `PaletteControl` swatch rows open with the right preset highlighted --
//! for example, whatever a `theme-switcher` mounted elsewhere on the page
//! (the playground's persistent sidebar instance, say) already applied --
//! instead of always highlighting Slate while the *values* are actually
//! correct. A role whose live value doesn't exactly match any of the 6
//! presets (a direct per-token edit, for instance) simply leaves that row
//! showing Slate highlighted without touching the live color, the same
//! documented tradeoff `theme-switcher` makes for its own reverse-matching.

use dioxus::prelude::*;

use adico_primitives::theme_mode::{
    ResolvedTheme, ThemeMode, apply_root_properties, clear_root_properties, read_root_properties,
    use_persisted_theme_mode,
};

use super::copy_button::CopyButton;
use crate::adico_lib::cn::cn;

/// Which appearance `ThemeBuilder` is currently editing/previewing. This is
/// independent of the persisted `theme_mode` global signal `mode-toggle`
/// drives -- `ThemeBuilder` is an editing surface a consumer mounts
/// occasionally (for example behind a settings dialog), not an always-active
/// mode switch, so it owns its own light/dark selection.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ThemeAppearance {
    #[default]
    Light,
    Dark,
}

impl ThemeAppearance {
    const ALL: [Self; 2] = [Self::Light, Self::Dark];

    const fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    const fn value(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    fn from_value(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.value() == value)
    }
}

/// A palette preset applied to a semantic role group (primary, secondary, or
/// accent).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Palette {
    #[default]
    Slate,
    Blue,
    Violet,
    Emerald,
    Rose,
    Amber,
}

impl Palette {
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

    const fn primary_tokens(self, appearance: ThemeAppearance) -> ColorTokens {
        match (self, appearance) {
            (Self::Slate, ThemeAppearance::Light) => {
                ColorTokens::new("222.2 47.4% 11.2%", "210 40% 98%")
            }
            (Self::Blue, ThemeAppearance::Light) => {
                ColorTokens::new("221.2 83.2% 53.3%", "210 40% 98%")
            }
            (Self::Violet, ThemeAppearance::Light) => {
                ColorTokens::new("262.1 83.3% 57.8%", "210 40% 98%")
            }
            (Self::Emerald, ThemeAppearance::Light) => {
                ColorTokens::new("160.1 84.1% 39.4%", "210 40% 98%")
            }
            (Self::Rose, ThemeAppearance::Light) => {
                ColorTokens::new("346.8 77.2% 49.8%", "210 40% 98%")
            }
            (Self::Amber, ThemeAppearance::Light) => {
                ColorTokens::new("37.7 92.1% 50.2%", "26 83.3% 14.1%")
            }
            (Self::Slate, ThemeAppearance::Dark) => {
                ColorTokens::new("210 40% 98%", "222.2 47.4% 11.2%")
            }
            (Self::Blue, ThemeAppearance::Dark) => {
                ColorTokens::new("213.1 93.9% 67.8%", "222.2 47.4% 11.2%")
            }
            (Self::Violet, ThemeAppearance::Dark) => {
                ColorTokens::new("263.4 70% 50.4%", "0 0% 100%")
            }
            (Self::Emerald, ThemeAppearance::Dark) => {
                ColorTokens::new("158.1 64.4% 51.6%", "2.7 19.3% 10.2%")
            }
            (Self::Rose, ThemeAppearance::Dark) => {
                ColorTokens::new("349.7 89.2% 60.2%", "0 0% 100%")
            }
            (Self::Amber, ThemeAppearance::Dark) => {
                ColorTokens::new("47.9 95.8% 53.1%", "26 83.3% 14.1%")
            }
        }
    }

    const fn surface_tokens(self, appearance: ThemeAppearance) -> ColorTokens {
        match (self, appearance) {
            (Self::Slate, ThemeAppearance::Light) => {
                ColorTokens::new("210 40% 96.1%", "222.2 47.4% 11.2%")
            }
            (Self::Blue, ThemeAppearance::Light) => {
                ColorTokens::new("214.3 94.6% 92.7%", "221.2 83.2% 29.4%")
            }
            (Self::Violet, ThemeAppearance::Light) => {
                ColorTokens::new("250 100% 95.3%", "262.1 83.3% 30%")
            }
            (Self::Emerald, ThemeAppearance::Light) => {
                ColorTokens::new("152.4 76% 92.2%", "161.4 93.5% 16.9%")
            }
            (Self::Rose, ThemeAppearance::Light) => {
                ColorTokens::new("355.6 100% 94.7%", "343.4 79.7% 25.7%")
            }
            (Self::Amber, ThemeAppearance::Light) => {
                ColorTokens::new("48 96.5% 88.8%", "26 83.3% 14.1%")
            }
            (Self::Slate, ThemeAppearance::Dark) => {
                ColorTokens::new("217.2 32.6% 17.5%", "210 40% 98%")
            }
            (Self::Blue, ThemeAppearance::Dark) => {
                ColorTokens::new("217.2 32.6% 17.5%", "219.4 100% 92%")
            }
            (Self::Violet, ThemeAppearance::Dark) => {
                ColorTokens::new("263.4 38.6% 17.8%", "250 100% 92%")
            }
            (Self::Emerald, ThemeAppearance::Dark) => {
                ColorTokens::new("163.1 36.7% 16.1%", "149.3 80.4% 90%")
            }
            (Self::Rose, ThemeAppearance::Dark) => {
                ColorTokens::new("343.4 43.8% 16.1%", "355.6 100% 94.7%")
            }
            (Self::Amber, ThemeAppearance::Dark) => {
                ColorTokens::new("30 47.8% 16.1%", "48 96.5% 88.8%")
            }
        }
    }

    /// The preset (if any) whose [`Self::primary_tokens`] exactly matches
    /// `(background, foreground)` for the given appearance. Lets hydration
    /// tell which preset (if any) is already live -- for example one a
    /// `theme-switcher` mounted elsewhere on the page just applied -- so the
    /// primary `PaletteControl` opens with the right swatch highlighted
    /// instead of always Slate.
    fn matching_primary(
        background: &str,
        foreground: &str,
        appearance: ThemeAppearance,
    ) -> Option<Self> {
        Self::ALL.into_iter().find(|palette| {
            let tokens = palette.primary_tokens(appearance);
            tokens.background == background && tokens.foreground == foreground
        })
    }

    /// The preset (if any) whose [`Self::surface_tokens`] exactly matches
    /// `(background, foreground)` for the given appearance. See
    /// [`Self::matching_primary`] -- same purpose, for the secondary/accent
    /// `PaletteControl`s.
    fn matching_surface(
        background: &str,
        foreground: &str,
        appearance: ThemeAppearance,
    ) -> Option<Self> {
        Self::ALL.into_iter().find(|palette| {
            let tokens = palette.surface_tokens(appearance);
            tokens.background == background && tokens.foreground == foreground
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ColorTokens {
    background: &'static str,
    foreground: &'static str,
}

impl ColorTokens {
    const fn new(background: &'static str, foreground: &'static str) -> Self {
        Self {
            background,
            foreground,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ThemeToken {
    Background,
    Foreground,
    Card,
    CardForeground,
    Popover,
    PopoverForeground,
    Primary,
    PrimaryForeground,
    Secondary,
    SecondaryForeground,
    Muted,
    MutedForeground,
    Accent,
    AccentForeground,
    Destructive,
    DestructiveForeground,
    Border,
    Input,
    Ring,
    Radius,
    SidebarBackground,
    SidebarForeground,
    SidebarPrimary,
    SidebarPrimaryForeground,
    SidebarAccent,
    SidebarAccentForeground,
    SidebarBorder,
    SidebarRing,
}

impl ThemeToken {
    const fn label(self) -> &'static str {
        match self {
            Self::Background => "--background",
            Self::Foreground => "--foreground",
            Self::Card => "--card",
            Self::CardForeground => "--card-foreground",
            Self::Popover => "--popover",
            Self::PopoverForeground => "--popover-foreground",
            Self::Primary => "--primary",
            Self::PrimaryForeground => "--primary-foreground",
            Self::Secondary => "--secondary",
            Self::SecondaryForeground => "--secondary-foreground",
            Self::Muted => "--muted",
            Self::MutedForeground => "--muted-foreground",
            Self::Accent => "--accent",
            Self::AccentForeground => "--accent-foreground",
            Self::Destructive => "--destructive",
            Self::DestructiveForeground => "--destructive-foreground",
            Self::Border => "--border",
            Self::Input => "--input",
            Self::Ring => "--ring",
            Self::Radius => "--radius",
            Self::SidebarBackground => "--sidebar",
            Self::SidebarForeground => "--sidebar-foreground",
            Self::SidebarPrimary => "--sidebar-primary",
            Self::SidebarPrimaryForeground => "--sidebar-primary-foreground",
            Self::SidebarAccent => "--sidebar-accent",
            Self::SidebarAccentForeground => "--sidebar-accent-foreground",
            Self::SidebarBorder => "--sidebar-border",
            Self::SidebarRing => "--sidebar-ring",
        }
    }

    const fn is_color(self) -> bool {
        !matches!(self, Self::Radius)
    }
}

const SURFACE_TOKENS: &[ThemeToken] = &[
    ThemeToken::Background,
    ThemeToken::Foreground,
    ThemeToken::Card,
    ThemeToken::CardForeground,
    ThemeToken::Popover,
    ThemeToken::PopoverForeground,
];
const ROLE_TOKENS: &[ThemeToken] = &[
    ThemeToken::Primary,
    ThemeToken::PrimaryForeground,
    ThemeToken::Secondary,
    ThemeToken::SecondaryForeground,
    ThemeToken::Muted,
    ThemeToken::MutedForeground,
    ThemeToken::Accent,
    ThemeToken::AccentForeground,
    ThemeToken::Destructive,
    ThemeToken::DestructiveForeground,
];
const STRUCTURAL_TOKENS: &[ThemeToken] = &[
    ThemeToken::Border,
    ThemeToken::Input,
    ThemeToken::Ring,
    ThemeToken::Radius,
];
const SIDEBAR_TOKENS: &[ThemeToken] = &[
    ThemeToken::SidebarBackground,
    ThemeToken::SidebarForeground,
    ThemeToken::SidebarPrimary,
    ThemeToken::SidebarPrimaryForeground,
    ThemeToken::SidebarAccent,
    ThemeToken::SidebarAccentForeground,
    ThemeToken::SidebarBorder,
    ThemeToken::SidebarRing,
];

#[derive(Clone, Copy)]
struct ThemeGroup {
    label: &'static str,
    tokens: &'static [ThemeToken],
}

const THEME_GROUPS: [ThemeGroup; 4] = [
    ThemeGroup {
        label: "Surfaces",
        tokens: SURFACE_TOKENS,
    },
    ThemeGroup {
        label: "Roles",
        tokens: ROLE_TOKENS,
    },
    ThemeGroup {
        label: "Structure",
        tokens: STRUCTURAL_TOKENS,
    },
    ThemeGroup {
        label: "Sidebar",
        tokens: SIDEBAR_TOKENS,
    },
];

/// Every [`ThemeToken`] this editor covers, in `THEME_GROUPS` order. Used to
/// build the `getComputedStyle` read-back request on mount -- see
/// [`hydrate_tokens`].
fn all_theme_tokens() -> Vec<ThemeToken> {
    THEME_GROUPS
        .iter()
        .flat_map(|group| group.tokens.iter().copied())
        .collect()
}

/// Adopts freshly read-back values into `active`, one per `tokens[i]` /
/// `values[i]` pair. A no-op if `values` doesn't have exactly one entry per
/// token (an unavailable/failed read, per [`read_root_properties`]'s
/// contract), and leaves any individually-empty value untouched rather than
/// clobbering `active`'s existing value with an empty string.
fn hydrate_tokens(active: &mut ThemeVariables, tokens: &[ThemeToken], values: &[String]) {
    if values.len() != tokens.len() {
        return;
    }
    for (token, value) in tokens.iter().zip(values) {
        if !value.is_empty() {
            active.set(*token, value.clone());
        }
    }
}

/// The complete set of semantic theme tokens for one appearance (light or
/// dark). This is the payload shape [`ThemeBuilder`]'s `on_theme_change`
/// callback delivers, so a consumer can persist or react to edits
/// programmatically instead of only copy-pasting the CSS export.
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeVariables {
    pub background: String,
    pub foreground: String,
    pub card: String,
    pub card_foreground: String,
    pub popover: String,
    pub popover_foreground: String,
    pub primary: String,
    pub primary_foreground: String,
    pub secondary: String,
    pub secondary_foreground: String,
    pub muted: String,
    pub muted_foreground: String,
    pub accent: String,
    pub accent_foreground: String,
    pub destructive: String,
    pub destructive_foreground: String,
    pub border: String,
    pub input: String,
    pub ring: String,
    pub radius: String,
    pub sidebar_background: String,
    pub sidebar_foreground: String,
    pub sidebar_primary: String,
    pub sidebar_primary_foreground: String,
    pub sidebar_accent: String,
    pub sidebar_accent_foreground: String,
    pub sidebar_border: String,
    pub sidebar_ring: String,
}

impl ThemeVariables {
    fn light() -> Self {
        Self {
            background: "0 0% 100%".into(),
            foreground: "222.2 84% 4.9%".into(),
            card: "0 0% 100%".into(),
            card_foreground: "222.2 84% 4.9%".into(),
            popover: "0 0% 100%".into(),
            popover_foreground: "222.2 84% 4.9%".into(),
            primary: "222.2 47.4% 11.2%".into(),
            primary_foreground: "210 40% 98%".into(),
            secondary: "210 40% 96.1%".into(),
            secondary_foreground: "222.2 47.4% 11.2%".into(),
            muted: "210 40% 96.1%".into(),
            muted_foreground: "215.4 16.3% 46.9%".into(),
            accent: "210 40% 96.1%".into(),
            accent_foreground: "222.2 47.4% 11.2%".into(),
            destructive: "0 84.2% 60.2%".into(),
            destructive_foreground: "210 40% 98%".into(),
            border: "214.3 31.8% 91.4%".into(),
            input: "214.3 31.8% 91.4%".into(),
            ring: "222.2 84% 4.9%".into(),
            radius: "0.5rem".into(),
            sidebar_background: "0 0% 98%".into(),
            sidebar_foreground: "240 5.3% 26.1%".into(),
            sidebar_primary: "240 5.9% 10%".into(),
            sidebar_primary_foreground: "0 0% 98%".into(),
            sidebar_accent: "240 4.8% 95.9%".into(),
            sidebar_accent_foreground: "240 5.9% 10%".into(),
            sidebar_border: "220 13% 91%".into(),
            sidebar_ring: "217.2 91.2% 59.8%".into(),
        }
    }

    fn dark() -> Self {
        Self {
            background: "222.2 84% 4.9%".into(),
            foreground: "210 40% 98%".into(),
            card: "222.2 84% 4.9%".into(),
            card_foreground: "210 40% 98%".into(),
            popover: "222.2 84% 4.9%".into(),
            popover_foreground: "210 40% 98%".into(),
            primary: "210 40% 98%".into(),
            primary_foreground: "222.2 47.4% 11.2%".into(),
            secondary: "217.2 32.6% 17.5%".into(),
            secondary_foreground: "210 40% 98%".into(),
            muted: "217.2 32.6% 17.5%".into(),
            muted_foreground: "215 20.2% 65.1%".into(),
            accent: "217.2 32.6% 17.5%".into(),
            accent_foreground: "210 40% 98%".into(),
            destructive: "0 62.8% 30.6%".into(),
            destructive_foreground: "210 40% 98%".into(),
            border: "217.2 32.6% 17.5%".into(),
            input: "217.2 32.6% 17.5%".into(),
            ring: "212.7 26.8% 83.9%".into(),
            radius: "0.5rem".into(),
            sidebar_background: "240 5.9% 10%".into(),
            sidebar_foreground: "240 4.8% 95.9%".into(),
            sidebar_primary: "224.3 76.3% 48%".into(),
            sidebar_primary_foreground: "0 0% 100%".into(),
            sidebar_accent: "240 3.7% 15.9%".into(),
            sidebar_accent_foreground: "240 4.8% 95.9%".into(),
            sidebar_border: "240 3.7% 15.9%".into(),
            sidebar_ring: "217.2 91.2% 59.8%".into(),
        }
    }

    fn get(&self, token: ThemeToken) -> &str {
        match token {
            ThemeToken::Background => &self.background,
            ThemeToken::Foreground => &self.foreground,
            ThemeToken::Card => &self.card,
            ThemeToken::CardForeground => &self.card_foreground,
            ThemeToken::Popover => &self.popover,
            ThemeToken::PopoverForeground => &self.popover_foreground,
            ThemeToken::Primary => &self.primary,
            ThemeToken::PrimaryForeground => &self.primary_foreground,
            ThemeToken::Secondary => &self.secondary,
            ThemeToken::SecondaryForeground => &self.secondary_foreground,
            ThemeToken::Muted => &self.muted,
            ThemeToken::MutedForeground => &self.muted_foreground,
            ThemeToken::Accent => &self.accent,
            ThemeToken::AccentForeground => &self.accent_foreground,
            ThemeToken::Destructive => &self.destructive,
            ThemeToken::DestructiveForeground => &self.destructive_foreground,
            ThemeToken::Border => &self.border,
            ThemeToken::Input => &self.input,
            ThemeToken::Ring => &self.ring,
            ThemeToken::Radius => &self.radius,
            ThemeToken::SidebarBackground => &self.sidebar_background,
            ThemeToken::SidebarForeground => &self.sidebar_foreground,
            ThemeToken::SidebarPrimary => &self.sidebar_primary,
            ThemeToken::SidebarPrimaryForeground => &self.sidebar_primary_foreground,
            ThemeToken::SidebarAccent => &self.sidebar_accent,
            ThemeToken::SidebarAccentForeground => &self.sidebar_accent_foreground,
            ThemeToken::SidebarBorder => &self.sidebar_border,
            ThemeToken::SidebarRing => &self.sidebar_ring,
        }
    }

    fn set(&mut self, token: ThemeToken, value: String) {
        match token {
            ThemeToken::Background => self.background = value,
            ThemeToken::Foreground => self.foreground = value,
            ThemeToken::Card => self.card = value,
            ThemeToken::CardForeground => self.card_foreground = value,
            ThemeToken::Popover => self.popover = value,
            ThemeToken::PopoverForeground => self.popover_foreground = value,
            ThemeToken::Primary => self.primary = value,
            ThemeToken::PrimaryForeground => self.primary_foreground = value,
            ThemeToken::Secondary => self.secondary = value,
            ThemeToken::SecondaryForeground => self.secondary_foreground = value,
            ThemeToken::Muted => self.muted = value,
            ThemeToken::MutedForeground => self.muted_foreground = value,
            ThemeToken::Accent => self.accent = value,
            ThemeToken::AccentForeground => self.accent_foreground = value,
            ThemeToken::Destructive => self.destructive = value,
            ThemeToken::DestructiveForeground => self.destructive_foreground = value,
            ThemeToken::Border => self.border = value,
            ThemeToken::Input => self.input = value,
            ThemeToken::Ring => self.ring = value,
            ThemeToken::Radius => self.radius = value,
            ThemeToken::SidebarBackground => self.sidebar_background = value,
            ThemeToken::SidebarForeground => self.sidebar_foreground = value,
            ThemeToken::SidebarPrimary => self.sidebar_primary = value,
            ThemeToken::SidebarPrimaryForeground => self.sidebar_primary_foreground = value,
            ThemeToken::SidebarAccent => self.sidebar_accent = value,
            ThemeToken::SidebarAccentForeground => self.sidebar_accent_foreground = value,
            ThemeToken::SidebarBorder => self.sidebar_border = value,
            ThemeToken::SidebarRing => self.sidebar_ring = value,
        }
    }

    /// The token pairs [`apply_root_properties`]/[`read_root_properties`]
    /// need to apply or read back this appearance on the document root.
    ///
    /// Deliberately only the 28 raw `--foo` custom properties, **not** the
    /// `--color-foo` Tailwind aliases the installed `@theme` block derives
    /// from them (`--color-primary: hsl(var(--primary))`, etc.): `var()`
    /// lookups are live, so setting `--primary` here already updates
    /// `--color-primary` everywhere it's used, with no separate write
    /// needed. Verified live: inline-setting `--color-primary` itself (as
    /// an earlier version of this function did) freezes it as a static
    /// value that stops tracking `--primary`, which both doubles the
    /// properties this component has to manage and defeats the very
    /// liveness `read_root_properties` depends on to see a change.
    fn root_property_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("--background", self.background.clone()),
            ("--foreground", self.foreground.clone()),
            ("--card", self.card.clone()),
            ("--card-foreground", self.card_foreground.clone()),
            ("--popover", self.popover.clone()),
            ("--popover-foreground", self.popover_foreground.clone()),
            ("--primary", self.primary.clone()),
            ("--primary-foreground", self.primary_foreground.clone()),
            ("--secondary", self.secondary.clone()),
            ("--secondary-foreground", self.secondary_foreground.clone()),
            ("--muted", self.muted.clone()),
            ("--muted-foreground", self.muted_foreground.clone()),
            ("--accent", self.accent.clone()),
            ("--accent-foreground", self.accent_foreground.clone()),
            ("--destructive", self.destructive.clone()),
            (
                "--destructive-foreground",
                self.destructive_foreground.clone(),
            ),
            ("--border", self.border.clone()),
            ("--input", self.input.clone()),
            ("--ring", self.ring.clone()),
            ("--radius", self.radius.clone()),
            ("--sidebar", self.sidebar_background.clone()),
            ("--sidebar-foreground", self.sidebar_foreground.clone()),
            ("--sidebar-primary", self.sidebar_primary.clone()),
            (
                "--sidebar-primary-foreground",
                self.sidebar_primary_foreground.clone(),
            ),
            ("--sidebar-accent", self.sidebar_accent.clone()),
            (
                "--sidebar-accent-foreground",
                self.sidebar_accent_foreground.clone(),
            ),
            ("--sidebar-border", self.sidebar_border.clone()),
            ("--sidebar-ring", self.sidebar_ring.clone()),
        ]
    }

    fn css_declarations(&self) -> String {
        let tokens = [
            ("--background", self.background.as_str()),
            ("--foreground", self.foreground.as_str()),
            ("--card", self.card.as_str()),
            ("--card-foreground", self.card_foreground.as_str()),
            ("--popover", self.popover.as_str()),
            ("--popover-foreground", self.popover_foreground.as_str()),
            ("--primary", self.primary.as_str()),
            ("--primary-foreground", self.primary_foreground.as_str()),
            ("--secondary", self.secondary.as_str()),
            ("--secondary-foreground", self.secondary_foreground.as_str()),
            ("--muted", self.muted.as_str()),
            ("--muted-foreground", self.muted_foreground.as_str()),
            ("--accent", self.accent.as_str()),
            ("--accent-foreground", self.accent_foreground.as_str()),
            ("--destructive", self.destructive.as_str()),
            (
                "--destructive-foreground",
                self.destructive_foreground.as_str(),
            ),
            ("--border", self.border.as_str()),
            ("--input", self.input.as_str()),
            ("--ring", self.ring.as_str()),
            ("--radius", self.radius.as_str()),
            ("--sidebar", self.sidebar_background.as_str()),
            ("--sidebar-foreground", self.sidebar_foreground.as_str()),
            ("--sidebar-primary", self.sidebar_primary.as_str()),
            (
                "--sidebar-primary-foreground",
                self.sidebar_primary_foreground.as_str(),
            ),
            ("--sidebar-accent", self.sidebar_accent.as_str()),
            (
                "--sidebar-accent-foreground",
                self.sidebar_accent_foreground.as_str(),
            ),
            ("--sidebar-border", self.sidebar_border.as_str()),
            ("--sidebar-ring", self.sidebar_ring.as_str()),
        ];

        tokens
            .into_iter()
            .map(|(name, value)| format!("  {name}: {value};"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ThemeSelection {
    appearance: ThemeAppearance,
    primary_palette: Palette,
    secondary_palette: Palette,
    accent_palette: Palette,
    light: ThemeVariables,
    dark: ThemeVariables,
    random_seed: u64,
}

impl Default for ThemeSelection {
    fn default() -> Self {
        Self {
            appearance: ThemeAppearance::Light,
            primary_palette: Palette::Slate,
            secondary_palette: Palette::Slate,
            accent_palette: Palette::Slate,
            light: ThemeVariables::light(),
            dark: ThemeVariables::dark(),
            random_seed: 1,
        }
    }
}

impl ThemeSelection {
    fn css_export(&self) -> String {
        let selector = match self.appearance {
            ThemeAppearance::Light => ":root",
            ThemeAppearance::Dark => ".dark",
        };
        format!(
            "{selector} {{\n{}\n}}\n",
            self.active_tokens().css_declarations()
        )
    }

    fn active_tokens(&self) -> &ThemeVariables {
        match self.appearance {
            ThemeAppearance::Light => &self.light,
            ThemeAppearance::Dark => &self.dark,
        }
    }

    fn active_tokens_mut(&mut self) -> &mut ThemeVariables {
        match self.appearance {
            ThemeAppearance::Light => &mut self.light,
            ThemeAppearance::Dark => &mut self.dark,
        }
    }

    fn tokens_mut_for(&mut self, appearance: ThemeAppearance) -> &mut ThemeVariables {
        match appearance {
            ThemeAppearance::Light => &mut self.light,
            ThemeAppearance::Dark => &mut self.dark,
        }
    }

    fn set_primary_palette(&mut self, palette: Palette) {
        self.primary_palette = palette;
        for appearance in ThemeAppearance::ALL {
            let colors = palette.primary_tokens(appearance);
            let tokens = self.tokens_mut_for(appearance);
            tokens.primary = colors.background.into();
            tokens.primary_foreground = colors.foreground.into();
            tokens.ring = colors.background.into();
            tokens.sidebar_primary = colors.background.into();
            tokens.sidebar_primary_foreground = colors.foreground.into();
            tokens.sidebar_ring = colors.background.into();
        }
    }

    fn set_secondary_palette(&mut self, palette: Palette) {
        self.secondary_palette = palette;
        for appearance in ThemeAppearance::ALL {
            let colors = palette.surface_tokens(appearance);
            let tokens = self.tokens_mut_for(appearance);
            tokens.secondary = colors.background.into();
            tokens.secondary_foreground = colors.foreground.into();
            tokens.muted = colors.background.into();
            tokens.muted_foreground = colors.foreground.into();
        }
    }

    fn set_accent_palette(&mut self, palette: Palette) {
        self.accent_palette = palette;
        for appearance in ThemeAppearance::ALL {
            let colors = palette.surface_tokens(appearance);
            let tokens = self.tokens_mut_for(appearance);
            tokens.accent = colors.background.into();
            tokens.accent_foreground = colors.foreground.into();
            tokens.sidebar_accent = colors.background.into();
            tokens.sidebar_accent_foreground = colors.foreground.into();
        }
    }

    fn generate_theme(&mut self) {
        self.random_seed = self
            .random_seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let mut state = self.random_seed;
        let primary = Palette::ALL[next_palette_index(&mut state)];
        let secondary = Palette::ALL[next_palette_index(&mut state)];
        let accent = Palette::ALL[next_palette_index(&mut state)];
        self.random_seed = state;
        self.set_primary_palette(primary);
        self.set_secondary_palette(secondary);
        self.set_accent_palette(accent);
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

fn next_palette_index(state: &mut u64) -> usize {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    ((*state >> 32) as usize) % Palette::ALL.len()
}

/// A self-contained theme customization builder: a full 28-semantic-token
/// editor with independent light/dark values, palette presets, a
/// deterministic "generate theme" action, and a CSS export. Applies its
/// edited tokens live to the document root via [`apply_root_properties`], so
/// it composes with `mode-toggle`/`theme-switcher` on the same mechanism.
///
/// `ThemeBuilder` owns its own light/dark appearance selection once mounted
/// (the `ThemeAppearanceControl` dropdown below edits `selection.appearance`
/// directly, for its own live preview and CSS export), since it's an editing
/// surface a consumer mounts occasionally (for example behind a settings
/// dialog), not an always-active mode switch like `mode-toggle`. But that
/// dropdown *also* writes the real persisted `theme_mode` signal (mapped to
/// `ThemeMode::Light`/`Dark`), the same signal `mode-toggle` drives --
/// without that, the choice would only ever live in this component's own
/// inline overrides, which get removed the instant it unmounts, so closing
/// the dialog would silently revert the appearance to whatever the persisted
/// mode still said. This component also *reads* that signal, once on mount
/// and again on any later change, to seed/re-sync which appearance it opens
/// showing (see the module doc comment's "Hydrates from the live theme"
/// section) -- so it ends up both a reader and a writer, unlike its
/// strictly one-way relationship with a `theme-switcher`'s own palette
/// state, which this component never touches at all.
///
/// Deliberately has no `radius` prop: every `rounded-md` in this file is
/// internal preview-swatch/mockup chrome inside the editor's own control
/// panel, not the panel's own bounded surface. `ThemeVariables::radius`
/// (`String`, edited live through this component) is the `--radius`
/// CSS-value editor — an unrelated concept, not to be confused with the
/// shared `Radius` enum.
#[component]
pub fn ThemeBuilder(
    #[props(default)] on_theme_change: Callback<ThemeVariables>,
    class: Option<String>,
) -> Element {
    let mut selection = use_signal(ThemeSelection::default);
    let mut hydrated = use_signal(|| false);
    let (mode, set_mode) = use_persisted_theme_mode();

    // Keeps `appearance` synced to the persisted mode on every later change
    // too (no DOM read here -- just which of `light`/`dark` is "active"),
    // so a mode-toggle flip while `ThemeBuilder` is mounted still re-syncs
    // its own appearance without needing to re-hydrate from the DOM.
    use_effect(move || {
        let appearance = match mode().resolve() {
            ResolvedTheme::Light => ThemeAppearance::Light,
            ResolvedTheme::Dark => ThemeAppearance::Dark,
        };
        selection.with_mut(|sel| sel.appearance = appearance);
    });

    // One-shot hydration: reads the DOM exactly once, on mount, not on every
    // later mode change. Re-reading on every mode change was tried first and
    // reverted: `read_root_properties` no longer clears inline values before
    // reading (see its own doc comment), and without that clear a re-read
    // would just keep echoing whatever this component itself last applied --
    // but *with* the clear, a later re-read was found to just as readily
    // wipe out a *different*, still-mounted editor's legitimate inline value
    // (a `theme-switcher` in the sidebar, say) out from under it. A single
    // read at mount has neither problem: nothing has been applied yet by
    // this instance, so there's no self-echo to guard against, and no
    // in-mount re-read means no chance to clobber a sibling later.
    use_effect(move || {
        if hydrated() {
            return;
        }
        let appearance = match mode().resolve() {
            ResolvedTheme::Light => ThemeAppearance::Light,
            ResolvedTheme::Dark => ThemeAppearance::Dark,
        };
        spawn(async move {
            let tokens = all_theme_tokens();
            let names: Vec<&str> = tokens.iter().map(|token| token.label()).collect();
            let values = read_root_properties(&names).await;
            selection.with_mut(|sel| {
                sel.appearance = appearance;
                hydrate_tokens(sel.tokens_mut_for(appearance), &tokens, &values);
                // Reverse-match which preset (if any) the freshly hydrated
                // values correspond to, so the palette swatch rows open
                // showing what's actually live -- for example a preset a
                // `theme-switcher` mounted elsewhere on the page just
                // applied -- instead of always Slate. `.clone()` ends the
                // borrow from `tokens_mut_for` above before `sel`'s other
                // fields are written below.
                let active = sel.active_tokens().clone();
                if let Some(found) = Palette::matching_primary(
                    &active.primary,
                    &active.primary_foreground,
                    appearance,
                ) {
                    sel.primary_palette = found;
                }
                if let Some(found) = Palette::matching_surface(
                    &active.secondary,
                    &active.secondary_foreground,
                    appearance,
                ) {
                    sel.secondary_palette = found;
                }
                if let Some(found) =
                    Palette::matching_surface(&active.accent, &active.accent_foreground, appearance)
                {
                    sel.accent_palette = found;
                }
            });
            hydrated.set(true);
        });
    });

    use_effect(move || {
        if !hydrated() {
            return;
        }
        let current = selection();
        apply_root_properties(&current.active_tokens().root_property_pairs());
        on_theme_change.call(current.active_tokens().clone());
    });

    use_drop(move || {
        let property_names: Vec<&str> = ThemeVariables::light()
            .root_property_pairs()
            .iter()
            .map(|(name, _)| *name)
            .collect();
        clear_root_properties(&property_names);
    });

    let current = selection();
    let active_tokens = current.active_tokens().clone();
    let css_export = current.css_export();
    let export_label = format!("{} CSS variables", current.appearance.label());

    rsx! {
        div {
            class: cn(&["space-y-3", class.as_deref().unwrap_or_default()]),
            ThemeAppearanceControl {
                value: current.appearance,
                on_change: move |appearance: ThemeAppearance| {
                    selection.write().appearance = appearance;
                    set_mode
                        .call(match appearance {
                            ThemeAppearance::Light => ThemeMode::Light,
                            ThemeAppearance::Dark => ThemeMode::Dark,
                        });
                },
            }
            PaletteControl {
                label: "Primary",
                appearance: current.appearance,
                primary_role: true,
                value: current.primary_palette,
                on_change: move |palette| selection.write().set_primary_palette(palette),
            }
            PaletteControl {
                label: "Secondary",
                appearance: current.appearance,
                primary_role: false,
                value: current.secondary_palette,
                on_change: move |palette| selection.write().set_secondary_palette(palette),
            }
            PaletteControl {
                label: "Accent",
                appearance: current.appearance,
                primary_role: false,
                value: current.accent_palette,
                on_change: move |palette| selection.write().set_accent_palette(palette),
            }
            ActiveRolePreview {
                primary: current.primary_palette,
                secondary: current.secondary_palette,
                accent: current.accent_palette,
            }
            div { class: "grid gap-2 sm:grid-cols-2",
                button {
                    class: "rounded-md bg-primary px-2 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90",
                    r#type: "button",
                    onclick: move |_| selection.write().generate_theme(),
                    "Generate theme"
                }
                button {
                    class: "rounded-md border border-input bg-background px-2 py-1.5 text-sm font-medium text-foreground hover:bg-accent hover:text-accent-foreground",
                    r#type: "button",
                    onclick: move |_| selection.write().reset(),
                    "Reset theme"
                }
            }
            for group in THEME_GROUPS {
                details { class: "rounded-md border border-border p-2",
                    summary { class: "cursor-pointer text-xs font-semibold", "{group.label}" }
                    div { class: "mt-2 grid gap-2",
                        for token in group.tokens {
                            SemanticTokenControl {
                                token: *token,
                                value: active_tokens.get(*token).to_owned(),
                                on_change: move |value| selection.write().active_tokens_mut().set(*token, value),
                            }
                        }
                    }
                }
            }
            div { class: "grid gap-1 text-xs",
                div { class: "flex items-center justify-between",
                    span { class: "font-medium text-foreground", "{export_label}" }
                    CopyButton { value: css_export.clone() }
                }
                textarea {
                    class: "h-40 w-full rounded-md border border-input bg-background p-2 font-mono text-[11px] text-foreground",
                    readonly: true,
                    "aria-label": "{export_label}",
                    value: "{css_export}",
                }
            }
        }
    }
}

#[component]
fn ThemeAppearanceControl(
    value: ThemeAppearance,
    on_change: EventHandler<ThemeAppearance>,
) -> Element {
    rsx! {
        label { class: "grid gap-1 text-xs font-medium",
            "Appearance"
            select {
                class: "rounded-md border border-input bg-background px-2 py-1.5 text-sm text-foreground",
                value: "{value.value()}",
                onchange: move |event| {
                    if let Some(appearance) = ThemeAppearance::from_value(&event.value()) {
                        on_change.call(appearance);
                    }
                },
                for appearance in ThemeAppearance::ALL {
                    option { value: "{appearance.value()}", "{appearance.label()}" }
                }
            }
        }
    }
}

#[component]
fn PaletteControl(
    label: &'static str,
    appearance: ThemeAppearance,
    primary_role: bool,
    value: Palette,
    on_change: EventHandler<Palette>,
) -> Element {
    rsx! {
        fieldset { class: "grid gap-1",
            legend { class: "text-xs font-medium", "{label}" }
            div { class: "grid grid-cols-3 gap-1",
                for palette in Palette::ALL {
                    {
                        let colors = if primary_role {
                            palette.primary_tokens(appearance)
                        } else {
                            palette.surface_tokens(appearance)
                        };
                        let style = format!(
                            "background-color: hsl({}); color: hsl({});",
                            colors.background, colors.foreground
                        );
                        rsx! {
                            button {
                                class: "rounded-md border border-border px-1 py-1 text-[10px] font-medium",
                                r#type: "button",
                                aria_pressed: palette == value,
                                style: "{style}",
                                onclick: move |_| on_change.call(palette),
                                "{palette.label()}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ActiveRolePreview(primary: Palette, secondary: Palette, accent: Palette) -> Element {
    rsx! {
        div { class: "grid grid-cols-3 gap-1", aria_label: "Selected semantic roles",
            div { class: "rounded-md bg-primary px-1 py-1 text-center text-[10px] font-medium text-primary-foreground",
                "Primary · {primary.label()}"
            }
            div { class: "rounded-md bg-secondary px-1 py-1 text-center text-[10px] font-medium text-secondary-foreground",
                "Secondary · {secondary.label()}"
            }
            div { class: "rounded-md bg-accent px-1 py-1 text-center text-[10px] font-medium text-accent-foreground",
                "Accent · {accent.label()}"
            }
        }
    }
}

#[component]
fn SemanticTokenControl(
    token: ThemeToken,
    value: String,
    on_change: EventHandler<String>,
) -> Element {
    let label = token.label();
    if token.is_color() {
        let color_value = hsl_to_hex(&value).unwrap_or_else(|| "#000000".to_string());
        rsx! {
            label { class: "grid gap-1 text-xs",
                code { class: "text-[10px] text-muted-foreground", "{label}" }
                div { class: "flex items-center gap-2",
                    input {
                        class: "h-8 w-10 cursor-pointer rounded border border-input bg-background p-0.5",
                        r#type: "color",
                        value: "{color_value}",
                        onchange: move |event| {
                            if let Some(hsl) = hex_to_hsl(&event.value()) {
                                on_change.call(hsl);
                            }
                        },
                    }
                    code { class: "min-w-0 truncate text-[10px] text-muted-foreground", "{value}" }
                }
            }
        }
    } else {
        rsx! {
            label { class: "grid gap-1 text-xs",
                code { class: "text-[10px] text-muted-foreground", "{label}" }
                input {
                    class: "rounded-md border border-input bg-background px-2 py-1 text-xs text-foreground",
                    r#type: "text",
                    value: "{value}",
                    oninput: move |event| on_change.call(event.value()),
                }
            }
        }
    }
}

fn hsl_to_hex(value: &str) -> Option<String> {
    let (hue, saturation, lightness) = parse_hsl(value)?;
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let hue_sector = hue / 60.0;
    let x = chroma * (1.0 - (hue_sector.rem_euclid(2.0) - 1.0).abs());
    let (red, green, blue) = match hue_sector {
        value if value < 1.0 => (chroma, x, 0.0),
        value if value < 2.0 => (x, chroma, 0.0),
        value if value < 3.0 => (0.0, chroma, x),
        value if value < 4.0 => (0.0, x, chroma),
        value if value < 5.0 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let offset = lightness - chroma / 2.0;
    Some(format!(
        "#{:02x}{:02x}{:02x}",
        ((red + offset) * 255.0).round() as u8,
        ((green + offset) * 255.0).round() as u8,
        ((blue + offset) * 255.0).round() as u8,
    ))
}

fn hex_to_hsl(value: &str) -> Option<String> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let red = u8::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let difference = max - min;
    let lightness = (max + min) / 2.0;
    let saturation = if difference == 0.0 {
        0.0
    } else {
        difference / (1.0 - (2.0 * lightness - 1.0).abs())
    };
    let hue = if difference == 0.0 {
        0.0
    } else if max == red {
        60.0 * ((green - blue) / difference).rem_euclid(6.0)
    } else if max == green {
        60.0 * ((blue - red) / difference + 2.0)
    } else {
        60.0 * ((red - green) / difference + 4.0)
    };
    Some(format!(
        "{hue:.1} {:.1}% {:.1}%",
        saturation * 100.0,
        lightness * 100.0
    ))
}

fn parse_hsl(value: &str) -> Option<(f64, f64, f64)> {
    let mut values = value.split_ascii_whitespace();
    let hue = values.next()?.parse::<f64>().ok()?.rem_euclid(360.0);
    let saturation = values.next()?.strip_suffix('%')?.parse::<f64>().ok()? / 100.0;
    let lightness = values.next()?.strip_suffix('%')?.parse::<f64>().ok()? / 100.0;
    if values.next().is_some()
        || !(0.0..=1.0).contains(&saturation)
        || !(0.0..=1.0).contains(&lightness)
    {
        return None;
    }
    Some((hue, saturation, lightness))
}

#[cfg(test)]
mod tests {
    use super::{
        Palette, ThemeAppearance, ThemeSelection, ThemeToken, ThemeVariables, hex_to_hsl,
        hsl_to_hex, hydrate_tokens,
    };

    #[test]
    fn palette_matching_round_trips_every_preset_and_appearance() {
        for appearance in ThemeAppearance::ALL {
            for palette in Palette::ALL {
                let primary = palette.primary_tokens(appearance);
                assert_eq!(
                    Palette::matching_primary(primary.background, primary.foreground, appearance),
                    Some(palette)
                );
                let surface = palette.surface_tokens(appearance);
                assert_eq!(
                    Palette::matching_surface(surface.background, surface.foreground, appearance),
                    Some(palette)
                );
            }
        }
    }

    #[test]
    fn palette_matching_is_none_for_an_unrecognized_value() {
        assert_eq!(
            Palette::matching_primary("1 2% 3%", "4 5% 6%", ThemeAppearance::Light),
            None
        );
    }

    #[test]
    fn every_palette_combination_supplies_the_complete_semantic_contract() {
        for primary in Palette::ALL {
            for secondary in Palette::ALL {
                for accent in Palette::ALL {
                    let mut selection = ThemeSelection::default();
                    selection.set_primary_palette(primary);
                    selection.set_secondary_palette(secondary);
                    selection.set_accent_palette(accent);
                    for appearance in ThemeAppearance::ALL {
                        selection.appearance = appearance;
                        let pairs = selection.active_tokens().root_property_pairs();
                        for token in [
                            "--background",
                            "--primary",
                            "--secondary",
                            "--accent",
                            "--destructive",
                            "--border",
                            "--input",
                            "--ring",
                            "--radius",
                            "--sidebar",
                            "--sidebar-primary",
                            "--sidebar-accent",
                        ] {
                            assert!(
                                pairs.iter().any(|(name, _)| *name == token),
                                "missing {token}"
                            );
                        }
                        assert!(
                            pairs.iter().all(|(name, _)| !name.starts_with("--color-")),
                            "root_property_pairs should stick to raw tokens -- the installed \
                             @theme block already derives --color-* aliases from them live"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn direct_edits_stay_with_the_selected_appearance() {
        let mut selection = ThemeSelection::default();
        selection
            .active_tokens_mut()
            .set(ThemeToken::Background, "240 100% 50%".into());
        selection.appearance = ThemeAppearance::Dark;
        assert_ne!(
            selection.active_tokens().get(ThemeToken::Background),
            "240 100% 50%"
        );
        selection.appearance = ThemeAppearance::Light;
        assert_eq!(
            selection.active_tokens().get(ThemeToken::Background),
            "240 100% 50%"
        );
    }

    #[test]
    fn generated_themes_change_the_palette_selection() {
        let mut selection = ThemeSelection::default();
        let initial = (
            selection.primary_palette,
            selection.secondary_palette,
            selection.accent_palette,
        );
        selection.generate_theme();
        assert_ne!(
            (
                selection.primary_palette,
                selection.secondary_palette,
                selection.accent_palette,
            ),
            initial
        );
    }

    #[test]
    fn color_picker_conversion_round_trips() {
        let hsl = "221.2 83.2% 53.3%";
        let hex = hsl_to_hex(hsl).expect("fixture HSL should convert");
        let round_trip = hex_to_hsl(&hex).expect("generated hex should convert");
        assert_eq!(hsl_to_hex(&round_trip), Some(hex));
    }

    #[test]
    fn token_overrides_update_the_raw_root_property_only() {
        let mut selection = ThemeSelection::default();
        selection
            .active_tokens_mut()
            .set(ThemeToken::Primary, "221.2 83.2% 53.3%".into());
        let pairs = selection.active_tokens().root_property_pairs();
        assert!(
            pairs
                .iter()
                .any(|(name, value)| *name == "--primary" && value == "221.2 83.2% 53.3%")
        );
        assert!(
            !pairs.iter().any(|(name, _)| *name == "--color-primary"),
            "no --color-primary pair should be written -- the installed @theme block's \
             var(--primary) lookup already derives it live from the raw property above"
        );
    }

    #[test]
    fn css_export_uses_the_active_appearance_and_canonical_theme_tokens() {
        let mut selection = ThemeSelection::default();
        selection
            .active_tokens_mut()
            .set(ThemeToken::Primary, "221.2 83.2% 53.3%".into());
        let light_css = selection.css_export();
        assert!(light_css.starts_with(":root {"));
        assert!(light_css.contains("--primary: 221.2 83.2% 53.3%;"));
        assert!(light_css.contains("--sidebar-ring:"));
        assert!(!light_css.contains("--color-primary:"));

        selection.appearance = ThemeAppearance::Dark;
        assert!(selection.css_export().starts_with(".dark {"));
    }

    #[test]
    fn hydrate_tokens_adopts_read_values_and_skips_empty_ones() {
        let mut variables = ThemeVariables::light();
        let tokens = [ThemeToken::Primary, ThemeToken::Background];
        let values = ["1 2% 3%".to_string(), String::new()];
        hydrate_tokens(&mut variables, &tokens, &values);
        assert_eq!(variables.primary, "1 2% 3%");
        assert_eq!(variables.background, ThemeVariables::light().background);
    }

    #[test]
    fn hydrate_tokens_is_a_no_op_on_a_length_mismatch() {
        let mut variables = ThemeVariables::light();
        let before = variables.clone();
        hydrate_tokens(&mut variables, &[ThemeToken::Primary], &[]);
        assert_eq!(variables, before);
    }

    #[test]
    fn reset_restores_the_default_theme() {
        let mut selection = ThemeSelection::default();
        selection.generate_theme();
        selection.appearance = ThemeAppearance::Dark;
        selection.reset();
        assert_eq!(selection, ThemeSelection::default());
    }
}
