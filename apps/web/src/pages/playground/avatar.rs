use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{AvatarControls, AvatarDemoState};

#[component]
pub fn AvatarPage() -> Element {
    let state = use_signal(AvatarDemoState::default);
    rsx! {
        Demo {
            name: "Avatar",
            controls: rsx! {
                AvatarControls { state }
            },
            components::ui::Avatar { size: state().size,
                components::ui::AvatarFallback { "AB" }
            }
        }
    }
}
