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
                components::ui::RadioItem { value: "blue".to_string(), index: 0usize, "Blue" }
                components::ui::RadioItem { value: "red".to_string(), index: 1usize, "Red" }
            }
        }
    }
}
