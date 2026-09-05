use dioxus::prelude::*;

use crate::components;
use crate::components::controls::TextControl;
use crate::components::demo::Demo;
use crate::generated::controls::{TextareaControls, TextareaDemoState};

#[component]
pub fn TextareaPage() -> Element {
    let state = use_signal(TextareaDemoState::default);
    let placeholder = use_signal(|| "Longer text".to_string());
    rsx! {
        Demo {
            name: "Textarea",
            controls: rsx! {
                TextControl { label: "Placeholder", value: placeholder }
                TextareaControls { state }
            },
            components::ui::Textarea {
                placeholder: placeholder(),
                disabled: state().disabled,
                readonly: state().readonly,
                required: state().required,
                invalid: state().invalid,
            }
        }
    }
}
