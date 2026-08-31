// SPDX-License-Identifier: MIT OR Apache-2.0

//! Re-exports [`crate::menu`]'s components under `DropdownMenu*` names.
//!
//! Base UI has no separate dropdown-menu component: `Menu` *is* the dropdown
//! menu -- a menu opened by clicking a trigger button, per the WAI-ARIA APG
//! [Menu Button pattern](https://www.w3.org/WAI/ARIA/apg/patterns/menu-button/).
//! This module previously carried its own, independent implementation that
//! differed from [`crate::menu`] in exactly one substantive way: it rendered
//! `role="listbox"`/`role="option"`/`aria-haspopup="listbox"` instead of the
//! APG-correct `role="menu"`/`role="menuitem"`/`aria-haspopup="menu"`.
//! Re-authoring it as a re-export (task 2.3) both eliminates the duplicated
//! open/focus/dismiss logic and fixes that role mismatch.
//!
//! See [`crate::menu`] for the full example and API surface, including
//! [`crate::menu::MenuCheckboxItem`], [`crate::menu::MenuRadioGroup`]/
//! [`crate::menu::MenuRadioItem`], [`crate::menu::MenuGroup`]/
//! [`crate::menu::MenuGroupLabel`], [`crate::menu::MenuSeparator`], and
//! nested [`crate::menu::MenuSubmenuRoot`]/[`crate::menu::MenuSubmenuTrigger`]
//! -- all directly usable here under their own names, since they compose
//! against the same menu scope regardless of which alias opened it.
//!
//! ## Styling
//!
//! Defines the same `data-state` (`open`/`closed`) and `data-disabled`
//! attributes as [`crate::menu`].

pub use crate::menu::{
    Menu as DropdownMenu, MenuContent as DropdownMenuContent,
    MenuContentProps as DropdownMenuContentProps, MenuItem as DropdownMenuItem,
    MenuItemProps as DropdownMenuItemProps, MenuProps as DropdownMenuProps,
    MenuTrigger as DropdownMenuTrigger, MenuTriggerProps as DropdownMenuTriggerProps,
};
