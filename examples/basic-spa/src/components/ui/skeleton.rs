//! Source-owned shadcn-style Skeleton for Dioxus.
//!
//! This item's classes are static enough that it does not depend on the
//! shared `cn` class-composition utility; the optional `class` override is
//! appended directly.

use dioxus::prelude::*;

/// Visual style for a [`Skeleton`] placeholder.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SkeletonVariant {
    /// A softly rounded placeholder suitable for text and rectangular media.
    #[default]
    Default,
    /// A circular placeholder suitable for avatars.
    Circle,
}

impl SkeletonVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "rounded-md",
            Self::Circle => "rounded-full",
        }
    }
}

/// Props for [`Skeleton`].
#[derive(Props, Clone, PartialEq)]
pub struct SkeletonProps {
    /// Visual shape of the placeholder.
    #[props(default)]
    pub variant: SkeletonVariant,
    /// Additional sizing or layout classes. A skeleton intentionally has no
    /// intrinsic size, so callers provide e.g. `h-4 w-40`.
    #[props(default)]
    pub class: Option<String>,
    /// Whether assistive technology should ignore this purely decorative
    /// placeholder. Defaults to true.
    #[props(default = true)]
    pub decorative: bool,
    /// Native div/global attributes for the rare non-decorative status use.
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
}

/// A pulsing placeholder shown in place of content still loading.
#[component]
pub fn Skeleton(props: SkeletonProps) -> Element {
    let class = [
        "animate-pulse motion-reduce:animate-none bg-muted",
        props.variant.class(),
        props.class.as_deref().unwrap_or_default(),
    ]
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(" ");
    rsx! {
        div {
            class,
            "aria-hidden": if props.decorative { "true" } else { "false" },
            ..props.attributes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_have_intentional_shapes() {
        assert_eq!(SkeletonVariant::Default.class(), "rounded-md");
        assert_eq!(SkeletonVariant::Circle.class(), "rounded-full");
    }
}
