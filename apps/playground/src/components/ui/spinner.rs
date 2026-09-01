//! Source-owned shadcn-style Spinner for Dioxus.
//!
//! This item's classes are static enough that it does not depend on the
//! shared `cn` class-composition utility; the optional `class` override is
//! appended directly.

use dioxus::prelude::*;

use adico_primitives::icons::LoaderCircle;

const SPINNER_CLASS: &str = "size-4 animate-spin";

/// An indeterminate loading indicator, backed by a native
/// `role="status"` live region.
#[component]
pub fn Spinner(class: Option<String>) -> Element {
    let class = [SPINNER_CLASS, class.as_deref().unwrap_or_default()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    rsx! {
        LoaderCircle { class, role: "status", "aria-label": "Loading" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_animates() {
        assert!(SPINNER_CLASS.contains("animate-spin"));
    }
}
