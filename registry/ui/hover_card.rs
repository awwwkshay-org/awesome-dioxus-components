//! Source-owned shadcn-style Hover Card composition for Dioxus, backed by
//! the owned adico primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::hover_card::HoverCard;
use adico_primitives::hover_card::{
    HoverCardContent as HoverCardPrimitiveContent, HoverCardTrigger as HoverCardPrimitiveTrigger,
};
pub use adico_primitives::{ContentAlign, ContentSide};

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;
use adico_primitives::scroll_area::scroll_area_visibility_class;

/// The element that shows the [`HoverCardContent`] on hover or focus.
#[component]
pub fn HoverCardTrigger(
    children: Element,
    id: Option<String>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "inline-flex items-center justify-center underline-offset-4 hover:underline",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        HoverCardPrimitiveTrigger { id, class, attributes, {children} }
    }
}

/// Styled content backed by the owned Hover Card hover/focus primitive.
#[component]
pub fn HoverCardContent(
    children: Element,
    id: Option<String>,
    #[props(default = Radius::Md)] radius: Radius,
    class: Option<String>,
    side: Option<ContentSide>,
    align: Option<ContentAlign>,
    /// Keep content mounted while closed. Defaults to `false`, matching
    /// Tooltip/Popover's own behavior -- the primitive's own default is
    /// `true` (for consumers that want to keep content mounted for exit
    /// animations).
    #[props(default = false)]
    force_mount: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let side = side.unwrap_or(ContentSide::Bottom);
    let align = align.unwrap_or(ContentAlign::Center);
    let class = cn(&[
        // Same viewport-relative gutter clamp as `popover.rs` -- see its
        // comment for why: no positioner width variable exists, and this is
        // a `fixed`, positioner-anchored element, so `100%` is the viewport.
        "z-50 w-64 max-w-[calc(100%-2rem)] max-h-[var(--adico-positioner-available-size)] overflow-y-auto border bg-popover p-4 text-popover-foreground shadow-md outline-none data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
        radius.class(),
        scroll_area_visibility_class(false),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        HoverCardPrimitiveContent {
            id,
            class,
            side,
            align,
            force_mount,
            attributes,
            {children}
        }
    }
}
