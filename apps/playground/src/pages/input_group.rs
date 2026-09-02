use dioxus::prelude::*;

use crate::components;
use crate::components::controls::SelectControl;
use crate::components::demo::Demo;

#[component]
pub fn InputGroupPage() -> Element {
    let mut addon_align = use_signal(|| components::ui::InputGroupAlign::InlineEnd);
    rsx! {
        Demo { name: "InputGroup",
            controls: rsx! {
                SelectControl {
                    label: "Addon align",
                    value: addon_align(),
                    options: crate::generated::controls::INPUT_GROUP_ALIGN_OPTIONS.to_vec(),
                    on_change: move |value| addon_align.set(value),
                }
            },
            div { class: "flex max-w-sm flex-col gap-4",
                components::ui::InputGroup {
                    components::ui::InputGroupInput { placeholder: "Label text" }
                    components::ui::InputGroupAddon { align: addon_align(),
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
