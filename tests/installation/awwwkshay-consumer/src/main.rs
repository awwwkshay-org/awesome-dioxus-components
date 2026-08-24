use dioxus::prelude::*;

fn app() -> Element {
    rsx! {
        components::ui::Card {
            components::ui::Button { "Install me" }
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
