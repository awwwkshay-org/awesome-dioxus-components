// No ARIA role applies (a scrollable container is not itself a widget); its spec is pure CSS
// overflow/scrollbar-width composition from its own `ScrollDirection`/`ScrollType` enums, one
// axis each, matching the 3x3 combinations `ScrollAreaViewport` itself already exhaustively
// matches on.
//
// `class` and `style` are declared as explicit merging props (not routed through the generic
// `..attributes` spread) because a caller-supplied value collides with this component's own
// internal class/style otherwise, and that collision resolves *oppositely* on SSR (keeps the
// first-written attribute) versus CSR (keeps the last-written attribute) -- verified against the
// vendored `dioxus-ssr`/`dioxus-interpreter-js` 0.7.9 source. `overflow-x`/`overflow-y`/
// `scrollbar-width` are assembled into one manually-built `style` string rather than using
// dioxus-html's typed per-property style attributes (`overflow_x`, `overflow_y`, ...), because
// dioxus-html declares no typed `scrollbar_width` attribute at all -- it would otherwise
// serialize as an inert plain HTML attribute instead of a CSS declaration.
//
// Overlay scrollbars (`ScrollAreaScrollbar`/`ScrollAreaThumb`/`ScrollAreaCorner`) are decorative
// siblings of the real scrolling element (`ScrollAreaViewport`), never a replacement for it: the
// viewport stays a genuine native-`overflow` element receiving real browser scroll events, and no
// part of this module uses `transform` to implement scrolling. Two independent reasons this
// matters, both load-bearing for other primitives: `positioner.rs`'s capture-phase
// `document.addEventListener('scroll', ..., true)` listener (keeping every anchored popover glued
// to its anchor) only fires for genuine native scroll events, and `message_scroller.rs`'s
// prepend-anchoring relies entirely on native CSS scroll anchoring (`overflow-anchor: auto`),
// which only holds for a real overflow element. Thumb dragging follows this crate's own
// established pointer-drag convention (see `registry/ui/resizable.rs`'s module doc comment):
// native per-element `onpointerdown`/`onpointermove`/`onpointerup`, no `pointer.rs`/`gesture.rs`
// global registry (documented elsewhere in this crate as unreliable), and a temporary
// `fixed inset-0` transparent overlay to catch pointer movement during a drag since Dioxus's
// `MountedData` exposes no `setPointerCapture`.
//
// `ScrollAreaContext` is deliberately `pub` with a `provide_scroll_area_context()` constructor,
// not private and `ScrollArea`-only: a component whose own element already carries an ARIA role
// or existing scroll instrumentation (e.g. `SelectList`, `CommandList`, `MessageScrollerViewport`)
// cannot be wrapped in a new `ScrollAreaViewport` node without breaking that structure, so it
// instead calls `provide_scroll_area_context()` itself, attaches `scroll_area_viewport_onmounted`/
// `scroll_area_viewport_onscroll` to its own existing element, and renders `ScrollAreaScrollbar`
// as a new sibling -- the same contract `ScrollArea`'s own wrapper form gets, applied in place.

//! Defines the [`ScrollArea`] component (and its [`ScrollAreaViewport`]/
//! [`ScrollAreaScrollbar`]/[`ScrollAreaThumb`]/[`ScrollAreaCorner`] parts) for
//! creating scrollable areas with themed, overlay scrollbars.

use std::rc::Rc;

use dioxus::html::geometry::euclid::Vector2D;
use dioxus::prelude::*;

/// Minimum thumb length in pixels, so a thumb never shrinks to an
/// unclickable sliver against a very long content run.
const MIN_THUMB_PX: f64 = 20.0;

/// The direction in which scrolling is allowed.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ScrollDirection {
    /// Allow vertical scrolling only.
    Vertical,
    /// Allow horizontal scrolling only.
    Horizontal,
    /// Allow scrolling in both directions.
    #[default]
    Both,
}

/// The type of scrolling behavior.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ScrollType {
    /// Browser default scrolling.
    #[default]
    Auto,
    /// Always show scrollbars.
    Always,
    /// Hide scrollbars but enable scrolling.
    Hidden,
}

/// How the axis not covered by a single-axis [`ScrollDirection`] behaves.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum CrossAxisOverflow {
    /// Clip the cross axis (`overflow: hidden`). Matches this component's original,
    /// unconditional behavior.
    #[default]
    Clip,
    /// Leave the cross axis at the browser default (`overflow: visible`).
    Visible,
}

/// Which edge a [`ScrollAreaScrollbar`]/[`ScrollAreaThumb`] pair tracks.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarOrientation {
    /// The right-edge, vertical scrollbar.
    Vertical,
    /// The bottom-edge, horizontal scrollbar.
    Horizontal,
}

impl ScrollbarOrientation {
    fn data_attr(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }
}

#[derive(Clone, Copy)]
struct ThumbDrag {
    orientation: ScrollbarOrientation,
    start_client: f64,
    start_scroll: f64,
}

/// Shared scroll-position/size state, read by [`ScrollAreaScrollbar`]/[`ScrollAreaThumb`]/
/// [`ScrollAreaCorner`] and written by [`scroll_area_viewport_onmounted`]/
/// [`scroll_area_viewport_onscroll`]. Provide one with [`provide_scroll_area_context`]
/// above whichever element is the real scrolling element.
#[derive(Clone, Copy)]
pub struct ScrollAreaContext {
    viewport_ref: Signal<Option<Rc<MountedData>>>,
    scroll_top: Signal<f64>,
    scroll_left: Signal<f64>,
    viewport_height: Signal<f64>,
    viewport_width: Signal<f64>,
    content_height: Signal<f64>,
    content_width: Signal<f64>,
    /// Becomes `true` after the first successful client-side measurement. `false` for
    /// the entire SSR render (no DOM to measure), which is exactly what keeps
    /// [`ScrollAreaScrollbar`]/[`ScrollAreaCorner`] unrendered until real geometry is
    /// known -- no guessed-size thumb ever flashes on hydration.
    measured: Signal<bool>,
    drag: Signal<Option<ThumbDrag>>,
}

impl ScrollAreaContext {
    fn new() -> Self {
        Self {
            viewport_ref: Signal::new(None),
            scroll_top: Signal::new(0.0),
            scroll_left: Signal::new(0.0),
            viewport_height: Signal::new(0.0),
            viewport_width: Signal::new(0.0),
            content_height: Signal::new(0.0),
            content_width: Signal::new(0.0),
            measured: Signal::new(false),
            drag: Signal::new(None),
        }
    }

    fn track_size(&self, orientation: ScrollbarOrientation) -> f64 {
        match orientation {
            ScrollbarOrientation::Vertical => (self.viewport_height)(),
            ScrollbarOrientation::Horizontal => (self.viewport_width)(),
        }
    }

    fn content_size(&self, orientation: ScrollbarOrientation) -> f64 {
        match orientation {
            ScrollbarOrientation::Vertical => (self.content_height)(),
            ScrollbarOrientation::Horizontal => (self.content_width)(),
        }
    }

    fn scroll_offset(&self, orientation: ScrollbarOrientation) -> f64 {
        match orientation {
            ScrollbarOrientation::Vertical => (self.scroll_top)(),
            ScrollbarOrientation::Horizontal => (self.scroll_left)(),
        }
    }

    fn max_scroll(&self, orientation: ScrollbarOrientation) -> f64 {
        (self.content_size(orientation) - self.track_size(orientation)).max(0.0)
    }

    /// Whether this axis has enough overflow to need a scrollbar at all. `false` before
    /// the first measurement lands, so nothing renders during SSR or before hydration
    /// measures real geometry.
    fn needs_scrollbar(&self, orientation: ScrollbarOrientation) -> bool {
        (self.measured)() && self.max_scroll(orientation) > 0.5
    }

    fn thumb_size(&self, orientation: ScrollbarOrientation) -> f64 {
        let track = self.track_size(orientation);
        let content = self.content_size(orientation);
        if track <= 0.0 || content <= 0.0 {
            return 0.0;
        }
        (track * (track / content))
            .clamp(0.0, track)
            .max(MIN_THUMB_PX)
            .min(track)
    }

    fn thumb_offset(&self, orientation: ScrollbarOrientation) -> f64 {
        let track = self.track_size(orientation);
        let thumb = self.thumb_size(orientation);
        let max_scroll = self.max_scroll(orientation);
        if max_scroll <= 0.0 || track <= thumb {
            return 0.0;
        }
        let ratio = (self.scroll_offset(orientation) / max_scroll).clamp(0.0, 1.0);
        ratio * (track - thumb)
    }

    async fn scroll_to(self, top: f64, left: f64, behavior: ScrollBehavior) {
        let Some(handle) = (self.viewport_ref)() else {
            return;
        };
        let _ = handle.scroll(Vector2D::new(left, top), behavior).await;
    }
}

/// Provides a fresh [`ScrollAreaContext`] to descendants. [`ScrollArea`] calls this
/// itself for its own wrapper form; a component adopting the scroll-area contract
/// in place on an element it already renders (design's in-place adoption shape) calls
/// this manually, above both that element and the [`ScrollAreaScrollbar`] siblings it
/// renders alongside it.
pub fn provide_scroll_area_context() -> ScrollAreaContext {
    use_context_provider(ScrollAreaContext::new)
}

/// Reads the nearest ancestor [`ScrollAreaContext`]. Panics if none is present --
/// [`ScrollAreaScrollbar`]/[`ScrollAreaThumb`]/[`ScrollAreaCorner`] must be rendered
/// under a [`ScrollArea`], or a component that called [`provide_scroll_area_context`].
pub fn use_scroll_area_context() -> ScrollAreaContext {
    use_context()
}

/// Programmatically scrolls the viewport to an absolute `(top, left)` offset, smoothly
/// animated. A no-op if the viewport hasn't mounted yet (e.g. called before first
/// render completes) or the offset is out of range (the browser clamps it).
///
/// Must be called from a descendant of a component that provided a
/// [`ScrollAreaContext`] (see [`provide_scroll_area_context`]'s doc comment) --
/// panics otherwise, the same contract every other primitive's context-scoped hook in
/// this crate already follows (e.g. `message_scroller.rs`'s
/// `use_message_scroller_scroll_to_bottom`, which this mirrors).
pub fn use_scroll_area_scroll_to() -> Callback<(f64, f64)> {
    let ctx: ScrollAreaContext = use_context();
    use_callback(move |(top, left)| {
        spawn(ctx.scroll_to(top, left, ScrollBehavior::Smooth));
    })
}

/// The `onmounted` handler the real scrolling element must attach, whether that's
/// [`ScrollAreaViewport`] (wrapper adoption) or an existing element a component already
/// renders (in-place adoption).
pub fn scroll_area_viewport_onmounted(
    mut ctx: ScrollAreaContext,
) -> impl FnMut(Event<MountedData>) {
    move |event: Event<MountedData>| {
        let handle = event.data();
        ctx.viewport_ref.set(Some(handle.clone()));
        spawn(async move {
            if let Ok(rect) = handle.get_client_rect().await {
                ctx.viewport_height.set(rect.height());
                ctx.viewport_width.set(rect.width());
            }
            if let Ok(size) = handle.get_scroll_size().await {
                ctx.content_height.set(size.height);
                ctx.content_width.set(size.width);
            }
            ctx.measured.set(true);
        });
    }
}

/// The `onresize` handler the real scrolling element should attach alongside
/// [`scroll_area_viewport_onmounted`]/[`scroll_area_viewport_onscroll`]. Live-verified
/// as necessary, not speculative: `onmounted`'s own `get_client_rect()`/
/// `get_scroll_size()` pair can race the browser's own layout pass on first paint (
/// confirmed live -- a freshly loaded page reproducibly measured a smaller content
/// size than the DOM's actual `scrollHeight`, self-correcting only once a later
/// `onscroll` event supplied an accurate live reading). `ResizeObserver` (backed by
/// Dioxus's native `onresize`, already established as reliable elsewhere in this
/// crate -- see `resizable.rs`'s `ResizablePanelGroup`, `message_scroller.rs`'s
/// `MessageScrollerContent`) always reports the settled post-layout size on its first
/// callback, so wiring it in corrects the race without a magic delay.
pub fn scroll_area_viewport_onresize(mut ctx: ScrollAreaContext) -> impl FnMut(Event<ResizeData>) {
    move |event: Event<ResizeData>| {
        let Ok(size) = event.data().get_content_box_size() else {
            return;
        };
        ctx.viewport_height.set(size.height);
        ctx.viewport_width.set(size.width);
        if let Some(handle) = (ctx.viewport_ref)() {
            spawn(async move {
                if let Ok(scroll_size) = handle.get_scroll_size().await {
                    ctx.content_height.set(scroll_size.height);
                    ctx.content_width.set(scroll_size.width);
                }
                ctx.measured.set(true);
            });
        }
    }
}

/// The `onscroll` handler the real scrolling element must attach, matching
/// [`scroll_area_viewport_onmounted`].
pub fn scroll_area_viewport_onscroll(mut ctx: ScrollAreaContext) -> impl FnMut(Event<ScrollData>) {
    move |event: Event<ScrollData>| {
        let data = event.data();
        ctx.scroll_top.set(data.scroll_top().max(0.0));
        ctx.scroll_left.set(data.scroll_left().max(0.0));
        if data.client_height() > 0 {
            ctx.viewport_height.set(data.client_height() as f64);
        }
        if data.client_width() > 0 {
            ctx.viewport_width.set(data.client_width() as f64);
        }
        if data.scroll_height() > 0 {
            ctx.content_height.set(data.scroll_height() as f64);
        }
        if data.scroll_width() > 0 {
            ctx.content_width.set(data.scroll_width() as f64);
        }
    }
}

/// Returns the visibility class [`ScrollArea`] itself renders, so call sites that
/// cannot wrap their scroll element in a new `ScrollArea` node (an element that
/// already carries an ARIA role, or one already instrumented with its own
/// `onmounted`/`onscroll` handlers) can apply the same themed-scrollbar contract to
/// their own element in place instead. Themed by `packages/adico-cli/src/css.rs`'s
/// `SCROLLBAR_CSS` (native-scrollbar fallback) as a stable, non-Tailwind class name.
pub fn scroll_area_visibility_class(always_show: bool) -> &'static str {
    if always_show {
        "dx-scroll-area-always-show"
    } else {
        "dx-scroll-area-auto-hide"
    }
}

/// The props for the [`ScrollArea`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ScrollAreaProps {
    /// The scroll direction.
    #[props(default)]
    pub direction: ReadSignal<ScrollDirection>,

    /// Whether the scrollbars should be always visible.
    #[props(default)]
    pub always_show_scrollbars: ReadSignal<bool>,

    /// The scroll type.
    #[props(default)]
    pub scroll_type: ReadSignal<ScrollType>,

    /// How the axis *not* covered by `direction` behaves when `direction` is
    /// single-axis (`Vertical`/`Horizontal`). Defaults to [`CrossAxisOverflow::Clip`],
    /// matching this component's original behavior.
    #[props(default)]
    pub cross_axis_overflow: ReadSignal<CrossAxisOverflow>,

    /// Extra classes, appended after (never replacing) the internal `dx-scroll-area-root`
    /// class. Applied to the outer positioning wrapper (Radix/shadcn's "Root"), not the
    /// inner scrolling element -- this is where sizing (`height`/`width`/`border`/
    /// margins) belongs; the viewport always fills whatever size the wrapper ends up
    /// being.
    #[props(default)]
    pub class: Option<String>,

    /// Extra inline style declarations, appended after (never replacing) the internal
    /// `position: relative; overflow: hidden;` base. Applied to the outer wrapper, for
    /// the same reason as `class` above.
    #[props(default)]
    pub style: Option<String>,

    /// Additional attributes to apply to the viewport element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the scroll area component.
    pub children: Element,
}

/// Computes the merged class/style/overflow values shared by [`ScrollAreaViewport`],
/// factored out so [`ScrollArea`]'s wrapper composition and a standalone
/// [`ScrollAreaViewport`] render identically.
fn viewport_style_and_class(
    direction: ScrollDirection,
    scroll_type: ScrollType,
    cross_axis_overflow: CrossAxisOverflow,
    always_show: bool,
    hide_native_scrollbar: bool,
    extra_class: Option<&str>,
    extra_style: Option<&str>,
) -> (String, String) {
    let scroll_value = match scroll_type {
        ScrollType::Auto => "auto",
        ScrollType::Always | ScrollType::Hidden => "scroll",
    };
    let cross_value = match cross_axis_overflow {
        CrossAxisOverflow::Clip => "hidden",
        CrossAxisOverflow::Visible => "visible",
    };
    let (overflow_x, overflow_y) = match direction {
        ScrollDirection::Vertical => (cross_value, scroll_value),
        ScrollDirection::Horizontal => (scroll_value, cross_value),
        ScrollDirection::Both => (scroll_value, scroll_value),
    };

    let mut computed_style = format!("overflow-x: {overflow_x}; overflow-y: {overflow_y};");
    if hide_native_scrollbar || scroll_type == ScrollType::Hidden {
        computed_style.push_str(" scrollbar-width: none;");
    }
    let merged_style = match extra_style {
        Some(extra) if !extra.is_empty() => format!("{computed_style} {extra}"),
        _ => computed_style,
    };

    let visibility_class = scroll_area_visibility_class(always_show);
    let merged_class = match extra_class {
        Some(extra) if !extra.is_empty() => format!("{visibility_class} {extra}"),
        _ => visibility_class.to_string(),
    };

    (merged_class, merged_style)
}

/// The props for the [`ScrollAreaViewport`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ScrollAreaViewportProps {
    /// The scroll direction.
    #[props(default)]
    pub direction: ReadSignal<ScrollDirection>,
    /// Whether the scrollbars should be always visible.
    #[props(default)]
    pub always_show_scrollbars: ReadSignal<bool>,
    /// The scroll type.
    #[props(default)]
    pub scroll_type: ReadSignal<ScrollType>,
    /// How the axis *not* covered by `direction` behaves.
    #[props(default)]
    pub cross_axis_overflow: ReadSignal<CrossAxisOverflow>,
    /// Whether the native scrollbar is unconditionally hidden regardless of
    /// `scroll_type` -- `true` when this viewport is paired with
    /// [`ScrollAreaScrollbar`]/[`ScrollAreaThumb`] overlay parts (the only sensible
    /// default once a custom thumb replaces the native one; showing both would look
    /// duplicated), `false` to fall back to `scroll_type`'s original native-scrollbar
    /// semantics for a viewport used standalone, with no overlay parts.
    #[props(default = true)]
    pub hide_native_scrollbar: bool,
    /// Extra classes, appended after the internal visibility class.
    #[props(default)]
    pub class: Option<String>,
    /// Extra inline style, appended after the internal computed style.
    #[props(default)]
    pub style: Option<String>,
    /// Additional attributes to apply to the viewport element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the viewport.
    pub children: Element,
}

/// # ScrollAreaViewport
///
/// The real scrolling element: a genuine native-`overflow` `div`, instrumented to
/// drive [`ScrollAreaScrollbar`]/[`ScrollAreaThumb`]/[`ScrollAreaCorner`] siblings via
/// [`ScrollAreaContext`]. Must be rendered under a component that called
/// [`provide_scroll_area_context`] -- [`ScrollArea`] does this for you; a component
/// adopting the scroll-area contract in place on its own existing element does not
/// use this component at all, instead attaching [`scroll_area_viewport_onmounted`]/
/// [`scroll_area_viewport_onscroll`] directly.
#[component]
pub fn ScrollAreaViewport(props: ScrollAreaViewportProps) -> Element {
    let ctx: ScrollAreaContext = use_context();
    let (merged_class, merged_style) = viewport_style_and_class(
        props.direction.cloned(),
        props.scroll_type.cloned(),
        props.cross_axis_overflow.cloned(),
        props.always_show_scrollbars.cloned(),
        props.hide_native_scrollbar,
        props.class.as_deref(),
        props.style.as_deref(),
    );

    let onmounted = scroll_area_viewport_onmounted(ctx);
    let onscroll = scroll_area_viewport_onscroll(ctx);
    let onresize = scroll_area_viewport_onresize(ctx);

    rsx! {
        div {
            class: "{merged_class}",
            style: "{merged_style}",
            "data-scroll-direction": props.direction.cloned().data_attr(),
            onmounted,
            onscroll,
            onresize,
            ..props.attributes,

            {props.children}
        }
    }
}

impl ScrollDirection {
    fn data_attr(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
            Self::Both => "both",
        }
    }
}

/// # ScrollArea
///
/// The `ScrollArea` component creates a scrollable area with a themed overlay
/// scrollbar. If you don't have any focusable content within the scroll area, you
/// should make the scroll area focusable by adding a `tabindex` attribute.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::scroll_area::{ScrollArea, ScrollDirection};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         ScrollArea {
///             style: "width: 10em; height: 10em;",
///             direction: ScrollDirection::Vertical,
///             tabindex: "0",
///             div {
///                 for i in 1..=20 {
///                     p {
///                         "Scrollable content item {i}"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// `class`/`style` (and sizing generally -- `height`/`width`/`border`/margins) apply to
/// the outer positioning wrapper, not the inner scrolling element, matching Radix/
/// shadcn's own Root-vs-Viewport split: the viewport always fills whatever size the
/// wrapper ends up being. Other attributes (e.g. `tabindex`) apply to the inner
/// scrolling element, since that's the element that actually receives keyboard focus.
///
/// The [`ScrollArea`] component defines the following data attributes you can use to control styling:
/// - `data-scroll-direction`: Indicates the scroll direction. Values are `vertical`, `horizontal`, or `both`.
#[component]
pub fn ScrollArea(props: ScrollAreaProps) -> Element {
    provide_scroll_area_context();
    let direction = props.direction.cloned();
    let show_vertical = matches!(direction, ScrollDirection::Vertical | ScrollDirection::Both);
    let show_horizontal = matches!(
        direction,
        ScrollDirection::Horizontal | ScrollDirection::Both
    );

    // Sizing (`height`/`width`/`border`/margins/flex participation) belongs on this
    // outer wrapper -- the element a caller's surrounding layout actually addresses --
    // not on the inner viewport, matching Radix/shadcn's own Root-vs-Viewport split.
    // Getting this backwards (an earlier version of this component routed the caller's
    // `class`/`style` to the viewport and hardcoded `height: 100%; width: 100%;` on the
    // wrapper) was caught live: the wrapper's forced 100%/100% stretched to fill its
    // ambient flex container instead of respecting the caller's intended size, since
    // nothing sized the wrapper itself. The viewport now always fills whatever size the
    // wrapper ends up being -- controlled entirely by the caller's `class`/`style` here.
    let root_class = match props.class.as_deref() {
        Some(extra) if !extra.is_empty() => format!("dx-scroll-area-root {extra}"),
        _ => "dx-scroll-area-root".to_string(),
    };
    let root_style = match props.style.as_deref() {
        Some(extra) if !extra.is_empty() => {
            format!("position: relative; overflow: hidden; {extra}")
        }
        _ => "position: relative; overflow: hidden;".to_string(),
    };

    rsx! {
        div {
            class: "{root_class}",
            style: "{root_style}",

            ScrollAreaViewport {
                direction: props.direction,
                always_show_scrollbars: props.always_show_scrollbars,
                scroll_type: props.scroll_type,
                cross_axis_overflow: props.cross_axis_overflow,
                style: "height: 100%; width: 100%;",
                attributes: props.attributes.clone(),
                {props.children}
            }

            if show_vertical {
                ScrollAreaScrollbar { orientation: ScrollbarOrientation::Vertical }
            }
            if show_horizontal {
                ScrollAreaScrollbar { orientation: ScrollbarOrientation::Horizontal }
            }
            ScrollAreaCorner {}
        }
    }
}

/// The props for the [`ScrollAreaScrollbar`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ScrollAreaScrollbarProps {
    /// Which edge this scrollbar tracks.
    pub orientation: ScrollbarOrientation,
    /// Extra classes appended to the internal track class.
    #[props(default)]
    pub class: Option<String>,
}

/// # ScrollAreaScrollbar
///
/// The overlay scrollbar track for one axis, containing a [`ScrollAreaThumb`]. Must
/// be rendered under a component that called [`provide_scroll_area_context`].
/// Renders nothing until [`ScrollAreaContext`] has measured real content/viewport
/// geometry (never during SSR, and not before hydration measures), and nothing at
/// all if that axis has no overflow to scroll.
#[component]
pub fn ScrollAreaScrollbar(props: ScrollAreaScrollbarProps) -> Element {
    let ctx: ScrollAreaContext = use_context();
    let orientation = props.orientation;

    if !ctx.needs_scrollbar(orientation) {
        return rsx! {};
    }

    let track_class = match props.class.as_deref() {
        Some(extra) if !extra.is_empty() => format!("dx-scroll-area-scrollbar {extra}"),
        _ => "dx-scroll-area-scrollbar".to_string(),
    };

    rsx! {
        div {
            class: "{track_class}",
            "data-orientation": orientation.data_attr(),
            ScrollAreaThumb { orientation }
        }
    }
}

/// # ScrollAreaThumb
///
/// The draggable overlay thumb inside a [`ScrollAreaScrollbar`]. Its size and
/// position are computed from [`ScrollAreaContext`]'s measured geometry; dragging it
/// scrolls the real viewport (native per-element pointer events, matching this
/// crate's `registry/ui/resizable.rs` drag convention -- see this module's own doc
/// comment).
#[component]
pub fn ScrollAreaThumb(orientation: ScrollbarOrientation) -> Element {
    let mut ctx: ScrollAreaContext = use_context();

    let size = ctx.thumb_size(orientation);
    let offset = ctx.thumb_offset(orientation);
    let size_offset_style = match orientation {
        ScrollbarOrientation::Vertical => format!("height: {size}px; top: {offset}px;"),
        ScrollbarOrientation::Horizontal => format!("width: {size}px; left: {offset}px;"),
    };

    let dragging = matches!(*ctx.drag.read(), Some(drag) if drag.orientation == orientation);

    rsx! {
        div {
            class: "dx-scroll-area-thumb",
            style: "{size_offset_style}",
            "data-orientation": orientation.data_attr(),
            onpointerdown: move |event: Event<PointerData>| {
                let point = event.client_coordinates();
                let start_client = match orientation {
                    ScrollbarOrientation::Vertical => point.y,
                    ScrollbarOrientation::Horizontal => point.x,
                };
                let start_scroll = ctx.scroll_offset(orientation);
                ctx.drag
                    .set(Some(ThumbDrag { orientation, start_client, start_scroll }));
            },
        }
        if dragging {
            div {
                class: "fixed inset-0 z-[100] select-none",
                onpointermove: move |event: Event<PointerData>| {
                    let Some(drag) = *ctx.drag.peek() else {
                        return;
                    };
                    if drag.orientation != orientation {
                        return;
                    }
                    let point = event.client_coordinates();
                    let client = match orientation {
                        ScrollbarOrientation::Vertical => point.y,
                        ScrollbarOrientation::Horizontal => point.x,
                    };
                    let track = ctx.track_size(orientation);
                    let thumb = ctx.thumb_size(orientation);
                    let max_scroll = ctx.max_scroll(orientation);
                    if track <= thumb || max_scroll <= 0.0 {
                        return;
                    }
                    let delta_px = client - drag.start_client;
                    let scroll_delta = delta_px * max_scroll / (track - thumb);
                    let new_offset = (drag.start_scroll + scroll_delta).clamp(0.0, max_scroll);
                    let (top, left) = match orientation {
                        ScrollbarOrientation::Vertical => (new_offset, (ctx.scroll_left)()),
                        ScrollbarOrientation::Horizontal => ((ctx.scroll_top)(), new_offset),
                    };
                    match orientation {
                        ScrollbarOrientation::Vertical => ctx.scroll_top.set(new_offset),
                        ScrollbarOrientation::Horizontal => ctx.scroll_left.set(new_offset),
                    }
                    spawn(ctx.scroll_to(top, left, ScrollBehavior::Instant));
                },
                onpointerup: move |_| ctx.drag.set(None),
            }
        }
    }
}

/// # ScrollAreaCorner
///
/// The small square where a vertical and horizontal [`ScrollAreaScrollbar`] meet.
/// Renders nothing unless both axes actually need a scrollbar.
#[component]
pub fn ScrollAreaCorner() -> Element {
    let ctx: ScrollAreaContext = use_context();
    if !(ctx.needs_scrollbar(ScrollbarOrientation::Vertical)
        && ctx.needs_scrollbar(ScrollbarOrientation::Horizontal))
    {
        return rsx! {};
    }
    rsx! {
        div { class: "dx-scroll-area-corner" }
    }
}
