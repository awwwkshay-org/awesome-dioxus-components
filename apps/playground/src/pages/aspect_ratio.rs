use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{ControlGroup, SelectControl};
use crate::components::demo::Demo;
use crate::generated::controls::AspectRatioControls;

#[component]
pub fn AspectRatioPage() -> Element {
    let ratio = use_signal(|| 16.0 / 9.0);
    rsx! {
        Demo {
            name: "Aspect Ratio",
            controls: rsx! {
                ControlGroup { part: "Aspect Ratio",
                    SelectControl {
                        label: "Ratio",
                        value: ratio,
                        options: &[("16:9", 16.0 / 9.0), ("4:3", 4.0 / 3.0), ("1:1", 1.0)],
                    }
                }
                AspectRatioControls {}
            },
            components::ui::AspectRatio { ratio: ratio(), class: "w-full max-w-sm",
                div {
                    class: "flex h-full w-full items-center justify-center rounded-md bg-muted text-muted-foreground",
                    "{ratio():.2}"
                }
            }
        }
    }
}
