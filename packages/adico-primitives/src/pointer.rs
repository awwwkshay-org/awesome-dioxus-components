// SPDX-License-Identifier: MIT OR Apache-2.0
// Forked from DioxusLabs/dioxus-components at bf007c15d0cf4d04d3181cc46cf12325aa773955.
// Upstream path: primitives/src/pointer.rs. See provenance/records/adico-primitives-wave2-risk.json.
//
// Adapted from upstream: the global `window.addEventListener('pointer*', ...)`
// `document::eval` script (installed lazily the first time the pointer
// tracker is read) is now behind this crate's established
// `#[cfg(any(feature = "web", feature = "native"))]` target-gated pattern
// with an SSR-safe stub for the native default, instead of an unconditional
// `dioxus::document::eval` call. `dioxus_document as document` (this crate's
// existing document-interop alias, see `lib.rs`) is used in place of
// upstream's `dioxus::document`. Desktop/mobile pointer-capture behavior
// beyond compiling under the `native` feature is not independently verified
// here (a WebView's pointer-event delivery may differ from a real browser's);
// this matches the migration queue's named risk for this file and is
// recorded, not silently assumed, in the Wave 2 migration record.

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

#[derive(Debug)]
struct Pointer {
    id: i32,
    position: ClientPoint,
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

#[cfg(any(feature = "web", feature = "native"))]
fn upsert_pointer(pointers: &mut Vec<Pointer>, pointer_id: i32, position: ClientPoint) {
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

#[cfg(all(test, any(feature = "web", feature = "native")))]
mod tests {
    use super::*;

    #[test]
    fn upsert_pointer_updates_existing_pointer() {
        let mut pointers = vec![Pointer {
            id: 1,
            position: ClientPoint::new(10.0, 20.0),
        }];

        upsert_pointer(&mut pointers, 1, ClientPoint::new(30.0, 40.0));

        assert_eq!(pointers.len(), 1);
        assert_eq!(pointers[0].position, ClientPoint::new(30.0, 40.0));
    }
}
