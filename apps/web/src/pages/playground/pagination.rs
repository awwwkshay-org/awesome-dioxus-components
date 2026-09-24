use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{SelectControl, TextControl};
use crate::components::demo::Demo;
use crate::generated::controls::{
    PaginationNextControls, PaginationNextDemoState, PaginationPreviousControls,
    PaginationPreviousDemoState,
};

/// The page renders only a realistic pagination row. The generated
/// `PaginationLinkControls` panel is deliberately omitted: the page links
/// derive `is_active` from `active_page`, so that panel has no realistic
/// binding target — and the spec forbids inventing a demo element (the old
/// "Standalone link: 4" chip) just to host a panel's bindings.
#[component]
pub fn PaginationPage() -> Element {
    let mut active_page = use_signal(|| 2usize);
    let previous_text = use_signal(|| "Previous".to_string());
    let next_text = use_signal(|| "Next".to_string());
    let previous_state = use_signal(PaginationPreviousDemoState::default);
    let next_state = use_signal(PaginationNextDemoState::default);
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
                span { class: "col-span-full text-xs font-semibold text-muted-foreground",
                    "Previous button"
                }
                PaginationPreviousControls { state: previous_state }
                span { class: "col-span-full text-xs font-semibold text-muted-foreground",
                    "Next button"
                }
                PaginationNextControls { state: next_state }
            },
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
        }
    }
}
