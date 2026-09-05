//! Source-owned shadcn-style Context Menu composition for Dioxus, backed by
//! the owned adico primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::context_menu::ContextMenu;
use adico_primitives::context_menu::{
    ContextMenuContent as ContextMenuPrimitiveContent, ContextMenuItem as ContextMenuPrimitiveItem,
    ContextMenuTrigger as ContextMenuPrimitiveTrigger,
};

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

/// The element that opens the [`ContextMenuContent`] on right-click or long-press.
#[component]
pub fn ContextMenuTrigger(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&["select-none", class.as_deref().unwrap_or_default()]);
    rsx! {
        ContextMenuPrimitiveTrigger { class, attributes, {children} }
    }
}

/// Styled content backed by the owned Context Menu positioning/dismissal/roving-focus primitive.
#[component]
pub fn ContextMenuContent(
    children: Element,
    id: Option<String>,
    #[props(default = Radius::Md)] radius: Radius,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "z-50 min-w-[8rem] overflow-hidden border bg-popover p-1 text-popover-foreground shadow-md data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ContextMenuPrimitiveContent { id, class, attributes, {children} }
    }
}

/// The visual treatment of a [`ContextMenuItem`], matching shadcn's own
/// `"default" | "destructive"` cva axis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ContextMenuItemVariant {
    #[default]
    Default,
    Destructive,
}

impl ContextMenuItemVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Destructive => "text-destructive focus:bg-destructive/10 focus:text-destructive",
        }
    }
}

/// A single selectable entry in a [`ContextMenuContent`].
#[component]
pub fn ContextMenuItem(
    value: ReadSignal<String>,
    index: ReadSignal<usize>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] on_select: Callback<String>,
    /// Indents the item to align with sibling items that have a leading
    /// icon, matching shadcn's own boolean toggle.
    #[props(default)]
    inset: bool,
    #[props(default)] variant: ContextMenuItemVariant,
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors focus:bg-accent focus:text-accent-foreground",
        variant.class(),
        if inset { "pl-8" } else { "" },
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ContextMenuPrimitiveItem {
            value,
            index,
            disabled,
            on_select,
            class,
            attributes,
            {children}
        }
    }
}
