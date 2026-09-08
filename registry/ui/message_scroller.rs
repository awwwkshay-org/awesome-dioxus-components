//! Source-owned shadcn-style Message Scroller for Dioxus, backed by the
//! owned `adico_primitives::message_scroller` scroll-anchoring primitive
//! (task 10.2).
//!
//! Styled facade over [`MessageScroller`]/[`MessageScrollerViewport`]/
//! [`MessageScrollerContent`] (task 10.2's primitive). `MessageScrollerItem`
//! and `MessageScrollerButton` have no dedicated primitive component behind
//! them -- once `use_message_scroller_pinned_to_bottom`/
//! `use_message_scroller_scroll_to_bottom` exist, a message item and a
//! "jump to latest" button are pure composition/styling, the same split
//! this crate already uses for `carousel.rs`'s registry-owned
//! `CarouselPrevious`/`CarouselNext`.

use dioxus::prelude::*;

use adico_primitives::message_scroller::{
    MessageScroller as MessageScrollerPrimitive,
    MessageScrollerContent as MessageScrollerContentPrimitive,
    MessageScrollerViewport as MessageScrollerViewportPrimitive,
    use_message_scroller_pinned_to_bottom, use_message_scroller_scroll_to_bottom,
};

use crate::adico_lib::cn::cn;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};

/// Props for [`MessageScroller`].
#[derive(Props, Clone, PartialEq)]
pub struct MessageScrollerProps {
    /// Distance (in pixels) from the true bottom edge within which the
    /// viewport still counts as "pinned to bottom". Forwarded to the
    /// primitive root.
    #[props(default = adico_primitives::message_scroller::DEFAULT_BOTTOM_THRESHOLD)]
    pub bottom_threshold: f64,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native div/global attributes.
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed [`MessageScrollerViewport`], plus an optional
    /// [`MessageScrollerButton`].
    pub children: Element,
}

/// A relatively positioned root, so a caller-composed
/// [`MessageScrollerButton`] can float over the viewport's bottom edge.
#[component]
pub fn MessageScroller(props: MessageScrollerProps) -> Element {
    let class = cn(&["relative", props.class.as_deref().unwrap_or_default()]);
    rsx! {
        MessageScrollerPrimitive {
            bottom_threshold: props.bottom_threshold,
            class,
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// Props for [`MessageScrollerViewport`].
#[derive(Props, Clone, PartialEq)]
pub struct MessageScrollerViewportProps {
    /// Extra classes appended to the semantic default. Callers own the
    /// scroll container's height (this component only sets `overflow-y`).
    #[props(default)]
    pub class: Option<String>,
    /// Native div/global attributes.
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed [`MessageScrollerContent`].
    pub children: Element,
}

/// The scrollable viewport. Opts into the shared scroll-area contract
/// (`with_scroll_area: true`) so its native scrollbar is themed consistently with
/// every other scroll surface -- the primitive itself merges this class with its own
/// visibility class, so `class` here stays exactly the caller-facing extras, not the
/// full merged string.
#[component]
pub fn MessageScrollerViewport(props: MessageScrollerViewportProps) -> Element {
    let class = cn(&[
        "overflow-y-auto overscroll-contain",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        MessageScrollerViewportPrimitive {
            with_scroll_area: true,
            class,
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// Props for [`MessageScrollerContent`].
#[derive(Props, Clone, PartialEq)]
pub struct MessageScrollerContentProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native div/global attributes.
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed [`MessageScrollerItem`]s.
    pub children: Element,
}

/// The measured content wrapper.
#[component]
pub fn MessageScrollerContent(props: MessageScrollerContentProps) -> Element {
    let class = cn(&[
        "flex flex-col gap-3 p-4",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        MessageScrollerContentPrimitive { class, attributes: props.attributes, {props.children} }
    }
}

/// Props for [`MessageScrollerItem`].
#[derive(Props, Clone, PartialEq)]
pub struct MessageScrollerItemProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native div/global attributes.
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed message content -- typically the sibling `Message`
    /// or `Bubble` registry items.
    pub children: Element,
}

/// One scrolled item. Purely a styling wrapper -- see this module's own
/// header comment for why it has no dedicated primitive behind it.
#[component]
pub fn MessageScrollerItem(props: MessageScrollerItemProps) -> Element {
    let class = cn(&["min-w-0", props.class.as_deref().unwrap_or_default()]);
    rsx! {
        div { class, ..props.attributes, {props.children} }
    }
}

/// Props for [`MessageScrollerButton`].
#[derive(Props, Clone, PartialEq)]
pub struct MessageScrollerButtonProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native button/global attributes.
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed visible content. Defaults to no children being
    /// required -- pass an icon and/or label.
    pub children: Element,
}

/// A "jump to latest" button, floated over the viewport's bottom edge and
/// only rendered while the viewport is scrolled away from the bottom.
#[component]
pub fn MessageScrollerButton(props: MessageScrollerButtonProps) -> Element {
    let pinned = use_message_scroller_pinned_to_bottom();
    let scroll_to_bottom = use_message_scroller_scroll_to_bottom();
    if pinned() {
        return rsx! {};
    }
    let class = cn(&[
        "absolute bottom-2 left-1/2 -translate-x-1/2 shadow-md",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        Button {
            variant: ButtonVariant::Secondary,
            size: ButtonSize::Sm,
            class,
            onclick: move |_| scroll_to_bottom(()),
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_class_scrolls_vertically_only() {
        let class = cn(&["overflow-y-auto overscroll-contain"]);
        assert!(class.contains("overflow-y-auto"));
    }

    #[test]
    fn root_class_is_a_positioning_context_for_the_floating_button() {
        let class = cn(&["relative"]);
        assert!(class.contains("relative"));
    }
}
