use dioxus::prelude::*;

use crate::components;
use crate::components::controls::SelectControl;
use crate::components::demo::Demo;
use crate::generated::controls::{SheetContentControls, SheetContentDemoState};

#[component]
pub fn SheetPage() -> Element {
    let side = use_signal(|| components::ui::SheetSide::Right);
    // `SheetContent`'s own real default is `show_close_button: true`; see
    // `pages/dialog.rs`'s identical override for why.
    let content_state = use_signal(|| SheetContentDemoState {
        show_close_button: true,
    });
    rsx! {
        Demo {
            name: "Sheet",
            controls: rsx! {
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
                components::ui::SheetTrigger { "Open sheet" }
                components::ui::SheetOverlay {}
                components::ui::SheetContent { side: side(), show_close_button: content_state().show_close_button,
                    components::ui::SheetHeader {
                        components::ui::SheetTitle { "Settings" }
                        components::ui::SheetDescription { "Adjust your preferences." }
                    }
                    components::ui::SheetFooter { "Done" }
                }
            }
        }
    }
}
