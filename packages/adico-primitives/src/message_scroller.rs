// SPDX-License-Identifier: MIT OR Apache-2.0

// No WAI-ARIA APG pattern applies directly (a scrollable message viewport is
// not itself a distinct ARIA widget); this module's own spec is auto-anchor
// scrolling for append-only/append-and-prepend chat-style lists: while the
// viewport is scrolled within `bottom_threshold` pixels of its bottom edge,
// newly appended content keeps it pinned to the new bottom ("stuck to
// bottom"); once a user scrolls away from the bottom, new content no longer
// force-scrolls the viewport (respecting user scroll intent) until
// `use_message_scroller_scroll_to_bottom` is invoked (typically from a
// "jump to latest" button, gated on `use_message_scroller_pinned_to_bottom`
// being `false`).
//
// Following the fix already established in this crate's own `virtual_list.rs`
// (see its header comment): tracking uses native Dioxus `onscroll`
// (`ScrollData`) and `onmounted`/`MountedData` for scroll-position/size
// measurement, and native `onresize` (`ResizeData`, backed by a real
// `ResizeObserver`) on the content wrapper to detect appended content --
// never a `document::eval` JS bridge, which is the documented unreliable
// channel in this Dioxus 0.7.9/0.7.10 web runtime.
//
// Prepend anchoring (restoring visual reading position when older content is
// inserted *above* the current scroll position, e.g. a "load older messages"
// action) is deliberately NOT hand-rolled here with offset math. Modern
// browsers already preserve visual position for off-screen content insertion
// via native CSS scroll anchoring (`overflow-anchor: auto`, the default);
// this primitive only force-scrolls the viewport when `pinned_to_bottom` is
// true, so it never fights that native behavior. A consumer that sets
// `overflow-anchor: none` on the viewport opts out of that browser guarantee
// on their own and is responsible for any resulting jump.
//
// SSR-safe: `onmounted`/`onscroll`/`onresize` simply never fire without a
// live DOM, and every `MountedData` call is awaited behind a `spawn`, not
// synchronously during render -- initial render always succeeds with the
// signal defaults (`pinned_to_bottom: true`, all sizes `0.0`) whether or not
// a browser is present.

//! Defines the [`MessageScroller`] root and its [`MessageScrollerViewport`]/
//! [`MessageScrollerContent`] parts, plus the [`use_message_scroller_pinned_to_bottom`]/
//! [`use_message_scroller_scroll_to_bottom`] hooks a styled facade uses to
//! build its own "item" and "jump to latest" button parts.

use std::rc::Rc;

use dioxus::html::geometry::euclid::Vector2D;
use dioxus::prelude::*;

use crate::scroll_area::{
    provide_scroll_area_context, scroll_area_viewport_onmounted, scroll_area_viewport_onresize,
    scroll_area_viewport_onscroll, scroll_area_visibility_class, use_scroll_area_context,
};

/// Default distance (in pixels) from the true bottom edge within which the
/// viewport is still considered "pinned to bottom".
pub const DEFAULT_BOTTOM_THRESHOLD: f64 = 48.0;

#[derive(Clone, Copy)]
struct MessageScrollerContext {
    pinned_to_bottom: Signal<bool>,
    viewport_ref: Signal<Option<Rc<MountedData>>>,
    scroll_offset: Signal<f64>,
    viewport_size: Signal<f64>,
    content_size: Signal<f64>,
    bottom_threshold: f64,
    // Set for the duration of a programmatic `scroll_to_bottom` call. The
    // browser fires intermediate `scroll` events while an animated
    // (`ScrollBehavior::Smooth`) scroll is still in flight, each reporting a
    // `scroll_top` short of the final target; without this guard,
    // `MessageScrollerViewport`'s `onscroll` handler would read those
    // transient positions as "the user scrolled away from the bottom" and
    // permanently un-pin the viewport mid-animation, even though the scroll
    // was this primitive's own doing. `onscroll` still records raw
    // offset/size while this is set -- only the pinned recomputation is
    // skipped, since the final `.set(true)` in `scroll_to_bottom` itself is
    // authoritative once the animation completes.
    programmatic_scroll: Signal<bool>,
}

impl MessageScrollerContext {
    fn distance_from_bottom(&self) -> f64 {
        ((self.content_size)() - (self.scroll_offset)() - (self.viewport_size)()).max(0.0)
    }

    fn max_scroll_offset(&self) -> f64 {
        ((self.content_size)() - (self.viewport_size)()).max(0.0)
    }

    async fn scroll_to_bottom(mut self, behavior: ScrollBehavior) {
        let Some(handle) = (self.viewport_ref)() else {
            return;
        };
        let target = self.max_scroll_offset();
        self.programmatic_scroll.set(true);
        let result = handle.scroll(Vector2D::new(0.0, target), behavior).await;
        if result.is_ok() {
            self.scroll_offset.set(target);
            self.pinned_to_bottom.set(true);
        }
        self.programmatic_scroll.set(false);
    }
}

/// The props for the [`MessageScroller`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MessageScrollerProps {
    /// Distance (in pixels) from the true bottom edge within which the
    /// viewport still counts as "pinned to bottom".
    #[props(default = DEFAULT_BOTTOM_THRESHOLD)]
    pub bottom_threshold: f64,

    /// Additional attributes to apply to the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the message scroller root -- typically one
    /// [`MessageScrollerViewport`], plus any "jump to latest" control built
    /// from [`use_message_scroller_pinned_to_bottom`]/
    /// [`use_message_scroller_scroll_to_bottom`].
    pub children: Element,
}

/// # MessageScroller
///
/// Provides scroll-anchoring context to a descendant [`MessageScrollerViewport`]/
/// [`MessageScrollerContent`] pair.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::message_scroller::*;
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         MessageScroller {
///             MessageScrollerViewport {
///                 style: "height: 20rem; overflow-y: auto;",
///                 MessageScrollerContent {
///                     "message 1"
///                     "message 2"
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn MessageScroller(props: MessageScrollerProps) -> Element {
    let ctx = MessageScrollerContext {
        pinned_to_bottom: use_signal(|| true),
        viewport_ref: use_signal(|| None),
        scroll_offset: use_signal(|| 0.0),
        viewport_size: use_signal(|| 0.0),
        content_size: use_signal(|| 0.0),
        bottom_threshold: props.bottom_threshold,
        programmatic_scroll: use_signal(|| false),
    };
    use_context_provider(|| ctx);
    // Provided unconditionally (harmless, unused if no descendant reads it) so a
    // consumer opting `MessageScrollerViewport` into the shared scroll-area contract
    // (`with_scroll_area: true`, see that component's doc comment) has a
    // `ScrollAreaContext` available, and can render `ScrollAreaScrollbar` as a sibling
    // of the viewport within this root's own children -- the in-place adoption shape,
    // not a new wrapper element this primitive would otherwise need to introduce.
    provide_scroll_area_context();

    rsx! {
        div { ..props.attributes, {props.children} }
    }
}

/// The props for the [`MessageScrollerViewport`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MessageScrollerViewportProps {
    /// Opts into the shared scroll-area contract (themed overlay scrollbar support,
    /// see `adico_primitives::scroll_area`): merges its `onmounted`/`onscroll`/
    /// `onresize` tracking into this element's own, and merges `class` with the
    /// internal visibility class instead of routing it through `attributes`. Defaults
    /// to `false`, preserving this primitive's original fully headless behavior --
    /// this is deliberately opt-in, not automatic, so a consumer of
    /// `adico-primitives` directly (not through this ecosystem's own themed registry
    /// facade) never has scroll-area styling opinions forced on them.
    #[props(default)]
    pub with_scroll_area: bool,

    /// Extra classes, merged with the internal visibility class when
    /// `with_scroll_area` is `true`. Has no effect when `with_scroll_area` is `false`
    /// (pass a class through `attributes` instead, as before).
    #[props(default)]
    pub class: Option<String>,

    /// Additional attributes to apply to the viewport element. Callers own
    /// the scroll container's `overflow`/height styling (this primitive is
    /// headless).
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the viewport -- typically one [`MessageScrollerContent`].
    pub children: Element,
}

/// The real scrollable element. Measures and tracks its own scroll position,
/// viewport size, and content size via native `onmounted`/`onscroll`.
#[component]
pub fn MessageScrollerViewport(props: MessageScrollerViewportProps) -> Element {
    let mut ctx: MessageScrollerContext = use_context();

    let mut onmounted = move |event: Event<MountedData>| {
        let handle = event.data();
        ctx.viewport_ref.set(Some(handle.clone()));
        spawn(async move {
            if let Ok(rect) = handle.get_client_rect().await {
                ctx.viewport_size.set(rect.height());
            }
            if let Ok(size) = handle.get_scroll_size().await {
                ctx.content_size.set(size.height);
            }
        });
    };

    let mut onscroll = move |event: Event<ScrollData>| {
        let data = event.data();
        ctx.scroll_offset.set(data.scroll_top().max(0.0));
        if data.client_height() > 0 {
            ctx.viewport_size.set(data.client_height() as f64);
        }
        if data.scroll_height() > 0 {
            ctx.content_size.set(data.scroll_height() as f64);
        }
        if !(ctx.programmatic_scroll)() {
            let pinned = ctx.distance_from_bottom() <= ctx.bottom_threshold;
            ctx.pinned_to_bottom.set(pinned);
        }
    };

    if !props.with_scroll_area {
        return rsx! {
            div { onmounted, onscroll, ..props.attributes, {props.children} }
        };
    }

    // In-place adoption shape (design.md D2): this element is already instrumented
    // (the `onmounted`/`onscroll` above), so the scroll-area contract is merged onto
    // it directly rather than wrapping it in a new `ScrollAreaViewport` node, which
    // would duplicate scroll-position tracking across two competing sources of truth.
    let scroll_area_ctx = use_scroll_area_context();
    let mut scroll_area_onmounted = scroll_area_viewport_onmounted(scroll_area_ctx);
    let mut scroll_area_onscroll = scroll_area_viewport_onscroll(scroll_area_ctx);
    let mut scroll_area_onresize = scroll_area_viewport_onresize(scroll_area_ctx);

    let merged_onmounted = move |event: Event<MountedData>| {
        onmounted(event.clone());
        scroll_area_onmounted(event);
    };
    let merged_onscroll = move |event: Event<ScrollData>| {
        onscroll(event.clone());
        scroll_area_onscroll(event);
    };
    let onresize = move |event: Event<ResizeData>| {
        scroll_area_onresize(event);
    };

    let visibility_class = scroll_area_visibility_class(false);
    let merged_class = match props.class.as_deref() {
        Some(extra) if !extra.is_empty() => format!("{visibility_class} {extra}"),
        _ => visibility_class.to_string(),
    };

    rsx! {
        div {
            class: "{merged_class}",
            style: "scrollbar-width: none;",
            onmounted: merged_onmounted,
            onscroll: merged_onscroll,
            onresize,
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`MessageScrollerContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MessageScrollerContentProps {
    /// Additional attributes to apply to the content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The rendered messages.
    pub children: Element,
}

/// The measured content wrapper. Watches its own `onresize` (a real
/// `ResizeObserver` under Dioxus's web renderer) and, if it grew while the
/// viewport was pinned to bottom, scrolls the viewport along with it.
#[component]
pub fn MessageScrollerContent(props: MessageScrollerContentProps) -> Element {
    let mut ctx: MessageScrollerContext = use_context();

    let onresize = move |event: Event<ResizeData>| {
        let rect = event.data().get_content_box_size().unwrap_or_default();
        let new_height = rect.height.max(0.0);
        let grew = new_height > (ctx.content_size)();
        ctx.content_size.set(new_height);
        if grew && (ctx.pinned_to_bottom)() {
            spawn(ctx.scroll_to_bottom(ScrollBehavior::Instant));
        }
    };

    rsx! {
        div { onresize, ..props.attributes, {props.children} }
    }
}

/// Whether the viewport is currently pinned to its bottom edge. A styled
/// facade uses this to conditionally render a "jump to latest" button
/// (typically only when this is `false`).
///
/// Must be called from a descendant of [`MessageScroller`] (panics
/// otherwise, the same contract every other primitive's root/part pairing in
/// this crate already follows -- see `MessageScroller`'s own doc example,
/// which nests its "jump to latest" control alongside
/// [`MessageScrollerViewport`], not outside it).
pub fn use_message_scroller_pinned_to_bottom() -> Memo<bool> {
    let ctx: MessageScrollerContext = use_context();
    use_memo(move || (ctx.pinned_to_bottom)())
}

/// Scrolls the viewport to its bottom edge and re-pins it. Wire this to a
/// "jump to latest" button's `on_click`/`on_select`.
///
/// Must be called from a descendant of [`MessageScroller`] (see
/// [`use_message_scroller_pinned_to_bottom`]'s doc comment).
pub fn use_message_scroller_scroll_to_bottom() -> Callback<()> {
    let ctx: MessageScrollerContext = use_context();
    use_callback(move |()| {
        spawn(ctx.scroll_to_bottom(ScrollBehavior::Smooth));
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(root: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(root);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[component]
    fn Basic() -> Element {
        rsx! {
            MessageScroller {
                MessageScrollerViewport {
                    MessageScrollerContent { "hello" }
                }
            }
        }
    }

    #[test]
    fn renders_without_a_browser() {
        let html = render(Basic);
        assert!(html.contains("hello"));
    }

    #[component]
    fn WithJumpButton() -> Element {
        rsx! {
            MessageScroller {
                MessageScrollerViewport {
                    MessageScrollerContent { "hello" }
                }
                JumpButton {}
            }
        }
    }

    #[component]
    fn JumpButton() -> Element {
        let pinned = use_message_scroller_pinned_to_bottom();
        let scroll_to_bottom = use_message_scroller_scroll_to_bottom();
        rsx! {
            button {
                onclick: move |_| scroll_to_bottom(()),
                "pinned={pinned()}"
            }
        }
    }

    #[test]
    fn pinned_to_bottom_defaults_to_true_before_any_measurement() {
        let html = render(WithJumpButton);
        assert!(html.contains("pinned=true"));
    }
}
