use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, SelectControl};
use crate::components::demo::Demo;
use crate::generated::controls::{SheetContentControls, SheetContentDemoState};

#[component]
pub fn SheetPage() -> Element {
    let mut open = use_signal(|| false);
    let side = use_signal(|| components::ui::SheetSide::Right);
    let mut marketing_emails = use_signal(|| false);
    // `SheetContent`'s own real default is `show_close_button: true`; see
    // `pages/dialog.rs`'s identical override for why.
    let content_state = use_signal(|| SheetContentDemoState {
        show_close_button: true,
    });
    rsx! {
        Demo {
            name: "Sheet",
            controls: rsx! {
                BoolControl { label: "Open", value: open }
                SelectControl {
                    label: "Side",
                    value: side,
                    options: &[
                        ("Right", components::ui::SheetSide::Right),
                        ("Left", components::ui::SheetSide::Left),
                        ("Top", components::ui::SheetSide::Top),
                        ("Bottom", components::ui::SheetSide::Bottom),
                    ],
                }
                SheetContentControls { state: content_state }
            },
            components::ui::Sheet {
                open: open(),
                on_open_change: move |value| open.set(value),
                components::ui::SheetTrigger { "Open sheet" }
                components::ui::SheetOverlay {}
                components::ui::SheetContent { side: side(), show_close_button: content_state().show_close_button,
                    components::ui::SheetHeader {
                        components::ui::SheetTitle { "Settings" }
                        components::ui::SheetDescription { "Adjust your preferences." }
                    }
                    div { class: "flex flex-col gap-4 py-2",
                        div { class: "flex flex-col gap-2",
                            components::ui::Label { html_for: "sheet-demo-display-name", "Display name" }
                            components::ui::Input {
                                id: "sheet-demo-display-name",
                                placeholder: "How others see you",
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            components::ui::Label { html_for: "sheet-demo-email", "Email" }
                            components::ui::Input {
                                id: "sheet-demo-email",
                                r#type: "email",
                                placeholder: "m@example.com",
                            }
                        }
                        div { class: "flex items-center justify-between",
                            components::ui::Label { html_for: "sheet-demo-marketing", "Marketing emails" }
                            components::ui::Switch {
                                id: "sheet-demo-marketing",
                                checked: ReadSignal::from(Signal::new(Some(marketing_emails()))),
                                on_checked_change: move |checked| marketing_emails.set(checked),
                            }
                        }
                    }
                    components::ui::SheetFooter {
                        components::ui::SheetClose { "Close" }
                        components::ui::Button { "Save" }
                    }
                }
            }
        }
    }
}
