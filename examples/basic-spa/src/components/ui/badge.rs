//! Source-owned shadcn-style Badge for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::{Radius, Tone};

/// The shape/structure of a [`Badge`], independent of its [`Tone`] color
/// (`BadgeProps::color`). `Destructive` and `Verified` are not shapes --
/// reach the same looks via `variant: Primary, color: Tone::Error` and
/// `variant: Primary, color: Tone::Success` respectively.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    /// Filled, solid surface.
    #[default]
    Primary,
    /// Muted filled surface.
    Secondary,
    /// Bordered, unfilled surface.
    Outline,
    /// Transparent-until-hover surface.
    Ghost,
    /// Inline semantic-link styling, while remaining a native `<span>`.
    Link,
}

impl BadgeVariant {
    /// Structural classes this shape owns beyond its color, factored out of
    /// [`Tone`]'s shared per-shape methods. `Badge`'s base class always sets
    /// `border`, so every shape except `Outline` needs `border-transparent`
    /// to hide it.
    fn shape_extra_class(self) -> &'static str {
        match self {
            Self::Outline => "",
            Self::Primary | Self::Secondary | Self::Ghost | Self::Link => "border-transparent",
        }
    }

    fn color_class(self, color: Tone) -> &'static str {
        match self {
            Self::Primary => color.solid_class(),
            Self::Secondary => color.soft_class(),
            Self::Outline => color.outline_class(),
            Self::Ghost => color.ghost_class(),
            Self::Link => color.link_class(),
        }
    }
}

/// Props for [`Badge`].
#[derive(Props, Clone, PartialEq)]
pub struct BadgeProps {
    /// Shape/structure of the badge, independent of `color`.
    #[props(default)]
    pub variant: BadgeVariant,
    /// Semantic color, independent of `variant`. `Tone::Default` renders
    /// `variant`'s exact pre-existing look (except `Outline`; see
    /// `registry/lib/variants.rs`'s `Tone::outline_class` doc comment); any
    /// other value recolors it.
    #[props(default)]
    pub color: Tone,
    /// Corner radius of the badge pill.
    #[props(default = Radius::Md)]
    pub radius: Radius,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native span/global attributes, including ARIA labels and event handlers.
    #[props(extends = GlobalAttributes)]
    #[props(extends = span)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed badge content.
    pub children: Element,
}

/// A small status/label pill with the default adico/shadcn visual language.
#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let class = cn(&[
        "inline-flex items-center border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
        props.variant.shape_extra_class(),
        props.variant.color_class(props.color),
        props.radius.class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        span { class, ..props.attributes, {props.children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shape_has_a_distinct_default_color_class() {
        assert!(
            BadgeVariant::Primary
                .color_class(Tone::Default)
                .contains("bg-primary")
        );
        assert!(
            BadgeVariant::Secondary
                .color_class(Tone::Default)
                .contains("bg-secondary")
        );
        assert!(
            BadgeVariant::Outline
                .color_class(Tone::Default)
                .contains("border")
        );
    }

    #[test]
    fn removed_destructive_shape_is_reachable_via_color() {
        let class = BadgeVariant::Primary.color_class(Tone::Error);
        assert!(class.contains("bg-destructive"));
        assert!(class.contains("text-destructive-foreground"));
    }

    #[test]
    fn removed_verified_shape_is_reachable_via_color() {
        // Reverses a previously deliberate hardcode -- see
        // statics/styling_usage/badge.json's updated rationale.
        let class = BadgeVariant::Primary.color_class(Tone::Success);
        assert!(class.contains("bg-success"));
        assert!(!class.contains("emerald"));
    }

    #[test]
    fn every_shape_recolors_with_a_non_default_tone() {
        for variant in [
            BadgeVariant::Primary,
            BadgeVariant::Secondary,
            BadgeVariant::Outline,
            BadgeVariant::Ghost,
            BadgeVariant::Link,
        ] {
            assert!(variant.color_class(Tone::Success).contains("success"));
            assert!(variant.color_class(Tone::Warning).contains("warning"));
            assert!(variant.color_class(Tone::Error).contains("destructive"));
            assert!(variant.color_class(Tone::Info).contains("info"));
        }
    }
}
