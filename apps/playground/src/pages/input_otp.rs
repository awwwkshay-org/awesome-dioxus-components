use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn InputOTPPage() -> Element {
    let mut value = use_signal(String::new);
    rsx! {
        Demo {
            name: "InputOTP",
            controls: rsx! {
                p { class: "self-end pb-2 text-sm text-muted-foreground", "Value: \"{value}\"" }
            },
            components::ui::InputOTP {
                length: 6usize,
                value: value(),
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
