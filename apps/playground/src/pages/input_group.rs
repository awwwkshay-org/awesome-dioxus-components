use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{
    InputGroupAddonControls, InputGroupAddonDemoState, InputGroupButtonControls,
    InputGroupButtonDemoState,
};

#[component]
pub fn InputGroupPage() -> Element {
    let addon_state = use_signal(|| InputGroupAddonDemoState {
        align: components::ui::InputGroupAlign::InlineEnd,
    });
    let button_state = use_signal(InputGroupButtonDemoState::default);
    rsx! {
        Demo { name: "Input Group",
            controls: rsx! {
                InputGroupAddonControls { state: addon_state }
                InputGroupButtonControls { state: button_state }
            },
            div { class: "flex max-w-sm flex-col gap-4",
                components::ui::InputGroup {
                    components::ui::InputGroupInput { placeholder: "Label text" }
                    components::ui::InputGroupAddon { align: addon_state().align,
                        components::ui::InputGroupText { "Label" }
                    }
                }
                components::ui::InputGroup {
                    components::ui::InputGroupAddon { components::ui::InputGroupText { "$" } }
                    components::ui::InputGroupInput { placeholder: "0.00" }
                    components::ui::InputGroupAddon {
                        align: components::ui::InputGroupAlign::InlineEnd,
                        components::ui::InputGroupText { "USD" }
                    }
                }
                components::ui::InputGroup {
                    components::ui::InputGroupInput { placeholder: "Search..." }
                    components::ui::InputGroupAddon {
                        align: components::ui::InputGroupAlign::InlineEnd,
                        components::ui::InputGroupButton { loading: button_state().loading, "Go" }
                    }
                }
                components::ui::InputGroup {
                    components::ui::InputGroupTextarea { placeholder: "Leave a comment", rows: 3 }
                    components::ui::InputGroupAddon {
                        align: components::ui::InputGroupAlign::BlockEnd,
                        components::ui::InputGroupButton { loading: button_state().loading, "Send" }
                    }
                }
            }
        }
    }
}
