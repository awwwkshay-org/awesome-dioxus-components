//! Source-owned shadcn-style Tooltip composition for Dioxus, backed by the
//! owned adico primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::tooltip::Tooltip;
use adico_primitives::tooltip::{
    TooltipContent as TooltipPrimitiveContent, TooltipTrigger as TooltipPrimitiveTrigger,
};
pub use adico_primitives::{ContentAlign, ContentSide};

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

/// The element that shows the [`TooltipContent`] on hover or focus.
#[component]
pub fn TooltipTrigger(
    children: Element,
    id: Option<String>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "inline-flex items-center justify-center",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TooltipPrimitiveTrigger { id, class, attributes, {children} }
    }
}

/// Styled content backed by the owned Tooltip ARIA/hover/focus primitive.
#[component]
pub fn TooltipContent(
    children: Element,
    id: Option<String>,
    #[props(default = Radius::Md)] radius: Radius,
    class: Option<String>,
    side: Option<ContentSide>,
    align: Option<ContentAlign>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let side = side.unwrap_or(ContentSide::Top);
    let align = align.unwrap_or(ContentAlign::Center);
    let class = cn(&[
        "z-50 overflow-hidden border bg-popover px-3 py-1.5 text-xs text-popover-foreground shadow-md animate-in fade-in-0 zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TooltipPrimitiveContent {
            id,
            class,
            side,
            align,
            attributes,
            {children}
        }
    }
}
