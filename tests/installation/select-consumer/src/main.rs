use dioxus::prelude::*;

fn app() -> Element {
    rsx! {
        components::ui::Select::<String> {
            components::ui::SelectTrigger {
                aria_label: "Choose a fruit",
                components::ui::SelectValue { placeholder: "Choose a fruit" }
            }
            components::ui::SelectList { aria_label: "Fruit options",
                components::ui::SelectOption::<String> {
                    index: 0usize,
                    value: "apple",
                    text_value: "Apple",
                    "Apple"
                }
                components::ui::SelectOption::<String> {
                    index: 1usize,
                    value: "banana",
                    text_value: "Banana",
                    "Banana"
                }
                components::ui::SelectOption::<String> {
                    index: 2usize,
                    value: "cherry",
                    text_value: "Cherry",
                    "Cherry"
                }
            }
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
