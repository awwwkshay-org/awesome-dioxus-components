use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{ButtonGroupControls, ButtonGroupDemoState};

#[component]
pub fn ButtonGroupPage() -> Element {
    let state = use_signal(ButtonGroupDemoState::default);
    rsx! {
        Demo { name: "ButtonGroup",
            controls: rsx! {
                ButtonGroupControls { state }
            },
            div { class: "flex flex-col gap-4",
                components::ui::ButtonGroup {
                    orientation: state().orientation,
                    class: "w-fit",
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Left" }
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Middle" }
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Right" }
                }
                components::ui::ButtonGroup {
                    orientation: state().orientation,
                    class: "w-fit",
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Copy" }
                    components::ui::ButtonGroupSeparator {
                        horizontal: state().orientation == components::ui::ButtonGroupOrientation::Horizontal,
                    }
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Paste" }
                }
                components::ui::ButtonGroup {
                    orientation: state().orientation,
                    class: "w-fit",
                    components::ui::ButtonGroupText { "https://" }
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "example.com" }
                }
            }
        }
    }
}
