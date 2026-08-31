//! Black-box tests for `adico_primitives::menubar`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/menubar.rs`.
//!
//! `Menubar` has no `default_open`-style prop (unlike `Accordion`'s `default_value`): which
//! menu is open is derived entirely from roving focus (`Menubar`'s own `use_effect` syncing
//! `open_menu` to `focus.focused_index()`), itself driven by pointer/keyboard events a bare
//! `VirtualDom::rebuild_in_place()` never dispatches. So these tests cover the default
//! (all-closed) render and the ARIA APG Menubar pattern's static role contract and disabled
//! propagation -- the "a menu is open, its content renders, only one menu is open at a time"
//! behavior is a real, named gap here, the same class of interaction-driven-state limitation
//! this crate documents elsewhere (see `date_picker.rs`/`hover_card.rs`).

use adico_primitives::menubar::{
    Menubar, MenubarContent, MenubarItem, MenubarMenu, MenubarTrigger,
};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn TwoMenuMenubar() -> Element {
    rsx! {
        Menubar {
            MenubarMenu { index: 0usize,
                MenubarTrigger { "File" }
                MenubarContent {
                    MenubarItem { index: 0usize, value: "new".to_string(), "New" }
                }
            }
            MenubarMenu { index: 1usize, disabled: true,
                MenubarTrigger { "Edit" }
                MenubarContent {
                    MenubarItem { index: 0usize, value: "undo".to_string(), "Undo" }
                }
            }
        }
    }
}

#[test]
fn the_root_uses_the_aria_apg_menubar_role() {
    let html = render(TwoMenuMenubar);
    assert!(html.contains(r#"role="menubar""#), "{html}");
}

#[test]
fn every_menu_uses_the_menu_role_and_defaults_to_closed() {
    let html = render(TwoMenuMenubar);
    assert!(html.contains(r#"role="menu""#), "{html}");
    assert!(html.contains(r#"data-state="closed""#), "{html}");
    // No menu is open by default, so no MenubarContent (and thus no item) renders.
    assert!(!html.contains("\">New<"), "{html}");
    assert!(!html.contains("\">Undo<"), "{html}");
}

#[test]
fn every_trigger_uses_the_menuitem_role() {
    let html = render(TwoMenuMenubar);
    let file_pos = html.find("File").expect("File trigger renders");
    let head = &html[..file_pos];
    let attr = "role=\"";
    let start = head.rfind(attr).expect("trigger has a role attribute") + attr.len();
    assert_eq!(&head[start..start + "menuitem".len()], "menuitem", "{html}");
}

/// A disabled menubar item stays focusable (per the ARIA APG Menubar pattern) so keyboard
/// users can navigate onto it and discover it's unavailable -- it must not carry a native
/// `disabled` attribute, which would drop it out of the tab order entirely.
#[test]
fn the_disabled_menu_marks_its_trigger_aria_disabled_but_not_disabled() {
    let html = render(TwoMenuMenubar);
    let edit_pos = html.find("Edit").expect("Edit trigger renders");
    let button_start = html[..edit_pos]
        .rfind("<button")
        .expect("Edit is inside a button");
    let button_tag = &html[button_start..edit_pos];
    assert!(button_tag.contains("data-disabled=true"), "{html}");
    assert!(button_tag.contains("aria-disabled=true"), "{html}");
    assert!(!button_tag.contains(" disabled"), "{html}");

    let file_pos = html.find("File").expect("File trigger renders");
    let button_start = html[..file_pos]
        .rfind("<button")
        .expect("File is inside a button");
    let button_tag = &html[button_start..file_pos];
    assert!(button_tag.contains("data-disabled=false"), "{html}");
}
