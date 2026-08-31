//! Black-box tests for `adico_primitives::accordion`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/accordion.rs`.

use adico_primitives::accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn TwoItemAccordion() -> Element {
    rsx! {
        Accordion { default_value: "item-1".to_string(),
            AccordionItem { value: "item-1".to_string(), index: 0usize,
                AccordionTrigger { "First" }
                AccordionContent { "First content" }
            }
            AccordionItem { value: "item-2".to_string(), index: 1usize, disabled: true,
                AccordionTrigger { "Second" }
                AccordionContent { "Second content" }
            }
        }
    }
}

#[test]
fn the_default_value_s_item_reports_open_and_expanded() {
    let html = render(TwoItemAccordion);
    assert!(html.contains("data-open=true"), "{html}");
    assert!(html.contains("aria-expanded=true"), "{html}");
    assert!(html.contains(r#"data-state="open""#), "{html}");
}

#[test]
fn the_non_default_item_reports_closed_and_does_not_render_its_content() {
    let html = render(TwoItemAccordion);
    assert!(html.contains("data-open=false"), "{html}");
    assert!(html.contains("aria-expanded=false"), "{html}");
    assert!(html.contains(r#"data-state="closed""#), "{html}");
}

#[test]
fn the_disabled_item_s_trigger_is_disabled_and_marked() {
    let html = render(TwoItemAccordion);
    assert!(html.contains("disabled=true"), "{html}");
    assert!(html.contains("data-disabled=true"), "{html}");
}

#[test]
fn the_trigger_s_aria_controls_matches_its_content_s_id() {
    let html = render(TwoItemAccordion);
    let marker = html.find("First").expect("trigger renders its text");
    let head = &html[..marker];
    let attr = "aria-controls=\"";
    let start = head.rfind(attr).expect("trigger has aria-controls") + attr.len();
    let end = head[start..].find('"').unwrap() + start;
    let controls_id = &head[start..end];

    assert!(html.contains(&format!(r#"id="{controls_id}""#)), "{html}");
}

// `AccordionContent` gates its markup on `use_animated_open`, whose real (`web`/`native`)
// implementation only flips its content-mounted signal from inside a `use_effect` that a plain
// `rebuild_in_place()` does not itself drive to completion — the same, already-established
// precedent `menu.rs`'s/`date_picker.rs`'s/`test_select.rs`'s tests document for this exact
// class of test. This runs only on the SSR-fallback path (no `web`/`native` feature), where
// `use_animated_open` returns `open` directly with no effect involved.
#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn the_default_value_s_content_renders_its_children() {
    let html = render(TwoItemAccordion);
    assert!(html.contains("First content"), "{html}");
    assert!(!html.contains("Second content"), "{html}");
}

#[component]
fn MultiAccordion() -> Element {
    rsx! {
        adico_primitives::accordion::AccordionMulti {
            default_values: vec!["a".to_string(), "b".to_string()],
            AccordionItem { value: "a".to_string(), index: 0usize,
                AccordionTrigger { "A" }
                AccordionContent { "A content" }
            }
            AccordionItem { value: "b".to_string(), index: 1usize,
                AccordionTrigger { "B" }
                AccordionContent { "B content" }
            }
            AccordionItem { value: "c".to_string(), index: 2usize,
                AccordionTrigger { "C" }
                AccordionContent { "C content" }
            }
        }
    }
}

#[test]
fn accordion_multi_marks_every_default_value_open() {
    let html = render(MultiAccordion);
    let open_count = html.matches("data-open=true").count();
    // 2 items (A, B) each render `data-open=true` twice: once on AccordionItem's own div,
    // once on AccordionContent (SSR-fallback path, no `web`/`native` feature).
    assert!(open_count >= 2, "{html}");
    assert!(html.contains("data-open=false"), "{html}");
}

#[component]
fn HorizontalDisabledAccordion() -> Element {
    rsx! {
        Accordion { horizontal: true, disabled: true,
            AccordionItem { value: "a".to_string(), index: 0usize,
                AccordionTrigger { "A" }
                AccordionContent { "A content" }
            }
        }
    }
}

#[test]
fn a_disabled_accordion_root_marks_data_disabled_and_disables_every_trigger() {
    let html = render(HorizontalDisabledAccordion);
    assert!(html.contains("data-disabled=true"), "{html}");
    assert!(html.contains("disabled=true"), "{html}");
}
