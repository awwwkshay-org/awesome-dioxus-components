//! Playground-only controls for exercising the installed shadcn-style theme
//! contract. The selected values are applied to the application shell, so
//! copied registry components continue to consume ordinary CSS variables.

use dioxus::prelude::*;

use crate::components::ui;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

impl ThemeMode {
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

    const fn primary_tokens(self, mode: ThemeMode) -> ColorTokens {
        match (self, mode) {
            (Self::Slate, ThemeMode::Light) => ColorTokens::new("222.2 47.4% 11.2%", "210 40% 98%"),
            (Self::Blue, ThemeMode::Light) => ColorTokens::new("221.2 83.2% 53.3%", "210 40% 98%"),
            (Self::Violet, ThemeMode::Light) => {
                ColorTokens::new("262.1 83.3% 57.8%", "210 40% 98%")
            }
            (Self::Emerald, ThemeMode::Light) => {
                ColorTokens::new("160.1 84.1% 39.4%", "210 40% 98%")
            }
            (Self::Rose, ThemeMode::Light) => ColorTokens::new("346.8 77.2% 49.8%", "210 40% 98%"),
            (Self::Amber, ThemeMode::Light) => {
                ColorTokens::new("37.7 92.1% 50.2%", "26 83.3% 14.1%")
            }
            (Self::Slate, ThemeMode::Dark) => ColorTokens::new("210 40% 98%", "222.2 47.4% 11.2%"),
            (Self::Blue, ThemeMode::Dark) => {
                ColorTokens::new("213.1 93.9% 67.8%", "222.2 47.4% 11.2%")
            }
            (Self::Violet, ThemeMode::Dark) => ColorTokens::new("263.4 70% 50.4%", "0 0% 100%"),
            (Self::Emerald, ThemeMode::Dark) => {
                ColorTokens::new("158.1 64.4% 51.6%", "2.7 19.3% 10.2%")
            }
            (Self::Rose, ThemeMode::Dark) => ColorTokens::new("349.7 89.2% 60.2%", "0 0% 100%"),
            (Self::Amber, ThemeMode::Dark) => {
                ColorTokens::new("47.9 95.8% 53.1%", "26 83.3% 14.1%")
            }
        }
    }

    const fn surface_tokens(self, mode: ThemeMode) -> ColorTokens {
        match (self, mode) {
            (Self::Slate, ThemeMode::Light) => {
                ColorTokens::new("210 40% 96.1%", "222.2 47.4% 11.2%")
            }
            (Self::Blue, ThemeMode::Light) => {
                ColorTokens::new("214.3 94.6% 92.7%", "221.2 83.2% 29.4%")
            }
            (Self::Violet, ThemeMode::Light) => {
                ColorTokens::new("250 100% 95.3%", "262.1 83.3% 30%")
            }
            (Self::Emerald, ThemeMode::Light) => {
                ColorTokens::new("152.4 76% 92.2%", "161.4 93.5% 16.9%")
            }
            (Self::Rose, ThemeMode::Light) => {
                ColorTokens::new("355.6 100% 94.7%", "343.4 79.7% 25.7%")
            }
            (Self::Amber, ThemeMode::Light) => ColorTokens::new("48 96.5% 88.8%", "26 83.3% 14.1%"),
            (Self::Slate, ThemeMode::Dark) => ColorTokens::new("217.2 32.6% 17.5%", "210 40% 98%"),
            (Self::Blue, ThemeMode::Dark) => {
                ColorTokens::new("217.2 32.6% 17.5%", "219.4 100% 92%")
            }
            (Self::Violet, ThemeMode::Dark) => {
                ColorTokens::new("263.4 38.6% 17.8%", "250 100% 92%")
            }
            (Self::Emerald, ThemeMode::Dark) => {
                ColorTokens::new("163.1 36.7% 16.1%", "149.3 80.4% 90%")
            }
            (Self::Rose, ThemeMode::Dark) => {
                ColorTokens::new("343.4 43.8% 16.1%", "355.6 100% 94.7%")
            }
            (Self::Amber, ThemeMode::Dark) => ColorTokens::new("30 47.8% 16.1%", "48 96.5% 88.8%"),
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

#[derive(Clone, Debug, PartialEq)]
struct ThemeVariables {
    background: String,
    foreground: String,
    card: String,
    card_foreground: String,
    popover: String,
    popover_foreground: String,
    primary: String,
    primary_foreground: String,
    secondary: String,
    secondary_foreground: String,
    muted: String,
    muted_foreground: String,
    accent: String,
    accent_foreground: String,
    destructive: String,
    destructive_foreground: String,
    border: String,
    input: String,
    ring: String,
    radius: String,
    sidebar_background: String,
    sidebar_foreground: String,
    sidebar_primary: String,
    sidebar_primary_foreground: String,
    sidebar_accent: String,
    sidebar_accent_foreground: String,
    sidebar_border: String,
    sidebar_ring: String,
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

    fn variables(&self) -> String {
        format!(
            "--background: {background}; --foreground: {foreground}; --card: {card}; \
             --card-foreground: {card_foreground}; --popover: {popover}; \
             --popover-foreground: {popover_foreground}; --primary: {primary}; \
             --primary-foreground: {primary_foreground}; --secondary: {secondary}; \
             --secondary-foreground: {secondary_foreground}; --muted: {muted}; \
             --muted-foreground: {muted_foreground}; --accent: {accent}; \
             --accent-foreground: {accent_foreground}; --destructive: {destructive}; \
             --destructive-foreground: {destructive_foreground}; --border: {border}; \
             --input: {input}; --ring: {ring}; --radius: {radius}; \
             --sidebar-background: {sidebar_background}; --sidebar-foreground: {sidebar_foreground}; \
             --sidebar-primary: {sidebar_primary}; \
             --sidebar-primary-foreground: {sidebar_primary_foreground}; \
             --sidebar-accent: {sidebar_accent}; \
             --sidebar-accent-foreground: {sidebar_accent_foreground}; \
             --sidebar-border: {sidebar_border}; --sidebar-ring: {sidebar_ring}; \
             --color-background: hsl({background}); --color-foreground: hsl({foreground}); \
             --color-card: hsl({card}); --color-card-foreground: hsl({card_foreground}); \
             --color-popover: hsl({popover}); --color-popover-foreground: hsl({popover_foreground}); \
             --color-primary: hsl({primary}); --color-primary-foreground: hsl({primary_foreground}); \
             --color-secondary: hsl({secondary}); --color-secondary-foreground: hsl({secondary_foreground}); \
             --color-muted: hsl({muted}); --color-muted-foreground: hsl({muted_foreground}); \
             --color-accent: hsl({accent}); --color-accent-foreground: hsl({accent_foreground}); \
             --color-destructive: hsl({destructive}); --color-destructive-foreground: hsl({destructive_foreground}); \
             --color-border: hsl({border}); --color-input: hsl({input}); --color-ring: hsl({ring}); \
             --color-sidebar: hsl({sidebar_background}); \
             --color-sidebar-foreground: hsl({sidebar_foreground}); \
             --color-sidebar-primary: hsl({sidebar_primary}); \
             --color-sidebar-primary-foreground: hsl({sidebar_primary_foreground}); \
             --color-sidebar-accent: hsl({sidebar_accent}); \
             --color-sidebar-accent-foreground: hsl({sidebar_accent_foreground}); \
             --color-sidebar-border: hsl({sidebar_border}); --color-sidebar-ring: hsl({sidebar_ring});",
            background = self.background,
            foreground = self.foreground,
            card = self.card,
            card_foreground = self.card_foreground,
            popover = self.popover,
            popover_foreground = self.popover_foreground,
            primary = self.primary,
            primary_foreground = self.primary_foreground,
            secondary = self.secondary,
            secondary_foreground = self.secondary_foreground,
            muted = self.muted,
            muted_foreground = self.muted_foreground,
            accent = self.accent,
            accent_foreground = self.accent_foreground,
            destructive = self.destructive,
            destructive_foreground = self.destructive_foreground,
            border = self.border,
            input = self.input,
            ring = self.ring,
            radius = self.radius,
            sidebar_background = self.sidebar_background,
            sidebar_foreground = self.sidebar_foreground,
            sidebar_primary = self.sidebar_primary,
            sidebar_primary_foreground = self.sidebar_primary_foreground,
            sidebar_accent = self.sidebar_accent,
            sidebar_accent_foreground = self.sidebar_accent_foreground,
            sidebar_border = self.sidebar_border,
            sidebar_ring = self.sidebar_ring,
        )
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
pub struct ThemeSelection {
    mode: ThemeMode,
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
            mode: ThemeMode::Light,
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
    pub const fn shell_class(&self) -> &'static str {
        match self.mode {
            ThemeMode::Light => "fixed inset-0 overflow-hidden bg-background text-foreground",
            ThemeMode::Dark => "dark fixed inset-0 overflow-hidden bg-background text-foreground",
        }
    }

    pub fn variables(&self) -> String {
        self.active_tokens().variables()
    }

    fn css_export(&self) -> String {
        let selector = match self.mode {
            ThemeMode::Light => ":root",
            ThemeMode::Dark => ".dark",
        };
        format!(
            "{selector} {{\n{}\n}}\n",
            self.active_tokens().css_declarations()
        )
    }

    fn active_tokens(&self) -> &ThemeVariables {
        match self.mode {
            ThemeMode::Light => &self.light,
            ThemeMode::Dark => &self.dark,
        }
    }

    fn active_tokens_mut(&mut self) -> &mut ThemeVariables {
        match self.mode {
            ThemeMode::Light => &mut self.light,
            ThemeMode::Dark => &mut self.dark,
        }
    }

    fn tokens_mut_for(&mut self, mode: ThemeMode) -> &mut ThemeVariables {
        match mode {
            ThemeMode::Light => &mut self.light,
            ThemeMode::Dark => &mut self.dark,
        }
    }

    fn set_primary_palette(&mut self, palette: Palette) {
        self.primary_palette = palette;
        for mode in ThemeMode::ALL {
            let colors = palette.primary_tokens(mode);
            let tokens = self.tokens_mut_for(mode);
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
        for mode in ThemeMode::ALL {
            let colors = palette.surface_tokens(mode);
            let tokens = self.tokens_mut_for(mode);
            tokens.secondary = colors.background.into();
            tokens.secondary_foreground = colors.foreground.into();
            tokens.muted = colors.background.into();
            tokens.muted_foreground = colors.foreground.into();
        }
    }

    fn set_tertiary_palette(&mut self, palette: Palette) {
        self.tertiary_palette = palette;
        for mode in ThemeMode::ALL {
            let colors = palette.surface_tokens(mode);
            let tokens = self.tokens_mut_for(mode);
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

#[component]
pub fn ThemeLauncher(mut open: Signal<bool>) -> Element {
    let theme = use_context::<Signal<ThemeSelection>>();
    let selection = theme();

    rsx! {
        section { class: "mt-3 shrink-0 rounded-lg border border-border bg-card p-3", aria_label: "Theme customization tray",
            div { class: "flex items-center justify-between gap-2",
                div { class: "min-w-0",
                    h2 { class: "text-sm font-semibold", "Theme" }
                    p { class: "truncate text-xs text-muted-foreground",
                        "{selection.mode.label()} · {selection.primary_palette.label()} / {selection.secondary_palette.label()} / {selection.tertiary_palette.label()}"
                    }
                }
                button {
                    class: "shrink-0 rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90",
                    r#type: "button",
                    aria_haspopup: "dialog",
                    aria_expanded: open(),
                    onclick: move |_| open.set(true),
                    "Customize"
                }
            }
        }
    }
}

#[component]
pub fn ThemeModal(mut theme: Signal<ThemeSelection>, mut open: Signal<bool>) -> Element {
    let copy_status = use_signal(|| None::<String>);
    let selection = theme();
    let active_tokens = selection.active_tokens().clone();
    let css_export = selection.css_export();
    let copy_label = format!("Copy {} CSS variables", selection.mode.label());

    rsx! {
        ui::Dialog {
            open: open(),
            on_open_change: move |value| open.set(value),
            ui::DialogOverlay {}
            ui::DialogContent { class: "max-h-[calc(100svh-2rem)] max-w-4xl overflow-y-auto p-5 sm:p-6",
                div { class: "flex items-start justify-between gap-4",
                    ui::DialogHeader {
                        ui::DialogTitle { "Theme palette" }
                        ui::DialogDescription { "Configure the live shadcn semantic theme contract, then copy the active appearance as CSS variables." }
                    }
                    button {
                        class: "rounded-md p-1 text-muted-foreground hover:bg-accent hover:text-accent-foreground",
                        r#type: "button",
                        aria_label: "Close theme palette",
                        onclick: move |_| open.set(false),
                        "×"
                    }
                }
                div { class: "mt-4 space-y-3",
                    ThemeModeControl {
                        value: selection.mode,
                        on_change: move |mode| theme.write().mode = mode,
                    }
                    PaletteControl {
                        label: "Primary",
                        mode: selection.mode,
                        primary_role: true,
                        value: selection.primary_palette,
                        on_change: move |palette| theme.write().set_primary_palette(palette),
                    }
                    PaletteControl {
                        label: "Secondary",
                        mode: selection.mode,
                        primary_role: false,
                        value: selection.secondary_palette,
                        on_change: move |palette| theme.write().set_secondary_palette(palette),
                    }
                    PaletteControl {
                        label: "Tertiary",
                        mode: selection.mode,
                        primary_role: false,
                        value: selection.tertiary_palette,
                        on_change: move |palette| theme.write().set_tertiary_palette(palette),
                    }
                    ActiveRolePreview {
                        primary: selection.primary_palette,
                        secondary: selection.secondary_palette,
                        tertiary: selection.tertiary_palette,
                    }
                    div { class: "grid gap-2 sm:grid-cols-3",
                        button {
                            class: "rounded-md bg-primary px-2 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90",
                            r#type: "button",
                            onclick: move |_| theme.write().generate_theme(),
                            "Generate theme"
                        }
                        button {
                            class: "rounded-md border border-input bg-background px-2 py-1.5 text-sm font-medium text-foreground hover:bg-accent hover:text-accent-foreground",
                            r#type: "button",
                            onclick: move |_| theme.write().reset(),
                            "Reset theme"
                        }
                        button {
                            class: "rounded-md border border-input bg-background px-2 py-1.5 text-sm font-medium text-foreground hover:bg-accent hover:text-accent-foreground",
                            r#type: "button",
                            onclick: move |_| copy_theme_css(css_export.clone(), copy_status),
                            "{copy_label}"
                        }
                    }
                    if let Some(status) = copy_status() {
                        p { class: "text-xs text-muted-foreground", role: "status", "{status}" }
                    }
                    for group in THEME_GROUPS {
                        details { class: "rounded-md border border-border p-2",
                            summary { class: "cursor-pointer text-xs font-semibold", "{group.label}" }
                            div { class: "mt-2 grid gap-2",
                                for token in group.tokens {
                                    SemanticTokenControl {
                                        token: *token,
                                        value: active_tokens.get(*token).to_owned(),
                                        on_change: move |value| theme.write().active_tokens_mut().set(*token, value),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn copy_theme_css(css: String, mut status: Signal<Option<String>>) {
    let Some(window) = web_sys::window() else {
        status.set(Some(
            "Clipboard is unavailable in this browser.".to_string(),
        ));
        return;
    };

    let promise = window.navigator().clipboard().write_text(&css);
    wasm_bindgen_futures::spawn_local(async move {
        let message = if wasm_bindgen_futures::JsFuture::from(promise).await.is_ok() {
            "CSS variables copied to the clipboard."
        } else {
            "The browser did not allow clipboard access."
        };
        status.set(Some(message.to_string()));
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn copy_theme_css(_: String, mut status: Signal<Option<String>>) {
    status.set(Some(
        "Clipboard copying is available in the web build.".to_string(),
    ));
}

#[component]
fn ThemeModeControl(value: ThemeMode, on_change: EventHandler<ThemeMode>) -> Element {
    rsx! {
        label { class: "grid gap-1 text-xs font-medium",
            "Appearance"
            select {
                class: "rounded-md border border-input bg-background px-2 py-1.5 text-sm text-foreground",
                value: "{value.value()}",
                onchange: move |event| {
                    if let Some(mode) = ThemeMode::from_value(&event.value()) {
                        on_change.call(mode);
                    }
                },
                for mode in ThemeMode::ALL {
                    option { value: "{mode.value()}", "{mode.label()}" }
                }
            }
        }
    }
}

#[component]
fn PaletteControl(
    label: &'static str,
    mode: ThemeMode,
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
                            palette.primary_tokens(mode)
                        } else {
                            palette.surface_tokens(mode)
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
    use super::{Palette, ThemeMode, ThemeSelection, ThemeToken, hex_to_hsl, hsl_to_hex};

    #[test]
    fn every_palette_combination_supplies_the_complete_semantic_contract() {
        for primary in Palette::ALL {
            for secondary in Palette::ALL {
                for tertiary in Palette::ALL {
                    let mut selection = ThemeSelection::default();
                    selection.set_primary_palette(primary);
                    selection.set_secondary_palette(secondary);
                    selection.set_tertiary_palette(tertiary);
                    for mode in ThemeMode::ALL {
                        selection.mode = mode;
                        let variables = selection.variables();
                        for token in [
                            ThemeToken::Background,
                            ThemeToken::Primary,
                            ThemeToken::Secondary,
                            ThemeToken::Accent,
                            ThemeToken::Destructive,
                            ThemeToken::Border,
                            ThemeToken::Input,
                            ThemeToken::Ring,
                            ThemeToken::Radius,
                            ThemeToken::SidebarBackground,
                            ThemeToken::SidebarPrimary,
                            ThemeToken::SidebarAccent,
                        ] {
                            assert!(
                                variables.contains(token.label()),
                                "missing {}",
                                token.label()
                            );
                        }
                        for alias in [
                            "--color-primary:",
                            "--color-secondary:",
                            "--color-accent:",
                            "--color-card:",
                            "--color-sidebar:",
                            "--color-sidebar-primary:",
                        ] {
                            assert!(variables.contains(alias), "missing {alias}");
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
        selection.mode = ThemeMode::Dark;
        assert_ne!(
            selection.active_tokens().get(ThemeToken::Background),
            "240 100% 50%"
        );
        selection.mode = ThemeMode::Light;
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
        let variables = selection.variables();
        assert!(variables.contains("--primary: 221.2 83.2% 53.3%"));
        assert!(variables.contains("--color-primary: hsl(221.2 83.2% 53.3%)"));
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

        selection.mode = ThemeMode::Dark;
        assert!(selection.css_export().starts_with(".dark {"));
    }

    #[test]
    fn reset_restores_the_default_theme() {
        let mut selection = ThemeSelection::default();
        selection.generate_theme();
        selection.mode = ThemeMode::Dark;
        selection.reset();
        assert_eq!(selection, ThemeSelection::default());
    }
}
