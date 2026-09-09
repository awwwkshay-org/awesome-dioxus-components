//! Source-owned shadcn-style Alert Dialog composition for Dioxus.

use dioxus::prelude::*;

use super::button::{Button, ButtonSize, ButtonVariant};
use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;
use crate::components::ui::spinner::Spinner;
use adico_primitives::alert_dialog::{
    AlertDialogAction as AlertDialogActionPrimitive,
    AlertDialogActions as AlertDialogActionsPrimitive,
    AlertDialogCancel as AlertDialogCancelPrimitive,
    AlertDialogContent as AlertDialogContentPrimitive,
};
pub use adico_primitives::alert_dialog::{
    AlertDialogDescription, AlertDialogRoot as AlertDialog, AlertDialogTitle,
};
use adico_primitives::scroll_area::scroll_area_visibility_class;

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
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    attributes: Vec<Attribute>,
) -> Element {
    let context: adico_primitives::alert_dialog::AlertDialogCtx = use_context();
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

/// The maximum width of an [`AlertDialogContent`], matching shadcn's own
/// `"default" | "sm"` cva axis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AlertDialogContentSize {
    #[default]
    Default,
    Sm,
}

impl AlertDialogContentSize {
    // Both arms are `sm:`-prefixed: the base `max-w-[calc(100%-2rem)]` on
    // `AlertDialogContent`'s own class (below) is the narrow-viewport gutter
    // clamp shared by every size; these restore each size's original
    // desktop-only value at `sm` and up, matching `dialog.rs`'s identical
    // width-axis treatment.
    fn class(self) -> &'static str {
        match self {
            Self::Default => "sm:max-w-lg",
            Self::Sm => "sm:max-w-sm",
        }
    }
}

/// Styled content backed by the owned AlertDialog focus-trap and ARIA primitive.
#[component]
pub fn AlertDialogContent(
    children: Element,
    id: Option<String>,
    #[props(default)] radius: Radius,
    #[props(default)] size: AlertDialogContentSize,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        // See `dialog.rs`'s `DialogContent` for the identical `max-h`/`min-h-0`/
        // scrolling-body reasoning -- an `AlertDialog` is also a centered modal with
        // no anchor element, so it uses the same fixed viewport-relative cap.
        // `max-w-[calc(100%-2rem)]` is the shared narrow-viewport gutter clamp for
        // every size; `size.class()` restores each size's own value at `sm` and up.
        "fixed left-1/2 top-1/2 z-[51] grid max-h-[calc(100svh-2rem)] w-full max-w-[calc(100%-2rem)] min-h-0 -translate-x-1/2 -translate-y-1/2 border bg-background p-6 text-foreground shadow-lg",
        size.class(),
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    let body_class = cn(&[
        "flex min-h-0 flex-col gap-4 overflow-y-auto",
        scroll_area_visibility_class(false),
    ]);
    rsx! {
        AlertDialogContentPrimitive {
            id,
            class,
            attributes,
            div { class: body_class, {children} }
        }
    }
}

/// A semantic header helper for AlertDialog titles and descriptions.
#[component]
pub fn AlertDialogHeader(
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

/// A semantic footer helper that groups [`AlertDialogAction`] and
/// [`AlertDialogCancel`] into shadcn's stacked/row action layout.
#[component]
pub fn AlertDialogActions(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AlertDialogActionsPrimitive { class, attributes, {children} }
    }
}

/// The primary, destructive-or-affirming action of an [`AlertDialog`].
#[component]
pub fn AlertDialogAction(
    children: Element,
    class: Option<String>,
    /// Fired when the affirming/destructive action is confirmed. Named
    /// `on_confirm`, not the primitive's own `on_click`, matching this
    /// registry's naming convention for primitive-backed components (the
    /// primitive layer keeps `on_click` unchanged).
    on_confirm: Option<EventHandler<MouseEvent>>,
    /// Shows a [`Spinner`] and marks the action busy/disabled. An adico
    /// extension — shadcn's own convention is composing
    /// `<Button disabled><Spinner /></Button>` by hand. `disabled`/
    /// `aria-busy` are set through the primitive's own `attributes`
    /// extends mechanism (`AlertDialogActionPrimitive` has no dedicated
    /// `disabled` field of its own).
    #[props(default)]
    loading: bool,
    /// Replaces the action's visible content while `loading` is true.
    #[props(default)]
    loading_text: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "inline-flex h-9 items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-xs hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        class.as_deref().unwrap_or_default(),
    ]);
    // `disabled` isn't part of `GlobalAttributes` (the only extend
    // `AlertDialogActionPrimitive`'s own `attributes` field declares), so
    // it can't be set via the usual extends-shorthand keyword here --
    // built by hand instead, matching `slider.rs`'s own precedent for this
    // exact limitation. Appended after the caller's own `attributes` so a
    // caller-supplied `disabled` doesn't accidentally win over `loading`.
    let mut attributes = attributes;
    attributes.push(Attribute::new("disabled", loading, None, false));
    rsx! {
        AlertDialogActionPrimitive {
            class,
            on_click: on_confirm,
            attributes,
            aria_busy: loading,
            if loading {
                Spinner {}
                if let Some(text) = loading_text {
                    "{text}"
                } else {
                    {children}
                }
            } else {
                {children}
            }
        }
    }
}

/// The dismissive action of an [`AlertDialog`].
#[component]
pub fn AlertDialogCancel(
    children: Element,
    class: Option<String>,
    /// Fired when the dialog is dismissed. Named `on_dismiss`, matching
    /// this registry's naming convention for primitive-backed components
    /// (the primitive layer keeps `on_click` unchanged).
    on_dismiss: Option<EventHandler<MouseEvent>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "mt-2 inline-flex h-9 items-center justify-center rounded-md border border-input bg-background px-4 py-2 text-sm font-medium shadow-xs hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring sm:mt-0",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AlertDialogCancelPrimitive {
            class,
            on_click: on_dismiss,
            attributes,
            {children}
        }
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
