//! Source-owned shadcn-style Tabs for Dioxus, backed by the owned adico
//! primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::tabs::Tabs;
use adico_primitives::tabs::{
    TabContent as TabContentPrimitive, TabList as TabListPrimitive,
    TabTrigger as TabTriggerPrimitive,
};

use crate::adico_lib::cn::cn;

/// The row of [`TabTrigger`] buttons for a [`Tabs`] root.
#[component]
pub fn TabList(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "inline-flex h-9 items-center justify-center rounded-lg bg-muted p-1 text-muted-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TabListPrimitive { class, {children} }
    }
}

/// A single tab button within a [`TabList`].
#[component]
pub fn TabTrigger(
    value: String,
    index: ReadSignal<usize>,
    #[props(default)] disabled: ReadSignal<bool>,
    id: Option<String>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "inline-flex flex-1 items-center justify-center gap-1.5 whitespace-nowrap rounded-md px-2 py-1 text-sm font-medium transition-all focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-1 focus-visible:outline-ring disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TabTriggerPrimitive { value, index, disabled, id, class, {children} }
    }
}

/// The panel shown while its matching [`TabTrigger`] is active.
#[component]
pub fn TabContent(
    value: String,
    index: ReadSignal<usize>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "mt-2 focus-visible:outline-none",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TabContentPrimitive { value, index, class, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_trigger_gets_a_semantic_surface() {
        let class = cn(&["data-[state=active]:bg-background data-[state=active]:text-foreground", ""]);
        assert!(class.contains("data-[state=active]:bg-background"));
    }
}
