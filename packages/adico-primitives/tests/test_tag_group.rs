//! Black-box tests for `adico_primitives::tag_group`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/tag_group.rs`.
//!
//! Two tiers: pure free-function tests for the registration/removal-focus bookkeeping
//! (`sync_tag_item`/`insert_tag_item`/`selected_values_after_removal`/
//! `next_focus_after_removal`), which had zero coverage before this task and carry this
//! file's most error-prone index math; and render-level tests for the public component API
//! (ARIA grid/row/gridcell roles, selection/removal data attributes), matching `select.rs`'s
//! own testing approach. `TagGroupState`'s interaction methods (`toggle_value`, `remove_items`,
//! keyboard handling) are exercised only indirectly, through the render-level tests' declarative
//! `default_value`/`removed`-adjacent states — the free-function tests cover the same
//! bookkeeping those methods delegate to.

use adico_primitives::selection::RcPartialEqValue;
use adico_primitives::tag_group::{
    TagGroup, TagGroupLabel, TagGroupMulti, TagItem, TagList, TagOption, TagRemoveButton,
    insert_tag_item, next_focus_after_removal, selected_values_after_removal, sync_tag_item,
};
use dioxus::prelude::*;

fn item(id: &str, index: usize) -> TagItem {
    TagItem {
        id: id.to_string(),
        index,
        value: RcPartialEqValue::new(id.to_string()),
        text_value: id.to_string(),
        disabled: false,
        removable: true,
        removed: false,
    }
}

fn ids(items: &[TagItem]) -> Vec<&str> {
    items.iter().map(|item| item.id.as_str()).collect()
}

#[test]
fn insert_tag_item_keeps_items_sorted_by_index() {
    let mut items = vec![item("a", 0), item("c", 2)];
    insert_tag_item(&mut items, item("b", 1));
    assert_eq!(ids(&items), ["a", "b", "c"]);
}

#[test]
fn sync_tag_item_updates_an_existing_item_and_reorders_by_new_index() {
    let mut items = vec![item("a", 0), item("b", 1), item("c", 2)];
    sync_tag_item(&mut items, item("a", 3));
    assert_eq!(ids(&items), ["b", "c", "a"]);
}

#[test]
fn sync_tag_item_preserves_removed_when_the_value_is_unchanged() {
    let mut removed_a = item("a", 0);
    removed_a.removed = true;
    let mut items = vec![removed_a];

    // Same id, same value, new index (e.g. a sibling was removed and indices shifted) — still
    // removed.
    sync_tag_item(&mut items, item("a", 0));
    assert!(
        items[0].removed,
        "re-registering the same value must not un-remove it"
    );
}

#[test]
fn sync_tag_item_clears_removed_when_the_value_changes() {
    let mut removed_a = item("a", 0);
    removed_a.removed = true;
    let mut items = vec![removed_a];

    let mut replacement = item("a", 0);
    replacement.value = RcPartialEqValue::new("different".to_string());
    sync_tag_item(&mut items, replacement);
    assert!(
        !items[0].removed,
        "an id being reused for a genuinely different value must not stay removed"
    );
}

#[test]
fn selected_values_after_removal_drops_only_the_removed_values() {
    let a = RcPartialEqValue::new("a".to_string());
    let b = RcPartialEqValue::new("b".to_string());
    let c = RcPartialEqValue::new("c".to_string());
    let selected = [a.clone(), b.clone(), c.clone()];
    let removed = [b];

    let remaining = selected_values_after_removal(&selected, &removed);
    assert_eq!(remaining.len(), 2);
    assert!(remaining.contains(&a));
    assert!(remaining.contains(&c));
}

#[test]
fn next_focus_after_removal_moves_to_the_next_remaining_item() {
    let items = vec![item("a", 0), item("b", 1), item("c", 2)];
    let removed = vec!["b".to_string()];

    let next = next_focus_after_removal(&items, 1, &removed, false);
    assert_eq!(next, Some(2), "focus should land on the next item, c");
}

#[test]
fn next_focus_after_removal_falls_back_to_the_previous_item_at_the_end_without_looping() {
    let items = vec![item("a", 0), item("b", 1), item("c", 2)];
    let removed = vec!["c".to_string()];

    let next = next_focus_after_removal(&items, 2, &removed, false);
    assert_eq!(
        next,
        Some(1),
        "removing the last item without looping should fall back to the new last item"
    );
}

#[test]
fn next_focus_after_removal_wraps_to_the_first_item_when_looping() {
    let items = vec![item("a", 0), item("b", 1), item("c", 2)];
    let removed = vec!["c".to_string()];

    let next = next_focus_after_removal(&items, 2, &removed, true);
    assert_eq!(next, Some(0), "looping should wrap back to the first item");
}

#[test]
fn next_focus_after_removal_returns_none_when_every_item_is_removed_or_unfocusable() {
    let mut only = item("a", 0);
    only.disabled = true;
    let items = vec![only];

    let next = next_focus_after_removal(&items, 0, &[], false);
    assert_eq!(next, None);
}

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn LabelsTagGroup() -> Element {
    rsx! {
        TagGroup::<String> {
            default_value: "bug".to_string(),
            TagGroupLabel { "Labels" }
            TagList {
                TagOption::<String> { index: 0usize, value: "bug".to_string(), text_value: "Bug".to_string(), "bug" }
                TagOption::<String> { index: 1usize, value: "feature".to_string(), text_value: "Feature".to_string(), disabled: true, "feature" }
            }
        }
    }
}

#[test]
fn tag_group_renders_a_grid_of_rows_and_gridcells() {
    let html = render(LabelsTagGroup);
    assert!(html.contains(r#"role="grid""#), "{html}");
    assert!(html.contains(r#"role="row""#), "{html}");
    assert!(html.contains(r#"role="gridcell""#), "{html}");
    assert!(html.contains(r#"aria-colcount="1""#), "{html}");
}

#[test]
fn tag_group_marks_the_default_value_selected_and_the_disabled_option_disabled() {
    let html = render(LabelsTagGroup);
    assert!(html.contains("data-selected=true"), "{html}");
    assert!(html.contains("data-disabled=true"), "{html}");
    assert!(html.contains("aria-disabled=true"), "{html}");
}

// `TagGroupLabel` -> `TagList`'s `aria-labelledby` wiring goes through `TagGroupCtx.labeled_by`,
// set inside a `use_effect` — the same effect-driven-state limitation `test_select.rs`'s
// module doc comment describes for `SelectGroupLabel`'s equivalent wiring, so it isn't
// assertable under a bare `rebuild_in_place()` here either. Confirmed empirically while
// writing this test (no `aria-labelledby` attribute appeared at all), not assumed.

#[component]
fn MultiSelectTagGroup() -> Element {
    rsx! {
        TagGroupMulti::<String> {
            default_values: vec!["bug".to_string(), "urgent".to_string()],
            TagList {
                TagOption::<String> { index: 0usize, value: "bug".to_string(), "bug" }
                TagOption::<String> { index: 1usize, value: "urgent".to_string(), "urgent" }
                TagOption::<String> { index: 2usize, value: "later".to_string(), "later" }
            }
        }
    }
}

#[test]
fn tag_group_multi_marks_every_default_value_selected_and_is_multiselectable() {
    let html = render(MultiSelectTagGroup);
    assert!(html.contains(r#"aria-multiselectable="true""#), "{html}");
    let selected_count = html.matches("data-selected=true").count();
    assert_eq!(selected_count, 2, "{html}");
}

#[component]
fn TagGroupWithRemoveButton() -> Element {
    rsx! {
        TagGroup::<String> {
            TagList {
                TagOption::<String> { index: 0usize, value: "bug".to_string(),
                    "bug"
                    TagRemoveButton { "x" }
                }
            }
        }
    }
}

// `TagRemoveButton`'s own `disabled`/`aria-label` state reads `TagGroupState::can_remove_item`/
// `text_value`, both of which look the option up in `TagGroupState::items` — populated by
// `TagOption`'s own `use_effect`-driven registration (the same class of limitation
// `test_select.rs`'s module doc comment describes), so under a bare `rebuild_in_place()` the
// button renders as if its option were never registered (`disabled=true`, an empty label).
// Confirmed empirically, not assumed. Only the button's own unconditional existence is
// assertable here.
#[test]
fn a_tag_option_with_a_remove_button_renders_a_remove_button() {
    let html = render(TagGroupWithRemoveButton);
    assert!(html.contains("<button"), "{html}");
    assert!(html.contains(">x</button>"), "{html}");
}

#[component]
fn TagGroupWithoutRemoveButton() -> Element {
    rsx! {
        TagGroup::<String> {
            TagList {
                TagOption::<String> { index: 0usize, value: "bug".to_string(), "bug" }
            }
        }
    }
}

#[test]
fn a_tag_option_without_a_remove_button_is_not_removable() {
    let html = render(TagGroupWithoutRemoveButton);
    assert!(!html.contains("Remove item"), "{html}");
}

// `TagGroupEmpty` gates on `mounted() && ctx.is_empty()`, where `mounted` is set inside a
// `use_effect` specifically so a single-render SSR pass never flashes the empty state before
// options have had a chance to register — which also means it can never be observed as
// genuinely "empty" under a bare `rebuild_in_place()` either, confirmed empirically (it always
// renders nothing on the first pass). `tag_group_empty_does_not_render_when_options_are_present`
// below still covers a real, non-effect-dependent guarantee: it never renders while options
// exist, regardless of mount timing.

#[test]
fn tag_group_empty_does_not_render_when_options_are_present() {
    let html = render(TagGroupWithoutRemoveButton);
    assert!(!html.contains("No tags yet"), "{html}");
}
