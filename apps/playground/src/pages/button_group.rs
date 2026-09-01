use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn ButtonGroupPage() -> Element {
    rsx! {
        Demo { name: "ButtonGroup",
            div { class: "flex flex-col gap-4",
                components::ui::ButtonGroup {
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Left" }
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Middle" }
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Right" }
                }
                components::ui::ButtonGroup {
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Copy" }
                    components::ui::ButtonGroupSeparator {}
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Paste" }
                }
                components::ui::ButtonGroup {
                    components::ui::ButtonGroupText { "https://" }
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "example.com" }
                }
                components::ui::ButtonGroup {
                    orientation: components::ui::ButtonGroupOrientation::Vertical,
                    class: "w-fit",
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Top" }
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Bottom" }
                }
            }
        }
    }
}
