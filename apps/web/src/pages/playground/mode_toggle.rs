use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::ModeToggleControls;

#[component]
pub fn ModeTogglePage() -> Element {
    rsx! {
        Demo {
            name: "Mode Toggle",
            controls: rsx! {
                ModeToggleControls {}
            },
            components::ui::ModeToggle {}
        }
    }
}
