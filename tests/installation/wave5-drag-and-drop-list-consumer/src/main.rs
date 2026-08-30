use dioxus::prelude::*;

fn app() -> Element {
    let items = ["Alpha", "Bravo", "Charlie"]
        .map(|t| rsx! { {t} })
        .to_vec();
    rsx! {
        components::ui::DragAndDropList { items, aria_label: "Reorderable items" }
    }
}

fn main() {
    launch(app);
}

// adico:start
pub mod adico_lib;
pub mod components;
// adico:end
