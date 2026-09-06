use adico_primitives::icons::{ChevronLeft, ChevronRight};
use dioxus::prelude::*;
use time::{PrimitiveDateTime, Time};

// `dx serve`/`dx build` compile this project's own `tailwind.css` into
// `assets/tailwind.css`. Linking it matters for more than looks here: the
// clock dial's diameter comes from the `size-56` utility, so a pointer test
// that measures the dial's box needs the stylesheet actually loaded.
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

/// Both time pickers below are bound to this one signal on purpose: the
/// segmented input, the columns view and the clock dial are three different
/// ways to set the *same* value, so a test can drive one and assert the
/// others converge on it.
fn app() -> Element {
    let mut picked = use_signal(|| None::<Time>);
    let mut picked_datetime = use_signal(|| None::<PrimitiveDateTime>);

    let time_readout = picked()
        .map(|t| format!("{:02}:{:02}", t.hour(), t.minute()))
        .unwrap_or_else(|| "none".to_string());
    let datetime_readout = picked_datetime()
        .map(|dt| {
            format!(
                "{}T{:02}:{:02}",
                dt.date(),
                dt.time().hour(),
                dt.time().minute()
            )
        })
        .unwrap_or_else(|| "none".to_string());

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        // Deliberately compact and laid out in one row: each picker's popup
        // is `position: fixed` and opens *below* its trigger, so a tall
        // stacked page pushes the lower popups past the bottom of a default
        // test viewport, where Playwright cannot scroll to them (a fixed
        // element off-screen is not scrollable into view).
        main { class: "flex flex-col gap-2 p-4",
            div { class: "flex gap-6",
                p { id: "time-value", "{time_readout}" }
                p { id: "datetime-value", "{datetime_readout}" }
            }
            div { class: "flex flex-row items-start gap-8",

            section { id: "digital",
                h2 { "Digital" }
                components::ui::TimePicker {
                    selected_time: picked(),
                    on_value_change: move |t| picked.set(t),
                    view: components::ui::TimePickerView::Digital,
                    components::ui::TimePickerPopover { open: None,
                        components::ui::TimePickerInput {
                            components::ui::TimePickerInputValue {}
                            components::ui::TimePickerTrigger { compact: true }
                            components::ui::TimePickerContent { components::ui::TimePickerBody {} }
                        }
                    }
                }
            }

            section { id: "analog",
                h2 { "Analog" }
                components::ui::TimePicker {
                    selected_time: picked(),
                    on_value_change: move |t| picked.set(t),
                    view: components::ui::TimePickerView::Analog,
                    components::ui::TimePickerPopover { open: None,
                        components::ui::TimePickerInput {
                            components::ui::TimePickerInputValue {}
                            components::ui::TimePickerTrigger { compact: true }
                            components::ui::TimePickerContent { components::ui::TimePickerBody {} }
                        }
                    }
                }
            }

            section { id: "datetime",
                h2 { "DateTime" }
                components::ui::DateTimePicker {
                    selected_datetime: picked_datetime(),
                    on_value_change: move |v| picked_datetime.set(v),
                    components::ui::DateTimePickerPopover { open: None,
                        components::ui::DateTimePickerTrigger {}
                        components::ui::DateTimePickerContent {
                            components::ui::DatePickerCalendar {
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
                            div { class: "flex flex-col gap-2",
                                components::ui::TimePickerInput { components::ui::TimePickerInputValue {} }
                                components::ui::TimePickerBody {}
                            }
                        }
                    }
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
