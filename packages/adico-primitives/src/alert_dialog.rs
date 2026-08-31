// SPDX-License-Identifier: MIT OR Apache-2.0
// Forked from DioxusLabs/dioxus-components at bf007c15d0cf4d04d3181cc46cf12325aa773955.
// Upstream path: primitives/src/alert_dialog.rs. See provenance/records/adico-primitives-wave2-risk.json.
//
// Adapted from upstream: AlertDialogRoot's unconditional `document::Script`
// and AlertDialogContent's own inline `document::eval` focus-trap script
// (`window.createFocusTrap`) duplicated exactly what this crate's
// `dialog` module already shares via `FocusTrapScript`/`use_focus_trap` --
// both are now delegated to those shared, target-gated (SSR-safe) helpers
// instead of being reimplemented here, matching the migration queue's
// explicit instruction to prefer delegation over duplication. Escape
// handling continues to use the shared `use_global_escape_listener`, already
// identical to upstream's own choice. Outside-dismiss is intentionally NOT
// added (unlike `dialog`/`DialogContent`): a WAI-ARIA alert dialog requires
// an explicit action and must not be dismissible by clicking outside it,
// which matches upstream's own omission of any outside-dismiss call here.
// Real-browser testing (a live `dx serve` fixture) then found that
// `use_global_escape_listener` alone never actually closes the dialog on
// Escape in this Dioxus 0.7.9/0.7.10 web runtime -- the exact gap already
// documented for `popover`/`context-menu` in
// provenance/records/adico-primitives-wave3-overlays.json. Testing also
// found `AlertDialogContent`'s focus trap does not move focus into the
// dialog on open (matching `dialog::DialogRoot`'s own tested behavior --
// its Playwright suite explicitly asserts the *trigger* stays focused, not
// dialog content), and upstream's own documented composition keeps the
// triggering button as a sibling of `AlertDialogContent` but a nested child
// of `AlertDialogRoot`. A `Content`-scoped Escape handler therefore never
// receives the keydown while focus is still on the trigger. `AlertDialogRoot`
// now carries a native `onkeydown` Escape handler on its own root instead --
// the same position `DialogRoot` already uses for exactly this reason -- and
// `AlertDialogCtx` is now `pub` with `is_open()`/`set_open()` accessors
// (matching `DialogCtx`'s existing public shape) so a registry facade can
// offer a context-aware `AlertDialogTrigger`, nested inside `AlertDialogRoot`
// like `DialogTrigger` already is, instead of the upstream doctest's
// external sibling button.

//! Defines the [`AlertDialogRoot`] component and its sub-components.

use crate::{
    FocusTrapScript, scroll_lock::use_scroll_lock, use_escape_key, use_focus_trap, use_id_or,
    use_unique_id,
};
use dioxus::prelude::*;

/// Context for the [`AlertDialogRoot`] component.
#[derive(Clone)]
pub struct AlertDialogCtx {
    open: Memo<bool>,
    set_open: Callback<bool>,
    labelledby: String,
    describedby: String,
}

impl AlertDialogCtx {
    /// Returns whether the alert dialog is open.
    pub fn is_open(&self) -> bool {
        self.open.cloned()
    }

    /// Sets whether the alert dialog is open.
    pub fn set_open(&self, open: bool) {
        self.set_open.call(open);
    }
}

/// The props for the [`AlertDialogRoot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogRootProps {
    /// The id of the alert dialog root element. If not provided, a unique id will be generated.
    pub id: ReadSignal<Option<String>>,
    /// Whether the alert dialog should be open by default. This is only used if the `open` signal is not provided.
    #[props(default)]
    pub default_open: bool,
    /// The open state of the alert dialog. If this is provided, it will be used to control the open state of the dialog.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,
    /// Callback to handle changes in the open state of the dialog.
    #[props(default)]
    pub on_open_change: Callback<bool>,
    /// Additional attributes to extend the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the alert dialog root element.
    pub children: Element,
}

/// # AlertDialogRoot
///
/// The entry point for the alert dialog. It manages the open state of the dialog and provides context to its children. You
/// can use it to create a backdrop for the dialog if needed. The contents will only be rendered when the dialog is open.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::alert_dialog::*;
///
/// #[component]
/// fn Demo() -> Element {
///     let mut open = use_signal(|| false);
///
///     rsx! {
///         button {
///             onclick: move |_| open.set(true),
///             "Show Alert Dialog"
///         }
///         AlertDialogRoot {
///             open: open(),
///             on_open_change: move |v| open.set(v),
///             AlertDialogContent {
///                 AlertDialogTitle { "Delete item" }
///                 AlertDialogDescription { "Are you sure you want to delete this item? This action cannot be undone." }
///                 AlertDialogActions {
///                     AlertDialogCancel { "Cancel" }
///                     AlertDialogAction {
///                         on_click: move |_| tracing::info!("Item deleted"),
///                         "Delete"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`AlertDialogRoot`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates if the alert dialog is open or closed. It can be either "open" or "closed".
#[component]
pub fn AlertDialogRoot(props: AlertDialogRootProps) -> Element {
    let labelledby = use_unique_id().to_string();
    let describedby = use_unique_id().to_string();
    let mut open_signal = use_signal(|| props.default_open);
    let set_open = use_callback(move |v: bool| {
        open_signal.set(v);
        props.on_open_change.call(v);
    });
    let open = use_memo(move || (props.open)().unwrap_or_else(&*open_signal));
    use_context_provider(|| AlertDialogCtx {
        open,
        set_open,
        labelledby,
        describedby,
    });

    let id = use_unique_id();
    let id = use_id_or(id, props.id);

    // Unlike upstream, this div (and its children, including any
    // `AlertDialogTrigger`) is always mounted rather than gated behind
    // `use_animated_open` -- matching `dialog::DialogRoot`'s actual pattern,
    // where the root always renders and only `DialogContent` gates itself on
    // the open state. `Content` below does the equivalent gating. This also
    // fixes Escape: an always-mounted root-level `onkeydown` (via
    // `use_escape_key`) reliably receives keyboard events wherever focus is,
    // unlike a listener scoped only to a conditionally-mounted Content.
    // Verified live via `dx serve`.
    let onkeydown = use_escape_key(move || set_open.call(false));

    rsx! {
        FocusTrapScript {}
        div {
            id,
            "data-state": if open() { "open" } else { "closed" },
            onkeydown,
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`AlertDialogContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogContentProps {
    /// The id of the alert dialog content element. If not provided, a unique id will be generated.
    pub id: ReadSignal<Option<String>>,

    /// The class to apply to the alert dialog content element.
    #[props(default)]
    pub class: Option<String>,

    /// Additional attributes to extend the content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the alert dialog content element.
    pub children: Element,
}

/// # AlertDialogContent
///
/// The content of the alert dialog. Any interactive content in the dialog should be placed
/// inside this component. It will trap focus within the dialog while it is open
///
/// This must be used inside an [`AlertDialogRoot`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::alert_dialog::*;
///
/// #[component]
/// fn Demo() -> Element {
///     let mut open = use_signal(|| false);
///
///     rsx! {
///         button {
///             onclick: move |_| open.set(true),
///             "Show Alert Dialog"
///         }
///         AlertDialogRoot {
///             open: open(),
///             on_open_change: move |v| open.set(v),
///             AlertDialogContent {
///                 AlertDialogTitle { "Delete item" }
///                 AlertDialogDescription { "Are you sure you want to delete this item? This action cannot be undone." }
///                 AlertDialogActions {
///                     AlertDialogCancel { "Cancel" }
///                     AlertDialogAction {
///                         on_click: move |_| tracing::info!("Item deleted"),
///                         "Delete"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn AlertDialogContent(props: AlertDialogContentProps) -> Element {
    let ctx: AlertDialogCtx = use_context();

    // `AlertDialogRoot` now always renders (see its own doc comment); this
    // gate -- matching `dialog::DialogContent`'s identical early return --
    // is what actually removes the content from the DOM while closed.
    if !ctx.is_open() {
        return rsx! {};
    }

    let open = ctx.open;

    // Escape is handled by `AlertDialogRoot`'s own `onkeydown` (via
    // `use_escape_key`), not here: see that hook's doc comment for why a
    // document-level listener does not work on `web`.
    let gen_id = use_unique_id();
    let id = use_id_or(gen_id, props.id);

    // Alert dialogs are always modal: unlike DialogContent, no outside-dismiss
    // listener is installed, matching the WAI-ARIA alertdialog pattern's
    // requirement that only an explicit action can close it.
    use_focus_trap(id, open, ReadSignal::new(Signal::new(true)));
    // Only rendered while open (see the early return above), matching
    // `dialog::DialogContent`'s identical scroll-lock wiring.
    use_scroll_lock(open);

    rsx! {
        div {
            id,
            role: "alertdialog",
            aria_modal: "true",
            aria_labelledby: ctx.labelledby.clone(),
            aria_describedby: ctx.describedby.clone(),
            class: props.class.clone().unwrap_or_else(|| "dx-alert-dialog".to_string()),
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`AlertDialogTitle`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogTitleProps {
    /// Additional attributes to extend the title element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the title element.
    pub children: Element,
}

/// # AlertDialogTitle
///
/// The title of the alert dialog. This will be used to label the dialog for accessibility purposes.
///
/// This must be used inside an [`AlertDialogRoot`] component and should be placed inside an [`AlertDialogContent`] component.
#[component]
pub fn AlertDialogTitle(props: AlertDialogTitleProps) -> Element {
    let ctx: AlertDialogCtx = use_context();
    rsx! {
        h2 { id: ctx.labelledby.clone(), ..props.attributes, {props.children} }
    }
}

/// The props for the [`AlertDialogDescription`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogDescriptionProps {
    /// Additional attributes to extend the description element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the description element.
    pub children: Element,
}

/// # AlertDialogDescription
///
/// The description of the alert dialog. This will be used to describe the dialog for accessibility purposes.
///
/// This must be used inside an [`AlertDialogRoot`] component and should be placed inside an [`AlertDialogContent`] component.
#[component]
pub fn AlertDialogDescription(props: AlertDialogDescriptionProps) -> Element {
    let ctx: AlertDialogCtx = use_context();
    rsx! {
        p { id: ctx.describedby.clone(), ..props.attributes, {props.children} }
    }
}

/// The props for the [`AlertDialogActions`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogActionsProps {
    /// Additional attributes to extend the actions element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the actions element.
    pub children: Element,
}

/// # AlertDialogActions
///
/// The actions of the alert dialog. This will be used to group the actions.
///
/// This must be used inside an [`AlertDialogRoot`] component and should be placed inside an [`AlertDialogContent`] component.
#[component]
pub fn AlertDialogActions(props: AlertDialogActionsProps) -> Element {
    rsx! {
        div { ..props.attributes, {props.children} }
    }
}

/// The props for the [`AlertDialogAction`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogActionProps {
    /// The click event handler for the action button.
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    /// Additional attributes to extend the action button element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the action button.
    pub children: Element,
}

/// # AlertDialogAction
///
/// An action button for the alert dialog. In addition to running the `on_click` callback, it will also close the dialog when clicked.
///
/// This must be used inside an [`AlertDialogRoot`] component and should be placed inside an [`AlertDialogContent`] component.
#[component]
pub fn AlertDialogAction(props: AlertDialogActionProps) -> Element {
    let ctx: AlertDialogCtx = use_context();
    let open = ctx.open;
    let set_open = ctx.set_open;
    let user_on_click = props.on_click;
    let on_click = use_callback(move |evt: MouseEvent| {
        set_open.call(false);
        if let Some(cb) = &user_on_click {
            cb.call(evt.clone());
        }
    });
    rsx! {
        button {
            tabindex: if open() { "0" } else { "-1" },
            r#type: "button",
            onclick: on_click,
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`AlertDialogCancel`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogCancelProps {
    /// The click event handler for the cancel button.
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    /// Additional attributes to extend the cancel button element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the cancel button.
    pub children: Element,
}

/// # AlertDialogCancel
///
/// An cancel button for the alert dialog. In addition to running the `on_click` callback, it will also close the dialog when clicked.
///
/// This must be used inside an [`AlertDialogRoot`] component and should be placed inside an [`AlertDialogContent`] component.
#[component]
pub fn AlertDialogCancel(props: AlertDialogCancelProps) -> Element {
    let ctx: AlertDialogCtx = use_context();
    let open = ctx.open;
    let set_open = ctx.set_open;
    let user_on_click = props.on_click;
    let on_click = use_callback(move |evt: MouseEvent| {
        set_open.call(false);
        if let Some(cb) = &user_on_click {
            cb.call(evt.clone());
        }
    });

    rsx! {
        button {
            tabindex: if open() { "0" } else { "-1" },
            r#type: "button",
            onclick: on_click,
            ..props.attributes,
            {props.children}
        }
    }
}
