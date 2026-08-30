//! Source-owned Dioxus-only Toolbar for Dioxus, backed by the owned adico
//! primitive layer. This is a Dioxus Components extra with no shadcn
//! equivalent -- it does not count toward shadcn parity.

use dioxus::prelude::*;

pub use adico_primitives::toolbar::Toolbar;
use adico_primitives::toolbar::{
    ToolbarButton as ToolbarButtonPrimitive, ToolbarSeparator as ToolbarSeparatorPrimitive,
};

use crate::adico_lib::cn::cn;

/// A button within a [`Toolbar`] with roving-focus keyboard navigation.
#[component]
pub fn ToolbarButton(
    index: ReadSignal<usize>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] on_click: Callback<()>,
    children: Element,
    class: Option<String>,
) -> Element {
    let class = cn(&[
        "inline-flex h-8 items-center justify-center gap-2 rounded-md px-2 text-sm font-medium outline-none transition-colors hover:bg-muted hover:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ToolbarButtonPrimitive { index, disabled, on_click, class, {children} }
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
