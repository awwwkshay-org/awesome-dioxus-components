use dioxus::prelude::*;

use super::data::{component_props, find_component};
use crate::components::cta_link::{CtaLink, CtaLinkVariant};
use crate::components::ui::badge::Badge;
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};

#[component]
pub fn DocsComponent(name: String) -> Element {
    let Some(component) = find_component(&name) else {
        return rsx! {
            div { class: "mx-auto flex w-full max-w-3xl flex-col gap-4 px-6 py-12",
                CtaLink { href: "/docs".to_string(), variant: CtaLinkVariant::Ghost, "← Back to components" }
                h1 { class: "text-3xl font-bold tracking-tight", "Not found" }
                p { class: "text-muted-foreground", "No registry component named \"{name}\"." }
            }
        };
    };
    let doc = component.documentation.as_ref();

    rsx! {
        div { class: "mx-auto flex w-full max-w-3xl flex-col gap-6 px-6 py-12",
            CtaLink { href: "/docs".to_string(), variant: CtaLinkVariant::Ghost, "← Back to components" }
            div { class: "flex items-center gap-3",
                h1 { class: "text-3xl font-bold tracking-tight", "{component.name}" }
                Badge { "{component.item_type}" }
            }
            p { class: "text-muted-foreground", "{component.description}" }

            if let Some(note) = doc.and_then(|d| d.composition_note.as_deref()) {
                Card {
                    CardHeader { CardTitle { "Composition note" } }
                    CardContent { p { class: "text-sm text-muted-foreground", "{note}" } }
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
                    CardContent { p { class: "text-sm text-muted-foreground", "{accessibility}" } }
                }
            }

            if let Some(keyboard) = doc.and_then(|d| d.keyboard.as_deref()) {
                Card {
                    CardHeader { CardTitle { "Keyboard" } }
                    CardContent { p { class: "text-sm text-muted-foreground", "{keyboard}" } }
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
