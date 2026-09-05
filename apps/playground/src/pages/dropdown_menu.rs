use dioxus::prelude::*;

use crate::components;
use crate::components::controls::BoolControl;
use crate::components::demo::Demo;
use crate::generated::controls::{
    DropdownMenuControls, DropdownMenuDemoState, DropdownMenuItemControls,
    DropdownMenuItemDemoState,
};

#[component]
pub fn DropdownMenuPage() -> Element {
    let disabled = use_signal(|| false);
    let menu_state = use_signal(DropdownMenuDemoState::default);
    let item_state = use_signal(DropdownMenuItemDemoState::default);
    rsx! {
        Demo {
            name: "DropdownMenu",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                DropdownMenuControls { state: menu_state }
                DropdownMenuItemControls { state: item_state }
            },
            components::ui::DropdownMenu {
                disabled: disabled(),
                open: menu_state().open,
                default_open: menu_state().default_open,
                components::ui::DropdownMenuTrigger { "Open menu" }
                components::ui::DropdownMenuContent {
                    components::ui::DropdownMenuItem::<String> {
                        value: "edit".to_string(),
                        index: 0usize,
                        inset: item_state().inset,
                        variant: item_state().variant,
                        on_select: move |_value| {},
                        "Edit"
                    }
                }
            }
        }
    }
}
