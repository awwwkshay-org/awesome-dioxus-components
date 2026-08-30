use adico_primitives::icons::{ChevronLeft, ChevronRight};
use dioxus::prelude::*;
use time::{Date, Weekday};

use crate::components;
use crate::components::controls::{BoolControl, SelectControl};
use crate::components::demo::Demo;

#[component]
pub fn CalendarPage() -> Element {
    let mut selected_date = use_signal(|| None::<Date>);
    let today = time::OffsetDateTime::now_utc().date();
    let mut view_date = use_signal(move || today);
    let disabled = use_signal(|| false);
    let mut first_day_of_week = use_signal(|| Weekday::Sunday);
    // Jump the visible month to a pre-existing selected date (e.g. on first
    // open) rather than always starting on today's month; mirrors the same
    // effect `DatePickerCalendar` already runs internally.
    use_effect(move || {
        if let Some(date) = selected_date() {
            view_date.set(date);
        }
    });
    rsx! {
        Demo {
            name: "Calendar",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                SelectControl {
                    label: "First day of week",
                    value: first_day_of_week(),
                    options: vec![("Sunday", Weekday::Sunday), ("Monday", Weekday::Monday)],
                    on_change: move |next| first_day_of_week.set(next),
                }
            },
            div { class: "flex w-full justify-center",
                components::ui::Calendar {
                    selected_date: selected_date(),
                    on_date_change: move |date| selected_date.set(date),
                    view_date: view_date(),
                    today,
                    on_view_change: move |new_view: Date| view_date.set(new_view),
                    disabled: disabled(),
                    first_day_of_week: first_day_of_week(),
                    components::ui::CalendarView {
                        components::ui::CalendarHeader {
                            components::ui::CalendarNavigation {
                                components::ui::CalendarPreviousMonthButton {
                                    ChevronLeft { class: "size-4", size: 16 }
                                }
                                div { class: "flex flex-1 items-center gap-1",
                                    components::ui::CalendarSelectMonth {
                                        components::ui::CalendarSelectMonthSelect {}
                                        components::ui::CalendarSelectMonthValue {}
                                    }
                                    components::ui::CalendarSelectYear {
                                        components::ui::CalendarSelectYearSelect {}
                                        components::ui::CalendarSelectYearValue {}
                                    }
                                }
                                components::ui::CalendarNextMonthButton {
                                    ChevronRight { class: "size-4", size: 16 }
                                }
                            }
                        }
                        components::ui::CalendarGrid {}
                    }
                }
            }
        }
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn calendar_page_builds_its_primitive_tree() {
        let mut dom = VirtualDom::new(CalendarPage);
        dom.rebuild_in_place();
        let html = dioxus::ssr::render(&dom);
        assert!(html.contains("Calendar"));
        assert!(html.contains("role=\"grid\""));
    }
}
