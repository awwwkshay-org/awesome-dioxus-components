use dioxus::prelude::*;

use adico_primitives::icons::{CircleCheck, X};
use components::ui::{
    Attachment, AttachmentAction, AttachmentActions, AttachmentContent, AttachmentDescription,
    AttachmentMedia, AttachmentState, AttachmentTitle, Avatar, AvatarFallback, Bubble,
    BubbleAlign, BubbleContent, Marker, MarkerContent, MarkerIcon, Message, MessageAlign,
    MessageAvatar, MessageContent, MessageHeader, MessageScroller, MessageScrollerButton,
    MessageScrollerContent, MessageScrollerItem, MessageScrollerViewport,
};

#[derive(Clone, PartialEq)]
struct ChatMessage {
    id: usize,
    sender: String,
    text: String,
    align: MessageAlign,
}

fn app() -> Element {
    let mut messages = use_signal(|| {
        vec![
            ChatMessage {
                id: 0,
                sender: "Assistant".to_string(),
                text: "Hi, how can I help?".to_string(),
                align: MessageAlign::Start,
            },
            ChatMessage {
                id: 1,
                sender: "You".to_string(),
                text: "Summarize this PDF.".to_string(),
                align: MessageAlign::End,
            },
        ]
    });
    let mut attachment_state = use_signal(|| AttachmentState::Uploading);
    let mut last_attachment_action = use_signal(String::new);

    let add_message = move |_| {
        let next_id = messages.read().len();
        messages.write().push(ChatMessage {
            id: next_id,
            sender: "Assistant".to_string(),
            text: format!("Reply {next_id}"),
            align: MessageAlign::Start,
        });
    };

    let mark_upload_done = move |_| {
        attachment_state.set(AttachmentState::Done);
    };

    rsx! {
        button { id: "add-message", onclick: add_message, "Add message" }

        MessageScroller {
            MessageScrollerViewport { id: "viewport", style: "height: 240px; overflow-y: auto;",
                MessageScrollerContent { id: "content",
                    for message in messages.read().iter() {
                        MessageScrollerItem { key: "{message.id}",
                            Message {
                                align: message.align,
                                avatar: rsx! {
                                    MessageAvatar { Avatar { AvatarFallback { "{message.sender.chars().next().unwrap_or('?')}" } } }
                                },
                                MessageHeader { "{message.sender}" }
                                MessageContent {
                                    Bubble { align: BubbleAlign::from(message.align),
                                        BubbleContent { align: BubbleAlign::from(message.align), "{message.text}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            MessageScrollerButton { id: "jump-to-bottom", "Jump to latest" }
        }

        Marker {
            MarkerIcon { CircleCheck {} }
            MarkerContent { "Step 1" }
        }

        Attachment { id: "attachment", state: attachment_state(),
            AttachmentMedia {}
            AttachmentContent {
                AttachmentTitle { "report.pdf" }
                AttachmentDescription { "2.4 MB" }
            }
            AttachmentActions {
                AttachmentAction {
                    aria_label: "Mark done",
                    onclick: mark_upload_done,
                    CircleCheck {}
                }
                AttachmentAction {
                    aria_label: "Remove",
                    onclick: move |_| last_attachment_action.set("removed".to_string()),
                    X {}
                }
            }
        }
        p { id: "last-attachment-action", "{last_attachment_action()}" }
    }
}

impl From<MessageAlign> for BubbleAlign {
    fn from(align: MessageAlign) -> Self {
        match align {
            MessageAlign::Start => BubbleAlign::Start,
            MessageAlign::End => BubbleAlign::End,
        }
    }
}

fn main() {
    launch(app);
}

// adico:start
pub mod adico_lib;
pub mod components;
// adico:end
