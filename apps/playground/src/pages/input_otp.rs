use adico_primitives::icons::{Eye, EyeOff};
use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::components::ui::button::{ButtonSize, ButtonVariant};
use crate::generated::controls::{InputOTPControls, InputOTPDemoState};

#[component]
pub fn InputOTPPage() -> Element {
    let state = use_signal(InputOTPDemoState::default);
    let mut value = use_signal(String::new);
    let mut masked = use_signal(|| false);
    // `length: ReadSignal<usize>` is generator-skipped (Signal-typed), so the
    // slot count is a hand-rolled page control — the `sheet.rs` precedent
    // for generator-skipped shapes.
    let length = use_signal(|| 6usize);
    let half = length() / 2;
    rsx! {
        Demo {
            name: "InputOTP",
            controls: rsx! {
                InputOTPControls { state }
                components::controls::SelectControl::<usize> {
                    label: "Length",
                    value: length,
                    options: &[("4", 4usize), ("6", 6usize), ("8", 8usize)],
                }
                p { class: "self-end pb-2 text-sm text-muted-foreground", "Value: \"{value}\"" }
            },
            div { class: "flex items-center gap-2",
                components::ui::InputOTP {
                    // Keyed by `default_value` and `length` so changing either
                    // control remounts the field with fresh uncontrolled state --
                    // `default_value` only seeds at creation, matching
                    // `otp_field.rs`'s `use_controlled` semantics.
                    key: "{state().default_value}-{length()}",
                    length: length(),
                    value: None,
                    default_value: state().default_value,
                    mask: masked(),
                    on_value_change: move |v| value.set(v),
                    components::ui::InputOTPGroup {
                        for index in 0..half {
                            components::ui::InputOTPSlot { index }
                        }
                    }
                    components::ui::InputOTPSeparator {}
                    components::ui::InputOTPGroup {
                        for index in half..length() {
                            components::ui::InputOTPSlot { index }
                        }
                    }
                }
                components::ui::Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Icon,
                    aria_label: if masked() { "Show code" } else { "Hide code" },
                    onclick: move |_| masked.toggle(),
                    if masked() {
                        EyeOff { class: "size-4" }
                    } else {
                        Eye { class: "size-4" }
                    }
                }
            }
        }
    }
}
