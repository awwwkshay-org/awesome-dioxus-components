//! Source-owned shadcn-style Badge for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;

/// A small status/label pill with the default adico/shadcn visual language.
#[component]
pub fn Badge(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "inline-flex items-center rounded-md border border-transparent bg-primary px-2.5 py-0.5 text-xs font-semibold text-primary-foreground transition-colors hover:bg-primary/80 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}
