use dioxus::prelude::*;

use crate::components;
use crate::components::controls::BoolControl;
use crate::components::demo::Demo;

#[component]
pub fn MenubarPage() -> Element {
    let disabled = use_signal(|| false);
    rsx! {
        Demo {
            name: "Menubar",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
            },
            components::ui::Menubar { disabled: disabled(),
                components::ui::MenubarMenu { index: 0usize,
                    components::ui::MenubarTrigger { "File" }
                    components::ui::MenubarContent {
                        components::ui::MenubarItem {
                            index: 0usize,
                            value: "new".to_string(),
                            on_select: move |_value| {},
                            "New"
                        }
                    }
                }
            }
        }
    }
}
