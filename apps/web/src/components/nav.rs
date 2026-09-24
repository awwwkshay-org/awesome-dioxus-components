//! The playground's nav list (every entry from `routes::nav_items()`,
//! rendered as an installed `SidebarMenu`), shared between the `>= md`
//! resizable nav column and the `< md` `Sheet` overlay in `routes.rs`'s
//! `PlaygroundLayout` so a route added to `nav_items()` shows up in both
//! without separate wiring.
//!
//! The filter lives here rather than in either shell so both presentations
//! get it from one rendering, for the same reason the list does. Its state is
//! deliberately per-mount: a filter that survived navigation would sit
//! invisible behind a closed sheet and read as missing entries the next time
//! the list opened.

use dioxus::prelude::*;

use crate::components::ui;
use crate::routes::{Route, nav_items};

#[component]
pub fn NavList(current_route: Route, onnavigate: Callback<Route>) -> Element {
    let mut filter = use_signal(String::new);
    let query = filter().trim().to_lowercase();

    let matches: Vec<(&'static str, Route)> = nav_items()
        .into_iter()
        // Filtering only ever removes entries, never reorders them, so the
        // flat A→Z guarantee holds for every filter value.
        .filter(|(label, _)| query.is_empty() || label.to_lowercase().contains(&query))
        .collect();

    rsx! {
        div { class: "flex min-h-0 flex-1 flex-col gap-2",
            ui::Input {
                r#type: "search".to_string(),
                value: filter(),
                placeholder: "Filter components".to_string(),
                oninput: move |event: FormEvent| filter.set(event.value()),
                aria_label: "Filter components",
                class: "h-8",
            }

            if matches.is_empty() {
                p { class: "px-2 py-6 text-center text-sm text-muted-foreground",
                    "No component matches “{filter()}”."
                }
            } else {
                ui::SidebarMenu {
                    for (label , route) in matches {
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
    }
}
