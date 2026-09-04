//! Black-box tests for `adico_primitives::menu`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/*.rs`.

use adico_primitives::menu::{
    Menu, MenuCheckboxItem, MenuContent, MenuItem, MenuRadioGroup, MenuRadioItem, MenuSubmenuRoot,
    MenuSubmenuTrigger, MenuTrigger,
};
use dioxus::prelude::*;

// See test_dropdown_menu.rs's identical comment: every test here that calls
// `render` is gated `cfg(not(any(feature = "web", feature = "native")))`
// (SSR-only), so under Cargo's workspace-wide feature unification (which
// enables `web` whenever another workspace member depends on
// `adico-primitives` with it) this function itself must carry the same gate
// or it's flagged dead code once every caller is compiled out.
#[cfg(not(any(feature = "web", feature = "native")))]
fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn UncheckedCheckboxItem() -> Element {
    rsx! {
        Menu { default_open: true,
            MenuTrigger { "Open" }
            MenuContent {
                MenuCheckboxItem { index: 0usize, "Wrap words" }
            }
        }
    }
}

#[component]
fn CheckedCheckboxItem() -> Element {
    rsx! {
        Menu { default_open: true,
            MenuTrigger { "Open" }
            MenuContent {
                MenuCheckboxItem { index: 0usize, default_checked: true, "Wrap words" }
            }
        }
    }
}

// `MenuContent` gates on `use_animated_open`, whose real (`web`/`native`) implementation only
// flips its content-mounted signal from inside a `use_effect` -- which a plain
// `rebuild_in_place()` schedules but does not itself drive to completion outside a running app.
// Matching `date_picker.rs`'s identical, already-established precedent for this exact class of
// test, these run only on the SSR-fallback path (no `web`/`native` feature), where
// `use_animated_open` returns `open` directly with no effect involved.
#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn checkbox_item_defaults_to_unchecked() {
    let html = render(UncheckedCheckboxItem);
    assert!(html.contains("data-state=\"unchecked\""));
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn checkbox_item_honors_default_checked() {
    let html = render(CheckedCheckboxItem);
    assert!(html.contains("data-state=\"checked\""));
}

#[component]
fn RadioGroupWithDefaultSelection() -> Element {
    rsx! {
        Menu { default_open: true,
            MenuTrigger { "Open" }
            MenuContent {
                MenuRadioGroup::<String> { default_value: "b".to_string(),
                    MenuRadioItem::<String> { value: "a", index: 0usize, "A" }
                    MenuRadioItem::<String> { value: "b", index: 1usize, "B" }
                }
            }
        }
    }
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn radio_group_marks_only_the_default_value_checked() {
    let html = render(RadioGroupWithDefaultSelection);
    assert!(html.contains(r#"data-state="unchecked" data-disabled=false tabindex="-1">A"#));
    assert!(html.contains(r#"data-state="checked" data-disabled=false tabindex="-1">B"#));
}

#[component]
fn ClosedSubmenu() -> Element {
    rsx! {
        Menu { default_open: true,
            MenuTrigger { "Open" }
            MenuContent {
                MenuSubmenuRoot { index: 0usize,
                    MenuSubmenuTrigger { "More" }
                    MenuContent {
                        MenuItem::<String> { value: "x".to_string(), index: 0usize, "X" }
                    }
                }
            }
        }
    }
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn submenu_defaults_to_closed_and_its_content_is_not_rendered() {
    let html = render(ClosedSubmenu);
    assert!(html.contains("data-state=\"closed\""));
    assert!(!html.contains("\">X<"));
}

// New coverage (task 2.3): the ARIA APG Menu Button pattern's role contract, which
// `dropdown_menu.rs` now inherits verbatim by delegating to this module (see
// `test_dropdown_menu.rs`) -- pinned here so a regression in the shared implementation is
// caught at the source.
#[component]
fn OpenMenuWithItem() -> Element {
    rsx! {
        Menu { default_open: true,
            MenuTrigger { "Open" }
            MenuContent {
                MenuItem::<String> { value: "edit".to_string(), index: 0usize, "Edit" }
            }
        }
    }
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn menu_content_and_items_use_aria_menu_roles_not_listbox() {
    let html = render(OpenMenuWithItem);
    assert!(html.contains(r#"role="menu""#), "{html}");
    assert!(html.contains(r#"role="menuitem""#), "{html}");
    assert!(html.contains(r#"aria-haspopup="menu""#), "{html}");
    assert!(!html.contains("role=\"listbox\""), "{html}");
    assert!(!html.contains("role=\"option\""), "{html}");
}

// task 7.6 (M6-closure batch, 2026-09-04): `MenuContent` now composes
// `positioner::Positioner` for anchored placement instead of a plain flow `div`.
// `aria-labelledby` referencing the trigger's own id is the one part of the old
// plain-`div` implementation `Positioner`'s `attributes` merge has to carry
// through unchanged -- pinned here so a regression in that merge (e.g. an
// attribute silently dropped) is caught, not just role="menu" (already covered
// above).
#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn menu_content_stays_labelled_by_its_trigger_after_the_positioner_move() {
    let html = render(OpenMenuWithItem);
    let trigger_id = html
        .split("id=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("trigger renders an id");
    assert!(
        html.contains(&format!(r#"aria-labelledby="{trigger_id}""#)),
        "content should be aria-labelledby its trigger's id ({trigger_id}): {html}"
    );
}

// task 7.6 (M6-closure batch): an item's `text_value` (typeahead registration)
// is purely additive -- rendering with or without it set must not change the
// item's own markup.
#[component]
fn OpenMenuWithTextValueItem() -> Element {
    rsx! {
        Menu { default_open: true,
            MenuTrigger { "Open" }
            MenuContent {
                MenuItem::<String> {
                    value: "banana".to_string(),
                    index: 0usize,
                    text_value: "Banana".to_string(),
                    "Banana",
                }
            }
        }
    }
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn menu_item_renders_the_same_whether_or_not_text_value_is_set() {
    let with_text_value = render(OpenMenuWithTextValueItem);
    let without_text_value = render(OpenMenuWithItem);
    // Only the visible label text should differ ("Banana" vs "Edit"); the
    // role/tabindex/data-disabled contract around it is unaffected by
    // whether `text_value` was set.
    assert!(
        with_text_value.contains(r#"role="menuitem""#),
        "{with_text_value}"
    );
    assert!(
        without_text_value.contains(r#"role="menuitem""#),
        "{without_text_value}"
    );
}

// task 7.6 (M6-closure batch): `MenuSubmenuTrigger`'s new hover-intent props
// (`open_delay_ms`/`close_delay_ms`) are additive defaults -- a submenu must
// still render its normal closed-by-default contract untouched.
#[component]
fn ClosedSubmenuWithCustomHoverDelays() -> Element {
    rsx! {
        Menu { default_open: true,
            MenuTrigger { "Open" }
            MenuContent {
                MenuSubmenuRoot { index: 0usize, open_delay_ms: 50u64, close_delay_ms: 50u64,
                    MenuSubmenuTrigger { "More" }
                    MenuContent {
                        MenuItem::<String> { value: "x".to_string(), index: 0usize, "X" }
                    }
                }
            }
        }
    }
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn submenu_with_custom_hover_delays_still_defaults_closed() {
    let html = render(ClosedSubmenuWithCustomHoverDelays);
    assert!(html.contains("data-state=\"closed\""), "{html}");
    assert!(!html.contains("\">X<"), "{html}");
}
