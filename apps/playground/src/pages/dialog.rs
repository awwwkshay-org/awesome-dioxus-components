use dioxus::prelude::*;

use crate::components;
use crate::components::controls::BoolControl;
use crate::components::demo::Demo;
use crate::generated::controls::{DialogContentControls, DialogContentDemoState};

#[component]
pub fn DialogPage() -> Element {
    let mut open = use_signal(|| false);
    let mut notifications = use_signal(|| true);
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
                components::ui::DialogTrigger { "Edit profile" }
                components::ui::DialogOverlay {}
                components::ui::DialogContent { show_close_button: content_state().show_close_button,
                    components::ui::DialogHeader {
                        components::ui::DialogTitle { "Edit profile" }
                        components::ui::DialogDescription {
                            "Make changes to your profile here. Click save when you're done."
                        }
                    }
                    div { class: "flex flex-col gap-4 py-2",
                        div { class: "flex flex-col gap-2",
                            components::ui::Label { html_for: "dialog-demo-name", "Name" }
                            components::ui::Input {
                                id: "dialog-demo-name",
                                value: Some("Ada Lovelace".to_string()),
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            components::ui::Label { html_for: "dialog-demo-username", "Username" }
                            components::ui::Input {
                                id: "dialog-demo-username",
                                value: Some("@ada".to_string()),
                            }
                        }
                        div { class: "flex items-center justify-between",
                            components::ui::Label { html_for: "dialog-demo-notifications", "Email notifications" }
                            components::ui::Switch {
                                id: "dialog-demo-notifications",
                                checked: ReadSignal::from(Signal::new(Some(notifications()))),
                                on_checked_change: move |checked| notifications.set(checked),
                            }
                        }
                    }
                    components::ui::DialogFooter {
                        components::ui::DialogClose { "Cancel" }
                        components::ui::Button { onclick: move |_| open.set(false), "Save changes" }
                    }
                }
            }
        }
    }
}
