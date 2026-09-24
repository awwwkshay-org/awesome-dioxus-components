use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{EmptyMediaControls, EmptyMediaDemoState};

#[component]
pub fn EmptyPage() -> Element {
    let state = use_signal(|| EmptyMediaDemoState {
        variant: components::ui::EmptyMediaVariant::Icon,
    });
    rsx! {
        Demo {
            name: "Empty",
            controls: rsx! {
                EmptyMediaControls { state }
            },
            components::ui::Empty { class: "border max-w-md",
                components::ui::EmptyHeader {
                    components::ui::EmptyMedia { variant: state().variant, "📭" }
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
