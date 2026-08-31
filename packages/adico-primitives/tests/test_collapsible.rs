//! Black-box tests for `adico_primitives::collapsible`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/collapsible.rs`.

use adico_primitives::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn ClosedCollapsible() -> Element {
    rsx! {
        Collapsible {
            CollapsibleTrigger { "Toggle" }
            CollapsibleContent { "Secret content" }
        }
    }
}

#[test]
fn a_closed_collapsible_reports_closed_state_and_does_not_render_its_content() {
    let html = render(ClosedCollapsible);
    assert!(html.contains("data-open=false"), "{html}");
    assert!(html.contains("aria-expanded=false"), "{html}");
    assert!(!html.contains("Secret content"), "{html}");
}

#[test]
fn the_trigger_s_aria_controls_matches_the_content_s_id() {
    let html = render(ClosedCollapsible);
    let marker = html.find("Toggle").expect("trigger renders its text");
    let head = &html[..marker];
    let attr = "aria-controls=\"";
    let start = head.rfind(attr).expect("trigger has aria-controls") + attr.len();
    let end = head[start..].find('"').unwrap() + start;
    let controls_id = &head[start..end];

    assert!(html.contains(&format!(r#"id="{controls_id}""#)), "{html}");
}

#[component]
fn OpenCollapsible() -> Element {
    rsx! {
        Collapsible { default_open: true,
            CollapsibleTrigger { "Toggle" }
            CollapsibleContent { "Secret content" }
        }
    }
}

#[test]
fn an_open_collapsible_reports_open_state_and_renders_its_content() {
    let html = render(OpenCollapsible);
    assert!(html.contains("data-open=true"), "{html}");
    assert!(html.contains("aria-expanded=true"), "{html}");
    assert!(html.contains("Secret content"), "{html}");
}

#[component]
fn ClosedKeepMountedCollapsible() -> Element {
    rsx! {
        Collapsible { keep_mounted: true,
            CollapsibleTrigger { "Toggle" }
            CollapsibleContent { "Secret content" }
        }
    }
}

#[test]
fn a_closed_but_keep_mounted_collapsible_still_renders_its_content() {
    let html = render(ClosedKeepMountedCollapsible);
    assert!(html.contains("data-open=false"), "{html}");
    assert!(html.contains("Secret content"), "{html}");
}

#[component]
fn DisabledCollapsible() -> Element {
    rsx! {
        Collapsible { disabled: true,
            CollapsibleTrigger { "Toggle" }
            CollapsibleContent { "Secret content" }
        }
    }
}

#[test]
fn a_disabled_collapsible_disables_its_trigger() {
    let html = render(DisabledCollapsible);
    assert!(html.contains("disabled=true"), "{html}");
    assert!(html.contains("data-disabled=true"), "{html}");
}
