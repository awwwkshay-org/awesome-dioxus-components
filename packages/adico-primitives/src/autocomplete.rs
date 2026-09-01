// Base UI's Autocomplete parts (`statics/catalogs/base-ui.json`'s
// `autocomplete` entry) are exactly Combobox's parts minus
// `chip`/`chip-remove`/`chips`/`item-indicator`/`label` -- Autocomplete has
// no multi-select chip display or explicit selected-item indicator. Every
// remaining part (`root`/`input`/`list`/`item`/`popup`/`positioner`/`empty`/
// `status`/`clear`/...) is either already something `combobox.rs`'s
// single-value `Combobox` implements, or small enough to add directly here.
// So this module is a thin re-export of `Combobox`'s single-value parts
// under Autocomplete names -- matching the `dropdown_menu.rs`
// re-exports-`menu` precedent (task 2.3) -- plus the two genuinely-missing
// small parts, rather than a from-scratch reimplementation of the same
// filterable-listbox-popup behavior `combobox.rs` already has, tested, and
// live-verified this session (task 7.8d).
//
// **Correction to design.md §8a / task 7.9's own text:** both say
// "Autocomplete SHALL compose `typeahead` for type-to-select." Checked
// `typeahead.rs`'s actual API before building on it: `best_match` returns a
// single `Option<usize>` via `min_by` over distance -- a jump-focus-to-one-
// item engine (what `select.rs` uses while its listbox is open but not
// text-filtering), not a multi-item relevance filter. Repurposing
// `normalized_distance` as a per-option filter predicate would mean
// inventing an untuned, untested distance threshold on a path where a wrong
// threshold silently hides matching results -- new unvalidated matching
// behavior, not composition. `typeahead.rs`'s own module doc already flags
// that neither reference axis tracks a "dedicated typeahead capability" for
// Autocomplete, i.e. this was already a live question, not a settled one.
// Filtering here composes `combobox::default_combobox_filter` (a plain
// case-insensitive substring match) instead, the same filter `Combobox`
// itself ships and this crate has actually tested.
//
// **A real, separate gap found while building this, not fixed here:**
// neither `combobox.rs` nor the `selectable::use_single_selectable_value`
// primitive it's built on expose any way to *clear* an already-selected
// value (only ever set it to `Some`) -- so [`AutocompleteClear`] can only
// clear the query text, not the selection, and says so in its own doc
// comment. Adding a generic clear-selection callback would mean extending
// `selectable.rs` itself (shared by `select.rs`/`combobox.rs` too), which is
// real, separate scope beyond one new primitive's own file.

//! Autocomplete: Base UI's simpler, non-chip sibling of [`crate::combobox`],
//! re-exporting [`crate::combobox`]'s single-value parts under Autocomplete
//! names and adding the two parts Base UI's Autocomplete has that
//! `combobox.rs` doesn't: [`AutocompleteStatus`] and [`AutocompleteClear`].

use dioxus::prelude::*;

pub use crate::combobox::{
    Combobox as AutocompleteRoot, ComboboxContext as AutocompleteContext,
    ComboboxEmpty as AutocompleteEmpty, ComboboxInput as AutocompleteInput,
    ComboboxInputProps as AutocompleteInputProps, ComboboxList as AutocompleteList,
    ComboboxListProps as AutocompleteListProps, ComboboxOption as AutocompleteItem,
    ComboboxOptionProps as AutocompleteItemProps, ComboboxProps as AutocompleteRootProps,
    default_combobox_filter,
};

/// The props for the [`AutocompleteStatus`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AutocompleteStatusProps {
    /// Additional attributes to apply to the status element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # AutocompleteStatus
///
/// The `AutocompleteStatus` component renders an `aria-live="polite"`
/// region announcing how many options currently match the query, so screen
/// reader users hear result-count updates as they type without needing to
/// navigate into the (visually presented) listbox.
///
/// This must be used inside an [`AutocompleteRoot`] component.
#[component]
pub fn AutocompleteStatus(props: AutocompleteStatusProps) -> Element {
    let ctx = use_context::<AutocompleteContext>();
    // A single predicate built once and applied in one filter pass, not
    // `is_visible` called per option (which would rebuild the predicate and
    // re-scan the whole options list for each one -- quadratic).
    let count = use_memo(move || {
        let predicate = ctx.predicate();
        ctx.selectable
            .options
            .read()
            .iter()
            .filter(|option| predicate(option))
            .count()
    });
    rsx! {
        div {
            role: "status",
            aria_live: "polite",
            ..props.attributes,
            {
                match count() {
                    0 => "No results".to_string(),
                    1 => "1 result".to_string(),
                    n => format!("{n} results"),
                }
            }
        }
    }
}

/// The props for the [`AutocompleteClear`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AutocompleteClearProps {
    /// Additional attributes to apply to the clear button.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the clear button.
    pub children: Element,
}

/// # AutocompleteClear
///
/// The `AutocompleteClear` component renders a button that resets the
/// query text back to empty. It does **not** clear an already-selected
/// value: neither `combobox.rs` nor the `selectable::use_single_selectable_value`
/// primitive it's built on expose a way to reset a selection to `None` (see
/// this module's own doc comment) -- that's real, separate scope.
///
/// This must be used inside an [`AutocompleteRoot`] component.
#[component]
pub fn AutocompleteClear(props: AutocompleteClearProps) -> Element {
    let ctx = use_context::<AutocompleteContext>();
    rsx! {
        button {
            r#type: "button",
            onclick: move |_| ctx.set_query.call(String::new()),
            ..props.attributes,
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuses_combobox_s_own_tested_substring_filter() {
        assert!(default_combobox_filter("ban", "Banana"));
        assert!(!default_combobox_filter("xyz", "Banana"));
    }
}
