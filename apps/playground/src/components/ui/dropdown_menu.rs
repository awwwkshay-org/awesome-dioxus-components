//! Styled Dropdown Menu parts backed by the owned primitive behavior layer.
//!
//! The primitive owns controlled open state, roving focus, dismissal, Escape,
//! and ARIA. This registry facade owns the semantic shadcn-like visual surface
//! and absolute popup positioning, so a menu never shifts surrounding layout.

use dioxus::prelude::*;

use adico_primitives::dropdown_menu::{
    DropdownMenu as PrimitiveDropdownMenu, DropdownMenuContent as PrimitiveDropdownMenuContent,
    DropdownMenuItem as PrimitiveDropdownMenuItem,
    DropdownMenuTrigger as PrimitiveDropdownMenuTrigger,
};
use adico_primitives::icons::{Check, ChevronRight, Circle};
use adico_primitives::menu::{
    MenuCheckboxItem as PrimitiveMenuCheckboxItem, MenuContent as PrimitiveMenuContent,
    MenuGroup as PrimitiveMenuGroup, MenuGroupLabel as PrimitiveMenuGroupLabel,
    MenuRadioGroup as PrimitiveMenuRadioGroup, MenuRadioItem as PrimitiveMenuRadioItem,
    MenuSeparator as PrimitiveMenuSeparator, MenuSubmenuRoot as PrimitiveMenuSubmenuRoot,
    MenuSubmenuTrigger as PrimitiveMenuSubmenuTrigger,
};
use adico_primitives::{ContentAlign, ContentSide};

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

/// The shared item visual base, kept identical across [`DropdownMenuItem`],
/// [`DropdownMenuCheckboxItem`], [`DropdownMenuRadioItem`], and
/// [`DropdownMenuSubTrigger`] so every selectable row in one menu reads the
/// same.
const ITEM_BASE_CLASS: &str = "relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors data-[disabled=true]:pointer-events-none data-[disabled=true]:opacity-50 focus:bg-accent focus:text-accent-foreground";

/// A positioned menu root retaining the primitive's controlled state API.
#[component]
pub fn DropdownMenu(
    #[props(default)] open: ReadSignal<Option<bool>>,
    #[props(default)] default_open: bool,
    #[props(default)] on_open_change: Callback<bool>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(true)))] roving_loop: ReadSignal<bool>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&[
        "relative inline-block",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveDropdownMenu {
            open,
            default_open,
            on_open_change,
            disabled,
            roving_loop,
            class,
            attributes,
            {children}
        }
    }
}

/// A standard trigger, already styled as a default-variant button on its own
/// real `<button>` element (unlike `DialogTrigger`/`SheetTrigger`, which
/// have no independent DOM identity and *are* a `Button`). **Do not** nest
/// the installed `Button` inside this trigger -- that produces an invalid
/// `<button><button>` pair, since this trigger's underlying primitive
/// (`crate::menu::MenuTrigger`) owns its own button semantics
/// (`aria-haspopup`/`aria-expanded`/keyboard handling) and renders a real
/// `<button>` of its own. Pass icon/text children directly, overriding
/// `class` for a different visual treatment -- see `mode_toggle.rs`'s
/// `DropdownMenuTrigger` (icon-only, `h-9 w-9 justify-center px-0`) for the
/// established pattern. (Corrected 2026-09-04: this doc previously
/// recommended nesting `Button` here, which does not match any real shipped
/// consumer -- `mode_toggle.rs` was already the correct precedent.)
#[component]
pub fn DropdownMenuTrigger(
    children: Element,
    /// Corner radius. Set the same value on [`DropdownMenuContent`] for a
    /// visually consistent trigger/popup pair — there is no shared context
    /// between them to thread one value automatically.
    #[props(default = Radius::Md)]
    radius: Radius,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "inline-flex h-9 items-center justify-center border border-input bg-background px-3 text-sm font-medium shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! { PrimitiveDropdownMenuTrigger { class, attributes, {children} } }
}

/// An opaque, layered menu surface. It is positioned beneath its trigger by
/// the shared `Positioner` primitive (`position: fixed`, computed viewport
/// coordinates), not by CSS `absolute`/`top-full` positioning classes.
#[component]
pub fn DropdownMenuContent(
    children: Element,
    id: Option<String>,
    /// Corner radius. See [`DropdownMenuTrigger::radius`]'s own doc comment.
    #[props(default = Radius::Md)]
    radius: Radius,
    /// Alignment of the content relative to its trigger.
    #[props(default = ContentAlign::Start)]
    align: ContentAlign,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "z-50 min-w-40 overflow-hidden bg-popover p-1 text-popover-foreground shadow-md outline-none",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! { PrimitiveDropdownMenuContent { id, class, align, attributes, {children} } }
}

/// The visual treatment of a [`DropdownMenuItem`], matching shadcn's own
/// `"default" | "destructive"` cva axis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DropdownMenuItemVariant {
    #[default]
    Default,
    Destructive,
}

impl DropdownMenuItemVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Destructive => "text-destructive focus:bg-destructive/10 focus:text-destructive",
        }
    }
}

/// A keyboard- and pointer-selectable menu item.
#[component]
pub fn DropdownMenuItem<T: Clone + PartialEq + 'static>(
    value: ReadSignal<T>,
    index: ReadSignal<usize>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] on_select: Callback<T>,
    /// Display text registered with the enclosing menu's typeahead search.
    /// Omit to keep this item out of typeahead matching entirely.
    #[props(default)]
    text_value: ReadSignal<Option<String>>,
    /// Indents the item to align with sibling items that have a leading
    /// icon, matching shadcn's own boolean toggle.
    #[props(default)]
    inset: bool,
    #[props(default)] variant: DropdownMenuItemVariant,
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        ITEM_BASE_CLASS,
        variant.class(),
        if inset { "pl-8" } else { "" },
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveDropdownMenuItem {
            value,
            index,
            disabled,
            on_select,
            text_value,
            class,
            attributes,
            {children}
        }
    }
}

/// A purely visual/ARIA grouping of related items — typically a
/// [`DropdownMenuLabel`] followed by [`DropdownMenuItem`]s. Does not affect
/// keyboard navigation ordering.
#[component]
pub fn DropdownMenuGroup(
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&[class.as_deref().unwrap_or_default()]);
    rsx! { PrimitiveMenuGroup { class, attributes, {children} } }
}

/// A non-interactive heading for a [`DropdownMenuGroup`] (or a whole menu).
#[component]
pub fn DropdownMenuLabel(
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
pub fn DropdownMenuSeparator(
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "-mx-1 my-1 h-px bg-border",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! { PrimitiveMenuSeparator { class, attributes } }
}

/// A trailing keyboard-shortcut hint (e.g. `⇧⌘P`) for a menu item. Purely
/// presentational — no primitive backs this and it registers no key binding,
/// matching upstream shadcn's own `DropdownMenuShortcut` (a styled `<span>`;
/// same precedent as the `command` registry item's `CommandShortcut`).
#[component]
pub fn DropdownMenuShortcut(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "ml-auto text-xs tracking-widest text-muted-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        span { class, {children} }
    }
}

/// A menu item with a checked state, toggled on select without closing the
/// menu. Controlled via `checked`/`on_checked_change` or uncontrolled via
/// `default_checked`. The check indicator is driven purely by the
/// primitive's own `data-state` attribute.
#[component]
pub fn DropdownMenuCheckboxItem(
    index: ReadSignal<usize>,
    #[props(default)] checked: ReadSignal<Option<bool>>,
    #[props(default)] default_checked: bool,
    #[props(default)] on_checked_change: Callback<bool>,
    #[props(default)] disabled: ReadSignal<bool>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&[
        ITEM_BASE_CLASS,
        "group pl-8",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveMenuCheckboxItem {
            index,
            checked,
            default_checked,
            on_checked_change,
            disabled,
            class,
            attributes,
            span { class: "pointer-events-none absolute left-2 flex size-3.5 items-center justify-center",
                Check { class: "size-4 opacity-0 group-data-[state=checked]:opacity-100" }
            }
            {children}
        }
    }
}

/// Groups [`DropdownMenuRadioItem`]s into a single-select set. Controlled
/// via `value`/`on_value_change` or uncontrolled via `default_value`.
#[component]
pub fn DropdownMenuRadioGroup<T: Clone + PartialEq + 'static>(
    #[props(default)] value: Option<ReadSignal<Option<T>>>,
    #[props(default)] default_value: Option<T>,
    #[props(default)] on_value_change: Callback<T>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&[class.as_deref().unwrap_or_default()]);
    rsx! {
        PrimitiveMenuRadioGroup::<T> {
            value,
            default_value,
            on_value_change,
            class,
            attributes,
            {children}
        }
    }
}

/// A single-select item within a [`DropdownMenuRadioGroup`]. Selecting it
/// sets the group's value and closes the menu. The dot indicator is driven
/// purely by the primitive's own `data-state` attribute.
#[component]
pub fn DropdownMenuRadioItem<T: Clone + PartialEq + 'static>(
    value: ReadSignal<T>,
    index: ReadSignal<usize>,
    #[props(default)] disabled: ReadSignal<bool>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&[
        ITEM_BASE_CLASS,
        "group pl-8",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveMenuRadioItem::<T> {
            value,
            index,
            disabled,
            class,
            attributes,
            span { class: "pointer-events-none absolute left-2 flex size-3.5 items-center justify-center",
                Circle { class: "size-2 fill-current opacity-0 group-data-[state=checked]:opacity-100" }
            }
            {children}
        }
    }
}

/// A nested submenu root. Acts as an item in the parent menu's keyboard
/// order (via `index`) while hosting its own [`DropdownMenuSubTrigger`] and
/// [`DropdownMenuSubContent`]. Nesting composes to any depth.
#[component]
pub fn DropdownMenuSub(
    index: ReadSignal<usize>,
    #[props(default)] open: ReadSignal<Option<bool>>,
    #[props(default)] default_open: bool,
    #[props(default)] on_open_change: Callback<bool>,
    #[props(default)] disabled: ReadSignal<bool>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&["relative", class.as_deref().unwrap_or_default()]);
    rsx! {
        PrimitiveMenuSubmenuRoot {
            index,
            open,
            default_open,
            on_open_change,
            disabled,
            class,
            attributes,
            {children}
        }
    }
}

/// The trigger row for a [`DropdownMenuSub`], styled like a regular item
/// with a trailing chevron. Opens on click, `ArrowRight`, or hover intent.
#[component]
pub fn DropdownMenuSubTrigger(
    /// Indents the trigger to align with sibling inset items.
    #[props(default)]
    inset: bool,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&[
        ITEM_BASE_CLASS,
        "data-[state=open]:bg-accent data-[state=open]:text-accent-foreground",
        if inset { "pl-8" } else { "" },
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveMenuSubmenuTrigger { class, attributes,
            {children}
            ChevronRight { class: "ml-auto size-4" }
        }
    }
}

/// A [`DropdownMenuSub`]'s flyout surface, opening to the trigger's right
/// (the primitive's positioner handles placement and collision flipping).
#[component]
pub fn DropdownMenuSubContent(
    /// Corner radius. See [`DropdownMenuTrigger`]'s own doc comment.
    #[props(default = Radius::Md)]
    radius: Radius,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&[
        "z-50 min-w-32 overflow-hidden bg-popover p-1 text-popover-foreground shadow-lg",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveMenuContent { side: ContentSide::Right, class, attributes, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_is_layered_and_opaque() {
        // Positioning is `Positioner`'s inline `position: fixed`, not a CSS
        // `absolute`/`top-full` class -- see `DropdownMenuContent`'s own
        // class string and doc comment.
        let class = cn(&["z-50 bg-popover"]);
        assert!(class.contains("z-50"));
        assert!(class.contains("bg-popover"));
    }
}
