use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn KbdPage() -> Element {
    rsx! {
        Demo { name: "Kbd",
            components::ui::KbdGroup {
                components::ui::Kbd { "Ctrl" }
                span { "+" }
                components::ui::Kbd { "K" }
            }
        }
    }
}
