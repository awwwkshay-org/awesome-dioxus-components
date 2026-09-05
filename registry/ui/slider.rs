//! Source-owned shadcn-style Slider for Dioxus, backed by the owned adico
//! primitive layer.

use dioxus::prelude::*;

use adico_primitives::slider::RangeSlider as RangeSliderPrimitive;
pub use adico_primitives::slider::{RangeSliderProps, SliderProps};
use adico_primitives::slider::{
    Slider as SliderPrimitive, SliderRange as SliderRangePrimitive,
    SliderThumb as SliderThumbPrimitive, SliderTrack as SliderTrackPrimitive,
};

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

/// Root shadcn class every `Slider`/`RangeSlider` needs, previously supplied
/// by neither the (bare re-exported) primitive nor this facade: without an
/// explicit `w-full`, the root has no intrinsic width of its own inside a
/// flex-centered demo container, `SliderTrack`'s own `w-full` then resolves
/// against that undetermined width, and the whole control collapses to a
/// zero-width, undraggable point (found live: reported directly by the user
/// as "Slider is non functional" -- confirmed via `getBoundingClientRect()`
/// showing the root at literally `width: 0`).
const SLIDER_ROOT_CLASS: &str = "relative flex w-full touch-none select-none items-center data-[orientation=vertical]:h-full data-[orientation=vertical]:w-auto data-[orientation=vertical]:flex-col";

/// A single-thumb slider with the default adico/shadcn root layout. See
/// [`adico_primitives::slider::Slider`] for the full behavior/prop
/// reference; this facade only adds the root's default class.
#[component]
pub fn Slider(props: SliderProps) -> Element {
    let class = cn(&[
        SLIDER_ROOT_CLASS,
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        SliderPrimitive {
            value: props.value,
            default_value: props.default_value,
            min: props.min,
            max: props.max,
            step: props.step,
            disabled: props.disabled,
            horizontal: props.horizontal,
            inverted: props.inverted,
            on_value_change: props.on_value_change,
            label: props.label,
            class,
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// A two-thumb range slider with the default adico/shadcn root layout. See
/// [`adico_primitives::slider::RangeSlider`] for the full behavior/prop
/// reference; this facade only adds the root's default class.
#[component]
pub fn RangeSlider(props: RangeSliderProps) -> Element {
    let class = cn(&[
        SLIDER_ROOT_CLASS,
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        RangeSliderPrimitive {
            value: props.value,
            default_value: props.default_value,
            min: props.min,
            max: props.max,
            step: props.step,
            disabled: props.disabled,
            horizontal: props.horizontal,
            inverted: props.inverted,
            on_value_change: props.on_value_change,
            label: props.label,
            class,
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// The track a [`Slider`]/[`RangeSlider`]'s thumb(s) move along.
///
/// Deliberately has no `overflow-hidden`: unlike upstream shadcn/Radix,
/// where the thumb is a sibling of the track (both children of the slider
/// root), adico's composition model nests [`SliderThumb`] *inside*
/// [`SliderTrack`] (see the primitive's own doc example). An `overflow-hidden`
/// track would therefore clip the thumb's `size-4` handle, which overflows
/// the track's own `h-1.5`/`w-1.5` cross-axis size by design (the handle
/// must be visibly larger than the track it rides on). [`SliderRange`]
/// carries its own full-radius corner instead, so the filled portion still
/// renders with pill-shaped ends without relying on clipping.
#[component]
pub fn SliderTrack(
    #[props(default = Radius::Full)] radius: Radius,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "relative grow bg-primary/20 data-[orientation=horizontal]:h-1.5 data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full data-[orientation=vertical]:w-1.5",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        SliderTrackPrimitive { class, {children} }
    }
}

/// The filled portion of the [`SliderTrack`] between the minimum and the
/// current value (or between the two thumbs of a [`RangeSlider`]).
#[component]
pub fn SliderRange(
    #[props(default = Radius::Full)] radius: Radius,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "absolute bg-primary data-[orientation=horizontal]:h-full data-[orientation=vertical]:w-full",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        SliderRangePrimitive { class, {children} }
    }
}

/// A draggable/keyboard-movable thumb within a [`SliderTrack`].
#[component]
pub fn SliderThumb(
    index: Option<usize>,
    #[props(default = Radius::Full)] radius: Radius,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "absolute block size-4 shrink-0 -translate-x-1/2 -translate-y-1/2 border border-primary bg-background shadow-sm transition-colors hover:ring-4 hover:ring-ring/50 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 data-[orientation=horizontal]:top-1/2 data-[orientation=vertical]:left-1/2",
        radius.class(),
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

    #[test]
    fn root_has_a_real_width_instead_of_collapsing_to_zero() {
        assert!(SLIDER_ROOT_CLASS.contains("w-full"));
    }

    #[test]
    fn caller_class_is_appended_after_the_root_class() {
        let class = cn(&[SLIDER_ROOT_CLASS, "extra-class"]);
        assert!(class.contains("w-full"));
        assert!(class.ends_with("extra-class"));
    }
}
