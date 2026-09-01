use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn TablePage() -> Element {
    rsx! {
        Demo { name: "Table",
            components::ui::Table { class: "max-w-md",
                components::ui::TableCaption { "A list of recent invoices." }
                components::ui::TableHeader {
                    components::ui::TableRow {
                        components::ui::TableHead { "Invoice" }
                        components::ui::TableHead { "Status" }
                        components::ui::TableHead { class: "text-right", "Amount" }
                    }
                }
                components::ui::TableBody {
                    components::ui::TableRow {
                        components::ui::TableCell { "INV001" }
                        components::ui::TableCell { "Paid" }
                        components::ui::TableCell { class: "text-right", "$250.00" }
                    }
                    components::ui::TableRow {
                        components::ui::TableCell { "INV002" }
                        components::ui::TableCell { "Pending" }
                        components::ui::TableCell { class: "text-right", "$150.00" }
                    }
                }
                components::ui::TableFooter {
                    components::ui::TableRow {
                        components::ui::TableCell { "Total" }
                        components::ui::TableCell {}
                        components::ui::TableCell { class: "text-right", "$400.00" }
                    }
                }
            }
        }
    }
}
