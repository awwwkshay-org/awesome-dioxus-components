//! Examples for `switch`.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::components::ui::label::Label;
use crate::components::ui::switch::{Switch, SwitchSize};

pub const SRC: &str = include_str!("switch.rs");

pub const METAS: &[DocExampleMeta] = &[
    DocExampleMeta {
        id: "sizes",
        title: "Sizes",
        description: "Two sizes, both hit-target sized for touch.",
    },
    DocExampleMeta {
        id: "controlled",
        title: "Controlled and disabled",
        description: "`default_checked` for uncontrolled use; `checked` plus `on_checked_change` to drive it from a signal.",
    },
];

pub fn render(id: &str) -> Element {
    match id {
        "sizes" => rsx! { Sizes {} },
        "controlled" => rsx! { Controlled {} },
        _ => rsx! {},
    }
}

#[component]
fn Sizes() -> Element {
    rsx! {
        div { class: "flex items-center gap-6",
            // doc-example:start sizes
            Switch { size: SwitchSize::Sm, aria_label: "Small" }
            Switch { size: SwitchSize::Default, aria_label: "Default" }
            // doc-example:end
        }
    }
}

#[component]
fn Controlled() -> Element {
    // doc-example:start controlled
    let mut enabled = use_signal(|| true);
    rsx! {
        div { class: "flex flex-col gap-3",
            div { class: "flex items-center gap-3",
                Switch {
                    id: "switch-controlled",
                    checked: ReadSignal::from(Signal::new(Some(enabled()))),
                    on_checked_change: move |value| enabled.set(value),
                }
                Label { html_for: "switch-controlled", "Notifications: {enabled()}" }
            }
            div { class: "flex items-center gap-3",
                Switch { id: "switch-disabled", disabled: true }
                Label { html_for: "switch-disabled", "Disabled" }
            }
        }
    }
    // doc-example:end
}
