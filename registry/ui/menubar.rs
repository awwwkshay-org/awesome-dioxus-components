//! Source-owned shadcn-style Menubar composition for Dioxus, backed by the
//! owned adico primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::menubar::MenubarMenu;
use adico_primitives::menubar::{
    Menubar as MenubarPrimitive, MenubarContent as MenubarPrimitiveContent,
    MenubarItem as MenubarPrimitiveItem, MenubarTrigger as MenubarPrimitiveTrigger,
};

use crate::adico_lib::cn::cn;

/// The horizontal bar containing one or more [`MenubarMenu`] entries.
#[component]
pub fn Menubar(
    #[props(default)] disabled: ReadSignal<bool>,
    children: Element,
    class: Option<String>,
) -> Element {
    let class = cn(&[
        "flex h-9 items-center gap-1 rounded-md border bg-background p-1",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        MenubarPrimitive { disabled, class, {children} }
    }
}

/// The button that opens a [`MenubarMenu`]'s [`MenubarContent`].
#[component]
pub fn MenubarTrigger(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex cursor-default select-none items-center rounded-sm px-3 py-1 text-sm font-medium outline-none focus:bg-accent focus:text-accent-foreground data-[state=open]:bg-accent data-[state=open]:text-accent-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        MenubarPrimitiveTrigger { class, {children} }
    }
}

/// Styled content backed by the owned Menubar positioning/roving-focus primitive.
#[component]
pub fn MenubarContent(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "z-50 min-w-[12rem] overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        MenubarPrimitiveContent { class, style: "position: absolute;", {children} }
    }
}

/// A single selectable entry in a [`MenubarContent`].
#[component]
pub fn MenubarItem(
    index: ReadSignal<usize>,
    value: String,
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
        MenubarPrimitiveItem { index, value, disabled, on_select, class, {children} }
    }
}
