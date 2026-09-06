use adico_primitives::icons::{Bell, ChartLine, House, Inbox, Settings, Users};
use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{
    SidebarControls, SidebarDemoState, SidebarMenuButtonControls, SidebarMenuButtonDemoState,
    SidebarProviderControls, SidebarProviderDemoState,
};

/// A dashboard-style sidebar: a "Platform" group (with icons), a separator,
/// a "Settings" group, and a footer user row composing the installed
/// `Avatar`. The `SidebarMenuButtonControls` panel binds to "Dashboard".
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
            wide: true,
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
            // whenever it's larger -- so a fixed height/`min-h-0` passed as
            // `class` here can never actually shrink it below `100svh`,
            // regardless of class string order. This crate's own `cn` (see
            // `adico_lib::cn::cn`) is plain concatenation, not a
            // conflict-resolving merge like `tailwind-merge`, so relying on
            // class order to win that fight isn't reliable either (found
            // live from a user screenshot: the sidebar preview's border
            // appeared broken because the box rendered ~864px tall inside
            // a spot budgeted for its height, spilling past this section's
            // viewport). `overflow-hidden` on this wrapper sidesteps the
            // fight instead of trying to win it: the inner
            // `SidebarProvider` can stay its natural (oversized, in this
            // constrained preview) height and simply gets clipped to what's
            // actually visible here.
            div { class: "h-[28rem] w-full overflow-hidden rounded-lg border",
                components::ui::SidebarProvider {
                    class: "h-full",
                    open: provider_state().open,
                    default_open: provider_state().default_open,
                    components::ui::Sidebar {
                        collapsible: sidebar_state().collapsible,
                        side: sidebar_state().side,
                        variant: sidebar_state().variant,
                        components::ui::SidebarHeader {
                            div { class: "px-2 py-1 text-sm font-semibold", "Acme Inc." }
                        }
                        components::ui::SidebarContent {
                            components::ui::SidebarGroup {
                                components::ui::SidebarGroupLabel { "Platform" }
                                components::ui::SidebarGroupContent {
                                    components::ui::SidebarMenu {
                                        components::ui::SidebarMenuItem {
                                            components::ui::SidebarMenuButton {
                                                is_active: menu_button_state().is_active,
                                                disabled: menu_button_state().disabled,
                                                variant: menu_button_state().variant,
                                                size: menu_button_state().size,
                                                loading: menu_button_state().loading,
                                                House { class: "size-4" }
                                                "Dashboard"
                                            }
                                        }
                                        components::ui::SidebarMenuItem {
                                            components::ui::SidebarMenuButton {
                                                ChartLine { class: "size-4" }
                                                "Analytics"
                                            }
                                        }
                                        components::ui::SidebarMenuItem {
                                            components::ui::SidebarMenuButton {
                                                Inbox { class: "size-4" }
                                                "Reports"
                                            }
                                        }
                                    }
                                }
                            }
                            components::ui::SidebarSeparator {}
                            components::ui::SidebarGroup {
                                components::ui::SidebarGroupLabel { "Settings" }
                                components::ui::SidebarGroupContent {
                                    components::ui::SidebarMenu {
                                        components::ui::SidebarMenuItem {
                                            components::ui::SidebarMenuButton {
                                                Users { class: "size-4" }
                                                "Members"
                                            }
                                        }
                                        components::ui::SidebarMenuItem {
                                            components::ui::SidebarMenuButton {
                                                Bell { class: "size-4" }
                                                "Notifications"
                                            }
                                        }
                                        components::ui::SidebarMenuItem {
                                            components::ui::SidebarMenuButton {
                                                Settings { class: "size-4" }
                                                "General"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        components::ui::SidebarFooter {
                            div { class: "flex items-center gap-2 px-2 py-1",
                                components::ui::Avatar { class: "size-7",
                                    components::ui::AvatarFallback { "AL" }
                                }
                                div { class: "flex min-w-0 flex-col text-left",
                                    span { class: "truncate text-sm font-medium", "Ada Lovelace" }
                                    span { class: "truncate text-xs text-muted-foreground", "ada@acme.dev" }
                                }
                            }
                        }
                        components::ui::SidebarRail {}
                    }
                    components::ui::SidebarInset { variant: sidebar_state().variant,
                        div { class: "flex items-center gap-2 border-b border-border p-3",
                            components::ui::SidebarTrigger { "☰" }
                        }
                        div { class: "min-h-0 flex-1 overflow-y-auto p-3 lg:p-6", "Main content" }
                    }
                }
            }
        }
    }
}
