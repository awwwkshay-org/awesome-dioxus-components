use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, SelectControl};
use crate::components::demo::Demo;

#[component]
pub fn ItemPage() -> Element {
    let mut variant = use_signal(|| components::ui::ItemVariant::Default);
    let disabled = use_signal(|| false);
    rsx! {
        Demo {
            name: "Item",
            controls: rsx! {
                SelectControl {
                    label: "Variant",
                    value: variant(),
                    options: crate::generated::controls::ITEM_VARIANT_OPTIONS.to_vec(),
                    on_change: move |value| variant.set(value),
                }
                BoolControl { label: "Disabled", value: disabled }
            },
            components::ui::ItemGroup {
                components::ui::Item { variant: variant(), disabled: disabled(), class: "w-full max-w-md",
                    components::ui::ItemContent {
                        components::ui::ItemTitle { "Row title" }
                        components::ui::ItemDescription { "Row description" }
                    }
                    components::ui::ItemActions {
                        components::ui::Badge { "Active" }
                    }
                }
            }
        }
    }
}
