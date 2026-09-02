use dioxus::prelude::*;

use crate::components;
use crate::components::controls::BoolControl;
use crate::components::demo::Demo;

#[component]
pub fn AccordionPage() -> Element {
    let allow_multiple_open = use_signal(|| false);
    rsx! {
        Demo {
            name: "Accordion",
            wide: true,
            controls: rsx! {
                BoolControl { label: "Allow multiple open", value: allow_multiple_open }
            },
            if allow_multiple_open() {
                components::ui::AccordionMulti {
                    components::ui::AccordionItem { value: "section-one", index: 0usize,
                        components::ui::AccordionTrigger { "Section one" }
                        components::ui::AccordionContent { "Section one content." }
                    }
                    components::ui::AccordionItem { value: "section-two", index: 1usize,
                        components::ui::AccordionTrigger { "Section two" }
                        components::ui::AccordionContent { "Section two content." }
                    }
                }
            } else {
                components::ui::Accordion {
                    components::ui::AccordionItem { value: "section-one", index: 0usize,
                        components::ui::AccordionTrigger { "Section one" }
                        components::ui::AccordionContent { "Section one content." }
                    }
                    components::ui::AccordionItem { value: "section-two", index: 1usize,
                        components::ui::AccordionTrigger { "Section two" }
                        components::ui::AccordionContent { "Section two content." }
                    }
                }
            }
        }
    }
}
