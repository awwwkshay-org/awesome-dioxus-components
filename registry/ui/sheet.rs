//! Source-owned shadcn-style Sheet for Dioxus, composing the owned Dialog
//! primitive under a slide-in styled variant.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;
use adico_primitives::dialog::{DialogContent as DialogPrimitiveContent, DialogCtx};
pub use adico_primitives::dialog::{
    DialogDescription as SheetDescription, DialogRoot as Sheet, DialogTitle as SheetTitle,
};

/// The viewport edge a [`Sheet`] slides in from.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SheetSide {
    /// Slides in from the top edge.
    Top,
    /// Slides in from the right edge (the default).
    Right,
    /// Slides in from the bottom edge.
    Bottom,
    /// Slides in from the left edge.
    Left,
}

impl Default for SheetSide {
    fn default() -> Self {
        SheetSide::Right
    }
}

impl SheetSide {
    fn class(self) -> &'static str {
        match self {
            SheetSide::Top => {
                "inset-x-0 top-0 border-b data-[state=closed]:slide-out-to-top data-[state=open]:slide-in-from-top"
            }
            SheetSide::Right => {
                "inset-y-0 right-0 h-full w-3/4 border-l sm:max-w-sm data-[state=closed]:slide-out-to-right data-[state=open]:slide-in-from-right"
            }
            SheetSide::Bottom => {
                "inset-x-0 bottom-0 border-t data-[state=closed]:slide-out-to-bottom data-[state=open]:slide-in-from-bottom"
            }
            SheetSide::Left => {
                "inset-y-0 left-0 h-full w-3/4 border-r sm:max-w-sm data-[state=closed]:slide-out-to-left data-[state=open]:slide-in-from-left"
            }
        }
    }
}

/// Opens the surrounding [`Sheet`] through the owned headless primitive.
#[component]
pub fn SheetTrigger(children: Element, class: Option<String>) -> Element {
    let context: DialogCtx = use_context();
    let class = cn(&[
        "inline-flex items-center justify-center",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        button {
            class,
            onclick: move |_| context.set_open(true),
            {children}
        }
    }
}

/// A visual overlay rendered only while the surrounding Sheet is open.
#[component]
pub fn SheetOverlay(class: Option<String>) -> Element {
    let context: DialogCtx = use_context();
    if !context.is_open() {
        return rsx! {};
    }
    let class = cn(&[
        "fixed inset-0 z-50 bg-black/80 data-[state=open]:animate-in data-[state=closed]:animate-out",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        // Mounted only while the sheet is visible, mirroring the Dialog overlay.
        style { "html {{ overflow: hidden; }}" }
        div {
            class,
            "aria-hidden": "true",
            "data-adico-sheet-overlay": "true",
            style: "position: fixed; inset: 0; z-index: 50;",
            onclick: move |_| context.set_open(false),
        }
    }
}

/// Styled content backed by the owned Dialog focus, dismissal, and ARIA primitive,
/// positioned along the chosen [`SheetSide`].
#[component]
pub fn SheetContent(children: Element, class: Option<String>, side: Option<SheetSide>) -> Element {
    let side = side.unwrap_or_default();
    let class = cn(&[
        "fixed z-50 gap-4 bg-background p-6 text-foreground shadow-lg transition ease-in-out",
        side.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        DialogPrimitiveContent {
            class,
            style: "position: fixed; z-index: 51;",
            {children}
        }
    }
}

/// A semantic header helper for Sheet titles and descriptions.
#[component]
pub fn SheetHeader(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex flex-col space-y-2 text-center sm:text-left",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// A footer region typically used for Sheet actions.
#[component]
pub fn SheetFooter(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}
