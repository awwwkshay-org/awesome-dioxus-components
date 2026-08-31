//! Black-box tests for `adico_primitives::pointer`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/pointer.rs`.
//!
//! `Pointer`/`upsert_pointer` are pure `Vec` bookkeeping, deliberately left un-gated (unlike
//! the rest of this module) so they're testable under default features — see their own doc
//! comments. The global `POINTERS` registry and its `document::eval`-backed listener are
//! gated behind `web`/`native` and have no independently testable logic of their own beyond
//! what `upsert_pointer` already covers; `pointer.rs`'s module doc comment records why this
//! listener's actual browser behavior is unconfirmed, which these tests do not attempt to
//! verify.

use adico_primitives::pointer::{Pointer, upsert_pointer};
use dioxus::html::geometry::ClientPoint;

#[test]
fn upsert_pointer_inserts_a_new_pointer_when_the_id_is_not_already_tracked() {
    let mut pointers = Vec::new();

    upsert_pointer(&mut pointers, 1, ClientPoint::new(10.0, 20.0));

    assert_eq!(pointers.len(), 1);
    assert_eq!(pointers[0].id, 1);
    assert_eq!(pointers[0].position, ClientPoint::new(10.0, 20.0));
}

#[test]
fn upsert_pointer_updates_the_position_of_an_existing_pointer_in_place() {
    let mut pointers = vec![Pointer {
        id: 1,
        position: ClientPoint::new(10.0, 20.0),
    }];

    upsert_pointer(&mut pointers, 1, ClientPoint::new(30.0, 40.0));

    assert_eq!(pointers.len(), 1);
    assert_eq!(pointers[0].position, ClientPoint::new(30.0, 40.0));
}

#[test]
fn upsert_pointer_tracks_multiple_independent_pointers_by_id() {
    let mut pointers = Vec::new();

    upsert_pointer(&mut pointers, 1, ClientPoint::new(1.0, 1.0));
    upsert_pointer(&mut pointers, 2, ClientPoint::new(2.0, 2.0));
    upsert_pointer(&mut pointers, 1, ClientPoint::new(9.0, 9.0));

    assert_eq!(pointers.len(), 2);
    let first = pointers.iter().find(|p| p.id == 1).expect("pointer 1");
    let second = pointers.iter().find(|p| p.id == 2).expect("pointer 2");
    assert_eq!(first.position, ClientPoint::new(9.0, 9.0));
    assert_eq!(second.position, ClientPoint::new(2.0, 2.0));
}
