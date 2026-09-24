use dioxus::prelude::*;

use adico_primitives::icons::Check;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{MarkerControls, MarkerDemoState};

#[component]
pub fn MarkerPage() -> Element {
    let state = use_signal(MarkerDemoState::default);
    rsx! {
        Demo {
            name: "Marker",
            controls: rsx! {
                MarkerControls { state }
            },
            components::ui::Marker { variant: state().variant,
                components::ui::MarkerIcon { Check { class: "size-3" } }
                components::ui::MarkerContent { "Step 1" }
            }
        }
    }
}
