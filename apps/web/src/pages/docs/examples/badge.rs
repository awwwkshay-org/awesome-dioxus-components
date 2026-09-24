//! Examples for `badge`.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::adico_lib::variants::Tone;
use crate::components::ui::badge::{Badge, BadgeVariant};

pub const SRC: &str = include_str!("badge.rs");

pub const METAS: &[DocExampleMeta] = &[
    DocExampleMeta {
        id: "variants",
        title: "Variants",
        description: "`BadgeVariant` mirrors `ButtonVariant`'s shape axis — a badge is a `<span>`, not a control, but the visual vocabulary is shared.",
    },
    DocExampleMeta {
        id: "tones",
        title: "Colours",
        description: "`Tone` is a separate axis, so any tone composes with any variant.",
    },
];

pub fn render(id: &str) -> Element {
    match id {
        "variants" => rsx! { Variants {} },
        "tones" => rsx! { Tones {} },
        _ => rsx! {},
    }
}

#[component]
fn Variants() -> Element {
    rsx! {
        div { class: "flex flex-wrap items-center gap-3",
            // doc-example:start variants
            Badge { variant: BadgeVariant::Primary, "Primary" }
            Badge { variant: BadgeVariant::Secondary, "Secondary" }
            Badge { variant: BadgeVariant::Outline, "Outline" }
            Badge { variant: BadgeVariant::Ghost, "Ghost" }
            Badge { variant: BadgeVariant::Link, "Link" }
            // doc-example:end
        }
    }
}

#[component]
fn Tones() -> Element {
    rsx! {
        div { class: "flex flex-wrap items-center gap-3",
            // doc-example:start tones
            Badge { color: Tone::Success, "Success" }
            Badge { color: Tone::Warning, "Warning" }
            Badge { color: Tone::Error, "Error" }
            Badge { color: Tone::Info, "Info" }
            Badge { variant: BadgeVariant::Outline, color: Tone::Success, "Outline success" }
            // doc-example:end
        }
    }
}
