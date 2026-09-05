use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{
    ItemControls, ItemDemoState, ItemMediaControls, ItemMediaDemoState,
};

#[component]
pub fn ItemPage() -> Element {
    let state = use_signal(ItemDemoState::default);
    let media_state = use_signal(ItemMediaDemoState::default);
    rsx! {
        Demo {
            name: "Item",
            controls: rsx! {
                ItemControls { state }
                ItemMediaControls { state: media_state }
            },
            components::ui::ItemGroup {
                components::ui::Item {
                    variant: state().variant,
                    size: state().size,
                    disabled: state().disabled,
                    class: "w-full max-w-md",
                    components::ui::ItemMedia { variant: media_state().variant, "📄" }
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
