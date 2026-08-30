use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn DragAndDropListPage() -> Element {
    let items = ["Alpha", "Bravo", "Charlie"].map(|t| rsx! { {t} }).to_vec();
    rsx! {
        Demo {
            name: "DragAndDropList",
            components::ui::DragAndDropList { items, aria_label: "Reorderable items" }
        }
    }
}
