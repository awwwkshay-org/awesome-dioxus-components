use dioxus::prelude::*;

use crate::components;
use crate::components::controls::TextControl;
use crate::components::demo::Demo;
use crate::generated::controls::{InputControls, InputDemoState};

#[component]
pub fn InputPage() -> Element {
    let state = use_signal(InputDemoState::default);
    let placeholder = use_signal(|| "Type here".to_string());
    rsx! {
        Demo {
            name: "Input",
            controls: rsx! {
                TextControl { label: "Placeholder", value: placeholder }
                InputControls { state }
            },
            div { class: "w-full max-w-sm",
                components::ui::Input {
                    placeholder: placeholder(),
                    r#type: state().r#type,
                    disabled: state().disabled,
                    readonly: state().readonly,
                    required: state().required,
                    invalid: state().invalid,
                }
            }
        }
    }
}
