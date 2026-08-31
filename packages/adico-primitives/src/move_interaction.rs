// No ARIA pattern applies to this low-level drag/keyboard-movement primitive; its spec is
// `crate::direction`'s existing RTL contract (arrow-key handedness) plus its actual consumers'
// needs (`slider`, `color_picker`). Its `get_client_rect()` calls only run from event/mount
// callbacks (never during initial render), so no target gating is needed beyond what
// `pointer.rs` already applies to the global position registry this depends on for continuous
// drag tracking — see that file's doc comment for the still-unconfirmed-in-a-live-browser
// finding this inherits (`gesture.rs`'s doc comment first recorded it).

//! Shared pointer-drag and arrow-key movement for track-style controls
//! (slider, color picker) via [`use_move_interaction`], built on
//! [`crate::pointer`]'s global position registry.
//!
//! This is one of three overlapping pointer/gesture helpers in the crate
//! (alongside [`crate::pointer`] and `selectable::pointer_select_*`); a
//! unified press/long-press/drag primitive consolidating all three is
//! tracked separately (see design.md §8a).

use crate::direction::Direction;
use crate::pointer;
use dioxus::html::geometry::ClientPoint;
use dioxus::html::geometry::Pixels;
use dioxus::html::geometry::euclid::Rect;
use dioxus::html::geometry::euclid::Vector2D;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use std::rc::Rc;

/// Keyboard modifier state attached to a move event.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MoveModifiers {
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub meta_key: bool,
    pub shift_key: bool,
}

/// A normalized movement delta.
///
/// Pointer deltas are reported in CSS pixels. Keyboard deltas use the caller's
/// provided step value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoveEvent {
    pub delta_x: f64,
    pub delta_y: f64,
    pub modifiers: MoveModifiers,
}

impl MoveEvent {
    /// Build a movement delta from an arrow-key press. `direction` flips
    /// which physical key (`ArrowLeft`/`ArrowRight`) produces a positive
    /// `delta_x`, so a caller's "next" direction stays consistent between
    /// LTR and RTL layouts; `ArrowUp`/`ArrowDown` are unaffected, since
    /// vertical axes have no handedness.
    pub fn from_keyboard(
        event: &Event<KeyboardData>,
        step: f64,
        direction: Direction,
    ) -> Option<Self> {
        let modifiers = event.data().modifiers();
        let modifiers = MoveModifiers {
            alt_key: modifiers.alt(),
            ctrl_key: modifiers.ctrl(),
            meta_key: modifiers.meta(),
            shift_key: modifiers.shift(),
        };
        let delta = if modifiers.shift_key {
            step * 10.0
        } else {
            step
        };
        let rtl = direction.is_rtl();

        let (delta_x, delta_y) = match event.data().key() {
            Key::ArrowUp => (0.0, delta),
            Key::ArrowDown => (0.0, -delta),
            Key::ArrowRight => (if rtl { -delta } else { delta }, 0.0),
            Key::ArrowLeft => (if rtl { delta } else { -delta }, 0.0),
            _ => return None,
        };

        Some(Self {
            delta_x,
            delta_y,
            modifiers,
        })
    }
}

/// Shared movement state for controls that support pointer dragging and arrow keys.
#[derive(Clone, Copy)]
pub struct MoveInteraction {
    rect: Signal<Option<Rect<f64, Pixels>>>,
    element: Signal<Option<Rc<MountedData>>>,
    active_pointer_id: Signal<Option<i32>>,
    last_pointer_position: CopyValue<Option<ClientPoint>>,
    dragging: Signal<bool>,
}

pub fn use_move_interaction(dragging: Signal<bool>) -> MoveInteraction {
    let rect = use_signal(|| None);
    let element = use_signal(|| None);
    let active_pointer_id = use_signal(|| None);
    let last_pointer_position = use_hook(|| CopyValue::new(None::<ClientPoint>));

    MoveInteraction {
        rect,
        element,
        active_pointer_id,
        last_pointer_position,
        dragging,
    }
}

impl MoveInteraction {
    pub fn rect(&self) -> Option<Rect<f64, Pixels>> {
        self.rect.cloned()
    }

    pub async fn set_mounted(&mut self, mounted: Rc<MountedData>) {
        if let Ok(rect) = mounted.get_client_rect().await {
            self.rect.set(Some(rect));
        }
        self.element.set(Some(mounted));
    }

    pub async fn refresh_rect(&mut self) -> Option<Rect<f64, Pixels>> {
        let element = (self.element)()?;

        if let Ok(rect) = element.get_client_rect().await {
            self.rect.set(Some(rect));
            Some(rect)
        } else {
            None
        }
    }

    pub fn start_pointer(&mut self, event: &Event<PointerData>) -> bool {
        event.prevent_default();
        event.stop_propagation();

        if self.active_pointer_id.read().is_some()
            || event.trigger_button() != Some(MouseButton::Primary)
        {
            return false;
        }

        let pointer_id = event.data().pointer_id();
        self.active_pointer_id.set(Some(pointer_id));
        pointer::track_pointer_down(pointer_id, event.client_coordinates());
        true
    }

    pub fn pointer_move(&mut self) -> Option<MoveEvent> {
        if !(self.dragging)() {
            return None;
        }

        let active_pointer_id = (self.active_pointer_id)()?;
        let Some(pointer_position) = pointer::pointer_position(active_pointer_id) else {
            self.end_pointer();
            return None;
        };

        let delta = if let Some(last_position) =
            self.last_pointer_position.replace(Some(pointer_position))
        {
            pointer_position - last_position
        } else {
            Vector2D::zero()
        };

        Some(MoveEvent {
            delta_x: delta.x,
            delta_y: delta.y,
            modifiers: MoveModifiers::default(),
        })
    }

    pub fn end_pointer(&mut self) {
        self.active_pointer_id.take();
        self.last_pointer_position.set(None);
        self.dragging.set(false);
    }
}
