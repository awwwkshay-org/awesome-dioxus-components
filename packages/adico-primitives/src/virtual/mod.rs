// SPDX-License-Identifier: MIT OR Apache-2.0
// Forked from DioxusLabs/dioxus-components at bf007c15d0cf4d04d3181cc46cf12325aa773955.
// Upstream path: primitives/src/virtual/mod.rs. See provenance/records/adico-primitives-wave5-extras.json.

//! Virtual list implementation using Dioxus Store for fine-grained reactivity.
//!
//! This module provides the core algorithms needed for efficient list virtualization:
//!
//! - Computing item positions from measured or estimated sizes
//! - Calculating the visible range using binary search
//! - Handling scroll position corrections when items resize

pub(crate) mod types;
mod utils;
mod virtualizer;

pub(crate) use virtualizer::{
    VirtualizerState, VirtualizerStateStoreExt, compute_measurements, get_total_size,
    get_virtual_items, resize_item, set_scroll_offset, set_viewport_size,
};
