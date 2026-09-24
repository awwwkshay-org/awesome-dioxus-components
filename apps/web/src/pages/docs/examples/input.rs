//! Examples for `input`.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::components::ui::input::Input;
use crate::components::ui::label::Label;

pub const SRC: &str = include_str!("input.rs");

pub const METAS: &[DocExampleMeta] = &[
    DocExampleMeta {
        id: "states",
        title: "States",
        description: "`disabled`, `readonly`, and `invalid` are separate props. `invalid` sets `aria-invalid`, which is what drives the error ring.",
    },
    DocExampleMeta {
        id: "types",
        title: "Input types",
        description: "`type` is forwarded to the native `<input>`, so browser behaviour (keyboards, pickers, validation) comes for free.",
    },
    DocExampleMeta {
        id: "labelled",
        title: "With a label",
        description: "Pair with `Label` and a matching `html_for`/`id` so clicking the label focuses the field.",
    },
];

pub fn render(id: &str) -> Element {
    match id {
        "states" => rsx! { States {} },
        "types" => rsx! { Types {} },
        "labelled" => rsx! { Labelled {} },
        _ => rsx! {},
    }
}

#[component]
fn States() -> Element {
    rsx! {
        div { class: "flex w-full max-w-sm flex-col gap-3",
            // doc-example:start states
            Input { placeholder: "Default" }
            Input { placeholder: "Disabled", disabled: true }
            Input { placeholder: "Read-only", readonly: true }
            Input { placeholder: "Invalid", invalid: true }
            // doc-example:end
        }
    }
}

#[component]
fn Types() -> Element {
    rsx! {
        div { class: "flex w-full max-w-sm flex-col gap-3",
            // doc-example:start types
            Input { r#type: "email", placeholder: "m@example.com" }
            Input { r#type: "password", placeholder: "Password" }
            Input { r#type: "number", placeholder: "42" }
            Input { r#type: "date" }
            // doc-example:end
        }
    }
}

#[component]
fn Labelled() -> Element {
    rsx! {
        // doc-example:start labelled
        div { class: "flex w-full max-w-sm flex-col gap-2",
            Label { html_for: "input-example-name", "Name" }
            Input { id: "input-example-name", placeholder: "Ada Lovelace" }
        }
        // doc-example:end
    }
}
