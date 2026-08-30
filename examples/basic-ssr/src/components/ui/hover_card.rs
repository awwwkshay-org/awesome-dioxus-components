//! Source-owned shadcn-style Hover Card composition for Dioxus, backed by
//! the owned adico primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::hover_card::HoverCard;
use adico_primitives::hover_card::{
    HoverCardContent as HoverCardPrimitiveContent, HoverCardTrigger as HoverCardPrimitiveTrigger,
};
pub use adico_primitives::{ContentAlign, ContentSide};

use crate::adico_lib::cn::cn;

/// The element that shows the [`HoverCardContent`] on hover or focus.
#[component]
pub fn HoverCardTrigger(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "inline-flex items-center justify-center underline-offset-4 hover:underline",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        HoverCardPrimitiveTrigger { class, {children} }
    }
}

/// Styled content backed by the owned Hover Card hover/focus primitive.
#[component]
pub fn HoverCardContent(
    children: Element,
    class: Option<String>,
    side: Option<ContentSide>,
    align: Option<ContentAlign>,
) -> Element {
    let side = side.unwrap_or(ContentSide::Bottom);
    let align = align.unwrap_or(ContentAlign::Center);
    let class = cn(&[
        "z-50 w-64 rounded-md border bg-popover p-4 text-popover-foreground shadow-md outline-none",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        HoverCardPrimitiveContent {
            class,
            side,
            align,
            // The primitive defaults `force_mount` to `true` for consumers
            // that want to keep content mounted for exit animations; this
            // styled facade instead mounts content only while open, matching
            // Tooltip/Popover's behavior.
            force_mount: false,
            style: "position: absolute; z-index: 50;",
            {children}
        }
    }
}
