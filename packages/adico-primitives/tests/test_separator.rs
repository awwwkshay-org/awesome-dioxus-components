//! Black-box tests for `adico_primitives::separator`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/separator.rs`.

use adico_primitives::separator::Separator;
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn DefaultSeparator() -> Element {
    rsx! {
        Separator {}
    }
}

#[test]
fn the_default_separator_is_horizontal_and_semantic() {
    let html = render(DefaultSeparator);
    assert!(html.contains(r#"role="separator""#), "{html}");
    assert!(html.contains(r#"aria-orientation="horizontal""#), "{html}");
    assert!(html.contains(r#"data-orientation="horizontal""#), "{html}");
}

#[component]
fn VerticalSeparator() -> Element {
    rsx! {
        Separator { horizontal: false }
    }
}

#[test]
fn a_vertical_separator_reports_vertical_orientation() {
    let html = render(VerticalSeparator);
    assert!(html.contains(r#"aria-orientation="vertical""#), "{html}");
    assert!(html.contains(r#"data-orientation="vertical""#), "{html}");
}

#[component]
fn DecorativeSeparator() -> Element {
    rsx! {
        Separator { decorative: true }
    }
}

#[test]
fn a_decorative_separator_drops_the_separator_role_and_aria_orientation() {
    let html = render(DecorativeSeparator);
    assert!(html.contains(r#"role="none""#), "{html}");
    assert!(!html.contains("aria-orientation"), "{html}");
    // Still carries the styling hook even when semantically removed.
    assert!(html.contains(r#"data-orientation="horizontal""#), "{html}");
}
