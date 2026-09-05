use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, OptionalBoolControl};
use crate::components::demo::Demo;

#[component]
pub fn DropdownMenuPage() -> Element {
    let disabled = use_signal(|| false);
    let open = use_signal(|| None::<bool>);
    rsx! {
        Demo {
            name: "DropdownMenu",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                OptionalBoolControl { label: "Open state", value: open }
            },
            components::ui::DropdownMenu { disabled: disabled(), open: open,
                components::ui::DropdownMenuTrigger { "Open menu" }
                components::ui::DropdownMenuContent {
                    components::ui::DropdownMenuItem::<String> {
                        value: "edit".to_string(),
                        index: 0usize,
                        on_select: move |_value| {},
                        "Edit"
                    }
                }
            }
        }
    }
}
