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
                    // No `MessageHeader`: the avatar already identifies the
                    // sender, so a name label here would be redundant. This
                    // also means the bubble is the column's first child, so
                    // `Message`'s `items-start` naturally aligns the avatar
                    // with the bubble's own top edge instead of a name line.
                    avatar: rsx! {
                        components::ui::MessageAvatar {
                            components::ui::Avatar { size: components::ui::AvatarSize::Sm,
                                components::ui::AvatarFallback { "AB" }
                            }
                        }
                    },
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
