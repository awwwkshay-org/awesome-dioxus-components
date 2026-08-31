// This crate's own established target-gating pattern is the spec here — no ARIA pattern
// applies to a low-level pointer-position registry, and design.md §8a's "unified
// press/long-press/drag" primitive that `gesture.rs` began is press/long-press only
// (context_menu.rs's/selectable.rs's duplicated timers): it does not cover this file's
// separate concern, continuous drag-position tracking, which `move_interaction.rs` still
// depends on unconsolidated, matching the same architecture this file already used —
// `#[cfg(any(feature = "web", feature = "native"))]` target-gating around a
// `document::eval`-backed global listener, with an SSR-safe stub for targets without a DOM.
// **Unverified in a live browser**, same finding `gesture.rs`'s own doc comment already
// records for this exact file: the `window.addEventListener('pointer*', ...)` listener is
// installed by the same long-lived, repeatedly-firing `document::eval` pattern
// `provenance/records/adico-primitives-wave3-overlays.json` documents as never actually
// registering in this Dioxus 0.7.9/0.7.10 web runtime. If that defect applies here too
// (plausible, not independently re-confirmed by this task), pointer dragging on
// `slider`/`color_picker` does not track position on `web` today. Not re-investigated here;
// see `gesture.rs`'s doc comment for why a fix is out of scope for a small task (no drop-in
// native-Dioxus-event substitute for a *global*, not per-element, pointer tracker).

//! A global pointer-position registry.
//!
//! Tracks the last-known screen position of every active pointer (touch,
//! pen, or mouse) via a single window-level listener, so controls that need
//! pointer position during a drag (see [`crate::move_interaction`]) don't
//! each install their own. Target-gated with an SSR-safe no-op on targets
//! without a DOM (SSR/native).

#[cfg(any(feature = "web", feature = "native"))]
use crate::dioxus_core::{Runtime, queue_effect};
use dioxus::html::geometry::ClientPoint;
#[cfg(any(feature = "web", feature = "native"))]
use dioxus::prelude::*;
#[cfg(any(feature = "web", feature = "native"))]
use dioxus_document as document;

/// `pub` only for `packages/adico-primitives/tests/`; not part of the intended public API.
/// Not target-gated, unlike `POINTERS`/`add_pointer`/etc.: this struct and [`upsert_pointer`]
/// are pure `Vec` bookkeeping with no DOM dependency, so they stay testable under default
/// (no `web`/`native`) features even though their only real caller is gated.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pointer {
    pub id: i32,
    pub position: ClientPoint,
}

#[cfg(any(feature = "web", feature = "native"))]
static POINTERS: GlobalSignal<Vec<Pointer>> = Global::new(|| {
    let runtime = Runtime::current();
    queue_effect(move || {
        runtime.spawn(ScopeId::ROOT, async move {
            let mut pointer_updates = document::eval(
                // clientX/clientY (not pageX/pageY) must match element handlers
                // that store `evt.client_coordinates()` and viewport-relative
                // rects from getBoundingClientRect.
                "window.addEventListener('pointerdown', (e) => {
                    dioxus.send(['down', [e.pointerId, e.clientX, e.clientY]]);
                });
                window.addEventListener('pointermove', (e) => {
                    dioxus.send(['move', [e.pointerId, e.clientX, e.clientY]]);
                });
                window.addEventListener('pointerup', (e) => {
                    dioxus.send(['up', [e.pointerId, e.clientX, e.clientY]]);
                });
                window.addEventListener('pointercancel', (e) => {
                    dioxus.send(['up', [e.pointerId, e.clientX, e.clientY]]);
                });",
            );

            while let Ok((event_type, (pointer_id, x, y))) =
                pointer_updates.recv::<(String, (i32, f64, f64))>().await
            {
                let position = ClientPoint::new(x, y);

                match event_type.as_str() {
                    "down" => add_pointer(pointer_id, position),
                    "move" => update_pointer(pointer_id, position),
                    "up" => remove_pointer(pointer_id),
                    _ => {}
                }
            }
        });
    });

    Vec::new()
});

#[cfg(any(feature = "web", feature = "native"))]
pub fn track_pointer_down(pointer_id: i32, position: ClientPoint) {
    add_pointer(pointer_id, position);
}

#[cfg(not(any(feature = "web", feature = "native")))]
pub fn track_pointer_down(_pointer_id: i32, _position: ClientPoint) {}

#[cfg(any(feature = "web", feature = "native"))]
pub fn pointer_position(pointer_id: i32) -> Option<ClientPoint> {
    POINTERS
        .read()
        .iter()
        .find(|pointer| pointer.id == pointer_id)
        .map(|pointer| pointer.position)
}

#[cfg(not(any(feature = "web", feature = "native")))]
pub fn pointer_position(_pointer_id: i32) -> Option<ClientPoint> {
    None
}

#[cfg(any(feature = "web", feature = "native"))]
fn add_pointer(pointer_id: i32, position: ClientPoint) {
    let mut pointers = POINTERS.write();
    upsert_pointer(&mut pointers, pointer_id, position);
}

/// `pub` only for `packages/adico-primitives/tests/`; not part of the intended public API.
pub fn upsert_pointer(pointers: &mut Vec<Pointer>, pointer_id: i32, position: ClientPoint) {
    if let Some(pointer) = pointers.iter_mut().find(|pointer| pointer.id == pointer_id) {
        pointer.position = position;
    } else {
        pointers.push(Pointer {
            id: pointer_id,
            position,
        });
    }
}

#[cfg(any(feature = "web", feature = "native"))]
fn update_pointer(pointer_id: i32, position: ClientPoint) {
    if let Some(pointer) = POINTERS
        .write()
        .iter_mut()
        .find(|pointer| pointer.id == pointer_id)
    {
        pointer.position = position;
    }
}

#[cfg(any(feature = "web", feature = "native"))]
fn remove_pointer(pointer_id: i32) {
    POINTERS.write().retain(|pointer| pointer.id != pointer_id);
}
