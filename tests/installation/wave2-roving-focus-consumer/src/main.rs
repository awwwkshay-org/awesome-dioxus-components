use dioxus::prelude::*;

fn app() -> Element {
    let mut radio_value = use_signal(|| "blue".to_string());
    let mut tab_value = use_signal(|| "tab1".to_string());

    rsx! {
        components::ui::Accordion { allow_multiple_open: false,
            components::ui::AccordionItem { index: 0usize,
                components::ui::AccordionTrigger { "Section one" }
                components::ui::AccordionContent { "Section one content." }
            }
            components::ui::AccordionItem { index: 1usize,
                components::ui::AccordionTrigger { "Section two" }
                components::ui::AccordionContent { "Section two content." }
            }
        }
        components::ui::RadioGroup {
            value: Some(radio_value()),
            on_value_change: move |value| radio_value.set(value),
            components::ui::RadioItem { value: "blue".to_string(), index: 0usize, "Blue" }
            components::ui::RadioItem { value: "red".to_string(), index: 1usize, "Red" }
        }
        components::ui::Tabs {
            value: Some(tab_value()),
            on_value_change: move |value| tab_value.set(value),
            components::ui::TabList {
                components::ui::TabTrigger { value: "tab1".to_string(), index: 0usize, "Tab 1" }
                components::ui::TabTrigger { value: "tab2".to_string(), index: 1usize, "Tab 2" }
            }
            components::ui::TabContent { value: "tab1".to_string(), index: 0usize, "Tab 1 content" }
            components::ui::TabContent { value: "tab2".to_string(), index: 1usize, "Tab 2 content" }
        }
        components::ui::ToggleGroup { horizontal: true,
            components::ui::ToggleItem { index: 0usize, "Bold" }
            components::ui::ToggleItem { index: 1usize, "Italic" }
        }
    }
}

fn main() {
    launch(app);
}

// adico:start
pub mod adico_lib;
pub mod components;
// adico:end
