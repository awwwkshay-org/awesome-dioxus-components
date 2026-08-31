//! Black-box tests for `adico_primitives::alert_dialog`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/alert_dialog.rs`.

use adico_primitives::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn OpenAlertDialog() -> Element {
    rsx! {
        AlertDialogRoot { default_open: true,
            AlertDialogContent {
                AlertDialogTitle { "Delete item" }
                AlertDialogDescription { "Are you sure?" }
                AlertDialogActions {
                    AlertDialogCancel { "Cancel" }
                    AlertDialogAction { "Delete" }
                }
            }
        }
    }
}

#[test]
fn an_open_alert_dialog_reports_the_modal_alertdialog_role_and_open_state() {
    let html = render(OpenAlertDialog);
    assert!(html.contains(r#"role="alertdialog""#), "{html}");
    assert!(html.contains(r#"aria-modal="true""#), "{html}");
    assert!(html.contains(r#"data-state="open""#), "{html}");
    assert!(html.contains("Delete item"), "{html}");
    assert!(html.contains("Are you sure?"), "{html}");
}

#[test]
fn the_title_and_description_are_linked_via_aria_labelledby_and_describedby() {
    let html = render(OpenAlertDialog);

    let title_marker = html.find("Delete item").expect("title renders its text");
    let title_head = &html[..title_marker];
    let id_attr = "id=\"";
    let id_start = title_head.rfind(id_attr).expect("title has an id") + id_attr.len();
    let id_end = title_head[id_start..].find('"').unwrap() + id_start;
    let title_id = &title_head[id_start..id_end];

    let desc_marker = html
        .find("Are you sure?")
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
fn ClosedAlertDialog() -> Element {
    rsx! {
        AlertDialogRoot {
            AlertDialogContent {
                AlertDialogTitle { "Delete item" }
            }
        }
    }
}

#[test]
fn a_closed_alert_dialog_reports_closed_state_and_does_not_render_its_content() {
    let html = render(ClosedAlertDialog);
    assert!(html.contains(r#"data-state="closed""#), "{html}");
    assert!(!html.contains(r#"role="alertdialog""#), "{html}");
    assert!(!html.contains("Delete item"), "{html}");
}

#[test]
fn an_open_dialog_s_action_and_cancel_buttons_are_tabbable() {
    let html = render(OpenAlertDialog);
    let action_marker = html.find("Delete").expect("action renders its text");
    let cancel_marker = html.find("Cancel").expect("cancel renders its text");
    assert!(action_marker > 0 && cancel_marker > 0);
    // Both buttons stay in the tab order (tabindex="0") while the dialog is open.
    assert!(html.contains(r#"tabindex="0""#), "{html}");
}
