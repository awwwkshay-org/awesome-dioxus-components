use dioxus::prelude::*;

fn app() -> Element {
    let mut popover_open = use_signal(|| false);

    rsx! {
        components::ui::Tooltip {
            components::ui::TooltipTrigger { "Hover me" }
            components::ui::TooltipContent { "Tooltip content" }
        }

        components::ui::Popover {
            open: popover_open(),
            on_open_change: move |value| popover_open.set(value),
            components::ui::PopoverTrigger { "Open popover" }
            components::ui::PopoverContent { "Popover content" }
        }

        components::ui::HoverCard {
            components::ui::HoverCardTrigger { "Dioxus" }
            components::ui::HoverCardContent { "Hover card content" }
        }

        components::ui::DropdownMenu {
            components::ui::DropdownMenuTrigger { "Open menu" }
            components::ui::DropdownMenuContent {
                components::ui::DropdownMenuItem::<String> {
                    value: "edit".to_string(),
                    index: 0usize,
                    on_select: move |_value| {},
                    "Edit"
                }
            }
        }

        components::ui::ContextMenu {
            components::ui::ContextMenuTrigger { "Right click here" }
            components::ui::ContextMenuContent {
                components::ui::ContextMenuItem {
                    value: "edit".to_string(),
                    index: 0usize,
                    on_select: move |_value| {},
                    "Edit"
                }
            }
        }

        components::ui::Menubar {
            components::ui::MenubarMenu { index: 0usize,
                components::ui::MenubarTrigger { "File" }
                components::ui::MenubarContent {
                    components::ui::MenubarItem {
                        index: 0usize,
                        value: "new".to_string(),
                        on_select: move |_value| {},
                        "New"
                    }
                }
            }
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
