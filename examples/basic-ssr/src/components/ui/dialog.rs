//! Source-owned shadcn-style Dialog composition for Dioxus.

use dioxus::prelude::*;

use super::button::{Button, ButtonSize, ButtonVariant};
use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;
pub use adico_primitives::dialog::{
    DialogContent as DialogPrimitiveContent, DialogDescription, DialogRoot as Dialog, DialogTitle,
};
use adico_primitives::icons::X;
use adico_primitives::scroll_area::scroll_area_visibility_class;

/// Opens the surrounding [`Dialog`] with the installed [`Button`] component.
///
/// This keeps dialog triggers visually and behaviorally consistent with every
/// other action in a consumer application while the Dialog primitive continues
/// to own focus, Escape, outside-dismissal, and ARIA behavior.
#[component]
pub fn DialogTrigger(
    children: Element,
    class: Option<String>,
    variant: Option<ButtonVariant>,
    size: Option<ButtonSize>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    attributes: Vec<Attribute>,
) -> Element {
    let context: adico_primitives::dialog::DialogCtx = use_context();
    rsx! {
        Button {
            class,
            variant: variant.unwrap_or_default(),
            size: size.unwrap_or_default(),
            onclick: move |_| context.set_open(true),
            attributes,
            {children}
        }
    }
}

/// A visual overlay rendered only while the surrounding Dialog is open.
#[component]
pub fn DialogOverlay(class: Option<String>) -> Element {
    let context: adico_primitives::dialog::DialogCtx = use_context();
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
            "data-adico-dialog-overlay": "true",
            onclick: move |_| context.set_open(false),
        }
    }
}

/// Styled content backed by the owned Dialog focus, dismissal, and ARIA primitive.
///
/// Renders a corner close button by default (`show_close_button`, matching
/// upstream shadcn's own `showCloseButton = true` default) -- previously
/// entirely absent here, a real, missing capability: without it, a sighted
/// mouse user has no visible way to close the dialog short of knowing to
/// click the backdrop or press Escape.
#[component]
pub fn DialogContent(
    children: Element,
    id: Option<String>,
    #[props(default)] radius: Radius,
    class: Option<String>,
    #[props(default = true)] show_close_button: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        // `max-h-[calc(100svh-2rem)]` + `min-h-0` cap the dialog to the viewport with
        // a small margin (matches the exact cap the playground's theme-builder
        // launcher previously had to apply caller-side to work around this gap --
        // that workaround is removed now that it's redundant, see `theme_builder_launcher.rs`).
        // Not anchored via `Positioner` (a centered modal has no anchor element), so
        // this is a fixed viewport-relative cap, not `--adico-positioner-available-size`.
        "fixed left-1/2 top-1/2 z-[51] grid max-h-[calc(100svh-2rem)] w-full max-w-lg min-h-0 -translate-x-1/2 -translate-y-1/2 border bg-background p-6 text-foreground shadow-lg",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    // `flex flex-col gap-4` on the scrolling body preserves the vertical spacing
    // between `DialogTitle`/`DialogDescription`/caller content that the outer
    // container's own `gap-4` used to provide directly between its grid items --
    // wrapping them in this one body div for scrolling would otherwise collapse
    // that spacing to zero.
    let body_class = cn(&[
        "flex min-h-0 flex-col gap-4 overflow-y-auto",
        scroll_area_visibility_class(false),
    ]);
    rsx! {
        DialogPrimitiveContent {
            id,
            class,
            attributes,
            div { class: body_class, {children} }
            if show_close_button {
                DialogClose { class: "absolute right-4 top-4" }
            }
        }
    }
}

/// A semantic header helper for Dialog titles and descriptions.
#[component]
pub fn DialogHeader(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "flex flex-col space-y-1.5 text-center sm:text-left",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! { div { class, ..attributes, {children} } }
}

/// A semantic footer helper, typically for [`Dialog`] action buttons.
/// Previously missing here entirely, even though [`DialogHeader`] already
/// existed -- a real, asymmetric composition gap against upstream, which
/// has always paired the two.
#[component]
pub fn DialogFooter(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! { div { class, ..attributes, {children} } }
}

/// A dismissible close control for a [`Dialog`]. Composable anywhere inside
/// [`DialogContent`] (for example, a "Cancel" button in a [`DialogFooter`]),
/// not just the default corner close button [`DialogContent`] renders.
/// Previously, closing a dialog from inside its own content required
/// reaching into `adico_primitives::dialog::DialogCtx` directly -- a
/// primitive-internals leak this component now avoids.
#[component]
pub fn DialogClose(
    children: Option<Element>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    attributes: Vec<Attribute>,
) -> Element {
    let context: adico_primitives::dialog::DialogCtx = use_context();
    let class = cn(&[
        "rounded-xs opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        button {
            r#type: "button",
            class,
            onclick: move |_| context.set_open(false),
            ..attributes,
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
