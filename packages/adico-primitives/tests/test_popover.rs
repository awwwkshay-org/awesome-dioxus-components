//! Black-box tests for `adico_primitives::popover`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/popover.rs`.

use adico_primitives::popover::{PopoverContent, PopoverRoot, PopoverTrigger};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn OpenModalPopover() -> Element {
    rsx! {
        PopoverRoot { default_open: true,
            PopoverTrigger { "Show Popover" }
            PopoverContent { "Popover content" }
        }
    }
}

#[test]
fn an_open_popover_root_reports_open_state_and_renders_its_trigger() {
    let html = render(OpenModalPopover);
    assert!(html.contains(r#"data-state="open""#), "{html}");
    assert!(html.contains("Show Popover"), "{html}");
}

#[component]
fn ClosedPopover() -> Element {
    rsx! {
        PopoverRoot {
            PopoverTrigger { "Show Popover" }
            PopoverContent { "Popover content" }
        }
    }
}

#[test]
fn a_closed_popover_reports_closed_state_and_does_not_render_its_content() {
    let html = render(ClosedPopover);
    assert!(html.contains(r#"data-state="closed""#), "{html}");
    assert!(!html.contains("Popover content"), "{html}");
}

// `PopoverContent` gates its markup on `use_animated_open`, whose real (`web`/`native`)
// implementation only flips its content-mounted signal from inside a `use_effect` -- see
// `test_accordion.rs` for the same, already-established gap. This runs only on the
// SSR-fallback path, where `use_animated_open` returns `open` directly with no effect
// involved.
#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn an_open_modal_popover_s_content_renders_the_modal_dialog_role() {
    let html = render(OpenModalPopover);
    assert!(html.contains(r#"role="dialog""#), "{html}");
    assert!(html.contains(r#"aria-modal="true""#), "{html}");
    assert!(html.contains("Popover content"), "{html}");
}

#[component]
fn OpenNonModalPopover() -> Element {
    rsx! {
        PopoverRoot { default_open: true, is_modal: false,
            PopoverTrigger { "Show Popover" }
            PopoverContent { "Popover content" }
        }
    }
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn a_non_modal_popover_s_content_omits_aria_modal() {
    let html = render(OpenNonModalPopover);
    assert!(html.contains(r#"role="dialog""#), "{html}");
    assert!(!html.contains("aria-modal"), "{html}");
}
