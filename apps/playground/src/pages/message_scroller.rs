use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{MessageScrollerControls, MessageScrollerDemoState};

#[component]
pub fn MessageScrollerPage() -> Element {
    let state = use_signal(|| MessageScrollerDemoState {
        bottom_threshold: 48.0,
    });
    rsx! {
        Demo {
            name: "MessageScroller",
            controls: rsx! {
                MessageScrollerControls { state }
            },
            components::ui::MessageScroller {
                bottom_threshold: state().bottom_threshold,
                class: "h-64 w-full max-w-sm rounded-md border",
                components::ui::MessageScrollerViewport { class: "h-full",
                    components::ui::MessageScrollerContent {
                        for index in 0..8usize {
                            components::ui::MessageScrollerItem { key: "{index}",
                                components::ui::Bubble {
                                    components::ui::BubbleContent { "Message {index + 1}" }
                                }
                            }
                        }
                    }
                }
                components::ui::MessageScrollerButton { "Jump to latest" }
            }
        }
    }
}
