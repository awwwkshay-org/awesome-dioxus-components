use dioxus::prelude::*;

fn main() {
    #[cfg(feature = "server")]
    dioxus::serve(|| async move { Ok(dioxus::server::router(App)) });

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        main {
            h1 { "adico basic-ssr example" }
            components::ui::Button { "SSR/hydration smoke check" }
            components::ui::Dialog {
                open: open(),
                on_open_change: move |value| open.set(value),
                components::ui::DialogTrigger { "Open dialog" }
                components::ui::DialogOverlay {}
                components::ui::DialogContent {
                    components::ui::DialogHeader {
                        components::ui::DialogTitle { "Hydration check" }
                        components::ui::DialogDescription { "Renders on the server and hydrates on the client." }
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
