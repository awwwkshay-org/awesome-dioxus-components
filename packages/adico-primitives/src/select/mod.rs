// SPDX-License-Identifier: MIT OR Apache-2.0
// Forked from DioxusLabs/dioxus-components at bf007c15d0cf4d04d3181cc46cf12325aa773955.
// Upstream path: primitives/src/select/mod.rs. See provenance/records/adico-primitives-dialog-select.json.

//! Defines the [`Select`] component and its sub-components, which provide a searchable select input with keyboard navigation.
//!
//! The Select component consists of several parts that work together:
//! - [`Select`] - The root container component
//! - [`SelectTrigger`] - The button that opens/closes the dropdown
//! - [`SelectList`] - The dropdown container for options
//! - [`SelectOption`] - Individual selectable options
//! - [`SelectItemIndicator`] - Visual indicator for selected items
//! - [`SelectGroup`] - Groups related options together
//! - [`SelectGroupLabel`] - Labels for option groups
//! - [`SelectValue`] - Displays the currently selected value
//!
//! ## Features
//!
//! - **Keyboard Navigation**: Full keyboard support with arrow keys, home/end, enter, and escape
//! - **Typeahead Search**: Smart text search that adapts to different keyboard layouts
//! - **Accessibility**: ARIA compliant with proper roles and attributes
//! - **Customizable**: Flexible styling through data attributes and CSS
//! - **Focus Management**: Automatic focus handling and restoration
//!
//! ## Typeahead Buffer Behavior
//!
//! The Select component implements an typeahead search buffer that lets you type while the dropdown is open to focus a matching
//! option. The buffer will be cleared after some amount of time has passed with no new input. The timeout is 1 second by default,
//! but can be configured by setting the [`SelectProps::typeahead_timeout`].
//!
//! ## Example
//!
//! ```rust
//! use dioxus::prelude::*;
//! use adico_primitives::select::{
//!     Select, SelectGroup, SelectGroupLabel, SelectItemIndicator,
//!     SelectList, SelectOption, SelectTrigger, SelectValue,
//! };
//!
//! #[component]
//! fn Demo() -> Element {
//!     rsx! {
//!         Select::<String> {
//!             SelectTrigger{
//!                 aria_label: "Select Trigger",
//!                 width: "12rem",
//!                 SelectValue { placeholder: "Select a fruit..." }
//!             }
//!             SelectList {
//!                 aria_label: "Select Demo",
//!                 SelectGroup {
//!                     SelectGroupLabel { "Fruits" }
//!                     SelectOption::<String> {
//!                         index: 0usize,
//!                         value: "apple",
//!                         "Apple"
//!                         SelectItemIndicator { "✔️" }
//!                     }
//!                     SelectOption::<String> {
//!                         index: 1usize,
//!                         value: "banana",
//!                         "Banana"
//!                         SelectItemIndicator { "✔️" }
//!                     }
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```

// Internal modules
mod components;
mod context;

// Re-export all public components and types
pub use components::{
    Select, SelectGroup, SelectGroupLabel, SelectGroupLabelProps, SelectGroupProps,
    SelectItemIndicator, SelectItemIndicatorProps, SelectList, SelectListProps, SelectMulti,
    SelectMultiProps, SelectOption, SelectOptionProps, SelectProps, SelectTrigger,
    SelectTriggerProps, SelectValue, SelectValueProps,
};

/// Test-only adapters used by the workspace's shared interaction helpers.
#[cfg(feature = "test-utils")]
#[doc(hidden)]
pub mod test_support {
    use crate::selection::{OptionState, RcPartialEqValue};
    use crate::typeahead::{AdaptiveKeyboard, best_match};

    /// Returns the selected option index for a typeahead sequence.
    pub fn typeahead_best_match(query: &str, options: &[&str]) -> Option<usize> {
        let options: Vec<_> = options
            .iter()
            .enumerate()
            .map(|(index, text_value)| OptionState {
                id: format!("test-option-{index}"),
                index,
                value: RcPartialEqValue::new(index),
                text_value: (*text_value).to_string(),
            })
            .collect();
        best_match(&AdaptiveKeyboard::new(), query, &options, |_| true)
    }
}
