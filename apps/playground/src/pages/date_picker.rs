use adico_primitives::icons::{ChevronLeft, ChevronRight};
use dioxus::prelude::*;
use time::Date;

use crate::components;
use crate::components::controls::BoolControl;
use crate::components::demo::Demo;

#[component]
pub fn DatePickerPage() -> Element {
    let mut picked_date = use_signal(|| None::<Date>);
    let disabled = use_signal(|| false);
    let read_only = use_signal(|| false);
    let mut open = use_signal(|| false);
    rsx! {
        Demo {
            name: "DatePicker",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                BoolControl { label: "Read only", value: read_only }
            },
            components::ui::DatePicker {
                selected_date: picked_date(),
                on_value_change: move |date| picked_date.set(date),
                disabled: disabled(),
                read_only: read_only(),
                components::ui::DatePickerPopover {
                    class: "playground-date-picker-popover-root",
                    open: Some(open()),
                    on_open_change: move |value| open.set(value),
                    components::ui::DatePickerInput {
                        components::ui::DatePickerInputValue {}
                        components::ui::DatePickerTrigger {}
                        components::ui::DatePickerContent {
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
                        }
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
    fn date_picker_uses_segment_width_placeholders_once() {
        let mut dom = VirtualDom::new(DatePickerPage);
        dom.rebuild_in_place();
        let html = dioxus::ssr::render(&dom);

        assert!(html.contains("YYYY"));
        assert!(html.contains("MM"));
        assert!(html.contains("DD"));
        assert!(!html.contains("YYYYY"));
        assert!(!html.contains("MMM"));
        assert!(!html.contains("DDD"));
    }
}
