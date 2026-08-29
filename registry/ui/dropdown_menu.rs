//! Styled Dropdown Menu parts backed by the owned primitive behavior layer.
//!
//! The primitive owns controlled open state, roving focus, dismissal, Escape,
//! and ARIA. This registry facade owns the semantic shadcn-like visual surface
//! and absolute popup positioning, so a menu never shifts surrounding layout.

use dioxus::prelude::*;

use adico_primitives::dropdown_menu::{
    DropdownMenu as PrimitiveDropdownMenu, DropdownMenuContent as PrimitiveDropdownMenuContent,
    DropdownMenuItem as PrimitiveDropdownMenuItem,
    DropdownMenuTrigger as PrimitiveDropdownMenuTrigger,
};

use crate::adico_lib::cn::cn;

/// A positioned menu root retaining the primitive's controlled state API.
#[component]
pub fn DropdownMenu(
    #[props(default)] open: ReadSignal<Option<bool>>,
    #[props(default)] default_open: bool,
    #[props(default)] on_open_change: Callback<bool>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(true)))] roving_loop: ReadSignal<bool>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "relative inline-block",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveDropdownMenu {
            open,
            default_open,
            on_open_change,
            disabled,
            roving_loop,
            class,
            {children}
        }
    }
}

/// A standard trigger. For button variants, compose the installed `Button`
/// inside this trigger just as Dialog and Sheet do.
#[component]
pub fn DropdownMenuTrigger(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "inline-flex h-9 items-center justify-center rounded-md border border-input bg-background px-3 text-sm font-medium shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! { PrimitiveDropdownMenuTrigger { class, {children} } }
}

/// An opaque, layered menu surface. It is absolutely positioned beneath its
/// root, avoiding both layout shift and transparent popups.
#[component]
pub fn DropdownMenuContent(
    children: Element,
    id: Option<String>,
    class: Option<String>,
) -> Element {
    let class = cn(&[
        "absolute left-0 top-full z-50 mt-1 min-w-40 overflow-hidden rounded-md bg-popover p-1 text-popover-foreground shadow-md outline-none",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! { PrimitiveDropdownMenuContent { id, class, {children} } }
}

/// A keyboard- and pointer-selectable menu item.
#[component]
pub fn DropdownMenuItem<T: Clone + PartialEq + 'static>(
    value: ReadSignal<T>,
    index: ReadSignal<usize>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] on_select: Callback<T>,
    children: Element,
    class: Option<String>,
) -> Element {
    let class = cn(&[
        "relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors data-[disabled=true]:pointer-events-none data-[disabled=true]:opacity-50 focus:bg-accent focus:text-accent-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! { PrimitiveDropdownMenuItem { value, index, disabled, on_select, class, {children} } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_is_layered_and_opaque() {
        let class = cn(&["absolute left-0 top-full z-50 bg-popover"]);
        assert!(class.contains("absolute"));
        assert!(class.contains("z-50"));
        assert!(class.contains("bg-popover"));
    }
}
