// SPDX-License-Identifier: MIT OR Apache-2.0

//! A shared, reference-counted page scroll lock.
//!
//! Several overlays may each call [`use_scroll_lock`] while open (a dialog
//! that opens another dialog, for example); the page's scroll is only
//! restored once every caller that requested a lock has released it (either
//! by becoming inactive or unmounting), so the first overlay to close never
//! prematurely re-enables scrolling while a second one is still open.
//!
//! This is the Radix/shadcn-style full-page `overflow: hidden` lock used by
//! modal surfaces (dialog, alert dialog: both call [`use_scroll_lock`] from
//! their own `open` state). It is a deliberately different technique from
//! [`crate::context_menu`]'s own wheel/touchmove suppression:
//! that menu is `position: fixed` and pinned to a click point, so hiding the
//! scrollbar via `overflow: hidden` would shift the page layout right under
//! the open menu; suppressing scroll input directly avoids that shift. Do
//! not migrate `context_menu` onto this primitive — the two exist for
//! different reasons and neither is a strict improvement on the other.

#[cfg(any(feature = "web", feature = "native"))]
use std::cell::Cell;
#[cfg(any(feature = "web", feature = "native"))]
use std::rc::Rc;

use dioxus::prelude::*;
#[cfg(any(feature = "web", feature = "native"))]
use dioxus_document as document;

// Only reachable through `use_scroll_lock`'s real (`web`/`native`) implementation below; the
// SSR/server-default `use_scroll_lock` stub is a hardcoded no-op that never touches this type,
// so it would otherwise be reported dead on that target.
#[cfg(any(feature = "web", feature = "native"))]
#[derive(Clone, Default)]
struct ScrollLockCount(Rc<Cell<usize>>);

/// While `active()` is true, hold a page scroll lock; released when
/// `active()` becomes `false` or the caller unmounts. A no-op on targets
/// without a DOM (SSR, or a build with neither the `web` nor `native`
/// feature enabled).
#[cfg(any(feature = "web", feature = "native"))]
pub fn use_scroll_lock(active: impl Readable<Target = bool> + Copy + 'static) {
    let count = use_hook(|| {
        try_consume_context::<ScrollLockCount>()
            .unwrap_or_else(|| provide_context(ScrollLockCount::default()))
    });
    let mut held = use_signal(|| false);

    use_effect({
        let count = count.clone();
        move || {
            let should_hold = active.cloned();
            if should_hold == held() {
                return;
            }
            held.set(should_hold);
            acquire_or_release(&count, should_hold);
        }
    });

    crate::use_effect_cleanup(move || {
        if held() {
            acquire_or_release(&count, false);
        }
    });
}

/// A no-op on targets without a DOM (SSR, or a build with neither the `web`
/// nor `native` feature enabled).
#[cfg(not(any(feature = "web", feature = "native")))]
pub fn use_scroll_lock(_active: impl Readable<Target = bool> + Copy + 'static) {}

/// The pure refcount arithmetic, kept free of any DOM call so it is directly
/// unit-testable outside a Dioxus runtime (unlike [`acquire_or_release`],
/// which calls `document::eval` and therefore requires one).
///
/// Reachable both through `acquire_or_release` (`web`/`native` only) and directly from this
/// file's own `#[cfg(test)]` tests (every target), hence `test` in the gate below.
#[cfg(any(feature = "web", feature = "native", test))]
fn next_count(previous: usize, acquire: bool) -> usize {
    if acquire {
        previous + 1
    } else {
        previous.saturating_sub(1)
    }
}

/// Only the `previous == 0`/`next == 0` transition edges actually touch
/// `document.body`'s style.
#[cfg(any(feature = "web", feature = "native"))]
fn acquire_or_release(count: &ScrollLockCount, acquire: bool) {
    let previous = count.0.get();
    let next = next_count(previous, acquire);
    count.0.set(next);

    if previous == 0 && next == 1 {
        lock_body_overflow();
    } else if previous == 1 && next == 0 {
        unlock_body_overflow();
    }
}

// `acquire_or_release` (their only caller) only compiles under `web`/`native`, so unlike
// `lock_body_overflow`/`unlock_body_overflow` elsewhere in this crate's other target-gated
// helpers, these have no SSR/server no-op counterpart to define -- `use_scroll_lock`'s own
// no-op stub above already short-circuits before reaching this file's refcounting machinery
// at all on that target.
#[cfg(any(feature = "web", feature = "native"))]
fn lock_body_overflow() {
    let _ = document::eval(
        "document.body.dataset.adicoScrollLockOverflow = document.body.style.overflow;
        document.body.style.overflow = 'hidden';",
    );
}

#[cfg(any(feature = "web", feature = "native"))]
fn unlock_body_overflow() {
    let _ = document::eval(
        "document.body.style.overflow = document.body.dataset.adicoScrollLockOverflow || '';
        delete document.body.dataset.adicoScrollLockOverflow;",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refcount_only_reaches_zero_once_every_holder_releases() {
        let mut count = next_count(0, true); // overlay A opens
        count = next_count(count, true); // overlay B opens
        assert_eq!(count, 2);

        count = next_count(count, false); // overlay A closes
        assert_eq!(count, 1, "still locked while B is open");

        count = next_count(count, false); // overlay B closes
        assert_eq!(count, 0, "unlocked once every holder has released");
    }

    #[test]
    fn releasing_below_zero_saturates_instead_of_underflowing() {
        assert_eq!(next_count(0, false), 0);
    }
}
