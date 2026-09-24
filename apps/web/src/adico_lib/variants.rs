//! Shared adico-owned style vocabulary for registry components.

/// Shared corner-radius scale for every component with a visible bounded
/// surface. `Default` tracks this project's `--radius` theme token, which
/// `tailwind.css` binds to the `rounded-lg` utility
/// (`--radius-lg: var(--radius)`) — so `Default` maps to `rounded-lg`, and
/// every other variant is named by its own visual size rather than by
/// Tailwind's suffix at that position. `Md` exists specifically so a
/// component whose current corner size is Tailwind's fixed (non-token)
/// `rounded-md` can declare `#[props(default = Radius::Md)]` and keep its
/// exact current visual, distinct from `Default`, which reactively tracks
/// the theme's `--radius` custom property.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Radius {
    None,
    Sm,
    Md,
    #[default]
    Default,
    Lg,
    Xl,
    Full,
}

impl Radius {
    pub fn class(self) -> &'static str {
        match self {
            Radius::None => "rounded-none",
            Radius::Sm => "rounded-sm",
            Radius::Md => "rounded-md",
            Radius::Default => "rounded-lg",
            Radius::Lg => "rounded-xl",
            Radius::Xl => "rounded-2xl",
            Radius::Full => "rounded-full",
        }
    }
}

/// Shared semantic-color vocabulary for presentational components that
/// expose an orthogonal `color` prop alongside their own shape/variant enum
/// (`button`, `badge`, `tag-group`'s `TagOption`). Named `Tone`, not `Color`,
/// to avoid colliding with `color_picker.rs`'s already-public `Color` type
/// (an RGB value, an unrelated concept).
///
/// `Error` resolves to the pre-existing `--destructive` theme token (see
/// [`Tone::name`]) -- only the Rust-facing name is new; nothing in
/// `packages/adico-cli/src/css.rs` or `theme_builder.rs` needed to change
/// for this rename.
///
/// Exposes one method per shape family (`solid`/`soft`/`outline`/`ghost`/
/// `link`) plus [`Tone::focus_ring_class`]. Each method's `Default` arm
/// reproduces that shape's pre-existing class from `button.rs`/`badge.rs`
/// (with one narrow, documented exception for `outline_class` -- see
/// design.md), so selecting `color: Tone::Default` changes nothing visually;
/// a non-`Default` value recolors that same shape. A consuming component
/// composes its own structural extras (padding, shadow, an extra dark-mode
/// hover, `border-transparent` bookkeeping) around these -- those genuinely
/// differ per component and predate this enum.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Default,
    Success,
    Warning,
    Error,
    Info,
}

impl Tone {
    /// The Tailwind color-name segment this tone's tokens are keyed under
    /// (`--{name}` / `--{name}-foreground`), and the single place that
    /// mapping lives -- every component composing a tone-specific Tailwind
    /// class should reference the token name from here rather than
    /// hardcoding `"success"`/`"warning"`/`"destructive"`/`"info"` again.
    pub fn name(self) -> &'static str {
        match self {
            Self::Default => "primary",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "destructive",
            Self::Info => "info",
        }
    }

    /// A filled surface: `{name}`-colored fill, `{name}-foreground` text, a
    /// slightly-transparent fill on hover. Matches `Button`'s and `Badge`'s
    /// pre-existing "primary"/"filled" shape once each component's own
    /// structural extras (`shadow-xs`, `border-transparent`) are factored
    /// out.
    pub fn solid_class(self) -> &'static str {
        match self {
            Self::Default => "bg-primary text-primary-foreground hover:bg-primary/90",
            Self::Success => "bg-success text-success-foreground hover:bg-success/90",
            Self::Warning => "bg-warning text-warning-foreground hover:bg-warning/90",
            Self::Error => "bg-destructive text-destructive-foreground hover:bg-destructive/90",
            Self::Info => "bg-info text-info-foreground hover:bg-info/90",
        }
    }

    /// A muted filled surface for `Default`, matching `Button`'s/`Badge`'s
    /// pre-existing "secondary" shape exactly; a lighter, `{name}`-tinted
    /// fill for every other tone (a "soft" treatment, since a fully solid
    /// fill is `solid_class`'s job).
    pub fn soft_class(self) -> &'static str {
        match self {
            Self::Default => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
            Self::Success => "bg-success/15 text-success hover:bg-success/25",
            Self::Warning => "bg-warning/15 text-warning hover:bg-warning/25",
            Self::Error => "bg-destructive/15 text-destructive hover:bg-destructive/25",
            Self::Info => "bg-info/15 text-info hover:bg-info/25",
        }
    }

    /// A bordered, unfilled surface. `Default`'s arm matches `Button`'s
    /// pre-existing "outline" shape exactly; `Badge`'s pre-existing outline
    /// shape diverged slightly (`border-border`/`bg-transparent`/
    /// `text-foreground`) and now renders this canonical form instead -- a
    /// deliberate, documented exception (see design.md), not an oversight.
    pub fn outline_class(self) -> &'static str {
        match self {
            Self::Default => {
                "border border-input bg-background hover:bg-accent hover:text-accent-foreground"
            }
            Self::Success => "border border-success text-success hover:bg-success/10",
            Self::Warning => "border border-warning text-warning hover:bg-warning/10",
            Self::Error => "border border-destructive text-destructive hover:bg-destructive/10",
            Self::Info => "border border-info text-info hover:bg-info/10",
        }
    }

    /// A transparent-until-hover surface. `Default`'s arm matches both
    /// `Button`'s and `Badge`'s pre-existing "ghost" shape exactly (`Button`
    /// additionally applies its own `dark:hover:bg-accent/50` as a
    /// structural extra, not part of this shared baseline).
    pub fn ghost_class(self) -> &'static str {
        match self {
            Self::Default => "hover:bg-accent hover:text-accent-foreground",
            Self::Success => "text-success hover:bg-success/10",
            Self::Warning => "text-warning hover:bg-warning/10",
            Self::Error => "text-destructive hover:bg-destructive/10",
            Self::Info => "text-info hover:bg-info/10",
        }
    }

    /// Inline, text-only, underline-on-hover styling. `Default`'s arm
    /// matches both `Button`'s and `Badge`'s pre-existing "link" shape
    /// exactly once each component's own sizing override
    /// (`Button`'s `h-auto px-0 py-0`) is factored out.
    pub fn link_class(self) -> &'static str {
        match self {
            Self::Default => "text-primary underline-offset-4 hover:underline",
            Self::Success => "text-success underline-offset-4 hover:underline",
            Self::Warning => "text-warning underline-offset-4 hover:underline",
            Self::Error => "text-destructive underline-offset-4 hover:underline",
            Self::Info => "text-info underline-offset-4 hover:underline",
        }
    }

    /// A tone-matched focus ring, for shapes with a colored fill or border
    /// where the ambient neutral `focus-visible:ring-ring/50` (set once at
    /// each component's root) would clash. Empty for `Default`, which relies
    /// on that ambient ring, matching `Button`'s pre-existing behavior
    /// (only its `Destructive`/tone variants ever carried an explicit ring
    /// color).
    pub fn focus_ring_class(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Success => "focus-visible:ring-success/20 dark:focus-visible:ring-success/40",
            Self::Warning => "focus-visible:ring-warning/20 dark:focus-visible:ring-warning/40",
            Self::Error => {
                "focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40"
            }
            Self::Info => "focus-visible:ring-info/20 dark:focus-visible:ring-info/40",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_maps_each_variant_to_its_tailwind_utility() {
        assert_eq!(Radius::None.class(), "rounded-none");
        assert_eq!(Radius::Sm.class(), "rounded-sm");
        assert_eq!(Radius::Md.class(), "rounded-md");
        assert_eq!(Radius::Default.class(), "rounded-lg");
        assert_eq!(Radius::Lg.class(), "rounded-xl");
        assert_eq!(Radius::Xl.class(), "rounded-2xl");
        assert_eq!(Radius::Full.class(), "rounded-full");
    }

    #[test]
    fn default_variant_is_default() {
        assert_eq!(Radius::default(), Radius::Default);
    }

    #[test]
    fn tone_name_matches_each_tones_theme_token_segment() {
        assert_eq!(Tone::Default.name(), "primary");
        assert_eq!(Tone::Success.name(), "success");
        assert_eq!(Tone::Warning.name(), "warning");
        // Only the Rust-facing name changed; the token itself stays `--destructive`.
        assert_eq!(Tone::Error.name(), "destructive");
        assert_eq!(Tone::Info.name(), "info");
    }

    const ALL_TONES: [Tone; 5] = [
        Tone::Default,
        Tone::Success,
        Tone::Warning,
        Tone::Error,
        Tone::Info,
    ];

    #[test]
    fn every_non_default_tone_names_itself_in_every_shape_family() {
        for tone in ALL_TONES {
            if tone == Tone::Default {
                continue;
            }
            let name = tone.name();
            for class in [
                tone.solid_class(),
                tone.soft_class(),
                tone.outline_class(),
                tone.ghost_class(),
                tone.link_class(),
            ] {
                assert!(class.contains(name), "{class} missing {name}");
            }
        }
    }

    #[test]
    fn default_tone_reproduces_each_shapes_pre_existing_look() {
        assert_eq!(
            Tone::Default.solid_class(),
            "bg-primary text-primary-foreground hover:bg-primary/90"
        );
        assert_eq!(
            Tone::Default.soft_class(),
            "bg-secondary text-secondary-foreground hover:bg-secondary/80"
        );
        assert_eq!(
            Tone::Default.outline_class(),
            "border border-input bg-background hover:bg-accent hover:text-accent-foreground"
        );
        assert_eq!(
            Tone::Default.ghost_class(),
            "hover:bg-accent hover:text-accent-foreground"
        );
        assert_eq!(
            Tone::Default.link_class(),
            "text-primary underline-offset-4 hover:underline"
        );
        assert_eq!(Tone::Default.focus_ring_class(), "");
    }

    #[test]
    fn tone_default_variant_is_default() {
        assert_eq!(Tone::default(), Tone::Default);
    }
}
