use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{MessageControls, MessageDemoState};

#[component]
pub fn MessagePage() -> Element {
    let state = use_signal(MessageDemoState::default);
    rsx! {
        Demo {
            name: "Message",
            controls: rsx! {
                MessageControls { state }
            },
            div { class: "w-full max-w-sm",
                components::ui::Message {
                    align: state().align,
                    avatar: rsx! {
                        components::ui::MessageAvatar {
                            components::ui::Avatar { size: components::ui::AvatarSize::Sm,
                                components::ui::AvatarFallback { "AB" }
                            }
                        }
                    },
                    components::ui::MessageHeader { "Alex" }
                    components::ui::MessageContent {
                        components::ui::Bubble {
                            components::ui::BubbleContent { "Hey, did you see the latest deploy?" }
                        }
                    }
                    components::ui::MessageFooter { "Read" }
                }
            }
        }
    }
}
