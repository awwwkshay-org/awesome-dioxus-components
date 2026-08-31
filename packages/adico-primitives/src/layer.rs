// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shared overlay/layer stacking: a single ordered registry of every
//! currently-mounted overlay (dialog, popover, context menu, etc.), used both
//! to decide which layer dismiss events (Escape, outside pointer/focus)
//! target and to derive a stable stacking `z_index` for CSS so nested
//! overlays paint in mount order. [`crate::use_escape_key`] and
//! [`crate::use_outside_dismiss`] both register through [`use_layer`].

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::core::{current_scope_id, use_drop};
use dioxus::prelude::*;

/// The base CSS `z-index` the bottommost layer receives; each layer above it
/// gets `BASE_Z_INDEX + depth`.
const BASE_Z_INDEX: i32 = 1000;

#[derive(Clone, Default)]
struct LayerStack(Rc<RefCell<Vec<ScopeId>>>);

/// A single caller's registration on the shared layer stack.
///
/// Callers are ordered on a LIFO stack keyed by their component scope, so
/// when several overlays are nested the most-recently-mounted one is
/// topmost. An overlay that registers more than once (e.g. a component
/// calling both [`crate::use_escape_key`] and [`crate::use_outside_dismiss`])
/// occupies exactly one position, not one per call.
#[derive(Clone)]
pub struct Layer {
    scope_id: ScopeId,
    stack: LayerStack,
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

/// Register the calling component as a layer on the shared overlay stack.
/// Safe to call more than once per component — the scope is only pushed once
/// and removed once its last registration drops.
pub fn use_layer() -> Layer {
    let scope_id = current_scope_id();
    let stack = use_hook(move || {
        let stack: LayerStack =
            try_consume_context().unwrap_or_else(|| provide_context(LayerStack::default()));
        {
            let mut layers = stack.0.borrow_mut();
            if !layers.contains(&scope_id) {
                layers.push(scope_id);
            }
        }
        stack
    });
    use_drop({
        let stack = stack.clone();
        move || stack.0.borrow_mut().retain(|id| *id != scope_id)
    });
    Layer { scope_id, stack }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack_with(scopes: &[usize]) -> LayerStack {
        let stack = LayerStack(Rc::new(RefCell::new(Vec::new())));
        stack
            .0
            .borrow_mut()
            .extend(scopes.iter().map(|id| ScopeId(*id)));
        stack
    }

    fn layer_at(stack: &LayerStack, scope_id: usize) -> Layer {
        Layer {
            scope_id: ScopeId(scope_id),
            stack: stack.clone(),
        }
    }

    #[test]
    fn a_lone_layer_is_topmost_at_depth_zero() {
        let stack = stack_with(&[0]);
        let only = layer_at(&stack, 0);

        assert!(only.is_topmost());
        assert_eq!(only.depth(), 0);
        assert_eq!(only.z_index(), BASE_Z_INDEX);
    }

    #[test]
    fn three_nested_layers_have_increasing_depth_and_z_index() {
        let stack = stack_with(&[0, 1, 2]);
        let outer = layer_at(&stack, 0);
        let middle = layer_at(&stack, 1);
        let inner = layer_at(&stack, 2);

        assert_eq!(outer.depth(), 0);
        assert_eq!(middle.depth(), 1);
        assert_eq!(inner.depth(), 2);
        assert!(outer.z_index() < middle.z_index());
        assert!(middle.z_index() < inner.z_index());

        assert!(!outer.is_topmost());
        assert!(!middle.is_topmost());
        assert!(inner.is_topmost());
    }

    #[test]
    fn removing_the_topmost_layer_restores_the_next_ones_topmost_status() {
        let stack = stack_with(&[0, 1]);
        let outer = layer_at(&stack, 0);

        stack.0.borrow_mut().retain(|id| *id != ScopeId(1));

        assert!(outer.is_topmost());
        assert_eq!(outer.depth(), 0);
    }

    /// Exercises the real hook (not the raw stack), verifying that a
    /// component calling [`use_layer`] twice still occupies exactly one slot
    /// on the shared stack rather than two.
    #[test]
    fn calling_the_hook_twice_in_one_component_registers_one_layer() {
        #[component]
        fn Root() -> Element {
            let first = use_layer();
            let second = use_layer();
            let layer_count = first.stack.0.borrow().len();

            assert_eq!(
                layer_count, 1,
                "duplicate hook calls must not duplicate the layer"
            );
            assert!(first.is_topmost());
            assert!(second.is_topmost());

            rsx! { "ok" }
        }

        let mut dom = VirtualDom::new(Root);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("ok"));
    }
}
