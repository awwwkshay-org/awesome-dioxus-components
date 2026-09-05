use dioxus::prelude::*;

use adico_primitives::icons::X;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{
    AttachmentControls, AttachmentDemoState, AttachmentMediaControls, AttachmentMediaDemoState,
};

#[component]
pub fn AttachmentPage() -> Element {
    let state = use_signal(AttachmentDemoState::default);
    let media_state = use_signal(AttachmentMediaDemoState::default);
    rsx! {
        Demo {
            name: "Attachment",
            controls: rsx! {
                AttachmentControls { state }
                AttachmentMediaControls { state: media_state }
            },
            components::ui::Attachment {
                state: state().state,
                size: state().size,
                orientation: state().orientation,
                class: "w-full max-w-sm",
                components::ui::AttachmentMedia { variant: media_state().variant }
                components::ui::AttachmentContent {
                    components::ui::AttachmentTitle { "quarterly-report.pdf" }
                    components::ui::AttachmentDescription { "2.4 MB" }
                }
                components::ui::AttachmentActions {
                    components::ui::AttachmentAction { aria_label: "Remove attachment",
                        X { class: "size-4" }
                    }
                }
            }
        }
    }
}
