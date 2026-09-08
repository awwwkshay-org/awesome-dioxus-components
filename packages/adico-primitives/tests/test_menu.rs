//! Black-box tests for `adico_primitives::menu`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/*.rs`.

use adico_primitives::dialog::{DialogContent, DialogRoot};
use adico_primitives::menu::{
    Menu, MenuCheckboxItem, MenuContent, MenuItem, MenuRadioGroup, MenuRadioItem, MenuSubmenuRoot,
    MenuSubmenuTrigger, MenuTrigger,
};
use dioxus::prelude::*;
use dioxus_core::{Event, Mutation};
use dioxus_html::{
    Code, EventData, Key, Location, Modifiers, SerializedHtmlEventConverter,
    SerializedKeyboardData, set_event_converter,
};

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

// Regression tests for task 5.2 (`openspec/changes/deduplicate-primitives`):
// before this change, root `Menu`'s `handle_keydown` closed the menu on
// *any* Escape press (`Key::Escape => ctx.set_open.call(false)`, no
// dismissal-layer check at all). Now routed through the shared
// `use_escape_key`, a root menu's own handler must recognize when it is not
// the topmost layer and leave itself open. See `test_tooltip.rs`'s
// equivalent tests for why this needs a genuine shared-ancestor structure
// (not independent siblings) to exercise the layer stack at all -- the
// `LayerStack` is only shared through Dioxus's own ancestor-scoped context
// lookup, confirmed empirically while writing that file's tests (see
// `openspec/changes/deduplicate-primitives` design.md's D12).
//
// No `MenuContent` in either menu below: this only needs each `Menu` root's
// own `onkeydown`/`data-state`, both unconditional (not gated by
// `use_animated_open`), so unlike this file's other tests these don't need
// the `cfg(not(any(feature = "web", feature = "native")))` SSR-only gate.

fn dispatch_escape(dom: &mut VirtualDom, nth: usize) -> String {
    let edits = dom.rebuild_to_vec();
    let id = edits
        .edits
        .iter()
        .filter_map(|edit| match edit {
            Mutation::NewEventListener { name, id } if name == "keydown" => Some(*id),
            _ => None,
        })
        .nth(nth)
        .unwrap_or_else(|| panic!("no keydown listener at index {nth}"));
    set_event_converter(Box::new(SerializedHtmlEventConverter));
    let event = Event::new(
        EventData::Keyboard(SerializedKeyboardData::new(
            Key::Escape,
            Code::Escape,
            Location::Standard,
            false,
            Modifiers::empty(),
            false,
        ))
        .into_any(),
        true,
    );
    dom.runtime().handle_event("keydown", event, id);
    dom.render_immediate_to_vec();
    dioxus_ssr::render(dom)
}

/// The value of the nearest `data-state="..."` attribute preceding `marker`
/// (a menu trigger's own text, unique in this fixture).
fn nearest_preceding_data_state<'a>(html: &'a str, marker: &str) -> &'a str {
    let marker_pos = html.find(marker).expect("marker text renders");
    let prefix = &html[..marker_pos];
    let attr_start = prefix
        .rfind(r#"data-state=""#)
        .expect("a data-state before marker")
        + r#"data-state=""#.len();
    let value_end = attr_start + prefix[attr_start..].find('"').unwrap();
    &prefix[attr_start..value_end]
}

#[component]
fn TwoMenusInsideAnOpenDialog() -> Element {
    // `DialogRoot` registers onto the shared layer stack first (it's the
    // genuine ancestor providing it); `MenuA` and `MenuB`, true descendants,
    // join the same stack in render order, with `MenuB` registering last and
    // therefore topmost -- listener index 0 is `DialogRoot`'s own
    // `onkeydown`, index 1 is menu A's root, index 2 is menu B's root.
    rsx! {
        DialogRoot { default_open: true,
            DialogContent {
                Menu { default_open: true, MenuTrigger { "Menu A" } }
                Menu { default_open: true, MenuTrigger { "Menu B" } }
            }
        }
    }
}

/// Positive guard: an Escape at the *topmost* root menu (B, mounted last)
/// still closes it, i.e. this consolidation didn't break ordinary dismissal.
///
/// **Not exercised here, and why:** the mirror case -- an Escape at a
/// *non-topmost* root menu should leave it open -- was attempted with this
/// same fixture and found confounded by `Menu`'s own separate, pre-existing
/// `use_effect` that closes a menu whenever its roving-focus collection has
/// nothing focused (`menu.rs`'s `Menu` body: `if *ctx.open.peek() != focused
/// { ctx.set_open.call(focused) }`). Dispatching a raw synthetic event at a
/// specific menu's own listener id and then calling
/// `dom.render_immediate_to_vec()` (needed to observe the change via SSR)
/// flushes *that* menu's pending effects too -- and since this synthetic
/// harness never establishes real DOM/roving focus, the effect closes menu A
/// regardless of the Escape/topmost outcome, making the HTML-based assertion
/// unable to distinguish "closed by the fix working" from "closed by this
/// unrelated effect." Confirmed by direct instrumentation of
/// `use_escape_key`'s `layer.is_topmost()` during development of this test
/// (not kept, to avoid debug output in shipped source): it correctly
/// computed `false` for the non-topmost menu and did not call `on_escape`
/// through the fixed code path -- the mechanism itself is verified in
/// isolation, without this confound, by `test_lib.rs`'s
/// `use_escape_key`-level tests instead.
#[test]
fn an_escape_at_the_topmost_root_menu_still_closes_it() {
    let mut dom = VirtualDom::new(TwoMenusInsideAnOpenDialog);
    let html = dispatch_escape(&mut dom, 2);
    assert_eq!(
        nearest_preceding_data_state(&html, "Menu B"),
        "closed",
        "root menu B (topmost) should still close on its own Escape: {html}"
    );
}
