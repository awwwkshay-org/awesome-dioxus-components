use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{
    BubbleContentControls, BubbleContentDemoState, BubbleReactionsControls,
    BubbleReactionsDemoState,
};

#[component]
pub fn BubblePage() -> Element {
    let state = use_signal(BubbleContentDemoState::default);
    let reactions_state = use_signal(BubbleReactionsDemoState::default);
    rsx! {
        Demo {
            name: "Bubble",
            controls: rsx! {
                BubbleContentControls { state }
                BubbleReactionsControls { state: reactions_state }
            },
            div { class: "flex w-full max-w-sm flex-col gap-2",
                components::ui::Bubble { align: state().align,
                    components::ui::BubbleContent {
                        align: state().align,
                        variant: state().variant,
                        "Hey, did you see the latest deploy?"
                    }
                    components::ui::BubbleReactions {
                        align: reactions_state().align,
                        side: reactions_state().side,
                        "👍 2"
                    }
                }
            }
        }
    }
}
