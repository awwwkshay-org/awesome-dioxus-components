use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, SelectControl, TextControl};
use crate::components::demo::Demo;

#[derive(Clone, Copy, PartialEq)]
enum ButtonContent {
    Text,
    Icon,
    IconAndText,
}

#[component]
pub fn ButtonPage() -> Element {
    let disabled = use_signal(|| false);
    let variant = use_signal(|| components::ui::ButtonVariant::Default);
    let size = use_signal(|| components::ui::ButtonSize::Default);
    let button_type = use_signal(|| "button");
    let content = use_signal(|| ButtonContent::Text);
    let label = use_signal(|| "Save changes".to_string());
    rsx! {
        Demo {
            name: "Button",
            controls: rsx! {
                SelectControl {
                    label: "Variant",
                    value: variant,
                    options: crate::generated::controls::BUTTON_VARIANT_OPTIONS,
                }
                SelectControl {
                    label: "Size",
                    value: size,
                    options: crate::generated::controls::BUTTON_SIZE_OPTIONS,
                }
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
                variant: variant(),
                size: size(),
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
