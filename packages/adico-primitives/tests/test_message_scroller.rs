//! Black-box tests for `adico_primitives::message_scroller`'s `with_scroll_area`
//! opt-in (the in-place adoption shape for the shared scroll-area contract, see
//! `scroll_area.rs`'s module doc comment). Existing scroll-anchoring behavior has no
//! dedicated test file of its own (only its rustdoc doctest); this file adds targeted
//! coverage for the new opt-in only, per this repo's test-placement convention (every
//! test lives under `packages/adico-primitives/tests/`).

use adico_primitives::message_scroller::{MessageScroller, MessageScrollerViewport};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn DefaultViewport() -> Element {
    rsx! {
        MessageScroller {
            MessageScrollerViewport { "content" }
        }
    }
}

#[test]
fn with_scroll_area_defaults_to_false_and_stays_fully_headless() {
    let html = render(DefaultViewport);
    // No scroll-area class or style opinions leak in unless explicitly opted into --
    // this primitive must stay usable with zero visual opinions by default.
    assert!(!html.contains("dx-scroll-area"), "{html}");
    assert!(!html.contains("scrollbar-width"), "{html}");
}

#[component]
fn ScrollAreaViewport() -> Element {
    rsx! {
        MessageScroller {
            MessageScrollerViewport { with_scroll_area: true, class: "caller-class", "content" }
        }
    }
}

#[test]
fn with_scroll_area_true_merges_caller_class_with_the_visibility_class() {
    let html = render(ScrollAreaViewport);
    // Both classes appear together, contiguously, in one `class="..."` attribute --
    // the same SSR/CSR class-merge contract `ScrollArea` itself provides.
    assert!(
        html.contains(r#"class="dx-scroll-area-auto-hide caller-class""#),
        "{html}"
    );
    assert!(html.contains("scrollbar-width: none"), "{html}");
}

#[component]
fn ScrollAreaViewportNoExtraClass() -> Element {
    rsx! {
        MessageScroller {
            MessageScrollerViewport { with_scroll_area: true, "content" }
        }
    }
}

#[test]
fn with_scroll_area_true_and_no_caller_class_still_renders_the_visibility_class() {
    let html = render(ScrollAreaViewportNoExtraClass);
    assert!(
        html.contains(r#"class="dx-scroll-area-auto-hide""#),
        "{html}"
    );
}
