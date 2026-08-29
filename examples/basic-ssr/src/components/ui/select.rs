//! Source-owned Select composition backed by adico's audited primitive layer.
//!
//! This initial registry façade intentionally preserves the primitive's
//! compositional Dioxus API. Consumers install and own this module, so they can
//! add local styling wrappers or replace individual exported parts without
//! coupling their application to an opaque styled-component crate.

pub use adico_primitives::select::{
    Select, SelectGroup, SelectGroupLabel, SelectItemIndicator, SelectList, SelectMulti,
    SelectOption, SelectTrigger, SelectValue,
};
