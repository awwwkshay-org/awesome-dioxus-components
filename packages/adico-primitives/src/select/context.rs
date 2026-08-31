// SPDX-License-Identifier: MIT OR Apache-2.0
// Forked from DioxusLabs/dioxus-components at bf007c15d0cf4d04d3181cc46cf12325aa773955.
// Upstream path: primitives/src/select/context.rs. See provenance/records/adico-primitives-dialog-select.json.

//! Context types and implementations for the select component.

use dioxus::prelude::*;

use crate::selectable::SelectableContext;
use crate::typeahead::Typeahead;

/// Main context for the select component containing all shared state
#[derive(Clone, Copy)]
pub(super) struct SelectContext {
    /// Shared selectable listbox state.
    pub selectable: SelectableContext,
    /// Buffered, auto-clearing typeahead search.
    pub typeahead: Typeahead,
}

impl SelectContext {
    pub fn set_open(&mut self, open: bool) {
        self.selectable.set_open(open);
    }

    pub fn multi(&self) -> bool {
        self.selectable.selection_mode.is_multiple()
    }

    /// Select the currently focused item
    pub fn select_current_item(&mut self) {
        self.selectable.select_focused();
    }

    /// Learn from a keyboard event mapping physical key to logical character
    pub fn learn_from_keyboard_event(&mut self, physical_code: &str, logical_char: char) {
        self.typeahead
            .learn_from_keyboard_event(physical_code, logical_char);
    }

    /// Add text to the typeahead buffer for searching and focus the best match
    pub fn add_to_typeahead_buffer(&mut self, text: &str) {
        let options = self.selectable.options.read();
        let collection = self.selectable.collection;

        if let Some(best_match_index) = self
            .typeahead
            .on_input(text, &options, move |index| collection.is_available(index))
        {
            self.selectable.collection.set_focus(Some(best_match_index));
        }
    }
}

/// Context for select group components
#[derive(Clone, Copy)]
pub(super) struct SelectGroupContext {
    /// ID of the element that labels this group
    pub labeled_by: Signal<Option<String>>,
}
