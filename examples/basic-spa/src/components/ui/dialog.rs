//! Source-owned shadcn-style Dialog composition for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;
pub use adico_primitives::dialog::{
    DialogContent as DialogPrimitiveContent, DialogDescription, DialogRoot as Dialog, DialogTitle,
};

/// Opens the surrounding [`Dialog`] through the owned headless primitive.
#[component]
pub fn DialogTrigger(children: Element, class: Option<String>) -> Element {
    let context: adico_primitives::dialog::DialogCtx = use_context();
    let class = cn(&[
        "inline-flex items-center justify-center",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        button {
            class,
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
        "fixed inset-0 z-50 bg-black/80 data-[state=open]:animate-in data-[state=closed]:animate-out",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        // This style is mounted only while a modal overlay is visible. Keeping
        // it with the source-owned overlay makes the behavior transparent to
        // consumers and naturally handles nested dialogs.
        style { "html {{ overflow: hidden; }}" }
        div {
            class,
            "aria-hidden": "true",
            "data-adico-dialog-overlay": "true",
            style: "position: fixed; inset: 0; z-index: 50;",
            onclick: move |_| context.set_open(false),
        }
    }
}

/// Styled content backed by the owned Dialog focus, dismissal, and ARIA primitive.
#[component]
pub fn DialogContent(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "fixed left-1/2 top-1/2 z-50 grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 rounded-lg border bg-background p-6 text-foreground shadow-lg",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        DialogPrimitiveContent {
            class,
            style: "position: fixed; z-index: 51;",
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
