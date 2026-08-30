use dioxus::prelude::*;

use crate::components;
use crate::components::controls::SelectControl;
use crate::components::demo::Demo;

#[component]
pub fn AspectRatioPage() -> Element {
    let mut ratio = use_signal(|| 16.0 / 9.0);
    rsx! {
        Demo {
            name: "AspectRatio",
            controls: rsx! {
                SelectControl {
                    label: "Ratio",
                    value: ratio(),
                    options: vec![("16:9", 16.0 / 9.0), ("4:3", 4.0 / 3.0), ("1:1", 1.0)],
                    on_change: move |value| ratio.set(value),
                }
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
