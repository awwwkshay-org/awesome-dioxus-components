//! Examples for `label`.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::components::ui::checkbox::Checkbox;
use crate::components::ui::input::Input;
use crate::components::ui::label::Label;

pub const SRC: &str = include_str!("label.rs");

pub const METAS: &[DocExampleMeta] = &[DocExampleMeta {
    id: "pairing",
    title: "Pairing with a control",
    description: "`html_for` must match the control's `id`. That association is what makes the label clickable and what a screen reader announces with the field.",
}];

pub fn render(id: &str) -> Element {
    match id {
        "pairing" => rsx! { Pairing {} },
        _ => rsx! {},
    }
}

#[component]
fn Pairing() -> Element {
    rsx! {
        div { class: "flex w-full max-w-sm flex-col gap-4",
            // doc-example:start pairing
            div { class: "flex flex-col gap-2",
                Label { html_for: "label-example-email", "Email" }
                Input { id: "label-example-email", r#type: "email", placeholder: "m@example.com" }
            }
            div { class: "flex items-center gap-2",
                Checkbox { id: "label-example-terms" }
                Label { html_for: "label-example-terms", "Accept terms and conditions" }
            }
            // doc-example:end
        }
    }
}
