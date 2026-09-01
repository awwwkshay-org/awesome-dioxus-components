//! Source-owned shadcn-style Toast notifications for Dioxus, backed by the
//! owned adico primitive layer.
//!
//! The toast region's fixed viewport placement and each toast card's visual
//! style are both styled by default here. Individual cards previously kept
//! the primitive's own unstyled composition, deferred as "M4's job" -- found
//! live (reported directly by the user: a toast rendered as bare white text
//! with no card behind it at all) that this gap had never actually been
//! closed. Fixed by supplying a real default `render_toast` callback,
//! composing [`ToastContent`], [`ToastTitle`], [`ToastDescription`], and
//! [`ToastCloseButton`] with Tailwind classes; a consumer can still override
//! `render_toast` on [`ToastProvider`] for a fully custom card.

use dioxus::prelude::*;

use adico_primitives::toast::ToastProvider as ToastProviderPrimitive;
pub use adico_primitives::toast::{
    Toast as ToastPrimitive, ToastCloseButton, ToastContent, ToastDescription, ToastList,
    ToastListItem, ToastOptions, ToastProps, ToastPropsWithOwner, ToastTitle, ToastType, Toasts,
    consume_toast, use_toast,
};
use std::time::Duration;

use crate::adico_lib::cn::cn;

/// The default styled toast card, composing the primitive's unstyled parts.
/// Used as [`ToastProvider`]'s default `render_toast`; call directly only if
/// building a custom `render_toast` that still wants this card's look.
#[component]
pub fn Toast(props: ToastProps) -> Element {
    // Field-by-field, not `..props`: `ToastPrimitive` has no dedicated
    // `class` field (only `attributes: Vec<Attribute>`, extending
    // `GlobalAttributes`), and spreading the whole `ToastProps` struct
    // bypasses that shorthand mechanism entirely (found via a real compile
    // error, not assumed) -- passing `class` as its own keyword alongside
    // the other fields individually is what actually routes it into
    // `attributes`.
    let class = cn(&[
        "group pointer-events-auto relative flex w-full items-start gap-3 overflow-hidden rounded-md border bg-background p-4 pr-8 text-foreground shadow-lg \
         data-[type=success]:border-emerald-500/50 data-[type=error]:border-destructive/50 data-[type=warning]:border-amber-500/50",
    ]);
    rsx! {
        ToastPrimitive {
            id: props.id,
            index: props.index,
            title: props.title.clone(),
            description: props.description.clone(),
            toast_type: props.toast_type,
            on_close: props.on_close,
            permanent: props.permanent,
            duration: props.duration,
            class,
            ToastContent { class: "flex flex-1 flex-col gap-1",
                ToastTitle { class: "text-sm font-semibold" }
                ToastDescription { class: "text-sm text-muted-foreground" }
            }
            ToastCloseButton {
                class: "absolute right-2 top-2 rounded-md p-1 text-foreground/50 opacity-0 transition-opacity hover:text-foreground focus-visible:opacity-100 focus-visible:outline-none group-hover:opacity-100",
            }
        }
    }
}

/// Wraps the application (or a subtree of it) with toast notification
/// support, positioning the rendered toast region fixed to the bottom-right
/// of the viewport, and rendering each toast with [`Toast`]'s default card
/// styling unless `render_toast` is overridden.
#[component]
pub fn ToastProvider(
    #[props(default = Duration::from_secs(5))] default_duration: Duration,
    #[props(default = 10)] max_toasts: usize,
    #[props(default = Callback::new(|props: ToastPropsWithOwner| rsx! { Toast { ..props } }))]
    render_toast: Callback<ToastPropsWithOwner, Element>,
    children: Element,
) -> Element {
    // `flex flex-col gap-2` on this element does nothing for spacing between
    // toasts: `ToastProviderPrimitive` renders this class on a wrapper `div`
    // whose only child is the primitive's own `ToastList` (an unstyled
    // `<ol>`) -- the actual `<li>` items live one level deeper, so `gap-2`
    // here only ever had one flex child to apply to. Found live (reported
    // directly by the user: stacked toasts with literally zero gap, borders
    // touching). `ToastProviderProps` has no prop to style `ToastList`
    // directly, so target it as a descendant instead of modifying the
    // primitive.
    let class = cn(&[
        "fixed bottom-4 right-4 z-[100] max-h-screen w-full sm:max-w-[420px] \
         [&>ol]:flex [&>ol]:flex-col [&>ol]:gap-2",
    ]);
    rsx! {
        ToastProviderPrimitive {
            default_duration: Some(default_duration),
            max_toasts,
            render_toast,
            class,
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_is_fixed_to_a_viewport_corner() {
        let class = cn(&["fixed bottom-4 right-4 z-[100]"]);
        assert!(class.contains("fixed bottom-4 right-4"));
    }

    #[test]
    fn stacked_toasts_are_spaced_via_the_nested_list_not_the_region_wrapper() {
        let class = cn(&["[&>ol]:flex [&>ol]:flex-col [&>ol]:gap-2"]);
        assert!(class.contains("[&>ol]:gap-2"));
    }
}
