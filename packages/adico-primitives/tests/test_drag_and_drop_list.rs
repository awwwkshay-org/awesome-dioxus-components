//! Black-box tests for `adico_primitives::drag_and_drop_list`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/drag_and_drop_list.rs`.

use adico_primitives::drag_and_drop_list::DragAndDropList;
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn items() -> Vec<Element> {
    ["Item1", "Item2", "Item3"].map(|t| rsx! { {t} }).to_vec()
}

#[component]
fn DefaultDragAndDropList() -> Element {
    rsx! {
        DragAndDropList { items: items() }
    }
}

#[test]
fn the_list_reports_the_sortable_list_role_and_a_default_label() {
    let html = render(DefaultDragAndDropList);
    assert!(
        html.contains(r#"aria-roledescription="sortable list""#),
        "{html}"
    );
    assert!(html.contains(r#"aria-label="Sortable list""#), "{html}");
    assert!(
        html.contains(r#"aria-describedby="dnd-instructions""#),
        "{html}"
    );
}

#[test]
fn the_instructions_are_visually_hidden_but_reachable_by_id() {
    let html = render(DefaultDragAndDropList);
    assert!(html.contains(r#"id="dnd-instructions""#), "{html}");
    assert!(html.contains("Press Enter to start reordering"), "{html}");
    assert!(html.contains("clip:rect(0,0,0,0)"), "{html}");
}

#[test]
fn the_live_region_reports_status_and_assertive_atomic_announcements() {
    let html = render(DefaultDragAndDropList);
    assert!(html.contains(r#"role="status""#), "{html}");
    assert!(html.contains(r#"aria-live="assertive""#), "{html}");
    assert!(html.contains(r#"aria-atomic="true""#), "{html}");
}

#[test]
fn every_item_reports_the_sortable_item_role_and_is_draggable_and_not_grabbed() {
    let html = render(DefaultDragAndDropList);
    assert_eq!(
        html.matches(r#"aria-roledescription="sortable item""#)
            .count(),
        3,
        "{html}"
    );
    assert_eq!(html.matches(r#"draggable="true""#).count(), 3, "{html}");
    assert_eq!(html.matches("aria-grabbed=\"false\"").count(), 3, "{html}");
    assert!(!html.contains("aria-grabbed=\"true\""), "{html}");
}

#[component]
fn LabelledDragAndDropList() -> Element {
    rsx! {
        DragAndDropList { items: items(), aria_label: "Reorder your favorites" }
    }
}

#[test]
fn a_custom_aria_label_overrides_the_default() {
    let html = render(LabelledDragAndDropList);
    assert!(
        html.contains(r#"aria-label="Reorder your favorites""#),
        "{html}"
    );
    assert!(!html.contains(r#"aria-label="Sortable list""#), "{html}");
}
