//! Source-owned shadcn-style Toast notifications for Dioxus, backed by the
//! owned adico primitive layer.
//!
//! Only the toast region's fixed viewport placement is styled by default
//! here (structural, not merely cosmetic -- an unpositioned toast list
//! renders inline in normal document flow and is unusable). Individual toast
//! cards keep the primitive's own unstyled composition, matching this
//! registry's existing "many parts, re-export unstyled" precedent for
//! `select`/`combobox`; a full default visual style for [`Toast`] itself is
//! M4's job. Consumers can supply their own styled `render_toast` callback
//! today by composing [`ToastContent`], [`ToastTitle`], [`ToastDescription`],
//! and [`ToastCloseButton`] with Tailwind classes.

use dioxus::prelude::*;

use adico_primitives::toast::ToastProvider as ToastProviderPrimitive;
pub use adico_primitives::toast::{
    Toast, ToastCloseButton, ToastContent, ToastDescription, ToastList, ToastListItem,
    ToastOptions, ToastProps, ToastPropsWithOwner, ToastTitle, ToastType, Toasts, consume_toast,
    use_toast,
};
use std::time::Duration;

use crate::adico_lib::cn::cn;

/// Wraps the application (or a subtree of it) with toast notification
/// support, positioning the rendered toast region fixed to the bottom-right
/// of the viewport.
#[component]
pub fn ToastProvider(
    #[props(default = Duration::from_secs(5))] default_duration: Duration,
    #[props(default = 10)] max_toasts: usize,
    children: Element,
) -> Element {
    let class = cn(&[
        "fixed bottom-4 right-4 z-[100] flex max-h-screen w-full flex-col gap-2 sm:max-w-[420px]",
    ]);
    rsx! {
        ToastProviderPrimitive {
            default_duration: Some(default_duration),
            max_toasts,
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
}
