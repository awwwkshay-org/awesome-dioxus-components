use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{ControlGroup, NumberControl};
use crate::components::demo::Demo;
use crate::generated::controls::ProgressControls;

#[component]
pub fn ProgressPage() -> Element {
    let value = use_signal(|| 50.0);
    rsx! {
        Demo {
            name: "Progress",
            controls: rsx! {
                ControlGroup { part: "Progress",
                    NumberControl {
                        label: "Value",
                        value,
                        min: 0.0,
                        max: 100.0,
                        step: 1.0,
                    }
                }
                ProgressControls {}
            },
            components::ui::Progress { value: value().clamp(0.0, 100.0), class: "w-full max-w-sm" }
        }
    }
}
