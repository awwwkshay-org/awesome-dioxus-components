use dioxus::prelude::*;

use components::ui::{DataTable, DataTableColumn, DataTableRowAction, DataTableRowActions};

#[derive(Clone, PartialEq)]
struct Person {
    id: String,
    name: String,
    status: String,
}

fn people() -> Vec<Person> {
    vec![
        Person {
            id: "1".to_string(),
            name: "Ada Lovelace".to_string(),
            status: "active".to_string(),
        },
        Person {
            id: "2".to_string(),
            name: "Grace Hopper".to_string(),
            status: "active".to_string(),
        },
        Person {
            id: "3".to_string(),
            name: "Alan Turing".to_string(),
            status: "invited".to_string(),
        },
    ]
}

fn app() -> Element {
    let mut last_action = use_signal(String::new);

    let columns = vec![
        DataTableColumn::new(
            "name",
            "Name",
            Callback::new(|row: Person| rsx! { "{row.name}" }),
        )
        .sortable(Callback::new(|row: Person| row.name.clone())),
        DataTableColumn::new(
            "status",
            "Status",
            Callback::new(|row: Person| rsx! { "{row.status}" }),
        )
        .sortable(Callback::new(|row: Person| row.status.clone())),
    ];

    rsx! {
        DataTable {
            columns,
            rows: people(),
            row_id: Callback::new(|row: Person| row.id.clone()),
            filter_key: Some(Callback::new(|row: Person| row.name.clone())),
            filter_placeholder: "Filter by name...".to_string(),
            page_size: 2usize,
            row_actions: Some(
                Callback::new(move |row: Person| {
                    let edit_id = row.id.clone();
                    let delete_id = row.id.clone();
                    rsx! {
                        DataTableRowActions {
                            row,
                            actions: vec![
                                DataTableRowAction::new(
                                    "Edit",
                                    Callback::new(move |_: Person| {
                                        last_action.set(format!("edit:{edit_id}"))
                                    }),
                                ),
                                DataTableRowAction::new(
                                        "Delete",
                                        Callback::new(move |_: Person| {
                                            last_action.set(format!("delete:{delete_id}"))
                                        }),
                                    )
                                    .destructive(),
                            ],
                        }
                    }
                }),
            ),
        }
        p { id: "last-action", "{last_action()}" }
    }
}

fn main() {
    launch(app);
}

// adico:start
pub mod adico_lib;
pub mod components;
// adico:end
