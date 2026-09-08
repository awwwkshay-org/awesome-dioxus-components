//! Black-box tests for `adico_primitives::segment`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/segment.rs`.
//!
//! This file also stands as the promotion-gap fix's own regression test (see
//! `openspec/changes/deduplicate-primitives`, task 2.1): `segment` used to be a
//! crate-private (`mod segment;`) module, reachable only from within
//! `adico-primitives` itself. These imports compile only because it is now `pub mod
//! segment;` — if a future change accidentally re-privates it, this file stops
//! compiling.

use adico_primitives::segment::{
    MeridiemSegment, NumericSegment, SegmentFieldContext, use_segment_field_provider,
};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn FieldWithSegments() -> Element {
    let _ctx: SegmentFieldContext = use_segment_field_provider(
        ReadSignal::new(Signal::new(false)),
        ReadSignal::new(Signal::new(false)),
        ReadSignal::new(Signal::new(false)),
    );
    let value = use_signal(|| Some(5i32));
    let meridiem = use_signal(|| Some(true));

    rsx! {
        NumericSegment {
            aria_label: "minute",
            index: 0usize,
            value,
            default: 0i32,
            on_value_change: move |_| {},
            min: 0i32,
            max: 59i32,
            max_length: 2usize,
            on_format_placeholder: move |_| "--".to_string(),
        }
        MeridiemSegment {
            index: 1usize,
            value: meridiem,
            on_value_change: move |_| {},
        }
    }
}

#[test]
fn a_numeric_segment_and_a_meridiem_segment_are_reachable_and_render_from_outside_the_crate() {
    let html = render(FieldWithSegments);
    assert!(html.contains(r#"role="spinbutton""#), "{html}");
    assert!(html.contains("05"), "{html}");
    assert!(html.contains("PM"), "{html}");
}
