//! Source-owned shadcn-style Toggle Group for Dioxus, backed by the owned
//! adico primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::toggle_group::ToggleGroup;
use adico_primitives::toggle_group::ToggleItem as ToggleItemPrimitive;

use crate::adico_lib::cn::cn;

/// A single pressable item within a [`ToggleGroup`].
#[component]
pub fn ToggleItem(
    index: ReadSignal<usize>,
    #[props(default)] disabled: ReadSignal<bool>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "inline-flex h-9 items-center justify-center gap-2 rounded-md px-2 text-sm font-medium outline-none transition-colors hover:bg-muted hover:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 data-[state=on]:bg-accent data-[state=on]:text-accent-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ToggleItemPrimitive { index, disabled, class, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressed_item_uses_the_same_semantic_accent_surface_as_toggle() {
        let class = cn(&["data-[state=on]:bg-accent data-[state=on]:text-accent-foreground", ""]);
        assert!(class.contains("data-[state=on]:bg-accent"));
    }
}
