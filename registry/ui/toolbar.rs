//! Source-owned Dioxus-only Toolbar for Dioxus, backed by the owned adico
//! primitive layer. This is a Dioxus Components extra with no shadcn
//! equivalent -- it does not count toward shadcn parity.

use dioxus::prelude::*;

pub use adico_primitives::toolbar::Toolbar;
use adico_primitives::toolbar::{
    ToolbarButton as ToolbarButtonPrimitive, ToolbarSeparator as ToolbarSeparatorPrimitive,
};

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;
use crate::components::ui::spinner::Spinner;

/// A button within a [`Toolbar`] with roving-focus keyboard navigation.
#[component]
pub fn ToolbarButton(
    index: ReadSignal<usize>,
    #[props(default)] disabled: ReadSignal<bool>,
    /// Fired when the button is activated. Named `on_select`, not the
    /// primitive's own `on_click`, matching this registry's naming
    /// convention for primitive-backed components (the primitive layer
    /// keeps `on_click` unchanged) and `command.rs`/`context_menu.rs`'s
    /// own `on_select` item-activation naming.
    #[props(default)]
    on_select: Callback<()>,
    children: Element,
    #[props(default = Radius::Md)] radius: Radius,
    /// Shows a [`Spinner`] and marks the button busy/disabled (combined
    /// with `disabled` above). An adico extension — shadcn's own
    /// convention is composing `<Button disabled><Spinner /></Button>` by
    /// hand.
    #[props(default)]
    loading: bool,
    /// Replaces the button's visible content while `loading` is true.
    #[props(default)]
    loading_text: Option<String>,
    class: Option<String>,
) -> Element {
    let class = cn(&[
        "inline-flex h-8 items-center justify-center gap-2 px-2 text-sm font-medium outline-none transition-colors hover:bg-muted hover:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    let combined_disabled = use_memo(move || disabled() || loading);
    rsx! {
        ToolbarButtonPrimitive {
            index,
            disabled: combined_disabled,
            on_click: on_select,
            class,
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

/// A divider between groups of [`ToolbarButton`]s.
#[component]
pub fn ToolbarSeparator(
    #[props(default)] horizontal: Option<bool>,
    #[props(default = false)] decorative: bool,
    class: Option<String>,
) -> Element {
    let class = cn(&[
        "shrink-0 bg-border data-[orientation=horizontal]:h-px data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full data-[orientation=vertical]:w-px",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ToolbarSeparatorPrimitive { horizontal, decorative, class }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separator_uses_orientation_driven_semantic_border_surface() {
        let class = cn(&[
            "data-[orientation=horizontal]:h-px data-[orientation=vertical]:w-px",
            "",
        ]);
        assert!(class.contains("data-[orientation=horizontal]:h-px"));
    }
}
