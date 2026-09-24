use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{NumberControl, TextControl};
use crate::components::demo::Demo;
use crate::generated::controls::{TextareaControls, TextareaDemoState};

#[component]
pub fn TextareaPage() -> Element {
    let state = use_signal(TextareaDemoState::default);
    let placeholder = use_signal(|| "Longer text".to_string());
    // `max_length: Option<u32>` is generator-skipped, so it is a hand-rolled
    // page control (the `sheet.rs` precedent): 0 means "no max" — no
    // maxlength attribute and no counter.
    let max_length = use_signal(|| 280.0_f64);
    let max = (max_length() > 0.0).then_some(max_length() as u32);
    rsx! {
        Demo {
            name: "Textarea",
            controls: rsx! {
                TextControl { label: "Placeholder", value: placeholder }
                NumberControl {
                    label: "Max length (0 = none)",
                    value: max_length,
                    min: 0.0,
                    step: 10.0,
                }
                TextareaControls { state }
            },
            components::ui::Textarea {
                // Keyed so switching between counter and no-counter mode
                // remounts cleanly with a fresh internal typed count.
                key: "{max:?}",
                placeholder: placeholder(),
                max_length: max,
                disabled: state().disabled,
                readonly: state().readonly,
                required: state().required,
                invalid: state().invalid,
            }
        }
    }
}
