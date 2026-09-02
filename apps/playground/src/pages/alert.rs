use dioxus::prelude::*;

use crate::components;
use crate::components::controls::SelectControl;
use crate::components::demo::Demo;

#[component]
pub fn AlertPage() -> Element {
    let mut variant = use_signal(|| components::ui::AlertVariant::Default);
    rsx! {
        Demo {
            name: "Alert",
            controls: rsx! {
                SelectControl {
                    label: "Variant",
                    value: variant(),
                    options: crate::generated::controls::ALERT_VARIANT_OPTIONS.to_vec(),
                    on_change: move |value| variant.set(value),
                }
            },
            components::ui::Alert { class: "max-w-md", variant: variant(),
                components::ui::AlertTitle { "Heads up!" }
                components::ui::AlertDescription { "You can add components to your app using the CLI." }
            }
        }
    }
}
