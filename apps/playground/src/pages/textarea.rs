use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, TextControl};
use crate::components::demo::Demo;

#[component]
pub fn TextareaPage() -> Element {
    let placeholder = use_signal(|| "Longer text".to_string());
    let disabled = use_signal(|| false);
    let readonly = use_signal(|| false);
    let required = use_signal(|| false);
    let invalid = use_signal(|| false);
    rsx! {
        Demo {
            name: "Textarea",
            controls: rsx! {
                TextControl { label: "Placeholder", value: placeholder }
                BoolControl { label: "Disabled", value: disabled }
                BoolControl { label: "Read only", value: readonly }
                BoolControl { label: "Required", value: required }
                BoolControl { label: "Invalid", value: invalid }
            },
            components::ui::Textarea {
                placeholder: placeholder(),
                disabled: disabled(),
                readonly: readonly(),
                required: required(),
                invalid: invalid(),
            }
        }
    }
}
