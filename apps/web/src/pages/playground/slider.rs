use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{
    SliderControls, SliderRangeControls, SliderThumbControls, SliderTrackControls,
};

#[component]
pub fn SliderPage() -> Element {
    rsx! {
        Demo {
            name: "Slider",
            controls: rsx! {
                SliderControls {}
                SliderTrackControls {}
                SliderRangeControls {}
                SliderThumbControls {}
            },
            components::ui::Slider { label: "Volume", default_value: 50.0,
                components::ui::SliderTrack {
                    components::ui::SliderRange {}
                    components::ui::SliderThumb {}
                }
            }
        }
    }
}
