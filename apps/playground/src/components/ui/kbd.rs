//! Source-owned shadcn-style Kbd (keyboard key) for Dioxus.
//!
//! This item's classes are static enough that it does not depend on the
//! shared `cn` class-composition utility; the optional `class` override is
//! appended directly.

use dioxus::prelude::*;

const KBD_CLASS: &str = "pointer-events-none inline-flex h-5 w-fit min-w-5 items-center justify-center gap-1 rounded-sm bg-muted px-1 font-sans text-xs font-medium text-muted-foreground select-none";
const KBD_GROUP_CLASS: &str = "inline-flex items-center gap-1";

/// A single visual keyboard key, e.g. `Kbd { "Ctrl" }`.
#[component]
pub fn Kbd(class: Option<String>, children: Element) -> Element {
    let class = [KBD_CLASS, class.as_deref().unwrap_or_default()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    rsx! {
        kbd { class, {children} }
    }
}

/// Groups related [`Kbd`] keys, e.g. a chord like `Kbd{"Ctrl"} + Kbd{"K"}`.
#[component]
pub fn KbdGroup(class: Option<String>, children: Element) -> Element {
    let class = [KBD_GROUP_CLASS, class.as_deref().unwrap_or_default()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    rsx! {
        kbd { class, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kbd_renders_on_a_muted_surface() {
        assert!(KBD_CLASS.contains("bg-muted"));
    }

    #[test]
    fn kbd_group_uses_an_inline_flex_layout() {
        assert!(KBD_GROUP_CLASS.contains("inline-flex"));
    }
}
