use dioxus::prelude::*;

use crate::components;
use crate::components::controls::BoolControl;
use crate::components::demo::Demo;
use crate::generated::controls::{DrawerContentControls, DrawerContentDemoState};

#[component]
pub fn DrawerPage() -> Element {
    let mut open = use_signal(|| false);
    // `DrawerContent`'s own real default is `show_close_button: true`; see
    // `pages/dialog.rs`'s identical override for why.
    let content_state = use_signal(|| DrawerContentDemoState {
        show_close_button: true,
    });
    rsx! {
        Demo {
            name: "Drawer",
            controls: rsx! {
                BoolControl { label: "Open", value: open }
                DrawerContentControls { state: content_state }
            },
            components::ui::Drawer {
                open: open(),
                on_open_change: move |value| open.set(value),
                components::ui::DrawerTrigger { "Open drawer" }
                components::ui::DrawerOverlay {}
                components::ui::DrawerContent { show_close_button: content_state().show_close_button,
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
