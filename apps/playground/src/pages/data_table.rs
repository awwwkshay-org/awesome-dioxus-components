use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{DataTableControls, DataTableDemoState};

#[derive(Clone, PartialEq)]
struct Person {
    id: String,
    name: String,
    status: String,
}

#[component]
pub fn DataTablePage() -> Element {
    let state = use_signal(|| DataTableDemoState {
        page_size: 5,
        loading: false,
    });
    let people = vec![
        Person {
            id: "1".to_string(),
            name: "Ada Lovelace".to_string(),
            status: "Active".to_string(),
        },
        Person {
            id: "2".to_string(),
            name: "Grace Hopper".to_string(),
            status: "Active".to_string(),
        },
        Person {
            id: "3".to_string(),
            name: "Margaret Hamilton".to_string(),
            status: "Invited".to_string(),
        },
    ];
    rsx! {
        Demo {
            name: "DataTable",
            wide: true,
            controls: rsx! {
                DataTableControls { state }
            },
            components::ui::DataTable {
                page_size: state().page_size,
                loading: state().loading,
                columns: vec![
                    components::ui::DataTableColumn::new(
                            "name",
                            "Name",
                            Callback::new(|row: Person| rsx! { "{row.name}" }),
                        )
                        .sortable(Callback::new(|row: Person| row.name.clone())),
                    components::ui::DataTableColumn::new(
                        "status",
                        "Status",
                        Callback::new(|row: Person| rsx! { "{row.status}" }),
                    ),
                ],
                rows: people,
                row_id: Callback::new(|row: Person| row.id.clone()),
            }
        }
    }
}
