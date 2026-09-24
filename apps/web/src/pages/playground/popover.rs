use adico_primitives::ContentAlign;
use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, ControlGroup, SelectControl};
use crate::components::demo::Demo;
use crate::generated::controls::{PopoverContentControls, PopoverControls, PopoverTriggerControls};

#[component]
pub fn PopoverPage() -> Element {
    let mut open = use_signal(|| false);
    let align = use_signal(|| ContentAlign::Center);
    rsx! {
        Demo {
            name: "Popover",
            controls: rsx! {
                ControlGroup { part: "Popover",
                    BoolControl { label: "Open", value: open }
                    SelectControl {
                        label: "Align",
                        value: align,
                        options: &[
                            ("Start", ContentAlign::Start),
                            ("Center", ContentAlign::Center),
                            ("End", ContentAlign::End),
                        ],
                    }
                }
                PopoverControls {}
                PopoverTriggerControls {}
                PopoverContentControls {}
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
