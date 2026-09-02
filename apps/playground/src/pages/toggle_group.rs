use std::collections::HashSet;

use dioxus::prelude::*;

use crate::components;
use crate::components::controls::SelectControl;
use crate::components::demo::Demo;

#[component]
pub fn ToggleGroupPage() -> Element {
    let mut pressed = use_signal(|| Some(HashSet::from([0usize])));
    let mut size = use_signal(|| components::ui::ToggleItemSize::Default);
    let mut variant = use_signal(|| components::ui::ToggleItemVariant::Default);
    rsx! {
        Demo {
            name: "ToggleGroup",
            controls: rsx! {
                SelectControl {
                    label: "Item size",
                    value: size(),
                    options: crate::generated::controls::TOGGLE_ITEM_SIZE_OPTIONS.to_vec(),
                    on_change: move |value| size.set(value),
                }
                SelectControl {
                    label: "Item variant",
                    value: variant(),
                    options: crate::generated::controls::TOGGLE_ITEM_VARIANT_OPTIONS.to_vec(),
                    on_change: move |value| variant.set(value),
                }
            },
            components::ui::ToggleGroup {
                pressed: ReadSignal::from(pressed),
                on_pressed_change: move |value| pressed.set(Some(value)),
                components::ui::ToggleItem { index: 0usize, size: size(), variant: variant(), "Bold" }
                components::ui::ToggleItem { index: 1usize, size: size(), variant: variant(), "Italic" }
                components::ui::ToggleItem { index: 2usize, size: size(), variant: variant(), "Underline" }
            }
        }
    }
}
