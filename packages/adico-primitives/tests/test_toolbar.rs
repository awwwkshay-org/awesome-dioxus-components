//! Black-box tests for `adico_primitives::toolbar`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/toolbar.rs`.

use adico_primitives::toolbar::{Toolbar, ToolbarButton, ToolbarSeparator};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn HorizontalToolbar() -> Element {
    rsx! {
        Toolbar { aria_label: "Text formatting".to_string(),
            ToolbarButton { index: 0usize, "Bold" }
            ToolbarSeparator {}
            ToolbarButton { index: 1usize, disabled: true, "Italic" }
        }
    }
}

#[test]
fn toolbar_renders_the_toolbar_role_and_horizontal_orientation_by_default() {
    let html = render(HorizontalToolbar);
    assert!(html.contains(r#"role="toolbar""#), "{html}");
    assert!(html.contains(r#"data-orientation="horizontal""#), "{html}");
    assert!(html.contains(r#"aria-label="Text formatting""#), "{html}");
}

#[test]
fn a_disabled_toolbar_button_is_disabled_and_marked() {
    let html = render(HorizontalToolbar);
    assert!(html.contains("disabled=true"), "{html}");
    assert!(html.contains("data-disabled=true"), "{html}");
}

#[test]
fn the_separator_defaults_to_a_real_separator_role_perpendicular_to_the_toolbar() {
    let html = render(HorizontalToolbar);
    // The toolbar is horizontal, so a separator with no explicit orientation inverts it.
    assert!(html.contains(r#"role="separator""#), "{html}");
    assert!(html.contains(r#"aria-orientation="vertical""#), "{html}");
    assert!(html.contains(r#"data-orientation="vertical""#), "{html}");
}

#[component]
fn VerticalToolbarWithDecorativeSeparator() -> Element {
    rsx! {
        Toolbar { horizontal: false,
            ToolbarButton { index: 0usize, "One" }
            ToolbarSeparator { decorative: true }
            ToolbarButton { index: 1usize, "Two" }
        }
    }
}

#[test]
fn vertical_toolbar_reports_vertical_orientation() {
    let html = render(VerticalToolbarWithDecorativeSeparator);
    assert!(html.contains(r#"data-orientation="vertical""#), "{html}");
}

#[test]
fn a_decorative_separator_uses_role_none_and_omits_aria_orientation() {
    let html = render(VerticalToolbarWithDecorativeSeparator);
    assert!(html.contains(r#"role="none""#), "{html}");
    assert!(!html.contains("aria-orientation"), "{html}");
    // Still carries a styling hook, inverted from the (vertical) toolbar.
    assert!(html.contains(r#"data-orientation="horizontal""#), "{html}");
}
