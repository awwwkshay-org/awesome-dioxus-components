use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn InputGroupPage() -> Element {
    rsx! {
        Demo { name: "InputGroup",
            div { class: "flex max-w-sm flex-col gap-4",
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
                        components::ui::InputGroupButton { "Go" }
                    }
                }
                components::ui::InputGroup {
                    components::ui::InputGroupTextarea { placeholder: "Leave a comment", rows: 3 }
                    components::ui::InputGroupAddon {
                        align: components::ui::InputGroupAlign::BlockEnd,
                        components::ui::InputGroupButton { "Send" }
                    }
                }
            }
        }
    }
}
