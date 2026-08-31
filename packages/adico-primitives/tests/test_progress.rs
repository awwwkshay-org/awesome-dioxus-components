//! Black-box tests for `adico_primitives::progress`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/progress.rs`.

use adico_primitives::progress::{Progress, ProgressIndicator};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn HalfwayProgress() -> Element {
    rsx! {
        Progress { value: 50.0, ProgressIndicator {} }
    }
}

#[test]
fn a_determinate_progress_bar_reports_its_value_and_loading_state() {
    let html = render(HalfwayProgress);
    assert!(html.contains(r#"role="progressbar""#), "{html}");
    assert!(html.contains("aria-valuemin=0"), "{html}");
    assert!(html.contains("aria-valuemax=100"), "{html}");
    assert!(html.contains("aria-valuenow=50"), "{html}");
    assert!(html.contains(r#"data-state="loading""#), "{html}");
    assert!(html.contains(r#"data-value="50""#), "{html}");
    assert!(html.contains("--progress-value: 50%"), "{html}");
}

#[component]
fn IndeterminateProgress() -> Element {
    rsx! {
        Progress { value: None::<f64>, ProgressIndicator {} }
    }
}

#[test]
fn a_progress_bar_with_no_value_reports_indeterminate_state_and_omits_valuenow() {
    let html = render(IndeterminateProgress);
    assert!(html.contains(r#"data-state="indeterminate""#), "{html}");
    assert!(!html.contains("aria-valuenow"), "{html}");
}

#[component]
fn CustomMaxProgress() -> Element {
    rsx! {
        Progress { value: 3.0, max: 4.0, ProgressIndicator {} }
    }
}

#[test]
fn a_custom_max_scales_the_percentage_correctly() {
    let html = render(CustomMaxProgress);
    assert!(html.contains("aria-valuemax=4"), "{html}");
    assert!(html.contains("--progress-value: 75%"), "{html}");
}
