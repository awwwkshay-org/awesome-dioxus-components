// SPDX-License-Identifier: MIT OR Apache-2.0
// Forked from DioxusLabs/dioxus-components at bf007c15d0cf4d04d3181cc46cf12325aa773955.
// Upstream path: primitives/src/portal.rs. See provenance/records/adico-primitives-wave2-risk.json.
//
// Ported unmodified. This is a pure-Rust "logical" portal implemented purely
// through Dioxus's own scope/context system (relaying a VNode from one place
// in the component tree to another within the same virtual DOM) -- not a
// `document::eval`/DOM-based portal. It has no browser API dependency and is
// SSR-safe by construction, unlike the `document::eval`-based positioning
// this crate has deliberately avoided elsewhere (see the Wave 3 record's
// note that only `toast.rs` actually depends on this module upstream).

//! A logical, VDOM-only content relay.
//!
//! This portal moves a [`dioxus::prelude::Element`] from one place in the
//! component tree to another *within the same virtual DOM* — it is not a
//! `document::eval`/DOM-based portal and cannot escape a CSS `overflow`,
//! `transform`, or stacking context. It is SSR-safe by construction, with no
//! browser API dependency. A DOM-based portal with `Backdrop`/`Viewport`
//! parts, needed for overlays to layer above ancestor stacking contexts, is
//! tracked separately (see design.md §8a).

use crate::dioxus_core::provide_root_context;
use dioxus::prelude::*;
use std::collections::HashMap;

use crate::use_effect_cleanup;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalId(usize);

#[derive(Clone, Copy, PartialEq)]
struct PortalCtx {
    portals: Signal<HashMap<usize, Signal<Element>>>,
}

/// Create a portal.
pub fn use_portal() -> PortalId {
    static NEXT_ID: GlobalSignal<usize> = Signal::global(|| 0);

    let (sig, id) = use_hook(|| {
        let mut next_id = NEXT_ID.write();
        let id = *next_id;
        *next_id += 1;

        let mut ctx = match try_consume_context::<PortalCtx>() {
            Some(ctx) => ctx,
            None => {
                let portals = Signal::new_in_scope(HashMap::new(), ScopeId::ROOT);
                let ctx = PortalCtx { portals };
                provide_root_context(ctx)
            }
        };

        let sig = Signal::new_in_scope(VNode::empty(), ScopeId::ROOT);
        ctx.portals.write().insert(id, sig);

        (sig, PortalId(id))
    });

    // Cleanup the portal.
    use_effect_cleanup(move || {
        let mut ctx = consume_context::<PortalCtx>();
        ctx.portals.write().remove(&id.0);
        sig.manually_drop();
    });

    id
}

#[component]
pub fn PortalIn(portal: PortalId, children: Element) -> Element {
    if let Some(mut ctx) = try_use_context::<PortalCtx>() {
        let mut portals = ctx.portals.write();
        if let Some(portal) = portals.get_mut(&portal.0) {
            portal.set(children);
        }
    }

    rsx! {}
}

#[component]
pub fn PortalOut(portal: PortalId) -> Element {
    if let Some(ctx) = try_use_context::<PortalCtx>() {
        if let Some(children) = ctx.portals.peek().get(&portal.0) {
            return rsx! {
                {*children}
            };
        }
    }

    rsx! {}
}
