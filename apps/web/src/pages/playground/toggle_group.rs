use std::collections::HashSet;

use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{ToggleItemControls, ToggleItemDemoState};

#[component]
pub fn ToggleGroupPage() -> Element {
    let mut pressed = use_signal(|| Some(HashSet::from([0usize])));
    let state = use_signal(ToggleItemDemoState::default);
    rsx! {
        Demo {
            name: "Toggle Group",
            controls: rsx! {
                ToggleItemControls { state }
            },
            components::ui::ToggleGroup {
                pressed: ReadSignal::from(pressed),
                on_pressed_change: move |value| pressed.set(Some(value)),
                components::ui::ToggleItem { index: 0usize, size: state().size, variant: state().variant, "Bold" }
                components::ui::ToggleItem { index: 1usize, size: state().size, variant: state().variant, "Italic" }
                components::ui::ToggleItem {
                    index: 2usize,
                    size: state().size,
                    variant: state().variant,
                    "Underline"
                }
            }
        }
    }
}
