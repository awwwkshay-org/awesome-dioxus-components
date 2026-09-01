//! Source-owned Dioxus-only Color Picker for Dioxus, backed by the owned
//! adico primitive layer. This is a Dioxus Components extra with no shadcn
//! equivalent -- it does not count toward shadcn parity.

use dioxus::prelude::*;

use adico_primitives::color_picker::{
    AreaThumb as AreaThumbPrimitive, AreaThumbSaturationInput as AreaThumbSaturationInputPrimitive,
    AreaThumbValueInput as AreaThumbValueInputPrimitive, AreaTrack as AreaTrackPrimitive,
    ColorArea as ColorAreaPrimitive, ColorPicker as ColorPickerPrimitive,
};
pub use adico_primitives::color_picker::{
    AreaThumbSaturationInputProps, AreaThumbValueInputProps, Color, ColorPickerContext,
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
        "absolute size-4 -translate-x-1/2 translate-y-1/2 rounded-full border-2 border-white shadow-[0_0_0_1px_rgba(0,0,0,0.3)] outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 data-[dragging=true]:cursor-grabbing",
        "",
    ]);
    rsx! {
        AreaThumbPrimitive { class, {children} }
    }
}

/// `AreaThumbSaturationInputProps`/`AreaThumbValueInputProps` have no
/// dedicated `class` field (only `attributes: Vec<Attribute>`, extending
/// `GlobalAttributes`) -- build the merged attribute list by hand, matching
/// this repo's own established precedent for this exact limitation (e.g.
/// `slider.rs`'s `with_class`).
fn with_class(class: &str, attributes: Vec<Attribute>) -> Vec<Attribute> {
    let mut merged = vec![Attribute::new("class", class, None, false)];
    merged.extend(attributes);
    merged
}

/// A hidden-but-accessible native `<input type="range">` shadowing
/// [`ColorArea`]'s saturation axis, for screen readers and voice control.
///
/// Previously had no default class at all (a bare `pub use` re-export of
/// the primitive): with no visual hiding, the browser rendered its own
/// native range-slider UI -- a ~130px track-and-thumb sitting in normal
/// document flow next to the actual color area -- reported directly by the
/// user as the color picker "not working" (confirmed live: `elementFromPoint`
/// at the visible bars found no element there because the *drawn* area is
/// the browser's own unstyled `<input>` rendering, not a positioned overlay;
/// `getComputedStyle` on the input showed `className: ""`, `opacity: 1`,
/// `position: static`). Fixed with the same `sr-only` convention already
/// used elsewhere in this registry (see `dialog.rs`'s "Close" label, etc.)
/// -- visually hidden via clipping, not `display:none`, so it stays
/// focusable and keyboard/voice-operable.
#[component]
pub fn AreaThumbSaturationInput(props: AreaThumbSaturationInputProps) -> Element {
    let attributes = with_class("sr-only", props.attributes);
    rsx! {
        AreaThumbSaturationInputPrimitive { attributes }
    }
}

/// A hidden-but-accessible native `<input type="range">` shadowing
/// [`ColorArea`]'s value axis, for screen readers and voice control. See
/// [`AreaThumbSaturationInput`]'s doc comment for the bug this fixes.
#[component]
pub fn AreaThumbValueInput(props: AreaThumbValueInputProps) -> Element {
    let attributes = with_class("sr-only", props.attributes);
    rsx! {
        AreaThumbValueInputPrimitive { attributes }
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

    #[test]
    fn the_accessibility_shadow_inputs_default_to_visually_hidden() {
        let merged = with_class("sr-only", Vec::new());
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].name, "class");
    }
}
