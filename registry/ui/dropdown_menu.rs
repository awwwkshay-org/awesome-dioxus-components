//! Source-owned Dropdown Menu composition backed by adico's audited primitive layer.
//!
//! This initial registry façade intentionally preserves the primitive's
//! compositional Dioxus API, matching the approach already used by `select`:
//! `DropdownMenuItem` is generic, so consumers install and own this module
//! and apply Tailwind classes directly to each part rather than coupling to
//! an opaque styled-component crate.

pub use adico_primitives::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
