use dioxus::prelude::*;

use super::data::{component_props, find_component};
use super::examples;
use crate::components::code_block::CodeBlock;
use crate::components::cta_link::{CtaLink, CtaLinkVariant};
use crate::components::doc_example::DocExample;
use crate::components::prose::Prose;
use crate::components::ui::badge::Badge;
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};

#[component]
pub fn DocsComponent(name: String) -> Element {
    let Some(component) = find_component(&name) else {
        return rsx! {
            div { class: "mx-auto flex w-full max-w-3xl flex-col gap-4 px-6 py-12",
                CtaLink { href: "/docs".to_string(), variant: CtaLinkVariant::Ghost, "← Back to components" }
                h1 { class: "text-h1", "Not found" }
                p { class: "text-muted-foreground", "No registry component named \"{name}\"." }
            }
        };
    };
    let doc = component.documentation.as_ref();

    rsx! {
        div { class: "mx-auto flex w-full max-w-3xl flex-col gap-6 px-6 py-12",
            CtaLink { href: "/docs".to_string(), variant: CtaLinkVariant::Ghost, "← Back to components" }
            div { class: "flex flex-wrap items-center gap-3",
                h1 { class: "text-h1", "{component.name}" }
                Badge { "{component.item_type}" }
            }
            Prose {
                text: component.description.clone(),
                class: "text-lead text-muted-foreground",
            }

            // Install command and playground link: the two things a reader is
            // most likely to want next, so they sit above the reference
            // material rather than at the bottom of the page.
            div { class: "flex flex-col gap-3",
                CodeBlock { code: format!("adico add {}", component.name) }
                div { class: "flex flex-wrap gap-3",
                    CtaLink {
                        href: format!("/playground/{}", component.name),
                        variant: CtaLinkVariant::Outline,
                        "Open in playground →"
                    }
                }
            }

            // Examples come before the prose: seeing the component is the
            // point of the page. A component with no example module skips
            // this section entirely rather than rendering an empty one.
            if let Some(item) = examples::for_item(&component.name) {
                section { class: "flex flex-col gap-8",
                    h2 { class: "text-h2", "Examples" }
                    for meta in item.metas {
                        // Keyed by component *and* example: navigating
                        // /docs/components/a → /b reuses the element at the
                        // same tree position, which otherwise carries the
                        // previous page's Preview/Code selection across with
                        // it. Including the component name forces a remount,
                        // so every page opens on Preview.
                        DocExample {
                            key: "{component.name}-{meta.id}",
                            id: meta.id.to_string(),
                            title: meta.title.to_string(),
                            description: meta.description.to_string(),
                            code: examples::extract(item.source, meta.id),
                            {(item.render)(meta.id)}
                        }
                    }
                }
            }

            if let Some(note) = doc.and_then(|d| d.composition_note.as_deref()) {
                Card {
                    CardHeader { CardTitle { "Composition note" } }
                    CardContent { Prose { text: note.to_string() } }
                }
            }

            if let Some(usage) = doc.and_then(|d| d.usage.as_deref()) {
                Card {
                    CardHeader { CardTitle { "Usage" } }
                    CardContent {
                        pre { class: "overflow-x-auto rounded-md bg-muted p-4 text-xs",
                            code { "{usage}" }
                        }
                    }
                }
            }

            if let Some(accessibility) = doc.and_then(|d| d.accessibility.as_deref()) {
                Card {
                    CardHeader { CardTitle { "Accessibility" } }
                    CardContent { Prose { text: accessibility.to_string() } }
                }
            }

            if let Some(keyboard) = doc.and_then(|d| d.keyboard.as_deref()) {
                Card {
                    CardHeader { CardTitle { "Keyboard" } }
                    CardContent { Prose { text: keyboard.to_string() } }
                }
            }

            {
                let item_key = component.name.replace('-', "_");
                match component_props().get(&item_key) {
                    Some(sub_components) if !sub_components.is_empty() => rsx! {
                        for (sub_component_name , fields) in sub_components {
                            Card {
                                CardHeader { CardTitle { "{sub_component_name} props" } }
                                CardContent {
                                    Table {
                                        TableHeader {
                                            TableRow {
                                                TableHead { "Name" }
                                                TableHead { "Type" }
                                                TableHead { "Default" }
                                            }
                                        }
                                        TableBody {
                                            for field in fields {
                                                TableRow {
                                                    TableCell { code { "{field.name}" } }
                                                    TableCell { code { "{field.type_name}" } }
                                                    TableCell {
                                                        code { {field.default.as_deref().unwrap_or("—")} }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    _ => rsx! {
                        Card {
                            CardHeader { CardTitle { "Props" } }
                            CardContent {
                                p { class: "text-sm text-muted-foreground",
                                    "No props are visible to introspection for this component — its root is a bare re-export of a primitive with no local wrapper."
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}
