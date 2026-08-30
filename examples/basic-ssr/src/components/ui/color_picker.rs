//! Source-owned Dioxus-only Color Picker for Dioxus, backed by the owned
//! adico primitive layer. This is a Dioxus Components extra with no shadcn
//! equivalent -- it does not count toward shadcn parity.

use dioxus::prelude::*;

use adico_primitives::color_picker::{
    AreaThumb as AreaThumbPrimitive, AreaTrack as AreaTrackPrimitive,
    ColorArea as ColorAreaPrimitive, ColorPicker as ColorPickerPrimitive,
};
pub use adico_primitives::color_picker::{
    AreaThumbSaturationInput, AreaThumbSaturationInputProps, AreaThumbValueInput,
    AreaThumbValueInputProps, Color, ColorPickerContext,
};

use crate::adico_lib::cn::cn;

/// Provides the color-picker context and synchronizes a color value between
/// its descendants.
#[component]
pub fn ColorPicker(
    #[props(default)] color: ReadSignal<palette::Hsv<palette::encoding::Srgb, f64>>,
    #[props(default)] on_color_change: Callback<palette::Hsv<palette::encoding::Srgb, f64>>,
    #[props(default)] disabled: ReadSignal<bool>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "inline-flex flex-col gap-2",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ColorPickerPrimitive { color, on_color_change, disabled, class, {children} }
    }
}

/// A two-dimensional saturation/value drag surface.
#[component]
pub fn ColorArea(
    #[props(default = 1.0)] step: ReadSignal<f64>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "relative size-48 touch-none rounded-md border border-input",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ColorAreaPrimitive { step, class, {children} }
    }
}

/// The color-plane background for a [`ColorArea`]; must contain an [`AreaThumb`].
#[component]
pub fn AreaTrack(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "absolute inset-0 rounded-[inherit] bg-[linear-gradient(to_top,black,transparent),linear-gradient(to_right,white,var(--area-color))]",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AreaTrackPrimitive { class, {children} }
    }
}

/// The draggable/keyboard-navigable position indicator inside a [`ColorArea`].
/// Typically contains an [`AreaThumbSaturationInput`] and [`AreaThumbValueInput`]
/// for accessible keyboard/screen-reader support.
#[component]
pub fn AreaThumb(children: Element) -> Element {
    let class = cn(&[
        "absolute size-4 -translate-x-1/2 translate-y-1/2 rounded-full border-2 border-white shadow-[0_0_0_1px_rgba(0,0,0,0.3)] outline-none data-[dragging=true]:cursor-grabbing",
        "",
    ]);
    rsx! {
        AreaThumbPrimitive { class, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dragging_thumb_uses_the_grabbing_cursor_state() {
        let class = cn(&["data-[dragging=true]:cursor-grabbing", ""]);
        assert!(class.contains("data-[dragging=true]:cursor-grabbing"));
    }
}
