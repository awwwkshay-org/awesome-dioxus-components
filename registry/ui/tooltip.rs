//! Source-owned shadcn-style Tooltip composition for Dioxus, backed by the
//! owned adico primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::tooltip::Tooltip;
use adico_primitives::tooltip::{
    TooltipContent as TooltipPrimitiveContent, TooltipTrigger as TooltipPrimitiveTrigger,
};
pub use adico_primitives::{ContentAlign, ContentSide};

use crate::adico_lib::cn::cn;

/// The element that shows the [`TooltipContent`] on hover or focus.
#[component]
pub fn TooltipTrigger(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "inline-flex items-center justify-center",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TooltipPrimitiveTrigger { class, {children} }
    }
}

/// Styled content backed by the owned Tooltip ARIA/hover/focus primitive.
#[component]
pub fn TooltipContent(
    children: Element,
    class: Option<String>,
    side: Option<ContentSide>,
    align: Option<ContentAlign>,
) -> Element {
    let side = side.unwrap_or(ContentSide::Top);
    let align = align.unwrap_or(ContentAlign::Center);
    let class = cn(&[
        "z-50 overflow-hidden rounded-md border bg-popover px-3 py-1.5 text-xs text-popover-foreground shadow-md",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TooltipPrimitiveContent {
            class,
            side,
            align,
            style: "position: absolute; z-index: 50;",
            {children}
        }
    }
}
