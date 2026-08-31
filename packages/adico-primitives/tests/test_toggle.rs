//! Black-box tests for `adico_primitives::toggle`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/toggle.rs`.

use adico_primitives::toggle::Toggle;
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn UnpressedToggle() -> Element {
    rsx! { Toggle { "B" } }
}

#[test]
fn an_unpressed_toggle_reports_off_state() {
    let html = render(UnpressedToggle);
    assert!(html.contains("aria-pressed=false"), "{html}");
    assert!(html.contains(r#"data-state="off""#), "{html}");
}

#[component]
fn PressedDisabledToggle() -> Element {
    rsx! { Toggle { default_pressed: true, disabled: true, "B" } }
}

#[test]
fn a_default_pressed_toggle_reports_on_state() {
    let html = render(PressedDisabledToggle);
    assert!(html.contains("aria-pressed=true"), "{html}");
    assert!(html.contains(r#"data-state="on""#), "{html}");
}

#[test]
fn a_disabled_toggle_is_disabled_and_marked() {
    let html = render(PressedDisabledToggle);
    assert!(html.contains("disabled=true"), "{html}");
    assert!(html.contains("data-disabled=true"), "{html}");
}
