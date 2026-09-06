//! Source-owned shadcn-style Carousel for Dioxus, built directly on native
//! CSS scroll-snap plus Dioxus's own per-element `MountedData::scroll`/
//! `onscroll`, with pointer-drag paging layered on top.
//!
//! Drag uses per-element pointer events (`onpointerdown` on the track, then
//! a transient full-screen overlay carrying `onpointermove`/`onpointerup`/
//! `onpointercancel` -- the same containment pattern as `resizable.rs`) and
//! has **no** dependency on `adico-primitives`' `pointer.rs` global
//! pointer-position registry. An earlier revision of this module cited a
//! "broken-on-web `document::eval` pattern" as the reason drag was omitted
//! entirely; that claim was later retracted after live-browser verification
//! (see `packages/adico-primitives/src/positioner.rs`'s 2026-09-03 note),
//! and the per-element approach used here never depended on it either way.
//! A mouse drag past 20% of the viewport pages one slide in the drag
//! direction; a shorter drag snaps back. Touch input is left to the
//! browser's own scroll-snap panning, which already pages natively.
//!
//! The remaining scope reduction from upstream's `embla-carousel-react` is
//! still deliberate: momentum/velocity physics, loop mode, autoplay
//! plugins, `CarouselApi`/`setApi`, and dot indicators are not built.
//! Registry-layer composition only, matching this repo's own precedent for
//! `resizable`'s similarly reduced-primitive-need shape.

use std::rc::Rc;

use dioxus::html::geometry::euclid::Vector2D;
use dioxus::prelude::*;

use super::button::{Button, ButtonSize, ButtonVariant};
use crate::adico_lib::cn::cn;
use adico_primitives::icons::{ArrowLeft, ArrowRight};

/// Below this many CSS pixels of remaining scroll room, treat an edge as
/// reached -- native smooth-scroll snapping rarely lands on an exact integer
/// offset.
const SCROLL_END_TOLERANCE: f64 = 1.0;

/// A released drag whose distance along the scroll axis exceeds this
/// fraction of the viewport pages one slide; anything shorter snaps back.
const DRAG_PAGE_THRESHOLD: f64 = 0.2;

/// An in-progress pointer drag on the track. `delta` is positive when the
/// pointer moved backward along the axis (content dragged toward the next
/// slide).
#[derive(Clone, Copy, PartialEq)]
struct CarouselDrag {
    start_coord: f64,
    start_offset: f64,
    delta: f64,
}

/// The axis a [`Carousel`] pages along.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselOrientation {
    /// Items are paged left/right (the default).
    #[default]
    Horizontal,
    /// Items are paged up/down.
    Vertical,
}

#[derive(Clone, Copy)]
struct CarouselContext {
    orientation: CarouselOrientation,
    content_ref: Signal<Option<Rc<MountedData>>>,
    scroll_offset: Signal<f64>,
    viewport_size: Signal<f64>,
    content_size: Signal<f64>,
    drag: Signal<Option<CarouselDrag>>,
}

impl CarouselContext {
    fn can_scroll_prev(&self) -> bool {
        *self.scroll_offset.read() > SCROLL_END_TOLERANCE
    }

    fn can_scroll_next(&self) -> bool {
        let offset = *self.scroll_offset.read();
        let viewport = *self.viewport_size.read();
        let content = *self.content_size.read();
        content > 0.0 && offset + viewport < content - SCROLL_END_TOLERANCE
    }

    /// Imperatively scrolls the content element to `target` along the
    /// carousel's axis. Writes `scroll_offset` optimistically -- the
    /// content's own `onscroll` handler (below) then overwrites it with the
    /// authoritative value as the scroll actually progresses.
    fn scroll_to(&mut self, target: f64, behavior: ScrollBehavior) {
        let Some(handle) = self.content_ref.peek().clone() else {
            return;
        };
        self.scroll_offset.set(target);
        let orientation = self.orientation;
        spawn(async move {
            let coordinates = match orientation {
                CarouselOrientation::Horizontal => Vector2D::new(target, 0.0),
                CarouselOrientation::Vertical => Vector2D::new(0.0, target),
            };
            let _ = handle.scroll(coordinates, behavior).await;
        });
    }

    fn max_offset(&self) -> f64 {
        (*self.content_size.peek() - *self.viewport_size.peek()).max(0.0)
    }

    /// Pages one viewport's worth in `direction` (-1.0 previous, 1.0 next),
    /// clamped to the scrollable range.
    fn page(&mut self, direction: f64) {
        let viewport = *self.viewport_size.peek();
        let current = *self.scroll_offset.peek();
        let target = (current + direction * viewport).clamp(0.0, self.max_offset());
        self.scroll_to(target, ScrollBehavior::Smooth);
    }

    fn scroll_prev(&mut self) {
        self.page(-1.0);
    }

    fn scroll_next(&mut self) {
        self.page(1.0);
    }
}

/// The root of a scroll-snap carousel: a `relative`-positioned wrapper around
/// a [`CarouselContent`] and optional [`CarouselPrevious`]/[`CarouselNext`]
/// paging controls.
///
/// Deliberately has no `radius` prop anywhere in this file: neither
/// `Carousel`, `CarouselContent`, nor `CarouselItem` has a bounded surface
/// of its own — the only `rounded-full` in this module is the shared nav-
/// button class used by `CarouselPrevious`/`CarouselNext`, which is not the
/// carousel's own surface.
#[component]
pub fn Carousel(
    children: Element,
    class: Option<String>,
    orientation: Option<CarouselOrientation>,
) -> Element {
    let orientation = orientation.unwrap_or_default();
    use_context_provider(|| CarouselContext {
        orientation,
        content_ref: Signal::new(None),
        scroll_offset: Signal::new(0.0),
        viewport_size: Signal::new(0.0),
        content_size: Signal::new(0.0),
        drag: Signal::new(None),
    });
    let class = cn(&["relative", class.as_deref().unwrap_or_default()]);
    rsx! {
        div { class, role: "region", "aria-roledescription": "carousel", {children} }
    }
}

/// The scrollable, scroll-snapping track of items. Focus it and use the
/// arrow keys matching its [`CarouselOrientation`] to page, drag it with the
/// mouse, or use [`CarouselPrevious`]/[`CarouselNext`].
#[component]
pub fn CarouselContent(children: Element, class: Option<String>) -> Element {
    let mut ctx: CarouselContext = use_context();
    let orientation = ctx.orientation;
    let dragging = ctx.drag.read().is_some();
    // Snap must be fully off while dragging: `scroll-snap-type` would fight
    // every instant reposition. Branching the whole axis class (rather than
    // appending `snap-none`) avoids relying on stylesheet order to resolve
    // two competing snap utilities.
    let axis_class = match (orientation, dragging) {
        (CarouselOrientation::Horizontal, false) => {
            "flex snap-x snap-mandatory overflow-x-auto -ml-4 cursor-grab"
        }
        (CarouselOrientation::Horizontal, true) => {
            "flex snap-none overflow-x-auto -ml-4 cursor-grabbing select-none"
        }
        (CarouselOrientation::Vertical, false) => {
            "flex h-[24rem] flex-col snap-y snap-mandatory overflow-y-auto -mt-4 cursor-grab"
        }
        (CarouselOrientation::Vertical, true) => {
            "flex h-[24rem] flex-col snap-none overflow-y-auto -mt-4 cursor-grabbing select-none"
        }
    };
    let class = cn(&[
        axis_class,
        "outline-none focus-visible:ring-2 focus-visible:ring-ring",
        class.as_deref().unwrap_or_default(),
    ]);

    let onmounted = move |event: Event<MountedData>| {
        let handle = event.data();
        ctx.content_ref.set(Some(handle.clone()));
        spawn(async move {
            if let Ok(rect) = handle.get_client_rect().await {
                let size = match orientation {
                    CarouselOrientation::Horizontal => rect.width(),
                    CarouselOrientation::Vertical => rect.height(),
                };
                ctx.viewport_size.set(size);
            }
            if let Ok(size) = handle.get_scroll_size().await {
                let content = match orientation {
                    CarouselOrientation::Horizontal => size.width,
                    CarouselOrientation::Vertical => size.height,
                };
                ctx.content_size.set(content);
            }
        });
    };

    let onscroll = move |event: Event<ScrollData>| {
        let data = event.data();
        let (offset, viewport, content) = match orientation {
            CarouselOrientation::Horizontal => (
                data.scroll_left(),
                data.client_width() as f64,
                data.scroll_width() as f64,
            ),
            CarouselOrientation::Vertical => (
                data.scroll_top(),
                data.client_height() as f64,
                data.scroll_height() as f64,
            ),
        };
        ctx.scroll_offset.set(offset.max(0.0));
        if viewport > 0.0 {
            ctx.viewport_size.set(viewport);
        }
        if content > 0.0 {
            ctx.content_size.set(content);
        }
    };

    let onkeydown = move |event: Event<KeyboardData>| match (orientation, event.key()) {
        (CarouselOrientation::Horizontal, Key::ArrowLeft)
        | (CarouselOrientation::Vertical, Key::ArrowUp) => {
            event.prevent_default();
            ctx.scroll_prev();
        }
        (CarouselOrientation::Horizontal, Key::ArrowRight)
        | (CarouselOrientation::Vertical, Key::ArrowDown) => {
            event.prevent_default();
            ctx.scroll_next();
        }
        _ => {}
    };

    // Mouse only: touch input already pans-and-snaps natively via the
    // overflow scroll container, and fighting it with instant repositions
    // would judder.
    let onpointerdown = move |event: Event<PointerData>| {
        if event.data().pointer_type() != "mouse" {
            return;
        }
        let point = event.client_coordinates();
        let start_coord = match orientation {
            CarouselOrientation::Horizontal => point.x,
            CarouselOrientation::Vertical => point.y,
        };
        ctx.drag.set(Some(CarouselDrag {
            start_coord,
            start_offset: *ctx.scroll_offset.peek(),
            delta: 0.0,
        }));
    };

    // Releasing (or losing) the pointer decides the page from the total
    // drag distance: past the threshold pages one slide in the drag
    // direction, anything shorter snaps back to where the drag began.
    let mut release = move |drag: CarouselDrag| {
        ctx.drag.set(None);
        let viewport = *ctx.viewport_size.peek();
        let max_offset = ctx.max_offset();
        let threshold = viewport * DRAG_PAGE_THRESHOLD;
        let target = if drag.delta > threshold {
            (drag.start_offset + viewport).clamp(0.0, max_offset)
        } else if drag.delta < -threshold {
            (drag.start_offset - viewport).clamp(0.0, max_offset)
        } else {
            drag.start_offset
        };
        ctx.scroll_to(target, ScrollBehavior::Smooth);
    };

    rsx! {
        div {
            class,
            tabindex: "0",
            onmounted,
            onscroll,
            onkeydown,
            onpointerdown,
            {children}
        }
        if let Some(drag) = *ctx.drag.read() {
            div {
                class: "fixed inset-0 z-[100] cursor-grabbing select-none",
                onpointermove: move |event: Event<PointerData>| {
                    let point = event.client_coordinates();
                    let current = match orientation {
                        CarouselOrientation::Horizontal => point.x,
                        CarouselOrientation::Vertical => point.y,
                    };
                    let delta = drag.start_coord - current;
                    ctx.drag.set(Some(CarouselDrag { delta, ..drag }));
                    let target = (drag.start_offset + delta).clamp(0.0, ctx.max_offset());
                    ctx.scroll_to(target, ScrollBehavior::Instant);
                },
                onpointerup: move |_| release(drag),
                onpointercancel: move |_| release(drag),
            }
        }
    }
}

/// A single slide in a [`CarouselContent`]. Defaults to one-per-view
/// (`basis-full`) -- pass `class: "basis-1/2"` (or any other `basis-*`
/// utility) for a multi-item-per-view layout, matching upstream's own
/// className-driven sizing.
#[component]
pub fn CarouselItem(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "min-w-0 shrink-0 grow-0 basis-full snap-start pl-4",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, role: "group", "aria-roledescription": "slide", {children} }
    }
}

fn nav_button_class(orientation: CarouselOrientation, class: Option<&str>) -> String {
    let position = match orientation {
        CarouselOrientation::Horizontal => "top-1/2 -translate-y-1/2",
        CarouselOrientation::Vertical => "left-1/2 -translate-x-1/2",
    };
    cn(&["absolute rounded-full", position, class.unwrap_or_default()])
}

/// Pages the surrounding [`Carousel`] one viewport backward. Disabled (and
/// not clickable) once already at the start.
#[component]
pub fn CarouselPrevious(class: Option<String>) -> Element {
    let mut ctx: CarouselContext = use_context();
    let position = match ctx.orientation {
        CarouselOrientation::Horizontal => "-left-12",
        CarouselOrientation::Vertical => "-top-12",
    };
    let class = nav_button_class(ctx.orientation, class.as_deref());
    let class = cn(&[&class, position]);
    let disabled = !ctx.can_scroll_prev();
    rsx! {
        Button {
            class,
            variant: ButtonVariant::Outline,
            size: ButtonSize::Icon,
            disabled,
            onclick: move |_| ctx.scroll_prev(),
            ArrowLeft { class: "size-4" }
            span { class: "sr-only", "Previous slide" }
        }
    }
}

/// Pages the surrounding [`Carousel`] one viewport forward. Disabled (and
/// not clickable) once already at the end.
#[component]
pub fn CarouselNext(class: Option<String>) -> Element {
    let mut ctx: CarouselContext = use_context();
    let position = match ctx.orientation {
        CarouselOrientation::Horizontal => "-right-12",
        CarouselOrientation::Vertical => "-bottom-12",
    };
    let class = nav_button_class(ctx.orientation, class.as_deref());
    let class = cn(&[&class, position]);
    let disabled = !ctx.can_scroll_next();
    rsx! {
        Button {
            class,
            variant: ButtonVariant::Outline,
            size: ButtonSize::Icon,
            disabled,
            onclick: move |_| ctx.scroll_next(),
            ArrowRight { class: "size-4" }
            span { class: "sr-only", "Next slide" }
        }
    }
}
