use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, SelectControl};
use crate::components::demo::Demo;

#[component]
pub fn SkeletonPage() -> Element {
    let mut variant = use_signal(|| components::ui::SkeletonVariant::Default);
    let decorative = use_signal(|| true);
    rsx! {
        Demo {
            name: "Skeleton",
            controls: rsx! {
                SelectControl {
                    label: "Shape",
                    value: variant(),
                    options: vec![
                        ("Rectangle", components::ui::SkeletonVariant::Default),
                        ("Circle", components::ui::SkeletonVariant::Circle),
                    ],
                    on_change: move |value| variant.set(value),
                }
                BoolControl { label: "Decorative", value: decorative }
            },
            components::ui::Skeleton {
                variant: variant(),
                decorative: decorative(),
                class: if variant() == components::ui::SkeletonVariant::Circle { "size-16" } else { "h-4 w-40" },
            }
        }
    }
}
