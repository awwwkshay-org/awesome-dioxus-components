//! Black-box tests for `adico_primitives::date_picker`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/date_picker.rs`. Also carries the module's 4 previously-inline
//! `#[cfg(test)] mod tests` tests, moved here verbatim.

#[cfg(not(any(feature = "web", feature = "native")))]
use adico_primitives::popover::{PopoverContent, PopoverTrigger};
use adico_primitives::{
    calendar::DateRange,
    date_picker::{DatePicker, DatePickerInput, DateRangePicker, DateRangePickerInput},
};
use dioxus::prelude::*;
#[cfg(not(any(feature = "web", feature = "native")))]
use dioxus_core::{Event, Mutation};
#[cfg(not(any(feature = "web", feature = "native")))]
use dioxus_html::{
    EventData, SerializedHtmlEventConverter, SerializedMouseData, set_event_converter,
};
use time::macros::date;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn ControlledDatePicker() -> Element {
    rsx! {
        DatePicker { selected_date: Some(date!(2026 - 05 - 07)),
            DatePickerInput {}
        }
    }
}

#[component]
fn ControlledDateRangePicker() -> Element {
    rsx! {
        DateRangePicker {
            selected_range: Some(DateRange::new(date!(2026 - 05 - 07), date!(2026 - 05 - 11))),
            DateRangePickerInput {}
        }
    }
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[component]
fn OpenDatePickerPopover() -> Element {
    rsx! {
        DatePicker {
            adico_primitives::date_picker::DatePickerPopover {
                open: Some(true),
                PopoverTrigger { "Select date" }
                PopoverContent { "Calendar popup" }
            }
        }
    }
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[component]
fn InteractiveDatePickerPopover() -> Element {
    rsx! {
        DatePicker {
            adico_primitives::date_picker::DatePickerPopover {
                PopoverTrigger { "Select date" }
                PopoverContent { "Interactive calendar popup" }
            }
        }
    }
}

#[test]
fn date_picker_input_renders_controlled_date_on_first_render() {
    let html = render(ControlledDatePicker);
    assert!(html.contains("2026"));
    assert!(html.contains("05"));
    assert!(html.contains("07"));
    assert!(!html.contains("YYYY"));
    assert!(!html.contains("MM"));
    assert!(!html.contains("DD"));
}

#[test]
fn date_range_picker_input_renders_controlled_range_on_first_render() {
    let html = render(ControlledDateRangePicker);
    assert!(html.contains("2026"));
    assert!(html.contains("05"));
    assert!(html.contains("07"));
    assert!(html.contains("11"));
    assert!(!html.contains("YYYY"));
    assert!(!html.contains("MM"));
    assert!(!html.contains("DD"));
}

// `use_animated_open` (lib.rs) only takes this synchronous path when neither platform feature
// is active; with "web"/"native" enabled it waits on a `document::eval` animation-end signal
// that a bare `VirtualDom`/SSR test has no real JS runtime to ever resolve, so the popover
// content would never mount and this assertion can't hold.
#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn date_picker_popover_honors_controlled_open_on_first_render() {
    let html = render(OpenDatePickerPopover);
    assert!(html.contains("data-state=\"open\""));
    assert!(html.contains("Calendar popup"));
}

// See the cfg note on `date_picker_popover_honors_controlled_open_on_first_render`.
#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn date_picker_trigger_opens_the_popover() {
    let mut dom = VirtualDom::new(InteractiveDatePickerPopover);
    let edits = dom.rebuild_to_vec();
    let trigger_id = edits
        .edits
        .iter()
        .find_map(|edit| match edit {
            Mutation::NewEventListener { name, id } if name == "click" => Some(*id),
            _ => None,
        })
        .expect("popover trigger click listener");

    set_event_converter(Box::new(SerializedHtmlEventConverter));
    let event = Event::new(
        EventData::Mouse(SerializedMouseData::default()).into_any(),
        true,
    );
    dom.runtime().handle_event("click", event, trigger_id);
    dom.render_immediate_to_vec();
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains("data-state=\"open\""));
    assert!(html.contains("Interactive calendar popup"));
}

#[test]
fn the_date_picker_root_reports_the_group_role_and_a_date_label() {
    let html = render(ControlledDatePicker);
    assert!(html.contains(r#"role="group""#), "{html}");
    assert!(html.contains(r#"aria-label="Date""#), "{html}");
    assert!(html.contains("data-disabled=false"), "{html}");
}

#[test]
fn the_date_range_picker_root_reports_the_group_role_and_a_date_range_label() {
    let html = render(ControlledDateRangePicker);
    assert!(html.contains(r#"role="group""#), "{html}");
    assert!(html.contains(r#"aria-label="Date Range""#), "{html}");
}

#[test]
fn the_default_date_input_composes_three_spinbutton_segments_and_hidden_separators() {
    let html = render(ControlledDatePicker);
    assert_eq!(html.matches(r#"role="spinbutton""#).count(), 3, "{html}");
    assert!(html.contains(r#"aria-label="year""#), "{html}");
    assert!(html.contains(r#"aria-label="month""#), "{html}");
    assert!(html.contains(r#"aria-label="day""#), "{html}");
    assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 2, "{html}");
}

#[test]
fn the_year_segment_reports_its_current_value_via_aria_valuenow() {
    let html = render(ControlledDatePicker);
    let marker = html
        .find(r#"aria-label="year""#)
        .expect("the year segment renders");
    let head = &html[..marker];
    let attr = "aria-valuenow=\"";
    let start = head.rfind(attr).expect("year segment has aria-valuenow") + attr.len();
    let end = head[start..].find('"').unwrap() + start;
    assert_eq!(&head[start..end], "2026", "{html}");
}
