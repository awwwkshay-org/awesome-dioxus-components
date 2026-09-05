//! Source-owned shadcn-style Popover composition for Dioxus, backed by the
//! owned adico primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::popover::PopoverRoot as Popover;
use adico_primitives::popover::{
    PopoverContent as PopoverPrimitiveContent, PopoverTrigger as PopoverPrimitiveTrigger,
};
pub use adico_primitives::{ContentAlign, ContentSide};

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

/// Opens the surrounding [`Popover`] through the owned headless primitive.
#[component]
pub fn PopoverTrigger(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "inline-flex items-center justify-center",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PopoverPrimitiveTrigger { class, attributes, {children} }
    }
}

/// Styled content backed by the owned Popover focus, dismissal, and ARIA primitive.
#[component]
pub fn PopoverContent(
    children: Element,
    id: Option<String>,
    #[props(default = Radius::Md)] radius: Radius,
    class: Option<String>,
    side: Option<ContentSide>,
    align: Option<ContentAlign>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let side = side.unwrap_or(ContentSide::Bottom);
    let align = align.unwrap_or(ContentAlign::Center);
    let class = cn(&[
        "z-50 w-72 border bg-popover p-4 text-popover-foreground shadow-md outline-none data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PopoverPrimitiveContent {
            id,
            class,
            side,
            align,
            attributes,
            {children}
        }
    }
}
