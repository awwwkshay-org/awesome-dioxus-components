use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, SelectControl};
use crate::components::demo::Demo;
use crate::generated::controls::{DrawerContentControls, DrawerContentDemoState};

#[component]
pub fn DrawerPage() -> Element {
    let mut open = use_signal(|| false);
    // `DrawerDirection` is an `Option<enum>` prop the generator skips, so
    // it is a hand-rolled page control (the `sheet.rs` side precedent).
    let direction = use_signal(|| components::ui::DrawerDirection::Bottom);
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
                SelectControl {
                    label: "Direction",
                    value: direction,
                    options: &[
                        ("Bottom", components::ui::DrawerDirection::Bottom),
                        ("Top", components::ui::DrawerDirection::Top),
                        ("Left", components::ui::DrawerDirection::Left),
                        ("Right", components::ui::DrawerDirection::Right),
                    ],
                }
                DrawerContentControls { state: content_state }
            },
            components::ui::Drawer {
                open: open(),
                on_open_change: move |value| open.set(value),
                components::ui::DrawerTrigger { "Open drawer" }
                components::ui::DrawerOverlay {}
                components::ui::DrawerContent {
                    direction: direction(),
                    show_close_button: content_state().show_close_button,
                    components::ui::DrawerHeader {
                        components::ui::DrawerTitle { "Move goal" }
                        components::ui::DrawerDescription { "Set your daily activity goal." }
                    }
                    div { class: "flex flex-col gap-4 py-2",
                        div { class: "flex flex-col gap-2",
                            components::ui::Label { html_for: "drawer-demo-goal", "Daily goal (calories)" }
                            components::ui::Input {
                                id: "drawer-demo-goal",
                                r#type: "number",
                                value: Some("350".to_string()),
                            }
                        }
                        p { class: "text-sm text-muted-foreground",
                            "You can change your goal at any time — progress carries over."
                        }
                    }
                    components::ui::DrawerFooter {
                        components::ui::Button { onclick: move |_| open.set(false), "Submit" }
                        components::ui::DrawerClose { "Cancel" }
                    }
                }
            }
        }
    }
}
