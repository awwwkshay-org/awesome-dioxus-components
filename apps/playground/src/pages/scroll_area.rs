use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::ScrollAreaControls;

#[component]
pub fn ScrollAreaPage() -> Element {
    rsx! {
        Demo {
            name: "Scroll Area",
            controls: rsx! {
                ScrollAreaControls {}
            },
            components::ui::ScrollArea {
                style: "height: 8em; width: 14em; border: 1px solid var(--border);",
                div {
                    for i in 1..=20 {
                        p { "Scrollable item {i}" }
                    }
                }
            }
        }
    }
}
