use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn CommandPage() -> Element {
    rsx! {
        Demo {
            name: "Command",
            controls: rsx! {
                p { class: "self-end pb-2 text-sm text-muted-foreground", "Type to filter, ArrowUp/ArrowDown to move, Enter to select." }
            },
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
