use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, SelectControl, TextControl};
use crate::components::demo::Demo;
use crate::generated::controls::{ButtonControls, ButtonDemoState};

#[derive(Clone, Copy, PartialEq)]
enum ButtonContent {
    Text,
    Icon,
    IconAndText,
}

#[component]
pub fn ButtonPage() -> Element {
    let state = use_signal(ButtonDemoState::default);
    let disabled = use_signal(|| false);
    let button_type = use_signal(|| "button");
    let content = use_signal(|| ButtonContent::Text);
    let label = use_signal(|| "Save changes".to_string());
    rsx! {
        Demo {
            name: "Button",
            controls: rsx! {
                ButtonControls { state }
                BoolControl { label: "Disabled", value: disabled }
                SelectControl {
                    label: "Native type",
                    value: button_type,
                    options: &[("Button", "button"), ("Submit", "submit"), ("Reset", "reset")],
                }
                SelectControl {
                    label: "Children",
                    value: content,
                    options: &[
                        ("Text", ButtonContent::Text),
                        ("Icon only", ButtonContent::Icon),
                        ("Icon and text", ButtonContent::IconAndText),
                    ],
                }
                TextControl { label: "Text", value: label }
            },
            components::ui::Button {
                variant: state().variant,
                size: state().size,
                loading: state().loading,
                disabled: disabled(),
                r#type: button_type(),
                aria_label: (content() == ButtonContent::Icon).then_some("Save changes"),
                if content() != ButtonContent::Text {
                    span { "aria-hidden": "true", "↗" }
                }
                if content() != ButtonContent::Icon {
                    "{label}"
                }
            }
        }
    }
}
