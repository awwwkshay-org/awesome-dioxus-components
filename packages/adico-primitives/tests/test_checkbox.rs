//! Black-box tests for `adico_primitives::checkbox`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/checkbox.rs`.

use adico_primitives::checkbox::{Checkbox, CheckboxIndicator, CheckboxState};
use dioxus::prelude::*;

#[test]
fn checkbox_state_maps_to_aria_checked_values() {
    assert_eq!(CheckboxState::Checked.to_aria_checked(), "true");
    assert_eq!(CheckboxState::Indeterminate.to_aria_checked(), "mixed");
    assert_eq!(CheckboxState::Unchecked.to_aria_checked(), "false");
}

#[test]
fn checkbox_state_maps_to_data_state_values() {
    assert_eq!(CheckboxState::Checked.to_data_state(), "checked");
    assert_eq!(
        CheckboxState::Indeterminate.to_data_state(),
        "indeterminate"
    );
    assert_eq!(CheckboxState::Unchecked.to_data_state(), "unchecked");
}

#[test]
fn negating_checkbox_state_toggles_between_checked_and_unchecked() {
    assert_eq!(!CheckboxState::Unchecked, CheckboxState::Checked);
    assert_eq!(!CheckboxState::Checked, CheckboxState::Unchecked);
    // Negating from indeterminate lands on unchecked, not a third state.
    assert_eq!(!CheckboxState::Indeterminate, CheckboxState::Unchecked);
}

#[test]
fn checkbox_state_converts_to_bool_treating_indeterminate_as_truthy() {
    assert!(bool::from(CheckboxState::Checked));
    assert!(bool::from(CheckboxState::Indeterminate));
    assert!(!bool::from(CheckboxState::Unchecked));
}

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn UncheckedCheckbox() -> Element {
    rsx! {
        Checkbox { name: "tos".to_string(),
            CheckboxIndicator { "CHECKMARK" }
        }
    }
}

#[test]
fn an_unchecked_checkbox_renders_the_checkbox_role_and_unchecked_state() {
    let html = render(UncheckedCheckbox);
    assert!(html.contains(r#"role="checkbox""#), "{html}");
    assert!(html.contains("aria-checked=\"false\""), "{html}");
    assert!(html.contains(r#"data-state="unchecked""#), "{html}");
}

#[test]
fn an_unchecked_checkbox_s_indicator_does_not_render_its_children() {
    let html = render(UncheckedCheckbox);
    assert!(!html.contains("CHECKMARK"), "{html}");
}

#[component]
fn IndeterminateCheckbox() -> Element {
    rsx! {
        Checkbox { default_checked: CheckboxState::Indeterminate,
            CheckboxIndicator { "CHECKMARK" }
        }
    }
}

#[test]
fn an_indeterminate_checkbox_reports_mixed_aria_checked() {
    let html = render(IndeterminateCheckbox);
    assert!(html.contains("aria-checked=\"mixed\""), "{html}");
    assert!(html.contains(r#"data-state="indeterminate""#), "{html}");
}

#[test]
fn an_indeterminate_checkbox_s_indicator_renders_since_it_is_not_unchecked() {
    let html = render(IndeterminateCheckbox);
    assert!(html.contains("CHECKMARK"), "{html}");
}

#[component]
fn CheckedCheckbox() -> Element {
    rsx! {
        Checkbox { default_checked: CheckboxState::Checked,
            CheckboxIndicator { "CHECKMARK" }
        }
    }
}

#[test]
fn a_checked_checkbox_s_indicator_renders_its_children() {
    let html = render(CheckedCheckbox);
    assert!(html.contains("aria-checked=\"true\""), "{html}");
    assert!(html.contains("CHECKMARK"), "{html}");
}

#[component]
fn RequiredDisabledCheckbox() -> Element {
    rsx! {
        Checkbox { required: true, disabled: true,
            CheckboxIndicator { "CHECKMARK" }
        }
    }
}

#[test]
fn a_required_checkbox_marks_aria_required() {
    let html = render(RequiredDisabledCheckbox);
    assert!(html.contains("aria-required=true"), "{html}");
}

#[test]
fn a_disabled_checkbox_is_disabled_and_marked() {
    let html = render(RequiredDisabledCheckbox);
    assert!(html.contains("disabled=true"), "{html}");
    assert!(html.contains("data-disabled=true"), "{html}");
}
