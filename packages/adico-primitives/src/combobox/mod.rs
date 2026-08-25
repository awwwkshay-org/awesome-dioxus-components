// SPDX-License-Identifier: MIT OR Apache-2.0
// Forked from DioxusLabs/dioxus-components at bf007c15d0cf4d04d3181cc46cf12325aa773955.
// Upstream path: primitives/src/combobox/mod.rs. See provenance/records/adico-primitives-wave4-collection.json.

//! Autocomplete input with a filterable popup list.
//!
//! `ComboboxInput` is the text input and trigger. `ComboboxList` contains
//! `ComboboxOption` children.

mod components;
mod context;

pub use components::{
    Combobox, ComboboxEmpty, ComboboxEmptyProps, ComboboxInput, ComboboxInputProps,
    ComboboxItemIndicator, ComboboxItemIndicatorProps, ComboboxList, ComboboxListProps,
    ComboboxOption, ComboboxOptionProps, ComboboxProps,
};

pub use context::default_combobox_filter;
