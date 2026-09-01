use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, SelectControl};
use crate::components::demo::Demo;

#[component]
pub fn SidebarPage() -> Element {
    let mut collapsible = use_signal(|| components::ui::SidebarCollapsible::Offcanvas);
    let mut side = use_signal(|| components::ui::SidebarSide::Left);
    let mut variant = use_signal(|| components::ui::SidebarVariant::Sidebar);
    let mut open = use_signal(|| Some(true));
    let active_settings = use_signal(|| true);
    let settings_disabled = use_signal(|| false);
    rsx! {
        Demo {
            name: "Sidebar",
            controls: rsx! {
                SelectControl {
                    label: "Collapsible",
                    value: collapsible(),
                    options: vec![
                        ("Offcanvas", components::ui::SidebarCollapsible::Offcanvas),
                        ("Icon", components::ui::SidebarCollapsible::Icon),
                        ("None", components::ui::SidebarCollapsible::None),
                    ],
                    on_change: move |value| collapsible.set(value),
                }
                SelectControl {
                    label: "Side",
                    value: side(),
                    options: vec![
                        ("Left", components::ui::SidebarSide::Left),
                        ("Right", components::ui::SidebarSide::Right),
                    ],
                    on_change: move |value| side.set(value),
                }
                SelectControl {
                    label: "Variant",
                    value: variant(),
                    options: vec![
                        ("Sidebar", components::ui::SidebarVariant::Sidebar),
                        ("Floating", components::ui::SidebarVariant::Floating),
                        ("Inset", components::ui::SidebarVariant::Inset),
                    ],
                    on_change: move |value| variant.set(value),
                }
                SelectControl {
                    label: "Open state",
                    value: open(),
                    options: vec![("Uncontrolled", None), ("Open", Some(true)), ("Closed", Some(false))],
                    on_change: move |value| open.set(value),
                }
                BoolControl { label: "Settings active", value: active_settings }
                BoolControl { label: "Settings disabled", value: settings_disabled }
            },
            components::ui::SidebarProvider { class: "h-64 min-h-0 rounded-lg border",
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
