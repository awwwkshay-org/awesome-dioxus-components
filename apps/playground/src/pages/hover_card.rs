use adico_primitives::ContentAlign;
use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, OptionalBoolControl, SelectControl};
use crate::components::demo::Demo;
use crate::generated::controls::{HoverCardContentControls, HoverCardContentDemoState};

/// A profile-preview hover card — the pattern hover cards exist for —
/// composing the installed `Avatar` (fallback initials; the playground is
/// offline, so no remote image URL).
#[component]
pub fn HoverCardPage() -> Element {
    let open = use_signal(|| None::<bool>);
    let disabled = use_signal(|| false);
    let content_state = use_signal(HoverCardContentDemoState::default);
    let align = use_signal(|| ContentAlign::Center);
    rsx! {
        Demo {
            name: "Hover Card",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                OptionalBoolControl { label: "Open state", value: open }
                SelectControl {
                    label: "Align",
                    value: align,
                    options: &[
                        ("Start", ContentAlign::Start),
                        ("Center", ContentAlign::Center),
                        ("End", ContentAlign::End),
                    ],
                }
                HoverCardContentControls { state: content_state }
            },
            components::ui::HoverCard { open: open, disabled: disabled(),
                components::ui::HoverCardTrigger { class: "font-medium underline underline-offset-4",
                    "@dioxus"
                }
                components::ui::HoverCardContent {
                    force_mount: content_state().force_mount,
                    align: Some(align()),
                    class: "w-80",
                    div { class: "flex gap-4",
                        components::ui::Avatar {
                            components::ui::AvatarFallback { "DX" }
                        }
                        div { class: "space-y-1",
                            h4 { class: "text-sm font-semibold", "@dioxus" }
                            p { class: "text-sm", "Fullstack app framework for web, desktop, and mobile — written in Rust." }
                            div { class: "flex items-center gap-4 pt-1",
                                span { class: "text-xs text-muted-foreground",
                                    span { class: "font-medium text-foreground", "24.1k " }
                                    "Stars"
                                }
                                span { class: "text-xs text-muted-foreground",
                                    span { class: "font-medium text-foreground", "412 " }
                                    "Contributors"
                                }
                            }
                            p { class: "text-xs text-muted-foreground", "Joined December 2019" }
                        }
                    }
                }
            }
        }
    }
}
