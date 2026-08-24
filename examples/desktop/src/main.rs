use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        main {
            h1 { "adico desktop example" }
            components::ui::Button { "Native smoke check" }
        }
    }
}

// adico:start
pub mod adico_lib;
pub mod components;
// adico:end
