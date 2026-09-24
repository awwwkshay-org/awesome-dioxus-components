//! Examples for `checkbox`.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::components::ui::checkbox::{Checkbox, CheckboxState};
use crate::components::ui::label::Label;

pub const SRC: &str = include_str!("checkbox.rs");

pub const METAS: &[DocExampleMeta] = &[
    DocExampleMeta {
        id: "states",
        title: "States",
        description: "`CheckboxState` is tri-state: `Unchecked`, `Checked`, and `Indeterminate` for a partially-selected group.",
    },
    DocExampleMeta {
        id: "controlled",
        title: "Controlled",
        description: "Drive it from a signal with `checked` plus `on_checked_change`. Click it — the label below tracks the state.",
    },
];

pub fn render(id: &str) -> Element {
    match id {
        "states" => rsx! { States {} },
        "controlled" => rsx! { Controlled {} },
        _ => rsx! {},
    }
}

#[component]
fn States() -> Element {
    rsx! {
        div { class: "flex flex-col gap-3",
            // doc-example:start states
            div { class: "flex items-center gap-2",
                Checkbox { id: "cb-unchecked", checked: CheckboxState::Unchecked }
                Label { html_for: "cb-unchecked", "Unchecked" }
            }
            div { class: "flex items-center gap-2",
                Checkbox { id: "cb-checked", checked: CheckboxState::Checked }
                Label { html_for: "cb-checked", "Checked" }
            }
            div { class: "flex items-center gap-2",
                Checkbox { id: "cb-indeterminate", checked: CheckboxState::Indeterminate }
                Label { html_for: "cb-indeterminate", "Indeterminate" }
            }
            div { class: "flex items-center gap-2",
                Checkbox { id: "cb-disabled", disabled: true }
                Label { html_for: "cb-disabled", "Disabled" }
            }
            // doc-example:end
        }
    }
}

#[component]
fn Controlled() -> Element {
    // doc-example:start controlled
    let mut checked = use_signal(|| CheckboxState::Unchecked);
    rsx! {
        div { class: "flex items-center gap-2",
            Checkbox {
                id: "cb-controlled",
                checked: checked(),
                on_checked_change: move |value| checked.set(value),
            }
            Label { html_for: "cb-controlled", "Subscribe — currently {checked():?}" }
        }
    }
    // doc-example:end
}
