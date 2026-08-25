//! Source-owned shadcn-style Skeleton for Dioxus.
//!
//! This item's classes are static enough that it does not depend on the
//! shared `cn` class-composition utility; the optional `class` override is
//! appended directly.

use dioxus::prelude::*;

/// A pulsing placeholder shown in place of content still loading.
#[component]
pub fn Skeleton(class: Option<String>) -> Element {
    let extra = class.as_deref().unwrap_or_default();
    let class = if extra.is_empty() {
        "animate-pulse rounded-md bg-muted".to_string()
    } else {
        format!("animate-pulse rounded-md bg-muted {extra}")
    };
    rsx! {
        div { class }
    }
}
