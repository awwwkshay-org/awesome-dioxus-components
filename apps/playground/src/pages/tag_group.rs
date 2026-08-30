use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn TagGroupPage() -> Element {
    let mut value = use_signal(|| Some("rust".to_string()));
    rsx! {
        Demo {
            name: "TagGroup",
            components::ui::TagGroup::<String> {
                value: Some(ReadSignal::from(value)),
                on_value_change: move |v| value.set(v),
                components::ui::TagGroupLabel { "Favorite language" }
                components::ui::TagList {
                    components::ui::TagOption::<String> { value: "rust".to_string(), index: 0usize, "Rust" }
                    components::ui::TagOption::<String> { value: "dioxus".to_string(), index: 1usize, "Dioxus" }
                    components::ui::TagOption::<String> {
                        value: "typescript".to_string(),
                        index: 2usize,
                        "TypeScript"
                    }
                }
            }
        }
    }
}
