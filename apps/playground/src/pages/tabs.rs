use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{TabListControls, TabListDemoState};

#[component]
pub fn TabsPage() -> Element {
    let mut value = use_signal(|| "tab1".to_string());
    let state = use_signal(TabListDemoState::default);
    rsx! {
        Demo {
            name: "Tabs",
            controls: rsx! {
                TabListControls { state }
            },
            components::ui::Tabs {
                value: Some(value()),
                on_value_change: move |v| value.set(v),
                components::ui::TabList {
                    variant: state().variant,
                    components::ui::TabTrigger { value: "tab1".to_string(), index: 0usize, "Tab 1" }
                    components::ui::TabTrigger { value: "tab2".to_string(), index: 1usize, "Tab 2" }
                }
                components::ui::TabContent { value: "tab1".to_string(), index: 0usize, "Tab 1 content" }
                components::ui::TabContent { value: "tab2".to_string(), index: 1usize, "Tab 2 content" }
            }
        }
    }
}
