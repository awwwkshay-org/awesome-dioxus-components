//! Black-box tests for `adico_primitives::tooltip`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/tooltip.rs`.

use adico_primitives::tooltip::{Tooltip, TooltipContent, TooltipTrigger};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn OpenTooltip() -> Element {
    rsx! {
        Tooltip { default_open: true,
            TooltipTrigger { "Hover me" }
            TooltipContent { "Tooltip text" }
        }
    }
}

#[test]
fn an_open_tooltip_reports_open_state_and_a_describedby_trigger() {
    let html = render(OpenTooltip);
    assert!(html.contains(r#"data-state="open""#), "{html}");
    assert!(html.contains("Hover me"), "{html}");
    // `TooltipContent` publishes its real id into the trigger's `aria-describedby` from a
    // `use_effect`, which a bare `rebuild_in_place()` does not drive to completion -- the same
    // effect-driven-state limitation this change has documented elsewhere (e.g.
    // test_select.rs, test_accordion.rs) -- so only presence, not the exact id match, is
    // asserted here.
    assert!(html.contains("aria-describedby"), "{html}");
}

#[component]
fn ClosedDisabledTooltip() -> Element {
    rsx! {
        Tooltip { disabled: true,
            TooltipTrigger { "Hover me" }
            TooltipContent { "Tooltip text" }
        }
    }
}

#[test]
fn a_closed_disabled_tooltip_reports_closed_and_disabled_state() {
    let html = render(ClosedDisabledTooltip);
    assert!(html.contains(r#"data-state="closed""#), "{html}");
    assert!(html.contains("data-disabled=true"), "{html}");
}

// `TooltipContent` gates its markup on `use_animated_open`, whose real (`web`/`native`)
// implementation only flips its content-mounted signal from inside a `use_effect` -- see
// `test_accordion.rs` for the same, already-established gap. This runs only on the
// SSR-fallback path, where `use_animated_open` returns `open` directly with no effect
// involved.
#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn an_open_tooltip_s_content_renders_the_tooltip_role_and_its_children() {
    let html = render(OpenTooltip);
    assert!(html.contains(r#"role="tooltip""#), "{html}");
    assert!(html.contains("Tooltip text"), "{html}");
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn a_closed_tooltip_does_not_render_its_content() {
    let html = render(ClosedDisabledTooltip);
    assert!(!html.contains("Tooltip text"), "{html}");
}
