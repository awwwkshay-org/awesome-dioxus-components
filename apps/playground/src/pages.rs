//! One page per currently migrated registry item. Each renders the same
//! working demo composition already verified against the real `adico` CLI
//! install, just split into its own route instead of one long scroll.

use adico_primitives::ContentAlign;
use adico_primitives::popover::{
    PopoverContent as PrimitivePopoverContent, PopoverTrigger as PrimitivePopoverTrigger,
};
use dioxus::prelude::*;
use time::Date;

use crate::components;
use crate::demo::Demo;

#[component]
pub fn ButtonPage() -> Element {
    rsx! {
        Demo { name: "Button",
            components::ui::Button { "Source-owned Button" }
        }
    }
}

#[component]
pub fn BadgePage() -> Element {
    rsx! {
        Demo { name: "Badge",
            components::ui::Badge { "New" }
        }
    }
}

#[component]
pub fn CardPage() -> Element {
    rsx! {
        Demo { name: "Card",
            components::ui::Card {
                components::ui::CardHeader {
                    components::ui::CardTitle { "Card title" }
                    components::ui::CardDescription { "Supporting description text." }
                }
                components::ui::CardContent { "Card body content." }
                components::ui::CardFooter { "Footer" }
            }
        }
    }
}

#[component]
pub fn InputPage() -> Element {
    rsx! {
        Demo { name: "Input",
            components::ui::Input { placeholder: "Type here" }
        }
    }
}

#[component]
pub fn TextareaPage() -> Element {
    rsx! {
        Demo { name: "Textarea",
            components::ui::Textarea { placeholder: "Longer text" }
        }
    }
}

#[component]
pub fn SkeletonPage() -> Element {
    rsx! {
        Demo { name: "Skeleton",
            components::ui::Skeleton { class: "h-4 w-40" }
        }
    }
}

#[component]
pub fn ItemPage() -> Element {
    rsx! {
        Demo { name: "Item",
            components::ui::ItemGroup {
                components::ui::Item {
                    components::ui::ItemContent {
                        components::ui::ItemTitle { "Row title" }
                        components::ui::ItemDescription { "Row description" }
                    }
                    components::ui::ItemActions {
                        components::ui::Badge { "Active" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn PaginationPage() -> Element {
    rsx! {
        Demo { name: "Pagination",
            components::ui::Pagination {
                components::ui::PaginationContent {
                    components::ui::PaginationItem { components::ui::PaginationPrevious {} }
                    components::ui::PaginationItem {
                        components::ui::PaginationLink { is_active: true, "1" }
                    }
                    components::ui::PaginationItem { components::ui::PaginationEllipsis {} }
                    components::ui::PaginationItem { components::ui::PaginationNext {} }
                }
            }
        }
    }
}

#[component]
pub fn DialogPage() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        Demo { name: "Dialog",
            components::ui::Dialog {
                open: open(),
                on_open_change: move |value| open.set(value),
                components::ui::DialogTrigger { "Open dialog" }
                components::ui::DialogOverlay {}
                components::ui::DialogContent {
                    components::ui::DialogHeader {
                        components::ui::DialogTitle { "Installed through adico" }
                        components::ui::DialogDescription { "This Dialog source belongs to this app." }
                    }
                }
            }
        }
    }
}

#[component]
pub fn SheetPage() -> Element {
    rsx! {
        Demo { name: "Sheet",
            components::ui::Sheet {
                components::ui::SheetTrigger { "Open sheet" }
                components::ui::SheetOverlay {}
                components::ui::SheetContent {
                    components::ui::SheetHeader {
                        components::ui::SheetTitle { "Settings" }
                        components::ui::SheetDescription { "Adjust your preferences." }
                    }
                    components::ui::SheetFooter { "Done" }
                }
            }
        }
    }
}

#[component]
pub fn SelectPage() -> Element {
    rsx! {
        Demo { name: "Select",
            components::ui::Select::<String> {
                components::ui::SelectTrigger {
                    aria_label: "Choose a fruit",
                    components::ui::SelectValue { placeholder: "Choose a fruit" }
                }
                components::ui::SelectList { aria_label: "Fruit options",
                    components::ui::SelectOption::<String> {
                        index: 0usize,
                        value: "apple",
                        text_value: "Apple",
                        "Apple"
                    }
                    components::ui::SelectOption::<String> {
                        index: 1usize,
                        value: "banana",
                        text_value: "Banana",
                        "Banana"
                    }
                }
            }
        }
    }
}

#[component]
pub fn ComboboxPage() -> Element {
    rsx! {
        Demo { name: "Combobox",
            components::ui::Combobox::<String> {
                components::ui::ComboboxInput { placeholder: "Search fruit" }
                components::ui::ComboboxList {
                    components::ui::ComboboxOption::<String> { value: "Apple".to_string(), index: 0usize, "Apple" }
                    components::ui::ComboboxOption::<String> { value: "Banana".to_string(), index: 1usize, "Banana" }
                    components::ui::ComboboxEmpty { "No results" }
                }
            }
        }
    }
}

#[component]
pub fn TooltipPage() -> Element {
    rsx! {
        Demo { name: "Tooltip",
            components::ui::Tooltip {
                components::ui::TooltipTrigger { "Hover me" }
                components::ui::TooltipContent { "Tooltip content" }
            }
        }
    }
}

#[component]
pub fn PopoverPage() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        Demo { name: "Popover",
            components::ui::Popover {
                open: open(),
                on_open_change: move |value| open.set(value),
                components::ui::PopoverTrigger { "Open popover" }
                components::ui::PopoverContent { "Popover content" }
            }
        }
    }
}

#[component]
pub fn HoverCardPage() -> Element {
    rsx! {
        Demo { name: "HoverCard",
            components::ui::HoverCard {
                components::ui::HoverCardTrigger { "Dioxus" }
                components::ui::HoverCardContent { "Hover card content" }
            }
        }
    }
}

#[component]
pub fn DropdownMenuPage() -> Element {
    rsx! {
        Demo { name: "DropdownMenu",
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
        }
    }
}

#[component]
pub fn ContextMenuPage() -> Element {
    rsx! {
        Demo { name: "ContextMenu",
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
        }
    }
}

#[component]
pub fn MenubarPage() -> Element {
    rsx! {
        Demo { name: "Menubar",
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
}

#[component]
pub fn CalendarPage() -> Element {
    let mut selected_date = use_signal(|| None::<Date>);
    let today = time::OffsetDateTime::now_utc().date();
    let mut view_date = use_signal(move || today);
    rsx! {
        Demo { name: "Calendar",
            components::ui::Calendar {
                selected_date: selected_date(),
                on_date_change: move |date| selected_date.set(date),
                view_date: view_date(),
                today,
                on_view_change: move |new_view: Date| view_date.set(new_view),
                components::ui::CalendarView {
                    components::ui::CalendarHeader {
                        components::ui::CalendarNavigation {
                            components::ui::CalendarPreviousMonthButton { "<" }
                            components::ui::CalendarMonthTitle {}
                            components::ui::CalendarNextMonthButton { ">" }
                        }
                    }
                    components::ui::CalendarGrid {}
                }
            }
        }
    }
}

#[component]
pub fn DatePickerPage() -> Element {
    let mut picked_date = use_signal(|| None::<Date>);
    rsx! {
        Demo { name: "DatePicker",
            components::ui::DatePicker {
                selected_date: picked_date(),
                on_value_change: move |date| picked_date.set(date),
                components::ui::DatePickerPopover {
                    components::ui::DatePickerInput {
                        PrimitivePopoverTrigger { "Select date" }
                        PrimitivePopoverContent {
                            align: ContentAlign::End,
                            components::ui::DatePickerCalendar {
                                calendar: components::ui::Calendar,
                                components::ui::CalendarView {
                                    components::ui::CalendarHeader {
                                        components::ui::CalendarNavigation {
                                            components::ui::CalendarPreviousMonthButton { "<" }
                                            components::ui::CalendarMonthTitle {}
                                            components::ui::CalendarNextMonthButton { ">" }
                                        }
                                    }
                                    components::ui::CalendarGrid {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn SidebarPage() -> Element {
    rsx! {
        Demo { name: "Sidebar",
            components::ui::SidebarProvider { class: "rounded-lg border",
                components::ui::Sidebar {
                    components::ui::SidebarHeader { "My App" }
                    components::ui::SidebarContent {
                        components::ui::SidebarGroup {
                            components::ui::SidebarGroupLabel { "Section" }
                            components::ui::SidebarGroupContent {
                                components::ui::SidebarMenu {
                                    components::ui::SidebarMenuItem {
                                        components::ui::SidebarMenuButton { "Overview" }
                                    }
                                    components::ui::SidebarMenuItem {
                                        components::ui::SidebarMenuButton { is_active: true, "Settings" }
                                    }
                                }
                            }
                        }
                        components::ui::SidebarSeparator {}
                    }
                    components::ui::SidebarFooter { "v1.0" }
                    components::ui::SidebarRail {}
                }
                components::ui::SidebarInset {
                    components::ui::SidebarTrigger { "☰" }
                    " Main content"
                }
            }
        }
    }
}
