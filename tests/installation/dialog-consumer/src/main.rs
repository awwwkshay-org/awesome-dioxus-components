use dioxus::prelude::*;

fn app() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        components::ui::Dialog {
            open: open(),
            on_open_change: move |value| open.set(value),
            components::ui::DialogTrigger { "Open dialog" }
            components::ui::DialogOverlay {}
                components::ui::DialogContent {
                    components::ui::DialogHeader {
                        components::ui::DialogTitle { "Edit profile" }
                        components::ui::DialogDescription { "Update your information." }
                    }
                    components::ui::Button { "Save changes" }
                    components::ui::Dialog {
                        components::ui::DialogTrigger { "Open nested dialog" }
                        components::ui::DialogOverlay {}
                        components::ui::DialogContent {
                            components::ui::DialogHeader {
                                components::ui::DialogTitle { "Nested dialog" }
                                components::ui::DialogDescription { "Nested content." }
                            }
                        }
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
