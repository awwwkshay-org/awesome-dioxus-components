use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::LabelControls;

#[component]
pub fn LabelPage() -> Element {
    rsx! {
        Demo {
            name: "Label",
            controls: rsx! {
                LabelControls {}
            },
            div { class: "grid w-full max-w-sm gap-1.5",
                components::ui::Label { html_for: "playground-label-demo-name", "Name" }
                components::ui::Input { id: "playground-label-demo-name", placeholder: "Enter your name" }
            }
        }
    }
}
