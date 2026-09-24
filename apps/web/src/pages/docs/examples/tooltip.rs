//! Examples for `tooltip`.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::components::ui::tooltip::{Tooltip, TooltipContent, TooltipTrigger};

pub const SRC: &str = include_str!("tooltip.rs");

pub const METAS: &[DocExampleMeta] = &[DocExampleMeta {
    id: "composition",
    title: "Composition",
    description: "Hover or focus the trigger. Focus matters: a tooltip that only appears on hover is unreachable by keyboard, so `TooltipTrigger` responds to both.",
}];

pub fn render(id: &str) -> Element {
    match id {
        "composition" => rsx! { Composition {} },
        _ => rsx! {},
    }
}

#[component]
fn Composition() -> Element {
    rsx! {
        // doc-example:start composition
        Tooltip {
            TooltipTrigger { "Hover me" }
            TooltipContent { "Tooltips describe, they do not label." }
        }
        // doc-example:end
    }
}
