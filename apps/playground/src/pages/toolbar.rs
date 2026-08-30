use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn ToolbarPage() -> Element {
    let mut active = use_signal(|| None::<&'static str>);
    rsx! {
        Demo {
            name: "Toolbar",
            div { class: "flex flex-col gap-2",
                components::ui::Toolbar { aria_label: "Text formatting",
                    components::ui::ToolbarButton {
                        index: 0usize,
                        on_click: move |_| active.set(Some("Bold")),
                        "Bold"
                    }
                    components::ui::ToolbarButton {
                        index: 1usize,
                        on_click: move |_| active.set(Some("Italic")),
                        "Italic"
                    }
                    components::ui::ToolbarSeparator {}
                    components::ui::ToolbarButton {
                        index: 2usize,
                        on_click: move |_| active.set(Some("Underline")),
                        "Underline"
                    }
                }
                p { class: "text-sm text-muted-foreground",
                    "Last action: {active().unwrap_or(\"none\")}"
                }
            }
        }
    }
}
