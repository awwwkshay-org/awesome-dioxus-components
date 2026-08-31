//! Black-box tests for `adico_primitives::selectable`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/selectable.rs`.
//!
//! `SelectableContext`'s open/selection/focus-navigation methods are already exercised
//! extensively, indirectly, through `select.rs`'s and `combobox.rs`'s own tests (both consume
//! it as their shared state). These tests instead cover `pointer_select_start`/
//! `pointer_select_commit`/`pointer_select_cancel`, which had zero coverage (inline or
//! external) before this task: the WAI-ARIA APG pointer-activation guidance this file's own
//! doc comment now cites — primary button only, and a touch that drifts past a small
//! threshold before release is a scroll, not a tap.

use adico_primitives::selectable::{
    pointer_select_cancel, pointer_select_commit, pointer_select_start,
};
use dioxus::html::geometry::{ClientPoint, ElementPoint, PagePoint, ScreenPoint};
use dioxus::html::input_data::{MouseButton, MouseButtonSet};
use dioxus::html::{
    HasPointerData, InteractionElementOffset, InteractionLocation, Modifiers, ModifiersInteraction,
    PointerInteraction,
};
use dioxus::prelude::*;

/// A minimal synthetic pointer event; see `test_move_interaction.rs`'s identical pattern for
/// why the full `HasPointerData` trait chain must be implemented even though only a few
/// methods are actually read by the code under test here.
struct TestPointerData {
    client: ClientPoint,
    trigger_button: Option<MouseButton>,
    pointer_type: &'static str,
}

impl ModifiersInteraction for TestPointerData {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}

impl InteractionLocation for TestPointerData {
    fn client_coordinates(&self) -> ClientPoint {
        self.client
    }

    fn screen_coordinates(&self) -> ScreenPoint {
        ScreenPoint::new(self.client.x, self.client.y)
    }

    fn page_coordinates(&self) -> PagePoint {
        PagePoint::new(self.client.x, self.client.y)
    }
}

impl InteractionElementOffset for TestPointerData {
    fn element_coordinates(&self) -> ElementPoint {
        ElementPoint::new(self.client.x, self.client.y)
    }
}

impl PointerInteraction for TestPointerData {
    fn trigger_button(&self) -> Option<MouseButton> {
        self.trigger_button
    }

    fn held_buttons(&self) -> MouseButtonSet {
        MouseButtonSet::empty()
    }
}

impl HasPointerData for TestPointerData {
    fn pointer_id(&self) -> i32 {
        1
    }

    fn width(&self) -> f64 {
        1.0
    }

    fn height(&self) -> f64 {
        1.0
    }

    fn pressure(&self) -> f32 {
        0.0
    }

    fn tangential_pressure(&self) -> f32 {
        0.0
    }

    fn tilt_x(&self) -> i32 {
        0
    }

    fn tilt_y(&self) -> i32 {
        0
    }

    fn twist(&self) -> i32 {
        0
    }

    fn pointer_type(&self) -> String {
        self.pointer_type.to_string()
    }

    fn is_primary(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn pointer_event(
    x: f64,
    y: f64,
    trigger_button: Option<MouseButton>,
    pointer_type: &'static str,
) -> Event<PointerData> {
    Event::new(
        std::rc::Rc::new(PointerData::new(TestPointerData {
            client: ClientPoint::new(x, y),
            trigger_button,
            pointer_type,
        })),
        true,
    )
}

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn PointerSelectStartRecordsThePrimaryButtonDownPosition() -> Element {
    let down_pos = use_signal(|| None);
    pointer_select_start(
        &pointer_event(10.0, 20.0, Some(MouseButton::Primary), "mouse"),
        false,
        down_pos,
    );
    assert_eq!(down_pos(), Some((10.0, 20.0)), "{down_pos:?}");
    rsx! { "ok" }
}

#[test]
fn pointer_select_start_records_the_primary_button_down_position() {
    let html = render(PointerSelectStartRecordsThePrimaryButtonDownPosition);
    assert!(html.contains("ok"), "{html}");
}

#[component]
fn PointerSelectStartIgnoresNonPrimaryButtons() -> Element {
    let down_pos = use_signal(|| None);
    pointer_select_start(
        &pointer_event(10.0, 20.0, Some(MouseButton::Secondary), "mouse"),
        false,
        down_pos,
    );
    assert_eq!(down_pos(), None, "{down_pos:?}");
    rsx! { "ok" }
}

#[test]
fn pointer_select_start_ignores_non_primary_buttons() {
    let html = render(PointerSelectStartIgnoresNonPrimaryButtons);
    assert!(html.contains("ok"), "{html}");
}

#[component]
fn PointerSelectStartIgnoresDisabledOptions() -> Element {
    let down_pos = use_signal(|| None);
    pointer_select_start(
        &pointer_event(10.0, 20.0, Some(MouseButton::Primary), "mouse"),
        true,
        down_pos,
    );
    assert_eq!(down_pos(), None, "{down_pos:?}");
    rsx! { "ok" }
}

#[test]
fn pointer_select_start_ignores_disabled_options() {
    let html = render(PointerSelectStartIgnoresDisabledOptions);
    assert!(html.contains("ok"), "{html}");
}

#[component]
fn PointerSelectCommitAcceptsAMouseClickWithNoDriftCheck() -> Element {
    let down_pos = use_signal(|| Some((0.0, 0.0)));
    // A mouse click at a wildly different position from where it went down still commits —
    // the drift check is touch-only, matching a mouse's own native click semantics.
    let committed = pointer_select_commit(
        &pointer_event(500.0, 500.0, Some(MouseButton::Primary), "mouse"),
        false,
        down_pos,
    );
    assert!(committed);
    assert_eq!(
        down_pos(),
        None,
        "commit must clear the recorded down position"
    );
    rsx! { "ok" }
}

#[test]
fn pointer_select_commit_accepts_a_mouse_click_with_no_drift_check() {
    let html = render(PointerSelectCommitAcceptsAMouseClickWithNoDriftCheck);
    assert!(html.contains("ok"), "{html}");
}

#[component]
fn PointerSelectCommitAcceptsATouchTapWithinTheDriftThreshold() -> Element {
    let down_pos = use_signal(|| Some((10.0, 10.0)));
    let committed = pointer_select_commit(
        &pointer_event(12.0, 12.0, Some(MouseButton::Primary), "touch"),
        false,
        down_pos,
    );
    assert!(committed, "a small (< 5px) touch drift is still a tap");
    rsx! { "ok" }
}

#[test]
fn pointer_select_commit_accepts_a_touch_tap_within_the_drift_threshold() {
    let html = render(PointerSelectCommitAcceptsATouchTapWithinTheDriftThreshold);
    assert!(html.contains("ok"), "{html}");
}

#[component]
fn PointerSelectCommitRejectsATouchThatDriftedPastTheThreshold() -> Element {
    let down_pos = use_signal(|| Some((10.0, 10.0)));
    let committed = pointer_select_commit(
        &pointer_event(30.0, 30.0, Some(MouseButton::Primary), "touch"),
        false,
        down_pos,
    );
    assert!(
        !committed,
        "a touch that drifted this far is a scroll, not a tap"
    );
    rsx! { "ok" }
}

#[test]
fn pointer_select_commit_rejects_a_touch_that_drifted_past_the_threshold() {
    let html = render(PointerSelectCommitRejectsATouchThatDriftedPastTheThreshold);
    assert!(html.contains("ok"), "{html}");
}

#[component]
fn PointerSelectCommitRejectsWithNoRecordedDownPosition() -> Element {
    let down_pos = use_signal(|| None);
    let committed = pointer_select_commit(
        &pointer_event(10.0, 10.0, Some(MouseButton::Primary), "mouse"),
        false,
        down_pos,
    );
    assert!(
        !committed,
        "a pointer-up with no matching pointer-down (e.g. it started on a disabled option) must not commit"
    );
    rsx! { "ok" }
}

#[test]
fn pointer_select_commit_rejects_with_no_recorded_down_position() {
    let html = render(PointerSelectCommitRejectsWithNoRecordedDownPosition);
    assert!(html.contains("ok"), "{html}");
}

#[component]
fn PointerSelectCancelClearsTheDownPosition() -> Element {
    let down_pos = use_signal(|| Some((10.0, 10.0)));
    pointer_select_cancel(down_pos);
    assert_eq!(down_pos(), None);
    rsx! { "ok" }
}

#[test]
fn pointer_select_cancel_clears_the_down_position() {
    let html = render(PointerSelectCancelClearsTheDownPosition);
    assert!(html.contains("ok"), "{html}");
}
