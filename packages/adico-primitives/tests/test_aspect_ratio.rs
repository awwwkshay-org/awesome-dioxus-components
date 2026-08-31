//! Black-box tests for `adico_primitives::aspect_ratio`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/aspect_ratio.rs`.

use adico_primitives::aspect_ratio::AspectRatio;
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn WidescreenRatio() -> Element {
    rsx! {
        AspectRatio { ratio: 16.0 / 9.0, "content" }
    }
}

#[test]
fn a_16_by_9_ratio_pads_at_100_times_9_over_16_percent() {
    let html = render(WidescreenRatio);
    // 100 / (16/9) = 56.25
    assert!(html.contains("padding-bottom: 56.25%"), "{html}");
    assert!(html.contains("content"), "{html}");
}

#[component]
fn SquareRatio() -> Element {
    rsx! {
        AspectRatio { "content" }
    }
}

#[test]
fn the_default_ratio_is_square() {
    let html = render(SquareRatio);
    assert!(html.contains("padding-bottom: 100%"), "{html}");
}

#[component]
fn WideRatio() -> Element {
    rsx! {
        AspectRatio { ratio: 2.0, "content" }
    }
}

#[test]
fn a_ratio_greater_than_one_pads_less_than_100_percent() {
    let html = render(WideRatio);
    assert!(html.contains("padding-bottom: 50%"), "{html}");
}
