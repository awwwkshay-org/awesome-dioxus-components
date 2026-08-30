use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn VirtualListPage() -> Element {
    rsx! {
        Demo {
            name: "VirtualList",
            components::ui::VirtualList {
                count: 1000usize,
                estimate_size: |_idx| 32,
                style: "height: 16em; width: 100%; overflow-y: auto; border: 1px solid var(--border);",
                render_item: move |idx: usize| rsx! {
                    div { key: "{idx}", class: "px-3 py-1 text-sm", "Row {idx}" }
                },
            }
        }
    }
}
