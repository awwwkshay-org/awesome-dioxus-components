use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, SelectControl};
use crate::components::demo::Demo;

#[component]
pub fn ContextMenuPage() -> Element {
    let disabled = use_signal(|| false);
    let mut open = use_signal(|| None::<bool>);
    rsx! {
        Demo {
            name: "ContextMenu",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                SelectControl {
                    label: "Open state", value: open(),
                    options: vec![("Uncontrolled", None), ("Closed", Some(false)), ("Open", Some(true))],
                    on_change: move |value| open.set(value),
                }
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
