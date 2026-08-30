use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, SelectControl, TextControl};
use crate::components::demo::Demo;

#[component]
pub fn PaginationPage() -> Element {
    let mut active_page = use_signal(|| 2usize);
    let compact = use_signal(|| false);
    let previous_text = use_signal(|| "Previous".to_string());
    let next_text = use_signal(|| "Next".to_string());
    rsx! {
        Demo {
            name: "Pagination",
            controls: rsx! {
                SelectControl {
                    label: "Active page",
                    value: active_page(),
                    options: vec![("Page 1", 1usize), ("Page 2", 2usize), ("Page 3", 3usize)],
                    on_change: move |value| active_page.set(value),
                }
                BoolControl { label: "Compact previous / next", value: compact }
                TextControl { label: "Previous text", value: previous_text }
                TextControl { label: "Next text", value: next_text }
            },
            components::ui::Pagination {
                components::ui::PaginationContent {
                    components::ui::PaginationItem {
                        components::ui::PaginationPrevious {
                            text: previous_text(),
                            compact: compact(),
                            onclick: move |_| active_page.set(active_page().saturating_sub(1).max(1)),
                        }
                    }
                    components::ui::PaginationItem {
                        components::ui::PaginationLink { is_active: active_page() == 1, onclick: move |_| active_page.set(1), "1" }
                    }
                    components::ui::PaginationItem {
                        components::ui::PaginationLink { is_active: active_page() == 2, onclick: move |_| active_page.set(2), "2" }
                    }
                    components::ui::PaginationItem {
                        components::ui::PaginationLink { is_active: active_page() == 3, onclick: move |_| active_page.set(3), "3" }
                    }
                    components::ui::PaginationItem { components::ui::PaginationEllipsis {} }
                    components::ui::PaginationItem {
                        components::ui::PaginationNext {
                            text: next_text(),
                            compact: compact(),
                            onclick: move |_| active_page.set((active_page() + 1).min(3)),
                        }
                    }
                }
            }
        }
    }
}
