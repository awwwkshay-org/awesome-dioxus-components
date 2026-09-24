use dioxus::prelude::*;

use super::data::ui_components;
use crate::components::ui::card::{Card, CardDescription, CardHeader, CardTitle};
use crate::routes::Route;

#[component]
pub fn DocsIndex() -> Element {
    let mut components = ui_components().collect::<Vec<_>>();
    components.sort_by(|a, b| a.name.cmp(&b.name));

    rsx! {
        div { class: "mx-auto flex w-full max-w-5xl flex-col gap-6 px-6 py-12",
            h1 { class: "text-3xl font-bold tracking-tight", "Components" }
            p { class: "max-w-2xl text-muted-foreground",
                "Every source-owned registry component, with its real usage, accessibility, and keyboard documentation."
            }
            div { class: "mt-4 grid gap-4 sm:grid-cols-2 lg:grid-cols-3",
                for component in components {
                    Link {
                        to: Route::DocsComponent {
                            name: component.name.clone(),
                        },
                        Card { class: "h-full transition-colors hover:border-primary/50",
                            CardHeader {
                                CardTitle { "{component.name}" }
                                CardDescription { "{component.description}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
