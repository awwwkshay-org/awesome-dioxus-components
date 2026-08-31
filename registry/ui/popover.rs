//! Source-owned shadcn-style Popover composition for Dioxus, backed by the
//! owned adico primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::popover::PopoverRoot as Popover;
use adico_primitives::popover::{
    PopoverContent as PopoverPrimitiveContent, PopoverTrigger as PopoverPrimitiveTrigger,
};
pub use adico_primitives::{ContentAlign, ContentSide};

use crate::adico_lib::cn::cn;

/// Opens the surrounding [`Popover`] through the owned headless primitive.
#[component]
pub fn PopoverTrigger(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "inline-flex items-center justify-center",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PopoverPrimitiveTrigger { class, {children} }
    }
}

/// Styled content backed by the owned Popover focus, dismissal, and ARIA primitive.
#[component]
pub fn PopoverContent(
    children: Element,
    class: Option<String>,
    side: Option<ContentSide>,
    align: Option<ContentAlign>,
) -> Element {
    let side = side.unwrap_or(ContentSide::Bottom);
    let align = align.unwrap_or(ContentAlign::Center);
    let class = cn(&[
        "z-50 w-72 rounded-md border bg-popover p-4 text-popover-foreground shadow-md outline-none data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PopoverPrimitiveContent {
            class,
            side,
            align,
            {children}
        }
    }
}
