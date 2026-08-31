//! Black-box tests for `adico_primitives::move_interaction`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/move_interaction.rs`.
//!
//! `MoveEvent::from_keyboard`'s keyboard-mapping tests are carried over unchanged from the
//! file's own prior inline suite (moved here, not rewritten). The `MoveInteraction` pointer
//! tests are new: `start_pointer`/`pointer_move`/`end_pointer` are all reachable through the
//! fully public `MoveInteraction` API without needing any `pub`-for-tests widening, so a
//! synthetic `HasPointerData` impl (mirroring the file's own prior `TestKeyboardData` pattern
//! for `HasKeyboardData`) drives them directly. `pointer_move`'s delta-computation branch
//! depends on `crate::pointer::pointer_position`, which is only backed by real position data
//! under the `web`/`native` features (see `pointer.rs`'s doc comment) — under default features
//! it always returns `None`, so only the "no position available, end the drag" branch is
//! reachable there; the delta branch has its own `#[cfg(any(feature = "web", feature =
//! "native"))]`-gated test below, verified with `cargo test -p adico-primitives --features
//! web` (not part of this repo's default `cargo test` baseline, matching `pointer.rs`'s own
//! pre-existing test-gating convention).

use adico_primitives::direction::Direction;
use adico_primitives::move_interaction::{MoveEvent, MoveModifiers, use_move_interaction};
use dioxus::html::geometry::{ClientPoint, ElementPoint, PagePoint, ScreenPoint};
use dioxus::html::input_data::{MouseButton, MouseButtonSet};
use dioxus::html::{
    Code, HasKeyboardData, HasPointerData, InteractionElementOffset, InteractionLocation, Location,
    Modifiers, ModifiersInteraction, PointerInteraction,
};
use dioxus::prelude::*;

struct TestKeyboardData {
    key: Key,
    modifiers: Modifiers,
}

impl ModifiersInteraction for TestKeyboardData {
    fn modifiers(&self) -> Modifiers {
        self.modifiers
    }
}

impl HasKeyboardData for TestKeyboardData {
    fn key(&self) -> Key {
        self.key.clone()
    }

    fn code(&self) -> Code {
        Code::Unidentified
    }

    fn location(&self) -> Location {
        Location::Standard
    }

    fn is_auto_repeating(&self) -> bool {
        false
    }

    fn is_composing(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn keyboard_event(key: Key, modifiers: Modifiers) -> Event<KeyboardData> {
    Event::new(
        std::rc::Rc::new(KeyboardData::new(TestKeyboardData { key, modifiers })),
        true,
    )
}

#[test]
fn keyboard_move_maps_arrow_keys() {
    assert_eq!(
        MoveEvent::from_keyboard(
            &keyboard_event(Key::ArrowUp, Modifiers::empty()),
            2.0,
            Direction::Ltr
        ),
        Some(MoveEvent {
            delta_x: 0.0,
            delta_y: 2.0,
            modifiers: MoveModifiers::default(),
        })
    );
    assert_eq!(
        MoveEvent::from_keyboard(
            &keyboard_event(Key::ArrowDown, Modifiers::empty()),
            2.0,
            Direction::Ltr
        )
        .map(|event| (event.delta_x, event.delta_y)),
        Some((0.0, -2.0))
    );
    assert_eq!(
        MoveEvent::from_keyboard(
            &keyboard_event(Key::ArrowRight, Modifiers::empty()),
            2.0,
            Direction::Ltr
        )
        .map(|event| (event.delta_x, event.delta_y)),
        Some((2.0, 0.0))
    );
    assert_eq!(
        MoveEvent::from_keyboard(
            &keyboard_event(Key::ArrowLeft, Modifiers::empty()),
            2.0,
            Direction::Ltr
        )
        .map(|event| (event.delta_x, event.delta_y)),
        Some((-2.0, 0.0))
    );
}

#[test]
fn keyboard_move_flips_left_right_for_rtl() {
    assert_eq!(
        MoveEvent::from_keyboard(
            &keyboard_event(Key::ArrowRight, Modifiers::empty()),
            2.0,
            Direction::Rtl
        )
        .map(|event| (event.delta_x, event.delta_y)),
        Some((-2.0, 0.0))
    );
    assert_eq!(
        MoveEvent::from_keyboard(
            &keyboard_event(Key::ArrowLeft, Modifiers::empty()),
            2.0,
            Direction::Rtl
        )
        .map(|event| (event.delta_x, event.delta_y)),
        Some((2.0, 0.0))
    );
    // Vertical keys are unaffected by direction.
    assert_eq!(
        MoveEvent::from_keyboard(
            &keyboard_event(Key::ArrowUp, Modifiers::empty()),
            2.0,
            Direction::Rtl
        )
        .map(|event| (event.delta_x, event.delta_y)),
        Some((0.0, 2.0))
    );
}

#[test]
fn keyboard_move_applies_shift_multiplier() {
    let expected_modifiers = MoveModifiers {
        shift_key: true,
        ..MoveModifiers::default()
    };

    assert_eq!(
        MoveEvent::from_keyboard(
            &keyboard_event(Key::ArrowRight, Modifiers::SHIFT),
            2.0,
            Direction::Ltr
        )
        .map(|event| (event.delta_x, event.delta_y, event.modifiers)),
        Some((20.0, 0.0, expected_modifiers))
    );
}

#[test]
fn keyboard_move_ignores_non_arrow_keys() {
    assert_eq!(
        MoveEvent::from_keyboard(
            &keyboard_event(Key::Character("a".to_string()), Modifiers::empty()),
            2.0,
            Direction::Ltr
        ),
        None
    );
}

/// A minimal synthetic pointer event, mirroring this file's own prior `TestKeyboardData`
/// pattern: only `pointer_id`, `client_coordinates`, and `trigger_button` are read by
/// `MoveInteraction`, but `HasPointerData`'s full trait chain must still be implemented.
struct TestPointerData {
    pointer_id: i32,
    client: ClientPoint,
    trigger_button: Option<MouseButton>,
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
        self.pointer_id
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
        "mouse".to_string()
    }

    fn is_primary(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn primary_pointer_down(pointer_id: i32, x: f64, y: f64) -> Event<PointerData> {
    Event::new(
        std::rc::Rc::new(PointerData::new(TestPointerData {
            pointer_id,
            client: ClientPoint::new(x, y),
            trigger_button: Some(MouseButton::Primary),
        })),
        true,
    )
}

fn secondary_pointer_down(pointer_id: i32, x: f64, y: f64) -> Event<PointerData> {
    Event::new(
        std::rc::Rc::new(PointerData::new(TestPointerData {
            pointer_id,
            client: ClientPoint::new(x, y),
            trigger_button: Some(MouseButton::Secondary),
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
fn StartPointerAcceptsThePrimaryButton() -> Element {
    let dragging = use_signal(|| false);
    let mut interaction = use_move_interaction(dragging);
    let started = interaction.start_pointer(&primary_pointer_down(1, 10.0, 10.0));
    assert!(started, "a primary-button pointer-down must start the drag");
    rsx! { "ok" }
}

#[test]
fn start_pointer_accepts_the_primary_button() {
    let html = render(StartPointerAcceptsThePrimaryButton);
    assert!(html.contains("ok"), "{html}");
}

#[component]
fn StartPointerRejectsNonPrimaryButtons() -> Element {
    let dragging = use_signal(|| false);
    let mut interaction = use_move_interaction(dragging);
    let started = interaction.start_pointer(&secondary_pointer_down(1, 10.0, 10.0));
    assert!(
        !started,
        "a non-primary-button pointer-down must not start the drag"
    );
    rsx! { "ok" }
}

#[test]
fn start_pointer_rejects_non_primary_buttons() {
    let html = render(StartPointerRejectsNonPrimaryButtons);
    assert!(html.contains("ok"), "{html}");
}

#[component]
fn StartPointerIgnoresASecondPointerWhileOneIsActive() -> Element {
    let dragging = use_signal(|| false);
    let mut interaction = use_move_interaction(dragging);
    let first = interaction.start_pointer(&primary_pointer_down(1, 10.0, 10.0));
    let second = interaction.start_pointer(&primary_pointer_down(2, 20.0, 20.0));
    assert!(first, "the first pointer starts the drag");
    assert!(
        !second,
        "a second pointer-down must be ignored while one is already active"
    );
    rsx! { "ok" }
}

#[test]
fn start_pointer_ignores_a_second_pointer_while_one_is_active() {
    let html = render(StartPointerIgnoresASecondPointerWhileOneIsActive);
    assert!(html.contains("ok"), "{html}");
}

// `pointer_move`'s delta-computation branch reads `crate::pointer::pointer_position`, which
// under default (no `web`/`native`) features always returns `None` — only reachable here is
// the "no position available" branch, which ends the drag. Under `web`/`native`,
// `start_pointer` itself calls `track_pointer_down`, so a position *is* available; that case
// is `with_a_live_pointer_position_source`'s concern below.
#[cfg(not(any(feature = "web", feature = "native")))]
#[component]
fn PointerMoveEndsTheDragWhenNoPositionIsAvailable() -> Element {
    let mut dragging = use_signal(|| false);
    let mut interaction = use_move_interaction(dragging);
    interaction.start_pointer(&primary_pointer_down(1, 10.0, 10.0));
    dragging.set(true);

    let moved = interaction.pointer_move();
    assert!(
        moved.is_none(),
        "no tracked position (default features) must not produce a move event"
    );
    assert!(
        !dragging(),
        "pointer_move must end the drag once no position is available"
    );
    rsx! { "ok" }
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn pointer_move_ends_the_drag_when_no_position_is_available() {
    let html = render(PointerMoveEndsTheDragWhenNoPositionIsAvailable);
    assert!(html.contains("ok"), "{html}");
}

#[component]
fn PointerMoveIsANoOpWhileNotDragging() -> Element {
    let dragging = use_signal(|| false);
    let mut interaction = use_move_interaction(dragging);
    interaction.start_pointer(&primary_pointer_down(1, 10.0, 10.0));
    // `dragging` was never set to `true`.
    let moved = interaction.pointer_move();
    assert!(moved.is_none(), "{moved:?}");
    rsx! { "ok" }
}

#[test]
fn pointer_move_is_a_no_op_while_not_dragging() {
    let html = render(PointerMoveIsANoOpWhileNotDragging);
    assert!(html.contains("ok"), "{html}");
}

#[component]
fn EndPointerResetsDraggingAndAllowsANewPointer() -> Element {
    let mut dragging = use_signal(|| false);
    let mut interaction = use_move_interaction(dragging);
    interaction.start_pointer(&primary_pointer_down(1, 10.0, 10.0));
    dragging.set(true);

    interaction.end_pointer();
    assert!(!dragging(), "end_pointer must clear dragging");

    let restarted = interaction.start_pointer(&primary_pointer_down(2, 0.0, 0.0));
    assert!(
        restarted,
        "a new pointer must be accepted once the previous one ended"
    );
    rsx! { "ok" }
}

#[test]
fn end_pointer_resets_dragging_and_allows_a_new_pointer() {
    let html = render(EndPointerResetsDraggingAndAllowsANewPointer);
    assert!(html.contains("ok"), "{html}");
}

// Only reachable with a real pointer-position source; verified with
// `cargo test -p adico-primitives --features web` (or `--features native`), not part of this
// repo's default `cargo test` baseline — see `pointer.rs`'s own pre-existing test-gating
// convention for the same reason.
#[cfg(any(feature = "web", feature = "native"))]
mod with_a_live_pointer_position_source {
    use super::*;
    use adico_primitives::pointer::track_pointer_down;

    #[component]
    fn PointerMoveComputesADeltaFromTrackedPositions() -> Element {
        let mut dragging = use_signal(|| false);
        let mut interaction = use_move_interaction(dragging);
        interaction.start_pointer(&primary_pointer_down(1, 10.0, 10.0));
        dragging.set(true);

        // The first `pointer_move` after starting has no prior position to diff against.
        let first = interaction.pointer_move();
        assert_eq!(
            first,
            Some(MoveEvent {
                delta_x: 0.0,
                delta_y: 0.0,
                modifiers: MoveModifiers::default(),
            }),
            "{first:?}"
        );

        track_pointer_down(1, ClientPoint::new(25.0, 40.0));
        let second = interaction.pointer_move();
        assert_eq!(
            second,
            Some(MoveEvent {
                delta_x: 15.0,
                delta_y: 30.0,
                modifiers: MoveModifiers::default(),
            }),
            "{second:?}"
        );
        rsx! { "ok" }
    }

    #[test]
    fn pointer_move_computes_a_delta_from_tracked_positions() {
        let html = render(PointerMoveComputesADeltaFromTrackedPositions);
        assert!(html.contains("ok"), "{html}");
    }
}
