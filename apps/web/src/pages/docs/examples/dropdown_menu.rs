//! Examples for `dropdown-menu`.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::components::ui::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel,
    DropdownMenuSeparator, DropdownMenuShortcut, DropdownMenuTrigger,
};

pub const SRC: &str = include_str!("dropdown_menu.rs");

pub const METAS: &[DocExampleMeta] = &[DocExampleMeta {
    id: "composition",
    title: "Composition",
    description: "`index` on each item seeds roving-focus keyboard navigation, so arrow keys and type-ahead work. Open it and press ↓.",
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
        DropdownMenu {
            DropdownMenuTrigger { "Open menu" }
            DropdownMenuContent { class: "min-w-56",
                DropdownMenuLabel { "My account" }
                DropdownMenuSeparator {}
                DropdownMenuGroup {
                    DropdownMenuItem::<String> {
                        value: "profile".to_string(),
                        index: 0usize,
                        on_select: move |_| {},
                        "Profile"
                        DropdownMenuShortcut { "⇧⌘P" }
                    }
                    DropdownMenuItem::<String> {
                        value: "billing".to_string(),
                        index: 1usize,
                        on_select: move |_| {},
                        "Billing"
                    }
                    DropdownMenuItem::<String> {
                        value: "settings".to_string(),
                        index: 2usize,
                        on_select: move |_| {},
                        "Settings"
                        DropdownMenuShortcut { "⌘S" }
                    }
                }
            }
        }
        // doc-example:end
    }
}
