use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, OptionalBoolControl};
use crate::components::demo::Demo;

#[component]
pub fn ContextMenuPage() -> Element {
    let disabled = use_signal(|| false);
    let open = use_signal(|| None::<bool>);
    rsx! {
        Demo {
            name: "ContextMenu",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                OptionalBoolControl { label: "Open state", value: open }
            },
            components::ui::ContextMenu { disabled: disabled(), open: open,
                components::ui::ContextMenuTrigger { "Right click here" }
                components::ui::ContextMenuContent {
                    components::ui::ContextMenuItem {
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
