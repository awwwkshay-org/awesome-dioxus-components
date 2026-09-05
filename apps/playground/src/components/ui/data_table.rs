//! Source-owned shadcn-style Data Table pattern for Dioxus.
//!
//! shadcn's own Data Table is a docs *recipe*, not a cataloged component: it
//! composes `Table` with `@tanstack/react-table`'s headless row-model
//! (sorting/filtering/pagination/row-selection state) plus Checkbox,
//! DropdownMenu, Button, Input, and Pagination. No tanstack-table equivalent
//! exists in this ecosystem (see task 9.1's audit), so this file hand-rolls
//! that state directly rather than depending on one -- consistent with how
//! `carousel.rs`/`resizable.rs` keep component-specific state source-owned.
//! Sort/filter/select/paginate state lives here, in owned source, so a
//! consumer can freely edit it (add a real comparator, wire a backend query,
//! change the page size control) the same way they would any other
//! shadcn-for-Dioxus registry item.
//!
//! Composes the sibling registry items this table needs directly (`Button`,
//! `Checkbox`, `DropdownMenu`, `Input`, `Pagination`, `Table`), matching
//! shadcn's own reference recipe, which does the same in React. This item's
//! `registryDependencies` guarantee those modules are installed alongside
//! it, so the cross-module paths below always resolve for a real consumer.

use std::collections::HashSet;

use dioxus::prelude::*;

use adico_primitives::icons::{ArrowUpDown, Ellipsis};

use crate::adico_lib::cn::cn;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::checkbox::{Checkbox, CheckboxState};
use crate::components::ui::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use crate::components::ui::input::Input;
use crate::components::ui::pagination::{
    Pagination, PaginationContent, PaginationItem, PaginationNext, PaginationPrevious,
};
use crate::components::ui::spinner::Spinner;
use crate::components::ui::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};

/// Sort direction applied to a [`DataTableColumn`] with a `sort_key`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// One column of a [`DataTable`]: its header label, how to render a row's
/// cell, and (if `Some`) how to derive a sortable string key from a row.
/// `sort_key` returns a `String` rather than requiring `T: Ord` so this
/// stays generic over arbitrary row types -- a consumer sorting numerically
/// can zero-pad, and a consumer sorting dates can use an ISO string.
#[derive(Clone)]
pub struct DataTableColumn<T: 'static> {
    pub id: &'static str,
    pub header: String,
    pub cell: Callback<T, Element>,
    #[doc(hidden)]
    pub sort_key: Option<Callback<T, String>>,
}

impl<T: 'static> DataTableColumn<T> {
    /// A column with no sort affordance.
    pub fn new(id: &'static str, header: impl Into<String>, cell: Callback<T, Element>) -> Self {
        Self {
            id,
            header: header.into(),
            cell,
            sort_key: None,
        }
    }

    /// Attaches a sort-key extractor, making this column's header clickable
    /// to cycle unsorted -> ascending -> descending -> unsorted.
    pub fn sortable(mut self, sort_key: Callback<T, String>) -> Self {
        self.sort_key = Some(sort_key);
        self
    }
}

impl<T: 'static> PartialEq for DataTableColumn<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: 'static> std::fmt::Debug for DataTableColumn<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataTableColumn")
            .field("id", &self.id)
            .field("header", &self.header)
            .field("sortable", &self.sort_key.is_some())
            .finish()
    }
}

/// Props for [`DataTable`].
#[derive(Props, Clone, PartialEq)]
pub struct DataTableProps<T: Clone + PartialEq + 'static> {
    /// Column definitions, in display order.
    pub columns: Vec<DataTableColumn<T>>,
    /// The full (unpaginated, unsorted, unfiltered) row set.
    pub rows: Vec<T>,
    /// A stable identifier for a row, used as its selection key.
    pub row_id: Callback<T, String>,
    /// Called whenever the selected-row-id set changes.
    #[props(default)]
    pub on_selected_change: Callback<HashSet<String>>,
    /// A fully-composed per-row actions cell (typically a `DropdownMenu`
    /// wrapping an `Ellipsis`-icon `Button` trigger). Omit to render no
    /// actions column at all.
    #[props(default)]
    pub row_actions: Option<Callback<T, Element>>,
    /// Derives the text a row is matched against for the built-in filter
    /// input. Omit to render no filter input.
    #[props(default)]
    pub filter_key: Option<Callback<T, String>>,
    /// Placeholder for the filter input, when `filter_key` is set.
    #[props(default)]
    pub filter_placeholder: Option<String>,
    /// Rows shown per page.
    #[props(default = 10)]
    pub page_size: usize,
    /// Shows a [`Spinner`] in place of the row data (rows/pagination
    /// controls are ignored while true). An adico extension — shadcn's
    /// own convention is composing `<Button disabled><Spinner /></Button>`
    /// by hand; there is no upstream precedent for a whole-table loading
    /// state at all.
    #[props(default)]
    pub loading: bool,
    /// Replaces the loading row's default "Loading..." text while
    /// `loading` is true.
    #[props(default)]
    pub loading_text: Option<String>,
    /// Extra classes appended to the outer wrapper.
    #[props(default)]
    pub class: Option<String>,
}

/// # DataTable
///
/// A generic, source-owned Data Table: sortable columns, row selection,
/// an optional text filter, and pagination, composed over `Table`,
/// `Checkbox`, `Input`, `Pagination`, and (via `row_actions`) `DropdownMenu`.
///
/// This is a *pattern*, not a black-box widget -- it takes `Vec<T>` and a
/// `Vec<DataTableColumn<T>>` you define per use, the same way shadcn's own
/// React recipe expects a `columns.tsx` file per table.
#[component]
pub fn DataTable<T: Clone + PartialEq + 'static>(props: DataTableProps<T>) -> Element {
    let mut sort: Signal<Option<(&'static str, SortDirection)>> = use_signal(|| None);
    let mut selected: Signal<HashSet<String>> = use_signal(HashSet::new);
    let mut filter_text = use_signal(String::new);
    let mut page_index = use_signal(|| 0usize);

    let columns = props.columns.clone();
    let row_id = props.row_id;
    let filter_key = props.filter_key;
    let page_size = props.page_size.max(1);

    let visible_rows = use_memo(move || {
        let mut rows = props.rows.clone();

        if let Some(filter_key) = filter_key {
            let needle = filter_text().to_lowercase();
            if !needle.is_empty() {
                rows.retain(|row| {
                    filter_key
                        .call(row.clone())
                        .to_lowercase()
                        .contains(&needle)
                });
            }
        }

        if let Some((sort_id, direction)) = sort() {
            if let Some(column) = columns.iter().find(|column| column.id == sort_id) {
                if let Some(sort_key) = column.sort_key {
                    rows.sort_by(|a, b| {
                        let ordering = sort_key.call(a.clone()).cmp(&sort_key.call(b.clone()));
                        match direction {
                            SortDirection::Ascending => ordering,
                            SortDirection::Descending => ordering.reverse(),
                        }
                    });
                }
            }
        }

        rows
    });

    let total_rows = use_memo(move || visible_rows().len());
    let total_pages = use_memo(move || total_rows().div_ceil(page_size).max(1));
    let page_rows = use_memo(move || {
        let start = page_index().min(total_pages().saturating_sub(1)) * page_size;
        visible_rows()
            .into_iter()
            .skip(start)
            .take(page_size)
            .collect::<Vec<_>>()
    });

    let page_row_ids = use_memo(move || {
        page_rows()
            .iter()
            .map(|row| row_id.call(row.clone()))
            .collect::<Vec<_>>()
    });
    let all_page_rows_selected = use_memo(move || {
        !page_row_ids().is_empty() && page_row_ids().iter().all(|id| selected().contains(id))
    });

    let mut toggle_sort = move |column_id: &'static str| {
        sort.set(match sort() {
            Some((id, SortDirection::Ascending)) if id == column_id => {
                Some((id, SortDirection::Descending))
            }
            Some((id, SortDirection::Descending)) if id == column_id => None,
            _ => Some((column_id, SortDirection::Ascending)),
        });
    };

    let mut toggle_row = move |id: String| {
        let mut next = selected();
        if !next.remove(&id) {
            next.insert(id);
        }
        selected.set(next.clone());
        props.on_selected_change.call(next);
    };

    let toggle_page = move |_| {
        let mut next = selected();
        if all_page_rows_selected() {
            for id in page_row_ids() {
                next.remove(&id);
            }
        } else {
            for id in page_row_ids() {
                next.insert(id);
            }
        }
        selected.set(next.clone());
        props.on_selected_change.call(next);
    };

    let class = cn(&[
        "w-full space-y-4",
        props.class.as_deref().unwrap_or_default(),
    ]);

    rsx! {
        div { class, aria_busy: props.loading,
            if filter_key.is_some() {
                div { class: "flex items-center",
                    Input {
                        placeholder: props.filter_placeholder.clone().unwrap_or_else(|| "Filter...".to_string()),
                        value: filter_text(),
                        class: "max-w-sm",
                        oninput: move |event: FormEvent| {
                            filter_text.set(event.value());
                            page_index.set(0);
                        },
                    }
                }
            }
            Table {
                TableHeader {
                    TableRow {
                        TableHead { class: "w-10",
                            Checkbox {
                                checked: ReadSignal::new(
                                    Signal::new(
                                        Some(if all_page_rows_selected() {
                                            CheckboxState::Checked
                                        } else {
                                            CheckboxState::Unchecked
                                        }),
                                    ),
                                ),
                                on_checked_change: toggle_page,
                                aria_label: Some("Select all rows on this page".to_string()),
                            }
                        }
                        for column in props.columns.iter() {
                            {
                                let header = column.header.clone();
                                let id = column.id;
                                let sortable = column.sort_key.is_some();
                                rsx! {
                                    TableHead {
                                        if sortable {
                                            Button {
                                                variant: ButtonVariant::Ghost,
                                                size: ButtonSize::Sm,
                                                class: "-ml-3 gap-1",
                                                onclick: move |_| toggle_sort(id),
                                                "{header}"
                                                ArrowUpDown { class: "size-3.5" }
                                            }
                                        } else {
                                            "{header}"
                                        }
                                    }
                                }
                            }
                        }
                        if props.row_actions.is_some() {
                            TableHead { class: "w-10", span { class: "sr-only", "Actions" } }
                        }
                    }
                }
                TableBody {
                    if props.loading {
                        TableRow {
                            TableCell {
                                class: "h-24 text-center text-muted-foreground",
                                div { class: "flex items-center justify-center gap-2",
                                    Spinner {}
                                    span {
                                        {props.loading_text.clone().unwrap_or_else(|| "Loading...".to_string())}
                                    }
                                }
                            }
                        }
                    } else if page_rows().is_empty() {
                        TableRow {
                            TableCell {
                                class: "h-24 text-center text-muted-foreground",
                                "No results."
                            }
                        }
                    } else {
                        for row in page_rows().iter() {
                            {
                                let row = row.clone();
                                let id = row_id.call(row.clone());
                                let is_selected = selected().contains(&id);
                                rsx! {
                                    TableRow {
                                        key: "{id}",
                                        "data-state": is_selected.then_some("selected"),
                                        TableCell {
                                            {
                                                let id = id.clone();
                                                rsx! {
                                                    Checkbox {
                                                        checked: ReadSignal::new(
                                                            Signal::new(
                                                                Some(if is_selected {
                                                                    CheckboxState::Checked
                                                                } else {
                                                                    CheckboxState::Unchecked
                                                                }),
                                                            ),
                                                        ),
                                                        on_checked_change: move |_| toggle_row(id.clone()),
                                                        aria_label: Some("Select row".to_string()),
                                                    }
                                                }
                                            }
                                        }
                                        for column in props.columns.iter() {
                                            TableCell { {column.cell.call(row.clone())} }
                                        }
                                        if let Some(row_actions) = props.row_actions {
                                            TableCell { {row_actions.call(row.clone())} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "flex items-center justify-end gap-4 py-2",
                div { class: "flex-1 text-sm text-muted-foreground",
                    "{selected().len()} of {total_rows()} row(s) selected."
                }
                div { class: "text-sm text-muted-foreground",
                    "Page {page_index() + 1} of {total_pages()}"
                }
                Pagination { class: "mx-0 w-auto",
                    PaginationContent {
                        PaginationItem {
                            PaginationPrevious {
                                compact: true,
                                onclick: move |_| page_index.set(page_index().saturating_sub(1)),
                            }
                        }
                        PaginationItem {
                            PaginationNext {
                                compact: true,
                                onclick: move |_| {
                                    page_index.set((page_index() + 1).min(total_pages().saturating_sub(1)));
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

/// One entry in a [`DataTableRowActions`] menu: a label and what happens
/// when it is chosen, given the row it was opened for.
#[derive(Clone)]
pub struct DataTableRowAction<T: 'static> {
    pub label: String,
    pub on_select: Callback<T>,
    pub destructive: bool,
}

impl<T: 'static> DataTableRowAction<T> {
    pub fn new(label: impl Into<String>, on_select: Callback<T>) -> Self {
        Self {
            label: label.into(),
            on_select,
            destructive: false,
        }
    }

    /// Marks this action as destructive (delete-style styling).
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }
}

impl<T: 'static> PartialEq for DataTableRowAction<T> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label && self.destructive == other.destructive
    }
}

impl<T: 'static> std::fmt::Debug for DataTableRowAction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataTableRowAction")
            .field("label", &self.label)
            .field("destructive", &self.destructive)
            .finish()
    }
}

/// Props for [`DataTableRowActions`].
#[derive(Props, Clone, PartialEq)]
pub struct DataTableRowActionsProps<T: Clone + PartialEq + 'static> {
    /// The row this menu was opened for.
    pub row: T,
    /// The menu's entries, in display order.
    pub actions: Vec<DataTableRowAction<T>>,
}

/// A per-row "..." actions menu: composes `DropdownMenu`/`Button`/`Ellipsis`
/// for use as [`DataTableProps::row_actions`] (e.g.
/// `row_actions: Some(Callback::new(|row| rsx! { DataTableRowActions { row, actions: vec![...] } }))`).
#[component]
pub fn DataTableRowActions<T: Clone + PartialEq + 'static>(
    props: DataTableRowActionsProps<T>,
) -> Element {
    rsx! {
        DropdownMenu {
            DropdownMenuTrigger {
                class: cn(&[
                    "size-8 justify-center border-none bg-transparent px-0 shadow-none hover:bg-accent hover:text-accent-foreground",
                ]),
                span { class: "sr-only", "Open menu" }
                Ellipsis { class: "size-4" }
            }
            DropdownMenuContent { class: "right-0 left-auto",
                for (index , action) in props.actions.iter().enumerate() {
                    {
                        let row = props.row.clone();
                        let handler = action.on_select;
                        let label = action.label.clone();
                        let item_class = if action.destructive {
                            "text-destructive focus:bg-destructive/10 focus:text-destructive"
                        } else {
                            ""
                        };
                        rsx! {
                            DropdownMenuItem::<usize> {
                                value: ReadSignal::new(Signal::new(index)),
                                index: ReadSignal::new(Signal::new(index)),
                                class: item_class,
                                on_select: move |_| handler.call(row.clone()),
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Callback::new` needs a live Dioxus runtime (it allocates a
    /// generational box internally), so these equality checks -- which
    /// exercise the hand-written `PartialEq` impls, not the callbacks
    /// themselves -- run inside a throwaway component/`VirtualDom`, matching
    /// `packages/adico-primitives/tests/test_collection.rs`'s own harness
    /// convention for the same constraint.
    fn with_runtime<T: 'static>(f: impl FnOnce() -> T + 'static) -> T {
        thread_local! {
            static RESULT: std::cell::RefCell<Option<Box<dyn std::any::Any>>> =
                const { std::cell::RefCell::new(None) };
            static RUN: std::cell::RefCell<Option<Box<dyn FnOnce()>>> =
                const { std::cell::RefCell::new(None) };
        }

        RUN.with(|cell| {
            *cell.borrow_mut() = Some(Box::new(move || {
                let value = f();
                RESULT.with(|result| *result.borrow_mut() = Some(Box::new(value)));
            }));
        });

        #[component]
        fn Root() -> Element {
            RUN.with(|cell| (cell.borrow_mut().take().unwrap())());
            rsx! {}
        }

        let mut dom = VirtualDom::new(Root);
        dom.rebuild_in_place();

        RESULT.with(|result| *result.borrow_mut().take().unwrap().downcast().unwrap())
    }

    #[test]
    fn column_equality_is_by_id_only() {
        let equal = with_runtime(|| {
            let a: DataTableColumn<u32> =
                DataTableColumn::new("name", "Name", Callback::new(|_| rsx! {}));
            let b: DataTableColumn<u32> =
                DataTableColumn::new("name", "Different label", Callback::new(|_| rsx! {}));
            a == b
        });
        assert!(equal);
    }

    #[test]
    fn row_action_equality_is_by_label_and_destructiveness() {
        let (equal, not_equal_when_destructive) = with_runtime(|| {
            let a: DataTableRowAction<u32> = DataTableRowAction::new("Edit", Callback::new(|_| {}));
            let b: DataTableRowAction<u32> = DataTableRowAction::new("Edit", Callback::new(|_| {}));
            (a == b, a != b.clone().destructive())
        });
        assert!(equal);
        assert!(not_equal_when_destructive);
    }
}
