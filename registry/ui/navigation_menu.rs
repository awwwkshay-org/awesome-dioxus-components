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
use crate::adico_lib::variants::Radius;
use adico_primitives::scroll_area::scroll_area_visibility_class;

/// The root of a navigation menu: a row of [`NavigationMenuItem`]s.
#[component]
pub fn NavigationMenu(
    children: Element,
    class: Option<String>,
    #[props(default)] disabled: ReadSignal<bool>,
    /// Milliseconds to wait after the pointer enters a trigger before
    /// opening its content, when nothing else is currently open.
    #[props(default = ReadSignal::new(Signal::new(200)))]
    delay_ms: ReadSignal<u64>,
    /// Milliseconds to wait after the pointer leaves a trigger (or its
    /// content) before closing it.
    #[props(default = ReadSignal::new(Signal::new(150)))]
    close_delay_ms: ReadSignal<u64>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "relative z-10 flex max-w-max flex-1 items-center justify-center",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        NavigationMenuPrimitiveRoot {
            class,
            disabled,
            delay_ms,
            close_delay_ms,
            attributes,
            {children}
        }
    }
}

/// The row of top-level items in a [`NavigationMenu`].
#[component]
pub fn NavigationMenuList(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "group flex flex-1 list-none items-center justify-center gap-1",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        NavigationMenuPrimitiveList { class, attributes, {children} }
    }
}

/// A single top-level entry in a [`NavigationMenuList`].
#[component]
pub fn NavigationMenuItem(
    index: ReadSignal<usize>,
    #[props(default)] disabled: ReadSignal<bool>,
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    rsx! {
        NavigationMenuPrimitiveItem {
            index,
            disabled,
            class,
            attributes,
            {children}
        }
    }
}

/// Opens the ancestor [`NavigationMenuItem`]'s [`NavigationMenuContent`] on
/// hover-intent or click.
#[component]
pub fn NavigationMenuTrigger(
    children: Element,
    /// Corner radius. Set the same value on [`NavigationMenuContent`]/
    /// [`NavigationMenuLink`] for a visually consistent family — there is
    /// no shared context between them to thread one value automatically.
    #[props(default = Radius::Md)]
    radius: Radius,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "group inline-flex h-9 w-max items-center justify-center bg-background px-4 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground disabled:pointer-events-none disabled:opacity-50 data-[state=open]:bg-accent/50",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        NavigationMenuPrimitiveTrigger { class, attributes,
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
    /// Corner radius. See [`NavigationMenuTrigger::radius`]'s own doc comment.
    #[props(default = Radius::Md)]
    radius: Radius,
    class: Option<String>,
    side: Option<ContentSide>,
    align: Option<ContentAlign>,
    /// Whether to keep the content mounted even when closed. Defaults to
    /// `true`, matching the primitive's own default.
    #[props(default = true)]
    force_mount: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let side = side.unwrap_or(ContentSide::Bottom);
    let align = align.unwrap_or(ContentAlign::Start);
    let class = cn(&[
        "z-50 min-w-[12rem] max-h-[var(--adico-positioner-available-size)] overflow-y-auto border bg-popover p-4 text-popover-foreground shadow-md data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
        radius.class(),
        scroll_area_visibility_class(false),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        NavigationMenuPrimitiveContent {
            class,
            side,
            align,
            force_mount,
            attributes,
            {children}
        }
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
    /// Corner radius. See [`NavigationMenuTrigger::radius`]'s own doc comment.
    #[props(default = Radius::Md)]
    radius: Radius,
    /// Whether selecting this link closes any open content. Defaults to
    /// `true`, matching the primitive's own default.
    #[props(default = true)]
    close_on_click: bool,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "block select-none space-y-1 p-3 text-sm leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground data-[active=true]:bg-accent/50 data-[active=true]:text-accent-foreground",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        NavigationMenuPrimitiveLink {
            active,
            href,
            close_on_click,
            class,
            attributes,
            {children}
        }
    }
}
