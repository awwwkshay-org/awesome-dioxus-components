// Preview Card's parts (`root`/`trigger`/`positioner`/`popup`/`arrow`/`backdrop`/`portal`/
// `viewport`) are exactly `hover_card.rs`'s own shape -- both are hover/focus-triggered,
// `Positioner`-placed disclosures over rich, potentially-interactive content, and this module
// composes `positioner::Positioner` the same way `HoverCard` does rather than reimplementing
// placement. The one genuine behavioral difference Base UI's `PreviewCard.Trigger` adds over
// this crate's `HoverCard` is `delay`/`closeDelay`: hovering opens after a delay (Base UI
// defaults to 600ms) and closing waits a shorter delay (300ms) so moving the pointer from the
// trigger toward the popup doesn't close it before the popup's own `on_mouse_enter` cancels the
// close -- `HoverCardTrigger` opens/closes instantly on mouseenter/mouseleave, which reads as
// flickery for a *card* (rich content worth deliberately hovering into), unlike a short tooltip
// label. `use_open_close_delay` below is genuinely new: a small per-open/close-attempt
// generation counter cancels a still-pending timer when a new open/close request supersedes it
// (e.g. the pointer re-enters before the close delay elapses), using this crate's existing
// `crate::time::sleep` (already `#[cfg(target_family = "wasm")]`/native-gated by `time.rs`
// itself, so this file needs no target-aware code of its own).

//! Defines the [`PreviewCard`] component and its subcomponents.

use dioxus::prelude::*;

use crate::{
    ContentAlign, ContentSide, positioner::Positioner, use_animated_open, use_controlled,
    use_id_or, use_unique_id,
};

#[derive(Clone, Copy)]
struct PreviewCardCtx {
    open: Memo<bool>,
    set_open: Callback<bool>,
    disabled: ReadSignal<bool>,
    delay_ms: ReadSignal<u64>,
    close_delay_ms: ReadSignal<u64>,

    content_id: Signal<String>,
    trigger_id: Signal<String>,

    /// Bumped on every open/close request; a pending timer only applies its
    /// effect if this counter hasn't moved on since it started sleeping.
    request_generation: Signal<u64>,
}

impl PreviewCardCtx {
    fn request_open(&self, open: bool) {
        let mut generation = self.request_generation;
        let this_generation = generation() + 1;
        generation.set(this_generation);

        let delay = if open {
            (self.delay_ms)()
        } else {
            (self.close_delay_ms)()
        };
        let set_open = self.set_open;
        let request_generation = self.request_generation;
        spawn(async move {
            if delay > 0 {
                crate::time::sleep(std::time::Duration::from_millis(delay)).await;
            }
            if request_generation() == this_generation {
                set_open.call(open);
            }
        });
    }
}

/// The props for the [`PreviewCard`] component
#[derive(Props, Clone, PartialEq)]
pub struct PreviewCardProps {
    /// Whether the preview card is open
    pub open: ReadSignal<Option<bool>>,

    /// Default open state
    #[props(default)]
    pub default_open: bool,

    /// Callback when open state changes
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// Whether the preview card is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Milliseconds to wait after the pointer enters the trigger before
    /// opening. Defaults to 600, matching Base UI's own default.
    #[props(default = ReadSignal::new(Signal::new(600)))]
    pub delay_ms: ReadSignal<u64>,

    /// Milliseconds to wait after the pointer leaves the trigger (or
    /// content) before closing. Defaults to 300, matching Base UI's own
    /// default.
    #[props(default = ReadSignal::new(Signal::new(300)))]
    pub close_delay_ms: ReadSignal<u64>,

    /// Additional attributes for the preview card
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the preview card
    pub children: Element,
}

/// # PreviewCard
///
/// The `PreviewCard` component wraps a [`PreviewCardTrigger`] and a
/// [`PreviewCardContent`]. It shows a rich preview after hovering (or
/// focusing) the trigger for [`PreviewCardProps::delay_ms`], distinct from
/// [`crate::hover_card::HoverCard`]'s instant open/close.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::{
///     ContentAlign, ContentSide,
///     preview_card::{PreviewCard, PreviewCardContent, PreviewCardTrigger},
/// };
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         PreviewCard {
///             PreviewCardTrigger {
///                 a { href: "#", "@dioxuslabs" }
///             }
///             PreviewCardContent {
///                 side: ContentSide::Bottom,
///                 div { "Dioxus is the Rust framework for building fullstack web, desktop, and mobile apps." }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`PreviewCard`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the current state of the preview card. Values are `open` or `closed`.
/// - `data-disabled`: Indicates whether the item is disabled. Values are `true` or `false`.
#[component]
pub fn PreviewCard(props: PreviewCardProps) -> Element {
    let (open, set_open) = use_controlled(props.open, props.default_open, props.on_open_change);
    let content_id = use_unique_id();
    let trigger_id = use_unique_id();

    use_context_provider(|| PreviewCardCtx {
        open,
        set_open,
        disabled: props.disabled,
        delay_ms: props.delay_ms,
        close_delay_ms: props.close_delay_ms,
        content_id,
        trigger_id,
        request_generation: Signal::new(0),
    });

    rsx! {
        div {
            "data-state": if open() { "open" } else { "closed" },
            "data-disabled": (props.disabled)(),
            ..props.attributes,

            {props.children}
        }
    }
}

/// The props for the [`PreviewCardTrigger`] component
#[derive(Props, Clone, PartialEq)]
pub struct PreviewCardTriggerProps {
    /// Optional ID for the trigger element
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes for the preview card trigger
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the preview card trigger
    pub children: Element,
}

/// # PreviewCardTrigger
///
/// The [`PreviewCardTrigger`] component triggers the [`PreviewCardContent`]
/// to appear, after [`PreviewCardProps::delay_ms`], when hovered or
/// focused.
///
/// This component must be used inside a [`PreviewCard`] component.
#[component]
pub fn PreviewCardTrigger(props: PreviewCardTriggerProps) -> Element {
    let ctx: PreviewCardCtx = use_context();
    let id = use_id_or(ctx.trigger_id, props.id);

    let open_event = move || {
        if !(ctx.disabled)() {
            ctx.request_open(true);
        }
    };
    let close_event = move || {
        if !(ctx.disabled)() {
            ctx.request_open(false);
        }
    };

    rsx! {
        div {
            id,
            tabindex: "0",

            onmouseenter: move |_| open_event(),
            onmouseleave: move |_| close_event(),
            onfocus: move |_| open_event(),
            onblur: move |_| close_event(),

            role: "button",
            aria_describedby: (ctx.open)().then(|| ctx.content_id.cloned()),

            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`PreviewCardContent`] component
#[derive(Props, Clone, PartialEq)]
pub struct PreviewCardContentProps {
    /// Optional ID for the preview card content
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Side of the trigger to place the preview card
    #[props(default = ContentSide::Bottom)]
    pub side: ContentSide,

    /// Alignment of the preview card relative to the trigger
    #[props(default = ContentAlign::Center)]
    pub align: ContentAlign,

    /// Whether to keep the preview card mounted even when closed
    #[props(default = true)]
    pub force_mount: bool,

    /// Additional attributes for the preview card content
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the preview card content
    pub children: Element,
}

/// # PreviewCardContent
///
/// The [`PreviewCardContent`] component defines the content of the parent
/// [`PreviewCard`]. Unlike [`crate::hover_card::HoverCardContent`] (which
/// uses `role="tooltip"`, matching a short text label), this renders with
/// no tooltip role -- its content is expected to be rich and potentially
/// interactive, which the ARIA tooltip role's own spec discourages.
///
/// This component must be used inside a [`PreviewCard`] component.
///
/// ## Styling
///
/// The [`PreviewCardContent`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the state of the preview card. Values are `open` or `closed`.
/// - `data-side`: Indicates the side of the trigger where the preview card is placed. Values are `top`, `right`, `bottom`, or `left`.
/// - `data-align`: Indicates the alignment of the preview card relative to the trigger. Values are `start`, `center`, or `end`.
#[component]
pub fn PreviewCardContent(props: PreviewCardContentProps) -> Element {
    let ctx: PreviewCardCtx = use_context();
    let is_open = (ctx.open)();
    if !is_open && !props.force_mount {
        return rsx!({});
    }

    let id = use_id_or(ctx.content_id, props.id);

    let handle_mouse_enter = move |_: Event<MouseData>| {
        if !(ctx.disabled)() {
            ctx.request_open(true);
        }
    };
    let handle_mouse_leave = move |_: Event<MouseData>| {
        if !(ctx.disabled)() {
            ctx.request_open(false);
        }
    };

    let render = use_animated_open(id, ctx.open);

    let mut merged_attributes = vec![dioxus_core::Attribute::new(
        "data-state",
        if is_open { "open" } else { "closed" },
        None,
        false,
    )];
    merged_attributes.extend(props.attributes);

    rsx! {
        if render() {
            Positioner {
                id: Some(id()),
                anchor_id: ctx.trigger_id,
                side: props.side,
                align: props.align,
                offset: 4.0,
                attributes: merged_attributes,

                on_mouse_enter: handle_mouse_enter,
                on_mouse_leave: handle_mouse_leave,

                {props.children}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_content_is_not_rendered_without_force_mount() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                PreviewCard { open: Some(false),
                    PreviewCardTrigger { "trigger" }
                    PreviewCardContent { force_mount: false, "content" }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(!html.contains("content"), "{html}");
    }

    #[test]
    fn open_root_marks_data_state_open() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                PreviewCard { open: Some(true),
                    PreviewCardTrigger { "trigger" }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains(r#"data-state="open""#), "{html}");
    }
}
