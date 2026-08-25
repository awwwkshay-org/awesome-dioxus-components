use adico_primitives::ContentAlign;
use adico_primitives::popover::{
    PopoverContent as PrimitivePopoverContent, PopoverTrigger as PrimitivePopoverTrigger,
};
use dioxus::prelude::*;
use time::Date;

fn app() -> Element {
    let mut selected_date = use_signal(|| None::<Date>);
    let today = time::OffsetDateTime::now_utc().date();
    let mut view_date = use_signal(move || today);
    let mut picked_date = use_signal(|| None::<Date>);

    rsx! {
        components::ui::Combobox::<String> {
            components::ui::ComboboxInput { placeholder: "Search fruit" }
            components::ui::ComboboxList {
                components::ui::ComboboxOption::<String> { value: "Apple".to_string(), index: 0usize, "Apple" }
                components::ui::ComboboxOption::<String> { value: "Banana".to_string(), index: 1usize, "Banana" }
                components::ui::ComboboxEmpty { "No results" }
            }
        }

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

        div {
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

        components::ui::SidebarProvider {
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
                "Main content"
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
