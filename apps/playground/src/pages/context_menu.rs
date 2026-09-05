use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, OptionalBoolControl};
use crate::components::demo::Demo;
use crate::generated::controls::{ContextMenuItemControls, ContextMenuItemDemoState};

#[component]
pub fn ContextMenuPage() -> Element {
    let disabled = use_signal(|| false);
    let open = use_signal(|| None::<bool>);
    let item_state = use_signal(ContextMenuItemDemoState::default);
    rsx! {
        Demo {
            name: "ContextMenu",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                OptionalBoolControl { label: "Open state", value: open }
                ContextMenuItemControls { state: item_state }
            },
            components::ui::ContextMenu { disabled: disabled(), open: open,
                components::ui::ContextMenuTrigger { "Right click here" }
                components::ui::ContextMenuContent {
                    components::ui::ContextMenuItem {
                        value: "edit".to_string(),
                        index: 0usize,
                        inset: item_state().inset,
                        variant: item_state().variant,
                        on_select: move |_value| {},
                        "Edit"
                    }
                }
            }
        }
    }
}
