//! Source-owned shadcn-style Sheet for Dioxus, composing the owned Dialog
//! primitive under a slide-in styled variant.

use dioxus::prelude::*;

use super::button::{Button, ButtonSize, ButtonVariant};
use crate::adico_lib::cn::cn;
use adico_primitives::dialog::{DialogContent as DialogPrimitiveContent, DialogCtx};
pub use adico_primitives::dialog::{
    DialogDescription as SheetDescription, DialogRoot as Sheet, DialogTitle as SheetTitle,
};
use adico_primitives::icons::X;

/// The viewport edge a [`Sheet`] slides in from.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SheetSide {
    /// Slides in from the top edge.
    Top,
    /// Slides in from the right edge (the default).
    #[default]
    Right,
    /// Slides in from the bottom edge.
    Bottom,
    /// Slides in from the left edge.
    Left,
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

/// Opens the surrounding [`Sheet`] with the installed [`Button`] component.
#[component]
pub fn SheetTrigger(
    children: Element,
    class: Option<String>,
    variant: Option<ButtonVariant>,
    size: Option<ButtonSize>,
) -> Element {
    let context: DialogCtx = use_context();
    rsx! {
        Button {
            class,
            variant: variant.unwrap_or_default(),
            size: size.unwrap_or_default(),
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
        "fixed inset-0 z-50 bg-black/50 data-[state=open]:animate-in data-[state=closed]:animate-out",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div {
            class,
            "aria-hidden": "true",
            "data-adico-sheet-overlay": "true",
            onclick: move |_| context.set_open(false),
        }
    }
}

/// Styled content backed by the owned Dialog focus, dismissal, and ARIA primitive,
/// positioned along the chosen [`SheetSide`].
///
/// Renders a corner close button by default (`show_close_button`, matching
/// upstream shadcn's own `showCloseButton = true` default) -- previously
/// entirely absent here, the same real, missing capability found in
/// `dialog.rs`.
#[component]
pub fn SheetContent(
    children: Element,
    class: Option<String>,
    side: Option<SheetSide>,
    #[props(default = true)] show_close_button: bool,
) -> Element {
    let side = side.unwrap_or_default();
    let class = cn(&[
        "fixed z-[51] gap-4 bg-background p-6 text-foreground shadow-lg transition ease-in-out",
        side.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        DialogPrimitiveContent {
            class,
            {children}
            if show_close_button {
                SheetClose { class: "absolute right-4 top-4" }
            }
        }
    }
}

/// A dismissible close control for a [`Sheet`]. Composable anywhere inside
/// [`SheetContent`], not just the default corner close button
/// [`SheetContent`] renders -- the same rationale as `dialog.rs`'s
/// `DialogClose`: previously, closing a sheet from inside its own content
/// required reaching into `adico_primitives::dialog::DialogCtx` directly.
#[component]
pub fn SheetClose(children: Option<Element>, class: Option<String>) -> Element {
    let context: DialogCtx = use_context();
    let class = cn(&[
        "rounded-xs opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        button {
            r#type: "button",
            class,
            onclick: move |_| context.set_open(false),
            match children {
                Some(children) => rsx! { {children} },
                None => rsx! {
                    X { class: "size-4" }
                    span { class: "sr-only", "Close" }
                },
            }
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
