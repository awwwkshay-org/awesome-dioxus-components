//! Examples for `tabs`.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::components::ui::tabs::{TabContent, TabList, TabTrigger, Tabs, TabsVariant};

pub const SRC: &str = include_str!("tabs.rs");

pub const METAS: &[DocExampleMeta] = &[
    DocExampleMeta {
        id: "variants",
        title: "Variants",
        description: "`Default` is a filled pill tray; `Line` is a flat underlined strip.",
    },
    DocExampleMeta {
        id: "controlled",
        title: "Controlled",
        description: "`TabTrigger` and `TabContent` pair by `value`, and `index` seeds roving-tabindex keyboard navigation — arrow keys move between tabs.",
    },
];

pub fn render(id: &str) -> Element {
    match id {
        "variants" => rsx! { Variants {} },
        "controlled" => rsx! { Controlled {} },
        _ => rsx! {},
    }
}

#[component]
fn Variants() -> Element {
    rsx! {
        div { class: "flex w-full max-w-md flex-col gap-6",
            // doc-example:start variants
            Tabs { default_value: "a".to_string(),
                TabList { variant: TabsVariant::Default,
                    TabTrigger { value: "a".to_string(), index: 0usize, "Account" }
                    TabTrigger { value: "b".to_string(), index: 1usize, "Password" }
                }
                TabContent { value: "a".to_string(), index: 0usize, "Account settings." }
                TabContent { value: "b".to_string(), index: 1usize, "Password settings." }
            }
            Tabs { default_value: "a".to_string(),
                TabList { variant: TabsVariant::Line,
                    TabTrigger { value: "a".to_string(), index: 0usize, "Account" }
                    TabTrigger { value: "b".to_string(), index: 1usize, "Password" }
                }
                TabContent { value: "a".to_string(), index: 0usize, "Account settings." }
                TabContent { value: "b".to_string(), index: 1usize, "Password settings." }
            }
            // doc-example:end
        }
    }
}

#[component]
fn Controlled() -> Element {
    // doc-example:start controlled
    let mut active = use_signal(|| "overview".to_string());
    rsx! {
        div { class: "flex w-full max-w-md flex-col gap-2",
            Tabs {
                value: Some(active()),
                on_value_change: move |value| active.set(value),
                TabList {
                    TabTrigger { value: "overview".to_string(), index: 0usize, "Overview" }
                    TabTrigger { value: "activity".to_string(), index: 1usize, "Activity" }
                }
                TabContent { value: "overview".to_string(), index: 0usize, "Overview panel." }
                TabContent { value: "activity".to_string(), index: 1usize, "Activity panel." }
            }
            p { class: "text-sm text-muted-foreground", "Active tab: {active()}" }
        }
    }
    // doc-example:end
}
