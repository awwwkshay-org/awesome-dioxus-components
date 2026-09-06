use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{MessageScrollerControls, MessageScrollerDemoState};

/// (sender initials, sent-by-me, text, timestamp)
const CONVERSATION: &[(&str, bool, &str, &str)] = &[
    (
        "AL",
        false,
        "Hey, did you see the latest deploy?",
        "9:41 AM",
    ),
    ("ME", true, "Yeah, just checked — looks clean.", "9:42 AM"),
    ("AL", false, "Carousel drag is smooth now.", "9:42 AM"),
    ("ME", true, "OTP masking works too.", "9:43 AM"),
    ("AL", false, "Nice. Let's ship it.", "9:44 AM"),
    ("ME", true, "On it.", "9:44 AM"),
    (
        "AL",
        false,
        "One more thing — check the nav order.",
        "9:45 AM",
    ),
    ("ME", true, "Already alphabetical 👍", "9:45 AM"),
];

/// A realistic chat transcript: each message renders through the installed
/// `Message`/`MessageContent`/`MessageFooter` parts with an `Avatar` and a
/// timestamp, alternating sent (right) and received (left) — composed
/// inside `MessageScroller` so it actually reads as a scrollable
/// conversation instead of a bare list of "Message N" placeholders. No
/// `MessageHeader`: the avatar's initials already identify the sender, and
/// omitting it lets `Message`'s `items-start` align the avatar with the
/// bubble's own top edge instead of a redundant name line.
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
                class: "h-80 w-full max-w-sm rounded-md border",
                components::ui::MessageScrollerViewport { class: "h-full p-3",
                    components::ui::MessageScrollerContent {
                        for (initials , sent_by_me , text , timestamp) in CONVERSATION.iter().copied() {
                            components::ui::MessageScrollerItem { key: "{initials}-{timestamp}",
                                components::ui::Message {
                                    align: if sent_by_me {
                                        components::ui::MessageAlign::End
                                    } else {
                                        components::ui::MessageAlign::Start
                                    },
                                    avatar: rsx! {
                                        components::ui::MessageAvatar {
                                            components::ui::Avatar { size: components::ui::AvatarSize::Sm,
                                                components::ui::AvatarFallback { "{initials}" }
                                            }
                                        }
                                    },
                                    components::ui::MessageContent {
                                        components::ui::Bubble {
                                            align: if sent_by_me {
                                                components::ui::BubbleAlign::End
                                            } else {
                                                components::ui::BubbleAlign::Start
                                            },
                                            components::ui::BubbleContent { "{text}" }
                                        }
                                    }
                                    components::ui::MessageFooter { "{timestamp}" }
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
