use adico_primitives::ContentAlign;
use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, ControlGroup, OptionalBoolControl, SelectControl};
use crate::components::demo::Demo;
use crate::generated::controls::{TooltipContentControls, TooltipControls, TooltipTriggerControls};

#[component]
pub fn TooltipPage() -> Element {
    let open = use_signal(|| None::<bool>);
    let disabled = use_signal(|| false);
    let align = use_signal(|| ContentAlign::Center);
    rsx! {
        Demo {
            name: "Tooltip",
            controls: rsx! {
                ControlGroup { part: "Tooltip",
                    BoolControl { label: "Disabled", value: disabled }
                    OptionalBoolControl { label: "Open state", value: open }
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
                TooltipControls {}
                TooltipTriggerControls {}
                TooltipContentControls {}
            },
            components::ui::Tooltip { open: open, disabled: disabled(),
                components::ui::TooltipTrigger { "Hover me" }
                components::ui::TooltipContent { align: align(), "Tooltip content" }
            }
        }
    }
}
