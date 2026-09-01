//! Source-owned shadcn-style ButtonGroup for Dioxus.

use dioxus::prelude::*;

use adico_primitives::separator::Separator as SeparatorPrimitive;

use crate::adico_lib::cn::cn;

/// The layout axis for a [`ButtonGroup`]'s children.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonGroupOrientation {
    /// Children are laid out left-to-right with adjoining vertical borders.
    #[default]
    Horizontal,
    /// Children are stacked top-to-bottom with adjoining horizontal borders.
    Vertical,
}

impl ButtonGroupOrientation {
    fn class(self) -> &'static str {
        match self {
            Self::Horizontal => {
                "[&>*:not(:first-child)]:rounded-l-none [&>*:not(:first-child)]:border-l-0 [&>*:not(:last-child)]:rounded-r-none"
            }
            Self::Vertical => {
                "flex-col [&>*:not(:first-child)]:rounded-t-none [&>*:not(:first-child)]:border-t-0 [&>*:not(:last-child)]:rounded-b-none"
            }
        }
    }

    fn attr(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// Visually joins related buttons (or other form controls, e.g. [`ButtonGroupText`])
/// into one cluster with shared, adjoining borders and radii.
#[component]
pub fn ButtonGroup(
    #[props(default)] orientation: ButtonGroupOrientation,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "flex w-fit items-stretch has-[>[data-slot=button-group]]:gap-2 [&>*]:focus-visible:relative [&>*]:focus-visible:z-10 has-[select[aria-hidden=true]:last-child]:[&>[data-slot=select-trigger]:last-of-type]:rounded-r-md [&>[data-slot=select-trigger]:not([class*='w-'])]:w-fit [&>input]:flex-1",
        orientation.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div {
            role: "group",
            "data-slot": "button-group",
            "data-orientation": orientation.attr(),
            class,
            {children}
        }
    }
}

/// Non-interactive text sharing a [`ButtonGroup`]'s joined border, e.g. a
/// fixed unit label ("$", "@", "km") adjoining a `Button`.
#[component]
pub fn ButtonGroupText(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "flex items-center gap-2 rounded-md border bg-muted px-4 text-sm font-medium shadow-xs [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// A divider between clusters within a [`ButtonGroup`], composing the owned
/// `adico_primitives::separator::Separator` primitive. Vertical by default,
/// matching a horizontal `ButtonGroup`'s own default orientation.
#[component]
pub fn ButtonGroupSeparator(#[props(default)] horizontal: bool, class: Option<String>) -> Element {
    let class = cn(&[
        "relative m-0! self-stretch bg-input data-[orientation=vertical]:h-auto",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        SeparatorPrimitive { class, horizontal, decorative: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_and_vertical_orientations_join_opposite_edges() {
        assert!(
            ButtonGroupOrientation::Horizontal
                .class()
                .contains("border-l-0")
        );
        assert!(
            ButtonGroupOrientation::Vertical
                .class()
                .contains("border-t-0")
        );
    }

    #[test]
    fn text_slot_shares_the_group_s_muted_bordered_surface() {
        let class = cn(&["rounded-md border bg-muted"]);
        assert!(class.contains("bg-muted"));
    }
}
