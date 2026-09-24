//! Source-owned shadcn-style Menubar composition for Dioxus, backed by the
//! owned adico primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::menubar::MenubarMenu;
// `MenuGroup`/`MenuGroupLabel`/`MenuSeparator` consume no menu context, so
// they compose safely inside a `MenubarContent` even though the menubar's
// own primitive scope is independent of `adico_primitives::menu`.
// Checkbox/radio/submenu parts are deliberately NOT re-faced here: they
// require the `MenuContext` only the dropdown-menu root provides and would
// panic at runtime inside a menubar.
use adico_primitives::menu::{
    MenuGroup as PrimitiveMenuGroup, MenuGroupLabel as PrimitiveMenuGroupLabel,
    MenuSeparator as PrimitiveMenuSeparator,
};
use adico_primitives::menubar::{
    Menubar as MenubarPrimitive, MenubarContent as MenubarPrimitiveContent,
    MenubarItem as MenubarPrimitiveItem, MenubarTrigger as MenubarPrimitiveTrigger,
};

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;
use adico_primitives::scroll_area::scroll_area_visibility_class;

/// The horizontal bar containing one or more [`MenubarMenu`] entries.
#[component]
pub fn Menubar(
    #[props(default)] disabled: ReadSignal<bool>,
    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    roving_loop: ReadSignal<bool>,
    children: Element,
    /// Corner radius. Set the same value on [`MenubarContent`] for a
    /// visually consistent bar/popup pair — there is no shared context
    /// between them to thread one value automatically.
    #[props(default = Radius::Md)]
    radius: Radius,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        // `w-full overflow-x-auto` (R4): without an explicit width, a `flex`
        // row just grows to fit its children instead of scrolling within
        // its own bounds -- same reasoning as `TabList`'s identical fix.
        "flex h-9 w-full items-center gap-1 overflow-x-auto border bg-background p-1",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        MenubarPrimitive {
            disabled,
            roving_loop,
            class,
            attributes,
            {children}
        }
    }
}

/// The button that opens a [`MenubarMenu`]'s [`MenubarContent`].
#[component]
pub fn MenubarTrigger(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "flex cursor-default select-none items-center rounded-sm px-3 py-1 text-sm font-medium outline-none focus:bg-accent focus:text-accent-foreground data-[state=open]:bg-accent data-[state=open]:text-accent-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        MenubarPrimitiveTrigger { class, attributes, {children} }
    }
}

/// Styled content backed by the owned Menubar positioning/roving-focus primitive.
#[component]
pub fn MenubarContent(
    children: Element,
    id: Option<String>,
    /// Corner radius. See [`Menubar::radius`]'s own doc comment.
    #[props(default = Radius::Md)]
    radius: Radius,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        // `min-w-[12rem]` alone can force overflow on its own, so it's paired
        // with the same viewport-relative gutter clamp `popover.rs` uses.
        "z-50 min-w-[12rem] max-w-[calc(100%-2rem)] max-h-[var(--adico-positioner-available-size)] overflow-y-auto border bg-popover p-1 text-popover-foreground shadow-md data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
        radius.class(),
        scroll_area_visibility_class(false),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        MenubarPrimitiveContent { id, class, attributes, {children} }
    }
}

/// The visual treatment of a [`MenubarItem`], matching shadcn's own
/// `"default" | "destructive"` cva axis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MenubarItemVariant {
    #[default]
    Default,
    Destructive,
}

impl MenubarItemVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Destructive => "text-destructive focus:bg-destructive/10 focus:text-destructive",
        }
    }
}

/// A single selectable entry in a [`MenubarContent`].
#[component]
pub fn MenubarItem(
    index: ReadSignal<usize>,
    value: String,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] on_select: Callback<String>,
    /// Indents the item to align with sibling items that have a leading
    /// icon, matching shadcn's own boolean toggle.
    #[props(default)]
    inset: bool,
    #[props(default)] variant: MenubarItemVariant,
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
        MenubarPrimitiveItem {
            index,
            value,
            disabled,
            on_select,
            class,
            attributes,
            {children}
        }
    }
}

/// A purely visual/ARIA grouping of related items — typically a
/// [`MenubarLabel`] followed by [`MenubarItem`]s. Does not affect keyboard
/// navigation ordering.
#[component]
pub fn MenubarGroup(
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&[class.as_deref().unwrap_or_default()]);
    rsx! { PrimitiveMenuGroup { class, attributes, {children} } }
}

/// A non-interactive heading for a [`MenubarGroup`] (or a whole menu).
#[component]
pub fn MenubarLabel(
    /// Indents the label to align with inset items, matching shadcn's own
    /// boolean toggle.
    #[props(default)]
    inset: bool,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&[
        "px-2 py-1.5 text-sm font-semibold",
        if inset { "pl-8" } else { "" },
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! { PrimitiveMenuGroupLabel { class, attributes, {children} } }
}

/// A visual divider between menu items or groups.
#[component]
pub fn MenubarSeparator(
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "-mx-1 my-1 h-px bg-border",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! { PrimitiveMenuSeparator { class, attributes } }
}

/// A trailing keyboard-shortcut hint (e.g. `⌘N`) for a menu item. Purely
/// presentational — no primitive backs this and it registers no key binding,
/// matching upstream shadcn's own `MenubarShortcut` (a styled `<span>`; same
/// precedent as the `command` registry item's `CommandShortcut`).
#[component]
pub fn MenubarShortcut(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "ml-auto text-xs tracking-widest text-muted-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        span { class, {children} }
    }
}
