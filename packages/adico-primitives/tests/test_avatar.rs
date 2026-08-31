//! Black-box tests for `adico_primitives::avatar`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/avatar.rs`.
//!
//! `Avatar`'s state machine only transitions away from its initial `Empty` state through
//! `AvatarImage`'s own `use_effect` (setting `Loading`/`Empty` from `src`) or real
//! `onload`/`onerror` browser events — the same effect-driven-state limitation documented
//! elsewhere in this change (`test_select.rs`'s module doc comment), so these tests cover only
//! what's observable from a single `rebuild_in_place()` pass: the initial `Empty` state and an
//! explicit `AvatarFallback`'s always-visible-while-empty rendering.

use adico_primitives::avatar::{Avatar, AvatarFallback};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn EmptyAvatarWithFallback() -> Element {
    rsx! {
        Avatar { aria_label: "Basic avatar".to_string(),
            AvatarFallback { "EA" }
        }
    }
}

#[test]
fn an_avatar_with_no_image_renders_the_img_role_and_empty_state() {
    let html = render(EmptyAvatarWithFallback);
    assert!(html.contains(r#"role="img""#), "{html}");
    assert!(html.contains(r#"data-state="empty""#), "{html}");
    assert!(html.contains(r#"aria-label="Basic avatar""#), "{html}");
}

#[test]
fn an_avatar_with_no_image_renders_its_explicit_fallback() {
    let html = render(EmptyAvatarWithFallback);
    assert!(html.contains("EA"), "{html}");
}

#[component]
fn EmptyAvatarWithNoFallback() -> Element {
    rsx! { Avatar {} }
}

#[test]
fn an_avatar_with_no_image_and_no_fallback_child_renders_nothing_extra() {
    // The built-in "??" placeholder only appears once an AvatarImage child has registered
    // itself (`has_image_child`), which — like the rest of the state machine — is
    // effect-driven and does not complete under a bare `rebuild_in_place()`. With no image
    // and no explicit fallback, this just confirms the root renders cleanly with nothing
    // inside it.
    let html = render(EmptyAvatarWithNoFallback);
    assert!(html.contains(r#"role="img""#), "{html}");
    assert!(!html.contains("??"), "{html}");
}
