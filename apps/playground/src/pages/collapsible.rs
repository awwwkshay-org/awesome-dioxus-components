use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

/// A realistic "show more" composition: two starred repositories always
/// visible, three more revealed by the trigger — the canonical shadcn
/// Collapsible pattern, exercising the styled chevron trigger.
#[component]
pub fn CollapsiblePage() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        Demo {
            name: "Collapsible",
            components::ui::Collapsible {
                open: open(),
                on_open_change: move |value| open.set(value),
                class: "w-full max-w-sm",
                div { class: "mb-2 flex items-center justify-between",
                    span { class: "text-sm font-semibold", "Starred repositories" }
                }
                div { class: "rounded-md border px-4 py-2 font-mono text-sm shadow-sm",
                    "dioxuslabs/dioxus"
                }
                components::ui::CollapsibleTrigger {
                    if open() {
                        "Show less"
                    } else {
                        "Show 3 more"
                    }
                }
                components::ui::CollapsibleContent {
                    div { class: "rounded-md border px-4 py-2 font-mono text-sm shadow-sm",
                        "tailwindlabs/tailwindcss"
                    }
                    div { class: "rounded-md border px-4 py-2 font-mono text-sm shadow-sm",
                        "rust-lang/rust"
                    }
                    div { class: "rounded-md border px-4 py-2 font-mono text-sm shadow-sm",
                        "radix-ui/primitives"
                    }
                }
            }
        }
    }
}
