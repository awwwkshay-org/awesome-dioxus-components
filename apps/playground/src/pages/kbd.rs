use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{KbdControls, KbdGroupControls};

#[component]
pub fn KbdPage() -> Element {
    rsx! {
        Demo {
            name: "Kbd",
            controls: rsx! {
                KbdGroupControls {}
                KbdControls {}
            },
            components::ui::KbdGroup {
                components::ui::Kbd { "Ctrl" }
                span { "+" }
                components::ui::Kbd { "K" }
            }
        }
    }
}
