//! Black-box tests for `adico_primitives::switch`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/switch.rs`.

use adico_primitives::switch::{Switch, SwitchThumb};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn UncheckedSwitch() -> Element {
    rsx! {
        Switch { name: "airplane-mode".to_string(),
            SwitchThumb {}
        }
    }
}

#[test]
fn an_unchecked_switch_renders_the_switch_role_and_unchecked_state() {
    let html = render(UncheckedSwitch);
    assert!(html.contains(r#"role="switch""#), "{html}");
    assert!(html.contains("aria-checked=false"), "{html}");
    assert!(html.contains(r#"data-state="unchecked""#), "{html}");
    assert!(html.contains("data-disabled=\"false\""), "{html}");
}

#[test]
fn an_unchecked_switch_renders_a_hidden_form_checkbox_input() {
    let html = render(UncheckedSwitch);
    assert!(html.contains(r#"type="checkbox""#), "{html}");
    assert!(html.contains(r#"name="airplane-mode""#), "{html}");
}

#[component]
fn CheckedRequiredSwitch() -> Element {
    rsx! {
        Switch { default_checked: true, required: true,
            SwitchThumb {}
        }
    }
}

#[test]
fn a_default_checked_switch_reports_checked_state() {
    let html = render(CheckedRequiredSwitch);
    assert!(html.contains("aria-checked=true"), "{html}");
    assert!(html.contains(r#"data-state="checked""#), "{html}");
}

#[test]
fn a_required_switch_marks_aria_required() {
    let html = render(CheckedRequiredSwitch);
    assert!(html.contains("aria-required=true"), "{html}");
}

#[component]
fn DisabledSwitch() -> Element {
    rsx! {
        Switch { disabled: true,
            SwitchThumb {}
        }
    }
}

#[test]
fn a_disabled_switch_is_disabled_and_marked() {
    let html = render(DisabledSwitch);
    assert!(html.contains("disabled=true"), "{html}");
    assert!(html.contains("data-disabled=\"true\""), "{html}");
}
