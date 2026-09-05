use dioxus::prelude::*;

use crate::components;
use crate::components::controls::BoolControl;
use crate::components::demo::Demo;
use crate::generated::controls::{DialogContentControls, DialogContentDemoState};

#[component]
pub fn DialogPage() -> Element {
    let mut open = use_signal(|| false);
    // `DialogContent`'s own real default is `show_close_button: true`; the
    // generator's fixed-default convention always starts a bool field at
    // `false` (design.md's D2), so this overrides the initial demo value
    // to match the real component's own default rather than silently
    // changing what this page shows on first load.
    let content_state = use_signal(|| DialogContentDemoState {
        show_close_button: true,
    });
    rsx! {
        Demo {
            name: "Dialog",
            controls: rsx! {
                BoolControl { label: "Open", value: open }
                DialogContentControls { state: content_state }
            },
            components::ui::Dialog {
                open: open(),
                on_open_change: move |value| open.set(value),
                components::ui::DialogTrigger { "Open dialog" }
                components::ui::DialogOverlay {}
                components::ui::DialogContent { show_close_button: content_state().show_close_button,
                    components::ui::DialogHeader {
                        components::ui::DialogTitle { "Installed through adico" }
                        components::ui::DialogDescription { "This Dialog source belongs to this app." }
                    }
                }
            }
        }
    }
}
