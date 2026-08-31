//! Black-box tests for `adico_primitives::calendar`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/calendar.rs`. Also
//! carries the module's 7 previously-inline `#[cfg(test)] mod tests` tests, moved here verbatim
//! (their subject functions/types were widened to `pub` for this purpose only).

use adico_primitives::calendar::{
    Calendar, CalendarDay, CalendarGrid, CalendarHeader, CalendarMonthTitle, CalendarNavigation,
    CalendarNextMonthButton, CalendarPreviousMonthButton, CalendarView, DateRange, RangeCalendar,
    WeekdaySet, calendar_grid_weeks, days_since, next_month, previous_month,
};
use dioxus::prelude::*;
use time::{Month, Weekday, macros::date};

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn test_weekday_set() {
    let mut weekdays = WeekdaySet::single(Weekday::Monday);
    assert!(weekdays.contains(Weekday::Monday));
    assert!(!weekdays.contains(Weekday::Tuesday));

    assert!(weekdays.remove(Weekday::Monday));
    assert!(!weekdays.contains(Weekday::Monday));
    assert!(!weekdays.remove(Weekday::Monday));

    let all_days = WeekdaySet(0b111_1111);
    let empty_set = WeekdaySet(0b000_0000);
    let single_set = WeekdaySet::single(Weekday::Friday);
    let part_size_set = WeekdaySet(0b010_1010);

    let days: Vec<_> = all_days.iter(Weekday::Sunday).collect();
    assert_eq!(days.len(), 7);
    assert_eq!(days[0], Weekday::Sunday);

    let mut iter = all_days.iter(Weekday::Wednesday);
    assert_eq!(iter.next(), Some(Weekday::Wednesday));
    assert_eq!(iter.next(), Some(Weekday::Thursday));

    assert_eq!(empty_set.first(), None);
    assert_eq!(single_set.first(), Some(Weekday::Friday));
    assert_eq!(part_size_set.first(), Some(Weekday::Tuesday));
    assert_eq!(all_days.first(), Some(Weekday::Monday));

    assert!(empty_set.is_empty());
    assert!(!part_size_set.is_empty());
    assert!(!single_set.is_empty());
    assert!(!all_days.is_empty());
}

#[test]
fn test_days_since() {
    let date = date!(2024 - 01 - 01); // Monday
    assert_eq!(days_since(date, Weekday::Monday), 0);
    assert_eq!(days_since(date, Weekday::Sunday), 1);
    assert_eq!(days_since(date, Weekday::Tuesday), 6);
}

#[test]
fn test_month_navigation() {
    let date = date!(2024 - 01 - 15);

    let next = next_month(date);
    assert!(next.is_some());
    assert_eq!(next.unwrap().month(), Month::February);
    assert_eq!(next.unwrap().year(), 2024);
    assert_eq!(next.unwrap().day(), 15);

    let prev = previous_month(date);
    assert!(prev.is_some());
    assert_eq!(prev.unwrap().month(), Month::December);
    assert_eq!(prev.unwrap().year(), 2023);
    assert_eq!(prev.unwrap().day(), 15);
}

#[test]
fn test_calendar_grid_weeks() {
    for (month, first_day) in [
        (date!(2021 - 02 - 15), Weekday::Monday),
        (date!(2024 - 05 - 15), Weekday::Sunday),
        (date!(2018 - 12 - 15), Weekday::Sunday),
    ] {
        let grid = calendar_grid_weeks(month, first_day);
        assert_eq!(grid.len(), 6, "every month renders six stable weeks");
        assert!(grid.iter().all(|week| week.len() == 7));
        assert_eq!(grid.iter().flatten().count(), 42);
    }
}

#[component]
fn ConsecutiveCalendarViews() -> Element {
    rsx! {
        Calendar { view_date: date!(2026 - 05 - 15),
            CalendarView { CalendarMonthTitle {} }
            CalendarView { CalendarMonthTitle {} }
            CalendarView { CalendarMonthTitle {} }
        }
    }
}

#[test]
fn implicit_calendar_views_render_consecutive_months_on_first_render() {
    let html = render(ConsecutiveCalendarViews);
    assert!(html.contains("May 2026"));
    assert!(html.contains("June 2026"));
    assert!(html.contains("July 2026"));
}

#[component]
fn CalendarDayWithCustomChild() -> Element {
    rsx! {
        Calendar { view_date: date!(2026 - 05 - 15),
            CalendarView {
                CalendarDay { date: date!(2026 - 05 - 15), "Custom day" }
            }
        }
    }
}

#[test]
fn calendar_day_forwards_custom_children() {
    let html = render(CalendarDayWithCustomChild);
    assert!(html.contains("Custom day"));
    assert!(!html.contains(">15</button>"));
}

#[component]
fn RangeCalendarDayWithCustomChild() -> Element {
    rsx! {
        RangeCalendar { view_date: date!(2026 - 05 - 15),
            CalendarView {
                CalendarDay { date: date!(2026 - 05 - 15), "Custom range day" }
            }
        }
    }
}

#[test]
fn range_calendar_day_forwards_custom_children() {
    let html = render(RangeCalendarDayWithCustomChild);
    assert!(html.contains("Custom range day"));
    assert!(!html.contains(">15</button>"));
}

#[component]
fn FullCalendar() -> Element {
    rsx! {
        Calendar {
            view_date: date!(2026 - 05 - 15),
            today: date!(2026 - 05 - 15),
            selected_date: Some(date!(2026 - 05 - 15)),
            CalendarView {
                CalendarHeader {
                    CalendarNavigation {
                        CalendarPreviousMonthButton { "<" }
                        CalendarMonthTitle {}
                        CalendarNextMonthButton { ">" }
                    }
                }
                CalendarGrid {}
            }
        }
    }
}

#[test]
fn the_calendar_root_reports_the_application_role_and_label() {
    let html = render(FullCalendar);
    assert!(html.contains(r#"role="application""#), "{html}");
    assert!(html.contains(r#"aria-label="Calendar""#), "{html}");
    assert!(html.contains("data-disabled=false"), "{html}");
}

#[test]
fn the_grid_uses_the_aria_grid_pattern_with_a_hidden_weekday_header_and_row_per_week() {
    let html = render(FullCalendar);
    assert!(html.contains(r#"role="grid""#), "{html}");
    assert!(html.contains(r#"aria-hidden="true""#), "{html}");
    assert!(html.contains(r#"role="row""#), "{html}");
}

#[test]
fn the_header_and_month_title_report_a_level_2_heading_with_the_formatted_month() {
    let html = render(FullCalendar);
    assert!(html.contains(r#"role="heading""#), "{html}");
    assert!(html.contains(r#"aria-level="2""#), "{html}");
    assert!(html.contains("May 2026"), "{html}");
}

#[test]
fn the_navigation_buttons_carry_descriptive_aria_labels() {
    let html = render(FullCalendar);
    assert!(html.contains(r#"aria-label="Previous month""#), "{html}");
    assert!(html.contains(r#"aria-label="Next month""#), "{html}");
}

#[test]
fn the_today_and_selected_day_reports_its_full_date_label_and_state() {
    let html = render(FullCalendar);
    let marker = html
        .find(r#"aria-label="Friday, May 15, 2026""#)
        .expect("May 15 2026's day cell renders its full-date aria-label");
    let tail = &html[marker..];
    let cell_end = tail
        .find("</button>")
        .map(|i| marker + i)
        .unwrap_or(html.len());
    let cell = &html[marker..cell_end];

    assert!(cell.contains("data-today=true"), "{cell}");
    assert!(cell.contains("data-selected=true"), "{cell}");
    assert!(cell.contains(r#"data-month="current""#), "{cell}");
}

#[component]
fn RangeCalendarWithSelection() -> Element {
    rsx! {
        RangeCalendar {
            view_date: date!(2026 - 05 - 15),
            today: date!(2026 - 05 - 15),
            selected_range: Some(DateRange::new(date!(2026 - 05 - 14), date!(2026 - 05 - 16))),
            CalendarView { CalendarGrid {} }
        }
    }
}

#[test]
fn a_range_calendar_marks_the_start_middle_and_end_of_the_highlighted_range() {
    let html = render(RangeCalendarWithSelection);

    for (label, marker) in [
        (
            r#"aria-label="Thursday, May 14, 2026""#,
            "data-selection-start=true",
        ),
        (
            r#"aria-label="Friday, May 15, 2026""#,
            "data-selection-between=true",
        ),
        (
            r#"aria-label="Saturday, May 16, 2026""#,
            "data-selection-end=true",
        ),
    ] {
        let start = html
            .find(label)
            .unwrap_or_else(|| panic!("missing {label} in {html}"));
        let end = html[start..]
            .find("</button>")
            .map(|i| start + i)
            .unwrap_or(html.len());
        assert!(html[start..end].contains(marker), "{}", &html[start..end]);
    }
}
