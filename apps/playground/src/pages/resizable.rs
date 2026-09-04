use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn ResizablePage() -> Element {
    rsx! {
        Demo {
            name: "Resizable",
            controls: rsx! {
                p { class: "self-end pb-2 text-sm text-muted-foreground", "Drag the handle, or focus it and use the arrow keys." }
            },
            components::ui::ResizablePanelGroup { class: "h-64 rounded-md border",
                components::ui::ResizablePanel { index: 0usize, default_size: 50.0,
                    div { class: "flex h-full items-center justify-center text-sm", "One" }
                }
                components::ui::ResizableHandle { handle_index: 0usize, with_handle: true }
                components::ui::ResizablePanel { index: 1usize, default_size: 50.0,
                    div { class: "flex h-full items-center justify-center text-sm", "Two" }
                }
            }
        }
    }
}
