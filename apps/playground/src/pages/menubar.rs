use dioxus::prelude::*;

use crate::components;
use crate::components::controls::BoolControl;
use crate::components::demo::Demo;
use crate::generated::controls::{MenubarItemControls, MenubarItemDemoState};

#[component]
pub fn MenubarPage() -> Element {
    let disabled = use_signal(|| false);
    let item_state = use_signal(|| MenubarItemDemoState {
        value: "new".to_string(),
        ..Default::default()
    });
    rsx! {
        Demo {
            name: "Menubar",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                MenubarItemControls { state: item_state }
            },
            components::ui::Menubar { disabled: disabled(),
                components::ui::MenubarMenu { index: 0usize,
                    components::ui::MenubarTrigger { "File" }
                    components::ui::MenubarContent {
                        components::ui::MenubarItem {
                            index: 0usize,
                            value: item_state().value,
                            inset: item_state().inset,
                            variant: item_state().variant,
                            on_select: move |_value| {},
                            "New"
                        }
                    }
                }
            }
        }
    }
}
