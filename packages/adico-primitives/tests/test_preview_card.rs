//! Black-box tests for `adico_primitives::preview_card`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/preview_card.rs`.
//!
//! Moved here (not newly written) from `preview_card.rs`'s own pre-existing inline `mod
//! tests`, per task 7.2 (`openspec/changes/deduplicate-primitives`): that module's own two
//! tests predate this repo's test-placement convention and are relocated as part of this D2
//! rewrite, not left behind. Both assert through `PreviewCard`/`PreviewCardContent` (the
//! facade), not `HoverCard` directly -- a facade test that bypasses the facade would prove
//! nothing about the defaults `preview_card.rs` actually supplies.

use adico_primitives::preview_card::{PreviewCard, PreviewCardContent, PreviewCardTrigger};
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

/// Dispatches a real mouseenter at the first `onmouseenter`-listening element in `dom`,
/// draining pending work once, without unwinding a panic (see the caller below).
fn hover_first_dispatch_only(dom: &mut VirtualDom) {
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
}

#[component]
fn ClosedPreviewCard() -> Element {
    rsx! {
        PreviewCard { open: Some(false),
            PreviewCardTrigger { "trigger" }
            PreviewCardContent { force_mount: false, "content" }
        }
    }
}

#[test]
fn closed_content_is_not_rendered_without_force_mount() {
    let html = render(ClosedPreviewCard);
    assert!(!html.contains("content"), "{html}");
}

#[component]
fn OpenPreviewCard() -> Element {
    rsx! {
        PreviewCard { open: Some(true),
            PreviewCardTrigger { "trigger" }
        }
    }
}

#[test]
fn open_root_marks_data_state_open() {
    let html = render(OpenPreviewCard);
    assert!(html.contains(r#"data-state="open""#), "{html}");
}

#[component]
fn OpenPreviewCardWithContent() -> Element {
    rsx! {
        PreviewCard { open: Some(true),
            PreviewCardTrigger { "trigger" }
            PreviewCardContent { "content" }
        }
    }
}

/// Regression coverage for task 7.2's facade defaults, not present before this rewrite:
/// `PreviewCardContent` must not carry `HoverCardContent`'s own default `role="tooltip"` --
/// its content is rich and potentially interactive, which the ARIA tooltip role's own spec
/// discourages (see this module's own doc comment).
#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn content_does_not_carry_a_tooltip_role() {
    // Same SSR-fallback-only gate `test_accordion.rs`/`test_tooltip.rs` document
    // elsewhere in this change: `PreviewCardContent` gates its markup on
    // `use_animated_open`, whose real (`web`/`native`) implementation only flips
    // its content-mounted signal from inside a `use_effect` a bare
    // `rebuild_in_place()` doesn't drive to completion.
    let html = render(OpenPreviewCardWithContent);
    assert!(html.contains("content"), "{html}");
    assert!(!html.contains(r#"role="tooltip""#), "{html}");
}

#[component]
fn UncontrolledClosedPreviewCard() -> Element {
    rsx! {
        PreviewCard { default_open: false,
            PreviewCardTrigger { "trigger" }
            PreviewCardContent { "content" }
        }
    }
}

/// Regression coverage for task 7.1/7.2 (`openspec/changes/deduplicate-primitives`): proves
/// `PreviewCard`'s facade (`preview_card.rs:120-121`) actually forwards its own non-zero
/// `delay_ms: 600`/`close_delay_ms: 300` defaults into `HoverCard`'s props, rather than the
/// forwarding line silently going missing in a future edit and `HoverCard`'s own `delay_ms:
/// 0` default winning instead -- a class of bug the compiler cannot catch, since both fields
/// are optional (`#[props(default = ..)]`) on `HoverCardProps`.
///
/// The proof works by exploiting `hover_intent::HoverIntent::request`'s own shape (see
/// `hover_intent.rs`): a request only reaches `crate::time::sleep` when its resolved
/// `delay_ms` is greater than zero. `crate::time::sleep`'s non-`wasm` implementation is
/// `tokio::time::sleep`, and this crate's own `[target.'cfg(not(target_family = "wasm"))'.
/// dependencies]` only enables `tokio`'s `"time"` feature -- not `"rt"`/`"macros"` -- so no
/// Tokio runtime exists in a bare `#[test]` (the same gap `test_toast.rs`'s own module doc
/// documents a failed `#[tokio::test(start_paused = true)]` attempt to work around, for the
/// same class of spawn+timer behavior). Driving that future to its first poll therefore
/// panics with Tokio's own "no reactor running" message -- deterministically, and only when
/// a genuinely non-zero delay was actually used. If the forwarding silently broke (effective
/// `delay_ms: 0`), the request would resolve instantly with no panic and the card would just
/// open -- exactly what `test_hover_card.rs`'s
/// `hovering_an_uncontrolled_trigger_opens_the_card_immediately` proves happens for a real
/// `delay_ms: 0`, the discriminating half of this same pair.
#[test]
fn preview_card_stays_closed_after_a_hover_because_its_delay_is_forwarded_and_not_zero() {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut dom = VirtualDom::new(UncontrolledClosedPreviewCard);
        hover_first_dispatch_only(&mut dom);
    }));
    let panic_payload = result.expect_err(
        "expected PreviewCard's configured (non-zero) delay to reach a real \
         tokio::time::sleep and panic outside a Tokio runtime -- if this did NOT panic, \
         PreviewCard's delay_ms/close_delay_ms silently failed to forward to HoverCard, \
         and the card opened instantly instead",
    );
    let message = panic_payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic_payload.downcast_ref::<&str>().copied())
        .unwrap_or_default();
    assert!(
        message.contains("Tokio") || message.contains("reactor"),
        "panicked, but not with the expected 'no Tokio runtime' message -- got: {message:?}"
    );
}
