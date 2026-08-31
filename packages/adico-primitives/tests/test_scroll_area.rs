//! Black-box tests for `adico_primitives::scroll_area`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/scroll_area.rs`.

use adico_primitives::scroll_area::{ScrollArea, ScrollDirection, ScrollType};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn AutoBoth() -> Element {
    rsx! { ScrollArea { "content" } }
}

#[test]
fn default_scroll_area_auto_overflows_both_axes_with_no_scrollbar_width_override() {
    let html = render(AutoBoth);
    assert!(html.contains("overflow-x:auto"), "{html}");
    assert!(html.contains("overflow-y:auto"), "{html}");
    assert!(!html.contains("scrollbar-width"), "{html}");
    assert!(html.contains(r#"data-scroll-direction="both""#), "{html}");
}

#[component]
fn AlwaysVertical() -> Element {
    rsx! {
        ScrollArea {
            direction: ScrollDirection::Vertical,
            scroll_type: ScrollType::Always,
            "content"
        }
    }
}

#[test]
fn always_vertical_scroll_area_hides_the_cross_axis_and_forces_the_scroll_axis() {
    let html = render(AlwaysVertical);
    assert!(html.contains("overflow-x:hidden"), "{html}");
    assert!(html.contains("overflow-y:scroll"), "{html}");
    assert!(
        html.contains(r#"data-scroll-direction="vertical""#),
        "{html}"
    );
}

#[component]
fn HiddenHorizontal() -> Element {
    rsx! {
        ScrollArea {
            direction: ScrollDirection::Horizontal,
            scroll_type: ScrollType::Hidden,
            "content"
        }
    }
}

#[test]
fn hidden_horizontal_scroll_area_keeps_scrolling_but_hides_the_scrollbar() {
    let html = render(HiddenHorizontal);
    assert!(html.contains("overflow-x:scroll"), "{html}");
    assert!(html.contains("overflow-y:hidden"), "{html}");
    assert!(html.contains(r#"scrollbar-width="none""#), "{html}");
    assert!(
        html.contains(r#"data-scroll-direction="horizontal""#),
        "{html}"
    );
}

#[component]
fn AlwaysShowScrollbars() -> Element {
    rsx! {
        ScrollArea {
            always_show_scrollbars: true,
            "content"
        }
    }
}

#[test]
fn always_show_scrollbars_uses_the_always_show_visibility_class() {
    let html = render(AlwaysShowScrollbars);
    assert!(html.contains("dx-scroll-area-always-show"), "{html}");
}

#[test]
fn auto_hide_is_the_default_visibility_class() {
    let html = render(AutoBoth);
    assert!(html.contains("dx-scroll-area-auto-hide"), "{html}");
}
