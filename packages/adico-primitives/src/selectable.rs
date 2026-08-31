// SPDX-License-Identifier: MIT OR Apache-2.0
// Forked from DioxusLabs/dioxus-components at bf007c15d0cf4d04d3181cc46cf12325aa773955.
// Upstream path: primitives/src/selectable.rs. See provenance/records/adico-primitives-dialog-select.json.

//! Shared state and behavior for select-like listbox components.

use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use crate::{
    Controlled,
    collection::{CollectionState, use_collection_provider},
    listbox::{ListboxOptionContext, use_listbox_option},
    selection, use_controlled,
};

pub use crate::selection::{OptionState, RcPartialEqValue};

/// Whether selecting an option should replace the current value or toggle it.
#[derive(Clone, Copy, PartialEq)]
pub enum SelectionMode {
    /// A single value is selected and the popup closes after selection.
    Single,
    /// Multiple values can be selected and the popup stays open after selection.
    Multiple,
}

impl SelectionMode {
    pub fn is_multiple(self) -> bool {
        matches!(self, Self::Multiple)
    }

    fn closes_on_select(self) -> bool {
        matches!(self, Self::Single)
    }
}

/// Shared context for components built around a selectable listbox.
#[derive(Clone, Copy)]
pub struct SelectableContext {
    pub open: Memo<bool>,
    pub set_open: Callback<bool>,
    pub values: Memo<Vec<RcPartialEqValue>>,
    pub set_value: Callback<RcPartialEqValue>,
    pub selection_mode: SelectionMode,
    pub options: Signal<Vec<OptionState>>,
    pub list_id: Signal<Option<String>>,
    pub collection: CollectionState,
    pub initial_focus: Signal<Option<usize>>,
    pub disabled: ReadSignal<bool>,
}

#[derive(Clone, Copy)]
pub struct SelectableOption<T: Clone + PartialEq + 'static> {
    pub id: Memo<String>,
    pub disabled: Memo<bool>,
    pub selected: Memo<bool>,
    pub focused: Memo<bool>,
    pub down_pos: Signal<Option<(f64, f64)>>,
    pub index: ReadSignal<usize>,
    pub value: ReadSignal<T>,
}

pub struct SelectableOptionConfig<T: Clone + PartialEq + 'static> {
    pub id: ReadSignal<Option<String>>,
    pub index: ReadSignal<usize>,
    pub value: ReadSignal<T>,
    pub text_value: ReadSignal<Option<String>>,
    pub option_disabled: ReadSignal<bool>,
    pub component_name: &'static str,
}

impl SelectableContext {
    pub fn set_open(&mut self, open: bool) {
        self.set_open.call(open);
    }

    pub fn selected_text(&self) -> Option<String> {
        let values = self.values.read();
        let options = self.options.read();
        selection::selected_text(values.iter(), &options)
    }

    /// Returns selected option labels in selection order. Multi-select inputs
    /// use this to present all selected values without owning selection state.
    pub fn selected_texts(&self) -> Vec<String> {
        let values = self.values.read();
        let options = self.options.read();
        values
            .iter()
            .filter_map(|value| {
                options
                    .iter()
                    .find(|option| &option.value == value)
                    .map(|option| option.text_value.clone())
            })
            .collect()
    }

    pub fn is_selected(&self, value: &RcPartialEqValue) -> bool {
        self.values.read().iter().any(|selected| selected == value)
    }

    pub fn is_empty(&self) -> bool {
        self.values.read().is_empty()
    }

    pub fn focused_option_id(&self) -> Option<String> {
        self.collection.focused_key()
    }

    pub fn select_focused(&mut self) {
        if !self.open.cloned() {
            return;
        }
        let Some(index) = self.collection.focused_index() else {
            return;
        };
        if !self.collection.is_available(index) {
            return;
        }
        let value = self
            .options
            .read()
            .iter()
            .find(|option| option.index == index)
            .map(|option| option.value.clone());
        if let Some(value) = value {
            self.select_value(value);
        }
    }

    fn matching_enabled_indices(&self, predicate: impl Fn(&OptionState) -> bool) -> Vec<usize> {
        let mut indices: Vec<_> = self
            .options
            .read()
            .iter()
            .filter(|option| self.collection.is_available(option.index) && predicate(option))
            .map(|option| option.index)
            .collect();
        indices.sort_unstable();
        indices.dedup();
        indices
    }

    pub fn first_matching_enabled_index(
        &self,
        predicate: impl Fn(&OptionState) -> bool,
    ) -> Option<usize> {
        self.matching_enabled_indices(predicate).first().copied()
    }

    pub fn last_matching_enabled_index(
        &self,
        predicate: impl Fn(&OptionState) -> bool,
    ) -> Option<usize> {
        self.matching_enabled_indices(predicate).last().copied()
    }

    pub fn focus_next_where(&mut self, predicate: impl Fn(&OptionState) -> bool) {
        let options = self.options;
        self.collection.focus_next_matching(move |index| {
            options
                .read()
                .iter()
                .any(|option| option.index == index && predicate(option))
        });
    }

    pub fn focus_prev_where(&mut self, predicate: impl Fn(&OptionState) -> bool) {
        let options = self.options;
        self.collection.focus_prev_matching(move |index| {
            options
                .read()
                .iter()
                .any(|option| option.index == index && predicate(option))
        });
    }

    pub fn focus_first_where(&mut self, predicate: impl Fn(&OptionState) -> bool) {
        let index = self.first_matching_enabled_index(predicate);
        self.collection.set_focus(index);
    }

    pub fn focus_last_where(&mut self, predicate: impl Fn(&OptionState) -> bool) {
        let index = self.last_matching_enabled_index(predicate);
        self.collection.set_focus(index);
    }

    pub fn select_value(&mut self, value: RcPartialEqValue) {
        self.set_value.call(value);
        if self.selection_mode.closes_on_select() {
            self.set_open(false);
        }
    }
}

pub fn use_single_selectable_value<T: Clone + PartialEq + 'static>(
    controlled_value: Option<ReadSignal<Option<T>>>,
    default_value: Option<T>,
    on_change: Callback<Option<T>>,
    component_name: &'static str,
) -> (Memo<Vec<RcPartialEqValue>>, Callback<RcPartialEqValue>) {
    let mut internal_value: Signal<Option<T>> = use_signal(|| default_value.clone());
    let value = use_memo(move || match controlled_value {
        Some(value) => value.cloned(),
        None => internal_value.cloned(),
    });
    let values = use_memo(move || value().map(RcPartialEqValue::new).into_iter().collect());
    let set_value = use_callback(move |incoming: RcPartialEqValue| {
        let value = incoming
            .as_ref::<T>()
            .unwrap_or_else(|| panic!("{component_name} and option value types must match"))
            .clone();
        internal_value.set(Some(value.clone()));
        on_change.call(Some(value));
    });

    (values, set_value)
}

pub fn use_selectable_root(
    values: Memo<Vec<RcPartialEqValue>>,
    set_value: Callback<RcPartialEqValue>,
    selection_mode: SelectionMode,
    disabled: ReadSignal<bool>,
    roving_loop: ReadSignal<bool>,
    open: Controlled<bool>,
) -> SelectableContext {
    let (open, set_open) = use_controlled(open.value, open.default.cloned(), open.on_change);
    let options: Signal<Vec<OptionState>> = use_signal(Vec::default);
    let list_id = use_signal(|| None);
    let collection = use_collection_provider(roving_loop);
    let initial_focus = use_signal(|| None);

    SelectableContext {
        open,
        set_open,
        values,
        set_value,
        selection_mode,
        options,
        list_id,
        collection,
        initial_focus,
        disabled,
    }
}

pub fn use_selectable_option<T: Clone + PartialEq + 'static>(
    selectable: SelectableContext,
    option: SelectableOptionConfig<T>,
) -> SelectableOption<T> {
    let SelectableOptionConfig {
        id,
        index,
        value,
        text_value,
        option_disabled,
        component_name,
    } = option;
    let disabled = {
        let root_disabled = selectable.disabled;
        use_memo(move || root_disabled.cloned() || option_disabled.cloned())
    };
    let id = use_listbox_option(
        id,
        index,
        value,
        text_value,
        selectable.options,
        component_name,
    );
    let selected = use_memo(move || selectable.is_selected(&RcPartialEqValue::new(value.cloned())));
    let focused = use_memo(move || selectable.collection.is_focused(index()));
    let down_pos: Signal<Option<(f64, f64)>> = use_signal(|| None);

    use_context_provider(|| ListboxOptionContext {
        selected: selected.into(),
    });

    SelectableOption {
        id,
        disabled,
        selected,
        focused,
        down_pos,
        index,
        value,
    }
}

pub fn pointer_select_start(
    event: &Event<PointerData>,
    disabled: bool,
    mut down_pos: Signal<Option<(f64, f64)>>,
) {
    if disabled || event.trigger_button() != Some(MouseButton::Primary) {
        return;
    }
    event.prevent_default();
    let p = event.client_coordinates();
    down_pos.set(Some((p.x, p.y)));
}

pub fn pointer_select_commit(
    event: &Event<PointerData>,
    disabled: bool,
    mut down_pos: Signal<Option<(f64, f64)>>,
) -> bool {
    if disabled || event.trigger_button() != Some(MouseButton::Primary) {
        return false;
    }
    let Some((x0, y0)) = down_pos.take() else {
        return false;
    };
    if event.pointer_type() == "touch" {
        let p = event.client_coordinates();
        let dx = p.x - x0;
        let dy = p.y - y0;
        if dx * dx + dy * dy > 25.0 {
            return false;
        }
    }
    true
}

pub fn pointer_select_cancel(mut down_pos: Signal<Option<(f64, f64)>>) {
    down_pos.set(None);
}
