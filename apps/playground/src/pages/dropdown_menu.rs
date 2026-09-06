use adico_primitives::ContentAlign;
use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, SelectControl};
use crate::components::demo::Demo;
use crate::generated::controls::{
    DropdownMenuControls, DropdownMenuDemoState, DropdownMenuItemControls,
    DropdownMenuItemDemoState,
};

/// A realistic account menu: labeled groups with shortcut hints, checkbox
/// items, a radio group, a submenu, and a destructive action — exercising
/// every part the installed `dropdown-menu` item exports. The
/// `DropdownMenuItemControls` panel binds to the "Profile" item.
#[component]
pub fn DropdownMenuPage() -> Element {
    let disabled = use_signal(|| false);
    let menu_state = use_signal(DropdownMenuDemoState::default);
    let item_state = use_signal(DropdownMenuItemDemoState::default);
    let mut status_bar = use_signal(|| true);
    let mut activity_bar = use_signal(|| false);
    let mut panel_position = use_signal(|| Some("bottom".to_string()));
    let align = use_signal(|| ContentAlign::Start);
    rsx! {
        Demo {
            name: "DropdownMenu",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                DropdownMenuControls { state: menu_state }
                DropdownMenuItemControls { state: item_state }
                SelectControl {
                    label: "Align",
                    value: align,
                    options: &[
                        ("Start", ContentAlign::Start),
                        ("Center", ContentAlign::Center),
                        ("End", ContentAlign::End),
                    ],
                }
            },
            components::ui::DropdownMenu {
                disabled: disabled(),
                open: menu_state().open,
                default_open: menu_state().default_open,
                components::ui::DropdownMenuTrigger { "Open menu" }
                components::ui::DropdownMenuContent { align: align(), class: "min-w-56",
                    components::ui::DropdownMenuLabel { "My Account" }
                    components::ui::DropdownMenuSeparator {}
                    components::ui::DropdownMenuGroup {
                        components::ui::DropdownMenuItem::<String> {
                            value: "profile".to_string(),
                            index: 0usize,
                            inset: item_state().inset,
                            variant: item_state().variant,
                            on_select: move |_value| {},
                            "Profile"
                            components::ui::DropdownMenuShortcut { "⇧⌘P" }
                        }
                        components::ui::DropdownMenuItem::<String> {
                            value: "billing".to_string(),
                            index: 1usize,
                            on_select: move |_value| {},
                            "Billing"
                        }
                        components::ui::DropdownMenuItem::<String> {
                            value: "settings".to_string(),
                            index: 2usize,
                            on_select: move |_value| {},
                            "Settings"
                            components::ui::DropdownMenuShortcut { "⌘S" }
                        }
                    }
                    components::ui::DropdownMenuSeparator {}
                    components::ui::DropdownMenuCheckboxItem {
                        index: 3usize,
                        checked: ReadSignal::from(Signal::new(Some(status_bar()))),
                        on_checked_change: move |checked| status_bar.set(checked),
                        "Status Bar"
                    }
                    components::ui::DropdownMenuCheckboxItem {
                        index: 4usize,
                        checked: ReadSignal::from(Signal::new(Some(activity_bar()))),
                        on_checked_change: move |checked| activity_bar.set(checked),
                        "Activity Bar"
                    }
                    components::ui::DropdownMenuSeparator {}
                    components::ui::DropdownMenuLabel { "Panel Position" }
                    components::ui::DropdownMenuRadioGroup::<String> {
                        value: ReadSignal::from(Signal::new(panel_position())),
                        on_value_change: move |value| panel_position.set(Some(value)),
                        components::ui::DropdownMenuRadioItem::<String> {
                            value: "top".to_string(),
                            index: 5usize,
                            "Top"
                        }
                        components::ui::DropdownMenuRadioItem::<String> {
                            value: "bottom".to_string(),
                            index: 6usize,
                            "Bottom"
                        }
                        components::ui::DropdownMenuRadioItem::<String> {
                            value: "right".to_string(),
                            index: 7usize,
                            "Right"
                        }
                    }
                    components::ui::DropdownMenuSeparator {}
                    components::ui::DropdownMenuSub { index: 8usize,
                        components::ui::DropdownMenuSubTrigger { "Invite users" }
                        components::ui::DropdownMenuSubContent {
                            components::ui::DropdownMenuItem::<String> {
                                value: "email".to_string(),
                                index: 0usize,
                                on_select: move |_value| {},
                                "Email"
                            }
                            components::ui::DropdownMenuItem::<String> {
                                value: "message".to_string(),
                                index: 1usize,
                                on_select: move |_value| {},
                                "Message"
                            }
                        }
                    }
                    components::ui::DropdownMenuSeparator {}
                    components::ui::DropdownMenuItem::<String> {
                        value: "logout".to_string(),
                        index: 9usize,
                        variant: components::ui::DropdownMenuItemVariant::Destructive,
                        on_select: move |_value| {},
                        "Log out"
                        components::ui::DropdownMenuShortcut { "⇧⌘Q" }
                    }
                }
            }
        }
    }
}
