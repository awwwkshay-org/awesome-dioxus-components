//! Source-owned shadcn-style Button for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;

/// A semantic button with the default adico/shadcn visual language.
#[component]
pub fn Button(children: Element, class: Option<String>, disabled: Option<bool>) -> Element {
    let class = cn(&[
        "inline-flex h-9 items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        button { class, disabled, {children} }
    }
}
