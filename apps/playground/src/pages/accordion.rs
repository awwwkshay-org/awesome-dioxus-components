use dioxus::prelude::*;

use crate::components;
use crate::components::controls::BoolControl;
use crate::components::demo::Demo;
use crate::generated::controls::{AccordionItemControls, AccordionItemDemoState};

#[component]
pub fn AccordionPage() -> Element {
    let allow_multiple_open = use_signal(|| false);
    // `AccordionItem.index` normally mirrors each item's fixed list position
    // (0, 1, ...), so only "Section one" is bound to the generated control --
    // "Section two" stays hardcoded at `1usize` as the reference sibling.
    // Setting both to the same index would collide their roving-tabindex
    // keyboard-nav order, which the control is meant to demonstrate, not hide.
    let item_one_state = use_signal(AccordionItemDemoState::default);
    rsx! {
        Demo {
            name: "Accordion",
            wide: true,
            controls: rsx! {
                BoolControl { label: "Allow multiple open", value: allow_multiple_open }
                AccordionItemControls { state: item_one_state }
            },
            if allow_multiple_open() {
                components::ui::AccordionMulti {
                    components::ui::AccordionItem { value: "section-one", index: item_one_state().index,
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
                    components::ui::AccordionItem { value: "section-one", index: item_one_state().index,
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
