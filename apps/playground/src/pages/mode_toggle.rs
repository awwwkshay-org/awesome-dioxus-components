use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn ModeTogglePage() -> Element {
    rsx! {
        Demo {
            name: "ModeToggle",
            components::ui::ModeToggle {}
        }
    }
}
