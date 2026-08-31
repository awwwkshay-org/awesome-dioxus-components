//! Black-box tests for `adico_primitives::radio_group`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/radio_group.rs`.

use adico_primitives::radio_group::{RadioGroup, RadioItem};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn ThreeOptionsSecondSelected() -> Element {
    rsx! {
        RadioGroup { default_value: "option2".to_string(), horizontal: true,
            RadioItem { value: "option1".to_string(), index: 0usize, "Blue" }
            RadioItem { value: "option2".to_string(), index: 1usize, "Red" }
            RadioItem { value: "option3".to_string(), index: 2usize, disabled: true, "Green" }
        }
    }
}

#[test]
fn radio_group_renders_radiogroup_and_radio_roles() {
    let html = render(ThreeOptionsSecondSelected);
    assert!(html.contains(r#"role="radiogroup""#), "{html}");
    assert!(html.contains(r#"role="radio""#), "{html}");
    assert!(html.contains(r#"data-orientation="horizontal""#), "{html}");
}

#[test]
fn the_default_value_s_item_is_checked_and_the_rest_are_not() {
    let html = render(ThreeOptionsSecondSelected);
    assert!(html.contains("aria-checked=true"), "{html}");
    assert!(html.contains(r#"data-state="checked""#), "{html}");
    assert!(html.contains("aria-checked=false"), "{html}");
    assert!(html.contains(r#"data-state="unchecked""#), "{html}");
}

#[test]
fn the_disabled_item_is_disabled_and_marked() {
    let html = render(ThreeOptionsSecondSelected);
    assert!(html.contains("disabled=true"), "{html}");
    assert!(html.contains("data-disabled=true"), "{html}");
}

#[component]
fn RequiredEmptyGroup() -> Element {
    rsx! {
        RadioGroup { required: true,
            RadioItem { value: "a".to_string(), index: 0usize, "A" }
            RadioItem { value: "b".to_string(), index: 1usize, "B" }
        }
    }
}

#[test]
fn a_required_group_with_no_selection_marks_aria_required_and_nothing_checked() {
    let html = render(RequiredEmptyGroup);
    assert!(html.contains("aria-required=true"), "{html}");
    assert!(!html.contains("aria-checked=true"), "{html}");
}
