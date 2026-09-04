//! Source-owned shadcn-style Navigation Menu composition for Dioxus, backed
//! by the owned adico primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::{ContentAlign, ContentSide};
use adico_primitives::{
    icons::ChevronDown,
    navigation_menu::{
        NavigationMenuContent as NavigationMenuPrimitiveContent,
        NavigationMenuItem as NavigationMenuPrimitiveItem,
        NavigationMenuLink as NavigationMenuPrimitiveLink,
        NavigationMenuList as NavigationMenuPrimitiveList,
        NavigationMenuRoot as NavigationMenuPrimitiveRoot,
        NavigationMenuTrigger as NavigationMenuPrimitiveTrigger,
    },
};

use crate::adico_lib::cn::cn;

/// The root of a navigation menu: a row of [`NavigationMenuItem`]s.
#[component]
pub fn NavigationMenu(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "relative z-10 flex max-w-max flex-1 items-center justify-center",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        NavigationMenuPrimitiveRoot { class, {children} }
    }
}

/// The row of top-level items in a [`NavigationMenu`].
#[component]
pub fn NavigationMenuList(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "group flex flex-1 list-none items-center justify-center gap-1",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        NavigationMenuPrimitiveList { class, {children} }
    }
}

/// A single top-level entry in a [`NavigationMenuList`].
#[component]
pub fn NavigationMenuItem(
    index: ReadSignal<usize>,
    #[props(default)] disabled: ReadSignal<bool>,
    children: Element,
    class: Option<String>,
) -> Element {
    rsx! {
        NavigationMenuPrimitiveItem { index, disabled, class, {children} }
    }
}

/// Opens the ancestor [`NavigationMenuItem`]'s [`NavigationMenuContent`] on
/// hover-intent or click.
#[component]
pub fn NavigationMenuTrigger(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "group inline-flex h-9 w-max items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground disabled:pointer-events-none disabled:opacity-50 data-[state=open]:bg-accent/50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        NavigationMenuPrimitiveTrigger { class,
            {children}
            ChevronDown {
                class: "relative top-px ml-1 size-3 transition duration-200 group-data-[state=open]:rotate-180",
                "aria-hidden": "true",
            }
        }
    }
}

/// The floating content opened by a [`NavigationMenuTrigger`].
#[component]
pub fn NavigationMenuContent(
    children: Element,
    class: Option<String>,
    side: Option<ContentSide>,
    align: Option<ContentAlign>,
) -> Element {
    let side = side.unwrap_or(ContentSide::Bottom);
    let align = align.unwrap_or(ContentAlign::Start);
    let class = cn(&[
        "min-w-[12rem] rounded-md border bg-popover p-4 text-popover-foreground shadow-md data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        NavigationMenuPrimitiveContent { class, side, align, {children} }
    }
}

/// A navigable item, usable either as a top-level activator (in place of a
/// [`NavigationMenuTrigger`]/[`NavigationMenuContent`] pair) or nested
/// inside a [`NavigationMenuContent`] as a sub-navigation link.
#[component]
pub fn NavigationMenuLink(
    #[props(default)] active: ReadSignal<bool>,
    href: Option<String>,
    children: Element,
    class: Option<String>,
) -> Element {
    let class = cn(&[
        "block select-none space-y-1 rounded-md p-3 text-sm leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground data-[active=true]:bg-accent/50 data-[active=true]:text-accent-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        NavigationMenuPrimitiveLink { active, href, class, {children} }
    }
}
