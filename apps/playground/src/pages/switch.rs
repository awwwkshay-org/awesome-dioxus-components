use dioxus::prelude::*;

use crate::components;
use crate::components::controls::SelectControl;
use crate::components::demo::Demo;

#[component]
pub fn SwitchPage() -> Element {
    let mut checked = use_signal(|| false);
    let mut size = use_signal(|| components::ui::SwitchSize::Default);
    rsx! {
        Demo {
            name: "Switch",
            controls: rsx! {
                SelectControl {
                    label: "Size",
                    value: size(),
                    options: crate::generated::controls::SWITCH_SIZE_OPTIONS.to_vec(),
                    on_change: move |value| size.set(value),
                }
            },
            components::ui::Switch {
                checked: checked(),
                on_checked_change: move |value| checked.set(value),
                size: size(),
                aria_label: "Enable notifications",
            }
        }
    }
}
