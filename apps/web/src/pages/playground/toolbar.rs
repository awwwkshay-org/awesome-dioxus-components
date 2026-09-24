use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{
    ToolbarButtonControls, ToolbarButtonDemoState, ToolbarSeparatorControls,
    ToolbarSeparatorDemoState,
};

#[component]
pub fn ToolbarPage() -> Element {
    let mut active = use_signal(|| None::<&'static str>);
    let button_state = use_signal(ToolbarButtonDemoState::default);
    let separator_state = use_signal(|| ToolbarSeparatorDemoState {
        horizontal: false,
        decorative: true,
    });
    rsx! {
        Demo {
            name: "Toolbar",
            controls: rsx! {
                ToolbarButtonControls { state: button_state }
                ToolbarSeparatorControls { state: separator_state }
            },
            div { class: "flex flex-col gap-2",
                components::ui::Toolbar { aria_label: "Text formatting",
                    components::ui::ToolbarButton {
                        index: 0usize,
                        loading: button_state().loading,
                        on_select: move |_| active.set(Some("Bold")),
                        "Bold"
                    }
                    components::ui::ToolbarButton {
                        index: 1usize,
                        loading: button_state().loading,
                        on_select: move |_| active.set(Some("Italic")),
                        "Italic"
                    }
                    components::ui::ToolbarSeparator {
                        horizontal: separator_state().horizontal,
                        decorative: separator_state().decorative,
                    }
                    components::ui::ToolbarButton {
                        index: 2usize,
                        loading: button_state().loading,
                        on_select: move |_| active.set(Some("Underline")),
                        "Underline"
                    }
                }
                p { class: "text-sm text-muted-foreground",
                    "Last action: {active().unwrap_or(\"none\")}"
                }
            }
        }
    }
}
