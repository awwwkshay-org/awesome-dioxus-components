//! Black-box tests for `adico_primitives::hover_intent`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/hover_intent.rs`.
//!
//! Per task 6.1 (`openspec/changes/deduplicate-primitives`): the plan's original premise --
//! that `navigation_menu`'s "switching between already-open items applies zero delay"
//! behavior already had test coverage somewhere to port -- didn't hold (neither
//! `packages/adico-primitives/tests/` nor `navigation_menu.rs`'s own inline `mod tests`
//! exercised `request_open`'s delay-selection logic at all). This file's coverage is new,
//! not ported.
//!
//! What's tested here is the primitive's generation-counter supersede semantics, made
//! deterministic by using `delay_ms: 0` for every request: a zero-await `spawn()`ed task
//! resolves within a single `render_immediate_to_vec()` call (the same mechanism this
//! change's Group 5 tests already rely on for effect flushing), so two requests dispatched
//! in immediate succession race only on generation order, never on real elapsed time. A
//! real, non-zero `delay_ms` actually waiting out its own duration is **not** covered here,
//! consistent with this crate's own established precedent: `test_toast.rs` documents a real
//! attempt at a `#[tokio::test(start_paused = true)]` harness for the same class of
//! spawn+timer behavior that didn't work through this crate's `VirtualDom` test harness. See
//! this change's manual playground checks for a real-time substitute.

use adico_primitives::hover_intent::use_hover_intent;
use dioxus::prelude::*;
use dioxus_core::{Event, Mutation};
use dioxus_html::{
    EventData, SerializedHtmlEventConverter, SerializedMouseData, set_event_converter,
};

/// Dispatches a real click at the `nth` (0-indexed) `onclick`-listening element in `dom`, in
/// mount/edit order, and returns the freshly re-rendered HTML.
fn click_nth(dom: &mut VirtualDom, nth: usize) -> String {
    let edits = dom.rebuild_to_vec();
    let id = edits
        .edits
        .iter()
        .filter_map(|edit| match edit {
            Mutation::NewEventListener { name, id } if name == "click" => Some(*id),
            _ => None,
        })
        .nth(nth)
        .unwrap_or_else(|| panic!("no click listener at index {nth}"));
    set_event_converter(Box::new(SerializedHtmlEventConverter));
    let event = Event::new(
        EventData::Mouse(SerializedMouseData::default()).into_any(),
        true,
    );
    dom.runtime().handle_event("click", event, id);
    dom.render_immediate_to_vec();
    dioxus_ssr::render(dom)
}

#[component]
fn BoolHoverIntentDemo() -> Element {
    // Accumulates every value the setter actually receives, not just the latest --
    // this is what distinguishes real supersede-by-generation cancellation (the
    // superseded request's setter call never happens at all: exactly one entry) from a
    // naive "no cancellation, just apply everything in order" implementation (both
    // requests would still reach the setter, landing on the right *final* value by
    // ordinary last-write-wins, but with two entries, not one).
    let mut calls = use_signal(Vec::<bool>::new);
    let hover = use_hover_intent::<bool>(Callback::new(move |v| calls.write().push(v)));
    rsx! {
        div {
            button {
                onclick: move |_| {
                    // Two requests fired back-to-back, both zero-delay: the second
                    // (`false`) must be the only one that ever reaches the setter,
                    // proving the first (`true`) was superseded by generation rather
                    // than merely overwritten by ordinary call order.
                    hover.request(true, 0);
                    hover.request(false, 0);
                },
                "fire"
            }
            "calls: {calls:?}"
        }
    }
}

#[test]
fn a_superseded_request_never_reaches_the_setter_only_the_latest_does() {
    let mut dom = VirtualDom::new(BoolHoverIntentDemo);
    let html = click_nth(&mut dom, 0);
    assert!(html.contains("calls: [false]"), "{html}");
}

#[component]
fn OptionUsizeHoverIntentDemo() -> Element {
    let mut applied = use_signal(|| None::<usize>);
    let hover = use_hover_intent::<Option<usize>>(Callback::new(move |v| applied.set(v)));
    rsx! {
        div {
            button {
                onclick: move |_| hover.request(Some(3), 0),
                "fire"
            }
            "applied: {applied:?}"
        }
    }
}

/// Regression coverage for the primitive being generic over the requested value, not
/// hardcoded to `bool` -- `navigation_menu`'s own payload is `Option<usize>` (see
/// design.md's D1).
#[test]
fn a_non_bool_payload_round_trips() {
    let mut dom = VirtualDom::new(OptionUsizeHoverIntentDemo);
    let html = click_nth(&mut dom, 0);
    assert!(html.contains("applied: Some(3)"), "{html}");
}

#[component]
fn SingleRequestHoverIntentDemo() -> Element {
    let mut applied = use_signal(|| false);
    let hover = use_hover_intent::<bool>(Callback::new(move |v| applied.set(v)));
    rsx! {
        div {
            button { onclick: move |_| hover.request(true, 0), "fire" }
            "applied: {applied}"
        }
    }
}

/// A single, non-superseded zero-delay request still applies -- this is what
/// distinguishes real supersede-by-generation behavior from a stub that simply drops
/// every request unconditionally (guards `a_superseded_request_never_applies_only_the_latest_does`
/// against a vacuous pass).
#[test]
fn a_single_zero_delay_request_applies() {
    let mut dom = VirtualDom::new(SingleRequestHoverIntentDemo);
    let html = click_nth(&mut dom, 0);
    assert!(html.contains("applied: true"), "{html}");
}
