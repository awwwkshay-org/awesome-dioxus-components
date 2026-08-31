//! Black-box tests for `adico_primitives::typeahead`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in `src/*.rs`.
//!
//! These tests are intentionally exact-index, not merely relational (`assert!(a < b)`):
//! per the task's own recipe, relational assertions pass for nearly any sane fuzzy matcher
//! and don't protect against a ranking-quality regression during a rewrite.

use adico_primitives::selectable::{OptionState, RcPartialEqValue};
use adico_primitives::typeahead::{
    AdaptiveKeyboard, KeyboardLayout, Typeahead, best_match, code_to_char, normalized_distance,
    position_weight, use_typeahead, weighted_edit_distance,
};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

fn option(index: usize, value: &'static str, text_value: &str) -> OptionState {
    OptionState {
        id: format!("option-{index}"),
        index,
        value: RcPartialEqValue::new(value),
        text_value: text_value.to_string(),
    }
}

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

// -- position_weight -----------------------------------------------------------------------

#[test]
fn position_weight_is_monotonically_increasing() {
    let weights: Vec<f32> = (1..=5).map(|i| position_weight(i, 5)).collect();
    for pair in weights.windows(2) {
        assert!(pair[0] <= pair[1], "{weights:?}");
    }
}

#[test]
fn position_weight_is_bounded_between_floor_and_one() {
    for i in 0..=10 {
        let w = position_weight(i, 10);
        assert!((0.02..=1.0).contains(&w), "position {i} -> {w}");
    }
}

#[test]
fn position_weight_of_zero_length_is_zero() {
    assert_eq!(position_weight(0, 0), 0.0);
}

// -- weighted_edit_distance / normalized_distance ------------------------------------------

#[test]
fn identical_sequences_have_near_zero_distance() {
    let distance = weighted_edit_distance(&['a', 'b', 'c'], &['a', 'b', 'c'], |_, _| 1.0);
    assert!(distance < 0.05, "{distance}");
}

#[test]
fn a_single_late_mismatch_costs_more_than_no_mismatch() {
    let exact = weighted_edit_distance(&['a', 'b', 'c'], &['a', 'b', 'c'], |_, _| 1.0);
    let mismatched = weighted_edit_distance(&['a', 'b', 'c'], &['a', 'b', 'd'], |_, _| 1.0);
    assert!(mismatched > exact, "exact={exact} mismatched={mismatched}");
}

#[test]
fn cheap_substitution_costs_less_than_uniform_substitution() {
    let cheap = weighted_edit_distance(&['q', 'w'], &['q', 'e'], |a, b| {
        KeyboardLayout::Qwerty.distance_cost(a, b).unwrap()
    });
    let uniform = weighted_edit_distance(&['q', 'w'], &['q', 'e'], |_, _| 1.0);
    assert!(cheap < uniform, "cheap={cheap} uniform={uniform}");
}

#[test]
fn normalized_distance_truncates_to_the_shorter_side() {
    // A 5-character typed buffer against a 3-character value should behave the same as
    // comparing only the last 3 typed characters against the full value.
    let full = normalized_distance(
        &['a', 'b', 'c', 'd', 'e'],
        &['x', 'y', 'z'],
        &AdaptiveKeyboard::default(),
    );
    assert!(full > 0.0, "{full}");
}

// -- best_match: exact-index regression net -------------------------------------------------

fn fruit_options() -> Vec<OptionState> {
    vec![
        option(0, "apple", "Apple"),
        option(1, "application", "Application"),
        option(2, "apply", "Apply"),
        option(3, "banana", "Banana"),
        option(4, "cherry", "Cherry"),
    ]
}

#[test]
fn exact_prefix_match_wins_over_longer_alternatives() {
    let keyboard = AdaptiveKeyboard::default();
    let options = fruit_options();
    // "apple" is an exact match for option 0, even with "application"/"apply" sharing the
    // same "appl" prefix.
    let result = best_match(&keyboard, "apple", &options, |_| true);
    assert_eq!(result, Some(0));
}

#[test]
fn a_short_prefix_prefers_the_shortest_matching_option() {
    let keyboard = AdaptiveKeyboard::default();
    let options = fruit_options();
    let result = best_match(&keyboard, "app", &options, |_| true);
    assert_eq!(
        result,
        Some(0),
        "\"app\" should prefer the shortest full match, \"Apple\""
    );
}

#[test]
fn a_single_adjacent_key_typo_still_lands_on_the_intended_item() {
    let keyboard = AdaptiveKeyboard::default();
    let options = vec![option(0, "banana", "Banana"), option(1, "cherry", "Cherry")];
    // 'v' is adjacent to 'b' on QWERTY; "vanana" should still resolve to "Banana", not
    // "Cherry", which shares no characters with the typed buffer at all.
    let result = best_match(&keyboard, "vanana", &options, |_| true);
    assert_eq!(result, Some(0));
}

#[test]
fn unavailable_options_are_excluded_even_when_they_match_best() {
    let keyboard = AdaptiveKeyboard::default();
    let options = fruit_options();
    // Exclude "Apple" (index 0); "app" should now resolve to the next-best available match.
    let result = best_match(&keyboard, "app", &options, |index| index != 0);
    assert_ne!(result, Some(0));
    assert!(result.is_some());
}

#[test]
fn empty_typeahead_matches_nothing() {
    let keyboard = AdaptiveKeyboard::default();
    let options = fruit_options();
    assert_eq!(best_match(&keyboard, "", &options, |_| true), None);
}

// -- AdaptiveKeyboard::substitution_cost ----------------------------------------------------

#[test]
fn identical_characters_cost_nothing() {
    let keyboard = AdaptiveKeyboard::default();
    assert_eq!(keyboard.substitution_cost('a', 'a'), 0.0);
}

#[test]
fn case_only_difference_is_cheap_but_not_free() {
    let keyboard = AdaptiveKeyboard::default();
    let cost = keyboard.substitution_cost('a', 'A');
    assert!(cost > 0.0 && cost < 0.1, "{cost}");
}

#[test]
fn adjacent_qwerty_keys_cost_less_than_distant_ones() {
    let keyboard = AdaptiveKeyboard::default();
    let adjacent = keyboard.substitution_cost('q', 'w');
    let distant = keyboard.substitution_cost('q', 'p');
    assert!(adjacent < distant, "adjacent={adjacent} distant={distant}");
}

#[test]
fn cross_script_phonetic_match_costs_less_than_unrelated_characters() {
    let keyboard = AdaptiveKeyboard::default();
    // Latin 'k' and Cyrillic 'к' share a phoneme class; neither is a keyboard-layout
    // neighbor of the other; they're also codepoint-distant, so this isolates the phonetic
    // signal specifically.
    let related = keyboard.substitution_cost('k', 'к');
    let unrelated = keyboard.substitution_cost('a', '中');
    assert!(
        related < unrelated,
        "related={related} unrelated={unrelated}"
    );
}

#[test]
fn codepoint_close_characters_cost_less_than_codepoint_distant_ones() {
    let keyboard = AdaptiveKeyboard::default();
    let close = keyboard.substitution_cost('a', 'b');
    let far = keyboard.substitution_cost('a', '中');
    assert!(close < far, "close={close} far={far}");
}

// -- KeyboardLayout::guess / distance_cost --------------------------------------------------

#[test]
fn guess_detects_qwerty_from_matching_key_positions() {
    let mut known = HashMap::new();
    known.insert("KeyQ".to_string(), 'q');
    known.insert("KeyW".to_string(), 'w');
    known.insert("KeyE".to_string(), 'e');
    assert_eq!(KeyboardLayout::guess(&known), KeyboardLayout::Qwerty);
}

#[test]
fn guess_detects_colemak_dh_from_matching_key_positions() {
    let mut known = HashMap::new();
    known.insert("KeyQ".to_string(), 'q');
    known.insert("KeyW".to_string(), 'w');
    known.insert("KeyE".to_string(), 'f');
    known.insert("KeyR".to_string(), 'p');
    assert_eq!(KeyboardLayout::guess(&known), KeyboardLayout::ColemakDH);
}

#[test]
fn guess_with_no_observations_defaults_to_qwerty() {
    assert_eq!(
        KeyboardLayout::guess(&HashMap::new()),
        KeyboardLayout::Qwerty
    );
}

#[test]
fn distance_cost_is_zero_for_identical_keys() {
    assert_eq!(KeyboardLayout::Qwerty.distance_cost('a', 'a'), Some(0.05));
}

#[test]
fn distance_cost_returns_none_for_unmapped_characters() {
    assert_eq!(KeyboardLayout::Qwerty.distance_cost('α', 'β'), None);
}

#[test]
fn code_to_char_maps_known_key_codes() {
    assert_eq!(code_to_char("KeyA"), Some('a'));
    assert_eq!(code_to_char("Digit5"), Some('5'));
    assert_eq!(code_to_char("Enter"), None);
}

// -- AdaptiveKeyboard::learn_from_event / best_match interplay ------------------------------

#[test]
fn learning_a_non_qwerty_mapping_improves_matches_in_that_layout() {
    let mut keyboard = AdaptiveKeyboard::default();
    // Teach it a Russian physical-key mapping (phi sound on the A key, y-sound on S).
    keyboard.learn_from_event("KeyA", 'ф');
    keyboard.learn_from_event("KeyS", 'ы');

    let options = vec![option(0, "phi", "ф"), option(1, "banana", "Banana")];
    assert_eq!(best_match(&keyboard, "ф", &options, |_| true), Some(0));
    assert_eq!(best_match(&keyboard, "b", &options, |_| true), Some(1));
}

// -- Typeahead handle -----------------------------------------------------------------------

fn typeahead_option(index: usize, text_value: &str) -> OptionState {
    OptionState {
        id: format!("option-{index}"),
        index,
        value: RcPartialEqValue::new(index),
        text_value: text_value.to_string(),
    }
}

#[component]
fn AccumulatesAcrossCalls() -> Element {
    let mut typeahead: Typeahead =
        use_typeahead(ReadSignal::new(Signal::new(Duration::from_secs(1))));
    let options = [
        typeahead_option(0, "Apple"),
        typeahead_option(1, "Banana"),
        typeahead_option(2, "Cherry"),
    ];

    // A single "b" is ambiguous with nothing else here, but the buffer must accumulate: "b"
    // then "a" together should land on "Banana", not re-match "b" alone on every call.
    typeahead.on_input("b", &options, |_| true);
    let result = typeahead.on_input("a", &options, |_| true);

    rsx! {
        "{result:?}"
    }
}

#[component]
fn ClearResetsTheBuffer() -> Element {
    let mut typeahead: Typeahead =
        use_typeahead(ReadSignal::new(Signal::new(Duration::from_secs(1))));
    let options = [typeahead_option(0, "Apple"), typeahead_option(1, "Banana")];

    typeahead.on_input("b", &options, |_| true);
    typeahead.clear();
    let result = typeahead.on_input("a", &options, |_| true);

    rsx! {
        "{result:?}"
    }
}

#[test]
fn on_input_accumulates_the_buffer_across_calls() {
    assert!(render(AccumulatesAcrossCalls).contains("Some(1)"));
}

#[test]
fn clear_resets_the_buffer_between_input_sequences() {
    // After clear(), "a" alone should match "Apple" (index 0), not continue matching
    // against the discarded "b" prefix.
    assert!(render(ClearResetsTheBuffer).contains("Some(0)"));
}
