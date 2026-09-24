//! Examples for `accordion`.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::components::ui::accordion::{
    Accordion, AccordionContent, AccordionItem, AccordionMulti, AccordionTrigger,
};

pub const SRC: &str = include_str!("accordion.rs");

pub const METAS: &[DocExampleMeta] = &[
    DocExampleMeta {
        id: "single",
        title: "Single",
        description: "`Accordion` keeps at most one section open — opening one closes the others.",
    },
    DocExampleMeta {
        id: "multiple",
        title: "Multiple",
        description: "`AccordionMulti` is the same composition, but sections open independently. `index` seeds keyboard navigation, so it must mirror list position.",
    },
];

pub fn render(id: &str) -> Element {
    match id {
        "single" => rsx! { Single {} },
        "multiple" => rsx! { Multiple {} },
        _ => rsx! {},
    }
}

#[component]
fn Single() -> Element {
    rsx! {
        div { class: "w-full max-w-md",
            // doc-example:start single
            Accordion {
                AccordionItem { value: "shipping", index: 0usize,
                    AccordionTrigger { "Shipping" }
                    AccordionContent { "Ships within two business days." }
                }
                AccordionItem { value: "returns", index: 1usize,
                    AccordionTrigger { "Returns" }
                    AccordionContent { "Free returns within 30 days." }
                }
            }
            // doc-example:end
        }
    }
}

#[component]
fn Multiple() -> Element {
    rsx! {
        div { class: "w-full max-w-md",
            // doc-example:start multiple
            AccordionMulti {
                AccordionItem { value: "one", index: 0usize,
                    AccordionTrigger { "First" }
                    AccordionContent { "Both sections can be open at once." }
                }
                AccordionItem { value: "two", index: 1usize,
                    AccordionTrigger { "Second" }
                    AccordionContent { "Opening this one leaves the first open." }
                }
            }
            // doc-example:end
        }
    }
}
