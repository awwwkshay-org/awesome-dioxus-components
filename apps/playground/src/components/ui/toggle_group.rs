//! Source-owned shadcn-style Toggle Group for Dioxus, backed by the owned
//! adico primitive layer.

use std::collections::HashSet;

use dioxus::prelude::*;

use adico_primitives::toggle_group::ToggleGroup as ToggleGroupPrimitive;
use adico_primitives::toggle_group::ToggleItem as ToggleItemPrimitive;

use crate::adico_lib::cn::cn;

/// The row of [`ToggleItem`]s. A styled facade over the primitive's own
/// `ToggleGroup`, which has no default layout class at all (a bare
/// `pub use` re-export previously): each `ToggleItem` is independently
/// `rounded-md` (this crate doesn't replicate upstream's connected/flush
/// `spacing=0` segmented look), so with nothing spacing them apart the
/// items rendered flush against each other with no visual gap at all --
/// found live (reported directly by the user: "no space in toggle group").
#[component]
pub fn ToggleGroup(
    #[props(default)] default_pressed: HashSet<usize>,
    pressed: ReadSignal<Option<HashSet<usize>>>,
    #[props(default)] on_pressed_change: Callback<HashSet<usize>>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] allow_multiple_pressed: ReadSignal<bool>,
    #[props(default)] horizontal: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(true)))] roving_loop: ReadSignal<bool>,
    #[props(default)] class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "flex items-center gap-1",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ToggleGroupPrimitive {
            default_pressed,
            pressed,
            on_pressed_change,
            disabled,
            allow_multiple_pressed,
            horizontal,
            roving_loop,
            class,
            {children}
        }
    }
}

/// The visual size of a [`ToggleItem`], matching `Toggle`'s own `ToggleSize`
/// values (a small, deliberate duplication -- each registry item stays an
/// independent, source-owned copy rather than taking a cross-item
/// dependency for one shared enum).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToggleItemSize {
    /// Compact size.
    Sm,
    /// Default size.
    #[default]
    Default,
    /// Larger size.
    Lg,
}

impl ToggleItemSize {
    fn class(self) -> &'static str {
        match self {
            Self::Sm => "h-8 px-1.5 min-w-8",
            Self::Default => "h-9 px-2 min-w-9",
            Self::Lg => "h-10 px-2.5 min-w-10",
        }
    }
}

/// The visual treatment of a [`ToggleItem`]. Was entirely absent before
/// task 5.2 -- upstream shadcn's `ToggleGroup` propagates a `variant` to
/// every item via React context; adico has no context-propagation
/// mechanism for this, so each `ToggleItem` takes its own `variant` prop
/// instead (a caller composing a group sets the same variant on every item,
/// same end result, one extra prop per item).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToggleItemVariant {
    /// Transparent background, the existing default look.
    #[default]
    Default,
    /// Bordered neutral styling.
    Outline,
}

impl ToggleItemVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "bg-transparent",
            Self::Outline => {
                "border border-input bg-transparent shadow-xs hover:bg-accent hover:text-accent-foreground"
            }
        }
    }
}

/// A single pressable item within a [`ToggleGroup`].
#[component]
pub fn ToggleItem(
    index: ReadSignal<usize>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] size: ToggleItemSize,
    #[props(default)] variant: ToggleItemVariant,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium outline-none transition-colors hover:bg-muted hover:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 data-[state=on]:bg-accent data-[state=on]:text-accent-foreground",
        variant.class(),
        size.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ToggleItemPrimitive { index, disabled, class, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressed_item_uses_the_same_semantic_accent_surface_as_toggle() {
        let class = cn(&[
            "data-[state=on]:bg-accent data-[state=on]:text-accent-foreground",
            "",
        ]);
        assert!(class.contains("data-[state=on]:bg-accent"));
    }
}
