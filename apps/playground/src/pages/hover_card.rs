use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, OptionalBoolControl};
use crate::components::demo::Demo;

#[component]
pub fn HoverCardPage() -> Element {
    let open = use_signal(|| None::<bool>);
    let disabled = use_signal(|| false);
    rsx! {
        Demo {
            name: "HoverCard",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                OptionalBoolControl { label: "Open state", value: open }
            },
            components::ui::HoverCard { open: open, disabled: disabled(),
                components::ui::HoverCardTrigger { "Dioxus" }
                components::ui::HoverCardContent { "Hover card content" }
            }
        }
    }
}
