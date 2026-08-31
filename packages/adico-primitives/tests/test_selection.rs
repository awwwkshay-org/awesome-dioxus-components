//! Black-box tests for `adico_primitives::selection`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/selection.rs`.

use adico_primitives::selection::{
    OptionState, RcPartialEqValue, option_text_value, remove_option_state, selected_text,
    sync_option_state,
};

fn option(id: &str, index: usize) -> OptionState {
    OptionState {
        id: id.to_string(),
        index,
        value: RcPartialEqValue::new(id.to_string()),
        text_value: id.to_string(),
    }
}

fn ids(options: &[OptionState]) -> Vec<&str> {
    options
        .iter()
        .map(|option| option.text_value.as_str())
        .collect()
}

fn indices(options: &[OptionState]) -> Vec<usize> {
    options.iter().map(|option| option.index).collect()
}

#[test]
fn sync_option_state_keeps_sorted_order() {
    let mut options = vec![option("a", 0), option("b", 1), option("c", 2)];

    sync_option_state(&mut options, option("d", 3));

    assert_eq!(ids(&options), ["a", "b", "c", "d"]);
    assert_eq!(indices(&options), [0, 1, 2, 3]);
}

#[test]
fn sync_option_state_updates_matching_id_and_reorders() {
    let mut options = vec![option("a", 0), option("b", 1), option("c", 2)];

    sync_option_state(&mut options, option("b", 3));

    assert_eq!(ids(&options), ["a", "c", "b"]);
    assert_eq!(indices(&options), [0, 2, 3]);
}

#[test]
fn removing_stale_option_does_not_remove_option_that_moved_to_same_index() {
    let mut options = vec![option("a", 0), option("b", 1)];

    sync_option_state(&mut options, option("b", 0));
    remove_option_state(&mut options, "a");

    assert_eq!(ids(&options), ["b"]);
    assert_eq!(indices(&options), [0]);
}

#[test]
fn option_text_value_prefers_an_explicit_text_value_over_the_option_s_own_value() {
    let value = 42usize;
    assert_eq!(
        option_text_value(&value, Some("forty-two".to_string()), "TestOption"),
        "forty-two"
    );
}

#[test]
fn option_text_value_falls_back_to_a_string_value_when_no_text_value_is_given() {
    let value = "Apple".to_string();
    assert_eq!(option_text_value(&value, None, "TestOption"), "Apple");
}

#[test]
fn option_text_value_falls_back_to_a_str_value_when_no_text_value_is_given() {
    let value: &str = "Banana";
    assert_eq!(option_text_value(&value, None, "TestOption"), "Banana");
}

#[test]
fn option_text_value_is_empty_for_a_non_string_value_with_no_text_value() {
    // This case also logs a `tracing::warn!` (component_name misconfiguration); the returned
    // value is what the caller ends up rendering, which is what matters here.
    let value = 42usize;
    assert_eq!(option_text_value(&value, None, "TestOption"), "");
}

#[test]
fn selected_text_joins_matching_option_labels_in_selection_order() {
    let options = vec![option("a", 0), option("b", 1), option("c", 2)];
    let values = [options[2].value.clone(), options[0].value.clone()];

    assert_eq!(
        selected_text(values.iter(), &options),
        Some("c, a".to_string())
    );
}

#[test]
fn selected_text_ignores_a_value_with_no_matching_option() {
    let options = vec![option("a", 0)];
    let unmatched = RcPartialEqValue::new("missing".to_string());

    assert_eq!(selected_text([&unmatched], &options), None);
}

#[test]
fn selected_text_is_none_for_no_selected_values() {
    let options = vec![option("a", 0)];

    assert_eq!(selected_text(std::iter::empty(), &options), None);
}
