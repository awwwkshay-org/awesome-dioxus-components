//! Examples for `table`.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::components::ui::table::{
    Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow,
};

pub const SRC: &str = include_str!("table.rs");

pub const METAS: &[DocExampleMeta] = &[DocExampleMeta {
    id: "composition",
    title: "Full composition",
    description: "`Table` renders real `<table>` semantics — caption, header, body, and footer are the native sections, so screen readers and `Ctrl+F` behave as expected.",
}];

pub fn render(id: &str) -> Element {
    match id {
        "composition" => rsx! { Composition {} },
        _ => rsx! {},
    }
}

#[component]
fn Composition() -> Element {
    rsx! {
        // doc-example:start composition
        Table { class: "w-full max-w-md",
            TableCaption { "A list of recent invoices." }
            TableHeader {
                TableRow {
                    TableHead { "Invoice" }
                    TableHead { "Status" }
                    TableHead { class: "text-right", "Amount" }
                }
            }
            TableBody {
                TableRow {
                    TableCell { "INV001" }
                    TableCell { "Paid" }
                    TableCell { class: "text-right", "$250.00" }
                }
                TableRow {
                    TableCell { "INV002" }
                    TableCell { "Pending" }
                    TableCell { class: "text-right", "$150.00" }
                }
            }
            TableFooter {
                TableRow {
                    TableCell { "Total" }
                    TableCell {}
                    TableCell { class: "text-right", "$400.00" }
                }
            }
        }
        // doc-example:end
    }
}
