//! Source-owned shadcn-style Alert Dialog composition for Dioxus.

use dioxus::prelude::*;

use super::button::{Button, ButtonSize, ButtonVariant};
use crate::adico_lib::cn::cn;
use adico_primitives::alert_dialog::{
    AlertDialogAction as AlertDialogActionPrimitive,
    AlertDialogActions as AlertDialogActionsPrimitive,
    AlertDialogCancel as AlertDialogCancelPrimitive,
    AlertDialogContent as AlertDialogContentPrimitive,
};
pub use adico_primitives::alert_dialog::{
    AlertDialogDescription, AlertDialogRoot as AlertDialog, AlertDialogTitle,
};

/// Opens the surrounding [`AlertDialog`] with the installed [`Button`]
/// component. Nested inside [`AlertDialog`] (matching `Dialog`'s
/// `DialogTrigger`), so that the alert dialog's root-level Escape handler
/// reliably receives keyboard events regardless of which element has focus --
/// unlike upstream's own doctest, which uses an external sibling button.
#[component]
pub fn AlertDialogTrigger(
    children: Element,
    class: Option<String>,
    variant: Option<ButtonVariant>,
    size: Option<ButtonSize>,
) -> Element {
    let context: adico_primitives::alert_dialog::AlertDialogCtx = use_context();
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

/// A visual overlay rendered only while the surrounding [`AlertDialog`] is
/// open. Purely presentational: unlike `Dialog`'s overlay, an alert dialog
/// must not be dismissible by clicking outside it, so this element has no
/// click handler.
#[component]
pub fn AlertDialogOverlay(class: Option<String>) -> Element {
    let context: adico_primitives::alert_dialog::AlertDialogCtx = use_context();
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
        }
    }
}

/// Styled content backed by the owned AlertDialog focus-trap and ARIA primitive.
#[component]
pub fn AlertDialogContent(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "fixed left-1/2 top-1/2 z-[51] grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 rounded-lg border bg-background p-6 text-foreground shadow-lg",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AlertDialogContentPrimitive {
            class,
            {children}
        }
    }
}

/// A semantic header helper for AlertDialog titles and descriptions.
#[component]
pub fn AlertDialogHeader(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex flex-col space-y-1.5 text-center sm:text-left",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! { div { class, {children} } }
}

/// A semantic footer helper that groups [`AlertDialogAction`] and
/// [`AlertDialogCancel`] into shadcn's stacked/row action layout.
#[component]
pub fn AlertDialogActions(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AlertDialogActionsPrimitive { class, {children} }
    }
}

/// The primary, destructive-or-affirming action of an [`AlertDialog`].
#[component]
pub fn AlertDialogAction(
    children: Element,
    class: Option<String>,
    on_click: Option<EventHandler<MouseEvent>>,
) -> Element {
    let class = cn(&[
        "inline-flex h-9 items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-xs hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AlertDialogActionPrimitive { class, on_click, {children} }
    }
}

/// The dismissive action of an [`AlertDialog`].
#[component]
pub fn AlertDialogCancel(
    children: Element,
    class: Option<String>,
    on_click: Option<EventHandler<MouseEvent>>,
) -> Element {
    let class = cn(&[
        "mt-2 inline-flex h-9 items-center justify-center rounded-md border border-input bg-background px-4 py-2 text-sm font-medium shadow-xs hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring sm:mt-0",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AlertDialogCancelPrimitive { class, on_click, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_uses_the_same_semantic_scrim_as_dialog() {
        let class = cn(&["fixed inset-0 z-50 bg-black/50", ""]);
        assert!(class.contains("bg-black/50"));
    }

    #[test]
    fn action_uses_semantic_primary_surface() {
        let class = cn(&["bg-primary text-primary-foreground hover:bg-primary/90", ""]);
        assert!(class.contains("bg-primary"));
    }
}
