//! Source-owned shadcn-style Slider for Dioxus, backed by the owned adico
//! primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::slider::{RangeSlider, Slider};
use adico_primitives::slider::{
    SliderRange as SliderRangePrimitive, SliderThumb as SliderThumbPrimitive,
    SliderTrack as SliderTrackPrimitive,
};

use crate::adico_lib::cn::cn;

/// The track a [`Slider`]/[`RangeSlider`]'s thumb(s) move along.
///
/// Deliberately has no `overflow-hidden`: unlike upstream shadcn/Radix,
/// where the thumb is a sibling of the track (both children of the slider
/// root), adico's composition model nests [`SliderThumb`] *inside*
/// [`SliderTrack`] (see the primitive's own doc example). An `overflow-hidden`
/// track would therefore clip the thumb's `size-4` handle, which overflows
/// the track's own `h-1.5`/`w-1.5` cross-axis size by design (the handle
/// must be visibly larger than the track it rides on). [`SliderRange`]
/// carries its own `rounded-full` instead, so the filled portion still
/// renders with pill-shaped ends without relying on clipping.
#[component]
pub fn SliderTrack(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "relative grow rounded-full bg-primary/20 data-[orientation=horizontal]:h-1.5 data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full data-[orientation=vertical]:w-1.5",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        SliderTrackPrimitive { class, {children} }
    }
}

/// The filled portion of the [`SliderTrack`] between the minimum and the
/// current value (or between the two thumbs of a [`RangeSlider`]).
#[component]
pub fn SliderRange(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "absolute rounded-full bg-primary data-[orientation=horizontal]:h-full data-[orientation=vertical]:w-full",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        SliderRangePrimitive { class, {children} }
    }
}

/// A draggable/keyboard-movable thumb within a [`SliderTrack`].
#[component]
pub fn SliderThumb(index: Option<usize>, class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "absolute block size-4 shrink-0 -translate-x-1/2 -translate-y-1/2 rounded-full border border-primary bg-background shadow-sm transition-colors hover:ring-4 hover:ring-ring/50 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 data-[orientation=horizontal]:top-1/2 data-[orientation=vertical]:left-1/2",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        SliderThumbPrimitive { index, class, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumb_centers_on_its_position_with_a_translate() {
        let class = cn(&["-translate-x-1/2 -translate-y-1/2", ""]);
        assert!(class.contains("-translate-x-1/2"));
        assert!(class.contains("-translate-y-1/2"));
    }
}
