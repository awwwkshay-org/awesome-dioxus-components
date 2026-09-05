use dioxus::prelude::*;

use crate::components::controls::TextControl;
use crate::components::demo::Demo;
use crate::generated::controls::{BadgeControls, BadgeDemoState, BadgePreview};

#[component]
pub fn BadgePage() -> Element {
    let state = use_signal(BadgeDemoState::default);
    let label = use_signal(|| "New".to_string());
    rsx! {
        Demo {
            name: "Badge",
            controls: rsx! {
                BadgeControls { state }
                TextControl { label: "Content", value: label }
            },
            BadgePreview { state: state(), "{label}" }
        }
    }
}
