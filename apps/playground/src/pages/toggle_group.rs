use std::collections::HashSet;

use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn ToggleGroupPage() -> Element {
    let mut pressed = use_signal(|| Some(HashSet::from([0usize])));
    rsx! {
        Demo {
            name: "ToggleGroup",
            components::ui::ToggleGroup {
                pressed: ReadSignal::from(pressed),
                on_pressed_change: move |value| pressed.set(Some(value)),
                components::ui::ToggleItem { index: 0usize, "Bold" }
                components::ui::ToggleItem { index: 1usize, "Italic" }
                components::ui::ToggleItem { index: 2usize, "Underline" }
            }
        }
    }
}
