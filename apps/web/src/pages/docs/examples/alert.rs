//! Examples for `alert`.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::components::ui::alert::{Alert, AlertDescription, AlertTitle, AlertVariant};

pub const SRC: &str = include_str!("alert.rs");

pub const METAS: &[DocExampleMeta] = &[
    DocExampleMeta {
        id: "composition",
        title: "Composition",
        description: "`Alert` is a shell; `AlertTitle` and `AlertDescription` are optional children, so a title-only or description-only alert is valid.",
    },
    DocExampleMeta {
        id: "variants",
        title: "Variants",
        description: "Each variant carries its own semantic colour.",
    },
];

pub fn render(id: &str) -> Element {
    match id {
        "composition" => rsx! { Composition {} },
        "variants" => rsx! { Variants {} },
        _ => rsx! {},
    }
}

#[component]
fn Composition() -> Element {
    rsx! {
        div { class: "flex w-full max-w-md flex-col gap-3",
            // doc-example:start composition
            Alert {
                AlertTitle { "Heads up!" }
                AlertDescription { "You can add components to your app using the CLI." }
            }
            Alert {
                AlertTitle { "Title only" }
            }
            // doc-example:end
        }
    }
}

#[component]
fn Variants() -> Element {
    rsx! {
        div { class: "flex w-full max-w-md flex-col gap-3",
            // doc-example:start variants
            Alert { variant: AlertVariant::Default,
                AlertTitle { "Default" }
                AlertDescription { "A neutral, informational alert." }
            }
            Alert { variant: AlertVariant::Success,
                AlertTitle { "Success" }
                AlertDescription { "Your changes were saved." }
            }
            Alert { variant: AlertVariant::Warning,
                AlertTitle { "Warning" }
                AlertDescription { "This action cannot be undone." }
            }
            Alert { variant: AlertVariant::Destructive,
                AlertTitle { "Destructive" }
                AlertDescription { "Something went wrong and needs attention." }
            }
            Alert { variant: AlertVariant::Info,
                AlertTitle { "Info" }
                AlertDescription { "A new version is available." }
            }
            // doc-example:end
        }
    }
}
