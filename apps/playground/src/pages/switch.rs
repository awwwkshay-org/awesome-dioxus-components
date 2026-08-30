use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn SwitchPage() -> Element {
    let mut checked = use_signal(|| false);
    rsx! {
        Demo {
            name: "Switch",
            components::ui::Switch {
                checked: checked(),
                on_checked_change: move |value| checked.set(value),
                aria_label: "Enable notifications",
            }
        }
    }
}
