//! Source-owned Dioxus-only Drag And Drop List for Dioxus, backed by the
//! owned adico primitive layer. This is a Dioxus Components extra with no
//! shadcn equivalent -- it does not count toward shadcn parity.

use dioxus::prelude::*;

use adico_primitives::drag_and_drop_list::{
    DragAndDropDropIndicator as DragAndDropDropIndicatorPrimitive,
    DragAndDropList as DragAndDropListPrimitive,
    DragAndDropListItem as DragAndDropListItemPrimitive,
    DragAndDropListItems as DragAndDropListItemsPrimitive,
};
pub use adico_primitives::drag_and_drop_list::{
    DragAndDropInstructions, DragAndDropItemContext, DragAndDropListRenderItem,
    DragAndDropLiveRegion, use_drag_and_drop_list_items,
};

use crate::adico_lib::cn::cn;

/// A reorderable list container. Keyboard reordering (Enter to lift/drop,
/// Arrow keys to move, Escape to cancel, Delete/Backspace to remove) is
/// always available; native pointer/mouse drag-and-drop is also wired but
/// not independently browser-verified (see the primitive's own doc comment).
#[component]
pub fn DragAndDropList(
    items: Vec<Element>,
    #[props(default)] aria_label: Option<String>,
    class: Option<String>,
    #[props(default)] children: Option<Element>,
) -> Element {
    let class = cn(&["flex flex-col gap-1", class.as_deref().unwrap_or_default()]);
    rsx! {
        DragAndDropListPrimitive { items, aria_label, class, children }
    }
}

/// The inner `ul` for sortable items.
#[component]
pub fn DragAndDropListItems(
    aria_label: String,
    class: Option<String>,
    #[props(default)] children: Option<Element>,
) -> Element {
    let class = cn(&["flex flex-col gap-1", class.as_deref().unwrap_or_default()]);
    rsx! {
        DragAndDropListItemsPrimitive { aria_label, class, children }
    }
}

/// A single draggable/reorderable row.
#[component]
pub fn DragAndDropListItem(
    index: usize,
    #[props(default)] item_key: Option<String>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "flex items-center gap-2 rounded-md border bg-background px-3 py-2 text-sm data-[is-grabbing=true]:opacity-50 data-[focus-visible=true]:ring-2 data-[focus-visible=true]:ring-ring/50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        DragAndDropListItemPrimitive { index, item_key, class, {children} }
    }
}

/// The drop-position indicator rendered between rows.
#[component]
pub fn DragAndDropDropIndicator(
    index: usize,
    position: &'static str,
    class: Option<String>,
) -> Element {
    let class = cn(&[
        "h-0.5 rounded-full bg-primary",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        DragAndDropDropIndicatorPrimitive { index, position, class }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dragging_item_uses_the_semantic_opacity_state() {
        let class = cn(&["data-[is-grabbing=true]:opacity-50", ""]);
        assert!(class.contains("data-[is-grabbing=true]:opacity-50"));
    }
}
