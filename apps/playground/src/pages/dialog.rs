use dioxus::prelude::*;

use crate::components;
use crate::components::controls::BoolControl;
use crate::components::demo::Demo;

#[component]
pub fn DialogPage() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        Demo {
            name: "Dialog",
            controls: rsx! {
                BoolControl { label: "Open", value: open }
            },
            components::ui::Dialog {
                open: open(),
                on_open_change: move |value| open.set(value),
                components::ui::DialogTrigger { "Open dialog" }
                components::ui::DialogOverlay {}
                components::ui::DialogContent {
                    components::ui::DialogHeader {
                        components::ui::DialogTitle { "Installed through adico" }
                        components::ui::DialogDescription { "This Dialog source belongs to this app." }
                    }
                }
            }
        }
    }
}
