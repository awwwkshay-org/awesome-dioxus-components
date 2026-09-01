//! Source-owned shadcn-style Dialog composition for Dioxus.

use dioxus::prelude::*;

use super::button::{Button, ButtonSize, ButtonVariant};
use crate::adico_lib::cn::cn;
pub use adico_primitives::dialog::{
    DialogContent as DialogPrimitiveContent, DialogDescription, DialogRoot as Dialog, DialogTitle,
};
use adico_primitives::icons::X;

/// Opens the surrounding [`Dialog`] with the installed [`Button`] component.
///
/// This keeps dialog triggers visually and behaviorally consistent with every
/// other action in a consumer application while the Dialog primitive continues
/// to own focus, Escape, outside-dismissal, and ARIA behavior.
#[component]
pub fn DialogTrigger(
    children: Element,
    class: Option<String>,
    variant: Option<ButtonVariant>,
    size: Option<ButtonSize>,
) -> Element {
    let context: adico_primitives::dialog::DialogCtx = use_context();
    rsx! {
        Button {
            class,
            variant: variant.unwrap_or_default(),
            size: size.unwrap_or_default(),
            onclick: move |_| context.set_open(true),
            {children}
        }
    }
}

/// A visual overlay rendered only while the surrounding Dialog is open.
#[component]
pub fn DialogOverlay(class: Option<String>) -> Element {
    let context: adico_primitives::dialog::DialogCtx = use_context();
    if !context.is_open() {
        return rsx! {};
    }
    let class = cn(&[
        "fixed inset-0 z-50 bg-black/50 data-[state=open]:animate-in data-[state=closed]:animate-out",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div {
            class,
            "aria-hidden": "true",
            "data-adico-dialog-overlay": "true",
            onclick: move |_| context.set_open(false),
        }
    }
}

/// Styled content backed by the owned Dialog focus, dismissal, and ARIA primitive.
///
/// Renders a corner close button by default (`show_close_button`, matching
/// upstream shadcn's own `showCloseButton = true` default) -- previously
/// entirely absent here, a real, missing capability: without it, a sighted
/// mouse user has no visible way to close the dialog short of knowing to
/// click the backdrop or press Escape.
#[component]
pub fn DialogContent(
    children: Element,
    class: Option<String>,
    #[props(default = true)] show_close_button: bool,
) -> Element {
    let class = cn(&[
        "fixed left-1/2 top-1/2 z-[51] grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 rounded-lg border bg-background p-6 text-foreground shadow-lg",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        DialogPrimitiveContent {
            class,
            {children}
            if show_close_button {
                DialogClose { class: "absolute right-4 top-4" }
            }
        }
    }
}

/// A semantic header helper for Dialog titles and descriptions.
#[component]
pub fn DialogHeader(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex flex-col space-y-1.5 text-center sm:text-left",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! { div { class, {children} } }
}

/// A semantic footer helper, typically for [`Dialog`] action buttons.
/// Previously missing here entirely, even though [`DialogHeader`] already
/// existed -- a real, asymmetric composition gap against upstream, which
/// has always paired the two.
#[component]
pub fn DialogFooter(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! { div { class, {children} } }
}

/// A dismissible close control for a [`Dialog`]. Composable anywhere inside
/// [`DialogContent`] (for example, a "Cancel" button in a [`DialogFooter`]),
/// not just the default corner close button [`DialogContent`] renders.
/// Previously, closing a dialog from inside its own content required
/// reaching into `adico_primitives::dialog::DialogCtx` directly -- a
/// primitive-internals leak this component now avoids.
#[component]
pub fn DialogClose(children: Option<Element>, class: Option<String>) -> Element {
    let context: adico_primitives::dialog::DialogCtx = use_context();
    let class = cn(&[
        "rounded-xs opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        button {
            r#type: "button",
            class,
            onclick: move |_| context.set_open(false),
            match children {
                Some(children) => rsx! { {children} },
                None => rsx! {
                    X { class: "size-4" }
                    span { class: "sr-only", "Close" }
                },
            }
        }
    }
}
