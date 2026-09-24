//! The Preview / Code presentation for one docs example.
//!
//! App-level wiring under `apps/web/src/components/`: it composes the
//! installed `Card`, `Tabs`, and (via `CodeBlock`) `CopyButton`, per
//! `adico-web-structure`'s "composes real registry components, not
//! app-specific reimplementations" requirement. Layout classes are applied to
//! this file's own wrapper elements, never passed into a registry component's
//! `class` where they could collide — `cn()` is a plain join with no
//! last-wins conflict resolution.

use dioxus::prelude::*;

use crate::components::code_block::CodeBlock;
use crate::components::prose::Prose;
use crate::components::ui::card::Card;
use crate::components::ui::tabs::{TabContent, TabList, TabTrigger, Tabs, TabsVariant};

/// One example: a title, a description, the live component, and its source.
#[component]
pub fn DocExample(
    id: String,
    title: String,
    description: String,
    code: Option<String>,
    children: Element,
) -> Element {
    // Tab values are namespaced by example id so several examples can sit on
    // one page without their tab state colliding.
    let preview_value = format!("{id}-preview");
    let code_value = format!("{id}-code");
    let mut active = use_signal({
        let preview_value = preview_value.clone();
        move || preview_value.clone()
    });

    rsx! {
        section { class: "flex flex-col gap-3",
            div { class: "flex flex-col gap-1",
                h3 { class: "text-base font-semibold tracking-tight", "{title}" }
                if !description.is_empty() {
                    Prose { text: description }
                }
            }

            Tabs {
                value: Some(active()),
                on_value_change: move |value| active.set(value),

                TabList {
                    variant: TabsVariant::Line,
                    TabTrigger { value: preview_value.clone(), index: 0usize, "Preview" }
                    // Only offer the Code tab when the source actually
                    // resolved; a tab that opens onto nothing is worse than
                    // no tab.
                    if code.is_some() {
                        TabTrigger { value: code_value.clone(), index: 1usize, "Code" }
                    }
                }

                TabContent { value: preview_value.clone(), index: 0usize,
                    Card { class: "mt-3",
                        div { class: "flex min-h-[7rem] w-full flex-wrap items-center justify-center gap-4 p-6",
                            {children}
                        }
                    }
                }

                if let Some(code) = code {
                    TabContent { value: code_value.clone(), index: 1usize,
                        CodeBlock { code, class: "mt-3" }
                    }
                }
            }
        }
    }
}
