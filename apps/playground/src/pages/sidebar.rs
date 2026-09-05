use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, OptionalBoolControl, SelectControl};
use crate::components::demo::Demo;

#[component]
pub fn SidebarPage() -> Element {
    let collapsible = use_signal(|| components::ui::SidebarCollapsible::Offcanvas);
    let side = use_signal(|| components::ui::SidebarSide::Left);
    let variant = use_signal(|| components::ui::SidebarVariant::Sidebar);
    // Defaults to `None` (uncontrolled), not `Some(true)`: `SidebarProvider`'s
    // own `default_open: true` already renders it open initially, and
    // matches this page's own trigger (`SidebarTrigger`) with nothing wired
    // to write clicks back into this signal -- the same tri-state,
    // no-sync-back pattern `select.rs`'s "Open state" control already uses
    // for the same reason. Defaulting to `Some(true)` instead made the
    // trigger silently non-interactive out of the box: `SidebarProvider`
    // would have stayed force-open regardless of clicks (found live from a
    // user screenshot -- the trigger button visibly did nothing).
    let open = use_signal(|| None::<bool>);
    let active_settings = use_signal(|| true);
    let settings_disabled = use_signal(|| false);
    rsx! {
        Demo {
            name: "Sidebar",
            controls: rsx! {
                SelectControl {
                    label: "Collapsible",
                    value: collapsible,
                    options: crate::generated::controls::SIDEBAR_COLLAPSIBLE_OPTIONS,
                }
                SelectControl {
                    label: "Side",
                    value: side,
                    options: crate::generated::controls::SIDEBAR_SIDE_OPTIONS,
                }
                SelectControl {
                    label: "Variant",
                    value: variant,
                    options: crate::generated::controls::SIDEBAR_VARIANT_OPTIONS,
                }
                OptionalBoolControl { label: "Open state", value: open }
                BoolControl { label: "Settings active", value: active_settings }
                BoolControl { label: "Settings disabled", value: settings_disabled }
            },
            // The rounded/border/height framing lives on this outer div, not
            // on `SidebarProvider` itself: `SidebarProvider`'s own base
            // classes hardcode `min-h-svh` (correct for a real consumer,
            // where it's the top-level app shell and should always span at
            // least the full viewport), and `min-height` wins over `height`
            // whenever it's larger -- so a `h-64`/`min-h-0` passed as
            // `class` here can never actually shrink it below `100svh`,
            // regardless of class string order. This crate's own `cn` (see
            // `adico_lib::cn::cn`) is plain concatenation, not a
            // conflict-resolving merge like `tailwind-merge`, so relying on
            // class order to win that fight isn't reliable either (found
            // live from a user screenshot: the sidebar preview's border
            // appeared broken because the boxed rendered ~864px tall inside
            // a spot budgeted for 16rem, spilling past this section's
            // `overflow-auto` viewport). `overflow-hidden` on this wrapper
            // sidesteps the fight instead of trying to win it: the inner
            // `SidebarProvider` can stay its natural (oversized, in this
            // constrained preview) height and simply gets clipped to what's
            // actually visible here.
            div { class: "h-64 w-full overflow-hidden rounded-lg border",
                components::ui::SidebarProvider { class: "h-full",
                    open: open,
                    components::ui::Sidebar {
                        collapsible: collapsible(),
                        side: side(),
                        variant: variant(),
                        components::ui::SidebarHeader { "My App" }
                        components::ui::SidebarContent {
                            components::ui::SidebarGroup {
                                components::ui::SidebarGroupLabel { "Section" }
                                components::ui::SidebarGroupContent {
                                    components::ui::SidebarMenu {
                                        components::ui::SidebarMenuItem {
                                            components::ui::SidebarMenuButton { "Overview" }
                                        }
                                        components::ui::SidebarMenuItem {
                                            components::ui::SidebarMenuButton { is_active: active_settings(), disabled: settings_disabled(), "Settings" }
                                        }
                                    }
                                }
                            }
                            components::ui::SidebarSeparator {}
                        }
                        components::ui::SidebarFooter { "v1.0" }
                        components::ui::SidebarRail {}
                    }
                    components::ui::SidebarInset { variant: variant(),
                        components::ui::SidebarTrigger { "☰" }
                        " Main content"
                    }
                }
            }
        }
    }
}
