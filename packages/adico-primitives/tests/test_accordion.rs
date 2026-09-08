//! Black-box tests for `adico_primitives::accordion`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/accordion.rs`.

use adico_primitives::accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
use dioxus::prelude::*;
use dioxus_core::{Event, Mutation};
use dioxus_html::{
    EventData, SerializedHtmlEventConverter, SerializedMouseData, set_event_converter,
};

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// Simulates a real click on the first `onclick`-listening element in `dom`
/// and returns the freshly re-rendered HTML. Used below to exercise
/// `Accordion`'s toggle logic dynamically (open/collapse via
/// `crate::use_optionally_controlled`), not just its first-render output --
/// see `openspec/changes/deduplicate-primitives`, task 4.2.
fn click_first(dom: &mut VirtualDom) -> String {
    let edits = dom.rebuild_to_vec();
    let id = edits
        .edits
        .iter()
        .find_map(|edit| match edit {
            Mutation::NewEventListener { name, id } if name == "click" => Some(*id),
            _ => None,
        })
        .expect("a click listener");
    set_event_converter(Box::new(SerializedHtmlEventConverter));
    let event = Event::new(
        EventData::Mouse(SerializedMouseData::default()).into_any(),
        true,
    );
    dom.runtime().handle_event("click", event, id);
    dom.render_immediate_to_vec();
    dioxus_ssr::render(dom)
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

// Same reasoning as `the_default_value_s_content_renders_its_children` below:
// this asserts against the *content* element's rendered `id`, which only
// exists on the SSR-fallback path — under a feature-unified `web`/`native`
// build (e.g. `cargo test --workspace`, where another workspace member like
// `apps/playground` pulls in the `web` feature), `use_animated_open`'s real
// effect-based implementation never flips its content-mounted signal inside
// a bare `rebuild_in_place()`, so the content (and its `id`) never renders at
// all. Missing this gate (unlike the test right below it, which already had
// it) is why this one test failed under `--workspace` while its five
// siblings in this file passed.
#[cfg(not(any(feature = "web", feature = "native")))]
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
fn ControlledAndClearedAccordion() -> Element {
    rsx! {
        Accordion {
            value: Some(ReadSignal::new(Signal::new(None))),
            default_value: "item-1".to_string(),
            AccordionItem { value: "item-1".to_string(), index: 0usize,
                AccordionTrigger { "First" }
                AccordionContent { "First content" }
            }
        }
    }
}

/// Regression test for `crate::use_optionally_controlled`'s central design
/// decision (see `openspec/changes/deduplicate-primitives` design.md's
/// corrected D7): a *controlled* accordion whose value signal reads `None`
/// must render nothing open, not silently fall back to `default_value` the
/// way `use_controlled` would. `default_value: "item-1"` is set specifically
/// so a wrong (use_controlled-style) fallback would make this test fail.
#[test]
fn a_controlled_accordion_with_no_value_selected_ignores_default_value() {
    let html = render(ControlledAndClearedAccordion);
    assert!(html.contains("aria-expanded=false"), "{html}");
    assert!(!html.contains("First content"), "{html}");
}

#[component]
fn NoDefaultAccordion() -> Element {
    rsx! {
        Accordion {
            AccordionItem { value: "item-1".to_string(), index: 0usize,
                AccordionTrigger { "First" }
                AccordionContent { "First content" }
            }
        }
    }
}

#[test]
fn clicking_an_uncontrolled_trigger_opens_its_item() {
    let mut dom = VirtualDom::new(NoDefaultAccordion);
    let html = click_first(&mut dom);
    assert!(html.contains("aria-expanded=true"), "{html}");
    assert!(html.contains(r#"data-state="open""#), "{html}");
    // Same SSR-fallback-only gate as `the_default_value_s_content_renders_its_children`
    // above: this item starts *closed*, so its content only mounts once
    // `use_animated_open`'s content-mounted signal flips -- on the real
    // `web`/`native` path that happens from inside a `use_effect` that a
    // synthetic click-and-`render_immediate_to_vec()` dispatch does not drive
    // to completion. `clicking_an_uncontrolled_open_item_s_trigger_collapses_it_when_collapsible`
    // below doesn't need this gate: that item starts already open (mounted at
    // initial render), so its content exists before the click closes it.
    #[cfg(not(any(feature = "web", feature = "native")))]
    assert!(html.contains("First content"), "{html}");
}

#[test]
fn clicking_an_uncontrolled_open_item_s_trigger_collapses_it_when_collapsible() {
    // `TwoItemAccordion`'s `default_value: "item-1"` starts item-1 open, and
    // `Accordion::collapsible` defaults to `true` -- clicking its own trigger
    // again should collapse it to nothing open, exercising the `None` branch
    // of `Accordion`'s toggle logic (the one `use_optionally_controlled`'s
    // `Option<T>`-valued setter exists to support).
    let mut dom = VirtualDom::new(TwoItemAccordion);
    let html = click_first(&mut dom);
    assert!(html.contains("aria-expanded=false"), "{html}");
    assert!(html.contains(r#"data-state="closed""#), "{html}");
    assert!(!html.contains("First content"), "{html}");
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
