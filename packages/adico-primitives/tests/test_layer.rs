//! Black-box tests for `adico_primitives::layer`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/*.rs`. Most of these
//! tests pass `ReadSignal::new(Signal::new(true))` (always open) to `use_layer` since they
//! exercise the stack's depth/ownership/topmost mechanics, not the open-tracking behavior
//! itself -- see the `a_layer_*`-prefixed tests at the bottom of this file for that.

use adico_primitives::layer::{Layer, LayerStack, use_layer, use_layer_member};
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

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
        stack: LayerStack(stack.0.clone()),
    }
}

#[test]
fn a_lone_layer_is_topmost_at_depth_zero() {
    let stack = stack_with(&[0]);
    let only = layer_at(&stack, 0);

    assert!(only.is_topmost());
    assert_eq!(only.depth(), 0);
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

/// Exercises the real hook (not the raw stack), verifying that a component calling
/// [`use_layer`] twice still occupies exactly one slot on the shared stack rather than two.
#[test]
fn calling_use_layer_twice_in_one_component_registers_one_layer() {
    #[component]
    fn Root() -> Element {
        let first = use_layer(ReadSignal::new(Signal::new(true)));
        let second = use_layer(ReadSignal::new(Signal::new(true)));
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

/// The regression this file exists to prevent: an overlay's "root" component (which calls
/// `use_escape_key` -> `use_layer`) and its "content" component (which calls
/// `use_outside_dismiss` -> `use_layer_member`, in a *separate* child scope) must resolve to
/// the same layer, so the root's own `is_topmost()` check stays true even after the content
/// mounts. Before the fix, `DialogContent`'s independent `use_layer` call always pushed a
/// later, higher-priority stack entry than `DialogRoot`'s own, permanently shadowing
/// `DialogRoot`'s Escape handling.
#[test]
fn root_and_member_in_separate_scopes_share_one_layer_slot() {
    #[component]
    fn Content() -> Element {
        let member = use_layer_member();
        assert!(
            member.is_topmost(),
            "the content scope's layer must still read as topmost"
        );
        rsx! { "content" }
    }

    #[component]
    fn Root() -> Element {
        let root_layer = use_layer(ReadSignal::new(Signal::new(true)));
        let slot_count = root_layer.stack.0.borrow().len();
        assert_eq!(
            slot_count, 1,
            "root + content must share one stack slot, not register two"
        );
        assert!(
            root_layer.is_topmost(),
            "the root's own layer must remain topmost after content joins it"
        );
        rsx! {
            Content {}
        }
    }

    let mut dom = VirtualDom::new(Root);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("content"));
}

/// A *genuinely nested* overlay (one overlay's root mounted inside another overlay's content)
/// must still resolve to its own, distinct layer slot — `use_layer`'s marker must not leak
/// across a nested root's own `use_layer` call, only across a same-overlay `use_layer_member`
/// call. Otherwise nested dialogs would collapse onto one shared layer and neither could tell
/// it apart from the other for Escape/outside-dismiss purposes.
#[test]
fn a_nested_overlays_own_root_gets_a_distinct_layer_slot() {
    #[component]
    fn InnerContent() -> Element {
        let member = use_layer_member();
        assert!(member.is_topmost());
        rsx! { "inner-content" }
    }

    #[component]
    fn InnerRoot() -> Element {
        let inner_layer = use_layer(ReadSignal::new(Signal::new(true)));
        assert!(
            inner_layer.is_topmost(),
            "the nested overlay's own root must be topmost once mounted"
        );
        rsx! {
            InnerContent {}
        }
    }

    #[component]
    fn OuterContent() -> Element {
        let outer_member = use_layer_member();
        rsx! {
            InnerRoot {}
            "outer-content:{outer_member.depth()}"
        }
    }

    #[component]
    fn OuterRoot() -> Element {
        let outer_layer = use_layer(ReadSignal::new(Signal::new(true)));
        rsx! {
            OuterContent {}
            "outer-root-depth:{outer_layer.depth()}"
        }
    }

    let mut dom = VirtualDom::new(OuterRoot);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    // Two distinct slots: the outer overlay (root+content sharing depth 0) and the inner
    // overlay's own root (depth 1), not one shared slot for everything.
    assert!(html.contains("outer-root-depth:0"), "{html}");
    assert!(html.contains("outer-content:0"), "{html}");
    assert!(html.contains("inner-content"), "{html}");
}

/// A layer whose `open` is `false` never occupies a stack slot at all -- the actual fix task
/// 2.3 found missing: a closed-but-mounted overlay (kept mounted for a close animation, or an
/// overlay root a consumer never conditionally renders in the first place) must not be able to
/// shadow anything else's `is_topmost()` check merely by existing in the tree.
#[test]
fn a_layer_with_open_false_occupies_no_stack_slot() {
    #[component]
    fn ClosedRoot() -> Element {
        let layer = use_layer(ReadSignal::new(Signal::new(false)));
        assert_eq!(
            layer.stack.0.borrow().len(),
            0,
            "a closed root must not register a slot"
        );
        assert!(!layer.is_topmost());
        rsx! { "closed" }
    }

    let mut dom = VirtualDom::new(ClosedRoot);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("closed"));
}

/// The concrete regression this fix targets: two independent (sibling, not nested) overlay
/// roots, one closed and one open. Before this fix, `use_layer` pushed a slot unconditionally
/// on mount regardless of `open`, so the *mount order* -- not which overlay was actually
/// showing -- decided `is_topmost()`. A closed root mounted in either position must never be
/// topmost, and the one truly open root must be, regardless of which one mounted first.
#[test]
fn a_closed_sibling_never_shadows_an_open_ones_topmost_status() {
    #[component]
    fn ClosedSibling() -> Element {
        let layer = use_layer(ReadSignal::new(Signal::new(false)));
        assert!(
            !layer.is_topmost(),
            "the closed sibling must never be topmost"
        );
        rsx! { "closed-child" }
    }

    #[component]
    fn OpenSibling() -> Element {
        let layer = use_layer(ReadSignal::new(Signal::new(true)));
        assert!(
            layer.is_topmost(),
            "the only actually-open sibling must be topmost"
        );
        assert_eq!(
            layer.stack.0.borrow().len(),
            1,
            "the closed sibling must not have occupied a slot"
        );
        rsx! { "open-child" }
    }

    #[component]
    fn Parent() -> Element {
        rsx! {
            ClosedSibling {}
            OpenSibling {}
        }
    }

    let mut dom = VirtualDom::new(Parent);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("closed-child"));
    assert!(html.contains("open-child"));
}

/// `use_layer_member`'s no-ancestor-owner fallback (the real path `context_menu.rs` exercises
/// today, since it has no `use_layer`-registering ancestor of its own) registers its own,
/// always-open layer rather than silently never registering.
#[test]
fn a_layer_member_without_an_ancestor_owner_registers_its_own_always_open_layer() {
    #[component]
    fn Standalone() -> Element {
        let member = use_layer_member();
        assert!(
            member.is_topmost(),
            "a standalone caller with no use_layer ancestor must still register and be topmost"
        );
        rsx! { "standalone" }
    }

    let mut dom = VirtualDom::new(Standalone);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("standalone"));
}
