// SPDX-License-Identifier: MIT OR Apache-2.0
// Forked from DioxusLabs/dioxus-components at bf007c15d0cf4d04d3181cc46cf12325aa773955.
// Upstream path: primitives/src/virtual_list.rs. See provenance/records/adico-primitives-wave5-extras.json.
//
// Adapted from upstream: real browser testing (Playwright, not just compile
// checks) found that upstream's scroll/resize bridge -- a long-lived
// `document::eval` subscription on the container element -- never registers
// in this Dioxus 0.7.9/0.7.10 web runtime, matching the exact defect class
// already recorded for `use_global_escape_listener`/`use_outside_dismiss` in
// docs/adico/m3-wave3-migration.md ("Popover's Escape key did nothing"):
// VirtualList rendered zero items because `viewport_size` never left its
// initial `0`. Following that record's own fix pattern (replace the broken
// long-lived eval listener with a native, reliably-firing Dioxus event
// instead of root-causing the interpreter-level registration failure), this
// module replaces the JS scroll bridge entirely with native `onscroll`
// (`dioxus::html::events::ScrollData`, which already carries `scroll_top`/
// `client_height`) and `onmounted`/`MountedData` for the initial measurement
// and for programmatically correcting scroll position -- the same
// `MountedData` API already used unconditionally (no target gating needed)
// by `move_interaction.rs`/`checkbox.rs`/`slider.rs` elsewhere in this crate.
// This removes the `serde`/`ScrollMsg` JS-payload plumbing entirely and is a
// strictly more reliable implementation of upstream's own documented
// behavior, not a behavior reduction. Scroll-end debouncing (upstream used a
// 600ms JS `setTimeout`) is now a Rust-side generation-counter timer using
// this crate's own target-aware `crate::time::sleep`.

//! Defines the [`VirtualList`] component for rendering large lists with virtualization.

use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use dioxus::html::geometry::euclid::Vector2D;
use dioxus::prelude::*;

use crate::r#virtual::{
    VirtualizerState, VirtualizerStateStoreExt, compute_measurements, get_total_size,
    get_virtual_items, resize_item, set_scroll_offset, set_viewport_size,
};

/// The props for the [`VirtualList`] component.
#[derive(Props, Clone, PartialEq)]
pub struct VirtualListProps {
    /// The total number of items in the list.
    pub count: ReadSignal<usize>,
    /// The amount of render buffer (in estimated row counts) above and below the viewport.
    #[props(default = ReadSignal::new(Signal::new(8)))]
    pub buffer: ReadSignal<usize>,
    /// Estimates the height of an item by index (used before measurement).
    /// For best scrollbar stability, return values close to actual heights.
    /// If not provided, uses adaptive estimation based on measured items.
    pub estimate_size: Option<Callback<usize, u32>>,
    /// Renders a single item by its absolute index.
    pub render_item: Callback<usize, Element>,
    /// Additional attributes to apply to the container element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # VirtualList
///
/// The `VirtualList` component virtualizes a large list by rendering only the visible slice plus a
/// configurable buffer. It supports dynamic row heights and keeps total scroll height with a
/// virtual canvas.
///
/// Each rendered item receives `aria-setsize` and `aria-posinset` attributes for accessibility,
/// allowing screen readers to announce the total list size even though only a subset of items
/// is present in the DOM.
///
/// The scroll/resize bridge relies on renderer support for `onscroll`/`onmounted` element
/// queries (present in the web and desktop renderers); on a renderer without that support the
/// container never receives a nonzero viewport size and renders zero items rather than failing
/// to build.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::virtual_list::VirtualList;
///
/// #[derive(Clone, PartialEq)]
/// struct Row {
///     title: String,
/// }
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         VirtualList {
///             count: 100usize,
///             buffer: 8usize,
///             // Optional: estimate height per item for smoother scrolling
///             // If omitted, uses adaptive estimation based on measured items
///             estimate_size: |_idx| 48,
///             render_item: move |idx: usize| rsx! {
///                 article { key: "{idx}", "Row {idx}" }
///             },
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`VirtualList`] component renders a container `div` with the class `dx-virtual-list-container`.
/// All user-provided `attributes` are spread onto the container element.
#[component]
pub fn VirtualList(props: VirtualListProps) -> Element {
    let VirtualListProps {
        count,
        buffer,
        estimate_size,
        render_item,
        attributes,
    } = props;

    let container_id = crate::use_unique_id();
    let mut container_handle: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    let mut scroll_generation = use_signal(|| 0u64);

    // Create the Store — only holds mutable shared state
    let state: Store<VirtualizerState> = use_store(|| VirtualizerState {
        scroll_offset: 0,
        viewport_size: 0,
        is_scrolling: false,
        item_size_cache: HashMap::new(),
        scroll_adjustments: 0,
        stable_total_size: None,
        stable_measurement_count: None,
        deferred_adjustments: 0,
    });

    // Measurements as a memo — recomputes when count or item_size_cache change.
    // Read (not peeked) by the render body so the component re-renders when the
    // memo invalidates; peeking a dirty memo returns stale data (Memo::peek does
    // not check the dirty flag), which can yield out-of-bounds indices when
    // `count` shrinks between renders.
    let measurements: Memo<Vec<crate::r#virtual::types::VirtualItem>> = use_memo(move || {
        let count = count();
        let isc = state.item_size_cache();
        let item_size_cache = isc.read();
        let estimate_cb = estimate_size.as_ref().map(|c| move |i: usize| c(i));
        compute_measurements(
            count,
            &item_size_cache,
            estimate_cb.as_ref().map(|f| f as &dyn Fn(usize) -> u32),
        )
    });

    let onmounted = move |event: Event<MountedData>| {
        let handle = event.data();
        spawn({
            let handle = handle.clone();
            async move {
                if let Ok(rect) = handle.get_client_rect().await {
                    set_viewport_size(&state, rect.height().round() as u32);
                }
            }
        });
        container_handle.set(Some(handle));
    };

    let onscroll = move |event: Event<ScrollData>| {
        let data = event.data();
        let offset = data.scroll_top().max(0.0).round() as u32;
        let viewport = data.client_height().max(0) as u32;
        if viewport > 0 {
            set_viewport_size(&state, viewport);
        }
        {
            let m = measurements.peek();
            set_scroll_offset(&state, &m, offset, true);
        }

        scroll_generation.with_mut(|generation| *generation += 1);
        let this_generation = *scroll_generation.peek();

        spawn(async move {
            crate::time::sleep(Duration::from_millis(600)).await;
            if *scroll_generation.peek() != this_generation {
                // A newer scroll event superseded this debounce window.
                return;
            }
            let correction = {
                let m = measurements.peek();
                set_scroll_offset(&state, &m, offset, false)
            };
            if let Some(delta) = correction {
                let new_scroll = (offset as i32 + delta).max(0) as u32;
                state.scroll_offset().set(new_scroll);
                if let Some(handle) = container_handle.peek().clone() {
                    let _ = handle
                        .scroll(
                            Vector2D::new(0.0, new_scroll as f64),
                            ScrollBehavior::Instant,
                        )
                        .await;
                }
            }
        });
    };

    let onresize_container = move |event: Event<ResizeData>| {
        let rect = event.data().get_content_box_size().unwrap_or_default();
        let viewport = rect.height.max(0.0).round() as u32;
        if viewport > 0 {
            set_viewport_size(&state, viewport);
        }
    };

    let onresize_item = move |idx| {
        move |event: Event<ResizeData>| {
            let rect = event.data().get_content_box_size().unwrap_or_default();
            let measured = rect.height.max(1.0).round() as u32;

            let m = measurements.peek();
            let adjustment = resize_item(&state, &m, idx, measured);
            drop(m);

            if let Some(delta) = adjustment {
                let current = *state.scroll_offset().peek();
                let new_scroll = (current as i32 + delta).max(0) as u32;
                state.scroll_offset().set(new_scroll);
                spawn(async move {
                    if let Some(handle) = container_handle.peek().clone() {
                        let _ = handle
                            .scroll(
                                Vector2D::new(0.0, new_scroll as f64),
                                ScrollBehavior::Instant,
                            )
                            .await;
                    }
                });
            }
        }
    };

    let m = measurements.read();
    let virtual_items = get_virtual_items(&state, &m, buffer());
    let total_height = get_total_size(&state, &m);

    let top_offset = virtual_items.first().map(|item| item.start()).unwrap_or(0);
    let canvas_height = total_height.max(*state.viewport_size().peek());
    let set_size = count.to_string();

    rsx! {
        div {
            id: container_id,
            role: "list",
            tabindex: "0",
            onmounted,
            onscroll,
            onresize: onresize_container,
            ..attributes,

            div {
                style: "position: relative; height:{canvas_height}px; width: 100%;",
                div {
                    style: "position: absolute; inset: 0 auto auto 0; width: 100%; transform: translateY({top_offset}px); will-change: transform;",
                    {virtual_items.iter().map(move |item| {
                        let idx = item.index();

                        rsx! {
                            div {
                                key: "{item.key()}",
                                role: "listitem",
                                "data-virtual-index": "{idx}",
                                "aria-setsize": "{set_size}",
                                "aria-posinset": "{idx + 1}",
                                onresize: onresize_item(idx),
                                {render_item(idx)}
                            }
                        }
                    })}
                }
            }
        }
    }
}
