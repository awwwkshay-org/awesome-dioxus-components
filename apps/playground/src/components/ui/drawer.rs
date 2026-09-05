//! Source-owned shadcn-style Drawer for Dioxus, composing the owned Dialog
//! primitive under an edge-anchored, mobile-oriented styled variant.
//!
//! Upstream shadcn's `drawer` is a wrapper around `vaul` (a real drag-to-dismiss
//! gesture library). This registry item deliberately does **not** attempt a
//! drag-to-dismiss gesture: it composes the same `dialog.rs` primitive
//! `sheet.rs` already does, so it gets a real, correct static open/close/
//! focus-trap/Escape/backdrop-dismiss foundation, but dragging the handle or
//! swiping the content will not close it. This is a named, deliberate scope
//! reduction, not an oversight -- see the M7 task audit
//! (`openspec/changes/build-adico-component-ecosystem/tasks.md`, task 8.1)
//! for why: a real drag gesture would depend on `pointer.rs`'s global
//! pointer-position registry, which is a documented, unconfirmed-in-browser
//! defect on `web` (see `gesture.rs`'s own module doc comment), and building
//! it correctly instead means bypassing that registry entirely with
//! per-element `onpointerdown`/`onpointermove`/`onpointerup` plus
//! `setPointerCapture` -- real, scoped-out follow-up work, not something to
//! bolt on silently here.

use dioxus::prelude::*;

use super::button::{Button, ButtonSize, ButtonVariant};
use crate::adico_lib::cn::cn;
use adico_primitives::dialog::{DialogContent as DialogPrimitiveContent, DialogCtx};
pub use adico_primitives::dialog::{
    DialogDescription as DrawerDescription, DialogRoot as Drawer, DialogTitle as DrawerTitle,
};
use adico_primitives::icons::X;

/// The viewport edge a [`Drawer`] slides in from.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DrawerDirection {
    /// Slides in from the top edge; shows the grab handle.
    Top,
    /// Slides in from the bottom edge (the default); shows the grab handle.
    #[default]
    Bottom,
    /// Slides in from the left edge.
    Left,
    /// Slides in from the right edge.
    Right,
}

impl DrawerDirection {
    fn class(self) -> &'static str {
        match self {
            DrawerDirection::Top => {
                "inset-x-0 top-0 rounded-b-[10px] border-b data-[state=closed]:slide-out-to-top data-[state=open]:slide-in-from-top"
            }
            DrawerDirection::Bottom => {
                "inset-x-0 bottom-0 mt-24 max-h-[80vh] rounded-t-[10px] border-t data-[state=closed]:slide-out-to-bottom data-[state=open]:slide-in-from-bottom"
            }
            DrawerDirection::Left => {
                "inset-y-0 left-0 h-full w-3/4 rounded-r-[10px] border-r sm:max-w-sm data-[state=closed]:slide-out-to-left data-[state=open]:slide-in-from-left"
            }
            DrawerDirection::Right => {
                "inset-y-0 right-0 h-full w-3/4 rounded-l-[10px] border-l sm:max-w-sm data-[state=closed]:slide-out-to-right data-[state=open]:slide-in-from-right"
            }
        }
    }

    /// The handle bar only appears on the two directions vaul shows it for
    /// upstream: `Top` and `Bottom`, where the panel edge it's attached to is
    /// horizontal.
    fn shows_handle(self) -> bool {
        matches!(self, DrawerDirection::Top | DrawerDirection::Bottom)
    }
}

/// Opens the surrounding [`Drawer`] with the installed [`Button`] component.
#[component]
pub fn DrawerTrigger(
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

/// A visual overlay rendered only while the surrounding Drawer is open.
#[component]
pub fn DrawerOverlay(class: Option<String>) -> Element {
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
            "data-adico-drawer-overlay": "true",
            onclick: move |_| context.set_open(false),
        }
    }
}

/// Styled content backed by the owned Dialog focus, dismissal, and ARIA
/// primitive, positioned along the chosen [`DrawerDirection`] with a grab
/// handle bar on `Top`/`Bottom` (purely visual -- see this module's doc
/// comment on why it isn't draggable).
///
/// Deliberately has no `radius` prop: every real corner here is either
/// [`DrawerDirection::class()`]'s side-specific, direction-dependent value
/// (`rounded-{t,b,l,r}-[10px]`, none representable by a bare
/// `Radius::class()` string) or decorative-internal (`DrawerClose`, the
/// grab-handle bar).
#[component]
pub fn DrawerContent(
    children: Element,
    class: Option<String>,
    direction: Option<DrawerDirection>,
    #[props(default = true)] show_close_button: bool,
) -> Element {
    let direction = direction.unwrap_or_default();
    let class = cn(&[
        "fixed z-[51] flex flex-col gap-4 bg-background p-6 text-foreground shadow-lg transition ease-in-out",
        direction.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        DialogPrimitiveContent {
            class,
            if direction.shows_handle() {
                div { class: "mx-auto mt-4 h-2 w-[100px] shrink-0 rounded-full bg-muted", "aria-hidden": "true" }
            }
            {children}
            if show_close_button {
                DrawerClose { class: "absolute right-4 top-4" }
            }
        }
    }
}

/// A dismissible close control for a [`Drawer`]. Composable anywhere inside
/// [`DrawerContent`], not just the default corner close button
/// [`DrawerContent`] renders.
#[component]
pub fn DrawerClose(children: Option<Element>, class: Option<String>) -> Element {
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

/// A semantic header helper for Drawer titles and descriptions.
#[component]
pub fn DrawerHeader(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex flex-col gap-1.5 p-4 text-center sm:text-left",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// A footer region typically used for Drawer actions.
#[component]
pub fn DrawerFooter(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "mt-auto flex flex-col gap-2 p-4",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}
