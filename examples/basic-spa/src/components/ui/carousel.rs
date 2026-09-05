//! Source-owned shadcn-style Carousel for Dioxus, built directly on native
//! CSS scroll-snap plus Dioxus's own per-element `MountedData::scroll`/
//! `onscroll` -- deliberately not a drag/swipe gesture, and with no
//! dependency on `adico-primitives`' `pointer.rs` global pointer-position
//! registry, a documented, unconfirmed-in-browser defect on `web` (see
//! `packages/adico-primitives/src/gesture.rs`'s own module doc comment).
//! This is a named, deliberate scope reduction from upstream's
//! `embla-carousel-react` (momentum drag, loop mode, autoplay plugins,
//! `CarouselApi`/`setApi`, dot indicators) -- see the M7 task audit
//! (`openspec/changes/build-adico-component-ecosystem/tasks.md`, task 8.1)
//! for why. Registry-layer composition only, matching this repo's own
//! precedent for `resizable`'s similarly reduced-primitive-need shape: no new
//! `adico-primitives` module was needed for scroll-snap paging plus native
//! `onscroll`-derived boundary state.

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

    /// Pages one viewport's worth in `direction` (-1.0 previous, 1.0 next),
    /// clamped to the scrollable range, and imperatively scrolls the
    /// content element there. Writes `scroll_offset` optimistically -- the
    /// content's own `onscroll` handler (below) then overwrites it with the
    /// authoritative value as the smooth scroll actually progresses.
    fn page(&mut self, direction: f64) {
        let Some(handle) = self.content_ref.peek().clone() else {
            return;
        };
        let viewport = *self.viewport_size.peek();
        let current = *self.scroll_offset.peek();
        let max_offset = (*self.content_size.peek() - viewport).max(0.0);
        let target = (current + direction * viewport).clamp(0.0, max_offset);
        self.scroll_offset.set(target);
        let orientation = self.orientation;
        spawn(async move {
            let coordinates = match orientation {
                CarouselOrientation::Horizontal => Vector2D::new(target, 0.0),
                CarouselOrientation::Vertical => Vector2D::new(0.0, target),
            };
            let _ = handle.scroll(coordinates, ScrollBehavior::Smooth).await;
        });
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
    });
    let class = cn(&["relative", class.as_deref().unwrap_or_default()]);
    rsx! {
        div { class, role: "region", "aria-roledescription": "carousel", {children} }
    }
}

/// The scrollable, scroll-snapping track of items. Focus it and use the
/// arrow keys matching its [`CarouselOrientation`] to page, or use
/// [`CarouselPrevious`]/[`CarouselNext`].
#[component]
pub fn CarouselContent(children: Element, class: Option<String>) -> Element {
    let mut ctx: CarouselContext = use_context();
    let orientation = ctx.orientation;
    let axis_class = match orientation {
        CarouselOrientation::Horizontal => "flex snap-x snap-mandatory overflow-x-auto -ml-4",
        CarouselOrientation::Vertical => {
            "flex h-[24rem] flex-col snap-y snap-mandatory overflow-y-auto -mt-4"
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

    rsx! {
        div {
            class,
            tabindex: "0",
            onmounted,
            onscroll,
            onkeydown,
            {children}
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
