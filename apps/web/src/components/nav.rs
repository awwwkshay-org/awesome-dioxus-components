//! The playground's nav list (every entry from `routes::nav_items()`,
//! rendered as an installed `SidebarMenu`), shared between the `>= md`
//! resizable nav column and the `< md` `Sheet` overlay in `routes.rs`'s
//! `Layout` so a route added to `nav_items()` shows up in both without
//! separate wiring.

use dioxus::prelude::*;

use crate::components::ui;
use crate::routes::{Route, nav_items};

#[component]
pub fn NavList(current_route: Route, onnavigate: Callback<Route>) -> Element {
    rsx! {
        ui::SidebarMenu {
            for (label , route) in nav_items() {
                ui::SidebarMenuItem {
                    div {
                        onclick: move |_| onnavigate.call(route.clone()),
                        ui::SidebarMenuButton {
                            is_active: current_route == route,
                            // See `routes.rs`'s former inline loop (this
                            // component's origin) for why the label needs its
                            // own `min-w-0 truncate` span: `SidebarMenuButton`
                            // does not truncate a bare text child on its own.
                            span { class: "min-w-0 flex-1 truncate", "{label}" }
                        }
                    }
                }
            }
        }
    }
}
