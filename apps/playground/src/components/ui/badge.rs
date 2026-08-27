//! Source-owned shadcn-style Badge for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;

/// The semantic presentation of a [`Badge`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    /// Primary status or category.
    #[default]
    Default,
    /// Low-emphasis secondary status.
    Secondary,
    /// Destructive or error status.
    Destructive,
    /// A neutral outlined label.
    Outline,
    /// A positive verified status, matching the Dioxus Components catalog.
    Verified,
}

impl BadgeVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => {
                "border-transparent bg-primary text-primary-foreground hover:bg-primary/80"
            }
            Self::Secondary => {
                "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80"
            }
            Self::Destructive => {
                "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80"
            }
            Self::Outline => {
                "border-border bg-transparent text-foreground hover:bg-accent hover:text-accent-foreground"
            }
            Self::Verified => {
                "border-transparent bg-emerald-600 text-white hover:bg-emerald-600/80 dark:bg-emerald-500 dark:hover:bg-emerald-500/80"
            }
        }
    }
}

/// Props for [`Badge`].
#[derive(Props, Clone, PartialEq)]
pub struct BadgeProps {
    /// Semantic presentation variant.
    #[props(default)]
    pub variant: BadgeVariant,
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
        "inline-flex items-center rounded-md border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
        props.variant.class(),
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
    fn every_variant_has_a_semantic_surface() {
        assert!(BadgeVariant::Default.class().contains("bg-primary"));
        assert!(BadgeVariant::Secondary.class().contains("bg-secondary"));
        assert!(BadgeVariant::Destructive.class().contains("bg-destructive"));
        assert!(BadgeVariant::Outline.class().contains("border-border"));
        assert!(BadgeVariant::Verified.class().contains("emerald"));
    }
}
