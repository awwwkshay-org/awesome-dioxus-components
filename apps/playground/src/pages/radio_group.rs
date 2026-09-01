use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn RadioGroupPage() -> Element {
    let mut value = use_signal(|| "blue".to_string());
    rsx! {
        Demo {
            name: "RadioGroup",
            components::ui::RadioGroup {
                value: Some(value()),
                on_value_change: move |v| value.set(v),
                div { class: "flex items-center gap-2",
                    components::ui::RadioItem { value: "blue".to_string(), index: 0usize, id: "radio-blue".to_string() }
                    components::ui::Label { html_for: "radio-blue".to_string(), "Blue" }
                }
                div { class: "flex items-center gap-2",
                    components::ui::RadioItem { value: "red".to_string(), index: 1usize, id: "radio-red".to_string() }
                    components::ui::Label { html_for: "radio-red".to_string(), "Red" }
                }
            }
        }
    }
}
