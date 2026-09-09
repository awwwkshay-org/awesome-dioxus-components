use adico_primitives::LocalDateExt as _;
use adico_primitives::icons::{ChevronLeft, ChevronRight};
use dioxus::prelude::*;
use time::{Date, Weekday};

use crate::components;
use crate::components::controls::{BoolControl, SelectControl};
use crate::components::demo::Demo;

/// The `CalendarView` subtree shared by both the flat, always-visible
/// calendar and the popover-wrapped instance below -- written once so the
/// two stay in sync.
#[component]
fn CalendarDemoBody(
    selected_date: ReadSignal<Option<Date>>,
    on_date_change: Callback<Option<Date>>,
    view_date: ReadSignal<Date>,
    today: Date,
    on_view_change: Callback<Date>,
    disabled: ReadSignal<bool>,
    first_day_of_week: ReadSignal<Weekday>,
) -> Element {
    rsx! {
        components::ui::Calendar {
            selected_date: selected_date(),
            on_date_change: move |date| on_date_change.call(date),
            view_date: view_date(),
            today,
            on_view_change: move |new_view: Date| on_view_change.call(new_view),
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

#[component]
pub fn CalendarPage() -> Element {
    let mut selected_date = use_signal(|| None::<Date>);
    let today = time::OffsetDateTime::now_local_date();
    let mut view_date = use_signal(move || today);
    let disabled = use_signal(|| false);
    let first_day_of_week = use_signal(|| Weekday::Sunday);
    let mut popover_open = use_signal(|| false);
    let trigger_label = use_memo(move || match selected_date() {
        Some(date) => format!("{date}"),
        None => "Pick a date".to_string(),
    });
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
                BoolControl { label: "Popover open", value: popover_open }
                SelectControl {
                    label: "First day of week",
                    value: first_day_of_week,
                    options: &[("Sunday", Weekday::Sunday), ("Monday", Weekday::Monday)],
                }
            },
            div { class: "flex w-full flex-col items-center gap-4",
                // Trigger + popover: shows the selected date, opens the same
                // calendar composition in a popup, positioned by the shared
                // `Positioner` like every other overlay-family component.
                components::ui::Popover {
                    open: popover_open(),
                    on_open_change: move |value| popover_open.set(value),
                    components::ui::PopoverTrigger { "{trigger_label()}" }
                    // `CalendarView` already owns its own `border bg-popover p-3`
                    // surface (see `registry/ui/calendar.rs`), so the popover
                    // frame is stripped here rather than only its padding --
                    // otherwise two borders and two backgrounds nest, leaving a
                    // visible seam. Matches `DatePickerContent`'s existing
                    // plain-utility pattern (`registry/ui/date_picker.rs`).
                    components::ui::PopoverContent { class: "w-auto border-0 bg-transparent p-0 shadow-none",
                        CalendarDemoBody {
                            selected_date: selected_date(),
                            on_date_change: move |date| selected_date.set(date),
                            view_date: view_date(),
                            today,
                            on_view_change: move |new_view| view_date.set(new_view),
                            disabled: disabled(),
                            first_day_of_week: first_day_of_week(),
                        }
                    }
                }
                // Flat, always-visible calendar: kept so Calendar's own
                // navigation/month/year-select props stay demoable without
                // first opening a popup.
                CalendarDemoBody {
                    selected_date: selected_date(),
                    on_date_change: move |date| selected_date.set(date),
                    view_date: view_date(),
                    today,
                    on_view_change: move |new_view| view_date.set(new_view),
                    disabled: disabled(),
                    first_day_of_week: first_day_of_week(),
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
