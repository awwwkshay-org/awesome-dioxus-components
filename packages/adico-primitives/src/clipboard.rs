// No dedicated WAI-ARIA APG pattern applies to clipboard copying itself; the
// consuming button owns its own semantics (e.g. a plain button with an
// `aria-label` reflecting idle/copied/failed state).

//! Target-gated clipboard-copy support (`use_clipboard`), following the same
//! feature-gated `document::eval` pattern `persisted_state.rs` already
//! established for browser interop: the actual clipboard API call stays
//! behind this primitive adapter, never called directly from registry UI
//! source (this repo's architecture rule -- see `CLAUDE.md`).
//!
//! There is no native-target (desktop/mobile) clipboard integration in this
//! crate today, so non-`web` targets resolve to [`ClipboardStatus::Failed`]
//! rather than silently pretending to succeed.

use std::time::Duration;

use dioxus::prelude::*;

use crate::time::sleep;

/// How long a resolved status ([`ClipboardStatus::Copied`] or
/// [`ClipboardStatus::Failed`]) stays visible before reverting to
/// [`ClipboardStatus::Idle`].
const STATUS_REVERT_DELAY: Duration = Duration::from_millis(1600);

/// The result of the most recent copy attempt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClipboardStatus {
    /// No copy attempt is in flight or was recently resolved.
    #[default]
    Idle,
    /// The most recent copy attempt succeeded.
    Copied,
    /// The most recent copy attempt failed (denied permission, or no
    /// clipboard integration exists on the running target).
    Failed,
}

/// Copies text to the system clipboard, resolving to a transient status
/// rather than a bare `bool` so a consumer can render a confirmation without
/// managing its own revert timer.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::clipboard::{ClipboardStatus, use_clipboard};
/// #[component]
/// fn Demo() -> Element {
///     let (status, copy) = use_clipboard();
///     rsx! {
///         button { onclick: move |_| copy("hello".to_string()),
///             match status() {
///                 ClipboardStatus::Copied => "Copied!",
///                 ClipboardStatus::Failed => "Copy failed",
///                 ClipboardStatus::Idle => "Copy",
///             }
///         }
///     }
/// }
/// ```
pub fn use_clipboard() -> (Memo<ClipboardStatus>, Callback<String>) {
    let mut status = use_signal(ClipboardStatus::default);
    let copy = use_callback(move |text: String| {
        spawn(async move {
            let resolved = if copy_to_clipboard(text).await {
                ClipboardStatus::Copied
            } else {
                ClipboardStatus::Failed
            };
            status.set(resolved);
            sleep(STATUS_REVERT_DELAY).await;
            status.set(ClipboardStatus::Idle);
        });
    });
    // Not a redundant closure: `status()` is a tracked reactive read, while
    // clippy's suggested `*status` goes through `Deref` and silently skips
    // Dioxus's subscription tracking, breaking reactivity (same rationale as
    // `date_picker.rs`'s `effective_open`).
    #[allow(clippy::redundant_closure)]
    let value = use_memo(move || status());
    (value, copy)
}

#[cfg(feature = "web")]
async fn copy_to_clipboard(text: String) -> bool {
    let mut eval = dioxus_document::eval(
        "const text = await dioxus.recv();
        try {
            await navigator.clipboard.writeText(text);
            dioxus.send(true);
        } catch (error) {
            dioxus.send(false);
        }",
    );
    if eval.send(text).is_err() {
        return false;
    }
    eval.recv::<bool>().await.unwrap_or(false)
}

#[cfg(not(feature = "web"))]
async fn copy_to_clipboard(_text: String) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_defaults_to_idle() {
        assert_eq!(ClipboardStatus::default(), ClipboardStatus::Idle);
    }

    // `use_clipboard`'s full status transition (Idle -> Copied/Failed ->
    // Idle) is a hook: it needs a live Dioxus runtime (signals, `spawn`, and
    // the `time` module's target-gated sleep) to drive, which this crate has
    // no headless async test harness for (no `tokio`/`futures` test runner
    // dev-dependency; `dioxus-ssr` here is used only for static-render
    // assertions elsewhere in the crate, not for async hook execution). The
    // `document::eval` path itself needs a real browser regardless. Verified
    // live instead: `dx serve` + a real `CopyButton` click, confirming both
    // the clipboard write and the Copied -> Idle revert.
    //
    // What's covered here without a runtime: the pure, cfg-gated logic
    // `copy_to_clipboard` reduces to on a non-`web` build -- always `false`,
    // never a silent pretend-success. This crate's default test features
    // omit `web`, so this is exactly the branch that compiles under `cargo
    // test -p adico-primitives`.
    #[test]
    fn non_web_targets_report_failure_not_silent_success() {
        // No async runtime available in this crate's tests; poll the future
        // manually once with a no-op waker. `copy_to_clipboard`'s non-`web`
        // body has no `.await` point, so it resolves on the first poll.
        use std::future::Future;
        use std::pin::pin;
        use std::task::{Context, Poll, Waker};

        let mut cx = Context::from_waker(Waker::noop());
        let mut future = pin!(copy_to_clipboard("hello".to_string()));
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(result) => {
                assert!(!result, "non-web build must report failure, not success")
            }
            Poll::Pending => panic!("copy_to_clipboard's non-web body has no await point"),
        }
    }
}
