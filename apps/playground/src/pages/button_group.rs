use dioxus::prelude::*;

use crate::components;
use crate::components::controls::SelectControl;
use crate::components::demo::Demo;

#[component]
pub fn ButtonGroupPage() -> Element {
    let mut orientation = use_signal(|| components::ui::ButtonGroupOrientation::Horizontal);
    rsx! {
        Demo { name: "ButtonGroup",
            controls: rsx! {
                SelectControl {
                    label: "Orientation",
                    value: orientation(),
                    options: crate::generated::controls::BUTTON_GROUP_ORIENTATION_OPTIONS.to_vec(),
                    on_change: move |value| orientation.set(value),
                }
            },
            div { class: "flex flex-col gap-4",
                components::ui::ButtonGroup {
                    orientation: orientation(),
                    class: "w-fit",
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Left" }
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Middle" }
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Right" }
                }
                components::ui::ButtonGroup {
                    orientation: orientation(),
                    class: "w-fit",
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Copy" }
                    components::ui::ButtonGroupSeparator {}
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Paste" }
                }
                components::ui::ButtonGroup {
                    orientation: orientation(),
                    class: "w-fit",
                    components::ui::ButtonGroupText { "https://" }
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "example.com" }
                }
            }
        }
    }
}
