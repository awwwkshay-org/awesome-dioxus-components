use dioxus::prelude::*;

use crate::components;
use crate::components::controls::SelectControl;
use crate::components::demo::Demo;

#[component]
pub fn EmptyPage() -> Element {
    let mut variant = use_signal(|| components::ui::EmptyMediaVariant::Icon);
    rsx! {
        Demo {
            name: "Empty",
            controls: rsx! {
                SelectControl {
                    label: "Media",
                    value: variant(),
                    options: crate::generated::controls::EMPTY_MEDIA_VARIANT_OPTIONS.to_vec(),
                    on_change: move |value| variant.set(value),
                }
            },
            components::ui::Empty { class: "border max-w-md",
                components::ui::EmptyHeader {
                    components::ui::EmptyMedia { variant: variant(), "📭" }
                    components::ui::EmptyTitle { "No results found" }
                    components::ui::EmptyDescription { "Try adjusting your search or filters." }
                }
                components::ui::EmptyContent {
                    components::ui::Button { "Clear filters" }
                }
            }
        }
    }
}
