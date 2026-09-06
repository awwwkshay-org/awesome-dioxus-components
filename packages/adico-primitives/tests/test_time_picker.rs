//! Black-box tests for `adico_primitives::time_picker`, per this repo's
//! test-placement convention (see `packages/adico-primitives/tests/test_date_picker.rs`'s
//! own header): SSR-render assertions live here, not inline in
//! `src/time_picker.rs` (pure-function unit tests for `angle_to_value`/
//! `to_display_hour` do stay inline, matching this crate's mixed convention
//! for pure logic vs. rendered-component behavior).

use adico_primitives::time_picker::{TimePicker, TimePickerInput};
use dioxus::prelude::*;
use time::macros::time;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn ControlledTimePicker24h() -> Element {
    rsx! {
        TimePicker { selected_time: Some(time!(14:30:00)),
            TimePickerInput {}
        }
    }
}

#[component]
fn ControlledTimePicker12h() -> Element {
    rsx! {
        TimePicker { selected_time: Some(time!(14:30:00)), is_12h: true,
            TimePickerInput {}
        }
    }
}

#[component]
fn ControlledTimePickerWithSeconds() -> Element {
    rsx! {
        TimePicker { selected_time: Some(time!(14:30:45)), show_seconds: true,
            TimePickerInput {}
        }
    }
}

#[test]
fn the_time_picker_root_reports_the_group_role_and_a_time_label() {
    let html = render(ControlledTimePicker24h);
    assert!(html.contains(r#"role="group""#), "{html}");
    assert!(html.contains(r#"aria-label="Time""#), "{html}");
}

#[test]
fn the_default_time_input_composes_two_spinbutton_segments_and_one_hidden_separator() {
    let html = render(ControlledTimePicker24h);
    assert_eq!(html.matches(r#"role="spinbutton""#).count(), 2, "{html}");
    assert!(html.contains(r#"aria-label="hour""#), "{html}");
    assert!(html.contains(r#"aria-label="minute""#), "{html}");
    assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 1, "{html}");
}

#[test]
fn a_24_hour_time_input_renders_the_canonical_hour() {
    let html = render(ControlledTimePicker24h);
    let marker = html
        .find(r#"aria-label="hour""#)
        .expect("the hour segment renders");
    let head = &html[..marker];
    let attr = "aria-valuenow=\"";
    let start = head.rfind(attr).expect("hour segment has aria-valuenow") + attr.len();
    let end = head[start..].find('"').unwrap() + start;
    assert_eq!(&head[start..end], "14", "{html}");
}

#[test]
fn a_12_hour_time_input_adds_a_meridiem_segment_and_displays_the_12_hour_value() {
    let html = render(ControlledTimePicker12h);
    // hour + minute + meridiem = 3 spinbuttons.
    assert_eq!(html.matches(r#"role="spinbutton""#).count(), 3, "{html}");
    assert!(html.contains("PM"), "{html}");

    let marker = html
        .find(r#"aria-label="hour""#)
        .expect("the hour segment renders");
    let head = &html[..marker];
    let attr = "aria-valuenow=\"";
    let start = head.rfind(attr).expect("hour segment has aria-valuenow") + attr.len();
    let end = head[start..].find('"').unwrap() + start;
    // 14:00 is 2 PM -- the 12-hour segment displays "2", not the canonical "14".
    assert_eq!(&head[start..end], "2", "{html}");
}

#[test]
fn show_seconds_adds_a_third_segment_and_a_second_separator() {
    let html = render(ControlledTimePickerWithSeconds);
    assert_eq!(html.matches(r#"role="spinbutton""#).count(), 3, "{html}");
    assert!(html.contains(r#"aria-label="second""#), "{html}");
    assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 2, "{html}");
}
