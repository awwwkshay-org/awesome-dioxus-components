use adico_primitives::icons::{Eye, EyeOff};
use dioxus::prelude::*;

use crate::components;
use crate::components::controls::TextControl;
use crate::components::demo::Demo;
use crate::generated::controls::{InputControls, InputDemoState};

#[component]
pub fn InputPage() -> Element {
    let state = use_signal(InputDemoState::default);
    let placeholder = use_signal(|| "Type here".to_string());
    let mut show_password = use_signal(|| false);
    rsx! {
        Demo {
            name: "Input",
            controls: rsx! {
                TextControl { label: "Placeholder", value: placeholder }
                InputControls { state }
            },
            div { class: "flex w-full max-w-sm flex-col gap-4",
                components::ui::Input {
                    placeholder: placeholder(),
                    r#type: state().r#type,
                    disabled: state().disabled,
                    readonly: state().readonly,
                    required: state().required,
                    invalid: state().invalid,
                }
                // Password show/hide is a composition, not an Input prop:
                // an InputGroup pairing a password-typed control with an
                // eye-toggle button — the same pattern a consumer copies.
                components::ui::InputGroup {
                    components::ui::InputGroupInput {
                        r#type: if show_password() { "text" } else { "password" },
                        placeholder: "Password",
                    }
                    components::ui::InputGroupAddon {
                        align: components::ui::InputGroupAlign::InlineEnd,
                        components::ui::InputGroupButton {
                            onclick: move |_| show_password.toggle(),
                            if show_password() {
                                EyeOff { class: "size-4" }
                            } else {
                                Eye { class: "size-4" }
                            }
                            span { class: "sr-only",
                                if show_password() { "Hide password" } else { "Show password" }
                            }
                        }
                    }
                }
            }
        }
    }
}
