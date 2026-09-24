use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{CommandDialogControls, CommandDialogDemoState};

#[component]
pub fn CommandPage() -> Element {
    // `CommandDialog`'s own real defaults are `title: "Command Palette"` and
    // `description: "Search for a command to run..."`; the generator's
    // fixed-default convention always starts a `String` field empty (design.md's
    // D2), so this overrides the initial demo values to match the real
    // component's own defaults rather than silently changing what this page
    // shows on first load.
    let dialog_state = use_signal(|| CommandDialogDemoState {
        title: "Command Palette".to_string(),
        description: "Search for a command to run...".to_string(),
        ..Default::default()
    });
    rsx! {
        Demo {
            name: "Command",
            controls: rsx! {
                p { class: "self-end pb-2 text-sm text-muted-foreground", "Type to filter, ArrowUp/ArrowDown to move, Enter to select." }
                CommandDialogControls { state: dialog_state }
            },
            div { class: "flex flex-col gap-4",
                components::ui::CommandDialog {
                    open: dialog_state().open,
                    title: dialog_state().title.clone(),
                    description: dialog_state().description.clone(),
                    show_close_button: dialog_state().show_close_button,
                    components::ui::CommandInput { placeholder: "Search...".to_string() }
                    components::ui::CommandList {
                        components::ui::CommandEmpty { "No results found." }
                        components::ui::CommandGroup { heading: "Suggestions".to_string(),
                            components::ui::CommandItem::<String> {
                                index: 0usize,
                                value: "profile-dialog".to_string(),
                                on_select: move |_| {},
                                "Profile"
                            }
                        }
                    }
                }
            div { class: "w-72 rounded-lg border shadow-md",
                components::ui::Command {
                    components::ui::CommandInput { placeholder: "Search...".to_string() }
                    components::ui::CommandList {
                        components::ui::CommandEmpty { "No results found." }
                        components::ui::CommandGroup { heading: "Suggestions".to_string(),
                            components::ui::CommandItem::<String> {
                                index: 0usize,
                                value: "profile".to_string(),
                                on_select: move |_| {},
                                "Profile"
                            }
                            components::ui::CommandItem::<String> {
                                index: 1usize,
                                value: "billing".to_string(),
                                on_select: move |_| {},
                                "Billing"
                            }
                            components::ui::CommandItem::<String> {
                                index: 2usize,
                                value: "settings".to_string(),
                                on_select: move |_| {},
                                "Settings"
                                components::ui::CommandShortcut { "⌘S" }
                            }
                        }
                        components::ui::CommandSeparator {}
                        components::ui::CommandGroup { heading: "Danger".to_string(),
                            components::ui::CommandItem::<String> {
                                index: 3usize,
                                value: "logout".to_string(),
                                on_select: move |_| {},
                                "Log out"
                            }
                        }
                    }
                }
            }
            }
        }
    }
}
