// SPDX-License-Identifier: MIT OR Apache-2.0

//! Owned headless runtime behavior for source-installed adico components.
//!
//! Each module here is a self-contained primitive, independently re-authored
//! against its own spec -- the relevant WAI-ARIA Authoring Practices Guide
//! pattern where one exists, or the closest applicable APG guidance plus its
//! consumers' actual needs otherwise (see `openspec/changes/
//! reauthor-primitives-from-independent-spec/design.md` for the full
//! decision). Every public module is a complete, tested surface, not a
//! facade over hidden support code: shared machinery (roving-tabindex
//! collection state, pointer/move-interaction tracking, scroll locking, the
//! floating-content positioner, the shared open/close layer stack, listbox
//! selection state, typeahead search) lives in its own public module
//! alongside the primitives that consume it. `dropdown_menu` re-exports
//! `menu`'s components directly; `context_menu` and `menubar` remain their
//! own independent implementations pending an evaluation of how much of
//! their content/item rendering can reuse `menu`'s.

#![forbid(unsafe_code)]

use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;
#[cfg(any(feature = "web", feature = "native"))]
use dioxus::prelude::{Asset, asset, manganis};
#[cfg(any(feature = "web", feature = "native"))]
use dioxus_document as document;

/// Lucide icon components, re-exported so copied registry source and
/// consumer applications depend on `adico-primitives` for icons rather than
/// adding `dioxus-icons` as a second, directly-installed crate.
pub use dioxus_icons::lucide as icons;

pub mod accordion;
pub mod alert_dialog;
pub mod aspect_ratio;
pub mod autocomplete;
pub mod avatar;
pub mod calendar;
pub mod checkbox;
pub mod checkbox_group;
pub mod collapsible;
pub mod color_picker;
pub mod combobox;
pub mod command;
pub mod context_menu;
pub mod date_picker;
pub mod dialog;
pub mod direction;
pub mod drag_and_drop_list;
pub mod dropdown_menu;
pub mod field;
pub mod fieldset;
pub mod form;
pub mod gesture;
pub mod hover_card;
pub mod label;
pub mod menu;
pub mod menubar;
pub mod message_scroller;
pub mod meter;
pub mod navigation_menu;
pub mod number_field;
pub mod otp_field;
pub mod popover;
pub mod preview_card;
pub mod progress;
pub mod radio_group;
pub mod scroll_area;
pub mod select;
pub mod separator;
pub mod slider;
pub mod switch;
pub mod tabs;
pub mod tag_group;
pub mod theme_mode;
pub mod time_picker;
pub mod toast;
pub mod toggle;
pub mod toggle_group;
pub mod toolbar;
pub mod tooltip;
pub mod typeahead;
pub mod virtual_list;

pub mod clipboard;
pub mod collection;
pub mod layer;
pub mod listbox;
pub mod move_interaction;
pub mod persisted_state;
pub mod pointer;
pub mod portal;
pub mod positioner;
pub mod scroll_lock;
mod segment;
pub mod selectable;
pub mod selection;
mod time;

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
/// layer is the topmost *open* registrant on the shared [`layer`] stack (also
/// used by [`use_outside_dismiss`]).
///
/// This is a returned handler rather than a bare "global" listener because a
/// hook cannot attach a native event listener to the caller's own JSX
/// element — wiring onto the caller's own root also gets nesting correctness
/// from ordinary DOM event bubbling for free (see below), which a
/// document-level listener would not. **Correction (2026-09-03):** this
/// comment previously also claimed the document-level-listener alternative
/// was chosen because that pattern "does not work" in this Dioxus web
/// runtime, citing a provenance record that does not exist in this
/// repository's history. See [`use_outside_dismiss`]'s doc comment for the
/// live-verification evidence that claim doesn't hold; this hook's own
/// design (a returned handler) stands on its own merits independent of that
/// retracted claim. Every current consumer (`dialog`, `popover`,
/// `alert_dialog`) already worked around the old (incorrectly diagnosed)
/// concern by hand-rolling the exact check this hook now centralizes.
///
/// Wiring this on each overlay's own (focusable) root gets nesting
/// correctness from ordinary DOM event bubbling: an inner overlay's
/// `stop_propagation` prevents an outer one from also reacting. The shared
/// layer stack's `is_topmost()` check is an additional guard for
/// compositions where two overlays are DOM siblings rather than
/// ancestor/descendant (for example if a future real DOM portal, see
/// `portal.rs`, moves overlay content out of its logical nesting).
///
/// `open` should be the *same* signal the caller uses to decide whether it is
/// currently showing its own content — typically an always-mounted overlay
/// root's own `open`/`is_open` state — so a closed-but-still-mounted overlay
/// (kept mounted for a close animation, or simply never conditionally
/// rendered by its consumer) never occupies a stack slot it isn't actually
/// using; see [`layer`]'s module doc comment for why this matters.
pub fn use_escape_key(
    open: impl Readable<Target = bool> + Copy + 'static,
    mut on_escape: impl FnMut() + Clone + 'static,
) -> impl FnMut(Event<KeyboardData>) + Clone {
    let layer = layer::use_layer(open);
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
/// **Correction (2026-09-03):** this doc comment previously claimed a
/// "known defect on `web`" — that this hook's long-lived `document::eval`
/// listener never registers, citing a provenance record
/// (`provenance/records/adico-primitives-wave3-overlays.json`) as evidence.
/// That record does not exist anywhere in this repository's git history —
/// the citation was never backed by a real file. Live-verified this session
/// via `dx serve` + real (non-synthetic) Chrome interaction, instrumenting
/// `Document.prototype.addEventListener` as a spy: the listener registers on
/// every mount (confirmed across repeated open/close cycles, i.e. the
/// "reopen" case the original claim specifically named), and outside-click
/// dismiss round-trips correctly for `popover`, `select`, `combobox`, and
/// `context_menu` in `apps/playground`, and for `popover` opened
/// sequentially after `dialog` in `examples/basic-spa` (the exact scenario
/// the prior claim's own reproduction described). One caveat found along the
/// way, not the original claim: a JS-dispatched *synthetic* `PointerEvent`
/// (`el.dispatchEvent(new PointerEvent(...))`, as opposed to a real/CDP
/// mouse click) did not reliably trigger dismissal in one trial — untested
/// further, and irrelevant to real user interaction, but worth knowing if a
/// future synthetic-event test reports a false negative here.
#[cfg(any(feature = "web", feature = "native"))]
pub fn use_outside_dismiss(
    id: impl Readable<Target = String> + Copy + 'static,
    on_dismiss: impl FnMut() + Clone + 'static,
) {
    // `use_layer_member`, not `use_layer`: this hook is typically called from a
    // separate component/scope than the same overlay's `use_escape_key` (e.g.
    // `DialogContent` here, `DialogRoot` there). Registering its own layer would
    // give this later-mounted scope a higher stack position than its own root,
    // permanently shadowing the root's `is_topmost()` check for Escape.
    let layer = layer::use_layer_member();
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
//
// `pub`, not `pub(crate)`: this resolves "now" in the local timezone of the
// device running the UI, with an explicit UTC fallback where that cannot be
// resolved (see the impl below) -- the same resolution every consumer of
// "now" in this ecosystem should share, not just this crate's own
// `date_picker` module. Playground code (a separate crate that only depends
// on `adico-primitives`) needs this for the same reason `date_picker.rs`
// already does.
pub trait LocalDateExt {
    /// Get the local date, falling back to the UTC date if the local offset
    /// cannot be resolved (see the impl below for when that happens).
    fn now_local_date() -> ::time::Date;

    /// Get the local time-of-day, falling back to the UTC time-of-day if the
    /// local offset cannot be resolved. Confirmed empirically (see
    /// `local_time_tests` below): the fallback is real on Linux/BSD native
    /// builds, but not exercised on native macOS/Windows or in the browser
    /// (`web` feature, resolved via `time/wasm-bindgen`), which both resolve
    /// a true local offset.
    fn now_local_time() -> ::time::Time;

    /// Get the local date and time-of-day together, with the same
    /// UTC-fallback behavior as [`now_local_date`](Self::now_local_date) and
    /// [`now_local_time`](Self::now_local_time). Resolves the offset once,
    /// so the date and time-of-day it returns are always from the same
    /// instant -- unlike calling `now_local_date()` and `now_local_time()`
    /// separately, which could observe a date rollover between the two calls.
    fn now_local_datetime() -> ::time::PrimitiveDateTime;
}

impl LocalDateExt for ::time::OffsetDateTime {
    fn now_local_date() -> ::time::Date {
        ::time::OffsetDateTime::now_local()
            .map(|x| x.date())
            .unwrap_or_else(|_| ::time::OffsetDateTime::now_utc().date())
    }

    fn now_local_time() -> ::time::Time {
        ::time::OffsetDateTime::now_local()
            .map(|x| x.time())
            .unwrap_or_else(|_| ::time::OffsetDateTime::now_utc().time())
    }

    fn now_local_datetime() -> ::time::PrimitiveDateTime {
        let now = ::time::OffsetDateTime::now_local()
            .unwrap_or_else(|_| ::time::OffsetDateTime::now_utc());
        ::time::PrimitiveDateTime::new(now.date(), now.time())
    }
}

#[cfg(test)]
mod local_time_tests {
    // Empirical finding (not assumed): probed live via `cargo test -p
    // adico-primitives` on this workspace's native macOS target, run under
    // `cargo test`'s default multithreaded test harness. `time`'s
    // `local-offset` feature is enabled (Cargo.toml), and on this platform
    // `OffsetDateTime::now_local()` resolves `Ok` even in a multithreaded
    // process -- `time` only refuses the OS offset (returning
    // `Err(IndeterminateOffset)`) on Unix-family targets without the
    // `unsound_local_offset` cfg; that restriction does not apply on this
    // target. So on native macOS (and, by the same `time`-internal target
    // gating, Windows), `now_local_date()`'s fallback branch is not
    // exercised in practice -- callers get the true local date, not a UTC
    // fallback. Linux/BSD native builds are the platform this fallback
    // exists for. The browser/wasm path resolves through `time/wasm-bindgen`
    // regardless (see the `web` Cargo feature) and is unaffected either way.
    #[test]
    fn now_local_resolves_on_this_native_target_rather_than_falling_back() {
        assert!(
            ::time::OffsetDateTime::now_local().is_ok(),
            "expected native macOS to resolve a real local offset; if this fails, \
             the platform gating documented above has changed and now_local_date()'s \
             UTC fallback is reachable here too"
        );
    }

    use super::LocalDateExt;

    #[test]
    fn now_local_time_and_datetime_agree_with_now_local_date() {
        // All three resolve independently on this target (confirmed above:
        // none hit the UTC-fallback branch here), so a datetime captured
        // around the same instant should have a consistent date component
        // and an hour-of-day matching a separately-resolved local time --
        // this would drift apart if either resolved through a different
        // offset (e.g. one silently falling back to UTC while the other
        // didn't).
        let date = ::time::OffsetDateTime::now_local_date();
        let time = ::time::OffsetDateTime::now_local_time();
        let datetime = ::time::OffsetDateTime::now_local_datetime();
        assert_eq!(datetime.date(), date);
        assert_eq!(datetime.time().hour(), time.hour());
    }
}
