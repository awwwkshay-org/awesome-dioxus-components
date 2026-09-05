use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, OptionalBoolControl};
use crate::components::demo::Demo;
use crate::generated::controls::{HoverCardContentControls, HoverCardContentDemoState};

#[component]
pub fn HoverCardPage() -> Element {
    let open = use_signal(|| None::<bool>);
    let disabled = use_signal(|| false);
    let content_state = use_signal(HoverCardContentDemoState::default);
    rsx! {
        Demo {
            name: "HoverCard",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                OptionalBoolControl { label: "Open state", value: open }
                HoverCardContentControls { state: content_state }
            },
            components::ui::HoverCard { open: open, disabled: disabled(),
                components::ui::HoverCardTrigger { "Dioxus" }
                components::ui::HoverCardContent { force_mount: content_state().force_mount, "Hover card content" }
            }
        }
    }
}
