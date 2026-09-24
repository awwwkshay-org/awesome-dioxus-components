//! Examples for `select`.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::components::ui::select::{Select, SelectList, SelectOption, SelectTrigger, SelectValue};

pub const SRC: &str = include_str!("select.rs");

pub const METAS: &[DocExampleMeta] = &[DocExampleMeta {
    id: "composition",
    title: "Composition",
    description: "`SelectValue` renders the chosen option's text, falling back to `placeholder`. `text_value` is what type-ahead matches against, so it stays plain text even when the option renders richer children.",
}];

pub fn render(id: &str) -> Element {
    match id {
        "composition" => rsx! { Composition {} },
        _ => rsx! {},
    }
}

#[component]
fn Composition() -> Element {
    // doc-example:start composition
    let mut value = use_signal(|| None::<String>);
    rsx! {
        Select::<String> {
            value: ReadSignal::from(value),
            on_value_change: move |next| value.set(next),
            SelectTrigger { class: "w-48", aria_label: "Choose a fruit",
                SelectValue { placeholder: "Choose a fruit" }
            }
            SelectList { class: "w-48", aria_label: "Fruit options",
                SelectOption::<String> { index: 0usize, value: "apple", text_value: "Apple", "Apple" }
                SelectOption::<String> { index: 1usize, value: "banana", text_value: "Banana", "Banana" }
                SelectOption::<String> { index: 2usize, value: "cherry", text_value: "Cherry", "Cherry" }
            }
        }
    }
    // doc-example:end
}
