use dioxus::prelude::*;

use crate::components;
use crate::components::controls::SelectControl;
use crate::components::demo::Demo;

#[component]
pub fn TabsPage() -> Element {
    let mut value = use_signal(|| "tab1".to_string());
    let mut variant = use_signal(|| components::ui::TabsVariant::Default);
    rsx! {
        Demo {
            name: "Tabs",
            controls: rsx! {
                SelectControl {
                    label: "Variant",
                    value: variant(),
                    options: vec![
                        ("Default", components::ui::TabsVariant::Default),
                        ("Line", components::ui::TabsVariant::Line),
                    ],
                    on_change: move |value| variant.set(value),
                }
            },
            components::ui::Tabs {
                value: Some(value()),
                on_value_change: move |v| value.set(v),
                components::ui::TabList {
                    variant: variant(),
                    components::ui::TabTrigger { value: "tab1".to_string(), index: 0usize, "Tab 1" }
                    components::ui::TabTrigger { value: "tab2".to_string(), index: 1usize, "Tab 2" }
                }
                components::ui::TabContent { value: "tab1".to_string(), index: 0usize, "Tab 1 content" }
                components::ui::TabContent { value: "tab2".to_string(), index: 1usize, "Tab 2 content" }
            }
        }
    }
}
