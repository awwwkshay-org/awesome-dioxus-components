//! Examples for `textarea`.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::components::ui::label::Label;
use crate::components::ui::textarea::Textarea;

pub const SRC: &str = include_str!("textarea.rs");

pub const METAS: &[DocExampleMeta] = &[
    DocExampleMeta {
        id: "states",
        title: "States",
        description: "The same state props as `Input`: `disabled`, `readonly`, `invalid`.",
    },
    DocExampleMeta {
        id: "counter",
        title: "Character counter",
        description: "Setting `max_length` adds the native `maxlength` attribute *and* renders a live counter. Type in it.",
    },
];

pub fn render(id: &str) -> Element {
    match id {
        "states" => rsx! { States {} },
        "counter" => rsx! { Counter {} },
        _ => rsx! {},
    }
}

#[component]
fn States() -> Element {
    rsx! {
        div { class: "flex w-full max-w-sm flex-col gap-3",
            // doc-example:start states
            Textarea { placeholder: "Default" }
            Textarea { placeholder: "Disabled", disabled: true }
            Textarea { placeholder: "Invalid", invalid: true }
            // doc-example:end
        }
    }
}

#[component]
fn Counter() -> Element {
    rsx! {
        div { class: "flex w-full max-w-sm flex-col gap-2",
            // doc-example:start counter
            Label { html_for: "textarea-example-bio", "Bio" }
            Textarea {
                id: "textarea-example-bio",
                placeholder: "Tell us about yourself",
                max_length: 280u32,
            }
            // doc-example:end
        }
    }
}
