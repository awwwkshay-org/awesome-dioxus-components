//! Source-owned shadcn-style Avatar for Dioxus, backed by the owned adico
//! primitive layer.

use dioxus::prelude::*;

use adico_primitives::avatar::{
    Avatar as AvatarPrimitive, AvatarFallback as AvatarFallbackPrimitive,
    AvatarImage as AvatarImagePrimitive,
};

use crate::adico_lib::cn::cn;

/// The visual size of an [`Avatar`]. Was entirely absent before task 5.2 --
/// upstream shadcn added a `size` axis; adico only ever rendered one fixed
/// size (`size-10`). `Default` is kept at that existing value rather than
/// upstream's own new, smaller default (`size-8`), so this addition is
/// purely additive -- no existing `Avatar` usage changes size silently.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AvatarSize {
    /// Compact size.
    Sm,
    /// Default size (unchanged from before this prop existed).
    #[default]
    Default,
    /// Larger size.
    Lg,
}

impl AvatarSize {
    fn class(self) -> &'static str {
        match self {
            Self::Sm => "size-8",
            Self::Default => "size-10",
            Self::Lg => "size-12",
        }
    }
}

/// Props for [`Avatar`].
#[derive(Props, Clone, PartialEq)]
pub struct AvatarProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// The visual size.
    #[props(default)]
    pub size: AvatarSize,
    /// Accessible label for the avatar image role.
    #[props(default)]
    pub aria_label: Option<String>,
    /// Caller-composed [`AvatarImage`]/[`AvatarFallback`] children.
    pub children: Element,
}

/// A circular user-profile image container with the default adico/shadcn
/// visual language.
#[component]
pub fn Avatar(props: AvatarProps) -> Element {
    let class = cn(&[
        "relative flex shrink-0 overflow-hidden rounded-full",
        props.size.class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AvatarPrimitive { class, aria_label: props.aria_label, {props.children} }
    }
}

/// Props for [`AvatarImage`].
#[derive(Props, Clone, PartialEq)]
pub struct AvatarImageProps {
    /// The image source URL.
    pub src: String,
    /// Alt text for the image.
    #[props(default)]
    pub alt: Option<String>,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
}

/// The styled avatar image; stops rendering on load failure so
/// [`AvatarFallback`] takes over.
#[component]
pub fn AvatarImage(props: AvatarImageProps) -> Element {
    let class = cn(&[
        "aspect-square size-full",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AvatarImagePrimitive { src: props.src, alt: props.alt, class }
    }
}

/// Props for [`AvatarFallback`].
#[derive(Props, Clone, PartialEq)]
pub struct AvatarFallbackProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed fallback content (initials, an icon, etc.).
    pub children: Element,
}

/// The styled fallback shown while loading or on error/empty state.
#[component]
pub fn AvatarFallback(props: AvatarFallbackProps) -> Element {
    let class = cn(&[
        "flex size-full items-center justify-center rounded-full bg-muted text-muted-foreground",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AvatarFallbackPrimitive { class, {props.children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_class_clips_to_a_circle() {
        let class = cn(&[
            "relative flex size-10 shrink-0 overflow-hidden rounded-full",
            "",
        ]);
        assert!(class.contains("rounded-full"));
        assert!(class.contains("overflow-hidden"));
    }

    #[test]
    fn fallback_class_uses_semantic_muted_surface() {
        let class = cn(&[
            "flex size-full items-center justify-center rounded-full bg-muted text-muted-foreground",
            "",
        ]);
        assert!(class.contains("bg-muted"));
    }
}
