//! Shared, debounced open/close-on-hover intent, generic over the requested value.
//!
//! Extracted per `openspec/changes/deduplicate-primitives` (D1): the same
//! generation-counter + `crate::time::sleep` body existed independently in `menu.rs`
//! (`MenuContext::request_hover_open`), `preview_card.rs` (`PreviewCardCtx::request_open`),
//! and `navigation_menu.rs` (`NavigationMenuCtx::request_open`) -- each file's own comment
//! already admitted this was "the same technique ... not shared code." This module is the
//! one implementation all three (plus `hover_card`, once task 7.1 gives it its own
//! delay-driven open/close path) now compose.
//!
//! Delay selection stays with each caller, not this primitive: `menu`'s open-vs-close delay
//! and `navigation_menu`'s "switching directly between two already-open items applies zero
//! delay" rule both read ambient state this primitive has no access to (which of two
//! configured delays applies; whether something else is already open), so each caller
//! resolves its own `delay_ms` and passes it to [`HoverIntent::request`] as a plain
//! argument -- see `design.md`'s D1 section for why an earlier draft's `skip_delay_when`
//! predicate parameter on the primitive itself was dropped in favor of this.

use dioxus::prelude::*;
use std::time::Duration;

/// A debounced, cancelable request to apply some value `T` (for example, a hover-open/close
/// boolean, or `navigation_menu`'s `Option<usize>` "which item is open") after a
/// caller-supplied delay. `Copy` is load-bearing: this is meant to be stored as a plain field
/// on a `#[derive(Clone, Copy)]` context struct (as every one of its three call sites already
/// is, since each flows through `use_context_provider`/`use_context` into event handlers
/// outside render) rather than returned as a non-`Copy` closure.
#[derive(Clone, Copy)]
pub struct HoverIntent<T: Copy + PartialEq + 'static> {
    setter: Callback<T>,
    /// Bumped on every `request`; a pending request only applies its effect if this
    /// counter hasn't moved on (to a newer request) since it started waiting.
    generation: Signal<u64>,
}

impl<T: Copy + PartialEq + 'static> HoverIntent<T> {
    /// Requests applying `value` after `delay_ms` milliseconds, superseding (silently
    /// dropping, not merely overwriting) any still-pending earlier request for this same
    /// `HoverIntent`.
    ///
    /// `delay_ms == 0` still resolves through a spawned task rather than applying
    /// synchronously in the caller's own frame -- consistent with what every one of the
    /// three implementations this consolidates already did (each unconditionally spawned;
    /// only the sleep inside was conditional on the delay being non-zero). It resolves on
    /// the very next microtask, with no timer wait.
    pub fn request(&self, value: T, delay_ms: u64) {
        let mut generation = self.generation;
        let this_generation = generation() + 1;
        generation.set(this_generation);

        let setter = self.setter;
        let generation = self.generation;
        spawn(async move {
            if delay_ms > 0 {
                crate::time::sleep(Duration::from_millis(delay_ms)).await;
            }
            if generation() == this_generation {
                setter.call(value);
            }
        });
    }
}

/// Creates a [`HoverIntent<T>`] that calls `setter` with the winning value of each
/// generation-debounced [`HoverIntent::request`].
pub fn use_hover_intent<T: Copy + PartialEq + 'static>(setter: Callback<T>) -> HoverIntent<T> {
    let generation = use_signal(|| 0u64);
    HoverIntent { setter, generation }
}
