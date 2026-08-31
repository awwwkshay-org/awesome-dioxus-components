// No ARIA pattern maps onto list virtualization itself (the rendered `role="list"`/
// `role="listitem"` pair, with `aria-setsize`/`aria-posinset` so a screen reader announces the
// true list size despite only a slice being in the DOM, is standard APG guidance for any
// partially-rendered collection); this file's own spec is its existing scroll-window
// contract: item positions derived from measured-or-estimated sizes, the visible range found
// by binary search over those positions plus a buffer, and scroll-position correction when an
// above-viewport item's measured size changes after being estimated. Flattened from
// virtual/{mod,types,utils,virtualizer}.rs (4 files) into this one, per this crate's one-file
// rule.
//
// The scroll/resize bridge itself was already a genuine adaptation, not ported-unmodified:
// real browser testing (Playwright, not just compile checks) found upstream's long-lived
// `document::eval` scroll subscription never registers in this Dioxus 0.7.9/0.7.10 web
// runtime, matching the exact defect class already recorded for
// `use_global_escape_listener`/`use_outside_dismiss` in this change's own layer.rs/positioner.rs
// investigations (Popover's/Select's Escape and positioning both hit variants of it) — so
// VirtualList rendered zero items, `viewport_size` never leaving its initial `0`. Following
// the same fix pattern used elsewhere in this crate (a native, reliably-firing Dioxus event
// instead of root-causing the interpreter-level registration failure), this module replaces
// the JS scroll bridge entirely with native `onscroll` (`dioxus::html::events::ScrollData`,
// which already carries `scroll_top`/`client_height`) and `onmounted`/`MountedData` for the
// initial measurement and for programmatically correcting scroll position — the same
// `MountedData` API already used unconditionally (no target gating needed) by
// `move_interaction.rs`/`checkbox.rs`/`slider.rs` elsewhere in this crate. This removes the
// `serde`/`ScrollMsg` JS-payload plumbing entirely and is a strictly more reliable
// implementation of upstream's own documented behavior, not a behavior reduction.
// Scroll-end debouncing (upstream used a 600ms JS `setTimeout`) is a Rust-side
// generation-counter timer using this crate's own target-aware `crate::time::sleep`.

//! Defines the [`VirtualList`] component for rendering large lists with virtualization.
//!
//! Also holds the pure scroll-window math backing it: computing item positions from
//! measured-or-estimated sizes, finding the visible range via binary search, and applying
//! scroll-position corrections when items resize.

use std::collections::HashMap;
use std::ops::{Range, RangeInclusive};
use std::rc::Rc;
use std::time::Duration;

use dioxus::html::geometry::euclid::Vector2D;
use dioxus::prelude::*;

/// A unique key for identifying items in the virtualizer. `pub` only for
/// `packages/adico-primitives/tests/`; not part of the intended public API.
pub type Key = usize;

/// A single virtualized item with computed position. `pub` only for
/// `packages/adico-primitives/tests/`; not part of the intended public API.
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualItem {
    key: Key,
    index: usize,
    start: u32,
    size: u32,
}

impl VirtualItem {
    pub fn new(key: Key, index: usize, start: u32, size: u32) -> Self {
        Self {
            key,
            index,
            start,
            size,
        }
    }

    pub fn key(&self) -> Key {
        self.key
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn start(&self) -> u32 {
        self.start
    }

    pub fn end(&self) -> u32 {
        self.start + self.size
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}

/// Binary search to find the nearest item at or before the given offset.
///
/// Returns the index of the item whose `start` position is closest to (but not exceeding) the
/// given offset. `pub` only for `packages/adico-primitives/tests/`; not part of the intended
/// public API.
pub fn find_nearest_binary_search(measurements: &[VirtualItem], offset: u32) -> usize {
    measurements
        .binary_search_by(|item| item.start().cmp(&offset))
        .unwrap_or_else(|idx| idx.saturating_sub(1))
}

/// Extract indices from a range with overscan applied. `pub` only for
/// `packages/adico-primitives/tests/`; not part of the intended public API.
pub fn default_range_extractor(
    range: Range<usize>,
    overscan: usize,
    count: usize,
) -> RangeInclusive<usize> {
    if count == 0 {
        return 0..=0;
    }

    let start = range.start.saturating_sub(overscan);
    let end = (range.end + overscan).min(count - 1);

    start..=end
}

/// Reactive virtualizer state.
///
/// Only holds mutable state shared between event handlers and the render body.
/// Prop-derived values live outside the Store and are read directly from signals.
#[derive(Clone, PartialEq, Store)]
pub struct VirtualizerState {
    // --- Reactive (`.read()` in render body → triggers re-renders) ---
    /// Current scroll offset from the container's `scrollTop`.
    pub scroll_offset: u32,
    /// Current viewport height.
    pub viewport_size: u32,
    /// Whether the user is actively scrolling.
    pub is_scrolling: bool,

    // --- Cache (`.peek()` only → never triggers re-renders) ---
    /// Measured sizes keyed by item key, populated by resize callbacks.
    pub item_size_cache: HashMap<Key, u32>,

    // --- Scroll adjustments (`.peek()`, bundled with cache) ---
    /// Accumulated scroll adjustment from items resizing above viewport.
    pub scroll_adjustments: i32,
    /// Frozen total size during active scrolling to prevent scrollbar drift.
    pub stable_total_size: Option<u32>,
    /// Item count the frozen total size was calculated for.
    pub stable_measurement_count: Option<usize>,
    /// Deferred scroll adjustments accumulated while scrolling.
    pub deferred_adjustments: i32,
}

// ---------------------------------------------------------------------------
// Public API – free functions operating on Store<VirtualizerState>
// ---------------------------------------------------------------------------

/// Compute item measurements from current state.
///
/// This is a pure function suitable for use inside a `use_memo`.
/// Adaptive estimation (average of measured sizes) is derived directly
/// from `item_size_cache` rather than tracked separately.
///
/// If `estimate` is provided, it is always used for unmeasured items.
/// Otherwise, falls back to the adaptive average of measured sizes, or 100px.
pub fn compute_measurements(
    count: usize,
    item_size_cache: &HashMap<Key, u32>,
    estimate: Option<&dyn Fn(usize) -> u32>,
) -> Vec<VirtualItem> {
    let adaptive_size = if estimate.is_none() && !item_size_cache.is_empty() {
        let sum: u64 = item_size_cache.values().map(|&v| v as u64).sum();
        Some((sum / item_size_cache.len() as u64) as u32)
    } else {
        None
    };

    let mut measurements = Vec::with_capacity(count);
    for i in 0..count {
        let key = i;
        let size = item_size_cache.get(&key).copied().unwrap_or_else(|| {
            if let Some(est) = estimate {
                est(i)
            } else {
                adaptive_size.unwrap_or(100)
            }
        });

        let start = measurements
            .last()
            .map(|m: &VirtualItem| m.end())
            .unwrap_or(0);
        measurements.push(VirtualItem::new(key, i, start, size));
    }
    measurements
}

/// Handle a scroll event.  Writes reactive fields (`scroll_offset`,
/// `is_scrolling`) which trigger component re-renders.
///
/// Returns an optional correction to apply when scrolling stops.
pub fn set_scroll_offset(
    state: &Store<VirtualizerState>,
    measurements: &[VirtualItem],
    offset: u32,
    is_scrolling: bool,
) -> Option<i32> {
    let was_scrolling = *state.is_scrolling().peek();
    let mut correction = None;

    // Reset adjustments when user starts a new scroll
    if is_scrolling && !was_scrolling {
        state.scroll_adjustments().set(0);
        state.deferred_adjustments().set(0);
        // Freeze total size when scrolling starts to prevent scrollbar drift
        let total = calculate_total_size(measurements);
        state.stable_total_size().set(Some(total));
        state
            .stable_measurement_count()
            .set(Some(measurements.len()));
    }

    // When scrolling stops, apply accumulated deferred adjustments
    if !is_scrolling && was_scrolling {
        state.stable_total_size().set(None);
        state.stable_measurement_count().set(None);

        let deferred = *state.deferred_adjustments().peek();
        if deferred != 0 {
            correction = Some(deferred);
            state.deferred_adjustments().set(0);
        }
    }

    // Reactive writes – these trigger re-renders.
    state.scroll_offset().set(offset);
    state.is_scrolling().set(is_scrolling);

    correction
}

/// Set the viewport size. Reactive write.
pub fn set_viewport_size(state: &Store<VirtualizerState>, size: u32) {
    if *state.viewport_size().peek() != size {
        state.viewport_size().set(size);
    }
}

/// Resize an item and return an optional scroll adjustment.
///
/// Called from resize event handlers.  All access uses `.peek()` so this
/// never triggers a component re-render.
pub fn resize_item(
    state: &Store<VirtualizerState>,
    measurements: &[VirtualItem],
    index: usize,
    new_size: u32,
) -> Option<i32> {
    let item = measurements.get(index)?;
    let key = item.key();
    let item_start = item.start();
    let item_size = item.size();

    // If already measured, only update if significantly different (>2px)
    {
        let isc = state.item_size_cache();
        let size_cache = isc.peek();
        if let Some(&cached_size) = size_cache.get(&key) {
            let remeasure_delta = (new_size as i32 - cached_size as i32).abs();
            if remeasure_delta <= 2 {
                return None;
            }
        }
    }

    let old_size = {
        let isc = state.item_size_cache();
        let size_cache = isc.peek();
        size_cache.get(&key).copied().unwrap_or(item_size)
    };
    let delta = new_size as i32 - old_size as i32;

    // For tiny deltas (sub-pixel rounding), still cache but don't adjust scroll
    let significant_delta = delta.abs() > 1;

    if delta == 0 {
        return None;
    }

    // Only adjust scroll for items ABOVE the viewport.
    let adjusted_scroll = {
        let offset = *state.scroll_offset().peek() as i32;
        let adj = *state.scroll_adjustments().peek();
        (offset + adj).max(0) as u32
    };
    let is_above_viewport = item_start < adjusted_scroll;
    let is_scrolling_now = *state.is_scrolling().peek();
    let should_adjust_now = significant_delta && !is_scrolling_now && is_above_viewport;

    state.item_size_cache().write().insert(key, new_size);

    if should_adjust_now {
        let adj = *state.scroll_adjustments().peek();
        state.scroll_adjustments().set(adj + delta);
        return Some(delta);
    } else if significant_delta && is_scrolling_now && is_above_viewport {
        let deferred = *state.deferred_adjustments().peek();
        state.deferred_adjustments().set(deferred + delta);
    }

    None
}

/// Return the virtual items to render.
///
/// **This is meant to be called in the render body.**  It `.read()`s
/// `scroll_offset` and `viewport_size` to subscribe the component.
pub fn get_virtual_items(
    state: &Store<VirtualizerState>,
    measurements: &[VirtualItem],
    overscan: usize,
) -> Vec<VirtualItem> {
    let range = match calculate_range(state, measurements) {
        Some(r) => r,
        None => return Vec::new(),
    };

    let count = measurements.len();
    let indexes = default_range_extractor(range, overscan, count);

    indexes
        .into_iter()
        .filter_map(|i| measurements.get(i).cloned())
        .collect()
}

/// Return the total scrollable size.
///
/// During active scrolling returns a frozen value to prevent scrollbar drift.
pub fn get_total_size(state: &Store<VirtualizerState>, measurements: &[VirtualItem]) -> u32 {
    let stable_measurement_count = *state.stable_measurement_count().peek();
    if stable_measurement_count == Some(measurements.len())
        && let Some(stable) = *state.stable_total_size().peek()
    {
        return stable;
    }
    calculate_total_size(measurements)
}

fn get_scroll_offset_for_measurements(
    scroll_offset: u32,
    viewport_size: u32,
    measurements: &[VirtualItem],
) -> u32 {
    let max_scroll_offset = calculate_total_size(measurements).saturating_sub(viewport_size);
    scroll_offset.min(max_scroll_offset)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Calculate the visible range.
///
/// `.read()`s `scroll_offset` and `viewport_size` (reactive subscription).
fn calculate_range(
    state: &Store<VirtualizerState>,
    measurements: &[VirtualItem],
) -> Option<Range<usize>> {
    // Reactive reads – subscribes the calling component.
    let scroll_offset = *state.scroll_offset().read();
    let viewport_size = *state.viewport_size().read();

    if measurements.is_empty() || viewport_size == 0 {
        return None;
    }

    if measurements.len() <= 1 {
        return Some(0..measurements.len());
    }

    let scroll_offset =
        get_scroll_offset_for_measurements(scroll_offset, viewport_size, measurements);
    let start_index = find_nearest_binary_search(measurements, scroll_offset);
    let mut end_index = start_index;
    let last_index = measurements.len() - 1;

    while end_index < last_index && measurements[end_index].end() < scroll_offset + viewport_size {
        end_index += 1;
    }

    Some(start_index..(end_index + 1))
}

/// Calculate total size from measurements.
fn calculate_total_size(measurements: &[VirtualItem]) -> u32 {
    measurements.last().map(|m| m.end()).unwrap_or(0)
}

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
    let measurements: Memo<Vec<VirtualItem>> = use_memo(move || {
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
