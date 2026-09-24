//! The docs navigation list: guides, then components.
//!
//! One shared rendering, so a guide added to `GUIDES` or a component added to
//! the registry appears here with no second edit — the same contract
//! `nav.rs` holds for the playground's nav.
//!
//! App-level wiring under `apps/web/src/components/`: composed from the
//! installed `Sidebar` family, requiring no change to `registry/ui/*.rs`.

use dioxus::prelude::*;

use crate::components::ui::sidebar::{
    SidebarGroup, SidebarGroupLabel, SidebarMenu, SidebarMenuButton, SidebarMenuItem,
};
use crate::pages::docs::data::ui_components;
use crate::pages::docs::guides::GUIDES;

/// `current_path` drives the active highlight; `onnavigate` lets a caller
/// close a mobile overlay on selection.
#[component]
pub fn DocsNavList(
    current_path: String,
    #[props(default)] onnavigate: Option<EventHandler<()>>,
) -> Element {
    let navigator = use_navigator();
    let go = move |href: String| {
        // Programmatic navigation rather than nesting a router `Link` inside
        // `SidebarMenuButton`: the button renders a native control, and
        // `adico-web-structure` forbids adding an `as_child` escape hatch to a
        // registry component purely for an app's convenience.
        navigator.push(href);
        if let Some(handler) = onnavigate {
            handler.call(());
        }
    };

    let mut components = ui_components().collect::<Vec<_>>();
    components.sort_by(|a, b| a.name.cmp(&b.name));

    rsx! {
        nav { class: "flex flex-col gap-4", aria_label: "Documentation",
            SidebarGroup {
                SidebarGroupLabel { "Guides" }
                SidebarMenu {
                    for (href , label , _) in GUIDES {
                        SidebarMenuItem { key: "{href}",
                            div { onclick: move |_| go(href.to_string()),
                                SidebarMenuButton { is_active: current_path == *href,
                                    // `SidebarMenuButton` does not truncate a
                                    // bare text child on its own -- same
                                    // reason `nav.rs` wraps its label.
                                    span { class: "min-w-0 flex-1 truncate", "{label}" }
                                }
                            }
                        }
                    }
                }
            }
            SidebarGroup {
                SidebarGroupLabel { "Components" }
                SidebarMenu {
                    for component in components {
                        SidebarMenuItem { key: "{component.name}",
                            div {
                                onclick: {
                                    let name = component.name.clone();
                                    move |_| go(format!("/docs/components/{name}"))
                                },
                                SidebarMenuButton {
                                    is_active: current_path
                                        == format!("/docs/components/{}", component.name),
                                    span { class: "min-w-0 flex-1 truncate", "{component.name}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
