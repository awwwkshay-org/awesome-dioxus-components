//! Black-box tests for `adico_primitives::toggle_group`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/toggle_group.rs`.

use adico_primitives::toggle_group::{ToggleGroup, ToggleItem};
use dioxus::prelude::*;
use std::collections::HashSet;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn MultiplePressedGroup() -> Element {
    rsx! {
        ToggleGroup {
            horizontal: true,
            allow_multiple_pressed: true,
            default_pressed: HashSet::from([0usize]),
            ToggleItem { index: 0usize, "B" }
            ToggleItem { index: 1usize, "I" }
        }
    }
}

#[test]
fn toggle_group_reports_orientation_and_multiple_pressed_data_attributes() {
    let html = render(MultiplePressedGroup);
    assert!(html.contains(r#"data-orientation="horizontal""#), "{html}");
    assert!(html.contains("data-allow-multiple-pressed=true"), "{html}");
}

#[test]
fn the_default_pressed_index_starts_pressed_and_the_rest_do_not() {
    let html = render(MultiplePressedGroup);
    assert!(html.contains("aria-pressed=true"), "{html}");
    assert!(html.contains("aria-pressed=false"), "{html}");
}

#[component]
fn DisabledSinglePressedGroup() -> Element {
    rsx! {
        ToggleGroup { horizontal: false, disabled: true,
            ToggleItem { index: 0usize, "B" }
        }
    }
}

#[test]
fn a_disabled_toggle_group_disables_its_items() {
    let html = render(DisabledSinglePressedGroup);
    assert!(html.contains("disabled=true"), "{html}");
}

#[test]
fn a_vertical_toggle_group_reports_vertical_orientation() {
    let html = render(DisabledSinglePressedGroup);
    assert!(html.contains(r#"data-orientation="vertical""#), "{html}");
}
