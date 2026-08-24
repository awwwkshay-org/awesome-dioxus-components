//! Minimal target-aware timing support for owned primitives.
//!
//! This deliberately avoids depending on a helper crate that enables Dioxus'
//! default feature set, keeping platform/runtime selection in the consumer app.

use std::time::Duration;

#[cfg(target_family = "wasm")]
pub(crate) async fn sleep(duration: Duration) {
    gloo_timers::future::sleep(duration).await;
}

#[cfg(not(target_family = "wasm"))]
pub(crate) async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}
