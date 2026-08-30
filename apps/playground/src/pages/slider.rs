use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn SliderPage() -> Element {
    rsx! {
        Demo {
            name: "Slider",
            components::ui::Slider { label: "Volume", default_value: 50.0,
                components::ui::SliderTrack {
                    components::ui::SliderRange {}
                    components::ui::SliderThumb {}
                }
            }
        }
    }
}
