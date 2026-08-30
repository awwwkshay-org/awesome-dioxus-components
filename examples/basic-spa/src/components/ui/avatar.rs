//! Source-owned shadcn-style Avatar for Dioxus, backed by the owned adico
//! primitive layer.

use dioxus::prelude::*;

use adico_primitives::avatar::{
    Avatar as AvatarPrimitive, AvatarFallback as AvatarFallbackPrimitive,
    AvatarImage as AvatarImagePrimitive,
};

use crate::adico_lib::cn::cn;

/// Props for [`Avatar`].
#[derive(Props, Clone, PartialEq)]
pub struct AvatarProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
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
        "relative flex size-10 shrink-0 overflow-hidden rounded-full",
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
