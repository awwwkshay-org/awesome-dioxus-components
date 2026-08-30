use dioxus::prelude::*;

use crate::components;
use crate::components::controls::BoolControl;
use crate::components::demo::Demo;

#[component]
pub fn AlertDialogPage() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        Demo {
            name: "AlertDialog",
            controls: rsx! {
                BoolControl { label: "Open", value: open }
            },
            components::ui::AlertDialog {
                open: open(),
                on_open_change: move |value| open.set(value),
                components::ui::AlertDialogTrigger { "Delete item" }
                components::ui::AlertDialogOverlay {}
                components::ui::AlertDialogContent {
                    components::ui::AlertDialogHeader {
                        components::ui::AlertDialogTitle { "Delete item" }
                        components::ui::AlertDialogDescription { "Are you sure? This cannot be undone." }
                    }
                    components::ui::AlertDialogActions {
                        components::ui::AlertDialogCancel { "Cancel" }
                        components::ui::AlertDialogAction { "Delete" }
                    }
                }
            }
        }
    }
}
