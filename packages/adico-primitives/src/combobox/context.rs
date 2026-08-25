// SPDX-License-Identifier: MIT OR Apache-2.0
// Forked from DioxusLabs/dioxus-components at bf007c15d0cf4d04d3181cc46cf12325aa773955.
// Upstream path: primitives/src/combobox/context.rs. See provenance/records/adico-primitives-wave4-collection.json.

//! Shared state for the combobox component.

use dioxus::prelude::*;

use crate::selectable::{OptionState, SelectableContext};

/// The default case-insensitive substring filter.
pub fn default_combobox_filter(query: &str, text: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty() || text.to_lowercase().contains(&query)
}

#[derive(Clone, Copy)]
pub(super) struct ComboboxContext {
    pub selectable: SelectableContext,
    pub query: Memo<String>,
    pub set_query: Callback<String>,
    pub filter: Callback<(String, String), bool>,
}

impl ComboboxContext {
    pub fn set_open(&mut self, open: bool) {
        if open {
            self.selectable.collection.clear_focus();
        }
        self.selectable.set_open(open);
    }

    // Rust 2024's return-position `impl Trait` captures the elided `&self`
    // lifetime by default, which would tie the returned closure to `self`
    // and conflict with the `&mut self.selectable` calls below even though
    // the closure only holds `Copy` data extracted before it's built.
    // `use<>` opts back out, matching upstream's (pre-edition-2024) behavior.
    fn predicate_for(&self, query: String) -> impl Fn(&OptionState) -> bool + use<> {
        let filter = self.filter;
        move |option| filter.call((query.clone(), option.text_value.clone()))
    }

    fn predicate(&self) -> impl Fn(&OptionState) -> bool + use<> {
        self.predicate_for(self.query.cloned())
    }

    pub fn is_visible(&self, tab_index: usize) -> bool {
        let predicate = self.predicate();
        self.selectable
            .options
            .read()
            .iter()
            .find(|option| option.index == tab_index)
            .is_some_and(predicate)
    }

    pub fn has_visible_options(&self) -> bool {
        self.selectable.options.read().iter().any(self.predicate())
    }

    pub fn open_with_empty_query_and_focus_first(&mut self) {
        let query = String::new();
        self.set_query.call(query.clone());
        let initial_focus = self
            .selectable
            .first_matching_enabled_index(self.predicate_for(query));
        self.selectable.initial_focus.set(initial_focus);
        self.set_open(true);
    }

    pub fn open_with_empty_query_and_focus_last(&mut self) {
        let query = String::new();
        self.set_query.call(query.clone());
        let initial_focus = self
            .selectable
            .last_matching_enabled_index(self.predicate_for(query));
        self.selectable.initial_focus.set(initial_focus);
        self.set_open(true);
    }

    pub fn focused_option_id(&self) -> Option<String> {
        self.selectable.focused_option_id()
    }

    pub fn focus_next_visible(&mut self) {
        self.selectable.focus_next_where(self.predicate());
    }

    pub fn focus_prev_visible(&mut self) {
        self.selectable.focus_prev_where(self.predicate());
    }

    pub fn focus_first_visible(&mut self) {
        self.selectable.focus_first_where(self.predicate());
    }

    pub fn focus_last_visible(&mut self) {
        self.selectable.focus_last_where(self.predicate());
    }

    pub fn select_focused(&mut self) {
        self.selectable.select_focused();
    }
}
