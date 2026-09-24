//! Examples for `button`.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::adico_lib::variants::Tone;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};

pub const SRC: &str = include_str!("button.rs");

pub const METAS: &[DocExampleMeta] = &[
    DocExampleMeta {
        id: "variants",
        title: "Variants",
        description: "Five shapes. `ButtonVariant` controls shape only — colour is a separate axis.",
    },
    DocExampleMeta {
        id: "tones",
        title: "Colours",
        description: "`Tone` is orthogonal to shape, so every tone works with every variant. There is no `Destructive` variant: that look is `color: Tone::Error`.",
    },
    DocExampleMeta {
        id: "sizes",
        title: "Sizes",
        description: "Four text sizes and four matching icon-only sizes.",
    },
    DocExampleMeta {
        id: "loading",
        title: "Loading and disabled",
        description: "`loading` swaps in a `Spinner`, disables the button, and sets `aria-busy`. With `loading_text` it replaces the label.",
    },
];

pub fn render(id: &str) -> Element {
    match id {
        "variants" => rsx! { Variants {} },
        "tones" => rsx! { Tones {} },
        "sizes" => rsx! { Sizes {} },
        "loading" => rsx! { Loading {} },
        _ => rsx! {},
    }
}

#[component]
fn Variants() -> Element {
    rsx! {
        div { class: "flex flex-wrap items-center gap-3",
            // doc-example:start variants
            Button { variant: ButtonVariant::Primary, "Primary" }
            Button { variant: ButtonVariant::Secondary, "Secondary" }
            Button { variant: ButtonVariant::Outline, "Outline" }
            Button { variant: ButtonVariant::Ghost, "Ghost" }
            Button { variant: ButtonVariant::Link, "Link" }
            // doc-example:end
        }
    }
}

#[component]
fn Tones() -> Element {
    rsx! {
        div { class: "flex flex-col gap-3",
            // doc-example:start tones
            div { class: "flex flex-wrap items-center gap-3",
                Button { color: Tone::Default, "Default" }
                Button { color: Tone::Success, "Success" }
                Button { color: Tone::Warning, "Warning" }
                Button { color: Tone::Error, "Error" }
                Button { color: Tone::Info, "Info" }
            }
            div { class: "flex flex-wrap items-center gap-3",
                Button { variant: ButtonVariant::Secondary, color: Tone::Success, "Success" }
                Button { variant: ButtonVariant::Outline, color: Tone::Error, "Error" }
                Button { variant: ButtonVariant::Ghost, color: Tone::Info, "Info" }
            }
            // doc-example:end
        }
    }
}

#[component]
fn Sizes() -> Element {
    rsx! {
        div { class: "flex flex-col gap-3",
            // doc-example:start sizes
            div { class: "flex flex-wrap items-center gap-3",
                Button { size: ButtonSize::Xs, "Extra small" }
                Button { size: ButtonSize::Sm, "Small" }
                Button { size: ButtonSize::Default, "Default" }
                Button { size: ButtonSize::Lg, "Large" }
            }
            div { class: "flex flex-wrap items-center gap-3",
                Button { size: ButtonSize::IconXs, aria_label: "Add", "+" }
                Button { size: ButtonSize::IconSm, aria_label: "Add", "+" }
                Button { size: ButtonSize::Icon, aria_label: "Add", "+" }
                Button { size: ButtonSize::IconLg, aria_label: "Add", "+" }
            }
            // doc-example:end
        }
    }
}

#[component]
fn Loading() -> Element {
    rsx! {
        div { class: "flex flex-wrap items-center gap-3",
            // doc-example:start loading
            Button { loading: true, "Save" }
            Button { loading: true, loading_text: "Saving…", "Save" }
            Button { variant: ButtonVariant::Outline, disabled: true, "Disabled" }
            // doc-example:end
        }
    }
}
