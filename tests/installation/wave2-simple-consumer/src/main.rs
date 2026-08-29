use dioxus::prelude::*;

fn app() -> Element {
    rsx! {
        components::ui::AspectRatio { ratio: 16.0 / 9.0,
            div { style: "background-color: lightblue; width: 100%; height: 100%;",
                "16:9"
            }
        }
        components::ui::Label { html_for: "name", "Name" }
        input { id: "name", placeholder: "Enter your name" }
        components::ui::Progress { value: 50.0 }
    }
}

fn main() {
    launch(app);
}

// adico:start
pub mod adico_lib;
pub mod components;
// adico:end
