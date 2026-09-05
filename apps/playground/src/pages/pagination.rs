use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{SelectControl, TextControl};
use crate::components::demo::Demo;
use crate::generated::controls::{
    PaginationLinkControls, PaginationLinkDemoState, PaginationNextControls,
    PaginationNextDemoState, PaginationPreviousControls, PaginationPreviousDemoState,
};

#[component]
pub fn PaginationPage() -> Element {
    let mut active_page = use_signal(|| 2usize);
    let previous_text = use_signal(|| "Previous".to_string());
    let next_text = use_signal(|| "Next".to_string());
    let previous_state = use_signal(PaginationPreviousDemoState::default);
    let next_state = use_signal(PaginationNextDemoState::default);
    // The three page-number links below derive their own `is_active` from
    // `active_page`, so binding the shared `PaginationLinkDemoState` there
    // would leave one of its two fields inert. This standalone link
    // demonstrates the generated panel in full instead.
    let link_state = use_signal(PaginationLinkDemoState::default);
    rsx! {
        Demo {
            name: "Pagination",
            controls: rsx! {
                SelectControl {
                    label: "Active page",
                    value: active_page,
                    options: &[("Page 1", 1usize), ("Page 2", 2usize), ("Page 3", 3usize)],
                }
                TextControl { label: "Previous text", value: previous_text }
                TextControl { label: "Next text", value: next_text }
                PaginationPreviousControls { state: previous_state }
                PaginationNextControls { state: next_state }
                PaginationLinkControls { state: link_state }
            },
            div { class: "flex flex-col gap-4",
                components::ui::Pagination {
                    components::ui::PaginationContent {
                        components::ui::PaginationItem {
                            components::ui::PaginationPrevious {
                                text: previous_text(),
                                compact: previous_state().compact,
                                onclick: move |_| active_page.set(active_page().saturating_sub(1).max(1)),
                            }
                        }
                        components::ui::PaginationItem {
                            components::ui::PaginationLink {
                                is_active: active_page() == 1,
                                onclick: move |_| active_page.set(1),
                                "1"
                            }
                        }
                        components::ui::PaginationItem {
                            components::ui::PaginationLink {
                                is_active: active_page() == 2,
                                onclick: move |_| active_page.set(2),
                                "2"
                            }
                        }
                        components::ui::PaginationItem {
                            components::ui::PaginationLink {
                                is_active: active_page() == 3,
                                onclick: move |_| active_page.set(3),
                                "3"
                            }
                        }
                        components::ui::PaginationItem { components::ui::PaginationEllipsis {} }
                        components::ui::PaginationItem {
                            components::ui::PaginationNext {
                                text: next_text(),
                                compact: next_state().compact,
                                onclick: move |_| active_page.set((active_page() + 1).min(3)),
                            }
                        }
                    }
                }
                div { class: "flex items-center gap-2",
                    span { class: "text-sm text-muted-foreground", "Standalone link:" }
                    components::ui::PaginationLink {
                        is_active: link_state().is_active,
                        loading: link_state().loading,
                        onclick: move |_| {},
                        "4"
                    }
                }
            }
        }
    }
}
