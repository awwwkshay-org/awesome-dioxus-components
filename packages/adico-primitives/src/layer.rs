// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shared overlay/layer stacking: a single ordered registry of every
//! currently-*open* overlay (dialog, popover, context menu, etc.), used both
//! to decide which layer dismiss events (Escape, outside pointer/focus)
//! target and to derive a stable stacking `z_index` for CSS so nested
//! overlays paint in open order. [`crate::use_escape_key`] and
//! [`crate::use_outside_dismiss`] both register through [`use_layer`].
//!
//! A layer joins the stack while its `open` state is `true` and leaves it as
//! soon as `open` becomes `false` -- not merely when its component unmounts
//! (`use_drop` is still a safety-net for the unmount-without-closing case).
//! This matches Floating UI/Base UI's model, where a floating element's
//! dismiss listeners are scoped to its own `open` boolean rather than a
//! separate "mounted" registry: an overlay's *root* component (e.g.
//! `DialogRoot`) commonly stays mounted for its whole lifetime regardless of
//! `open`, only its *content* conditionally renders, so a mount-based stack
//! would have permanently misattributed "topmost" status to whichever root
//! happened to mount first rather than whichever overlay is actually open
//! right now -- exactly the closed-but-still-registered footgun this design
//! replaces (see `openspec/changes/reauthor-primitives-from-independent-spec/
//! tasks.md` task 2.3's finding).

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::core::{current_scope_id, use_drop};
use dioxus::prelude::*;

/// The base CSS `z-index` the bottommost layer receives; each layer above it
/// gets `BASE_Z_INDEX + depth`.
const BASE_Z_INDEX: i32 = 1000;

/// `pub` field only for `packages/adico-primitives/tests/` to build raw fixtures without
/// going through the hooks; not part of this crate's intended public surface.
#[derive(Clone, Default)]
pub struct LayerStack(pub Rc<RefCell<Vec<ScopeId>>>);

/// A single caller's registration on the shared layer stack.
///
/// Callers are ordered on a LIFO stack keyed by their component scope, so
/// when several overlays are nested the most-recently-mounted one is
/// topmost. An overlay that registers more than once (e.g. a component
/// calling both [`crate::use_escape_key`] and [`crate::use_outside_dismiss`])
/// occupies exactly one position, not one per call, via [`use_layer_member`]
/// joining the [`use_layer`]-registering component's own slot.
#[derive(Clone)]
pub struct Layer {
    /// `pub` only for `packages/adico-primitives/tests/`; not part of the intended API.
    pub scope_id: ScopeId,
    /// `pub` only for `packages/adico-primitives/tests/`; not part of the intended API.
    pub stack: LayerStack,
}

impl Layer {
    /// Whether this is the most-recently-mounted layer still registered.
    /// Dismiss channels (Escape, outside pointer/focus) only act for the
    /// topmost layer, so an inner overlay never lets an outer one dismiss
    /// first.
    pub fn is_topmost(&self) -> bool {
        self.stack.0.borrow().last() == Some(&self.scope_id)
    }

    /// This layer's position from the bottom of the stack (`0` for the
    /// first-mounted, still-registered layer).
    pub fn depth(&self) -> usize {
        self.stack
            .0
            .borrow()
            .iter()
            .position(|id| *id == self.scope_id)
            .unwrap_or_default()
    }

    /// A CSS `z-index` for this layer, increasing with mount order so later
    /// (more deeply nested) overlays paint above earlier ones.
    pub fn z_index(&self) -> i32 {
        BASE_Z_INDEX + self.depth() as i32
    }
}

/// Marks which [`Layer`] the calling scope's overlay "owns", so other
/// components belonging to the same logical overlay (e.g. a modal's content,
/// registering separately from its root) can join that layer instead of
/// each occupying their own stack slot. Provided by [`use_layer`], read by
/// [`use_layer_member`].
#[derive(Clone)]
struct LayerOwner(Layer);

/// Register the calling component as a fresh layer on the shared overlay
/// stack while `open()` is `true`, always creating its own slot — this is the
/// entry point for whichever component establishes an overlay's identity (its
/// "root": `DialogRoot`, `PopoverRoot`, and similarly-shaped components). It
/// also provides a [`LayerOwner`] marker, scoped to its own subtree, so
/// descendant calls to [`use_layer_member`] — including a *nested* overlay's
/// own root, which must not join an ancestor's layer — resolve correctly: a
/// nested root calls this function too, and providing its own marker here
/// shadows the outer one for everything inside it.
///
/// The very first render joins the stack synchronously if `open()` is
/// already `true` (so [`Layer::is_topmost`] is correct immediately, even
/// under a bare `rebuild_in_place()` that never drives an effect to
/// completion); every render after that reacts to `open` changing via a
/// `use_effect`, matching this crate's established
/// [`crate::scroll_lock::use_scroll_lock`] convention for reactive,
/// DOM-adjacent shared state.
pub fn use_layer(open: impl Readable<Target = bool> + Copy + 'static) -> Layer {
    let scope_id = current_scope_id();
    let stack = use_hook(move || {
        let stack: LayerStack =
            try_consume_context().unwrap_or_else(|| provide_context(LayerStack::default()));
        if open.cloned() {
            let mut layers = stack.0.borrow_mut();
            if !layers.contains(&scope_id) {
                layers.push(scope_id);
            }
        }
        stack
    });
    use_effect({
        let stack = stack.clone();
        move || {
            let mut layers = stack.0.borrow_mut();
            let present = layers.contains(&scope_id);
            match (open.cloned(), present) {
                (true, false) => layers.push(scope_id),
                (false, true) => layers.retain(|id| *id != scope_id),
                _ => {}
            }
        }
    });
    use_drop({
        let stack = stack.clone();
        move || stack.0.borrow_mut().retain(|id| *id != scope_id)
    });
    let layer = Layer { scope_id, stack };
    use_hook(|| provide_context(LayerOwner(layer.clone())));
    layer
}

/// Join the nearest ancestor overlay's [`use_layer`] registration instead of
/// registering a new stack slot — the entry point for a component that is
/// part of the *same* logical overlay as an ancestor that already called
/// [`use_layer`] (e.g. `DialogContent` joining its `DialogRoot`'s layer, so
/// both agree on [`Layer::is_topmost`] rather than the later-mounted content
/// always shadowing its own root). Falls back to registering its own,
/// always-open layer if no owner is found — a case that does happen in
/// practice today ([`crate::context_menu`] has no `use_layer`-registering
/// ancestor of its own), so this treats a missing owner as "always open"
/// (preserving the old unconditional-registration behavior for that caller)
/// rather than panicking or silently never registering.
pub fn use_layer_member() -> Layer {
    let owner = use_hook(try_consume_context::<LayerOwner>);
    match owner {
        Some(owner) => owner.0,
        None => use_layer(ReadSignal::new(Signal::new(true))),
    }
}
