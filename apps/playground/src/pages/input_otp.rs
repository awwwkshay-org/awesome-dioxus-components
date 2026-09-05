use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{InputOTPControls, InputOTPDemoState};

#[component]
pub fn InputOTPPage() -> Element {
    let state = use_signal(InputOTPDemoState::default);
    let mut value = use_signal(String::new);
    rsx! {
        Demo {
            name: "InputOTP",
            controls: rsx! {
                InputOTPControls { state }
                p { class: "self-end pb-2 text-sm text-muted-foreground", "Value: \"{value}\"" }
            },
            components::ui::InputOTP {
                // Keyed by `default_value` so changing the control remounts the
                // field with the new initial value -- `default_value` only seeds
                // uncontrolled internal state at creation, matching
                // `otp_field.rs`'s `use_controlled` semantics.
                key: "{state().default_value}",
                length: 6usize,
                value: None,
                default_value: state().default_value,
                on_value_change: move |v| value.set(v),
                components::ui::InputOTPGroup {
                    components::ui::InputOTPSlot { index: 0usize }
                    components::ui::InputOTPSlot { index: 1usize }
                    components::ui::InputOTPSlot { index: 2usize }
                }
                components::ui::InputOTPSeparator {}
                components::ui::InputOTPGroup {
                    components::ui::InputOTPSlot { index: 3usize }
                    components::ui::InputOTPSlot { index: 4usize }
                    components::ui::InputOTPSlot { index: 5usize }
                }
            }
        }
    }
}
