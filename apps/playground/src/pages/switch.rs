use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{SwitchControls, SwitchDemoState};

#[component]
pub fn SwitchPage() -> Element {
    let state = use_signal(SwitchDemoState::default);
    rsx! {
        Demo {
            name: "Switch",
            controls: rsx! {
                SwitchControls { state }
            },
            components::ui::Switch {
                checked: ReadSignal::from(Signal::new(state().checked)),
                default_checked: state().default_checked,
                size: state().size,
                aria_label: "Enable notifications",
            }
        }
    }
}
