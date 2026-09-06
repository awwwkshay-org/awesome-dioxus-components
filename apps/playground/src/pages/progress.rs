use dioxus::prelude::*;

use crate::components;
use crate::components::controls::NumberControl;
use crate::components::demo::Demo;

#[component]
pub fn ProgressPage() -> Element {
    let value = use_signal(|| 50.0);
    rsx! {
        Demo {
            name: "Progress",
            controls: rsx! {
                NumberControl {
                    label: "Value",
                    value,
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                }
            },
            components::ui::Progress { value: value().clamp(0.0, 100.0), class: "w-full max-w-sm" }
        }
    }
}
