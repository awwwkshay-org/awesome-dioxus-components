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
pub mod gesture;
pub mod hover_card;
pub mod label;
pub mod menu;
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
pub mod positioner;
pub mod scroll_lock;
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

/// Returns a keydown handler for the caller to wire onto their own root
/// element's `onkeydown`, calling `on_escape` when Escape is pressed and this
/// layer is the topmost registrant on the shared [`layer`] stack (also used
/// by [`use_outside_dismiss`]).
///
/// This is a returned handler rather than a bare "global" listener because a
/// hook cannot attach a native event listener to the caller's own JSX
/// element, and the alternative — a document-level `document::eval` listener
/// registered once and left running — does not work on this crate's primary
/// target: real browser and Playwright testing confirmed that pattern's
/// long-lived, repeatedly-firing `document.addEventListener` call never
/// actually registers in this Dioxus 0.7.9/0.7.10 web runtime (see
/// `provenance/records/adico-primitives-wave3-overlays.json`). Every current
/// consumer (`dialog`, `popover`, `alert_dialog`) already worked around this
/// by hand-rolling the exact check this hook now centralizes.
///
/// Wiring this on each overlay's own (focusable) root gets nesting
/// correctness from ordinary DOM event bubbling: an inner overlay's
/// `stop_propagation` prevents an outer one from also reacting. The shared
/// layer stack's `is_topmost()` check is an additional guard for
/// compositions where two overlays are DOM siblings rather than
/// ancestor/descendant (for example if a future real DOM portal, see
/// `portal.rs`, moves overlay content out of its logical nesting).
pub fn use_escape_key(
    mut on_escape: impl FnMut() + Clone + 'static,
) -> impl FnMut(Event<KeyboardData>) + Clone {
    let layer = layer::use_layer();
    move |event: Event<KeyboardData>| {
        if event.key() == Key::Escape && layer.is_topmost() {
            on_escape();
            event.prevent_default();
            event.stop_propagation();
        }
    }
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
/// [`layer`] stack (also used by [`use_escape_key`]). A no-op on targets
/// without a DOM (SSR/native).
///
/// **Known defect on `web`:** this hook uses the same long-lived,
/// repeatedly-firing `document::eval` listener pattern documented as
/// non-functional in this Dioxus 0.7.9/0.7.10 web runtime (see
/// `provenance/records/adico-primitives-wave3-overlays.json`) — unlike
/// [`use_escape_key`], there is no equivalent fix available as a bare hook,
/// since there is no native Dioxus document-level pointer event to return a
/// handler for. `context_menu` and `popover`, the two current real
/// consumers, do not currently have a working outside-dismiss on `web`. A
/// real fix needs a composition-level change: an invisible full-viewport
/// backdrop element behind the popup content, with a native `onclick`/
/// `onpointerdown` handler instead of this hook — the same technique
/// `dialog`'s registry facade already uses for its own outside-dismiss. That
/// is 7.8 migration scope, not a primitive-only fix.
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

/// Manage keyboard focus for the element identified by `id` while `open` is
/// true. Requires [`FocusTrapScript`] to be rendered somewhere in the tree. A
/// no-op on targets without a DOM (SSR/native).
///
/// When `is_modal` is true, this is a full trap: Tab cycles only among the
/// container's focusable descendants (recognizing any element with an
/// explicit `tabindex`, not just natively-focusable tags), backed by focus
/// guards so focus can't otherwise escape, and closing restores focus to
/// whatever was focused before it opened. When `is_modal` is false, this is a
/// non-modal focus scope: Tab is never trapped and the user can freely leave
/// the container, but closing still restores focus the same way — a modal
/// dialog and a non-modal popover both give that courtesy, only the former
/// also contains Tab.
#[cfg(any(feature = "web", feature = "native"))]
pub fn use_focus_trap(id: Memo<String>, open: Memo<bool>, is_modal: ReadSignal<bool>) {
    use_effect(move || {
        let eval = document::eval(
            r#"let id = await dioxus.recv();
            let is_open = await dioxus.recv();
            let is_modal = await dioxus.recv();
            let container = document.getElementById(id);

            if (is_open && !container.trap) {
                container.trap = is_modal
                    ? window.createFocusTrap(container)
                    : window.createFocusScope();
            }
            if (!is_open && container.trap) {
                container.trap.remove();
                container.trap = null;
            }"#,
        );
        let _ = eval.send(id.to_string());
        let _ = eval.send(open.cloned());
        let _ = eval.send(is_modal.cloned());
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

    /// The side directly across from this one, used by [`crate::positioner`]
    /// to flip placement when the preferred side has no room.
    pub(crate) fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
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
