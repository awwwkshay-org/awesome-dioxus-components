use dioxus::prelude::*;

use crate::components;
use crate::components::controls::BoolControl;
use crate::components::demo::Demo;
use crate::generated::controls::{MenubarItemControls, MenubarItemDemoState};

/// A desktop-style File / Edit / View menubar with grouped items, separator
/// dividers, and shortcut hints — using every part the installed `menubar`
/// item exports. `MenubarItem` indices restart per menu. The
/// `MenubarItemControls` panel binds to File → New.
#[component]
pub fn MenubarPage() -> Element {
    let disabled = use_signal(|| false);
    let item_state = use_signal(|| MenubarItemDemoState {
        value: "new".to_string(),
        ..Default::default()
    });
    rsx! {
        Demo {
            name: "Menubar",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                MenubarItemControls { state: item_state }
            },
            components::ui::Menubar { disabled: disabled(),
                components::ui::MenubarMenu { index: 0usize,
                    components::ui::MenubarTrigger { "File" }
                    components::ui::MenubarContent {
                        components::ui::MenubarGroup {
                            components::ui::MenubarItem {
                                index: 0usize,
                                value: item_state().value,
                                inset: item_state().inset,
                                variant: item_state().variant,
                                on_select: move |_value| {},
                                "New"
                                components::ui::MenubarShortcut { "⌘N" }
                            }
                            components::ui::MenubarItem {
                                index: 1usize,
                                value: "open".to_string(),
                                on_select: move |_value| {},
                                "Open…"
                                components::ui::MenubarShortcut { "⌘O" }
                            }
                        }
                        components::ui::MenubarSeparator {}
                        components::ui::MenubarItem {
                            index: 2usize,
                            value: "save".to_string(),
                            on_select: move |_value| {},
                            "Save"
                            components::ui::MenubarShortcut { "⌘S" }
                        }
                        components::ui::MenubarSeparator {}
                        components::ui::MenubarItem {
                            index: 3usize,
                            value: "quit".to_string(),
                            on_select: move |_value| {},
                            "Quit"
                            components::ui::MenubarShortcut { "⌘Q" }
                        }
                    }
                }
                components::ui::MenubarMenu { index: 1usize,
                    components::ui::MenubarTrigger { "Edit" }
                    components::ui::MenubarContent {
                        components::ui::MenubarGroup {
                            components::ui::MenubarItem {
                                index: 0usize,
                                value: "undo".to_string(),
                                on_select: move |_value| {},
                                "Undo"
                                components::ui::MenubarShortcut { "⌘Z" }
                            }
                            components::ui::MenubarItem {
                                index: 1usize,
                                value: "redo".to_string(),
                                on_select: move |_value| {},
                                "Redo"
                                components::ui::MenubarShortcut { "⇧⌘Z" }
                            }
                        }
                        components::ui::MenubarSeparator {}
                        components::ui::MenubarLabel { "Clipboard" }
                        components::ui::MenubarGroup {
                            components::ui::MenubarItem {
                                index: 2usize,
                                value: "cut".to_string(),
                                on_select: move |_value| {},
                                "Cut"
                                components::ui::MenubarShortcut { "⌘X" }
                            }
                            components::ui::MenubarItem {
                                index: 3usize,
                                value: "copy".to_string(),
                                on_select: move |_value| {},
                                "Copy"
                                components::ui::MenubarShortcut { "⌘C" }
                            }
                            components::ui::MenubarItem {
                                index: 4usize,
                                value: "paste".to_string(),
                                on_select: move |_value| {},
                                "Paste"
                                components::ui::MenubarShortcut { "⌘V" }
                            }
                        }
                    }
                }
                components::ui::MenubarMenu { index: 2usize,
                    components::ui::MenubarTrigger { "View" }
                    components::ui::MenubarContent {
                        components::ui::MenubarItem {
                            index: 0usize,
                            value: "zoom-in".to_string(),
                            on_select: move |_value| {},
                            "Zoom In"
                            components::ui::MenubarShortcut { "⌘+" }
                        }
                        components::ui::MenubarItem {
                            index: 1usize,
                            value: "zoom-out".to_string(),
                            on_select: move |_value| {},
                            "Zoom Out"
                            components::ui::MenubarShortcut { "⌘-" }
                        }
                        components::ui::MenubarSeparator {}
                        components::ui::MenubarItem {
                            index: 2usize,
                            value: "fullscreen".to_string(),
                            on_select: move |_value| {},
                            "Enter Full Screen"
                        }
                    }
                }
            }
        }
    }
}
