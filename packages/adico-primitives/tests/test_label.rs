//! Black-box tests for `adico_primitives::label`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/label.rs`.

use adico_primitives::label::Label;
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn NameLabel() -> Element {
    rsx! {
        Label { html_for: "name".to_string(), "Name" }
    }
}

#[test]
fn a_label_renders_a_native_label_element_with_a_for_attribute() {
    let html = render(NameLabel);
    assert!(html.contains("<label"), "{html}");
    assert!(html.contains(r#"for="name""#), "{html}");
    assert!(html.contains("Name"), "{html}");
}
