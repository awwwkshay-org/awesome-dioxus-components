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

use dioxus::prelude::*;

use adico_primitives::theme_mode::{apply_root_properties, clear_root_properties};

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
/// tertiary/accent).
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
            Self::SidebarBackground => "--sidebar-background",
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

    /// The token pairs [`apply_root_properties`] needs to apply this
    /// appearance to the document root, including the `--color-*` Tailwind
    /// aliases every installed component's utility classes resolve against.
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
            ("--sidebar-background", self.sidebar_background.clone()),
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
            ("--color-background", format!("hsl({})", self.background)),
            ("--color-foreground", format!("hsl({})", self.foreground)),
            ("--color-card", format!("hsl({})", self.card)),
            (
                "--color-card-foreground",
                format!("hsl({})", self.card_foreground),
            ),
            ("--color-popover", format!("hsl({})", self.popover)),
            (
                "--color-popover-foreground",
                format!("hsl({})", self.popover_foreground),
            ),
            ("--color-primary", format!("hsl({})", self.primary)),
            (
                "--color-primary-foreground",
                format!("hsl({})", self.primary_foreground),
            ),
            ("--color-secondary", format!("hsl({})", self.secondary)),
            (
                "--color-secondary-foreground",
                format!("hsl({})", self.secondary_foreground),
            ),
            ("--color-muted", format!("hsl({})", self.muted)),
            (
                "--color-muted-foreground",
                format!("hsl({})", self.muted_foreground),
            ),
            ("--color-accent", format!("hsl({})", self.accent)),
            (
                "--color-accent-foreground",
                format!("hsl({})", self.accent_foreground),
            ),
            ("--color-destructive", format!("hsl({})", self.destructive)),
            (
                "--color-destructive-foreground",
                format!("hsl({})", self.destructive_foreground),
            ),
            ("--color-border", format!("hsl({})", self.border)),
            ("--color-input", format!("hsl({})", self.input)),
            ("--color-ring", format!("hsl({})", self.ring)),
            (
                "--color-sidebar",
                format!("hsl({})", self.sidebar_background),
            ),
            (
                "--color-sidebar-foreground",
                format!("hsl({})", self.sidebar_foreground),
            ),
            (
                "--color-sidebar-primary",
                format!("hsl({})", self.sidebar_primary),
            ),
            (
                "--color-sidebar-primary-foreground",
                format!("hsl({})", self.sidebar_primary_foreground),
            ),
            (
                "--color-sidebar-accent",
                format!("hsl({})", self.sidebar_accent),
            ),
            (
                "--color-sidebar-accent-foreground",
                format!("hsl({})", self.sidebar_accent_foreground),
            ),
            (
                "--color-sidebar-border",
                format!("hsl({})", self.sidebar_border),
            ),
            (
                "--color-sidebar-ring",
                format!("hsl({})", self.sidebar_ring),
            ),
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
            ("--sidebar-background", self.sidebar_background.as_str()),
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
    tertiary_palette: Palette,
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
            tertiary_palette: Palette::Slate,
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

    fn set_tertiary_palette(&mut self, palette: Palette) {
        self.tertiary_palette = palette;
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
        let tertiary = Palette::ALL[next_palette_index(&mut state)];
        self.random_seed = state;
        self.set_primary_palette(primary);
        self.set_secondary_palette(secondary);
        self.set_tertiary_palette(tertiary);
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
/// Unlike `mode-toggle`, `ThemeBuilder` does not read or write the persisted
/// `theme_mode` global signal -- it owns its own light/dark appearance
/// selection, since it's an editing surface a consumer mounts occasionally
/// (for example behind a settings dialog), not an always-active mode switch.
#[component]
pub fn ThemeBuilder(
    #[props(default)] on_theme_change: Callback<ThemeVariables>,
    class: Option<String>,
) -> Element {
    let mut selection = use_signal(ThemeSelection::default);

    use_effect(move || {
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
                on_change: move |appearance| selection.write().appearance = appearance,
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
                label: "Tertiary",
                appearance: current.appearance,
                primary_role: false,
                value: current.tertiary_palette,
                on_change: move |palette| selection.write().set_tertiary_palette(palette),
            }
            ActiveRolePreview {
                primary: current.primary_palette,
                secondary: current.secondary_palette,
                tertiary: current.tertiary_palette,
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
            label { class: "grid gap-1 text-xs",
                span { class: "font-medium text-foreground", "{export_label}" }
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
fn ActiveRolePreview(primary: Palette, secondary: Palette, tertiary: Palette) -> Element {
    rsx! {
        div { class: "grid grid-cols-3 gap-1", aria_label: "Selected semantic roles",
            div { class: "rounded-md bg-primary px-1 py-1 text-center text-[10px] font-medium text-primary-foreground",
                "Primary · {primary.label()}"
            }
            div { class: "rounded-md bg-secondary px-1 py-1 text-center text-[10px] font-medium text-secondary-foreground",
                "Secondary · {secondary.label()}"
            }
            div { class: "rounded-md bg-accent px-1 py-1 text-center text-[10px] font-medium text-accent-foreground",
                "Tertiary · {tertiary.label()}"
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
    use super::{Palette, ThemeAppearance, ThemeSelection, ThemeToken, hex_to_hsl, hsl_to_hex};

    #[test]
    fn every_palette_combination_supplies_the_complete_semantic_contract() {
        for primary in Palette::ALL {
            for secondary in Palette::ALL {
                for tertiary in Palette::ALL {
                    let mut selection = ThemeSelection::default();
                    selection.set_primary_palette(primary);
                    selection.set_secondary_palette(secondary);
                    selection.set_tertiary_palette(tertiary);
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
                            "--sidebar-background",
                            "--sidebar-primary",
                            "--sidebar-accent",
                        ] {
                            assert!(
                                pairs.iter().any(|(name, _)| *name == token),
                                "missing {token}"
                            );
                        }
                        for alias in [
                            "--color-primary",
                            "--color-secondary",
                            "--color-accent",
                            "--color-card",
                            "--color-sidebar",
                            "--color-sidebar-primary",
                        ] {
                            assert!(
                                pairs.iter().any(|(name, _)| *name == alias),
                                "missing {alias}"
                            );
                        }
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
            selection.tertiary_palette,
        );
        selection.generate_theme();
        assert_ne!(
            (
                selection.primary_palette,
                selection.secondary_palette,
                selection.tertiary_palette,
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
    fn token_overrides_update_the_matching_tailwind_aliases() {
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
            pairs.iter().any(
                |(name, value)| *name == "--color-primary" && value == "hsl(221.2 83.2% 53.3%)"
            )
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
    fn reset_restores_the_default_theme() {
        let mut selection = ThemeSelection::default();
        selection.generate_theme();
        selection.appearance = ThemeAppearance::Dark;
        selection.reset();
        assert_eq!(selection, ThemeSelection::default());
    }
}
