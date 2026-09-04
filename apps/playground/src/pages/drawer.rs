use dioxus::prelude::*;

use crate::components;
use crate::components::controls::BoolControl;
use crate::components::demo::Demo;

#[component]
pub fn DrawerPage() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        Demo {
            name: "Drawer",
            controls: rsx! {
                BoolControl { label: "Open", value: open }
            },
            components::ui::Drawer {
                open: open(),
                on_open_change: move |value| open.set(value),
                components::ui::DrawerTrigger { "Open drawer" }
                components::ui::DrawerOverlay {}
                components::ui::DrawerContent {
                    components::ui::DrawerHeader {
                        components::ui::DrawerTitle { "Installed through adico" }
                        components::ui::DrawerDescription { "This Drawer source belongs to this app." }
                    }
                    components::ui::DrawerFooter {
                        components::ui::DrawerClose { "Close" }
                    }
                }
            }
        }
    }
}
