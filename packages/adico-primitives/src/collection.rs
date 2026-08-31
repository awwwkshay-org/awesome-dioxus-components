// SPDX-License-Identifier: MIT OR Apache-2.0

//! Ordered interactive collection state shared by roving-focus components.
//!
//! Authored against the WAI-ARIA Authoring Practices Guide's roving-tabindex and grid
//! keyboard-navigation patterns (single tab stop per collection, arrow/Home/End movement,
//! optional wraparound), not ported from another project's implementation. This file has no
//! counterpart entry in either compat axis (`statics/primitive_compatibility.json`) — it is
//! an internal-only primitive consumed by roughly a dozen components (accordion, tabs,
//! toolbar, radio-group, toggle-group, menubar, context-menu, dropdown-menu, tag-group,
//! date-picker, and others), so its spec is the ARIA pattern above plus what those consumers
//! actually need, not a feature-parity diff against Base UI or dioxus-primitives.

use std::rc::Rc;

use dioxus::prelude::*;

use crate::direction::Direction;
use crate::use_effect_cleanup;

/// Which axis arrow-key navigation moves along, for [`CollectionState::navigate_key`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    /// `ArrowUp`/`ArrowDown` move focus; `ArrowLeft`/`ArrowRight` are ignored.
    #[default]
    Vertical,
    /// `ArrowLeft`/`ArrowRight` move focus (flipped for [`Direction::Rtl`]);
    /// `ArrowUp`/`ArrowDown` are ignored.
    Horizontal,
}

/// One item's registration state within a [`CollectionState`]. Constructed for you by
/// [`use_item`] in normal usage; `pub` only so `packages/adico-primitives/tests/` can build
/// fixtures for [`CollectionState::register_item`]/[`unregister_item`] directly — not part of
/// the primitive's intended consumer-facing API.
#[derive(Debug, Clone, PartialEq)]
pub struct CollectionItemState {
    pub index: usize,
    pub key: Option<String>,
    pub disabled: bool,
    pub hidden: bool,
    pub selected: bool,
}

impl CollectionItemState {
    fn available(&self) -> bool {
        !self.disabled && !self.hidden
    }

    fn same_identity(&self, other: &Self) -> bool {
        match (&self.key, &other.key) {
            (Some(a), Some(b)) => a == b,
            (None, None) => self.index == other.index,
            _ => false,
        }
    }
}

/// A requested initial focus placement.
#[derive(Clone, Copy)]
pub enum CollectionPlacement {
    First,
    Last,
}

/// Group-level options for a collection.
#[derive(Clone, Copy, Default)]
pub struct CollectionOptions {
    /// When no item is selected and none is focused, make every available item
    /// a tab stop instead of only the first. Native HTML radio-group semantics.
    pub tabbable_when_empty: bool,
}

/// A cloneable handle for ordered item registration and roving-focus navigation.
#[derive(Clone, Copy)]
pub struct CollectionState {
    roving_loop: ReadSignal<bool>,
    tabbable_when_empty: bool,
    recent: Signal<Option<usize>>,
    focused: Signal<Option<usize>>,
    focus_key: Signal<Option<String>>,
    items: Signal<Vec<CollectionItemState>>,
}

impl CollectionState {
    pub fn new(roving_loop: ReadSignal<bool>, options: CollectionOptions) -> Self {
        Self {
            roving_loop,
            tabbable_when_empty: options.tabbable_when_empty,
            recent: Signal::new(None),
            focused: Signal::new(None),
            focus_key: Signal::new(None),
            items: Signal::new(Vec::new()),
        }
    }

    /// Whether arrow navigation wraps around the ends of the collection.
    pub fn loops(&self) -> bool {
        (self.roving_loop)()
    }

    /// The backing loop signal, for nested collections that inherit the parent's
    /// looping behavior.
    pub fn loop_signal(&self) -> ReadSignal<bool> {
        self.roving_loop
    }

    /// Register (or re-register, on identity match) one item. `pub` only for
    /// `packages/adico-primitives/tests/`; normal usage goes through [`use_item`].
    pub fn register_item(&mut self, item: CollectionItemState) {
        let index = item.index;
        let key = item.key.clone();
        let available = item.available();
        if self.items.peek().iter().any(|existing| existing == &item) {
            return;
        }
        let previous_index = self
            .items
            .peek()
            .iter()
            .find(|existing| existing.same_identity(&item))
            .map(|item| item.index);
        sync_item(&mut self.items.write(), item);
        if let Some(previous_index) = previous_index {
            self.move_focus_for_reindexed_key(previous_index, index, key.as_deref());
        } else {
            self.adopt_focus_key_for_index(index, key.clone());
        }
        self.clear_focus_if_unavailable(index, key.as_deref(), available);
    }

    /// Remove a previously registered item. `pub` only for
    /// `packages/adico-primitives/tests/`; normal usage goes through [`use_item`]'s cleanup.
    pub fn unregister_item(&mut self, item: &CollectionItemState) {
        let removing_focused = self.is_focused_item(item);
        let removed_focused = {
            let mut items = self.items.write();
            let removed = items.iter().any(|existing| existing.same_identity(item));
            items.retain(|existing| !existing.same_identity(item));
            removed && removing_focused
        };

        if removed_focused {
            self.clear_focus();
        }
    }

    pub fn set_focus(&mut self, index: Option<usize>) {
        let target = match index {
            Some(idx) if self.is_known_unavailable(idx) => None,
            other => other,
        };
        let key = target.and_then(|idx| self.key_for_index(idx));
        if let Some(idx) = target {
            self.recent.set(Some(idx));
        }
        // Only notify subscribers when the value actually changes. A redundant
        // clear (clearing focus when nothing is focused) must not wake effects
        // reading focus state, or it re-triggers the context-menu auto-close
        // bug guarded against on `focused` below.
        if *self.focus_key.peek() != key {
            self.focus_key.set(key);
        }
        if *self.focused.peek() != target {
            self.focused.set(target);
        }
    }

    pub fn set_focus_key(&mut self, key: Option<String>) {
        let index = key.as_deref().and_then(|key| self.index_for_key(key));
        if let Some(index) = index {
            self.recent.set(Some(index));
        }
        if *self.focused.peek() != index {
            self.focused.set(index);
        }
        if *self.focus_key.peek() != key {
            self.focus_key.set(key);
        }
    }

    pub fn clear_focus(&mut self) {
        self.set_focus(None);
    }

    pub fn focused_index(&self) -> Option<usize> {
        (self.focused)()
    }

    pub fn recent_focus(&self) -> Option<usize> {
        (self.recent)()
    }

    pub fn recent_focus_or_default(&self) -> usize {
        self.recent_focus()
            .filter(|&index| self.is_available(index))
            .or_else(|| self.selected_available_index())
            .or_else(|| self.first_available_index())
            .unwrap_or_default()
    }

    pub fn focused_key(&self) -> Option<String> {
        let focused = self.focused_index()?;
        let key = (self.focus_key)();
        let items = self.items.read();
        match key {
            Some(key) => items
                .iter()
                .find(|item| item.available() && item.key.as_deref() == Some(key.as_str()))
                .and_then(|item| item.key.clone()),
            None => items
                .iter()
                .find(|item| item.index == focused && item.available())
                .and_then(|item| item.key.clone()),
        }
    }

    pub fn any_focused(&self) -> bool {
        self.focused.read().is_some()
    }

    pub fn is_focused(&self, index: usize) -> bool {
        self.focused_index() == Some(index)
    }

    pub fn is_available(&self, index: usize) -> bool {
        self.items
            .read()
            .iter()
            .any(|item| item.index == index && item.available())
    }

    pub fn first_available_index(&self) -> Option<usize> {
        self.items
            .read()
            .iter()
            .find(|item| item.available())
            .map(|item| item.index)
    }

    pub fn last_available_index(&self) -> Option<usize> {
        self.items
            .read()
            .iter()
            .rev()
            .find(|item| item.available())
            .map(|item| item.index)
    }

    pub fn selected_available_index(&self) -> Option<usize> {
        self.items
            .read()
            .iter()
            .find(|item| item.selected && item.available())
            .map(|item| item.index)
    }

    pub fn roving_tabindex(&self, index: usize) -> &'static str {
        if !self.is_available(index) {
            return "-1";
        }
        if !self.loops() {
            return "0";
        }
        // The roving anchor: the most recently focused item, else the selected
        // item, else the first available one. Mirrors React Aria's
        // `focusedKey ?? firstSelectedKey ?? firstKey`.
        let anchor = self
            .recent_focus()
            .filter(|&index| self.is_available(index))
            .or_else(|| self.selected_available_index());
        match anchor {
            Some(anchor) => {
                if anchor == index {
                    "0"
                } else {
                    "-1"
                }
            }
            // No focus and no selection: either every item is a tab stop
            // (native radio-group semantics) or just the first one.
            None if self.tabbable_when_empty => "0",
            None if self.first_available_index() == Some(index) => "0",
            None => "-1",
        }
    }

    pub fn focus_first(&mut self) {
        self.set_focus(self.first_available_index());
    }

    pub fn focus_last(&mut self) {
        self.set_focus(self.last_available_index());
    }

    pub fn focus_next(&mut self) {
        let indices = self.available_indices();
        self.set_focus(next_index_after(
            &indices,
            self.recent_focus(),
            self.loops(),
        ));
    }

    pub fn focus_prev(&mut self) {
        let indices = self.available_indices();
        self.set_focus(prev_index_before(
            &indices,
            self.recent_focus(),
            self.loops(),
        ));
    }

    /// Move focus to the next available item matching `predicate`, starting from
    /// the currently focused item. The collection owns the ordering; callers only
    /// describe which items qualify.
    pub fn focus_next_matching(&mut self, predicate: impl Fn(usize) -> bool) {
        let indices = self.available_indices_matching(predicate);
        self.set_focus(next_index_after(
            &indices,
            self.focused_index(),
            self.loops(),
        ));
    }

    /// Move focus to the previous available item matching `predicate`, starting
    /// from the currently focused item.
    pub fn focus_prev_matching(&mut self, predicate: impl Fn(usize) -> bool) {
        let indices = self.available_indices_matching(predicate);
        self.set_focus(prev_index_before(
            &indices,
            self.focused_index(),
            self.loops(),
        ));
    }

    /// Move focus in response to an arrow/Home/End key press, honoring
    /// `orientation` and `direction` (in [`Orientation::Horizontal`], a
    /// right-to-left [`Direction`] flips which physical arrow key means
    /// "next"). Returns whether the key was handled, so callers can
    /// conditionally call `prevent_default()` — this replaces each
    /// component's own hand-rolled arrow-key match arms with one shared,
    /// direction-aware implementation.
    pub fn navigate_key(
        &mut self,
        key: Key,
        orientation: Orientation,
        direction: Direction,
    ) -> bool {
        match key {
            Key::ArrowUp if orientation == Orientation::Vertical => self.focus_prev(),
            Key::ArrowDown if orientation == Orientation::Vertical => self.focus_next(),
            Key::ArrowLeft if orientation == Orientation::Horizontal => {
                if direction.is_rtl() {
                    self.focus_next();
                } else {
                    self.focus_prev();
                }
            }
            Key::ArrowRight if orientation == Orientation::Horizontal => {
                if direction.is_rtl() {
                    self.focus_prev();
                } else {
                    self.focus_next();
                }
            }
            Key::Home => self.focus_first(),
            Key::End => self.focus_last(),
            _ => return false,
        }
        true
    }

    /// Move focus in a 2-D grid, where items are laid out `columns` wide in
    /// row-major index order (for example, a calendar month grid). Unlike
    /// [`Self::navigate_key`], a move onto an unavailable or out-of-bounds
    /// cell is dropped rather than skipping to the next available one —
    /// grids don't auto-skip past disabled cells the way a 1-D roving list
    /// does — and there is no wraparound regardless of [`Self::loops`].
    /// `ArrowLeft`/`ArrowRight` move by one column, flipped for
    /// [`Direction::Rtl`]; `ArrowUp`/`ArrowDown` move by one row. Returns
    /// whether the key was handled, so callers can conditionally call
    /// `prevent_default()`.
    pub fn navigate_grid_key(&mut self, key: Key, columns: usize, direction: Direction) -> bool {
        if columns == 0 {
            return false;
        }
        let columns = columns as isize;
        let rtl = direction.is_rtl();
        match key {
            Key::ArrowUp => self.move_by_grid_offset(-columns),
            Key::ArrowDown => self.move_by_grid_offset(columns),
            Key::ArrowLeft => self.move_by_grid_offset(if rtl { 1 } else { -1 }),
            Key::ArrowRight => self.move_by_grid_offset(if rtl { -1 } else { 1 }),
            Key::Home => self.focus_first(),
            Key::End => self.focus_last(),
            _ => return false,
        }
        true
    }

    /// Move focus to `focused_index() + offset` if that index exists and is
    /// available; otherwise leave focus unchanged. Absent any current focus,
    /// lands on the first available item instead of computing an offset.
    fn move_by_grid_offset(&mut self, offset: isize) {
        let Some(current) = self.focused_index() else {
            self.try_focus_placement(CollectionPlacement::First);
            return;
        };
        let Some(target) = current.checked_add_signed(offset) else {
            return;
        };
        if self.is_available(target) {
            self.set_focus(Some(target));
        }
    }

    pub fn try_focus_placement(&mut self, placement: CollectionPlacement) -> bool {
        let index = match placement {
            CollectionPlacement::First => self.first_available_index(),
            CollectionPlacement::Last => self.last_available_index(),
        };
        if let Some(index) = index {
            self.set_focus(Some(index));
            true
        } else {
            false
        }
    }

    fn control_mount_focus(&self, index: usize, controlled_ref: Signal<Option<Rc<MountedData>>>) {
        let controlled_ref = controlled_ref();
        if self.is_focused(index) && self.is_available(index) {
            if let Some(md) = controlled_ref {
                spawn(async move {
                    let _ = md.set_focus(true).await;
                });
            }
        }
    }

    fn available_indices(&self) -> Vec<usize> {
        self.available_indices_matching(|_| true)
    }

    fn available_indices_matching(&self, predicate: impl Fn(usize) -> bool) -> Vec<usize> {
        let mut indices: Vec<_> = self
            .items
            .read()
            .iter()
            .filter(|item| item.available() && predicate(item.index))
            .map(|item| item.index)
            .collect();
        indices.sort_unstable();
        indices.dedup();
        indices
    }

    fn is_known_unavailable(&self, index: usize) -> bool {
        let items = self.items.peek();
        let mut known = false;
        for item in items.iter().filter(|item| item.index == index) {
            known = true;
            if item.available() {
                return false;
            }
        }
        known
    }

    fn key_for_index(&self, index: usize) -> Option<String> {
        let items = self.items.peek();
        items
            .iter()
            .find(|item| item.index == index && item.available())
            .or_else(|| items.iter().find(|item| item.index == index))
            .and_then(|item| item.key.clone())
    }

    fn index_for_key(&self, key: &str) -> Option<usize> {
        self.items
            .peek()
            .iter()
            .find(|item| item.key.as_deref() == Some(key) && item.available())
            .map(|item| item.index)
    }

    fn clear_focus_if_unavailable(&mut self, index: usize, key: Option<&str>, available: bool) {
        if available {
            return;
        }
        let is_focused = match key {
            Some(key) => self.focus_key.peek().as_deref() == Some(key),
            None => self.focus_key.peek().is_none() && *self.focused.peek() == Some(index),
        };
        if is_focused {
            self.clear_focus();
        }
    }

    fn move_focus_for_reindexed_key(
        &mut self,
        previous_index: usize,
        index: usize,
        key: Option<&str>,
    ) {
        if previous_index == index {
            return;
        }

        let moving_focused_item = match key {
            Some(key) => self.focus_key.peek().as_deref() == Some(key),
            None => self.focus_key.peek().is_none() && *self.focused.peek() == Some(previous_index),
        };

        if moving_focused_item {
            self.focused.set(Some(index));
            self.recent.set(Some(index));
        }
    }

    fn adopt_focus_key_for_index(&mut self, index: usize, key: Option<String>) {
        if key.is_none() {
            return;
        }
        if *self.focused.peek() == Some(index) && self.focus_key.peek().is_none() {
            self.focus_key.set(key);
        }
    }

    fn is_focused_item(&self, item: &CollectionItemState) -> bool {
        match item.key.as_deref() {
            Some(key) => self.focus_key.peek().as_deref() == Some(key),
            None => self.focus_key.peek().is_none() && *self.focused.peek() == Some(item.index),
        }
    }
}

pub fn use_collection_provider(roving_loop: ReadSignal<bool>) -> CollectionState {
    use_collection_provider_with(roving_loop, CollectionOptions::default())
}

pub fn use_collection_provider_with(
    roving_loop: ReadSignal<bool>,
    options: CollectionOptions,
) -> CollectionState {
    use_context_provider(|| CollectionState::new(roving_loop, options))
}

/// Everything an item needs from its collection, returned by the single
/// per-item entry point [`collection_item`]. Mirrors React Aria's
/// `useSelectableItem`, which returns one `itemProps` bundle (tabindex, focus
/// handling, focused state) instead of forcing each component to hand-roll it.
#[derive(Clone, Copy)]
pub struct CollectionItem {
    /// Roving `tabindex` for the item's focusable element (`"0"` or `"-1"`).
    pub tabindex: Memo<&'static str>,
    focused: Memo<bool>,
    controlled_ref: Signal<Option<Rc<MountedData>>>,
}

impl CollectionItem {
    /// Whether this item is the currently focused one.
    pub fn focused(&self) -> bool {
        (self.focused)()
    }

    /// A mounted handler that lets the collection drive DOM focus for this item.
    /// Attach it to the focusable element.
    pub fn onmounted(self) -> impl FnMut(MountedEvent) {
        let mut controlled_ref = self.controlled_ref;
        move |event: MountedEvent| controlled_ref.set(Some(event.data()))
    }
}

/// Begin registering one collection item. Chain only the optional inputs the
/// component actually has — `key`, `disabled`, `hidden`, `selected` — then pass
/// the builder to [`use_item`]. Unset inputs default to "none".
///
/// This is a plain constructor and runs no hooks; the hook is [`use_item`].
pub fn collection_item(
    collection: CollectionState,
    index: impl Readable<Target = usize> + Copy + 'static,
) -> CollectionItemBuilder {
    CollectionItemBuilder {
        collection,
        index: Rc::new(move || index.cloned()),
        key: Rc::new(|| None),
        disabled: Rc::new(|| false),
        hidden: Rc::new(|| false),
        selected: Rc::new(|| false),
    }
}

/// Builder for a single collection item. Construct it with [`collection_item`]
/// and pass it to [`use_item`] to register.
#[must_use = "pass the builder to use_item() to register the item"]
pub struct CollectionItemBuilder {
    collection: CollectionState,
    index: Rc<dyn Fn() -> usize>,
    key: Rc<dyn Fn() -> Option<String>>,
    disabled: Rc<dyn Fn() -> bool>,
    hidden: Rc<dyn Fn() -> bool>,
    selected: Rc<dyn Fn() -> bool>,
}

impl CollectionItemBuilder {
    /// Whether the item is currently disabled (skipped by roving focus).
    pub fn disabled(mut self, disabled: impl Fn() -> bool + 'static) -> Self {
        self.disabled = Rc::new(disabled);
        self
    }

    /// The item key, used for focus identity and `focused_key()` lookups.
    pub fn key(mut self, key: impl Fn() -> Option<String> + 'static) -> Self {
        self.key = Rc::new(key);
        self
    }

    /// Whether the item is currently hidden (e.g. filtered out or removed).
    pub fn hidden(mut self, hidden: impl Fn() -> bool + 'static) -> Self {
        self.hidden = Rc::new(hidden);
        self
    }

    /// Whether this item is the selected one. When nothing is focused yet, the
    /// selected item becomes the roving tab stop — mirrors React Aria seeding
    /// `focusedKey` from `firstSelectedKey`.
    pub fn selected(mut self, selected: impl Fn() -> bool + 'static) -> Self {
        self.selected = Rc::new(selected);
        self
    }
}

/// Register an item builder and return its handle: a roving `tabindex`,
/// `focused()` state, and an `onmounted()` focus handler. This is the hook — it
/// calls `use_effect`/`use_signal`/`use_memo`, so call it unconditionally, once
/// per render.
pub fn use_item(builder: CollectionItemBuilder) -> CollectionItem {
    let CollectionItemBuilder {
        mut collection,
        index,
        key,
        disabled,
        hidden,
        selected,
    } = builder;

    let mut previous_item: Signal<Option<CollectionItemState>> = use_signal(|| None);

    use_effect({
        let index = index.clone();
        let disabled = disabled.clone();
        let hidden = hidden.clone();
        move || {
            let item = CollectionItemState {
                index: index(),
                key: key(),
                disabled: disabled(),
                hidden: hidden(),
                selected: selected(),
            };
            let stale_item = previous_item.peek().clone();
            if let Some(stale_item) = stale_item {
                if !stale_item.same_identity(&item) {
                    collection.unregister_item(&stale_item);
                }
            }
            collection.register_item(item.clone());
            previous_item.set(Some(item));
        }
    });

    use_effect_cleanup(move || {
        if let Some(item) = previous_item.peek().as_ref() {
            collection.unregister_item(item);
        }
    });

    // Keep DOM focus in sync when this item becomes focused. The handler that
    // populates `controlled_ref` may be attached in a different component.
    let controlled_ref: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    use_effect({
        let index = index.clone();
        move || {
            if disabled() || hidden() {
                return;
            }
            collection.control_mount_focus(index(), controlled_ref);
        }
    });

    let tabindex = use_memo({
        let index = index.clone();
        move || collection.roving_tabindex(index())
    });
    let focused = use_memo(move || collection.is_focused(index()));

    CollectionItem {
        tabindex,
        focused,
        controlled_ref,
    }
}

pub fn use_deferred_collection_focus(
    mut collection: CollectionState,
    mut placement: Signal<Option<CollectionPlacement>>,
    active: impl Fn() -> bool + Copy + 'static,
) {
    use_effect(move || {
        if !active() {
            placement.set(None);
            return;
        }
        let Some(placement_value) = placement() else {
            return;
        };
        if collection.try_focus_placement(placement_value) {
            placement.set(None);
        }
    });
}

fn sync_item(items: &mut Vec<CollectionItemState>, item: CollectionItemState) {
    if let Some(position) = items
        .iter()
        .position(|existing| existing.same_identity(&item))
    {
        items.remove(position);
    }
    let insert_at = items.partition_point(|existing| existing.index <= item.index);
    items.insert(insert_at, item);
}

fn next_index_after(indices: &[usize], current: Option<usize>, roving_loop: bool) -> Option<usize> {
    match current {
        Some(current) => {
            let next_position = indices.partition_point(|&index| index <= current);
            indices
                .get(next_position)
                .copied()
                .or_else(|| roving_loop.then(|| indices.first().copied()).flatten())
        }
        None => indices.first().copied(),
    }
}

fn prev_index_before(
    indices: &[usize],
    current: Option<usize>,
    roving_loop: bool,
) -> Option<usize> {
    match current {
        Some(current) => {
            let prev_position = indices.partition_point(|&index| index < current);
            prev_position
                .checked_sub(1)
                .and_then(|position| indices.get(position).copied())
                .or_else(|| roving_loop.then(|| indices.last().copied()).flatten())
        }
        None if roving_loop => indices.last().copied(),
        None => indices.first().copied(),
    }
}
