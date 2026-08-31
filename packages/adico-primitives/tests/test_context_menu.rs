//! Black-box tests for `adico_primitives::context_menu`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/context_menu.rs`.
//!
//! `ContextMenu` (unlike `Menubar`) does support `default_open`, so the open-content path is
//! directly reachable: with no `oncontextmenu`/long-press ever firing, `ContextMenuCtx::position`
//! stays at its `(0, 0)` default, giving deterministic `left`/`top` inline styles to assert on.

use adico_primitives::context_menu::{
    ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger,
};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn ClosedContextMenu() -> Element {
    rsx! {
        ContextMenu {
            ContextMenuTrigger { "right click here" }
            ContextMenuContent {
                ContextMenuItem { value: "edit".to_string(), index: 0usize, "Edit" }
            }
        }
    }
}

#[test]
fn a_closed_context_menu_does_not_render_its_content() {
    let html = render(ClosedContextMenu);
    assert!(html.contains(r#"data-state="closed""#), "{html}");
    assert!(!html.contains("\">Edit<"), "{html}");
}

#[test]
fn the_trigger_uses_the_aria_button_and_haspopup_menu_contract() {
    let html = render(ClosedContextMenu);
    assert!(html.contains(r#"role="button""#), "{html}");
    assert!(html.contains(r#"aria-haspopup="menu""#), "{html}");
    assert!(html.contains(r#"aria-expanded=false"#), "{html}");
}

#[component]
fn OpenContextMenuAtOrigin() -> Element {
    rsx! {
        ContextMenu { default_open: true,
            ContextMenuTrigger { "right click here" }
            ContextMenuContent {
                ContextMenuItem { value: "edit".to_string(), index: 0usize, "Edit" }
            }
        }
    }
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn an_open_context_menu_renders_at_the_default_zero_position() {
    // `use_animated_open` only flips its content-mounted signal from inside a `use_effect` on
    // the `web`/`native` targets, which a bare `rebuild_in_place()` doesn't drive to
    // completion -- the same documented limitation as `menu.rs`/`date_picker.rs`. On the
    // SSR-fallback path exercised here, it returns `open` directly, so this is reachable.
    let html = render(OpenContextMenuAtOrigin);
    assert!(html.contains(r#"role="menu""#), "{html}");
    assert!(html.contains(r#"aria-orientation="vertical""#), "{html}");
    assert!(html.contains(r#"data-state="open""#), "{html}");
    assert!(html.contains("left:0px"), "{html}");
    assert!(html.contains("top:0px"), "{html}");
    assert!(html.contains(r#"role="menuitem""#), "{html}");
    assert!(html.contains(">Edit<"), "{html}");
}

#[component]
fn OpenContextMenuWithDisabledItem() -> Element {
    rsx! {
        ContextMenu { default_open: true,
            ContextMenuTrigger { "right click here" }
            ContextMenuContent {
                ContextMenuItem {
                    value: "edit".to_string(),
                    index: 0usize,
                    disabled: true,
                    "Edit"
                }
            }
        }
    }
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn a_disabled_item_is_marked_aria_and_data_disabled() {
    let html = render(OpenContextMenuWithDisabledItem);
    assert!(html.contains("aria-disabled=true"), "{html}");
    assert!(html.contains("data-disabled=true"), "{html}");
}
