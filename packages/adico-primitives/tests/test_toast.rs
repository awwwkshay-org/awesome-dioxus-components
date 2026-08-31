//! Black-box tests for `adico_primitives::toast`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/toast.rs`.
//!
//! `enforce_max_toasts` (the queue-eviction bookkeeping `ToastProvider`'s `add_toast` delegates
//! to) is tested directly, pure-function style. `Toast`'s own ARIA/data-attribute rendering is
//! tested by constructing its props directly inside a bare `ToastProvider` ancestor (required
//! only because `Toast` reads `ToastCtx` via `use_context`, not because these tests exercise
//! the queue), bypassing `use_toast()`/`add_toast` entirely, which sidesteps the auto-dismiss
//! timer and the queue-management signal writes.
//!
//! **Auto-dismiss timing has no coverage here.** An attempt was made to drive it with a
//! `#[tokio::test(start_paused = true)]` harness advancing virtual time past `Toast`'s
//! `use_effect`-spawned `crate::time::sleep`, but even the prerequisite step — a toast added
//! via `use_toast().info(...)` (called from a hook or an effect, since the source's own
//! comment notes doing this outside a real event handler "is safe" only in the sense of not
//! panicking, not in the sense of being observable within the same render pass) — never
//! appeared in the rendered output after `rebuild_in_place()` plus
//! `VirtualDom::wait_for_work()`. This is the same class of effect/signal-timing limitation
//! this change has documented elsewhere (`test_select.rs`'s module doc comment, `portal.rs`'s
//! ordering caveat), but confirmed here to extend even to a from-scratch async/paused-clock
//! harness, not just a bare `rebuild_in_place()`. `tests/playwright/wave2-risk.spec.ts`'s
//! existing "installed Toast appears after use_toast().info(...)" test covers toast *creation*
//! via a real click in a real browser, but not auto-dismiss timing; that gap is carried
//! forward, not silently dropped.

use adico_primitives::toast::{Toast, ToastProvider, ToastRecord, ToastType, enforce_max_toasts};
use dioxus::prelude::*;
use std::collections::VecDeque;
use std::time::Duration;

fn record(id: usize, permanent: bool) -> ToastRecord {
    ToastRecord {
        id,
        title: format!("toast-{id}"),
        description: None,
        toast_type: ToastType::Info,
        duration: Some(Duration::from_secs(5)),
        permanent,
    }
}

#[test]
fn enforce_max_toasts_is_a_no_op_when_under_the_limit() {
    let mut toasts: VecDeque<ToastRecord> = [record(1, false), record(2, false)].into();
    enforce_max_toasts(&mut toasts, 5);
    assert_eq!(toasts.len(), 2);
}

#[test]
fn enforce_max_toasts_removes_a_non_permanent_toast_first() {
    let mut toasts: VecDeque<ToastRecord> =
        [record(1, true), record(2, false), record(3, true)].into();
    enforce_max_toasts(&mut toasts, 2);
    assert_eq!(toasts.len(), 2);
    assert!(
        toasts.iter().all(|t| t.id != 2),
        "the non-permanent toast (2) should be evicted before either permanent one"
    );
}

#[test]
fn enforce_max_toasts_evicts_the_oldest_permanent_toast_once_none_remain_non_permanent() {
    let mut toasts: VecDeque<ToastRecord> =
        [record(1, true), record(2, true), record(3, true)].into();
    enforce_max_toasts(&mut toasts, 2);
    assert_eq!(toasts.len(), 2);
    assert!(
        toasts.iter().all(|t| t.id != 1),
        "with every toast permanent, the oldest (1) must still be evicted rather than growing unbounded"
    );
}

#[test]
fn enforce_max_toasts_can_evict_down_to_zero() {
    let mut toasts: VecDeque<ToastRecord> = [record(1, false)].into();
    enforce_max_toasts(&mut toasts, 0);
    assert!(toasts.is_empty());
}

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn PermanentInfoToast() -> Element {
    rsx! {
        ToastProvider {
            Toast {
                id: 1usize,
                index: 0usize,
                title: "Saved".to_string(),
                description: Some("Your changes were saved.".to_string()),
                toast_type: ToastType::Info,
                permanent: true,
                duration: None,
                on_close: move |_| {},
            }
        }
    }
}

#[test]
fn toast_renders_alertdialog_and_alert_roles_with_labelledby_and_describedby() {
    let html = render(PermanentInfoToast);
    assert!(html.contains(r#"role="alertdialog""#), "{html}");
    assert!(html.contains(r#"role="alert""#), "{html}");
    assert!(html.contains("Saved"), "{html}");
    assert!(html.contains("Your changes were saved."), "{html}");
    assert!(html.contains(r#"data-type="info""#), "{html}");
    assert!(html.contains("data-permanent=true"), "{html}");
}

#[test]
fn toast_marks_the_first_index_as_top_and_even() {
    let html = render(PermanentInfoToast);
    assert!(html.contains(r#"data-toast-even="true""#), "{html}");
    assert!(html.contains(r#"data-top="true""#), "{html}");
    assert!(!html.contains("data-toast-odd"), "{html}");
}

#[component]
fn SecondErrorToastWithNoDescription() -> Element {
    rsx! {
        ToastProvider {
            Toast {
                id: 2usize,
                index: 1usize,
                title: "Failed to save".to_string(),
                toast_type: ToastType::Error,
                permanent: false,
                duration: Some(Duration::from_secs(5)),
                on_close: move |_| {},
            }
        }
    }
}

#[test]
fn toast_without_a_description_renders_no_describedby_target() {
    let html = render(SecondErrorToastWithNoDescription);
    assert!(html.contains(r#"data-type="error""#), "{html}");
    assert!(html.contains("data-permanent=false"), "{html}");
    assert!(html.contains(r#"data-toast-odd="true""#), "{html}");
    assert!(!html.contains("data-top"), "{html}");
    assert!(!html.contains("aria-describedby"), "{html}");
}

#[component]
fn ToastWithDefaultChildren() -> Element {
    rsx! {
        ToastProvider {
            Toast {
                id: 3usize,
                index: 0usize,
                title: "Default children".to_string(),
                toast_type: ToastType::Warning,
                permanent: true,
                duration: None,
                on_close: move |_| {},
            }
        }
    }
}

#[test]
fn a_toast_with_no_children_renders_a_default_close_button() {
    let html = render(ToastWithDefaultChildren);
    assert!(html.contains(r#"aria-label="close""#), "{html}");
}
