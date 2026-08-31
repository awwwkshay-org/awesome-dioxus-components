//! Black-box tests for `adico_primitives::collection`. `register_item`/`unregister_item`/
//! `CollectionItemState` are widened to `pub` solely so this external test crate can build
//! fixtures directly, per this repo's test-placement convention (see
//! `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test lives
//! under `packages/adico-primitives/tests/`, never as an inline `#[cfg(test)]` module in
//! `src/*.rs`.

use adico_primitives::collection::{
    CollectionItemState, CollectionOptions, CollectionPlacement, CollectionState, Orientation,
};
use adico_primitives::direction::Direction;
use dioxus::prelude::*;

fn item(index: usize) -> CollectionItemState {
    CollectionItemState {
        index,
        key: None,
        disabled: false,
        hidden: false,
        selected: false,
    }
}

fn disabled_item(index: usize) -> CollectionItemState {
    CollectionItemState {
        disabled: true,
        ..item(index)
    }
}

fn keyed_item(index: usize, key: &str) -> CollectionItemState {
    CollectionItemState {
        index,
        key: Some(key.to_string()),
        disabled: false,
        hidden: false,
        selected: false,
    }
}

/// Renders `render` inside a throwaway component and returns the resulting SSR HTML, so tests
/// can inspect whatever string `render` computes from a `CollectionState`.
fn render_str(render: impl Fn() -> String + 'static) -> String {
    #[derive(Clone)]
    struct Harness(std::rc::Rc<dyn Fn() -> String>);

    thread_local! {
        static HARNESS: std::cell::RefCell<Option<Harness>> = const { std::cell::RefCell::new(None) };
    }

    HARNESS.with(|cell| {
        *cell.borrow_mut() = Some(Harness(std::rc::Rc::new(render)));
    });

    #[component]
    fn Root() -> Element {
        let result = HARNESS.with(|cell| (cell.borrow().as_ref().unwrap().0)());
        rsx! { "{result}" }
    }

    let mut dom = VirtualDom::new(Root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn render_focused_index(render: impl Fn() -> Option<usize> + 'static) -> String {
    render_str(move || format!("{:?}", render()))
}

fn three_item_collection() -> CollectionState {
    let mut collection = CollectionState::new(
        ReadSignal::new(Signal::new(false)),
        CollectionOptions::default(),
    );
    collection.register_item(item(0));
    collection.register_item(item(1));
    collection.register_item(item(2));
    collection.set_focus(Some(0));
    collection
}

/// A 3x3 grid in row-major order:
/// ```text
/// 0 1 2
/// 3 4 5
/// 6 7 8
/// ```
fn grid_collection() -> CollectionState {
    let mut collection = CollectionState::new(
        ReadSignal::new(Signal::new(false)),
        CollectionOptions::default(),
    );
    for index in 0..9 {
        collection.register_item(item(index));
    }
    collection.set_focus(Some(4));
    collection
}

fn five_item_collection() -> CollectionState {
    let mut collection = CollectionState::new(
        ReadSignal::new(Signal::new(false)),
        CollectionOptions::default(),
    );
    for index in 0..5 {
        collection.register_item(item(index));
    }
    collection.set_focus(Some(0));
    collection
}

// -- navigate_key ------------------------------------------------------------------------

#[test]
fn vertical_arrow_down_moves_to_next() {
    let html = render_focused_index(|| {
        let mut collection = three_item_collection();
        collection.navigate_key(Key::ArrowDown, Orientation::Vertical, Direction::Ltr);
        collection.focused_index()
    });
    assert!(html.contains("Some(1)"), "{html}");
}

#[test]
fn vertical_arrow_up_moves_to_previous() {
    let html = render_focused_index(|| {
        let mut collection = three_item_collection();
        collection.set_focus(Some(1));
        collection.navigate_key(Key::ArrowUp, Orientation::Vertical, Direction::Ltr);
        collection.focused_index()
    });
    assert!(html.contains("Some(0)"), "{html}");
}

#[test]
fn vertical_orientation_ignores_left_right() {
    let html = render_focused_index(|| {
        let mut collection = three_item_collection();
        collection.navigate_key(Key::ArrowRight, Orientation::Vertical, Direction::Ltr);
        collection.focused_index()
    });
    assert!(html.contains("Some(0)"), "{html}");
}

#[test]
fn horizontal_ltr_arrow_right_moves_to_next() {
    let html = render_focused_index(|| {
        let mut collection = three_item_collection();
        collection.navigate_key(Key::ArrowRight, Orientation::Horizontal, Direction::Ltr);
        collection.focused_index()
    });
    assert!(html.contains("Some(1)"), "{html}");
}

#[test]
fn horizontal_rtl_flips_arrow_right_to_previous() {
    let html = render_focused_index(|| {
        let mut collection = three_item_collection();
        collection.set_focus(Some(1));
        collection.navigate_key(Key::ArrowRight, Orientation::Horizontal, Direction::Rtl);
        collection.focused_index()
    });
    assert!(html.contains("Some(0)"), "{html}");
}

#[test]
fn horizontal_rtl_flips_arrow_left_to_next() {
    let html = render_focused_index(|| {
        let mut collection = three_item_collection();
        collection.navigate_key(Key::ArrowLeft, Orientation::Horizontal, Direction::Rtl);
        collection.focused_index()
    });
    assert!(html.contains("Some(1)"), "{html}");
}

#[test]
fn home_and_end_are_direction_independent() {
    let end_html = render_focused_index(|| {
        let mut collection = three_item_collection();
        collection.navigate_key(Key::End, Orientation::Horizontal, Direction::Rtl);
        collection.focused_index()
    });
    assert!(end_html.contains("Some(2)"), "{end_html}");

    let home_html = render_focused_index(|| {
        let mut collection = three_item_collection();
        collection.set_focus(Some(2));
        collection.navigate_key(Key::Home, Orientation::Horizontal, Direction::Rtl);
        collection.focused_index()
    });
    assert!(home_html.contains("Some(0)"), "{home_html}");
}

#[test]
fn unhandled_key_returns_false_and_does_not_move_focus() {
    let html = render_focused_index(|| {
        let mut collection = three_item_collection();
        let handled = collection.navigate_key(
            Key::Character("a".to_string()),
            Orientation::Vertical,
            Direction::Ltr,
        );
        assert!(!handled);
        collection.focused_index()
    });
    assert!(html.contains("Some(0)"), "{html}");
}

// -- navigate_grid_key -------------------------------------------------------------------

#[test]
fn arrow_down_moves_one_row() {
    let html = render_focused_index(|| {
        let mut collection = grid_collection();
        collection.navigate_grid_key(Key::ArrowDown, 3, Direction::Ltr);
        collection.focused_index()
    });
    assert!(html.contains("Some(7)"), "{html}");
}

#[test]
fn arrow_up_moves_one_row() {
    let html = render_focused_index(|| {
        let mut collection = grid_collection();
        collection.navigate_grid_key(Key::ArrowUp, 3, Direction::Ltr);
        collection.focused_index()
    });
    assert!(html.contains("Some(1)"), "{html}");
}

#[test]
fn arrow_up_from_top_row_does_not_move() {
    let html = render_focused_index(|| {
        let mut collection = grid_collection();
        collection.set_focus(Some(1));
        collection.navigate_grid_key(Key::ArrowUp, 3, Direction::Ltr);
        collection.focused_index()
    });
    assert!(html.contains("Some(1)"), "{html}");
}

#[test]
fn ltr_arrow_right_moves_one_column() {
    let html = render_focused_index(|| {
        let mut collection = grid_collection();
        collection.navigate_grid_key(Key::ArrowRight, 3, Direction::Ltr);
        collection.focused_index()
    });
    assert!(html.contains("Some(5)"), "{html}");
}

#[test]
fn rtl_flips_arrow_right_to_the_previous_column() {
    let html = render_focused_index(|| {
        let mut collection = grid_collection();
        collection.navigate_grid_key(Key::ArrowRight, 3, Direction::Rtl);
        collection.focused_index()
    });
    assert!(html.contains("Some(3)"), "{html}");
}

#[test]
fn disabled_target_cell_is_not_focused() {
    let html = render_focused_index(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(false)),
            CollectionOptions::default(),
        );
        for index in 0..9 {
            if index == 7 {
                collection.register_item(disabled_item(index));
            } else {
                collection.register_item(item(index));
            }
        }
        collection.set_focus(Some(4));
        collection.navigate_grid_key(Key::ArrowDown, 3, Direction::Ltr);
        collection.focused_index()
    });
    assert!(html.contains("Some(4)"), "{html}");
}

#[test]
fn home_and_end_go_to_the_grid_boundaries() {
    let end_html = render_focused_index(|| {
        let mut collection = grid_collection();
        collection.navigate_grid_key(Key::End, 3, Direction::Ltr);
        collection.focused_index()
    });
    assert!(end_html.contains("Some(8)"), "{end_html}");

    let home_html = render_focused_index(|| {
        let mut collection = grid_collection();
        collection.navigate_grid_key(Key::Home, 3, Direction::Ltr);
        collection.focused_index()
    });
    assert!(home_html.contains("Some(0)"), "{home_html}");
}

#[test]
fn zero_columns_is_unhandled_and_does_not_move_focus() {
    let html = render_focused_index(|| {
        let mut collection = grid_collection();
        let handled = collection.navigate_grid_key(Key::ArrowDown, 0, Direction::Ltr);
        assert!(!handled);
        collection.focused_index()
    });
    assert!(html.contains("Some(4)"), "{html}");
}

// -- roving_tabindex ----------------------------------------------------------------------

#[test]
fn unavailable_index_is_always_negative_one() {
    let html = render_str(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(true)),
            CollectionOptions::default(),
        );
        collection.register_item(item(0));
        collection.roving_tabindex(5).to_string()
    });
    assert!(html.contains("-1"), "{html}");
}

#[test]
fn non_looping_collection_makes_every_available_item_a_tab_stop() {
    let html = render_str(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(false)),
            CollectionOptions::default(),
        );
        collection.register_item(item(0));
        collection.register_item(item(1));
        collection.set_focus(Some(1));
        format!(
            "{}/{}",
            collection.roving_tabindex(0),
            collection.roving_tabindex(1)
        )
    });
    assert!(html.contains("0/0"), "{html}");
}

#[test]
fn no_focus_or_selection_falls_back_to_first_available() {
    let html = render_str(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(true)),
            CollectionOptions::default(),
        );
        collection.register_item(item(0));
        collection.register_item(item(1));
        format!(
            "{}/{}",
            collection.roving_tabindex(0),
            collection.roving_tabindex(1)
        )
    });
    assert!(html.contains("0/-1"), "{html}");
}

#[test]
fn tabbable_when_empty_makes_every_available_item_a_tab_stop() {
    let html = render_str(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(true)),
            CollectionOptions {
                tabbable_when_empty: true,
            },
        );
        collection.register_item(item(0));
        collection.register_item(item(1));
        format!(
            "{}/{}",
            collection.roving_tabindex(0),
            collection.roving_tabindex(1)
        )
    });
    assert!(html.contains("0/0"), "{html}");
}

#[test]
fn selected_item_anchors_tabindex_when_nothing_focused() {
    let html = render_str(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(true)),
            CollectionOptions::default(),
        );
        collection.register_item(item(0));
        collection.register_item(CollectionItemState {
            selected: true,
            ..item(1)
        });
        format!(
            "{}/{}",
            collection.roving_tabindex(0),
            collection.roving_tabindex(1)
        )
    });
    assert!(html.contains("-1/0"), "{html}");
}

#[test]
fn recent_focus_takes_precedence_over_selection() {
    let html = render_str(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(true)),
            CollectionOptions::default(),
        );
        collection.register_item(item(0));
        collection.register_item(CollectionItemState {
            selected: true,
            ..item(1)
        });
        // Focus item 0, then clear focus without clearing `recent`.
        collection.set_focus(Some(0));
        collection.clear_focus();
        format!(
            "{}/{}",
            collection.roving_tabindex(0),
            collection.roving_tabindex(1)
        )
    });
    assert!(html.contains("0/-1"), "{html}");
}

// -- focus_next_matching / focus_prev_matching / recent_focus_or_default ------------------

#[test]
fn focus_next_matching_skips_non_matching_items() {
    let html = render_focused_index(|| {
        let mut collection = five_item_collection();
        // Only even indices match; starting from 0 the next match is 2.
        collection.focus_next_matching(|index| index % 2 == 0);
        collection.focused_index()
    });
    assert!(html.contains("Some(2)"), "{html}");
}

#[test]
fn focus_prev_matching_skips_non_matching_items() {
    let html = render_focused_index(|| {
        let mut collection = five_item_collection();
        collection.set_focus(Some(4));
        collection.focus_prev_matching(|index| index % 2 == 0);
        collection.focused_index()
    });
    assert!(html.contains("Some(2)"), "{html}");
}

#[test]
fn focus_next_matching_with_no_match_clears_focus() {
    let html = render_focused_index(|| {
        let mut collection = five_item_collection();
        collection.focus_next_matching(|_| false);
        collection.focused_index()
    });
    assert!(html.contains("None"), "{html}");
}

#[test]
fn recent_focus_or_default_prefers_recent_then_selected_then_first() {
    // Recent focus, still available.
    let html = render_focused_index(|| {
        let collection = five_item_collection();
        Some(collection.recent_focus_or_default())
    });
    assert!(html.contains("Some(0)"), "{html}");

    // No recent focus: falls back to the selected item.
    let html = render_focused_index(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(false)),
            CollectionOptions::default(),
        );
        collection.register_item(item(0));
        collection.register_item(CollectionItemState {
            selected: true,
            ..item(1)
        });
        Some(collection.recent_focus_or_default())
    });
    assert!(html.contains("Some(1)"), "{html}");

    // Neither recent nor selected: falls back to the first available item.
    let html = render_focused_index(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(false)),
            CollectionOptions::default(),
        );
        collection.register_item(item(0));
        collection.register_item(item(1));
        Some(collection.recent_focus_or_default())
    });
    assert!(html.contains("Some(0)"), "{html}");
}

// -- set_focus_key / focused_key / register-unregister reindexing -------------------------

#[test]
fn set_focus_key_moves_focus_to_the_matching_item() {
    let html = render_str(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(false)),
            CollectionOptions::default(),
        );
        collection.register_item(keyed_item(0, "a"));
        collection.register_item(keyed_item(1, "b"));
        collection.set_focus_key(Some("b".to_string()));
        format!(
            "{:?}/{}",
            collection.focused_index(),
            collection.focused_key().unwrap_or_default()
        )
    });
    assert!(html.contains("Some(1)/b"), "{html}");
}

#[test]
fn set_focus_key_unknown_key_clears_focus() {
    let html = render_str(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(false)),
            CollectionOptions::default(),
        );
        collection.register_item(keyed_item(0, "a"));
        collection.set_focus_key(Some("missing".to_string()));
        format!("{:?}", collection.focused_index())
    });
    assert!(html.contains("None"), "{html}");
}

#[test]
fn focus_follows_a_reindexed_key_on_reregistration() {
    let html = render_str(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(false)),
            CollectionOptions::default(),
        );
        collection.register_item(keyed_item(0, "a"));
        collection.register_item(keyed_item(1, "b"));
        collection.set_focus_key(Some("b".to_string()));
        // "b" reorders from index 1 to index 0 (e.g. a list reorder). This is the same
        // identity (same key), so it is re-registered directly, without an intervening
        // unregister — matching how `use_item`'s per-render effect actually drives it.
        // Focus should follow the key to its new index.
        collection.register_item(keyed_item(0, "b"));
        format!("{:?}", collection.focused_index())
    });
    assert!(html.contains("Some(0)"), "{html}");
}

#[test]
fn focus_clears_when_the_focused_item_becomes_disabled() {
    let html = render_str(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(false)),
            CollectionOptions::default(),
        );
        collection.register_item(keyed_item(0, "a"));
        collection.set_focus_key(Some("a".to_string()));
        // Re-registering the same identity as disabled must clear focus.
        collection.register_item(CollectionItemState {
            disabled: true,
            ..keyed_item(0, "a")
        });
        format!("{:?}", collection.focused_index())
    });
    assert!(html.contains("None"), "{html}");
}

#[test]
fn unregistering_the_focused_item_clears_focus() {
    let html = render_str(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(false)),
            CollectionOptions::default(),
        );
        collection.register_item(keyed_item(0, "a"));
        collection.register_item(keyed_item(1, "b"));
        collection.set_focus_key(Some("a".to_string()));
        collection.unregister_item(&keyed_item(0, "a"));
        format!("{:?}", collection.focused_index())
    });
    assert!(html.contains("None"), "{html}");
}

// -- try_focus_placement -------------------------------------------------------------------
//
// `use_deferred_collection_focus` wraps this same call in a `use_effect`, so it isn't itself
// testable through `rebuild_in_place()` alone: matching `date_picker.rs`/`menu.rs`'s
// already-established precedent for this exact class of hook (an effect that a plain
// `rebuild_in_place()` schedules but does not drive to completion outside a running app),
// this tests the underlying state transition the effect performs instead.

#[test]
fn try_focus_placement_first_focuses_the_first_available_item() {
    let html = render_focused_index(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(false)),
            CollectionOptions::default(),
        );
        collection.register_item(item(0));
        collection.register_item(item(1));
        let placed = collection.try_focus_placement(CollectionPlacement::First);
        assert!(placed);
        collection.focused_index()
    });
    assert!(html.contains("Some(0)"), "{html}");
}

#[test]
fn try_focus_placement_last_focuses_the_last_available_item() {
    let html = render_focused_index(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(false)),
            CollectionOptions::default(),
        );
        collection.register_item(item(0));
        collection.register_item(item(1));
        let placed = collection.try_focus_placement(CollectionPlacement::Last);
        assert!(placed);
        collection.focused_index()
    });
    assert!(html.contains("Some(1)"), "{html}");
}

#[test]
fn try_focus_placement_on_an_empty_collection_returns_false() {
    let html = render_focused_index(|| {
        let mut collection = CollectionState::new(
            ReadSignal::new(Signal::new(false)),
            CollectionOptions::default(),
        );
        let placed = collection.try_focus_placement(CollectionPlacement::First);
        assert!(!placed);
        collection.focused_index()
    });
    assert!(html.contains("None"), "{html}");
}
