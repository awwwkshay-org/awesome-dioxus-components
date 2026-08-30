//! Source-owned shadcn-style Context Menu composition for Dioxus, backed by
//! the owned adico primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::context_menu::ContextMenu;
use adico_primitives::context_menu::{
    ContextMenuContent as ContextMenuPrimitiveContent, ContextMenuItem as ContextMenuPrimitiveItem,
    ContextMenuTrigger as ContextMenuPrimitiveTrigger,
};

use crate::adico_lib::cn::cn;

/// The element that opens the [`ContextMenuContent`] on right-click or long-press.
#[component]
pub fn ContextMenuTrigger(children: Element, class: Option<String>) -> Element {
    let class = cn(&["select-none", class.as_deref().unwrap_or_default()]);
    rsx! {
        ContextMenuPrimitiveTrigger { class, {children} }
    }
}

/// Styled content backed by the owned Context Menu positioning/dismissal/roving-focus primitive.
#[component]
pub fn ContextMenuContent(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "z-50 min-w-[8rem] overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ContextMenuPrimitiveContent { class, {children} }
    }
}

/// A single selectable entry in a [`ContextMenuContent`].
#[component]
pub fn ContextMenuItem(
    value: ReadSignal<String>,
    index: ReadSignal<usize>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] on_select: Callback<String>,
    children: Element,
    class: Option<String>,
) -> Element {
    let class = cn(&[
        "relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors focus:bg-accent focus:text-accent-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ContextMenuPrimitiveItem { value, index, disabled, on_select, class, {children} }
    }
}
