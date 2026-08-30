use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{SelectControl, TextControl};
use crate::components::demo::Demo;

#[component]
pub fn BadgePage() -> Element {
    let mut variant = use_signal(|| components::ui::BadgeVariant::Default);
    let label = use_signal(|| "New".to_string());
    rsx! {
        Demo {
            name: "Badge",
            controls: rsx! {
                SelectControl {
                    label: "Variant",
                    value: variant(),
                    options: vec![
                        ("Default", components::ui::BadgeVariant::Default),
                        ("Secondary", components::ui::BadgeVariant::Secondary),
                        ("Destructive", components::ui::BadgeVariant::Destructive),
                        ("Outline", components::ui::BadgeVariant::Outline),
                        ("Verified", components::ui::BadgeVariant::Verified),
                    ],
                    on_change: move |value| variant.set(value),
                }
                TextControl { label: "Content", value: label }
            },
            components::ui::Badge { variant: variant(), "{label}" }
        }
    }
}
