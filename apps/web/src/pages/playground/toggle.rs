use dioxus::prelude::*;

use crate::components::demo::Demo;
use crate::generated::controls::{ToggleControls, ToggleDemoState, TogglePreview};

#[component]
pub fn TogglePage() -> Element {
    let state = use_signal(ToggleDemoState::default);
    rsx! {
        Demo {
            name: "Toggle",
            controls: rsx! {
                ToggleControls { state }
            },
            TogglePreview { state: state(), "Bold" }
        }
    }
}
