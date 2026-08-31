//! Black-box tests for `adico_primitives::portal`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/portal.rs`.
//!
//! `portal.rs` is a same-VDOM content relay, not a browser-DOM-escaping portal (see the
//! module's own doc comment) — `use_portal()`/`PortalIn`/`PortalOut` are all already fully
//! public, so no items needed widening for these tests. The spec exercised here: content set
//! by `PortalIn` is visible through `PortalOut` when `PortalIn` renders first (the order
//! `toast.rs`, this module's one real consumer, already uses), a later `PortalIn` overwrites
//! earlier content (there is exactly one slot per portal, not a queue), an unmatched
//! `PortalOut` (wrong id, or before any `PortalIn` targets it) renders nothing, and
//! independent `use_portal()` calls never share a slot. A real, previously-undocumented
//! ordering caveat surfaced while writing these tests — see `an_out_declared_before_its_in_
//! renders_stale_empty_content_on_the_first_pass` — and is now called out in the module's own
//! doc comment rather than left silently discoverable only by tripping over it.

use adico_primitives::portal::{PortalIn, PortalOut, use_portal};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn OutBeforeIn() -> Element {
    let portal = use_portal();
    rsx! {
        div { id: "out", PortalOut { portal } }
        PortalIn { portal, "relayed content" }
    }
}

#[test]
fn an_out_declared_before_its_in_renders_stale_empty_content_on_the_first_pass() {
    // `PortalOut` renders whatever its portal's `Signal<Element>` already holds *at the moment
    // `PortalOut` itself renders* — it has no way to react to a `PortalIn` that hasn't run yet
    // within the same pass, even though both mount in the same `rebuild_in_place()`. This
    // module's one real consumer (`toast.rs`) always declares `PortalIn` before `PortalOut`,
    // which is why this has never surfaced as a bug in practice; a caller that reverses the
    // order (e.g. a viewport placed early in a layout, fed by content declared later) would
    // see nothing until some *later*, independent reactive update happens to re-render
    // `PortalOut`. Documented in the module's own doc comment as a real ordering
    // requirement, not fixed here — this file's spec is the existing same-VDOM relay's actual
    // behavior, not a redesign.
    let html = render(OutBeforeIn);
    assert!(!html.contains("relayed content"), "{html}");
    assert!(html.contains(r#"<div id="out"></div>"#), "{html}");
}

#[component]
fn InBeforeOut() -> Element {
    let portal = use_portal();
    rsx! {
        PortalIn { portal, "relayed content" }
        div { id: "out", PortalOut { portal } }
    }
}

#[test]
fn content_relays_through_the_portal_when_in_is_declared_before_out() {
    let html = render(InBeforeOut);
    assert!(html.contains("relayed content"), "{html}");
}

#[component]
fn OutWithNoMatchingIn() -> Element {
    let portal = use_portal();
    rsx! {
        div { id: "out", PortalOut { portal } }
        "sibling content"
    }
}

#[test]
fn an_out_with_no_matching_in_renders_nothing() {
    let html = render(OutWithNoMatchingIn);
    assert!(html.contains("sibling content"), "{html}");
    assert!(html.contains(r#"id="out""#), "{html}");
    // The `<div id="out">` element itself still renders (it's `PortalOut`'s *caller* markup);
    // only its content, which comes from the never-called `PortalIn`, stays empty.
    assert!(html.contains(r#"<div id="out"></div>"#), "{html}");
}

#[component]
fn LaterInOverwritesEarlierIn() -> Element {
    let portal = use_portal();
    rsx! {
        PortalIn { portal, "first" }
        PortalIn { portal, "second" }
        div { id: "out", PortalOut { portal } }
    }
}

#[test]
fn a_later_in_call_overwrites_an_earlier_one_rather_than_appending() {
    // One `Signal<Element>` slot per portal, not a queue — this is a relay, not a collector.
    let html = render(LaterInOverwritesEarlierIn);
    assert!(!html.contains("first"), "{html}");
    assert!(html.contains("second"), "{html}");
}

#[component]
fn TwoIndependentPortalsDoNotCollide() -> Element {
    let portal_a = use_portal();
    let portal_b = use_portal();
    rsx! {
        PortalIn { portal: portal_a, "content a" }
        PortalIn { portal: portal_b, "content b" }
        div { id: "out-a", PortalOut { portal: portal_a } }
        div { id: "out-b", PortalOut { portal: portal_b } }
    }
}

#[test]
fn two_independent_portals_from_separate_use_portal_calls_do_not_collide() {
    let html = render(TwoIndependentPortalsDoNotCollide);
    let out_a = html
        .split(r#"<div id="out-a">"#)
        .nth(1)
        .and_then(|rest| rest.split("</div>").next())
        .expect("out-a renders");
    let out_b = html
        .split(r#"<div id="out-b">"#)
        .nth(1)
        .and_then(|rest| rest.split("</div>").next())
        .expect("out-b renders");
    assert_eq!(out_a, "content a", "{html}");
    assert_eq!(out_b, "content b", "{html}");
}

#[component]
fn PortalIdsAreStableAcrossRerenders() -> Element {
    let portal = use_portal();
    let mut rendered_twice = use_signal(|| 0);
    // Reading and writing the same signal a component owns forces this component to run more
    // than once within a single `rebuild_in_place()`, without needing an event loop or a real
    // user interaction — `use_portal()`'s `use_hook` must still resolve to the same id the
    // second time, or `PortalIn`/`PortalOut` would silently stop agreeing on which slot to use.
    if rendered_twice() == 0 {
        rendered_twice.set(1);
    }
    rsx! {
        PortalIn { portal, "stable id content" }
        div { id: "out", PortalOut { portal } }
    }
}

#[test]
fn use_portal_s_id_is_stable_across_a_component_s_own_rerenders() {
    let html = render(PortalIdsAreStableAcrossRerenders);
    assert!(html.contains("stable id content"), "{html}");
}
