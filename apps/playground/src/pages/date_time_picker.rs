use adico_primitives::icons::{ChevronLeft, ChevronRight};
use dioxus::prelude::*;
use time::PrimitiveDateTime;

use crate::components;
use crate::components::controls::{BoolControl, SelectControl};
use crate::components::demo::Demo;
use crate::components::ui::TimePickerView;
use crate::generated::controls::{DateTimePickerPopoverControls, DateTimePickerPopoverDemoState};

const VIEW_OPTIONS: &[(&str, TimePickerView)] = &[
    ("Digital", TimePickerView::Digital),
    ("Analog", TimePickerView::Analog),
];

#[component]
pub fn DateTimePickerPage() -> Element {
    let mut picked = use_signal(|| None::<PrimitiveDateTime>);
    let disabled = use_signal(|| false);
    let read_only = use_signal(|| false);
    let is_12h = use_signal(|| false);
    let show_seconds = use_signal(|| false);
    let view = use_signal(|| TimePickerView::Digital);
    let popover_state = use_signal(DateTimePickerPopoverDemoState::default);
    // The popup surface is `position: fixed` with an otherwise-auto height,
    // so `TimePickerColumns`'s `fill_height` (a CSS `height: 100%` + flex
    // stretch) has no definite ancestor height to resolve against here --
    // confirmed empirically to render unclipped, full-content-height columns
    // instead of matching the calendar, rather than guessed from CSS theory
    // alone. Measuring the calendar's real rendered height and applying it
    // as an explicit inline style is the same technique `TimePickerClock`
    // already uses for its own dial-face rect. Uses `get_scroll_size()`
    // (scrollHeight), not `get_client_rect()` (getBoundingClientRect) --
    // the popover's own entrance animation (`data-[state=open]:zoom-in-95`)
    // is a CSS `transform: scale()`, which `get_client_rect()` reports
    // mid-transition (confirmed empirically: it returned 95% of the
    // settled height), while `scrollHeight` reflects the post-layout size
    // regardless of that paint-time transform. Measured once on mount, not
    // re-measured on month navigation -- a 5-week vs. 6-week month can leave
    // a small height mismatch, a documented trade-off rather than a silent
    // one.
    let mut calendar_height = use_signal(|| None::<f64>);
    let time_panel_style = use_memo(move || {
        calendar_height()
            .map(|height| format!("height: {height}px;"))
            .unwrap_or_default()
    });
    rsx! {
        Demo {
            name: "DateTimePicker",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                BoolControl { label: "Read only", value: read_only }
                BoolControl { label: "12-hour", value: is_12h }
                BoolControl { label: "Show seconds", value: show_seconds }
                SelectControl { label: "View", value: view, options: VIEW_OPTIONS }
                DateTimePickerPopoverControls { state: popover_state }
            },
            components::ui::DateTimePicker {
                selected_datetime: picked(),
                on_value_change: move |value| picked.set(value),
                disabled: disabled(),
                read_only: read_only(),
                is_12h: is_12h(),
                show_seconds: show_seconds(),
                view: view(),
                components::ui::DateTimePickerPopover {
                    open: popover_state().open,
                    default_open: popover_state().default_open,
                    components::ui::DateTimePickerTrigger {}
                    components::ui::DateTimePickerContent {
                        div {
                            // `self-start` decouples this measurement wrapper from the
                            // row's default `align-items: stretch` -- without it, this
                            // div gets stretched to match its sibling's height instead
                            // of reporting the calendar's own true natural height,
                            // poisoning the very measurement `time_panel_style` relies
                            // on (confirmed empirically: it measured 1757px instead of
                            // the calendar's real ~330px, because the sibling's own
                            // unclipped column content was that tall).
                            class: "self-start",
                            onmounted: move |event| async move {
                                if let Ok(size) = event.data().get_scroll_size().await {
                                    calendar_height.set(Some(size.height));
                                }
                            },
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
                        div {
                            // No divider border of its own: `TimePickerBody`'s
                            // view part now supplies its own card surface (like
                            // `CalendarView` does), so the two panels read as
                            // two cards rather than needing a hand-drawn rule.
                            class: "flex min-h-0 flex-col gap-2",
                            style: "{time_panel_style}",
                            components::ui::TimePickerInput { components::ui::TimePickerInputValue {} }
                            components::ui::TimePickerBody {
                                class: "flex-1 min-h-0",
                                fill_height: true,
                            }
                        }
                    }
                }
            }
        }
    }
}
