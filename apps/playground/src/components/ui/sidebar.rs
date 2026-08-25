//! Source-owned shadcn-style Sidebar composition for Dioxus.
//!
//! Unlike most registry items, upstream ships `Sidebar` as a styled preview
//! component (not a headless `adico-primitives` module): its only shared
//! state is a plain open/closed signal, which this facade gets directly from
//! `adico_primitives::use_controlled` (the same controlled/uncontrolled
//! helper `dialog`/`sheet`/etc. already use internally).
//!
//! Adapted from upstream: the reference implementation detects mobile
//! viewports with a `document::eval` `while let Ok(result) = eval.recv()...`
//! loop and swaps in a `Sheet` overlay. That is the same long-lived,
//! repeatedly-firing `document::eval` pattern already found non-functional
//! in this Dioxus runtime while testing the Wave 3 overlay batch (see
//! `docs/adico/m3-wave3-migration.md`). Rather than ship a silently broken
//! mobile mode, this initial pass renders one collapsible layout driven by
//! CSS (`data-state`/`data-collapsible` attributes plus Tailwind transition
//! classes) and defers a real viewport-driven mobile sheet mode to M4/M5
//! hardening once that gap has a real fix.

use dioxus::prelude::*;

use adico_primitives::{separator::Separator as SeparatorPrimitive, use_controlled};

use crate::adico_lib::cn::cn;

/// The side of the viewport a [`Sidebar`] is docked to.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum SidebarSide {
    #[default]
    Left,
    Right,
}

impl SidebarSide {
    fn as_str(self) -> &'static str {
        match self {
            SidebarSide::Left => "left",
            SidebarSide::Right => "right",
        }
    }
}

/// How a [`Sidebar`] behaves when collapsed.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum SidebarCollapsible {
    /// Collapses fully off-canvas (width 0).
    #[default]
    Offcanvas,
    /// Collapses to an icon-only rail.
    Icon,
    /// Never collapses.
    None,
}

impl SidebarCollapsible {
    fn as_str(self) -> &'static str {
        match self {
            SidebarCollapsible::Offcanvas => "offcanvas",
            SidebarCollapsible::Icon => "icon",
            SidebarCollapsible::None => "none",
        }
    }
}

#[derive(Clone, Copy)]
struct SidebarCtx {
    open: Memo<bool>,
    set_open: Callback<bool>,
}

impl SidebarCtx {
    fn toggle(&self) {
        self.set_open.call(!self.open.cloned());
    }
}

fn use_sidebar() -> SidebarCtx {
    use_context::<SidebarCtx>()
}

/// The props for the [`SidebarProvider`] component.
#[derive(Props, Clone, PartialEq)]
pub struct SidebarProviderProps {
    /// The controlled open state.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,
    /// The default open state when uncontrolled.
    #[props(default = true)]
    pub default_open: bool,
    /// Callback fired when the open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,
    /// Additional attributes for the wrapper element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the provider, typically a [`Sidebar`] and [`SidebarInset`].
    pub children: Element,
}

/// Provides shared open/closed state to every [`Sidebar`] part beneath it.
#[component]
pub fn SidebarProvider(props: SidebarProviderProps) -> Element {
    let (open, set_open) = use_controlled(props.open, props.default_open, props.on_open_change);
    use_context_provider(|| SidebarCtx { open, set_open });

    let class = cn(&["flex min-h-svh w-full"]);
    rsx! {
        div {
            class,
            "data-slot": "sidebar-wrapper",
            style: "--sidebar-width: 16rem; --sidebar-width-icon: 3rem;",
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`Sidebar`] component.
#[derive(Props, Clone, PartialEq)]
pub struct SidebarProps {
    /// Which side of the viewport the sidebar docks to.
    #[props(default)]
    pub side: SidebarSide,
    /// How the sidebar behaves when collapsed.
    #[props(default)]
    pub collapsible: SidebarCollapsible,
    /// Additional CSS classes to append.
    #[props(default)]
    pub class: Option<String>,
    /// The children of the sidebar, typically [`SidebarHeader`]/[`SidebarContent`]/[`SidebarFooter`].
    pub children: Element,
}

/// The collapsible sidebar panel itself.
///
/// ## Styling
///
/// Defines `data-state` (`expanded`/`collapsed`), `data-side`, and
/// `data-collapsible` attributes for consumer styling hooks.
#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    let ctx = use_sidebar();
    let open = ctx.open;
    let state = if open() { "expanded" } else { "collapsed" };
    let collapsible = props.collapsible.as_str();

    let width_class = match (open(), props.collapsible) {
        (true, _) => "w-[--sidebar-width]",
        (false, SidebarCollapsible::Icon) => "w-[--sidebar-width-icon]",
        (false, SidebarCollapsible::None) => "w-[--sidebar-width]",
        (false, SidebarCollapsible::Offcanvas) => "w-0 overflow-hidden border-transparent",
    };
    let side_class = match props.side {
        SidebarSide::Left => "left-0 border-r",
        SidebarSide::Right => "right-0 border-l",
    };

    let class = cn(&[
        "flex h-svh flex-col bg-sidebar text-sidebar-foreground shrink-0 transition-[width] duration-200 ease-linear",
        width_class,
        side_class,
        props.class.as_deref().unwrap_or_default(),
    ]);

    rsx! {
        aside {
            class,
            "data-slot": "sidebar",
            "data-state": state,
            "data-side": props.side.as_str(),
            "data-collapsible": collapsible,
            {props.children}
        }
    }
}

/// The button that toggles a [`Sidebar`] open and closed.
#[component]
pub fn SidebarTrigger(children: Element, class: Option<String>) -> Element {
    let ctx = use_sidebar();
    let class = cn(&[
        "inline-flex h-7 w-7 items-center justify-center rounded-md text-sm hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        button {
            class,
            r#type: "button",
            "data-slot": "sidebar-trigger",
            aria_label: "Toggle Sidebar",
            onclick: move |_| ctx.toggle(),
            {children}
        }
    }
}

/// A thin edge rail that also toggles the [`Sidebar`], for pointer users
/// who prefer dragging the boundary over pressing the explicit trigger.
#[component]
pub fn SidebarRail(class: Option<String>) -> Element {
    let ctx = use_sidebar();
    let class = cn(&[
        "absolute inset-y-0 z-20 w-4 -translate-x-1/2 cursor-col-resize bg-transparent hover:after:bg-sidebar-border after:absolute after:inset-y-0 after:left-1/2 after:w-px",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        button {
            class,
            r#type: "button",
            "data-slot": "sidebar-rail",
            aria_label: "Toggle Sidebar",
            tabindex: -1,
            title: "Toggle Sidebar",
            onclick: move |_| ctx.toggle(),
        }
    }
}

/// The main content area beside the [`Sidebar`].
#[component]
pub fn SidebarInset(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "relative flex w-full flex-1 flex-col bg-background",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        main { class, "data-slot": "sidebar-inset", {children} }
    }
}

/// A header region pinned to the top of a [`Sidebar`].
#[component]
pub fn SidebarHeader(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex flex-col gap-2 p-2",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, "data-slot": "sidebar-header", {children} }
    }
}

/// The scrollable main region of a [`Sidebar`].
#[component]
pub fn SidebarContent(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex min-h-0 flex-1 flex-col gap-2 overflow-auto p-2",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, "data-slot": "sidebar-content", {children} }
    }
}

/// A footer region pinned to the bottom of a [`Sidebar`].
#[component]
pub fn SidebarFooter(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex flex-col gap-2 p-2",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, "data-slot": "sidebar-footer", {children} }
    }
}

/// A horizontal rule between [`Sidebar`] sections, composing the owned
/// `adico_primitives::separator::Separator` primitive.
#[component]
pub fn SidebarSeparator(class: Option<String>) -> Element {
    let class = cn(&[
        "mx-2 w-auto bg-sidebar-border",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        SeparatorPrimitive { class, horizontal: true, decorative: true }
    }
}

/// A labeled group of related [`SidebarMenu`] items.
#[component]
pub fn SidebarGroup(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "relative flex w-full min-w-0 flex-col p-2",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, "data-slot": "sidebar-group", {children} }
    }
}

/// The label heading a [`SidebarGroup`].
#[component]
pub fn SidebarGroupLabel(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-medium text-sidebar-foreground/70",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, "data-slot": "sidebar-group-label", {children} }
    }
}

/// The content wrapper inside a [`SidebarGroup`].
#[component]
pub fn SidebarGroupContent(children: Element, class: Option<String>) -> Element {
    let class = cn(&["w-full text-sm", class.as_deref().unwrap_or_default()]);
    rsx! {
        div { class, "data-slot": "sidebar-group-content", {children} }
    }
}

/// The list container for [`SidebarMenuItem`]s.
#[component]
pub fn SidebarMenu(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex w-full min-w-0 flex-col gap-1",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ul { class, "data-slot": "sidebar-menu", {children} }
    }
}

/// A single entry in a [`SidebarMenu`].
#[component]
pub fn SidebarMenuItem(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "group/menu-item relative",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        li { class, "data-slot": "sidebar-menu-item", {children} }
    }
}

/// The clickable/navigable control inside a [`SidebarMenuItem`].
#[component]
pub fn SidebarMenuButton(
    children: Element,
    #[props(default)] is_active: bool,
    class: Option<String>,
) -> Element {
    let class = cn(&[
        "flex h-8 w-full items-center gap-2 overflow-hidden rounded-md px-2 text-left text-sm outline-none transition-[width,height,padding] hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        button {
            class,
            r#type: "button",
            "data-slot": "sidebar-menu-button",
            "data-active": is_active,
            {children}
        }
    }
}
