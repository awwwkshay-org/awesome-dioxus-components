use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, OptionalBoolControl};
use crate::components::demo::Demo;

#[component]
pub fn TooltipPage() -> Element {
    let open = use_signal(|| None::<bool>);
    let disabled = use_signal(|| false);
    rsx! {
        Demo {
            name: "Tooltip",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                OptionalBoolControl { label: "Open state", value: open }
            },
            components::ui::Tooltip { open: open, disabled: disabled(),
                components::ui::TooltipTrigger { "Hover me" }
                components::ui::TooltipContent { "Tooltip content" }
            }
        }
    }
}
