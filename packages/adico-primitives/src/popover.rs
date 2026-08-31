// No dedicated WAI-ARIA APG "Popover" pattern exists; this follows the closest APG guidance --
// the Dialog pattern -- applied conditionally: `PopoverContent` renders `role="dialog"` with
// `aria-modal="true"` and focus-trapping only when `is_modal` is set (the default), otherwise
// it's a non-modal disclosure positioned by the shared `Positioner`, dismissed on Escape or an
// outside click. This file was already a genuine adaptation, not ported-unmodified, and that
// adaptation is unchanged here:
//
// Adapted from upstream: `PopoverContent`'s inline `document::eval` focus-trap
// script is replaced with this crate's existing target-gated
// `use_focus_trap`/`FocusTrapScript` (the same internals `dialog` already
// uses), rather than duplicating the eval string. This keeps the module
// SSR-safe without upstream's unconditional `document::eval` call.

//! Defines the [`PopoverRoot`] component and its sub-components.

use dioxus::prelude::*;

use crate::{
    ContentAlign, ContentSide, FocusTrapScript, positioner::Positioner, use_animated_open,
    use_controlled, use_escape_key, use_focus_trap, use_id_or, use_outside_dismiss, use_unique_id,
};

#[derive(Clone, Copy)]
struct PopoverCtx {
    #[allow(unused)]
    open: Memo<bool>,
    #[allow(unused)]
    set_open: Callback<bool>,

    // Whether the dialog is a modal and should capture focus.
    #[allow(unused)]
    is_modal: ReadSignal<bool>,
    labelledby: Signal<String>,
    root_id: Memo<String>,
}

/// The props for the [`PopoverRoot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct PopoverRootProps {
    /// Whether the popover is a modal and should capture focus.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub is_modal: ReadSignal<bool>,

    /// The controlled open state of the popover.
    pub open: ReadSignal<Option<bool>>,

    /// The default open state when uncontrolled.
    #[props(default)]
    pub default_open: bool,

    /// Callback fired when the open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// The id of the popover root element.
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes to apply to the popover root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the popover root component.
    pub children: Element,
}

/// # PopoverRoot
///
/// The `PopoverRoot` component wraps all the popover components and manages the state. You can define a
/// [`PopoverTrigger`] component to toggle the popover's open state, and a [`PopoverContent`] component
/// to define the content that appears when the popover is open under this component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::popover::*;
///
/// #[component]
/// fn Demo() -> Element {
///     let mut open = use_signal(|| false);
///
///     rsx! {
///         PopoverRoot {
///             open: open(),
///             on_open_change: move |v| open.set(v),
///             PopoverTrigger {
///                 "Show Popover"
///             }
///             PopoverContent {
///                 h3 { "Delete Item?" }
///                 button {
///                     onclick: move |_| {
///                         open.set(false);
///                     },
///                     "Yes!"
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`PopoverRoot`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates if the popover is open or closed. Values are `open` or `closed`.
#[component]
pub fn PopoverRoot(props: PopoverRootProps) -> Element {
    let labelledby = use_unique_id();
    let gen_root_id = use_unique_id();
    let root_id = use_id_or(gen_root_id, props.id);

    let (open, set_open) = use_controlled(props.open, props.default_open, props.on_open_change);

    use_context_provider(|| PopoverCtx {
        open,
        set_open,
        is_modal: props.is_modal,
        labelledby,
        root_id,
    });

    let onkeydown = use_escape_key(open, move || set_open.call(false));

    rsx! {
        div {
            id: root_id,
            "data-state": if open() { "open" } else { "closed" },
            onkeydown,
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`PopoverContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct PopoverContentProps {
    /// The id of the popover content element.
    pub id: ReadSignal<Option<String>>,

    /// CSS class for the popover content.
    #[props(default)]
    pub class: Option<String>,

    /// Side of the trigger to place the popover.
    #[props(default = ContentSide::Bottom)]
    pub side: ContentSide,

    /// Alignment of the popover relative to the trigger.
    #[props(default = ContentAlign::Center)]
    pub align: ContentAlign,

    /// Additional attributes to apply to the content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the popover content component.
    pub children: Element,
}

/// # PopoverContent
///
/// The `PopoverContent` component defines the content of the popover. This component will
/// only be rendered if the popover is open, and it will handle focus trapping if the popover is modal.
///
/// This must be used inside a [`PopoverRoot`] component.
///
/// ## Styling
///
/// The [`PopoverContent`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates if the popover is open or closed. Values are `open` or `closed`.
/// - `data-side`: Indicates the side where the popover is positioned relative to the trigger. Possible values are `top`, `right`, `bottom`, and `left`.
/// - `data-align`: Indicates the alignment of the popover relative to the trigger. Possible values are `start`, `center`, and `end`.
#[component]
pub fn PopoverContent(props: PopoverContentProps) -> Element {
    let ctx: PopoverCtx = use_context();
    let open = ctx.open;
    let is_modal = ctx.is_modal;

    let gen_id = use_unique_id();
    let id = use_id_or(gen_id, props.id);

    let render = use_animated_open(id, ctx.open);

    use_focus_trap(id, open, is_modal);

    rsx! {
        FocusTrapScript {}
        if render() {
            PopoverContentRendered {
                id,
                class: props.class,
                side: props.side,
                align: props.align,
                attributes: props.attributes,
                children: props.children
            }
        }
    }
}

/// The rendered content of the popover. This is separated out so the global event listener
/// is only added when the popover is actually rendered.
#[component]
pub fn PopoverContentRendered(
    id: String,
    class: Option<String>,
    side: ContentSide,
    align: ContentAlign,
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let ctx: PopoverCtx = use_context();
    let open = ctx.open;
    let is_open = open();
    let set_open = ctx.set_open;

    // Escape is handled by `PopoverRoot`'s own `onkeydown` (via
    // `use_escape_key`), not here: see that hook's doc comment for why a
    // document-level listener does not work on `web`.
    use_outside_dismiss(ctx.root_id, move || set_open.call(false));

    // `"data-state"` is a custom (non-`GlobalAttributes`-identifier) key,
    // which Dioxus's rsx macro can't mix with a `..spread` on a *component*
    // call (unlike a plain html element) — build it into the merged
    // attribute list by hand instead.
    let mut merged_attributes = vec![dioxus_core::Attribute::new(
        "data-state",
        if is_open { "open" } else { "closed" },
        None,
        false,
    )];
    merged_attributes.extend(attributes);

    rsx! {
        Positioner {
            id,
            anchor_id: ctx.labelledby,
            side,
            align,
            offset: 4.0,
            role: "dialog",
            aria_modal: (ctx.is_modal)().then_some("true"),
            aria_labelledby: ctx.labelledby,
            aria_hidden: (!is_open).then_some("true"),
            class: class.unwrap_or_else(|| "dx-popover-content".to_string()),
            attributes: merged_attributes,
            {children}
        }
    }
}

/// The props for the [`PopoverTrigger`] component.
#[derive(Props, Clone, PartialEq)]
pub struct PopoverTriggerProps {
    /// Additional attributes to apply to the trigger element.
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    pub attributes: Vec<Attribute>,

    /// The children of the trigger component.
    pub children: Element,
}

/// # PopoverTrigger
///
/// The `PopoverTrigger` is a button that toggles the visibility of the [`PopoverContent`].
///
/// This must be used inside a [`PopoverRoot`] component.
#[component]
pub fn PopoverTrigger(props: PopoverTriggerProps) -> Element {
    let ctx: PopoverCtx = use_context();
    let mut id = ctx.labelledby;
    let id_attribute = props
        .attributes
        .iter()
        .find(|attr| attr.name == "id")
        .cloned();
    use_effect(use_reactive!(|id_attribute| {
        if let Some(id_attribute) = id_attribute {
            match &id_attribute.value {
                dioxus_core::AttributeValue::Text(val) => id.set(val.to_string()),
                dioxus_core::AttributeValue::Float(val) => id.set(val.to_string()),
                dioxus_core::AttributeValue::Int(val) => id.set(val.to_string()),
                dioxus_core::AttributeValue::Bool(val) => id.set(val.to_string()),
                _ => {}
            }
        }
    }));

    rsx! {
        button {
            id,
            r#type: "button",
            onclick: move |e| {
                e.stop_propagation();
                ctx.set_open.call(!(ctx.open)());
            },
            ..props.attributes,
            {props.children}
        }
    }
}
