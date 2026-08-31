//! Black-box tests for `adico_primitives::virtual_list`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/virtual_list.rs`.
//!
//! `find_nearest_binary_search`/`default_range_extractor` are tested directly (previously
//! uncovered even by the file's own prior inline suite, which only exercised the higher-level
//! `Store`-backed functions). The six `Store`-backed tests below are moved unchanged from that
//! prior inline suite.

use adico_primitives::virtual_list::{
    VirtualItem, VirtualizerState, VirtualizerStateStoreExt, compute_measurements,
    default_range_extractor, find_nearest_binary_search, get_total_size, get_virtual_items,
    resize_item, set_scroll_offset,
};
use dioxus::prelude::*;
use std::collections::HashMap;

fn item(index: usize, start: u32, size: u32) -> VirtualItem {
    VirtualItem::new(index, index, start, size)
}

#[test]
fn find_nearest_binary_search_finds_an_exact_match() {
    let measurements = vec![item(0, 0, 50), item(1, 50, 50), item(2, 100, 50)];
    assert_eq!(find_nearest_binary_search(&measurements, 50), 1);
}

#[test]
fn find_nearest_binary_search_falls_back_to_the_item_before_an_inexact_offset() {
    let measurements = vec![item(0, 0, 50), item(1, 50, 50), item(2, 100, 50)];
    // 75 falls inside item 1's [50, 100) span, with no item starting exactly at 75.
    assert_eq!(find_nearest_binary_search(&measurements, 75), 1);
}

#[test]
fn find_nearest_binary_search_clamps_to_zero_for_an_offset_before_the_first_item() {
    let measurements = vec![item(0, 10, 50)];
    assert_eq!(find_nearest_binary_search(&measurements, 0), 0);
}

#[test]
fn default_range_extractor_applies_overscan_on_both_sides() {
    let extracted = default_range_extractor(10..20, 3, 100);
    assert_eq!(extracted, 7..=23);
}

#[test]
fn default_range_extractor_clamps_overscan_at_the_start() {
    let extracted = default_range_extractor(0..5, 3, 100);
    assert_eq!(extracted, 0..=8);
}

#[test]
fn default_range_extractor_clamps_overscan_at_the_end() {
    let extracted = default_range_extractor(95..99, 10, 100);
    assert_eq!(extracted, 85..=99);
}

#[test]
fn default_range_extractor_returns_a_single_zero_range_for_an_empty_list() {
    let extracted = default_range_extractor(0..0, 5, 0);
    assert_eq!(extracted, 0..=0);
}

/// Run a closure inside a Dioxus runtime context so that `Store`/`CopyValue` APIs are
/// available. The closure runs inside a component render.
fn with_runtime(f: impl Fn() + 'static) {
    use std::cell::Cell;
    use std::rc::Rc;

    let result = Rc::new(Cell::new(false));
    let result2 = result.clone();
    let test_fn = Rc::new(f);
    let mut dom = VirtualDom::new_with_props(
        |props: TestHarnessProps| {
            (props.test_fn)();
            props.result.set(true);
            rsx! { div {} }
        },
        TestHarnessProps {
            test_fn,
            result: result2,
        },
    );
    dom.rebuild_in_place();
    assert!(result.get(), "Test component did not run");
}

#[derive(Clone, Props)]
struct TestHarnessProps {
    test_fn: std::rc::Rc<dyn Fn()>,
    result: std::rc::Rc<std::cell::Cell<bool>>,
}

impl PartialEq for TestHarnessProps {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

fn create_test_state() -> Store<VirtualizerState> {
    Store::new(VirtualizerState {
        scroll_offset: 0,
        viewport_size: 600,
        is_scrolling: false,
        item_size_cache: HashMap::new(),
        scroll_adjustments: 0,
        stable_total_size: None,
        stable_measurement_count: None,
        deferred_adjustments: 0,
    })
}

fn make_measurements(state: &Store<VirtualizerState>) -> Vec<VirtualItem> {
    let isc = state.item_size_cache();
    let cache = isc.peek();
    compute_measurements(100, &cache, Some(&|_| 50))
}

#[test]
fn test_stable_total_size_ignored_when_count_changes() {
    with_runtime(|| {
        let state = create_test_state();
        let m = make_measurements(&state);
        set_scroll_offset(&state, &m, 1000, true);

        assert_eq!(get_total_size(&state, &m), 5000);

        let smaller = compute_measurements(4, &HashMap::new(), Some(&|_| 50));
        assert_eq!(get_total_size(&state, &smaller), 200);
    });
}

#[test]
fn test_range_clamps_stale_scroll_offset_after_count_shrinks() {
    with_runtime(|| {
        let state = create_test_state();
        let smaller = compute_measurements(4, &HashMap::new(), Some(&|_| 50));
        state.scroll_offset().set(10_000);
        state.viewport_size().set(600);

        let virtual_items = get_virtual_items(&state, &smaller, 0);
        let indexes: Vec<_> = virtual_items.iter().map(VirtualItem::index).collect();

        assert_eq!(indexes, vec![0, 1, 2, 3]);
    });
}

#[test]
fn test_resize_item_below_viewport() {
    with_runtime(|| {
        let state = create_test_state();
        let m = make_measurements(&state);
        set_scroll_offset(&state, &m, 0, false);

        let m = make_measurements(&state);
        let adjustment = resize_item(&state, &m, 50, 100);
        assert!(adjustment.is_none());
    });
}

#[test]
fn test_resize_item_above_viewport() {
    with_runtime(|| {
        let state = create_test_state();
        let m = make_measurements(&state);
        set_scroll_offset(&state, &m, 1000, false);

        let m = make_measurements(&state);
        let adjustment = resize_item(&state, &m, 5, 100);
        assert!(adjustment.is_some());
        assert_eq!(adjustment.unwrap(), 50);
    });
}

#[test]
fn test_deferred_adjustments_during_scrolling() {
    with_runtime(|| {
        let state = create_test_state();

        let m = make_measurements(&state);
        set_scroll_offset(&state, &m, 1000, true);

        let m = make_measurements(&state);
        let adjustment = resize_item(&state, &m, 5, 100);
        assert!(adjustment.is_none(), "Should not adjust during scrolling");

        let m = make_measurements(&state);
        let adjustment = resize_item(&state, &m, 3, 80);
        assert!(adjustment.is_none(), "Should not adjust during scrolling");

        let m = make_measurements(&state);
        let correction = set_scroll_offset(&state, &m, 1000, false);
        assert_eq!(
            correction,
            Some(80),
            "Should return accumulated delta: 50 + 30 = 80"
        );
    });
}

#[test]
fn test_no_deferred_adjustments_for_items_below_viewport() {
    with_runtime(|| {
        let state = create_test_state();

        let m = make_measurements(&state);
        set_scroll_offset(&state, &m, 1000, true);

        let m = make_measurements(&state);
        let adjustment = resize_item(&state, &m, 50, 100);
        assert!(adjustment.is_none());

        let m = make_measurements(&state);
        let correction = set_scroll_offset(&state, &m, 1000, false);
        assert!(
            correction.is_none(),
            "No deferred adjustment for items below viewport"
        );
    });
}
