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
    let mut variant = use_signal(|| components::ui::ButtonVariant::Default);
    let mut size = use_signal(|| components::ui::ButtonSize::Default);
    let mut button_type = use_signal(|| "button".to_string());
    let mut content = use_signal(|| ButtonContent::Text);
    let label = use_signal(|| "Save changes".to_string());
    rsx! {
        Demo {
            name: "Button",
            controls: rsx! {
                SelectControl {
                    label: "Variant",
                    value: variant(),
                    options: vec![
                        ("Default", components::ui::ButtonVariant::Default),
                        ("Destructive", components::ui::ButtonVariant::Destructive),
                        ("Outline", components::ui::ButtonVariant::Outline),
                        ("Secondary", components::ui::ButtonVariant::Secondary),
                        ("Ghost", components::ui::ButtonVariant::Ghost),
                        ("Link", components::ui::ButtonVariant::Link),
                    ],
                    on_change: move |value| variant.set(value),
                }
                SelectControl {
                    label: "Size",
                    value: size(),
                    options: vec![
                        ("Default", components::ui::ButtonSize::Default),
                        ("Extra small", components::ui::ButtonSize::Xs),
                        ("Small", components::ui::ButtonSize::Sm),
                        ("Large", components::ui::ButtonSize::Lg),
                        ("Icon", components::ui::ButtonSize::Icon),
                        ("Icon extra small", components::ui::ButtonSize::IconXs),
                        ("Icon small", components::ui::ButtonSize::IconSm),
                        ("Icon large", components::ui::ButtonSize::IconLg),
                    ],
                    on_change: move |value| size.set(value),
                }
                BoolControl { label: "Disabled", value: disabled }
                SelectControl {
                    label: "Native type",
                    value: button_type(),
                    options: vec![
                        ("Button", "button".to_string()),
                        ("Submit", "submit".to_string()),
                        ("Reset", "reset".to_string()),
                    ],
                    on_change: move |value| button_type.set(value),
                }
                SelectControl {
                    label: "Children",
                    value: content(),
                    options: vec![
                        ("Text", ButtonContent::Text),
                        ("Icon only", ButtonContent::Icon),
                        ("Icon and text", ButtonContent::IconAndText),
                    ],
                    on_change: move |value| content.set(value),
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
