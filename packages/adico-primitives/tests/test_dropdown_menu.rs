//! Black-box tests for `adico_primitives::dropdown_menu`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`).
//!
//! `dropdown_menu` is now a re-export of `menu` (task 2.3), so its own behavioral coverage lives
//! in `test_menu.rs`. This file only pins the re-export wiring itself: that the `DropdownMenu*`
//! names resolve, compose, and render the ARIA APG Menu Button pattern's roles -- the one
//! substantive difference from this module's pre-2.3 implementation, which used
//! `role="listbox"`/`"option"` instead.

use adico_primitives::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn OpenDropdownMenuWithItem() -> Element {
    rsx! {
        DropdownMenu { default_open: true,
            DropdownMenuTrigger { "Open" }
            DropdownMenuContent {
                DropdownMenuItem::<String> { value: "edit".to_string(), index: 0usize, "Edit" }
            }
        }
    }
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn dropdown_menu_renders_the_aria_menu_button_pattern_roles() {
    let html = render(OpenDropdownMenuWithItem);
    assert!(html.contains(r#"aria-haspopup="menu""#), "{html}");
    assert!(html.contains(r#"role="menu""#), "{html}");
    assert!(html.contains(r#"role="menuitem""#), "{html}");
    assert!(!html.contains("role=\"listbox\""), "{html}");
    assert!(!html.contains("role=\"option\""), "{html}");
    assert!(!html.contains("aria-haspopup=\"listbox\""), "{html}");
}

#[component]
fn ClosedDropdownMenu() -> Element {
    rsx! {
        DropdownMenu { default_open: false,
            DropdownMenuTrigger { "Open" }
            DropdownMenuContent {
                DropdownMenuItem::<String> { value: "edit".to_string(), index: 0usize, "Edit" }
            }
        }
    }
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn dropdown_menu_content_is_not_rendered_while_closed() {
    let html = render(ClosedDropdownMenu);
    assert!(html.contains("data-state=\"closed\""), "{html}");
    assert!(!html.contains("\">Edit<"), "{html}");
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn dropdown_menu_content_s_aria_labelledby_matches_its_trigger_s_id() {
    let html = render(OpenDropdownMenuWithItem);
    let attr = "aria-labelledby=\"";
    let start = html.find(attr).expect("content has aria-labelledby") + attr.len();
    let end = html[start..].find('"').unwrap() + start;
    let labelledby_id = &html[start..end];

    assert!(html.contains(&format!(r#"id="{labelledby_id}""#)), "{html}");
}
