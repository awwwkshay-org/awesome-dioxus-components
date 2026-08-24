//! Company-curated Card shell for the Awwwkshay design system, reusing the
//! official adico `cn` class-composition helper as an explicit cross-registry
//! dependency.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;

/// A source-owned card shell with the Awwwkshay visual language.
#[component]
pub fn Card(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "rounded-lg border bg-card p-6 text-card-foreground shadow-sm",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}
