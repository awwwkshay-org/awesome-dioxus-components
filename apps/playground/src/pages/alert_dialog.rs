use dioxus::prelude::*;

use crate::components;
use crate::components::controls::BoolControl;
use crate::components::demo::Demo;
use crate::generated::controls::{
    AlertDialogActionControls, AlertDialogActionDemoState, AlertDialogContentControls,
    AlertDialogContentDemoState,
};

#[component]
pub fn AlertDialogPage() -> Element {
    let mut open = use_signal(|| false);
    let content_state = use_signal(AlertDialogContentDemoState::default);
    let action_state = use_signal(AlertDialogActionDemoState::default);
    rsx! {
        Demo {
            name: "AlertDialog",
            controls: rsx! {
                BoolControl { label: "Open", value: open }
                AlertDialogContentControls { state: content_state }
                AlertDialogActionControls { state: action_state }
            },
            components::ui::AlertDialog {
                open: open(),
                on_open_change: move |value| open.set(value),
                components::ui::AlertDialogTrigger { "Delete item" }
                components::ui::AlertDialogOverlay {}
                components::ui::AlertDialogContent { size: content_state().size,
                    components::ui::AlertDialogHeader {
                        components::ui::AlertDialogTitle { "Delete item" }
                        components::ui::AlertDialogDescription { "Are you sure? This cannot be undone." }
                    }
                    components::ui::AlertDialogActions {
                        components::ui::AlertDialogCancel { "Cancel" }
                        components::ui::AlertDialogAction { loading: action_state().loading, "Delete" }
                    }
                }
            }
        }
    }
}
