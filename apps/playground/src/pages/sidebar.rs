use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{
    SidebarControls, SidebarDemoState, SidebarMenuButtonControls, SidebarMenuButtonDemoState,
    SidebarProviderControls, SidebarProviderDemoState,
};

#[component]
pub fn SidebarPage() -> Element {
    let sidebar_state = use_signal(SidebarDemoState::default);
    // Defaults to `open: None` (uncontrolled), not `Some(true)`:
    // `SidebarProvider`'s own `default_open: true` already renders it open
    // initially, and matches this page's own trigger (`SidebarTrigger`)
    // with nothing wired to write clicks back into this signal -- the same
    // tri-state, no-sync-back pattern `select.rs`'s "Open state" control
    // already uses for the same reason. Defaulting to `Some(true)` instead
    // made the trigger silently non-interactive out of the box:
    // `SidebarProvider` would have stayed force-open regardless of clicks
    // (found live from a user screenshot -- the trigger button visibly did
    // nothing).
    let provider_state = use_signal(|| SidebarProviderDemoState {
        open: None,
        default_open: true,
    });
    let menu_button_state = use_signal(|| SidebarMenuButtonDemoState {
        is_active: true,
        ..Default::default()
    });
    rsx! {
        Demo {
            name: "Sidebar",
            controls: rsx! {
                SidebarControls { state: sidebar_state }
                SidebarProviderControls { state: provider_state }
                SidebarMenuButtonControls { state: menu_button_state }
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
                components::ui::SidebarProvider {
                    class: "h-full",
                    open: provider_state().open,
                    default_open: provider_state().default_open,
                    components::ui::Sidebar {
                        collapsible: sidebar_state().collapsible,
                        side: sidebar_state().side,
                        variant: sidebar_state().variant,
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
                                            components::ui::SidebarMenuButton {
                                                is_active: menu_button_state().is_active,
                                                disabled: menu_button_state().disabled,
                                                variant: menu_button_state().variant,
                                                size: menu_button_state().size,
                                                loading: menu_button_state().loading,
                                                "Settings"
                                            }
                                        }
                                    }
                                }
                            }
                            components::ui::SidebarSeparator {}
                        }
                        components::ui::SidebarFooter { "v1.0" }
                        components::ui::SidebarRail {}
                    }
                    components::ui::SidebarInset { variant: sidebar_state().variant,
                        components::ui::SidebarTrigger { "☰" }
                        " Main content"
                    }
                }
            }
        }
    }
}
