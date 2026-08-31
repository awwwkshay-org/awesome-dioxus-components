//! Black-box tests for `adico_primitives::tabs`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/tabs.rs`.
//!
//! `TabContent`'s `aria-controls` linkage (published into `TabsContext::tab_content_ids` via a
//! `use_effect`) is not assertable here — the same effect-driven-state limitation this change
//! has documented elsewhere (e.g. `test_select.rs`'s module doc comment) — but `TabContent`'s
//! own selected/hidden render state does not depend on that effect, so it stays testable.

use adico_primitives::tabs::{TabContent, TabList, TabTrigger, Tabs};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn TwoTabs() -> Element {
    rsx! {
        Tabs { default_value: "tab1".to_string(), horizontal: true,
            TabList {
                TabTrigger { value: "tab1".to_string(), index: 0usize, "Tab 1" }
                TabTrigger { value: "tab2".to_string(), index: 1usize, disabled: true, "Tab 2" }
            }
            TabContent { index: 0usize, value: "tab1".to_string(), "Tab 1 Content" }
            TabContent { index: 1usize, value: "tab2".to_string(), "Tab 2 Content" }
        }
    }
}

#[test]
fn tabs_render_tablist_tab_and_tabpanel_roles() {
    let html = render(TwoTabs);
    assert!(html.contains(r#"role="tablist""#), "{html}");
    assert!(html.contains(r#"role="tab""#), "{html}");
    assert!(html.contains(r#"role="tabpanel""#), "{html}");
    assert!(html.contains(r#"data-orientation="horizontal""#), "{html}");
}

#[test]
fn the_default_value_s_trigger_and_panel_are_marked_active() {
    let html = render(TwoTabs);
    assert!(html.contains("aria-selected=true"), "{html}");
    assert!(html.contains(r#"data-state="active""#), "{html}");
    assert!(html.contains("Tab 1 Content"), "{html}");
}

#[test]
fn the_inactive_panel_is_hidden_and_not_rendered() {
    let html = render(TwoTabs);
    assert!(html.contains("hidden=true"), "{html}");
    assert!(!html.contains("Tab 2 Content"), "{html}");
}

#[test]
fn the_disabled_trigger_is_disabled_and_marked() {
    let html = render(TwoTabs);
    assert!(html.contains("disabled=true"), "{html}");
    assert!(html.contains("data-disabled=true"), "{html}");
}

#[component]
fn VerticalTabsSecondActive() -> Element {
    rsx! {
        Tabs { default_value: "tab2".to_string(), horizontal: false,
            TabList {
                TabTrigger { value: "tab1".to_string(), index: 0usize, "Tab 1" }
                TabTrigger { value: "tab2".to_string(), index: 1usize, "Tab 2" }
            }
            TabContent { index: 0usize, value: "tab1".to_string(), "Tab 1 Content" }
            TabContent { index: 1usize, value: "tab2".to_string(), "Tab 2 Content" }
        }
    }
}

#[test]
fn a_non_default_first_tab_selects_the_matching_trigger_and_panel() {
    let html = render(VerticalTabsSecondActive);
    assert!(html.contains(r#"data-orientation="vertical""#), "{html}");
    assert!(html.contains("Tab 2 Content"), "{html}");
    assert!(!html.contains("Tab 1 Content"), "{html}");
}
