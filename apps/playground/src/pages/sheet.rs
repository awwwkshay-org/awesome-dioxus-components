use dioxus::prelude::*;

use crate::components;
use crate::components::controls::SelectControl;
use crate::components::demo::Demo;

#[component]
pub fn SheetPage() -> Element {
    let mut side = use_signal(|| components::ui::SheetSide::Right);
    rsx! {
        Demo {
            name: "Sheet",
            controls: rsx! {
                SelectControl {
                    label: "Side",
                    value: side(),
                    options: vec![
                        ("Right", components::ui::SheetSide::Right),
                        ("Left", components::ui::SheetSide::Left),
                        ("Top", components::ui::SheetSide::Top),
                        ("Bottom", components::ui::SheetSide::Bottom),
                    ],
                    on_change: move |value| side.set(value),
                }
            },
            components::ui::Sheet {
                components::ui::SheetTrigger { "Open sheet" }
                components::ui::SheetOverlay {}
                components::ui::SheetContent { side: side(),
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
