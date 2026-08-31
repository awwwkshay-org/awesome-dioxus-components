//! Black-box tests for `adico_primitives::select`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/select.rs`.
//!
//! `select.rs`'s own state-machine logic (selection matching, roving focus, typeahead
//! matching) is independently covered by `selectable.rs`/`collection.rs`/`typeahead.rs`'s own
//! consumers and `test_collection.rs`/`test_typeahead.rs`; these tests instead cover this
//! rewrite's actual regression surface: the ARIA/render wiring, and the new
//! `Positioner`/`layer` participation (anchored positioning, root-level Escape, outside-pointer
//! dismiss) that this file's listbox previously had none of at all. The underlying "root +
//! separately-scoped descendant share one layer slot" invariant that
//! `use_escape_key`/`use_outside_dismiss` rely on is already regression-tested generically in
//! `test_layer.rs` (`root_and_member_in_separate_scopes_share_one_layer_slot`); this file only
//! checks that `Select`'s tree actually wires up to trigger that shared path, not the
//! mechanism itself.
//!
//! `SelectValue`'s joined-text display (`selected_texts()`) reads `SelectableContext::options`,
//! which `SelectOption` populates through a `use_effect` (`listbox.rs`'s option-registration
//! hook) — a plain `rebuild_in_place()` schedules but does not drive that effect to completion
//! outside a running app, the same documented limitation `collection.rs`'s and `date_picker.rs`'s
//! tests hit for effect-driven state. `is_empty()`/`aria-selected`, by contrast, read the
//! `values`/comparison memos directly with no effect in the path, so those stay assertable here;
//! the joined-text case is left to `tests/playwright/select.spec.ts`, which already exercises it.

use adico_primitives::select::{
    Select, SelectList, SelectMulti, SelectOption, SelectTrigger, SelectValue,
};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn ClosedFruitSelect() -> Element {
    rsx! {
        Select::<String> {
            SelectTrigger { aria_label: "Choose a fruit",
                SelectValue { placeholder: "Choose a fruit" }
            }
            SelectList { aria_label: "Fruit options",
                SelectOption::<String> { index: 0usize, value: "apple", text_value: "Apple", "Apple" }
                SelectOption::<String> { index: 1usize, value: "banana", text_value: "Banana", "Banana" }
            }
        }
    }
}

#[test]
fn closed_select_shows_placeholder_and_collapsed_trigger_with_no_listbox_markup() {
    let html = render(ClosedFruitSelect);
    assert!(html.contains("aria-expanded=false"), "{html}");
    assert!(html.contains("Choose a fruit"), "{html}");
    assert!(html.contains("data-placeholder=true"), "{html}");
    assert!(!html.contains(r#"role="listbox""#), "{html}");
    assert!(!html.contains(r#"role="option""#), "{html}");
}

#[test]
fn closed_select_trigger_has_a_stable_generated_id() {
    // `SelectTrigger` previously rendered no `id` at all; `SelectList`'s `Positioner` now
    // anchors to it, so the trigger must always render one.
    let html = render(ClosedFruitSelect);
    assert!(html.contains(r#"<button id="adico-"#), "{html}");
}

#[test]
fn closed_select_root_carries_data_state_and_a_root_id() {
    let html = render(ClosedFruitSelect);
    assert!(html.contains(r#"data-state="closed""#), "{html}");
    // The root div's generated id, distinct from the trigger's own, bounds
    // `use_outside_dismiss`'s pointer-outside check.
    assert!(html.contains(r#"<div id="adico-"#), "{html}");
}

#[component]
fn DisabledFruitSelect() -> Element {
    rsx! {
        Select::<String> {
            disabled: true,
            SelectTrigger { aria_label: "Choose a fruit",
                SelectValue { placeholder: "Choose a fruit" }
            }
            SelectList { aria_label: "Fruit options",
                SelectOption::<String> { index: 0usize, value: "apple", text_value: "Apple", "Apple" }
            }
        }
    }
}

#[test]
fn disabled_select_disables_its_trigger_and_marks_the_root() {
    let html = render(DisabledFruitSelect);
    assert!(html.contains("disabled=true"), "{html}");
    assert!(html.contains("data-disabled=true"), "{html}");
}

#[component]
fn SelectWithDefaultValue() -> Element {
    rsx! {
        Select::<String> {
            default_value: "banana".to_string(),
            SelectTrigger { aria_label: "Choose a fruit",
                SelectValue { placeholder: "Choose a fruit" }
            }
            SelectList { aria_label: "Fruit options",
                SelectOption::<String> { index: 0usize, value: "apple", text_value: "Apple", "Apple" }
                SelectOption::<String> { index: 1usize, value: "banana", text_value: "Banana", "Banana" }
            }
        }
    }
}

#[test]
fn select_with_a_default_value_stops_showing_the_placeholder() {
    let html = render(SelectWithDefaultValue);
    assert!(html.contains("data-placeholder=false"), "{html}");
}

// `SelectList` gates its listbox markup on `use_listbox_container` -> `use_animated_open`,
// whose real (`web`/`native`) implementation only flips its content-mounted signal from
// inside a `use_effect` that a plain `rebuild_in_place()` schedules but does not itself drive
// to completion outside a running app — the same, already-established precedent `menu.rs`'s
// and `date_picker.rs`'s inline tests document for this exact class of test. These run only
// on the SSR-fallback path (no `web`/`native` feature), where `use_animated_open` returns
// `open` directly with no effect involved.
#[cfg(not(any(feature = "web", feature = "native")))]
mod open_by_default {
    use super::*;
    use adico_primitives::select::{SelectGroup, SelectGroupLabel, SelectItemIndicator};

    #[component]
    fn OpenFruitSelect() -> Element {
        rsx! {
            Select::<String> {
                default_open: true,
                default_value: "banana".to_string(),
                SelectTrigger { aria_label: "Choose a fruit",
                    SelectValue { placeholder: "Choose a fruit" }
                }
                SelectList { aria_label: "Fruit options",
                    SelectGroup {
                        SelectGroupLabel { "Fruits" }
                        SelectOption::<String> { index: 0usize, value: "apple", text_value: "Apple", "Apple" }
                        SelectOption::<String> { index: 1usize, value: "banana", text_value: "Banana", "Banana",
                            SelectItemIndicator { "check" }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn open_select_renders_an_anchored_listbox_with_its_options() {
        let html = render(OpenFruitSelect);
        assert!(html.contains("aria-expanded=true"), "{html}");
        assert!(html.contains(r#"role="listbox""#), "{html}");
        assert!(html.contains(r#"data-state="open""#), "{html}");
        // `Positioner` renders `position: fixed` inline before its first JS measurement
        // resolves — this is the new positioning participation this file previously had none
        // of at all.
        assert!(html.contains("position: fixed"), "{html}");
        assert!(html.contains(r#"role="option""#), "{html}");
    }

    #[test]
    fn open_select_marks_only_the_default_value_s_option_selected() {
        let html = render(OpenFruitSelect);
        assert!(
            html.contains("aria-selected=false") && html.contains(">Apple<"),
            "{html}"
        );
        assert!(html.contains("aria-selected=true"), "{html}");
        assert!(html.contains(">Bananacheck<"), "{html}");
    }

    // `SelectGroupLabel` -> `SelectGroup`'s `aria-labelledby` wiring goes through
    // `SelectGroupContext.labeled_by`, set inside a `use_effect` — the same
    // effect-driven-state limitation `test_select.rs`'s module doc comment describes for
    // `SelectValue`'s joined text, so it isn't assertable in this harness either;
    // `select.spec.ts`'s axe scan over `[role="listbox"]` covers it in a real browser instead.
}

#[component]
fn OpenFruitSelectMulti() -> Element {
    rsx! {
        SelectMulti::<String> {
            default_open: true,
            default_values: vec!["apple".to_string(), "cherry".to_string()],
            SelectTrigger { aria_label: "Choose fruits",
                SelectValue { placeholder: "Choose fruits" }
            }
            SelectList { aria_label: "Fruit options",
                SelectOption::<String> { index: 0usize, value: "apple", text_value: "Apple", "Apple" }
                SelectOption::<String> { index: 1usize, value: "banana", text_value: "Banana", "Banana" }
                SelectOption::<String> { index: 2usize, value: "cherry", text_value: "Cherry", "Cherry" }
            }
        }
    }
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn select_multi_marks_every_default_value_s_option_selected_and_is_multiselectable() {
    let html = render(OpenFruitSelectMulti);
    assert!(html.contains("aria-multiselectable=true"), "{html}");
    assert!(
        html.contains("aria-selected=true") && html.contains(">Apple<"),
        "{html}"
    );
    assert!(
        html.contains("aria-selected=false") && html.contains(">Banana<"),
        "{html}"
    );
    assert!(
        html.contains("aria-selected=true") && html.contains(">Cherry<"),
        "{html}"
    );
}
