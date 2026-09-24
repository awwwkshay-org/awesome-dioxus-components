use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{AlertControls, AlertDemoState};

#[component]
pub fn AlertPage() -> Element {
    let state = use_signal(AlertDemoState::default);
    rsx! {
        Demo {
            name: "Alert",
            controls: rsx! {
                AlertControls { state }
            },
            components::ui::Alert { class: "max-w-md", variant: state().variant,
                components::ui::AlertTitle { "Heads up!" }
                components::ui::AlertDescription { "You can add components to your app using the CLI." }
            }
        }
    }
}
