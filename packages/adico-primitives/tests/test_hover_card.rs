//! Black-box tests for `adico_primitives::hover_card`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/hover_card.rs`.

use adico_primitives::hover_card::{HoverCard, HoverCardContent, HoverCardTrigger};
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

/// Dispatches a real mouseenter at the first `onmouseenter`-listening element in `dom`
/// and returns the freshly re-rendered HTML.
fn hover_first(dom: &mut VirtualDom) -> String {
    let edits = dom.rebuild_to_vec();
    let id = edits
        .edits
        .iter()
        .find_map(|edit| match edit {
            Mutation::NewEventListener { name, id } if name == "mouseenter" => Some(*id),
            _ => None,
        })
        .expect("a mouseenter listener");
    set_event_converter(Box::new(SerializedHtmlEventConverter));
    let event = Event::new(
        EventData::Mouse(SerializedMouseData::default()).into_any(),
        true,
    );
    dom.runtime().handle_event("mouseenter", event, id);
    dom.render_immediate_to_vec();
    dioxus_ssr::render(dom)
}

#[component]
fn OpenHoverCard() -> Element {
    rsx! {
        HoverCard { default_open: true,
            HoverCardTrigger { "Dioxus" }
            HoverCardContent { "Rich content" }
        }
    }
}

#[test]
fn an_open_hover_card_reports_open_state_and_a_button_role_trigger_describedby_the_content() {
    let html = render(OpenHoverCard);
    assert!(html.contains(r#"data-state="open""#), "{html}");
    assert!(html.contains(r#"role="button""#), "{html}");
    // Unlike tooltip.rs's TooltipContent, HoverCardContent's id is set eagerly by
    // use_id_or/use_unique_id at render time (no use_effect indirection), so the trigger's
    // aria-describedby is directly assertable here without the effect-driven-state gap
    // documented elsewhere in this change.
    assert!(html.contains("aria-describedby"), "{html}");
}

#[component]
fn ClosedDisabledHoverCard() -> Element {
    rsx! {
        HoverCard { disabled: true,
            HoverCardTrigger { "Dioxus" }
            HoverCardContent { force_mount: false, "Rich content" }
        }
    }
}

#[test]
fn a_closed_disabled_hover_card_reports_closed_and_disabled_state_and_omits_describedby() {
    let html = render(ClosedDisabledHoverCard);
    assert!(html.contains(r#"data-state="closed""#), "{html}");
    assert!(html.contains("data-disabled=true"), "{html}");
    assert!(!html.contains("aria-describedby"), "{html}");
}

#[component]
fn ClosedHoverCardForceMounted() -> Element {
    rsx! {
        HoverCard {
            HoverCardTrigger { "Dioxus" }
            HoverCardContent { "Rich content" }
        }
    }
}

// `force_mount` (which defaults to `true`) only bypasses HoverCardContent's own early
// `!is_open && !force_mount` return; every path still ends at `use_animated_open`'s `render()`
// gate, whose SSR-fallback implementation (see lib.rs's `#[cfg(not(any(feature = "web", feature
// = "native")))] use_animated_open`) returns `open` unmodified with no force-mount override --
// so a closed HoverCard's content does not actually render even with the (default)
// `force_mount: true`, a real, previously-undocumented interaction surfaced by writing this
// test, not a change made here. See `src/hover_card.rs`'s module doc comment.
#[test]
fn a_closed_hover_card_with_default_force_mount_still_does_not_render_its_content_on_ssr() {
    let html = render(ClosedHoverCardForceMounted);
    // Positive anchor first: proves the tree actually rendered, so the negative assertion
    // below isn't vacuously true from an unrelated render failure.
    assert!(html.contains("Dioxus"), "{html}");
    assert!(!html.contains("Rich content"), "{html}");
}

#[test]
fn a_closed_hover_card_without_force_mount_does_not_render_its_content() {
    let html = render(ClosedDisabledHoverCard);
    assert!(html.contains("Dioxus"), "{html}");
    assert!(!html.contains("Rich content"), "{html}");
}

#[component]
fn ClosedHoverCard() -> Element {
    rsx! {
        HoverCard {
            HoverCardTrigger { "Dioxus" }
            HoverCardContent { "Rich content" }
        }
    }
}

/// Regression coverage for task 7.1 (`openspec/changes/deduplicate-primitives`):
/// before this task, `HoverCardTrigger`'s mouseenter handler called
/// `ctx.set_open.call(true)` directly. Migrated onto `hover_intent::HoverIntent`
/// (needed so `delay_ms`/`close_delay_ms` work at all), the *default* `delay_ms:
/// 0` case still resolves via a spawned task rather than synchronously (see
/// design.md's D2 Correction) -- this test proves that resolution still actually
/// happens within one dispatch-and-render cycle, not just that the static
/// declarative `default_open`/`open` props still render correctly (which none of
/// this file's other tests exercise via a real event at all). This is also the
/// discriminating half of the pair proving `preview_card`'s facade actually
/// forwards its own non-zero `delay_ms`/`close_delay_ms` to `HoverCard` (see
/// `test_preview_card.rs`'s
/// `preview_card_stays_closed_after_a_hover_because_its_delay_is_forwarded_and_not_zero`):
/// if the same dispatch-and-render technique did *not* open a bare `HoverCard`
/// here, that other test's "still closed" assertion would be meaningless.
#[test]
fn hovering_an_uncontrolled_trigger_opens_the_card_immediately() {
    let mut dom = VirtualDom::new(ClosedHoverCard);
    let html = hover_first(&mut dom);
    assert!(html.contains(r#"data-state="open""#), "{html}");
    assert!(html.contains("Rich content"), "{html}");
}
