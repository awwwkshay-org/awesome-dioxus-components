// SPDX-License-Identifier: MIT OR Apache-2.0

//! A unified press/long-press pointer-gesture primitive, consolidating logic
//! previously duplicated separately in `context_menu.rs` (a long-press timer
//! with its own drift-cancel check) and `selectable.rs` (a tap-vs-drag-scroll
//! drift check with no timer). See design.md §8a.
//!
//! **Scope note on this task's own premise:** the task that named this
//! primitive ("consolidating `pointer.rs`, `move_interaction.rs`, and
//! `selectable.rs`'s pointer-drift check... verify `drag_and_drop_list` and
//! `context_menu`'s independently reimplemented long-press") is only half
//! right: `drag_and_drop_list.rs` has no long-press timer at all — it uses
//! native HTML5 drag-and-drop (`draggable`, `ondragstart`/`ondragover`/
//! `ondrop`), not a manual press-and-hold gesture. `context_menu.rs` is the
//! one real long-press consumer. This mirrors task 7.3d's correction about
//! `calendar.rs` not actually using `CollectionState`: the task's premise is
//! recorded as wrong here rather than silently worked around.
//!
//! **A more significant finding, while investigating this task:**
//! `pointer.rs`'s global pointer-position registry — which
//! `move_interaction.rs`'s drag tracking (`MoveInteraction::pointer_move`,
//! used by `slider`/`color_picker`) depends on for continuous position
//! updates during a drag — is fed exclusively by the same long-lived,
//! repeatedly-firing `document::eval` listener pattern
//! `provenance/records/adico-primitives-wave3-overlays.json` documents as
//! never registering `addEventListener` in this Dioxus 0.7.9/0.7.10 web
//! runtime. `tests/playwright/wave2-risk.spec.ts`'s Slider test only exercises
//! keyboard control (`ArrowRight`/`ArrowLeft`), never an actual pointer drag,
//! so this has never been exercised in a real browser. If the same defect
//! applies here (plausible, but unconfirmed without a browser), pointer
//! dragging on `slider`/`color_picker` does not work on `web` today. Fixing
//! this is **not** attempted in this task: unlike `use_escape_key` (7.4d),
//! there is no drop-in native-Dioxus-event substitute for a *global* pointer
//! tracker — a real fix would mean redesigning `slider`/`color_picker`'s
//! pointer handling around per-element `onpointermove`/`onpointerup` plus
//! `setPointerCapture` (so the drag keeps tracking even once the pointer
//! leaves the thumb's bounds), which is a real behavior change to two
//! already-shipped, partially browser-verified components — too invasive to
//! attempt blind. Recorded as a follow-up, not silently left implicit.
//!
//! This primitive itself — the long-press timer and the shared drift-check
//! math — has no dependency on `pointer.rs`'s registry or any long-lived
//! `document::eval` listener: it only uses this crate's own target-aware
//! `time::sleep` (a plain async timer, not a browser-event bridge) and the
//! pointer event's own coordinates, so it is not suspected of the same
//! defect class.

use std::time::Duration;

use dioxus::prelude::*;
use dioxus_core::Task;

use crate::time::sleep;

/// Whether `current` has moved past `tolerance_squared` (in squared CSS
/// pixels) away from `start` — the crate's shared drift-check math,
/// previously duplicated separately in `context_menu.rs` (long-press cancel,
/// `LONG_PRESS_MOVE_TOLERANCE_SQ = 100.0`, i.e. a 10px tolerance) and
/// `selectable.rs` (tap-vs-drag-scroll commit, `25.0`, i.e. 5px). Takes the
/// tolerance pre-squared, matching both existing call sites, so callers never
/// need to take a square root.
pub fn moved_past_threshold(
    start: (f64, f64),
    current: (f64, f64),
    tolerance_squared: f64,
) -> bool {
    let dx = current.0 - start.0;
    let dy = current.1 - start.1;
    dx * dx + dy * dy > tolerance_squared
}

/// Shared state for [`use_long_press`].
#[derive(Clone, Copy)]
pub struct LongPress {
    task: Signal<Option<Task>>,
    start: Signal<Option<(f64, f64)>>,
    move_tolerance_squared: f64,
}

impl LongPress {
    /// Call from `onpointerdown`. Starts the long-press timer at the
    /// pointer's current position; does nothing if a press is already
    /// in-flight. `on_long_press` may be async (e.g. to await a viewport
    /// measurement before opening a menu, matching `context_menu.rs`'s own
    /// long-press). Filtering to a specific pointer type (mouse vs.
    /// touch/pen) or mouse button, if a caller needs it, stays the caller's
    /// responsibility — checking `event.pointer_type()`/`trigger_button()`
    /// before calling this at all — since neither is a generic gesture
    /// concern (touch/pen pointerdown events don't reliably report a mouse
    /// button the way this crate's other pointer helpers, e.g.
    /// `selectable::pointer_select_start`, filter on for a *mouse* gesture).
    pub fn on_pointer_down<F>(
        &mut self,
        event: &Event<PointerData>,
        duration: Duration,
        on_long_press: impl FnOnce((f64, f64)) -> F + 'static,
    ) where
        F: std::future::Future<Output = ()> + 'static,
    {
        if self.start.peek().is_some() {
            return;
        }
        let point = event.client_coordinates();
        let position = (point.x, point.y);
        self.start.set(Some(position));
        let mut task_signal = self.task;
        task_signal.set(Some(spawn(async move {
            sleep(duration).await;
            task_signal.set(None);
            on_long_press(position).await;
        })));
    }

    /// Call from `onpointermove`. Cancels the in-flight press if the pointer
    /// has moved past this instance's move tolerance.
    pub fn on_pointer_move(&mut self, event: &Event<PointerData>) {
        let Some(start) = self.start.peek().as_ref().copied() else {
            return;
        };
        let point = event.client_coordinates();
        if moved_past_threshold(start, (point.x, point.y), self.move_tolerance_squared) {
            self.cancel();
        }
    }

    /// Call from `onpointerup`/`onpointercancel`, or any other condition
    /// that should abort an in-flight press before its timer fires.
    pub fn cancel(&mut self) {
        if let Some(task) = self.task.write().take() {
            task.cancel();
        }
        self.start.set(None);
    }

    /// Whether a press is currently timing (started, not yet fired or
    /// cancelled).
    pub fn is_pending(&self) -> bool {
        self.start.peek().is_some()
    }
}

/// Build a [`LongPress`] gesture: an `on_long_press` callback fires once the
/// pointer has been held for `move_tolerance_squared`-bounded stillness for
/// the duration passed to [`LongPress::on_pointer_down`]. Consolidates the
/// timer/drift-cancel logic `context_menu.rs`'s own long-press hand-rolled
/// (see this module's doc comment): the same spawn-a-sleep-then-fire timer
/// shape and the same peek-then-cancel drift check, now shared.
pub fn use_long_press(move_tolerance_squared: f64) -> LongPress {
    let task = use_signal(|| None);
    let start = use_signal(|| None);
    LongPress {
        task,
        start,
        move_tolerance_squared,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moved_past_threshold_is_false_within_tolerance() {
        assert!(!moved_past_threshold((0.0, 0.0), (3.0, 4.0), 25.0));
    }

    #[test]
    fn moved_past_threshold_is_true_beyond_tolerance() {
        assert!(moved_past_threshold((0.0, 0.0), (3.0, 4.1), 25.0));
    }

    #[test]
    fn moved_past_threshold_matches_context_menus_own_tolerance() {
        // context_menu.rs's LONG_PRESS_MOVE_TOLERANCE_SQ = 100.0 (10px).
        assert!(!moved_past_threshold((0.0, 0.0), (9.0, 0.0), 100.0));
        assert!(moved_past_threshold((0.0, 0.0), (11.0, 0.0), 100.0));
    }

    #[test]
    fn moved_past_threshold_matches_selectables_own_tolerance() {
        // selectable.rs's tap-vs-drag-scroll tolerance = 25.0 (5px).
        assert!(!moved_past_threshold((0.0, 0.0), (4.0, 0.0), 25.0));
        assert!(moved_past_threshold((0.0, 0.0), (6.0, 0.0), 25.0));
    }

    #[component]
    fn LongPressHarness() -> Element {
        let long_press = use_long_press(100.0);
        rsx! {
            "{long_press.is_pending()}"
        }
    }

    #[test]
    fn long_press_starts_with_no_pending_press() {
        let mut dom = VirtualDom::new(LongPressHarness);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("false"));
    }
}
