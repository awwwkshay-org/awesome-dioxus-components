use dioxus::prelude::*;

use crate::components;
use crate::components::controls::SelectControl;
use crate::components::demo::Demo;

#[component]
pub fn TogglePage() -> Element {
    let mut pressed = use_signal(|| Some(false));
    let size = use_signal(|| components::ui::ToggleSize::Default);
    let variant = use_signal(|| components::ui::ToggleVariant::Default);
    rsx! {
        Demo {
            name: "Toggle",
            controls: rsx! {
                SelectControl {
                    label: "Size",
                    value: size,
                    options: crate::generated::controls::TOGGLE_SIZE_OPTIONS,
                }
                SelectControl {
                    label: "Variant",
                    value: variant,
                    options: crate::generated::controls::TOGGLE_VARIANT_OPTIONS,
                }
            },
            components::ui::Toggle {
                pressed: ReadSignal::from(pressed),
                on_pressed_change: move |value| pressed.set(Some(value)),
                size: size(),
                variant: variant(),
                "Bold"
            }
        }
    }
}
