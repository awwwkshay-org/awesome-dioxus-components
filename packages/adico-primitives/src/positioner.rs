// SPDX-License-Identifier: MIT OR Apache-2.0

//! A shared, collision-aware anchored-placement engine (Base UI's
//! `Positioner`/`Arrow` anatomy — see design.md §8a) so every popup-shaped
//! component (popover, hover-card, tooltip, select, combobox, dropdown-menu,
//! context-menu, menubar) can compose one placement implementation instead
//! of reimplementing it.
//!
//! [`compute_position`] is pure arithmetic — no DOM, fully unit-testable —
//! taking an anchor rect, floating-content size, and viewport size, and
//! returning where to place the floating content plus which side/align it
//! actually used (a `side` may flip to its opposite when the preferred side
//! doesn't have room; the cross axis is clamped to stay in the viewport
//! rather than flipping `align`, matching Floating UI's `shift` behavior).
//!
//! **Scope note:** this only computes a position once, from rects the caller
//! measured (e.g. via `MountedData::get_client_rect()` in an `onmounted`
//! callback — a one-shot, non-eval measurement already used elsewhere in
//! this crate, such as `move_interaction.rs`). It deliberately does **not**
//! continuously reposition on scroll/resize via ResizeObserver/
//! IntersectionObserver/MutationObserver bridges: those need a long-lived,
//! repeatedly-firing browser listener, the exact pattern
//! `provenance/records/adico-primitives-wave3-overlays.json` documents as
//! non-functional via `document::eval` in this Dioxus 0.7.9/0.7.10 web
//! runtime, and no native Dioxus event exists for arbitrary-element resize
//! or intersection the way `use_escape_key` (task 7.4d) found for keydown.
//! That observer-bridge capability remains unimplemented and unverified,
//! tracked as a follow-up rather than built unverifiable.

use std::rc::Rc;

use dioxus::html::geometry::Pixels;
use dioxus::html::geometry::euclid::Rect;
use dioxus::prelude::*;

use crate::{ContentAlign, ContentSide};

/// A resolved placement: where to put the floating content, and which
/// side/align it actually used (either may differ from what the caller
/// requested, if collision handling adjusted it).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub side: ContentSide,
    pub align: ContentAlign,
}

/// Compute where to place a `floating_width` x `floating_height` box
/// relative to `anchor`, inside a `viewport_width` x `viewport_height`
/// viewport (both measured from the viewport's own top-left origin, matching
/// `getBoundingClientRect()`'s coordinate space).
///
/// `side`/`align` are the caller's preference. If the preferred `side`
/// doesn't have `floating_size + offset` of room and the opposite side has
/// more, the placement flips to the opposite side. The cross axis (the axis
/// `align` controls) is then clamped to stay within the viewport, inset by
/// `collision_padding`, rather than changing `align`.
#[allow(clippy::too_many_arguments)]
pub fn compute_position(
    anchor: Rect<f64, Pixels>,
    floating_width: f64,
    floating_height: f64,
    viewport_width: f64,
    viewport_height: f64,
    side: ContentSide,
    align: ContentAlign,
    offset: f64,
    collision_padding: f64,
) -> Position {
    let side = resolve_side(
        anchor,
        floating_width,
        floating_height,
        viewport_width,
        viewport_height,
        side,
        offset,
    );
    let (x, y) = place(anchor, floating_width, floating_height, side, align, offset);
    let (x, y) = match side {
        ContentSide::Top | ContentSide::Bottom => (
            clamp_cross(x, floating_width, viewport_width, collision_padding),
            y,
        ),
        ContentSide::Left | ContentSide::Right => (
            x,
            clamp_cross(y, floating_height, viewport_height, collision_padding),
        ),
    };
    Position { x, y, side, align }
}

/// The space available between `anchor`'s edge and the viewport edge on the
/// given `side`.
fn space_for_side(
    anchor: Rect<f64, Pixels>,
    viewport_width: f64,
    viewport_height: f64,
    side: ContentSide,
) -> f64 {
    match side {
        ContentSide::Top => anchor.origin.y,
        ContentSide::Bottom => viewport_height - (anchor.origin.y + anchor.size.height),
        ContentSide::Left => anchor.origin.x,
        ContentSide::Right => viewport_width - (anchor.origin.x + anchor.size.width),
    }
}

fn resolve_side(
    anchor: Rect<f64, Pixels>,
    floating_width: f64,
    floating_height: f64,
    viewport_width: f64,
    viewport_height: f64,
    preferred: ContentSide,
    offset: f64,
) -> ContentSide {
    let needed = match preferred {
        ContentSide::Top | ContentSide::Bottom => floating_height + offset,
        ContentSide::Left | ContentSide::Right => floating_width + offset,
    };
    let available = space_for_side(anchor, viewport_width, viewport_height, preferred);
    if available >= needed {
        return preferred;
    }
    let opposite = preferred.opposite();
    let opposite_available = space_for_side(anchor, viewport_width, viewport_height, opposite);
    if opposite_available > available {
        opposite
    } else {
        preferred
    }
}

fn place(
    anchor: Rect<f64, Pixels>,
    floating_width: f64,
    floating_height: f64,
    side: ContentSide,
    align: ContentAlign,
    offset: f64,
) -> (f64, f64) {
    match side {
        ContentSide::Top => (
            align_cross(anchor.origin.x, anchor.size.width, floating_width, align),
            anchor.origin.y - floating_height - offset,
        ),
        ContentSide::Bottom => (
            align_cross(anchor.origin.x, anchor.size.width, floating_width, align),
            anchor.origin.y + anchor.size.height + offset,
        ),
        ContentSide::Left => (
            anchor.origin.x - floating_width - offset,
            align_cross(anchor.origin.y, anchor.size.height, floating_height, align),
        ),
        ContentSide::Right => (
            anchor.origin.x + anchor.size.width + offset,
            align_cross(anchor.origin.y, anchor.size.height, floating_height, align),
        ),
    }
}

fn align_cross(anchor_pos: f64, anchor_size: f64, floating_size: f64, align: ContentAlign) -> f64 {
    match align {
        ContentAlign::Start => anchor_pos,
        ContentAlign::Center => anchor_pos + anchor_size / 2.0 - floating_size / 2.0,
        ContentAlign::End => anchor_pos + anchor_size - floating_size,
    }
}

fn clamp_cross(pos: f64, floating_size: f64, viewport_size: f64, padding: f64) -> f64 {
    let max = (viewport_size - floating_size - padding).max(padding);
    let min = padding.min(max);
    pos.clamp(min, max)
}

/// Given a resolved [`Position`], compute where along the floating box's
/// cross axis to place an arrow of `arrow_size` so it points at `anchor`'s
/// center — clamped so the arrow never renders outside the floating box's
/// own bounds. The returned value is an offset from the floating box's
/// top-left corner (a `left` for a `Top`/`Bottom` placement, or a `top` for
/// a `Left`/`Right` placement).
pub fn arrow_offset(
    anchor: Rect<f64, Pixels>,
    position: Position,
    floating_width: f64,
    floating_height: f64,
    arrow_size: f64,
) -> f64 {
    let (anchor_center, own_pos, floating_size) = match position.side {
        ContentSide::Top | ContentSide::Bottom => (
            anchor.origin.x + anchor.size.width / 2.0,
            position.x,
            floating_width,
        ),
        ContentSide::Left | ContentSide::Right => (
            anchor.origin.y + anchor.size.height / 2.0,
            position.y,
            floating_height,
        ),
    };
    let local = anchor_center - own_pos;
    let half = arrow_size / 2.0;
    let max = (floating_size - half).max(half);
    local.clamp(half, max)
}

/// Measurement shared by [`Positioner`] with its [`Arrow`] child: the
/// anchor's rect and the floating content's own size, alongside the
/// resolved [`Position`] once computed.
#[derive(Clone, Copy, Default)]
struct PositionerCtx {
    anchor: Signal<Option<Rect<f64, Pixels>>>,
    position: Signal<Option<Position>>,
    floating_size: Signal<(f64, f64)>,
}

/// Fetch `anchor_id`'s `getBoundingClientRect()` plus the current viewport
/// size, in one round trip. A one-shot request/response `document::eval`
/// call (send one value, receive one value, done) — not the long-lived
/// listen-repeatedly pattern documented as non-functional on `web` (see this
/// module's own doc comment).
#[cfg(any(feature = "web", feature = "native"))]
async fn measure_anchor_and_viewport(anchor_id: &str) -> Option<(Rect<f64, Pixels>, f64, f64)> {
    let mut eval = dioxus_document::eval(
        "const anchorId = await dioxus.recv();
        const anchor = document.getElementById(anchorId);
        const rect = anchor
            ? anchor.getBoundingClientRect()
            : { x: 0, y: 0, width: 0, height: 0 };
        dioxus.send([rect.x, rect.y, rect.width, rect.height, window.innerWidth, window.innerHeight]);",
    );
    let _ = eval.send(anchor_id);
    let (x, y, width, height, viewport_width, viewport_height): (f64, f64, f64, f64, f64, f64) =
        eval.recv().await.ok()?;
    Some((
        Rect::new((x, y).into(), (width, height).into()),
        viewport_width,
        viewport_height,
    ))
}

/// A no-op on targets without a DOM (SSR/native without the `web`/`native`
/// feature).
#[cfg(not(any(feature = "web", feature = "native")))]
async fn measure_anchor_and_viewport(_anchor_id: &str) -> Option<(Rect<f64, Pixels>, f64, f64)> {
    None
}

/// The props for the [`Positioner`] component.
#[derive(Props, Clone, PartialEq)]
pub struct PositionerProps {
    /// The `id` of this positioner's own rendered element (not the anchor).
    /// Lets a caller compose `Positioner` as the floating content's own root
    /// element — e.g. so `use_animated_open`/`use_outside_dismiss` can target
    /// it — instead of nesting another element inside for that purpose.
    #[props(default)]
    pub id: Option<String>,
    /// The `id` of the anchor element to position relative to.
    pub anchor_id: ReadSignal<String>,
    /// The preferred side; may flip to its opposite if there's no room.
    #[props(default = ContentSide::Bottom)]
    pub side: ContentSide,
    /// The preferred alignment along the cross axis.
    #[props(default = ContentAlign::Center)]
    pub align: ContentAlign,
    /// Gap, in pixels, between the anchor and the floating content.
    #[props(default = 0.0)]
    pub offset: f64,
    /// Minimum gap, in pixels, kept between the floating content and the
    /// viewport edge when the cross axis is clamped.
    #[props(default = 8.0)]
    pub collision_padding: f64,
    /// Called when the pointer enters the positioned element — for
    /// hover-triggered content (hover-card, tooltip) that should stay open
    /// while the pointer is over the floating box itself, not just the
    /// anchor. `#[props(extends = GlobalAttributes)]` does not cover event
    /// listeners on a *component* call the way it does on a plain html
    /// element, so this needs to be a dedicated prop rather than passed
    /// through `attributes`.
    #[props(default)]
    pub on_mouse_enter: Callback<Event<MouseData>>,
    /// Called when the pointer leaves the positioned element. See
    /// `on_mouse_enter`.
    #[props(default)]
    pub on_mouse_leave: Callback<Event<MouseData>>,
    /// Called once the positioned element mounts, alongside `Positioner`'s
    /// own internal use of `onmounted` to measure and place it — lets a
    /// caller obtain the same `MountedData` (e.g. to manage its own focus)
    /// without nesting another element inside `Positioner` just to get an
    /// `onmounted` of its own. See `on_mouse_enter`'s doc comment for why
    /// this needs to be a dedicated prop rather than passed through
    /// `attributes`.
    #[props(default)]
    pub on_mounted: Callback<Event<MountedData>>,
    /// Called on a `keydown` targeting the positioned element or one of its
    /// descendants. See `on_mouse_enter`'s doc comment for why this needs to
    /// be a dedicated prop.
    #[props(default)]
    pub on_keydown: Callback<Event<KeyboardData>>,
    /// Called when focus leaves the positioned element or one of its
    /// descendants. See `on_mouse_enter`'s doc comment for why this needs to
    /// be a dedicated prop.
    #[props(default)]
    pub on_blur: Callback<Event<FocusData>>,
    /// Called on `pointerdown` targeting the positioned element or one of
    /// its descendants — e.g. so a combobox's listbox can
    /// `event.prevent_default()` to keep focus in its input rather than
    /// having it stolen by the pointer interaction. See `on_mouse_enter`'s
    /// doc comment for why this needs to be a dedicated prop.
    #[props(default)]
    pub on_pointer_down: Callback<Event<PointerData>>,
    /// Additional attributes for the positioned element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The floating content, and optionally an [`Arrow`].
    pub children: Element,
}

/// # Positioner
///
/// Anchors its children to the element identified by `anchor_id`, computing
/// placement once when it mounts via [`compute_position`]. See this module's
/// doc comment for what this does and does not cover (no continuous
/// repositioning on scroll/resize). SSR-safe: renders its children in a
/// fixed-position container with no computed offset yet, since there is no
/// DOM to measure on the server.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::positioner::{Positioner, Arrow};
/// use adico_primitives::{ContentAlign, ContentSide};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         button { id: "my-anchor", "Trigger" }
///         Positioner {
///             anchor_id: "my-anchor",
///             side: ContentSide::Bottom,
///             align: ContentAlign::Center,
///             "Floating content"
///             Arrow {}
///         }
///     }
/// }
/// ```
#[component]
pub fn Positioner(props: PositionerProps) -> Element {
    let mut ctx = use_context_provider(PositionerCtx::default);
    let anchor_id = props.anchor_id;
    let side = props.side;
    let align = props.align;
    let offset = props.offset;
    let collision_padding = props.collision_padding;
    let mut floating_ref: Signal<Option<Rc<MountedData>>> = use_signal(|| None);

    let recompute = move || {
        spawn(async move {
            let Some(floating) = floating_ref() else {
                return;
            };
            let Ok(floating_rect) = floating.get_client_rect().await else {
                return;
            };
            let anchor_id = anchor_id.cloned();
            let Some((anchor_rect, viewport_width, viewport_height)) =
                measure_anchor_and_viewport(&anchor_id).await
            else {
                return;
            };
            ctx.anchor.set(Some(anchor_rect));
            ctx.floating_size
                .set((floating_rect.size.width, floating_rect.size.height));
            ctx.position.set(Some(compute_position(
                anchor_rect,
                floating_rect.size.width,
                floating_rect.size.height,
                viewport_width,
                viewport_height,
                side,
                align,
                offset,
                collision_padding,
            )));
        });
    };

    let position = ctx.position;
    let style = match position() {
        // `visibility: visible` is explicit, not the default omitted, on
        // purpose: found live (task 5.4) that once the `None` branch below
        // has rendered `visibility: hidden` at least once, Dioxus's `web`
        // style-attribute patching only sets/updates properties present in
        // the *new* style string -- it does not clear a property that
        // existed in the previous string but is absent from the new one.
        // Omitting `visibility` here left it permanently stuck at `hidden`
        // even once `position` genuinely became `Some` with a real,
        // correctly-computed offset -- every `Positioner`-anchored surface
        // (popover, hover-card, tooltip, select, combobox) opened logically
        // (correct ARIA, correct computed left/top) but stayed invisible.
        Some(p) => format!(
            "position: fixed; left: {}px; top: {}px; visibility: visible;",
            p.x, p.y
        ),
        None => "position: fixed; visibility: hidden;".to_string(),
    };

    rsx! {
        div {
            id: props.id.clone(),
            style,
            "data-side": position().map(|p| p.side.as_str()),
            "data-align": position().map(|p| p.align.as_str()),
            onmounted: move |evt: Event<MountedData>| {
                floating_ref.set(Some(evt.data()));
                recompute();
                props.on_mounted.call(evt);
            },
            onmouseenter: move |event| props.on_mouse_enter.call(event),
            onmouseleave: move |event| props.on_mouse_leave.call(event),
            onkeydown: move |event| props.on_keydown.call(event),
            onblur: move |event| props.on_blur.call(event),
            onpointerdown: move |event| props.on_pointer_down.call(event),
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`Arrow`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ArrowProps {
    /// The arrow's size in pixels, used to keep it clear of the floating
    /// box's own corners.
    #[props(default = 8.0)]
    pub size: f64,
    /// Additional attributes for the arrow element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # Arrow
///
/// Renders a small element inside a [`Positioner`], offset along the cross
/// axis so it points at the anchor's center. Must be used inside a
/// [`Positioner`]. Renders with no offset until the positioner has computed
/// a placement (including under SSR, where it never will).
#[component]
pub fn Arrow(props: ArrowProps) -> Element {
    let ctx: PositionerCtx = use_context();
    let offset = match ((ctx.anchor)(), (ctx.position)()) {
        (Some(anchor), Some(position)) => {
            let (floating_width, floating_height) = (ctx.floating_size)();
            Some(arrow_offset(
                anchor,
                position,
                floating_width,
                floating_height,
                props.size,
            ))
        }
        _ => None,
    };

    let style = match ((ctx.position)(), offset) {
        (Some(position), Some(offset)) => match position.side {
            ContentSide::Top | ContentSide::Bottom => {
                format!("position: absolute; left: {offset}px;")
            }
            ContentSide::Left | ContentSide::Right => {
                format!("position: absolute; top: {offset}px;")
            }
        },
        _ => "position: absolute;".to_string(),
    };

    rsx! {
        span {
            style,
            ..props.attributes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect<f64, Pixels> {
        Rect::new((x, y).into(), (width, height).into())
    }

    const VIEWPORT_WIDTH: f64 = 800.0;
    const VIEWPORT_HEIGHT: f64 = 600.0;

    #[test]
    fn places_below_the_anchor_when_bottom_is_preferred_and_fits() {
        let anchor = rect(100.0, 100.0, 50.0, 20.0);
        let position = compute_position(
            anchor,
            120.0,
            40.0,
            VIEWPORT_WIDTH,
            VIEWPORT_HEIGHT,
            ContentSide::Bottom,
            ContentAlign::Start,
            8.0,
            8.0,
        );

        assert_eq!(position.side, ContentSide::Bottom);
        assert_eq!(position.x, 100.0);
        assert_eq!(position.y, 100.0 + 20.0 + 8.0);
    }

    #[test]
    fn flips_to_bottom_when_top_has_no_room() {
        // Anchor sits 10px from the top of the viewport; a 200px-tall
        // floating box can't fit above it, but there's plenty of room below.
        let anchor = rect(100.0, 10.0, 50.0, 20.0);
        let position = compute_position(
            anchor,
            120.0,
            200.0,
            VIEWPORT_WIDTH,
            VIEWPORT_HEIGHT,
            ContentSide::Top,
            ContentAlign::Start,
            8.0,
            8.0,
        );

        assert_eq!(position.side, ContentSide::Bottom);
        assert_eq!(position.y, 10.0 + 20.0 + 8.0);
    }

    #[test]
    fn keeps_the_preferred_side_when_neither_side_has_more_room() {
        // A floating box taller than the whole viewport never fits either
        // way; keep the caller's preference rather than flipping pointlessly.
        let anchor = rect(100.0, 300.0, 50.0, 20.0);
        let position = compute_position(
            anchor,
            120.0,
            10_000.0,
            VIEWPORT_WIDTH,
            VIEWPORT_HEIGHT,
            ContentSide::Top,
            ContentAlign::Start,
            8.0,
            8.0,
        );

        assert_eq!(position.side, ContentSide::Top);
    }

    #[test]
    fn center_align_centers_on_the_anchor_before_clamping() {
        let anchor = rect(300.0, 100.0, 40.0, 20.0);
        let position = compute_position(
            anchor,
            100.0,
            40.0,
            VIEWPORT_WIDTH,
            VIEWPORT_HEIGHT,
            ContentSide::Bottom,
            ContentAlign::Center,
            0.0,
            8.0,
        );

        // Anchor center is at 320; a 100-wide box centered on it starts at 270.
        assert_eq!(position.x, 270.0);
    }

    #[test]
    fn end_align_right_edges_align_with_the_anchors_end() {
        let anchor = rect(300.0, 100.0, 40.0, 20.0);
        let position = compute_position(
            anchor,
            100.0,
            40.0,
            VIEWPORT_WIDTH,
            VIEWPORT_HEIGHT,
            ContentSide::Bottom,
            ContentAlign::End,
            0.0,
            8.0,
        );

        assert_eq!(position.x, 300.0 + 40.0 - 100.0);
    }

    #[test]
    fn clamps_the_cross_axis_instead_of_overflowing_the_viewport_edge() {
        // Anchor near the right edge; a wide, end-aligned floating box would
        // overflow past the viewport's right edge without clamping.
        let anchor = rect(780.0, 100.0, 20.0, 20.0);
        let position = compute_position(
            anchor,
            200.0,
            40.0,
            VIEWPORT_WIDTH,
            VIEWPORT_HEIGHT,
            ContentSide::Bottom,
            ContentAlign::End,
            0.0,
            8.0,
        );

        assert_eq!(position.x, VIEWPORT_WIDTH - 200.0 - 8.0);
    }

    #[test]
    fn left_and_right_sides_place_along_the_horizontal_axis() {
        let anchor = rect(400.0, 100.0, 40.0, 20.0);
        let position = compute_position(
            anchor,
            80.0,
            30.0,
            VIEWPORT_WIDTH,
            VIEWPORT_HEIGHT,
            ContentSide::Right,
            ContentAlign::Start,
            5.0,
            8.0,
        );

        assert_eq!(position.side, ContentSide::Right);
        assert_eq!(position.x, 400.0 + 40.0 + 5.0);
        assert_eq!(position.y, 100.0);
    }

    #[test]
    fn arrow_offset_points_at_the_anchor_center_for_a_bottom_placement() {
        let anchor = rect(300.0, 100.0, 40.0, 20.0);
        let position = Position {
            x: 270.0,
            y: 128.0,
            side: ContentSide::Bottom,
            align: ContentAlign::Center,
        };

        // Anchor center is at 320; the floating box starts at 270, so the
        // arrow should be 50px into the floating box.
        assert_eq!(arrow_offset(anchor, position, 100.0, 40.0, 10.0), 50.0);
    }

    #[test]
    fn arrow_offset_clamps_within_the_floating_boxs_own_bounds() {
        // The anchor's center falls outside the floating box's span (e.g.
        // after cross-axis clamping moved the box away from the anchor); the
        // arrow must still stay inside the box, not point off its edge.
        let anchor = rect(780.0, 100.0, 20.0, 20.0);
        let position = Position {
            x: 592.0,
            y: 128.0,
            side: ContentSide::Bottom,
            align: ContentAlign::End,
        };

        let offset = arrow_offset(anchor, position, 200.0, 40.0, 10.0);
        assert!((5.0..=195.0).contains(&offset));
    }
}
