//! Source-owned shadcn-style Resizable panel group for Dioxus, with no
//! dedicated `adico-primitives` module.
//!
//! Upstream shadcn's `resizable` wraps `react-resizable-panels`. 8.1's task
//! audit (`openspec/changes/build-adico-component-ecosystem/tasks.md`, task
//! 8.1) originally suggested reusing `adico_primitives::move_interaction`'s
//! existing track-drag for the resize handle -- but `move_interaction.rs`'s
//! own pointer tracking (`MoveInteraction::pointer_move`) reads
//! `adico_primitives::pointer`'s *global* pointer-position registry, which
//! is the exact documented, unconfirmed-in-browser-but-real defect
//! `gesture.rs`'s own module doc comment names (fed by a `document::eval`
//! listener pattern already confirmed non-functional on `web` elsewhere in
//! this crate). Reusing `move_interaction.rs` here would route this new
//! component through the same defect 8.1 was trying to avoid for
//! `drawer`/`carousel`.
//!
//! Instead, this drags entirely on native, per-element Dioxus pointer events
//! (`onpointerdown`/`onpointermove`/`onpointerup`), the same class of API
//! `gesture.rs`'s own long-press already uses successfully with no
//! `pointer.rs` involvement, plus `carousel.rs`'s own precedent (this
//! session) that per-element `MountedData`/native `onscroll`/`onresize` work
//! correctly in a real browser. There is no `setPointerCapture` call (no
//! existing precedent for it anywhere in this crate, and no public API on
//! Dioxus's `MountedData` to reach the raw DOM element for one) -- instead,
//! while a handle is being dragged, a temporary `fixed inset-0` transparent
//! overlay is rendered to catch `onpointermove`/`onpointerup` regardless of
//! where the pointer travels, the standard technique for pointer dragging
//! without capture. This sidesteps `pointer.rs` entirely rather than reusing
//! it, correcting 8.1's own suggestion rather than silently working around
//! it -- see task 8.4's own completion note for why.
//!
//! Panel sizes are literal `%`-of-container-width/height flex bases (not
//! `flex-grow` ratios of remaining space after handle width) -- at ordinary
//! handle widths (a handful of pixels) this is a negligible, common
//! simplification, not a chased sub-pixel precision goal.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;
use adico_primitives::scroll_area::scroll_area_visibility_class;

const KEYBOARD_STEP_PERCENT: f64 = 5.0;

/// The axis a [`ResizablePanelGroup`] lays its panels out (and resizes)
/// along.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ResizableDirection {
    /// Panels sit side by side; handles drag left/right (the default).
    #[default]
    Horizontal,
    /// Panels stack top to bottom; handles drag up/down.
    Vertical,
}

#[derive(Clone, Copy)]
struct PanelConstraints {
    size: f64,
    min: f64,
    max: f64,
}

#[derive(Clone, Copy)]
struct DragState {
    handle_index: usize,
    start_pos: f64,
    start_prev_size: f64,
    start_next_size: f64,
}

#[derive(Clone, Copy)]
struct ResizableContext {
    direction: ResizableDirection,
    container_size: Signal<f64>,
    panels: Signal<Vec<PanelConstraints>>,
    drag: Signal<Option<DragState>>,
}

/// Computes the delta (in percent) actually applied to the pair of panels
/// straddling a handle, clamping so neither panel crosses its own
/// min/max while their combined size stays exactly constant -- the
/// standard mutual-clamp a resizable split needs (clamping each side
/// independently would let the pair's total drift on every drag that hits
/// a bound).
fn clamp_delta(wanted: f64, prev: PanelConstraints, next: PanelConstraints) -> f64 {
    let max_delta = (prev.max - prev.size).min(next.size - next.min);
    let min_delta = (prev.min - prev.size).max(next.size - next.max);
    wanted.clamp(min_delta, max_delta)
}

/// Resizes the pair of panels straddling `handle_index`, computing the new
/// sizes from `base_prev`/`base_next` (the sizes to measure `wanted` against
/// -- the drag overlay passes the sizes captured at drag-start so repeated
/// small pointer moves don't accumulate clamping error; the keyboard handler
/// passes each panel's own current size for a plain incremental step).
fn resize_pair_from(
    panels: &mut [PanelConstraints],
    handle_index: usize,
    base_prev: f64,
    base_next: f64,
    wanted: f64,
) {
    let Some(prev) = panels.get(handle_index).copied() else {
        return;
    };
    let Some(next) = panels.get(handle_index + 1).copied() else {
        return;
    };
    let actual = clamp_delta(
        wanted,
        PanelConstraints {
            size: base_prev,
            ..prev
        },
        PanelConstraints {
            size: base_next,
            ..next
        },
    );
    if let Some(entry) = panels.get_mut(handle_index) {
        entry.size = base_prev + actual;
    }
    if let Some(entry) = panels.get_mut(handle_index + 1) {
        entry.size = base_next - actual;
    }
}

/// The root of a resizable split: a `flex` container of [`ResizablePanel`]s
/// separated by [`ResizableHandle`]s, all laid out along its
/// [`ResizableDirection`].
#[component]
pub fn ResizablePanelGroup(
    children: Element,
    class: Option<String>,
    direction: Option<ResizableDirection>,
) -> Element {
    let direction = direction.unwrap_or_default();
    let mut ctx = use_context_provider(|| ResizableContext {
        direction,
        container_size: Signal::new(0.0),
        panels: Signal::new(Vec::new()),
        drag: Signal::new(None),
    });

    let flex_direction = match direction {
        ResizableDirection::Horizontal => "flex-row",
        ResizableDirection::Vertical => "flex-col",
    };
    let class = cn(&[
        "flex h-full w-full",
        flex_direction,
        class.as_deref().unwrap_or_default(),
    ]);

    let mut measure = move |rect_size: f64| {
        ctx.container_size.set(rect_size);
    };
    let onmounted = move |event: Event<MountedData>| {
        let handle = event.data();
        spawn(async move {
            if let Ok(rect) = handle.get_client_rect().await {
                let size = match direction {
                    ResizableDirection::Horizontal => rect.width(),
                    ResizableDirection::Vertical => rect.height(),
                };
                measure(size);
            }
        });
    };
    let onresize = move |event: Event<ResizeData>| {
        let box_size = event.data().get_content_box_size().unwrap_or_default();
        let size = match direction {
            ResizableDirection::Horizontal => box_size.width,
            ResizableDirection::Vertical => box_size.height,
        };
        if size > 0.0 {
            measure(size);
        }
    };

    let overlay_cursor = match direction {
        ResizableDirection::Horizontal => "col-resize",
        ResizableDirection::Vertical => "row-resize",
    };

    rsx! {
        div { class, onmounted, onresize,
            {children}
            if let Some(drag) = *ctx.drag.read() {
                div {
                    class: "fixed inset-0 z-[100] select-none",
                    style: "cursor: {overlay_cursor};",
                    onpointermove: move |event: Event<PointerData>| {
                        let point = event.client_coordinates();
                        let pos = match direction {
                            ResizableDirection::Horizontal => point.x,
                            ResizableDirection::Vertical => point.y,
                        };
                        let container = *ctx.container_size.peek();
                        if container <= 0.0 {
                            return;
                        }
                        let wanted = (pos - drag.start_pos) / container * 100.0;
                        ctx.panels
                            .with_mut(|panels| {
                                resize_pair_from(
                                    panels,
                                    drag.handle_index,
                                    drag.start_prev_size,
                                    drag.start_next_size,
                                    wanted,
                                );
                            });
                    },
                    onpointerup: move |_| ctx.drag.set(None),
                }
            }
        }
    }
}

/// A single panel within a [`ResizablePanelGroup`]. `index` must match this
/// panel's position among its siblings (0-based); `default_size`/`min_size`/
/// `max_size` are percentages of the group's own size along its resize
/// axis. Sizes are shared group state driven by dragging, not by this
/// panel's own props after the first render -- `default_size` only seeds
/// the group's size table the first time this index is seen.
#[component]
pub fn ResizablePanel(
    index: ReadSignal<usize>,
    #[props(default = 50.0)] default_size: f64,
    #[props(default = 10.0)] min_size: f64,
    #[props(default = 90.0)] max_size: f64,
    children: Element,
    class: Option<String>,
) -> Element {
    let ctx: ResizableContext = use_context();
    let idx = index.cloned();
    use_effect(move || {
        let mut panels = ctx.panels;
        panels.with_mut(|panels| {
            if panels.len() <= idx {
                panels.resize(
                    idx + 1,
                    PanelConstraints {
                        size: default_size,
                        min: min_size,
                        max: max_size,
                    },
                );
            }
        });
    });

    let size = ctx
        .panels
        .read()
        .get(idx)
        .map(|panel| panel.size)
        .unwrap_or(default_size);
    let class = cn(&[
        "overflow-auto",
        scroll_area_visibility_class(false),
        class.as_deref().unwrap_or_default(),
    ]);

    rsx! {
        div { class, style: "flex: 0 0 {size}%;", {children} }
    }
}

/// The draggable divider between the panels at `handle_index` and
/// `handle_index + 1` in a [`ResizablePanelGroup`]. Drag with the pointer,
/// or focus it and use the arrow keys matching the group's
/// [`ResizableDirection`] to step-resize by 5% per press.
#[component]
pub fn ResizableHandle(
    handle_index: ReadSignal<usize>,
    #[props(default)] with_handle: bool,
    class: Option<String>,
) -> Element {
    let mut ctx: ResizableContext = use_context();
    let idx = handle_index.cloned();
    let direction = ctx.direction;

    let axis_class = match direction {
        ResizableDirection::Horizontal => "w-px cursor-col-resize",
        ResizableDirection::Vertical => "h-px w-full cursor-row-resize",
    };
    let class = cn(&[
        "relative flex shrink-0 items-center justify-center bg-border outline-none focus-visible:ring-2 focus-visible:ring-ring",
        axis_class,
        class.as_deref().unwrap_or_default(),
    ]);

    let onpointerdown = move |event: Event<PointerData>| {
        event.prevent_default();
        let panels = ctx.panels.peek();
        let (Some(prev), Some(next)) = (panels.get(idx).copied(), panels.get(idx + 1).copied())
        else {
            return;
        };
        drop(panels);
        let point = event.client_coordinates();
        let pos = match direction {
            ResizableDirection::Horizontal => point.x,
            ResizableDirection::Vertical => point.y,
        };
        ctx.drag.set(Some(DragState {
            handle_index: idx,
            start_pos: pos,
            start_prev_size: prev.size,
            start_next_size: next.size,
        }));
    };

    let onkeydown = move |event: Event<KeyboardData>| {
        let forward = match (direction, event.key()) {
            (ResizableDirection::Horizontal, Key::ArrowRight) => true,
            (ResizableDirection::Horizontal, Key::ArrowLeft) => false,
            (ResizableDirection::Vertical, Key::ArrowDown) => true,
            (ResizableDirection::Vertical, Key::ArrowUp) => false,
            _ => return,
        };
        event.prevent_default();
        let step = if forward {
            KEYBOARD_STEP_PERCENT
        } else {
            -KEYBOARD_STEP_PERCENT
        };
        ctx.panels.with_mut(|panels| {
            let (Some(prev), Some(next)) = (panels.get(idx).copied(), panels.get(idx + 1).copied())
            else {
                return;
            };
            resize_pair_from(panels, idx, prev.size, next.size, step);
        });
    };

    rsx! {
        div {
            class,
            role: "separator",
            "aria-orientation": match direction {
                ResizableDirection::Horizontal => "vertical",
                ResizableDirection::Vertical => "horizontal",
            },
            tabindex: "0",
            onpointerdown,
            onkeydown,
            if with_handle {
                div { class: "z-10 flex h-4 w-3 items-center justify-center rounded-xs border bg-border" }
            }
        }
    }
}
