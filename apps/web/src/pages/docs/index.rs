use dioxus::prelude::*;

use super::data::ui_components;
use super::guides::GUIDES;
use crate::components::ui::card::{Card, CardDescription, CardHeader, CardTitle};
use crate::routes::Route;

#[component]
pub fn DocsIndex() -> Element {
    let mut components = ui_components().collect::<Vec<_>>();
    components.sort_by(|a, b| a.name.cmp(&b.name));

    rsx! {
        div { class: "mx-auto flex w-full max-w-5xl flex-col gap-6 px-6 py-12",
            h1 { class: "text-h1", "Documentation" }
            p { class: "text-lead max-w-2xl text-muted-foreground",
                "Guides for setting adico up and theming it, then every source-owned registry component with live examples, accessibility, and keyboard documentation."
            }

            // Guides are listed here as well as in the sidebar, because the
            // sidebar is `lg:` and up. This is what keeps every guide route
            // reachable at narrow widths without a second mobile nav
            // mechanism beside the playground's sheet.
            section { class: "flex flex-col gap-4",
                h2 { class: "text-h2", "Guides" }
                div { class: "grid items-stretch gap-4 sm:grid-cols-2",
                    for (href , title , summary) in GUIDES {
                        Link { key: "{href}", class: "block min-w-0", to: *href,
                            Card { class: "h-full transition-colors hover:border-primary/50",
                                CardHeader {
                                    CardTitle { class: "min-w-0 break-words", "{title}" }
                                    CardDescription { class: "min-w-0 break-words text-pretty",
                                        "{summary}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            h2 { class: "text-h2", "Components" }
            div { class: "mt-4 grid items-stretch gap-4 sm:grid-cols-2 lg:grid-cols-3",
                for component in components {
                    Link {
                        class: "block min-w-0",
                        to: Route::DocsComponent {
                            name: component.name.clone(),
                        },
                        Card { class: "h-full transition-colors hover:border-primary/50",
                            CardHeader {
                                CardTitle { class: "min-w-0 break-words", "{component.name}" }
                                // Several registry descriptions contain a long
                                // slash-joined composition list with no spaces
                                // (`(Bubble/BubbleContent/BubbleReactions/...)`).
                                // A flex item defaults to `min-width: auto`, so
                                // without `min-w-0` that token refuses to shrink
                                // and spills past the card border; `break-words`
                                // then wraps the token itself.
                                CardDescription { class: "min-w-0 break-words text-pretty",
                                    "{component.description}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
