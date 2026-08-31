//! Source-owned shadcn-style Dialog composition for Dioxus.

use dioxus::prelude::*;

use super::button::{Button, ButtonSize, ButtonVariant};
use crate::adico_lib::cn::cn;
pub use adico_primitives::dialog::{
    DialogContent as DialogPrimitiveContent, DialogDescription, DialogRoot as Dialog, DialogTitle,
};

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
#[component]
pub fn DialogContent(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "fixed left-1/2 top-1/2 z-[51] grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 rounded-lg border bg-background p-6 text-foreground shadow-lg",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        DialogPrimitiveContent {
            class,
            {children}
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
