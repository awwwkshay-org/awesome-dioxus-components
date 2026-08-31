//! Black-box tests for `adico_primitives::dialog`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/dialog.rs`.

use adico_primitives::dialog::{DialogContent, DialogDescription, DialogRoot, DialogTitle};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn OpenDialog() -> Element {
    rsx! {
        DialogRoot { default_open: true,
            DialogContent {
                DialogTitle { "Item information" }
                DialogDescription { "Here is some additional information about the item." }
            }
        }
    }
}

#[test]
fn an_open_dialog_reports_the_modal_dialog_role_and_open_state() {
    let html = render(OpenDialog);
    assert!(html.contains(r#"role="dialog""#), "{html}");
    assert!(html.contains(r#"aria-modal="true""#), "{html}");
    assert!(html.contains(r#"data-state="open""#), "{html}");
    assert!(html.contains("Item information"), "{html}");
    assert!(
        html.contains("Here is some additional information about the item."),
        "{html}"
    );
}

#[test]
fn the_title_and_description_are_linked_via_aria_labelledby_and_describedby() {
    let html = render(OpenDialog);

    let title_marker = html
        .find("Item information")
        .expect("title renders its text");
    let title_head = &html[..title_marker];
    let id_attr = "id=\"";
    let id_start = title_head.rfind(id_attr).expect("title has an id") + id_attr.len();
    let id_end = title_head[id_start..].find('"').unwrap() + id_start;
    let title_id = &title_head[id_start..id_end];

    let desc_marker = html
        .find("Here is some additional information about the item.")
        .expect("description renders its text");
    let desc_head = &html[..desc_marker];
    let desc_id_start = desc_head.rfind(id_attr).expect("description has an id") + id_attr.len();
    let desc_id_end = desc_head[desc_id_start..].find('"').unwrap() + desc_id_start;
    let desc_id = &desc_head[desc_id_start..desc_id_end];

    assert!(
        html.contains(&format!(r#"aria-labelledby="{title_id}""#)),
        "{html}"
    );
    assert!(
        html.contains(&format!(r#"aria-describedby="{desc_id}""#)),
        "{html}"
    );
}

#[component]
fn ClosedDialog() -> Element {
    rsx! {
        DialogRoot {
            DialogContent {
                DialogTitle { "Item information" }
            }
        }
    }
}

#[test]
fn a_closed_dialog_reports_closed_state_and_does_not_render_its_content() {
    let html = render(ClosedDialog);
    assert!(html.contains(r#"data-state="closed""#), "{html}");
    assert!(!html.contains(r#"role="dialog""#), "{html}");
    assert!(!html.contains("Item information"), "{html}");
}

#[component]
fn NonModalOpenDialog() -> Element {
    rsx! {
        DialogRoot { default_open: true, is_modal: false,
            DialogContent {
                DialogTitle { "Preferences" }
            }
        }
    }
}

#[test]
fn a_non_modal_dialog_still_reports_the_dialog_role_but_stays_aria_modal_true() {
    // `DialogContent` renders `aria_modal: "true"` unconditionally regardless of `is_modal` --
    // unlike `popover.rs`'s `PopoverContentRendered`, which conditions its `aria_modal` on
    // `is_modal` -- so `is_modal: false` only affects focus-trap behavior (a no-op on the
    // SSR-fallback target this test runs under), not the rendered `aria-modal` attribute.
    let html = render(NonModalOpenDialog);
    assert!(html.contains(r#"role="dialog""#), "{html}");
    assert!(html.contains(r#"aria-modal="true""#), "{html}");
}
