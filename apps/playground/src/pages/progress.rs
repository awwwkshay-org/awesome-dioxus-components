use dioxus::prelude::*;

use crate::components;
use crate::components::controls::SelectControl;
use crate::components::demo::Demo;

#[component]
pub fn ProgressPage() -> Element {
    let value = use_signal(|| 50.0);
    rsx! {
        Demo {
            name: "Progress",
            controls: rsx! {
                SelectControl {
                    label: "Value",
                    value,
                    options: &[("0%", 0.0), ("25%", 25.0), ("50%", 50.0), ("75%", 75.0), ("100%", 100.0)],
                }
            },
            components::ui::Progress { value: value(), class: "w-full max-w-sm" }
        }
    }
}
