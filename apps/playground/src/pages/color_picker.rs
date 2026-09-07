use dioxus::prelude::*;
use palette::{IntoColor, encoding};

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{ColorPickerPopoverControls, ColorPickerPopoverDemoState};

#[component]
pub fn ColorPickerPage() -> Element {
    let mut color = use_signal(|| -> palette::Hsv<encoding::Srgb, f64> {
        components::ui::color_picker::Color::new(155, 128, 255)
            .into_format::<f64>()
            .into_color()
    });
    let popover_state = use_signal(ColorPickerPopoverDemoState::default);
    rsx! {
        Demo {
            name: "Color Picker",
            controls: rsx! {
                ColorPickerPopoverControls { state: popover_state }
            },
            components::ui::ColorPicker {
                color: color(),
                on_color_change: move |c| color.set(c),
                components::ui::ColorPickerPopover {
                    open: popover_state().open,
                    default_open: popover_state().default_open,
                    components::ui::ColorPickerTrigger {}
                    components::ui::PopoverContent { class: "w-auto p-4",
                        components::ui::ColorArea {
                            components::ui::AreaTrack {
                                components::ui::AreaThumb {
                                    components::ui::AreaThumbSaturationInput {}
                                    components::ui::AreaThumbValueInput {}
                                }
                            }
                        }
                        components::ui::HueSlider {}
                        components::ui::ColorPickerFields {}
                    }
                }
            }
        }
    }
}
