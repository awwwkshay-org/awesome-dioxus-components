use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, OptionalBoolControl};
use crate::components::demo::Demo;
use crate::generated::controls::{ContextMenuItemControls, ContextMenuItemDemoState};

/// A browser-style right-click menu: a navigation group with shortcut hints
/// (Forward disabled, as it usually is), then a labeled section — using the
/// grouping/label/separator/shortcut parts the installed `context-menu`
/// item exports. The `ContextMenuItemControls` panel binds to "Back".
#[component]
pub fn ContextMenuPage() -> Element {
    let disabled = use_signal(|| false);
    let open = use_signal(|| None::<bool>);
    let item_state = use_signal(ContextMenuItemDemoState::default);
    rsx! {
        Demo {
            name: "Context Menu",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                OptionalBoolControl { label: "Open state", value: open }
                ContextMenuItemControls { state: item_state }
            },
            components::ui::ContextMenu { disabled: disabled(), open: open,
                components::ui::ContextMenuTrigger {
                    div { class: "flex h-36 w-72 items-center justify-center rounded-md border border-dashed border-border text-sm text-muted-foreground",
                        "Right click here"
                    }
                }
                components::ui::ContextMenuContent { class: "min-w-56",
                    components::ui::ContextMenuGroup {
                        components::ui::ContextMenuItem {
                            value: "back".to_string(),
                            index: 0usize,
                            inset: item_state().inset,
                            variant: item_state().variant,
                            on_select: move |_value| {},
                            "Back"
                            components::ui::ContextMenuShortcut { "⌘[" }
                        }
                        components::ui::ContextMenuItem {
                            value: "forward".to_string(),
                            index: 1usize,
                            disabled: true,
                            on_select: move |_value| {},
                            "Forward"
                            components::ui::ContextMenuShortcut { "⌘]" }
                        }
                        components::ui::ContextMenuItem {
                            value: "reload".to_string(),
                            index: 2usize,
                            on_select: move |_value| {},
                            "Reload"
                            components::ui::ContextMenuShortcut { "⌘R" }
                        }
                    }
                    components::ui::ContextMenuSeparator {}
                    components::ui::ContextMenuLabel { "This Page" }
                    components::ui::ContextMenuGroup {
                        components::ui::ContextMenuItem {
                            value: "save".to_string(),
                            index: 3usize,
                            on_select: move |_value| {},
                            "Save As…"
                            components::ui::ContextMenuShortcut { "⌘S" }
                        }
                        components::ui::ContextMenuItem {
                            value: "print".to_string(),
                            index: 4usize,
                            on_select: move |_value| {},
                            "Print…"
                            components::ui::ContextMenuShortcut { "⌘P" }
                        }
                    }
                    components::ui::ContextMenuSeparator {}
                    components::ui::ContextMenuItem {
                        value: "inspect".to_string(),
                        index: 5usize,
                        on_select: move |_value| {},
                        "Inspect"
                    }
                }
            }
        }
    }
}
