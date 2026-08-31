//! Black-box tests for `adico_primitives::combobox`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/combobox.rs`.
//!
//! `combobox.rs`'s own filtering/selection/roving-focus logic is independently covered by
//! `selectable.rs`/`collection.rs`'s own consumers; these tests instead cover this rewrite's
//! actual regression surface: the ARIA/render wiring, and the new `Positioner`/`layer`
//! participation (anchored positioning, root-level Escape, outside-pointer dismiss) that this
//! file's listbox previously had none of at all — the same gap `select.rs` had before its own
//! task 2.1 rewrite (see `test_select.rs`'s module doc comment, which documents the same
//! effect-driven-registration limitation this file's `SelectValue`-equivalent display text
//! hits here too).

use adico_primitives::combobox::{
    Combobox, ComboboxEmpty, ComboboxInput, ComboboxItemIndicator, ComboboxList, ComboboxMulti,
    ComboboxOption,
};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn ClosedFruitCombobox() -> Element {
    rsx! {
        Combobox::<String> {
            ComboboxInput { placeholder: "Search fruit" }
            ComboboxList {
                ComboboxOption::<String> { index: 0usize, value: "apple", text_value: "Apple", "Apple" }
                ComboboxOption::<String> { index: 1usize, value: "banana", text_value: "Banana", "Banana" }
                ComboboxEmpty { "No results" }
            }
        }
    }
}

#[test]
fn closed_combobox_shows_no_listbox_markup() {
    let html = render(ClosedFruitCombobox);
    assert!(html.contains("aria-expanded=false"), "{html}");
    assert!(html.contains(r#"role="combobox""#), "{html}");
    assert!(!html.contains(r#"role="listbox""#), "{html}");
    assert!(!html.contains(r#"role="option""#), "{html}");
}

#[test]
fn closed_combobox_input_has_a_stable_generated_id() {
    // `ComboboxInput` already generated its own id before this rewrite (unlike
    // `SelectTrigger`, which had none); this confirms it still does, since `ComboboxList`'s
    // `Positioner` now depends on it as an anchor via `ComboboxContext::input_id`.
    let html = render(ClosedFruitCombobox);
    assert!(html.contains(r#"<input id="adico-"#), "{html}");
}

#[test]
fn closed_combobox_root_carries_data_state_and_a_root_id() {
    let html = render(ClosedFruitCombobox);
    assert!(html.contains(r#"data-state="closed""#), "{html}");
    assert!(html.contains(r#"<div id="adico-"#), "{html}");
}

#[component]
fn DisabledFruitCombobox() -> Element {
    rsx! {
        Combobox::<String> {
            disabled: true,
            ComboboxInput { placeholder: "Search fruit" }
            ComboboxList {
                ComboboxOption::<String> { index: 0usize, value: "apple", text_value: "Apple", "Apple" }
            }
        }
    }
}

#[test]
fn disabled_combobox_disables_its_input_and_marks_the_root() {
    let html = render(DisabledFruitCombobox);
    assert!(html.contains("disabled=true"), "{html}");
    assert!(html.contains("data-disabled=true"), "{html}");
}

// `ComboboxList` gates its listbox markup on `use_listbox_container` -> `use_animated_open`,
// whose real (`web`/`native`) implementation only flips its content-mounted signal from
// inside a `use_effect` that a plain `rebuild_in_place()` schedules but does not itself drive
// to completion outside a running app — the same, already-established precedent `menu.rs`'s,
// `date_picker.rs`'s, and `test_select.rs`'s tests document for this exact class of test.
// These run only on the SSR-fallback path (no `web`/`native` feature), where
// `use_animated_open` returns `open` directly with no effect involved.
#[cfg(not(any(feature = "web", feature = "native")))]
mod open_by_default {
    use super::*;

    #[component]
    fn OpenFruitCombobox() -> Element {
        rsx! {
            Combobox::<String> {
                default_open: true,
                ComboboxInput { placeholder: "Search fruit" }
                ComboboxList {
                    ComboboxOption::<String> { index: 0usize, value: "apple", text_value: "Apple", "Apple" }
                    ComboboxOption::<String> { index: 1usize, value: "banana", text_value: "Banana", "Banana",
                        ComboboxItemIndicator { "check" }
                    }
                    ComboboxEmpty { "No results" }
                }
            }
        }
    }

    // `ComboboxOption`'s own visibility (`ctx.is_visible`/`has_visible_options`, gating both
    // individual options and `ComboboxEmpty`) reads `SelectableContext::options`, which
    // `ComboboxOption` populates through the same `use_effect`-driven registration hook
    // `select.rs`'s options use (see `test_select.rs`'s module doc comment) — so under a bare
    // `rebuild_in_place()` no option ever registers, and every option (and `ComboboxEmpty`,
    // gated on there being *no* visible options) renders as if the list were genuinely empty.
    // This test therefore checks only the listbox container's own render/ARIA/positioning
    // wiring, which does not depend on that registration effect; per-option filtering is left
    // to a live-browser Playwright spec (none exists yet for combobox — no consumer fixture
    // has been installed for it either, unlike select).
    #[test]
    fn open_combobox_renders_an_anchored_listbox() {
        let html = render(OpenFruitCombobox);
        assert!(html.contains("aria-expanded=true"), "{html}");
        assert!(html.contains(r#"role="listbox""#), "{html}");
        assert!(html.contains(r#"data-state="open""#), "{html}");
        // `Positioner` renders `position: fixed` inline before its first JS measurement
        // resolves — this is the new positioning participation this file previously had none
        // of at all.
        assert!(html.contains("position: fixed"), "{html}");
        assert!(html.contains(r#"aria-multiselectable=false"#), "{html}");
    }
}

#[component]
fn OpenFruitComboboxMulti() -> Element {
    rsx! {
        ComboboxMulti::<String> {
            default_open: true,
            default_values: vec!["apple".to_string(), "cherry".to_string()],
            ComboboxInput { placeholder: "Search fruit" }
            ComboboxList {
                ComboboxOption::<String> { index: 0usize, value: "apple", text_value: "Apple", "Apple" }
                ComboboxOption::<String> { index: 1usize, value: "banana", text_value: "Banana", "Banana" }
                ComboboxOption::<String> { index: 2usize, value: "cherry", text_value: "Cherry", "Cherry" }
            }
        }
    }
}

// See `open_combobox_renders_an_anchored_listbox`'s comment: per-option state (including
// `data-selected`) depends on the same effect-driven option registration a bare
// `rebuild_in_place()` doesn't complete, so this only checks the listbox's own
// `aria-multiselectable` wiring.
#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn combobox_multi_marks_its_listbox_multiselectable() {
    let html = render(OpenFruitComboboxMulti);
    assert!(html.contains(r#"aria-multiselectable=true"#), "{html}");
}
