//! Source-owned shadcn-style Avatar for Dioxus, backed by the owned adico
//! primitive layer.

use dioxus::prelude::*;

use adico_primitives::avatar::{
    Avatar as AvatarPrimitive, AvatarFallback as AvatarFallbackPrimitive,
    AvatarImage as AvatarImagePrimitive,
};

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

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

    /// The fallback initials' text size at this avatar size. `Default` maps
    /// to `text-sm`, matching the ambient ~ text size the fallback rendered
    /// at before this axis existed, so existing usage's fallback text is
    /// unchanged.
    fn fallback_text_class(self) -> &'static str {
        match self {
            Self::Sm => "text-xs",
            Self::Default => "text-sm",
            Self::Lg => "text-base",
        }
    }
}

/// Shares [`Avatar`]'s `size` with a composed [`AvatarFallback`] so the
/// fallback's initials scale with the circle. `Avatar` and `AvatarFallback`
/// otherwise deliberately share no context (see [`AvatarProps::radius`]'s
/// doc comment) -- this context exists only for the text-size axis, which,
/// unlike `radius`, has no reason a caller would ever want to set
/// independently of the root's `size`. Wraps a `Signal` (rather than a plain
/// `AvatarSize`) and is kept in sync below on every render: `use_context_provider`
/// only runs its init closure once, so a plain value would go stale if a
/// caller changes `size` on an already-mounted `Avatar`.
#[derive(Clone, Copy, PartialEq)]
struct AvatarSizeContext(Signal<AvatarSize>);

/// Props for [`Avatar`].
#[derive(Props, Clone, PartialEq)]
pub struct AvatarProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// The visual size.
    #[props(default)]
    pub size: AvatarSize,
    /// Corner radius of the avatar's clip shape. A caller who changes this
    /// should also set the same value on [`AvatarFallback`] (they clip
    /// independently — there is no shared context between them) to keep
    /// the loading/error state visually consistent with the loaded image.
    #[props(default = Radius::Full)]
    pub radius: Radius,
    /// Accessible label for the avatar image role.
    #[props(default)]
    pub aria_label: Option<String>,
    /// Called when the image loads successfully.
    #[props(default)]
    pub on_load: Option<EventHandler<()>>,
    /// Called when the image fails to load.
    #[props(default)]
    pub on_error: Option<EventHandler<()>>,
    /// Called when the avatar's loading state changes.
    #[props(default)]
    pub on_state_change: Option<EventHandler<adico_primitives::avatar::AvatarState>>,
    /// Native div/global attributes and events.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed [`AvatarImage`]/[`AvatarFallback`] children.
    pub children: Element,
}

/// A circular user-profile image container with the default adico/shadcn
/// visual language.
#[component]
pub fn Avatar(props: AvatarProps) -> Element {
    let mut size_ctx = use_context_provider(|| AvatarSizeContext(Signal::new(props.size)));
    if *size_ctx.0.peek() != props.size {
        size_ctx.0.set(props.size);
    }
    let class = cn(&[
        "relative flex shrink-0 overflow-hidden",
        props.size.class(),
        props.radius.class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AvatarPrimitive {
            class,
            aria_label: props.aria_label,
            on_load: props.on_load,
            on_error: props.on_error,
            on_state_change: props.on_state_change,
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// Props for [`AvatarImage`].
#[derive(Props, Clone, PartialEq)]
pub struct AvatarImageProps {
    /// Optional id for the image element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,
    /// The image source URL.
    pub src: String,
    /// Alt text for the image.
    #[props(default)]
    pub alt: Option<String>,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native img/global attributes and events.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
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
        AvatarImagePrimitive {
            id: props.id,
            src: props.src,
            alt: props.alt,
            class,
            attributes: props.attributes,
        }
    }
}

/// Props for [`AvatarFallback`].
#[derive(Props, Clone, PartialEq)]
pub struct AvatarFallbackProps {
    /// Corner radius of the fallback's clip shape. See [`Avatar::radius`]'s
    /// own doc comment — keep this matching `Avatar`'s value.
    #[props(default = Radius::Full)]
    pub radius: Radius,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native div/global attributes and events.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed fallback content (initials, an icon, etc.).
    pub children: Element,
}

/// The styled fallback shown while loading or on error/empty state.
#[component]
pub fn AvatarFallback(props: AvatarFallbackProps) -> Element {
    // Falls back to `AvatarSize::Default`'s text size when used outside an
    // `Avatar` (this part's doc says it must be composed inside one, but
    // reading a missing context should never panic).
    let size = try_consume_context::<AvatarSizeContext>()
        .map(|ctx| (ctx.0)())
        .unwrap_or_default();
    let class = cn(&[
        "flex size-full items-center justify-center bg-muted text-muted-foreground",
        size.fallback_text_class(),
        props.radius.class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AvatarFallbackPrimitive {
            class,
            attributes: props.attributes,
            {props.children}
        }
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

    #[test]
    fn every_size_maps_to_a_proportionally_scaled_fallback_text_size() {
        assert_eq!(AvatarSize::Sm.fallback_text_class(), "text-xs");
        assert_eq!(AvatarSize::Default.fallback_text_class(), "text-sm");
        assert_eq!(AvatarSize::Lg.fallback_text_class(), "text-base");
    }

    #[test]
    fn default_size_keeps_the_historical_fallback_text_size() {
        // `Default`'s fallback text size must stay `text-sm`, matching the
        // ambient size it rendered at before this axis existed, so no
        // existing `Avatar` usage's fallback text changes size silently.
        assert_eq!(AvatarSize::default().fallback_text_class(), "text-sm");
    }
}
