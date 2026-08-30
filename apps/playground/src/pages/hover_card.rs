use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, SelectControl};
use crate::components::demo::Demo;

#[component]
pub fn HoverCardPage() -> Element {
    let mut open = use_signal(|| None::<bool>);
    let disabled = use_signal(|| false);
    rsx! {
        Demo {
            name: "HoverCard",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                SelectControl {
                    label: "Open state",
                    value: open(),
                    options: vec![("Uncontrolled", None), ("Closed", Some(false)), ("Open", Some(true))],
                    on_change: move |value| open.set(value),
                }
            },
            components::ui::HoverCard { open: open, disabled: disabled(),
                components::ui::HoverCardTrigger { "Dioxus" }
                components::ui::HoverCardContent { "Hover card content" }
            }
        }
    }
}
