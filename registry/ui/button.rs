//! Source-owned shadcn-style Button for Dioxus.
//!
//! `Button` is a styled native `<button>`. Visible content is caller-composed
//! through `children`, so text-only, icon-only, and icon-plus-text buttons use
//! the same accessible native control. Use [`ButtonVariant`] and
//! [`ButtonSize`] for the stable visual API; use ordinary Dioxus button/global
//! attributes for `type`, `name`, `value`, form behavior, events, and ARIA.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::{Radius, Tone};
use crate::components::ui::spinner::Spinner;

/// The shape/structure of a [`Button`], independent of its [`Tone`] color
/// (`ButtonProps::color`). `Destructive` is not a shape -- reach the same
/// look via `variant: Primary, color: Tone::Error`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Filled, solid surface.
    #[default]
    Primary,
    /// Muted filled surface.
    Secondary,
    /// Bordered, unfilled surface.
    Outline,
    /// Transparent-until-hover surface.
    Ghost,
    /// Inline semantic-link styling, while remaining a native button.
    Link,
}

impl ButtonVariant {
    /// Structural classes this shape owns beyond its color, factored out of
    /// [`Tone`]'s shared per-shape methods so `Button`'s exact pre-existing
    /// look survives at `color: Tone::Default` (see `registry/lib/variants.rs`).
    fn shape_extra_class(self) -> &'static str {
        match self {
            Self::Primary | Self::Secondary | Self::Outline => "shadow-xs",
            Self::Ghost => "dark:hover:bg-accent/50",
            Self::Link => "h-auto px-0 py-0",
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

/// The visual size for a [`Button`]. Icon sizes are intended for icon-only
/// caller composition and should include an accessible `aria-label`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// Standard text-button size.
    #[default]
    Default,
    /// Extra-small text button.
    Xs,
    /// Small text button.
    Sm,
    /// Large text button.
    Lg,
    /// Standard square icon button.
    Icon,
    /// Extra-small square icon button.
    IconXs,
    /// Small square icon button.
    IconSm,
    /// Large square icon button.
    IconLg,
}

impl ButtonSize {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "h-9 px-4 py-2",
            Self::Xs => "h-6 gap-1 px-2 text-xs",
            Self::Sm => "h-8 gap-1.5 px-3",
            Self::Lg => "h-10 px-6",
            Self::Icon => "size-9",
            Self::IconXs => "size-6",
            Self::IconSm => "size-8",
            Self::IconLg => "size-10",
        }
    }
}

/// Props for [`Button`].
#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    /// Shape/structure of the button, independent of `color`.
    #[props(default)]
    pub variant: ButtonVariant,
    /// Semantic color, independent of `variant`. `Tone::Default` renders
    /// `variant`'s exact pre-existing look; any other value recolors it.
    #[props(default)]
    pub color: Tone,
    /// Visual size.
    #[props(default)]
    pub size: ButtonSize,
    /// Corner radius of the button surface.
    #[props(default = Radius::Md)]
    pub radius: Radius,
    /// Extra classes appended to the component's semantic base classes.
    #[props(default)]
    pub class: Option<String>,
    /// Native click handler. This makes Button composable as the action surface
    /// for registry components such as DialogTrigger and SheetTrigger.
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    /// Shows a [`Spinner`] and marks the button busy/disabled. An adico
    /// extension — shadcn's own convention is composing
    /// `<Button disabled><Spinner /></Button>` by hand at each call site.
    #[props(default)]
    pub loading: bool,
    /// Replaces the button's visible content while `loading` is true. Has
    /// no effect when `loading` is false.
    #[props(default)]
    pub loading_text: Option<String>,
    /// Native button and global attributes, including `disabled`, `type`, and
    /// event handlers. Because Dioxus requires an element's attribute
    /// spread to be its last attribute, a caller's own `disabled` here
    /// takes precedence over `loading`'s — same precedent as every other
    /// `attributes`-accepting component in this registry.
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed visible content.
    pub children: Element,
}

/// A semantic native button with the default adico/shadcn visual language.
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let class = cn(&[
        "inline-flex shrink-0 items-center justify-center gap-2 whitespace-nowrap text-sm font-medium outline-none transition-all focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 [&_svg]:shrink-0",
        props.variant.shape_extra_class(),
        props.variant.color_class(props.color),
        props.color.focus_ring_class(),
        props.size.class(),
        props.radius.class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        button {
            class,
            onclick: move |event| props.onclick.call(event),
            disabled: props.loading,
            aria_busy: props.loading,
            ..props.attributes,
            if props.loading {
                Spinner {}
                if let Some(text) = props.loading_text {
                    "{text}"
                } else {
                    {props.children}
                }
            } else {
                {props.children}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shape_has_a_distinct_default_color_class() {
        assert!(
            ButtonVariant::Primary
                .color_class(Tone::Default)
                .contains("bg-primary")
        );
        assert!(
            ButtonVariant::Outline
                .color_class(Tone::Default)
                .contains("border")
        );
        assert!(
            ButtonVariant::Secondary
                .color_class(Tone::Default)
                .contains("bg-secondary")
        );
        assert!(
            ButtonVariant::Ghost
                .color_class(Tone::Default)
                .contains("hover:bg-accent")
        );
        assert!(
            ButtonVariant::Link
                .color_class(Tone::Default)
                .contains("underline")
        );
    }

    #[test]
    fn every_shape_recolors_with_a_non_default_tone() {
        for variant in [
            ButtonVariant::Primary,
            ButtonVariant::Secondary,
            ButtonVariant::Outline,
            ButtonVariant::Ghost,
            ButtonVariant::Link,
        ] {
            assert!(variant.color_class(Tone::Success).contains("success"));
            assert!(variant.color_class(Tone::Warning).contains("warning"));
            assert!(variant.color_class(Tone::Error).contains("destructive"));
            assert!(variant.color_class(Tone::Info).contains("info"));
        }
    }

    #[test]
    fn removed_destructive_shape_is_reachable_via_color() {
        // variant: Primary, color: Error must match the old `Destructive`
        // shape's look (modulo `text-white` -> `text-destructive-foreground`,
        // both resolving to the same near-white color).
        let class = ButtonVariant::Primary.color_class(Tone::Error);
        assert!(class.contains("bg-destructive"));
        assert!(class.contains("text-destructive-foreground"));
        assert!(class.contains("hover:bg-destructive/90"));
    }

    #[test]
    fn icon_sizes_remain_square() {
        assert_eq!(ButtonSize::Icon.class(), "size-9");
        assert!(ButtonSize::IconLg.class().contains("size-10"));
    }
}
