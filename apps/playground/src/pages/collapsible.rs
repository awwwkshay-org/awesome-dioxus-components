use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn CollapsiblePage() -> Element {
    let mut open = use_signal(|| true);
    rsx! {
        Demo {
            name: "Collapsible",
            components::ui::Collapsible {
                open: open(),
                on_open_change: move |value| open.set(value),
                components::ui::CollapsibleTrigger { "Toggle section" }
                components::ui::CollapsibleContent { "Collapsible content" }
            }
        }
    }
}
