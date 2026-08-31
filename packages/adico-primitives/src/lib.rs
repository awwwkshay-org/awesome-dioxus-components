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

use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;
#[cfg(any(feature = "web", feature = "native"))]
use dioxus::prelude::{Asset, asset, manganis};
#[cfg(any(feature = "web", feature = "native"))]
use dioxus_document as document;

pub use ::dioxus_core;

/// Lucide icon components, re-exported so copied registry source and
/// consumer applications depend on `adico-primitives` for icons rather than
/// adding `dioxus-icons` as a second, directly-installed crate.
pub use dioxus_icons::lucide as icons;

pub mod dialog;
pub mod select;

pub mod accordion;
pub mod alert_dialog;
pub mod aspect_ratio;
pub mod avatar;
pub mod calendar;
pub mod checkbox;
pub mod collapsible;
pub mod color_picker;
pub mod combobox;
pub mod context_menu;
pub mod date_picker;
pub mod direction;
pub mod drag_and_drop_list;
pub mod dropdown_menu;
pub mod hover_card;
pub mod label;
pub mod menubar;
pub mod popover;
pub mod progress;
pub mod radio_group;
pub mod scroll_area;
pub mod separator;
pub mod slider;
pub mod switch;
pub mod tabs;
pub mod tag_group;
pub mod theme_mode;
pub mod toast;
pub mod toggle;
pub mod toggle_group;
pub mod toolbar;
pub mod tooltip;
pub mod typeahead;
pub mod virtual_list;

pub mod collection;
pub mod layer;
pub mod listbox;
pub mod move_interaction;
pub mod pointer;
pub mod portal;
pub mod selectable;
pub mod selection;
mod time;
mod r#virtual;

#[cfg(any(feature = "web", feature = "native"))]
const FOCUS_TRAP_JS: Asset = asset!("/src/js/focus-trap.js");

/// Generate a runtime-unique identifier suitable for ARIA relationships.
pub fn use_unique_id() -> Signal<String> {
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    #[allow(unused_mut)]
    let mut initial_value = use_hook(|| {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        format!("adico-{id}")
    });

    use_signal(|| initial_value)
}

/// Resolve to `user_id` when set, falling back to `generated_id` otherwise.
///
/// Lets a component accept an optional caller-supplied `id` prop while still
/// generating one internally (via [`use_unique_id`]) for ARIA relationships.
pub fn use_id_or<T: Clone + PartialEq + 'static>(
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

/// A controlled-or-uncontrolled prop trio for primitive state, consumed by
/// [`use_controlled`]-style hooks such as [`selectable::use_selectable_root`].
#[derive(Clone, Copy)]
pub struct Controlled<T: Clone + PartialEq + 'static> {
    /// The externally controlled value, if the caller is controlling it.
    pub value: ReadSignal<Option<T>>,
    /// The initial value when uncontrolled.
    pub default: ReadSignal<T>,
    /// Called whenever the value changes, controlled or not.
    pub on_change: Callback<T>,
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

/// Call `on_escape` when Escape is pressed, but only for the topmost caller.
///
/// Callers are ordered on a LIFO stack keyed by their component scope (the
/// shared [`layer`] stack, also used by [`use_outside_dismiss`]), so when
/// several overlays are nested only the most-recently-mounted one reacts to a
/// given Escape press.
pub fn use_global_escape_listener(mut on_escape: impl FnMut() + Clone + 'static) {
    let layer = layer::use_layer();
    use_global_keydown_listener("Escape", move || {
        if layer.is_topmost() {
            on_escape();
        }
    });
}

#[cfg(any(feature = "web", feature = "native"))]
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

#[cfg(not(any(feature = "web", feature = "native")))]
fn use_global_keydown_listener(_key: &'static str, _on_keydown: impl FnMut() + Clone + 'static) {}

/// Call `on_dismiss` when a pointerdown or focus event lands outside the
/// element identified by `id`, but only for the topmost caller on the shared
/// [`layer`] stack (also used by [`use_global_escape_listener`]). A no-op on
/// targets without a DOM (SSR/native).
#[cfg(any(feature = "web", feature = "native"))]
pub fn use_outside_dismiss(
    id: impl Readable<Target = String> + Copy + 'static,
    on_dismiss: impl FnMut() + Clone + 'static,
) {
    let layer = layer::use_layer();
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
        let layer = layer.clone();
        spawn(async move {
            while let Ok(true) = eval.recv().await {
                if layer.is_topmost() {
                    on_dismiss();
                }
            }
        });
        move || {
            let _ = eval.send(true);
        }
    });
}

/// Call `on_dismiss` when a pointerdown or focus event lands outside the
/// element identified by `id`. A no-op on targets without a DOM (SSR/native).
#[cfg(not(any(feature = "web", feature = "native")))]
pub fn use_outside_dismiss(
    _id: impl Readable<Target = String> + Copy + 'static,
    _on_dismiss: impl FnMut() + Clone + 'static,
) {
}

/// Presence: keep content mounted (returning `true`) until any CSS animations
/// on the element identified by `id` finish, so a close transition can play
/// before the element is removed. Returns `open` unmodified on targets
/// without a DOM (SSR/native), where there is no animation to await.
#[cfg(any(feature = "web", feature = "native"))]
pub fn use_animated_open(
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

/// Presence: keep content mounted (returning `true`) until any CSS animations
/// on the element identified by `id` finish, so a close transition can play
/// before the element is removed. Returns `open` unmodified on targets
/// without a DOM (SSR/native), where there is no animation to await.
#[cfg(not(any(feature = "web", feature = "native")))]
pub fn use_animated_open(
    _id: impl Readable<Target = String> + Copy + 'static,
    open: impl Readable<Target = bool> + Copy + 'static,
) -> impl Fn() -> bool + Copy {
    move || open.cloned()
}

/// Trap keyboard focus inside the element identified by `id` while `is_modal`
/// and `open` are both true. Requires [`FocusTrapScript`] to be rendered
/// somewhere in the tree. A no-op on targets without a DOM (SSR/native).
#[cfg(any(feature = "web", feature = "native"))]
pub fn use_focus_trap(id: Memo<String>, open: Memo<bool>, is_modal: ReadSignal<bool>) {
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

/// Trap keyboard focus inside the element identified by `id` while `is_modal`
/// and `open` are both true. Requires [`FocusTrapScript`] to be rendered
/// somewhere in the tree. A no-op on targets without a DOM (SSR/native).
#[cfg(not(any(feature = "web", feature = "native")))]
pub fn use_focus_trap(_id: Memo<String>, _open: Memo<bool>, _is_modal: ReadSignal<bool>) {}

/// Loads the focus-trap browser script that [`use_focus_trap`] depends on.
/// Render this once, anywhere in the tree, alongside any component that uses
/// `use_focus_trap`. Renders nothing on targets without a DOM (SSR/native).
#[cfg(any(feature = "web", feature = "native"))]
#[component]
pub fn FocusTrapScript() -> Element {
    rsx! {
        document::Script {
            src: FOCUS_TRAP_JS,
            defer: true
        }
    }
}

/// Loads the focus-trap browser script that [`use_focus_trap`] depends on.
/// Render this once, anywhere in the tree, alongside any component that uses
/// `use_focus_trap`. Renders nothing on targets without a DOM (SSR/native).
#[cfg(not(any(feature = "web", feature = "native")))]
#[component]
pub fn FocusTrapScript() -> Element {
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
