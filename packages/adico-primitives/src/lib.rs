// SPDX-License-Identifier: MIT OR Apache-2.0
// Derived from DioxusLabs/dioxus-components at bf007c15d0cf4d04d3181cc46cf12325aa773955.
// Upstream path: primitives/src/lib.rs. See provenance/records/adico-primitives-dialog-select.json.

//! Owned headless runtime behavior for source-installed adico components.
//!
//! The initial Dialog and Select implementation is a provenance-tracked fork
//! of Dioxus Components. Public modules are intentionally small facades; the
//! inherited support modules remain private until their behavior is covered.

#![forbid(unsafe_code)]
#![allow(
    dead_code,
    reason = "the source-preserving initial fork retains private upstream support APIs until focused coverage is added"
)]
#![allow(
    clippy::collapsible_if,
    reason = "the initial fork preserves upstream control flow until focused behavior coverage permits refactoring"
)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::core::{current_scope_id, use_drop};
use dioxus::prelude::*;
#[cfg(any(feature = "web", feature = "desktop"))]
use dioxus::prelude::{Asset, asset, manganis};
#[cfg(any(feature = "web", feature = "desktop"))]
use dioxus_document as document;

pub use ::dioxus_core;

pub mod dialog;
pub mod select;

pub mod calendar;
pub mod combobox;
pub mod context_menu;
pub mod date_picker;
pub mod dropdown_menu;
pub mod hover_card;
pub mod menubar;
pub mod popover;
pub mod separator;
pub mod tooltip;

mod collection;
mod listbox;
mod selectable;
mod selection;
mod time;

#[cfg(any(feature = "web", feature = "desktop"))]
const FOCUS_TRAP_JS: Asset = asset!("/src/js/focus-trap.js");

/// Generate a runtime-unique identifier suitable for ARIA relationships.
fn use_unique_id() -> Signal<String> {
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    #[allow(unused_mut)]
    let mut initial_value = use_hook(|| {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        format!("adico-{id}")
    });

    use_signal(|| initial_value)
}

fn use_id_or<T: Clone + PartialEq + 'static>(
    mut generated_id: Signal<T>,
    user_id: ReadSignal<Option<T>>,
) -> Memo<T> {
    let has_user_id = use_memo(move || user_id().is_some());
    use_effect(move || {
        if let Some(id) = user_id() {
            generated_id.set(id);
        }
    });
    use_memo(move || {
        if has_user_id() {
            user_id().expect("user ID was present when memo was created")
        } else {
            generated_id.peek().clone()
        }
    })
}

/// A controlled-or-uncontrolled prop trio for internal primitive state.
#[derive(Clone, Copy)]
pub(crate) struct Controlled<T: Clone + PartialEq + 'static> {
    pub(crate) value: ReadSignal<Option<T>>,
    pub(crate) default: ReadSignal<T>,
    pub(crate) on_change: Callback<T>,
}

/// Make a signal controllable by an optional external value.
pub fn use_controlled<T: Clone + PartialEq + 'static>(
    prop: ReadSignal<Option<T>>,
    default: T,
    on_change: Callback<T>,
) -> (Memo<T>, Callback<T>) {
    let mut internal_value = use_signal(|| prop.cloned().unwrap_or(default));
    let value = use_memo(move || prop.cloned().unwrap_or_else(&*internal_value));
    let set_value = use_callback(move |value: T| {
        internal_value.set(value.clone());
        on_change.call(value);
    });
    (value, set_value)
}

fn use_effect_cleanup<F: FnOnce() + 'static>(#[allow(unused)] cleanup: F) {
    dioxus_core::use_drop(cleanup);
}

fn use_effect_with_cleanup<F: FnMut() -> C + 'static, C: FnOnce() + 'static>(mut effect: F) {
    let mut cleanup = use_hook(|| CopyValue::new(None as Option<C>));
    use_effect(move || {
        if let Some(cleanup) = cleanup.take() {
            cleanup();
        }
        cleanup.set(Some(effect()));
    });
    dioxus_core::use_drop(move || {
        if let Some(cleanup) = cleanup.take() {
            cleanup();
        }
    });
}

#[derive(Clone)]
struct EscapeListenerStack(Rc<RefCell<Vec<ScopeId>>>);

fn use_global_escape_listener(mut on_escape: impl FnMut() + Clone + 'static) {
    let scope_id = current_scope_id();
    let stack = use_hook(move || {
        let stack: EscapeListenerStack = try_consume_context()
            .unwrap_or_else(|| provide_context(EscapeListenerStack(Default::default())));
        stack.0.borrow_mut().push(scope_id);
        stack
    });
    use_drop({
        let stack = stack.clone();
        move || stack.0.borrow_mut().retain(|id| *id != scope_id)
    });
    use_global_keydown_listener("Escape", move || {
        if stack.0.borrow().last() == Some(&scope_id) {
            on_escape();
        }
    });
}

#[cfg(any(feature = "web", feature = "desktop"))]
fn use_global_keydown_listener(key: &'static str, on_keydown: impl FnMut() + Clone + 'static) {
    use_effect_with_cleanup(move || {
        let mut eval = document::eval(
            "let targetKey = await dioxus.recv();
            function listener(event) {
                if (event.key === targetKey) {
                    event.preventDefault();
                    dioxus.send(true);
                }
            }
            document.addEventListener('keydown', listener);
            await dioxus.recv();
            document.removeEventListener('keydown', listener);",
        );
        let _ = eval.send(key);
        let mut on_keydown = on_keydown.clone();
        spawn(async move {
            while let Ok(true) = eval.recv().await {
                on_keydown();
            }
        });
        move || {
            let _ = eval.send(true);
        }
    });
}

#[cfg(not(any(feature = "web", feature = "desktop")))]
fn use_global_keydown_listener(_key: &'static str, _on_keydown: impl FnMut() + Clone + 'static) {}

#[cfg(any(feature = "web", feature = "desktop"))]
fn use_outside_dismiss(
    id: impl Readable<Target = String> + Copy + 'static,
    on_dismiss: impl FnMut() + Clone + 'static,
) {
    use_effect_with_cleanup(move || {
        let mut eval = document::eval(
            "const id = await dioxus.recv();
            const dismiss = event => {
                const root = document.getElementById(id);
                if (root && !root.contains(event.target)) dioxus.send(true);
            };
            document.addEventListener('pointerdown', dismiss, true);
            document.addEventListener('focusin', dismiss, true);
            await dioxus.recv();
            document.removeEventListener('pointerdown', dismiss, true);
            document.removeEventListener('focusin', dismiss, true);",
        );
        let _ = eval.send(id.cloned());
        let mut on_dismiss = on_dismiss.clone();
        spawn(async move {
            while let Ok(true) = eval.recv().await {
                on_dismiss();
            }
        });
        move || {
            let _ = eval.send(true);
        }
    });
}

#[cfg(not(any(feature = "web", feature = "desktop")))]
fn use_outside_dismiss(
    _id: impl Readable<Target = String> + Copy + 'static,
    _on_dismiss: impl FnMut() + Clone + 'static,
) {
}

#[cfg(any(feature = "web", feature = "desktop"))]
fn use_animated_open(
    id: impl Readable<Target = String> + Copy + 'static,
    open: impl Readable<Target = bool> + Copy + 'static,
) -> impl Fn() -> bool + Copy {
    let animating = use_signal(|| false);
    let mut show_in_dom = use_signal(|| false);
    use_effect(move || {
        let is_open = open.cloned();
        if is_open {
            show_in_dom.set(true);
        } else {
            spawn(async move {
                let mut eval = document::eval(
                    "const id = await dioxus.recv();
                    const element = document.getElementById(id);
                    if (element && element.getAnimations().length > 0) {
                        Promise.all(element.getAnimations().map(animation => animation.finished)).then(() => dioxus.send(true));
                    } else { dioxus.send(true); }",
                );
                let _ = eval.send(id.cloned());
                _ = eval.recv::<bool>().await;
                // The close-animation task from the initial closed render can
                // complete after a trigger has already reopened the layer.
                // Never let that stale task remove currently-open content.
                if !open.cloned() {
                    show_in_dom.set(false);
                }
            });
        }
    });
    move || show_in_dom() || animating()
}

#[cfg(not(any(feature = "web", feature = "desktop")))]
fn use_animated_open(
    _id: impl Readable<Target = String> + Copy + 'static,
    open: impl Readable<Target = bool> + Copy + 'static,
) -> impl Fn() -> bool + Copy {
    move || open.cloned()
}

#[cfg(any(feature = "web", feature = "desktop"))]
pub(crate) fn use_focus_trap(id: Memo<String>, open: Memo<bool>, is_modal: ReadSignal<bool>) {
    use_effect(move || {
        if !is_modal() {
            return;
        }
        let eval = document::eval(
            r#"let id = await dioxus.recv();
            let is_open = await dioxus.recv();
            let dialog = document.getElementById(id);

            if (is_open) {
                dialog.trap = window.createFocusTrap(dialog);
            }
            if (!is_open && dialog.trap) {
                dialog.trap.remove();
                dialog.trap = null;
            }"#,
        );
        let _ = eval.send(id.to_string());
        let _ = eval.send(open.cloned());
    });
}

#[cfg(not(any(feature = "web", feature = "desktop")))]
pub(crate) fn use_focus_trap(_id: Memo<String>, _open: Memo<bool>, _is_modal: ReadSignal<bool>) {}

#[cfg(any(feature = "web", feature = "desktop"))]
#[component]
pub(crate) fn FocusTrapScript() -> Element {
    rsx! {
        document::Script {
            src: FOCUS_TRAP_JS,
            defer: true
        }
    }
}

#[cfg(not(any(feature = "web", feature = "desktop")))]
#[component]
pub(crate) fn FocusTrapScript() -> Element {
    rsx! {}
}

/// The side where overlay content will be displayed relative to its trigger.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentSide {
    /// The content will appear above the trigger.
    Top,
    /// The content will appear to the right of the trigger.
    Right,
    /// The content will appear below the trigger.
    Bottom,
    /// The content will appear to the left of the trigger.
    Left,
}

impl ContentSide {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Right => "right",
            Self::Bottom => "bottom",
            Self::Left => "left",
        }
    }
}

/// The alignment of overlay content relative to its trigger.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentAlign {
    /// The content will be aligned to the start of the trigger.
    Start,
    /// The content will be centered relative to the trigger.
    Center,
    /// The content will be aligned to the end of the trigger.
    End,
}

impl ContentAlign {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
}

// `::time::` (crate-root-relative) disambiguates the external `time` crate
// from this crate's own local `time` module (target-aware sleep support).
pub(crate) trait LocalDateExt {
    /// A small extension method function to get the local date with a fallback to UTC date if this fails
    fn now_local_date() -> ::time::Date;
}

impl LocalDateExt for ::time::OffsetDateTime {
    fn now_local_date() -> ::time::Date {
        ::time::OffsetDateTime::now_local()
            .map(|x| x.date())
            .unwrap_or_else(|_| ::time::OffsetDateTime::now_utc().date())
    }
}
