use adico_primitives::ContentAlign;
use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, SelectControl};
use crate::components::demo::Demo;

#[component]
pub fn PopoverPage() -> Element {
    let mut open = use_signal(|| false);
    let mut align = use_signal(|| ContentAlign::Center);
    rsx! {
        Demo {
            name: "Popover",
            controls: rsx! {
                BoolControl { label: "Open", value: open }
                SelectControl {
                    label: "Align",
                    value: align(),
                    options: vec![
                        ("Start", ContentAlign::Start),
                        ("Center", ContentAlign::Center),
                        ("End", ContentAlign::End),
                    ],
                    on_change: move |value| align.set(value),
                }
            },
            components::ui::Popover {
                open: open(),
                on_open_change: move |value| open.set(value),
                components::ui::PopoverTrigger { "Open popover" }
                components::ui::PopoverContent { align: align(), "Popover content" }
            }
        }
    }
}
