use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn TogglePage() -> Element {
    let mut pressed = use_signal(|| Some(false));
    rsx! {
        Demo {
            name: "Toggle",
            components::ui::Toggle {
                pressed: ReadSignal::from(pressed),
                on_pressed_change: move |value| pressed.set(Some(value)),
                "Bold"
            }
        }
    }
}
