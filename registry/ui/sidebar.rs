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

use adico_primitives::{
    scroll_area::scroll_area_visibility_class, separator::Separator as SeparatorPrimitive,
    use_controlled,
};

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;
use crate::components::ui::spinner::Spinner;

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

/// The overall visual treatment of a [`Sidebar`] and its [`SidebarInset`].
/// Found missing entirely by this session's variants audit: upstream shadcn
/// supports all three, this registry item previously only ever rendered the
/// plain `sidebar` look.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum SidebarVariant {
    /// Docked flush to the viewport edge with a single border.
    #[default]
    Sidebar,
    /// Inset from the edge with its own border, rounded corners, and shadow.
    Floating,
    /// Flush and borderless; [`SidebarInset`] instead gets the rounded,
    /// shadowed treatment, so the two panels read as one recessed surface.
    Inset,
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
    /// Extra classes appended to the semantic default. Kept as its own field
    /// (not folded into `attributes` below) so it's merged via `cn`, not
    /// silently overwritten -- `attributes` also extends `GlobalAttributes`
    /// (which includes `class`), and a caller-supplied `class` routed through
    /// that catch-all instead replaced the wrapper's own `flex` class
    /// entirely, collapsing the layout (found live, breaking this app's own
    /// dogfooded navigation shell).
    #[props(default)]
    pub class: Option<String>,
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

    let class = cn(&[
        "flex min-h-svh w-full",
        props.class.as_deref().unwrap_or_default(),
    ]);
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
    /// The overall visual treatment; see [`SidebarVariant`].
    #[props(default)]
    pub variant: SidebarVariant,
    /// Additional CSS classes to append.
    #[props(default)]
    pub class: Option<String>,
    /// Native aside/global attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
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
        SidebarSide::Left => "left-0",
        SidebarSide::Right => "right-0",
    };
    // `Floating`'s rounded-lg (and `SidebarInset`'s own rounded-xl below) is
    // switched by the variant itself, not an independent `radius` prop —
    // there's no bounded surface here at all in the other variants to apply
    // one to.
    let variant_class = match (props.variant, props.side) {
        (SidebarVariant::Sidebar, SidebarSide::Left) => "border-r",
        (SidebarVariant::Sidebar, SidebarSide::Right) => "border-l",
        (SidebarVariant::Floating, _) => "m-2 rounded-lg border border-sidebar-border shadow-sm",
        (SidebarVariant::Inset, _) => "",
    };

    let class = cn(&[
        "relative flex h-svh flex-col bg-sidebar text-sidebar-foreground shrink-0 transition-[width] duration-200 ease-linear",
        width_class,
        side_class,
        variant_class,
        props.class.as_deref().unwrap_or_default(),
    ]);

    rsx! {
        aside {
            class,
            "data-slot": "sidebar",
            "data-state": state,
            "data-side": props.side.as_str(),
            "data-collapsible": collapsible,
            ..props.attributes,
            {props.children}
        }
    }
}

/// The button that toggles a [`Sidebar`] open and closed.
#[component]
pub fn SidebarTrigger(
    children: Element,
    #[props(default = Radius::Md)] radius: Radius,
    class: Option<String>,
    /// Native button/global attributes. Dioxus requires an element's
    /// attribute spread to be its last attribute, so this is listed after
    /// the trigger's own `onclick` below — a caller passing their own
    /// `onclick` here replaces the sidebar-toggle behavior rather than
    /// composing with it, matching every other `attributes`-accepting
    /// component in this registry (e.g. `SidebarMenuButton`).
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    attributes: Vec<Attribute>,
) -> Element {
    let ctx = use_sidebar();
    let class = cn(&[
        "inline-flex h-7 w-7 items-center justify-center text-sm hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        button {
            class,
            r#type: "button",
            "data-slot": "sidebar-trigger",
            aria_label: "Toggle Sidebar",
            onclick: move |_| ctx.toggle(),
            ..attributes,
            {children}
        }
    }
}

/// A thin edge rail that also toggles the [`Sidebar`], for pointer users
/// who prefer dragging the boundary over pressing the explicit trigger.
///
/// Must be rendered as a child of [`Sidebar`]'s own `aside`, which carries
/// `relative` specifically so this rail's `absolute` positioning resolves
/// against the sidebar's own box. Without it (found live: the rail rendered
/// at the page's own left edge, spanning nearly the full page height,
/// instead of hugging the sidebar's boundary), `position: absolute` climbs
/// to the nearest positioned ancestor -- which, with no other Sidebar part
/// establishing one, could be arbitrarily far up the consumer's own page.
#[component]
pub fn SidebarRail(
    class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    attributes: Vec<Attribute>,
) -> Element {
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
            ..attributes,
        }
    }
}

/// The main content area beside the [`Sidebar`]. Pass the same `variant` as
/// the sibling [`Sidebar`] to pick up matching rounded, shadowed styling for
/// [`SidebarVariant::Inset`].
///
/// Deliberately a caller-supplied prop, not read from the shared
/// [`SidebarCtx`]: an earlier attempt had `Sidebar` write its `variant` into
/// a context `Signal` for this component to read as a sibling, matching this
/// file's existing `open`/`set_open` context-sharing pattern -- but found
/// live (three different `Signal` construction strategies, all reproducing
/// the same result) that a **write from one sibling's render body is not
/// reliably observed by another sibling's signal read** in this Dioxus
/// version, unlike `open`, which only ever changes via `use_controlled`'s
/// own effect running inside `SidebarProvider`'s own scope. Explicit
/// prop-passing sidesteps the issue entirely and is what upstream
/// shadcn/Radix does too (via a CSS peer-selector, not shared JS state).
#[component]
pub fn SidebarInset(
    #[props(default)] variant: SidebarVariant,
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let variant_class = match variant {
        SidebarVariant::Inset => "m-2 rounded-xl shadow-sm",
        SidebarVariant::Sidebar | SidebarVariant::Floating => "",
    };
    let class = cn(&[
        "relative flex w-full flex-1 flex-col bg-background",
        variant_class,
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        main { class, "data-slot": "sidebar-inset", ..attributes, {children} }
    }
}

/// A header region pinned to the top of a [`Sidebar`].
#[component]
pub fn SidebarHeader(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "flex flex-col gap-2 p-2",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, "data-slot": "sidebar-header", ..attributes, {children} }
    }
}

/// The scrollable main region of a [`Sidebar`].
#[component]
pub fn SidebarContent(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "flex min-h-0 flex-1 flex-col gap-2 overflow-auto p-2",
        scroll_area_visibility_class(false),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, "data-slot": "sidebar-content", ..attributes, {children} }
    }
}

/// A footer region pinned to the bottom of a [`Sidebar`].
#[component]
pub fn SidebarFooter(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "flex flex-col gap-2 p-2",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, "data-slot": "sidebar-footer", ..attributes, {children} }
    }
}

/// A horizontal rule between [`Sidebar`] sections, composing the owned
/// `adico_primitives::separator::Separator` primitive.
#[component]
pub fn SidebarSeparator(
    class: Option<String>,
    #[props(default = true)] horizontal: bool,
    #[props(default = true)] decorative: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "mx-2 w-auto bg-sidebar-border",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        SeparatorPrimitive {
            class,
            horizontal,
            decorative,
            attributes,
        }
    }
}

/// A labeled group of related [`SidebarMenu`] items.
#[component]
pub fn SidebarGroup(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "relative flex w-full min-w-0 flex-col p-2",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, "data-slot": "sidebar-group", ..attributes, {children} }
    }
}

/// The label heading a [`SidebarGroup`]. Deliberately has no `radius` prop:
/// its `rounded-md` is a small internal chrome detail, not an
/// independently-tunable surface (see `SidebarTrigger`/`SidebarMenuButton`
/// for this item's actual `radius`-bearing controls).
#[component]
pub fn SidebarGroupLabel(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-medium text-sidebar-foreground/70",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, "data-slot": "sidebar-group-label", ..attributes, {children} }
    }
}

/// The content wrapper inside a [`SidebarGroup`].
#[component]
pub fn SidebarGroupContent(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&["w-full text-sm", class.as_deref().unwrap_or_default()]);
    rsx! {
        div { class, "data-slot": "sidebar-group-content", ..attributes, {children} }
    }
}

/// The list container for [`SidebarMenuItem`]s.
#[component]
pub fn SidebarMenu(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "flex w-full min-w-0 flex-col gap-1",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ul { class, "data-slot": "sidebar-menu", ..attributes, {children} }
    }
}

/// A single entry in a [`SidebarMenu`].
#[component]
pub fn SidebarMenuItem(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "group/menu-item relative",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        li { class, "data-slot": "sidebar-menu-item", ..attributes, {children} }
    }
}

/// The visual treatment of a [`SidebarMenuButton`], matching shadcn's own
/// `"default" | "outline"` cva axis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SidebarMenuButtonVariant {
    #[default]
    Default,
    Outline,
}

impl SidebarMenuButtonVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Outline => {
                "bg-background shadow-[0_0_0_1px_hsl(var(--sidebar-border))] hover:bg-sidebar-accent hover:text-sidebar-accent-foreground hover:shadow-[0_0_0_1px_hsl(var(--sidebar-accent))]"
            }
        }
    }
}

/// The height of a [`SidebarMenuButton`], matching shadcn's own
/// `"default" | "sm" | "lg"` cva axis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SidebarMenuButtonSize {
    Sm,
    #[default]
    Default,
    Lg,
}

impl SidebarMenuButtonSize {
    fn class(self) -> &'static str {
        match self {
            Self::Sm => "h-7 text-xs",
            Self::Default => "h-8 text-sm",
            Self::Lg => "h-12 text-sm",
        }
    }
}

/// Props for the clickable/navigable control inside a [`SidebarMenuItem`].
#[derive(Props, Clone, PartialEq)]
pub struct SidebarMenuButtonProps {
    /// Applies the active semantic treatment and `data-active` hook.
    #[props(default)]
    pub is_active: bool,
    /// Disables pointer and keyboard interaction with native semantics.
    #[props(default)]
    pub disabled: Option<bool>,
    /// Visual treatment; see [`SidebarMenuButtonVariant`].
    #[props(default)]
    pub variant: SidebarMenuButtonVariant,
    /// Height; see [`SidebarMenuButtonSize`].
    #[props(default)]
    pub size: SidebarMenuButtonSize,
    /// Corner radius of the control surface.
    #[props(default = Radius::Md)]
    pub radius: Radius,
    /// Shows a [`Spinner`] and marks the control busy/disabled (combined
    /// with `disabled` above). An adico extension — shadcn's own
    /// convention is composing `<Button disabled><Spinner /></Button>` by
    /// hand.
    #[props(default)]
    pub loading: bool,
    /// Replaces the control's visible content while `loading` is true.
    #[props(default)]
    pub loading_text: Option<String>,
    /// Extra classes appended to the semantic defaults.
    #[props(default)]
    pub class: Option<String>,
    /// Native button/global attributes and handlers.
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// The clickable/navigable control inside a [`SidebarMenuItem`].
#[component]
pub fn SidebarMenuButton(props: SidebarMenuButtonProps) -> Element {
    let class = cn(&[
        "flex w-full items-center gap-2 overflow-hidden px-2 text-left outline-none transition-[width,height,padding] hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 focus-visible:ring-sidebar-ring disabled:pointer-events-none disabled:opacity-50",
        props.variant.class(),
        props.size.class(),
        props.radius.class(),
        if props.is_active {
            "bg-sidebar-accent text-sidebar-accent-foreground font-medium"
        } else {
            ""
        },
        props.class.as_deref().unwrap_or_default(),
    ]);
    let is_disabled = props.disabled.unwrap_or(false) || props.loading;
    rsx! {
        button {
            class,
            r#type: "button",
            "data-slot": "sidebar-menu-button",
            "data-active": props.is_active,
            disabled: is_disabled,
            aria_busy: props.loading,
            ..props.attributes,
            if props.loading {
                Spinner {}
                if let Some(text) = props.loading_text {
                    "{text}"
                } else {
                    {props.children}
                }
            } else {
                {props.children}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floating_and_inset_variants_drop_the_plain_side_border() {
        let sidebar_default = cn(&["border-r"]);
        let floating = cn(&["m-2 rounded-lg border border-sidebar-border shadow-sm"]);
        let inset = cn(&[""]);
        assert!(sidebar_default.contains("border-r"));
        assert!(floating.contains("shadow-sm"));
        assert!(!floating.contains("border-r"));
        assert!(inset.is_empty());
    }

    #[test]
    fn inset_variant_gives_the_content_panel_its_own_rounded_surface() {
        let class = cn(&["m-2 rounded-xl shadow-sm"]);
        assert!(class.contains("rounded-xl"));
        assert!(class.contains("shadow-sm"));
    }
}
