use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        main { class: "space-y-6 p-6",
            h1 { "adico basic-spa example" }
            components::ui::Button { "Source-owned Button" }
            components::ui::Dialog {
                components::ui::DialogTrigger { "Open dialog" }
                components::ui::DialogOverlay {}
                components::ui::DialogContent {
                    components::ui::DialogHeader {
                        components::ui::DialogTitle { "Installed through adico" }
                        components::ui::DialogDescription {
                            "This Dialog source belongs to this example."
                        }
                    }
                }
            }
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
                }
            }
        }
    }
}

// adico:start
pub mod adico_lib;
pub mod components;
// adico:end
