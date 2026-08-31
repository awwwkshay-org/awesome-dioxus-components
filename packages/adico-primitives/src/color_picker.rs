// No dedicated WAI-ARIA APG "Color Picker" pattern exists; this follows the closest APG
// guidance -- the Slider pattern, applied per axis -- composited into a 2D interaction:
// `ColorPicker`/`ColorArea` are `role="group"`, and each of `AreaThumbSaturationInput`/
// `AreaThumbValueInput` is a native `type="range"` input with `aria-valuetext` (a
// human-readable "Saturation 62%, vibrant orange" label, not a bare number, since raw
// saturation/value percentages alone don't convey what's changing) and `aria-orientation`,
// so either axis remains independently operable and announced. `AreaThumb` itself provides the
// two-dimensional drag surface and keyboard entry point, handing focus to whichever axis input
// the keypress moved. Flattened `color_picker/color_naming.rs` into this file (task 5.7) --
// its human-readable Oklch color-naming logic backs each axis input's `aria-valuetext`; no
// logic change, same free functions, now local instead of a submodule. This file was already a
// genuine adaptation, not ported-unmodified, and that adaptation is unchanged here:
//
// Adapted from upstream: `crate::dioxus_elements::geometry::ClientPoint` is an
// upstream-internal re-export path specific to their own crate layout that
// does not exist here (the same class of adaptation Wave 2's accordion.rs
// needed for `crate::dioxus_elements::Key`); replaced with
// `dioxus::html::geometry::ClientPoint`, matching this crate's own
// `move_interaction.rs`. No document::eval, portal, or DOM measurement API
// beyond the already-owned, already target-gated `move_interaction`/
// `pointer` modules (reused unmodified from Wave 2's Slider import), so this
// needed no additional target-gated adapter work of its own. Read the actual
// upstream source (not the M3 migration queue's one-line dependency note,
// which said this item composes label/popover/slider): the ColorPicker
// primitive itself only depends on `move_interaction`, not those three
// registry items -- a real primitive-dependency correction, recorded per the
// M1 lesson to always verify against source.

//! Defines the [`ColorPicker`] component and its sub-components.

use crate::direction::use_direction;
use crate::move_interaction::{MoveEvent, use_move_interaction};
use dioxus::html::geometry::ClientPoint;
use dioxus::html::geometry::PixelsSize;
use dioxus::html::geometry::euclid::Size2D;
use dioxus::prelude::*;
use palette::{FromColor, Hsv, IntoColor, Oklch, RgbHue, Srgb, encoding};

use std::rc::Rc;

/// Represents an sRGB color.
pub type Color = Srgb<u8>;

const COLOR_AREA_MIN: f64 = 0.0;
const COLOR_AREA_MAX: f64 = 100.0;
const COLOR_AREA_RANGE: f64 = COLOR_AREA_MAX - COLOR_AREA_MIN;

fn color_hex(color: Color) -> String {
    format!("#{color:X}")
}

fn area_value_from_hsv(hsv: Hsv<encoding::Srgb, f64>) -> ClientPoint {
    ClientPoint::new(hsv.saturation, hsv.value) * COLOR_AREA_RANGE
}

fn set_area_value(ctx: ColorPickerContext, value: ClientPoint) {
    let scaled = value / COLOR_AREA_RANGE;
    ctx.set_sv(scaled.x, scaled.y);
}

fn snap_area_value(value: ClientPoint, step: f64) -> ClientPoint {
    value.map(|v| (v / step).round() * step)
}

fn clamp_area_value(value: ClientPoint, step: f64) -> ClientPoint {
    let clamped = value.map(|v| v.clamp(COLOR_AREA_MIN, COLOR_AREA_MAX));
    snap_area_value(clamped, step)
}

fn area_percent(value: ClientPoint) -> PixelsSize {
    let scaled = value.map(|v| ((v - COLOR_AREA_MIN) / COLOR_AREA_RANGE * 100.0).clamp(0.0, 100.0));
    PixelsSize::new(scaled.x, scaled.y)
}

// --- Human-readable color naming ---
//
// Converts an sRGB color into Oklch and classifies it into a descriptive label like "very dark
// grayish blue" or "vibrant orange". Flattened in from the former color_picker/color_naming.rs
// submodule (task 5.7); behavior is unchanged.

/// Lightness threshold between orange and brown.
const ORANGE_LIGHTNESS_THRESHOLD: f64 = 0.68;

/// Lightness threshold between pure yellow and "yellow green".
const YELLOW_GREEN_LIGHTNESS_THRESHOLD: f64 = 0.85;

/// The maximum lightness considered to be "dark".
const MAX_DARK_LIGHTNESS: f64 = 0.55;

/// The chroma threshold between gray and color.
const GRAY_THRESHOLD: f64 = 0.001;

/// Build a descriptive name for `color` (e.g. `"vibrant red"`, `"very dark grayish blue"`).
///
/// "pub" only for packages/adico-primitives/tests/; not part of the intended public API.
pub fn color_name(color: Color) -> String {
    let (l, c, h) = to_oklch(color);

    match l {
        ..0.001 => return String::from("black"),
        0.999.. => return String::from("white"),
        _ => {}
    }

    let (hue, l) = oklch_hue(l, c, h);

    let (lightness, chroma) = color_modifiers(l, c);

    let mut parts = Vec::new();
    if !lightness.is_empty() {
        parts.push(lightness);
    }
    if !chroma.is_empty() {
        parts.push(chroma);
    }
    if !hue.is_empty() {
        parts.push(&hue);
    }

    parts.join(" ")
}

fn color_modifiers(lightness: f64, chroma: f64) -> (&'static str, &'static str) {
    match (lightness, chroma) {
        (..0.3, GRAY_THRESHOLD..=0.1) => ("very dark", "grayish"),
        (..0.3, 0.15..) => ("very dark", "vibrant"),
        (..0.3, _) => ("very dark", ""),

        (0.3..MAX_DARK_LIGHTNESS, GRAY_THRESHOLD..=0.1) => ("dark", "grayish"),
        (0.3..MAX_DARK_LIGHTNESS, 0.15..) => ("dark", "vibrant"),
        (0.3..MAX_DARK_LIGHTNESS, _) => ("dark", ""),

        (MAX_DARK_LIGHTNESS..0.7, GRAY_THRESHOLD..=0.1) => ("", "grayish"),
        (MAX_DARK_LIGHTNESS..0.7, 0.15..) => ("", "vibrant"),
        (MAX_DARK_LIGHTNESS..0.7, _) => ("", ""),

        (0.7..0.85, GRAY_THRESHOLD..=0.1) => ("light", "pale"),
        (0.7..0.85, 0.15..) => ("light", "vibrant"),
        (0.7..0.85, _) => ("light", ""),

        (0.85.., GRAY_THRESHOLD..=0.1) => ("very light", "pale"),
        (0.85.., 0.15..) => ("very light", "vibrant"),
        (0.85.., _) => ("very light", ""),

        (_, GRAY_THRESHOLD..=0.1) => ("very light", "grayish"),
        (_, 0.15..) => ("very light", "vibrant"),
        _ => ("very light", ""),
    }
}

/// Converts the RGB color to a (L, C, h) tuple.
fn to_oklch(color: Color) -> (f64, f64, f64) {
    let oklch: Oklch<f64> = color.into_format::<f64>().into_color();
    let (l, c, h) = oklch.into_components();
    (l, c, h.into_degrees())
}

fn oklch_hue(lightness: f64, chroma: f64, hue: f64) -> (String, f64) {
    if let ..GRAY_THRESHOLD = chroma {
        return ("gray".to_string(), lightness);
    }

    let hue = hue.rem_euclid(360.0);

    match (hue, lightness) {
        (0.0..=7.5, _) | (349.0..360.0, _) => ("pink".to_string(), lightness),
        (7.5..15.0, _) => ("pink red".to_string(), lightness),
        (15.0..=31.5, _) => ("red".to_string(), lightness),
        (31.5..48.0, _) => ("red orange".to_string(), lightness),
        (48.0..=71.0, ..ORANGE_LIGHTNESS_THRESHOLD) => ("brown".to_string(), lightness),
        (71.0..94.0, ..ORANGE_LIGHTNESS_THRESHOLD) => ("brown yellow".to_string(), lightness),
        (48.0..=71.0, _) => (
            "orange".to_string(),
            (lightness - ORANGE_LIGHTNESS_THRESHOLD) + MAX_DARK_LIGHTNESS,
        ),
        (71.0..94.0, _) => (
            "orange yellow".to_string(),
            (lightness - ORANGE_LIGHTNESS_THRESHOLD) + MAX_DARK_LIGHTNESS,
        ),
        (94.0..135.0, ..YELLOW_GREEN_LIGHTNESS_THRESHOLD) => {
            ("yellow green".to_string(), lightness)
        }
        (94.0..=114.5, _) => ("yellow".to_string(), lightness),
        (114.5..135.0, _) => ("yellow green".to_string(), lightness),
        (135.0..=155.0, _) => ("green".to_string(), lightness),
        (155.0..175.0, _) => ("green cyan".to_string(), lightness),
        (175.0..=219.5, _) => ("cyan".to_string(), lightness),
        (219.5..264.0, _) => ("cyan blue".to_string(), lightness),
        (264.0..=274.0, _) => ("blue".to_string(), lightness),
        (274.0..284.0, _) => ("blue purple".to_string(), lightness),
        (284.0..=302.0, _) => ("purple".to_string(), lightness),
        (302.0..320.0, _) => ("purple magenta".to_string(), lightness),
        (320.0..=334.5, _) => ("magenta".to_string(), lightness),
        (334.5..349.0, _) => ("magenta pink".to_string(), lightness),
        _ => unreachable!("Unexpected hue"),
    }
}

/// Context provided by [`ColorPicker`] to its descendants.
///
/// The picker is controlled in HSV — [`Self::color`] echoes the controlled
/// prop, and the setter methods emit `on_color_change` after applying the
/// requested edit on top of the current value.
#[derive(Clone, Copy)]
pub struct ColorPickerContext {
    color: ReadSignal<Hsv<encoding::Srgb, f64>>,
    on_color_change: Callback<Hsv<encoding::Srgb, f64>>,
}

impl ColorPickerContext {
    /// Read the current HSV color.
    pub fn color(&self) -> Hsv<encoding::Srgb, f64> {
        (self.color)()
    }

    /// Replace the entire HSV color.
    pub fn set_color(&self, c: Hsv<encoding::Srgb, f64>) {
        self.on_color_change.call(c);
    }

    /// Set hue, keeping saturation and value.
    pub fn set_hue(&self, h: f64) {
        let current = (self.color)();
        self.on_color_change.call(Hsv::<encoding::Srgb, f64>::new(
            RgbHue::new(h),
            current.saturation,
            current.value,
        ));
    }

    /// Set saturation and value as a pair, keeping hue.
    pub fn set_sv(&self, s: f64, v: f64) {
        let current = (self.color)();
        self.on_color_change
            .call(Hsv::<encoding::Srgb, f64>::new(current.hue, s, v));
    }
}

/// The props for the [`ColorPicker`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ColorPickerProps {
    /// The selected color
    #[props(default)]
    pub color: ReadSignal<Hsv<encoding::Srgb, f64>>,

    /// Callback when color changes
    #[props(default)]
    pub on_color_change: Callback<Hsv<encoding::Srgb, f64>>,

    /// Whether the color picker is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Additional attributes to extend the color picker element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the color picker element
    pub children: Element,
}

/// # ColorPicker
///
/// The [`ColorPicker`] component provides the color picker context and
/// synchronizes a color value between multiple color components.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::color_picker::*;
/// #[component]
/// fn Demo() -> Element {
///    use palette::{IntoColor, encoding};
///    let mut color = use_signal(|| -> palette::Hsv<encoding::Srgb, f64> {
///        Color::new(155, 128, 255).into_format::<f64>().into_color()
///    });
///    rsx! {
///            ColorPicker {
///                color: color(),
///                on_color_change: move |c| {
///                    tracing::info!("Color changed: {:?}", c);
///                    color.set(c);
///                },
///                ColorArea {
///                    AreaTrack {
///                        AreaThumb {
///                            AreaThumbSaturationInput {}
///                            AreaThumbValueInput {}
///                        }
///                    }
///                }
///            }
///    }
///}
/// ```
///
/// # Styling
///
/// The [`ColorPicker`] component defines the following data attributes you can use to control styling:
/// - `data-disabled`: Indicates if the ColorPicker is disabled. Possible values are `true` or `false`.
#[component]
pub fn ColorPicker(props: ColorPickerProps) -> Element {
    use_context_provider(|| ColorPickerContext {
        color: props.color,
        on_color_change: props.on_color_change,
    });

    rsx! {
        div {
            role: "group",
            aria_label: "Color picker",
            "data-disabled": (props.disabled)(),
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`ColorArea`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ColorAreaProps {
    /// The step value
    #[props(default = 1.0)]
    pub step: ReadSignal<f64>,

    /// Additional attributes to extend the color area element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the color area element
    pub children: Element,
}

/// # ColorArea
///
/// The [`ColorArea`] allows users to adjust two channels of color value against a two-dimensional gradient background.
/// Compose it with [`AreaTrack`] and [`AreaThumb`] inside a [`ColorPicker`].
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::color_picker::*;
/// #[component]
/// fn Demo() -> Element {
///    use palette::{IntoColor, encoding};
///    let mut color = use_signal(|| -> palette::Hsv<encoding::Srgb, f64> {
///        Color::new(155, 128, 255).into_format::<f64>().into_color()
///    });
///    rsx! {
///            ColorPicker {
///                color: color(),
///                on_color_change: move |c| {
///                    tracing::info!("Color changed: {:?}", c);
///                    color.set(c);
///                },
///                ColorArea {
///                    AreaTrack {
///                        AreaThumb {
///                            AreaThumbSaturationInput {}
///                            AreaThumbValueInput {}
///                        }
///                    }
///                }
///            }
///    }
///}
/// ```
#[component]
pub fn ColorArea(props: ColorAreaProps) -> Element {
    let picker_ctx = use_context::<ColorPickerContext>();
    let mut dragging = use_signal(|| false);

    // Thumb position is read straight from HSV state so saturation is preserved
    // at brightness=0.
    let value = use_memo(move || area_value_from_hsv(picker_ctx.color()));

    let area_ctx = use_context_provider(|| ColorAreaContext {
        value,
        step: props.step,
        dragging: dragging.into(),
    });

    let mut movement = use_move_interaction(dragging);
    let mut granular_value = use_hook(|| CopyValue::new(value()));

    let size = movement.rect().map(|r| r.size);

    use_effect(move || {
        if !dragging() {
            return;
        }

        let Some(size) = size else {
            return;
        };

        let Some(move_event) = movement.pointer_move() else {
            return;
        };

        let d_s = move_event.delta_x / size.width * COLOR_AREA_RANGE;
        let d_h = move_event.delta_y / size.height * COLOR_AREA_RANGE;

        let new_value = granular_value() + Size2D::new(d_s, -d_h);
        granular_value.set(new_value);
        set_area_value(picker_ctx, clamp_area_value(new_value, (area_ctx.step)()));
    });

    rsx! {
        div {
            role: "group",
            onmounted: move |e| async move {
                let mut movement = movement;
                movement.set_mounted(e.data()).await;
            },
            onresize: move |_| async move {
                let mut movement = movement;
                movement.refresh_rect().await;
            },
            onpointerdown: move |e| {
                if !movement.start_pointer(&e) {
                    return;
                }

                // Handle pointer interaction
                spawn(async move {
                    let mut movement = movement;

                    // Update the bounding rect of the slider in case it moved
                    if let Some(r) = movement.refresh_rect().await {
                        let size = r.size;

                        // Get the mouse position relative to the slider
                        let top_left = r.origin;
                        let relative_pos = e.client_coordinates() - top_left.cast_unit();

                        let x = (relative_pos.x / size.width) * COLOR_AREA_RANGE;
                        let y = COLOR_AREA_MAX - ((relative_pos.y / size.height) * COLOR_AREA_RANGE);
                        let pt = ClientPoint::new(x, y);
                        granular_value.set(pt);
                        set_area_value(picker_ctx, clamp_area_value(pt, (area_ctx.step)()));
                    }

                    dragging.set(true);
                });
            },
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`AreaTrack`] component
#[derive(Props, Clone, PartialEq)]
pub struct AreaTrackProps {
    /// Additional attributes to apply to the track element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the track which should include a [`AreaThumb`]
    pub children: Element,
}

/// # AreaTrack
///
/// The track component for [`ColorArea`]. It renders the color plane background
/// and should contain an [`AreaThumb`].
///
/// This must be used inside a [`ColorArea`] component.
#[component]
pub fn AreaTrack(props: AreaTrackProps) -> Element {
    let picker_ctx = use_context::<ColorPickerContext>();
    let area_color = color_hex(
        Srgb::<f64>::from_color(Hsv::<encoding::Srgb, f64>::new(
            picker_ctx.color().hue,
            1.0,
            1.0,
        ))
        .into_format(),
    );

    rsx! {
        div {
            style: "--area-color: {area_color}",
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`AreaThumb`] component
#[derive(Props, Clone, PartialEq)]
pub struct AreaThumbProps {
    /// Additional attributes to apply to the thumb element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the thumb element
    pub children: Element,
}

/// The props for the [`AreaThumbSaturationInput`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AreaThumbSaturationInputProps {
    /// Additional attributes to apply to the saturation input element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The props for the [`AreaThumbValueInput`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AreaThumbValueInputProps {
    /// Additional attributes to apply to the value input element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # AreaThumb
///
/// The thumb component for [`ColorArea`]. It supports mouse/touch interaction
/// through [`ColorArea`] and keyboard navigation with arrow keys.
///
/// This must be used inside a [`ColorArea`] component.
#[component]
pub fn AreaThumb(props: AreaThumbProps) -> Element {
    let picker_ctx = use_context::<ColorPickerContext>();
    let area_ctx = use_context::<ColorAreaContext>();
    let direction = use_direction();

    let mut button_ref: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    let saturation_input_ref: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    let value_input_ref: Signal<Option<Rc<MountedData>>> = use_signal(|| None);

    let thumb_ctx = use_context_provider(|| AreaThumbContext {
        saturation_input_ref,
        value_input_ref,
    });

    use_effect(move || {
        if let Some(button) = button_ref() {
            let dragging = area_ctx.dragging.cloned();
            if dragging {
                spawn(async move {
                    _ = button.set_focus(true).await;
                });
            }
        }
    });

    let percent = area_percent((area_ctx.value)());
    let style = format!(
        "left: {:.2}%; top: {:.2}%;",
        percent.width,
        100. - percent.height
    );
    let thumb_color = color_hex(Srgb::<f64>::from_color(picker_ctx.color()).into_format());

    rsx! {
        div {
            aria_label: "Color area",
            "data-dragging": area_ctx.dragging,
            style,
            background_color: thumb_color,
            tabindex: 0,
            onmounted: move |evt| {
                // Store the mounted data for focus management
                button_ref.set(Some(evt.data()));
            },
            onmousedown: move |evt| {
                // Don't focus the button. The dragging state will handle focus
                evt.prevent_default();
            },
            ontouchstart: move |evt| {
                // Don't focus the button. The dragging state will handle focus
                evt.prevent_default();
            },
            // First arrow press from the wrapper applies the step and hands
            // focus to the matching axis input so AT announces the channel.
            onkeydown: move |evt: Event<KeyboardData>| async move {
                let Some(move_event) = MoveEvent::from_keyboard(&evt, (area_ctx.step)(), direction)
                else {
                    return;
                };
                evt.prevent_default();

                let new_value =
                    (area_ctx.value)() + Size2D::new(move_event.delta_x, move_event.delta_y);
                set_area_value(picker_ctx, clamp_area_value(new_value, (area_ctx.step)()));

                let target = if move_event.delta_x != 0.0 {
                    (thumb_ctx.saturation_input_ref)()
                } else {
                    (thumb_ctx.value_input_ref)()
                };
                if let Some(target) = target {
                    _ = target.set_focus(true).await;
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

/// The saturation axis input for [`AreaThumb`].
#[component]
pub fn AreaThumbSaturationInput(props: AreaThumbSaturationInputProps) -> Element {
    let picker_ctx = use_context::<ColorPickerContext>();
    let area_ctx = use_context::<ColorAreaContext>();
    let mut thumb_ctx = use_context::<AreaThumbContext>();
    let direction = use_direction();

    let percent = area_percent((area_ctx.value)());
    let current = (area_ctx.value)();
    let min = COLOR_AREA_MIN;
    let max = COLOR_AREA_MAX;
    let step = (area_ctx.step)();
    let color_label = color_name(Srgb::<f64>::from_color(picker_ctx.color()).into_format());

    rsx! {
        input {
            r#type: "range",
            aria_label: "Saturation",
            aria_roledescription: "2D Slider",
            aria_valuetext: format!("Saturation {:.0}%, {color_label}", percent.width),
            aria_orientation: "horizontal",
            tabindex: "-1",
            min: "{min}",
            max: "{max}",
            step: "{step}",
            value: format!("{}", current.x),
            onmounted: move |evt| {
                thumb_ctx.saturation_input_ref.set(Some(evt.data()));
            },
            // Cross-axis arrows hand focus to the value input so AT
            // announces the new channel.
            onkeydown: move |evt: Event<KeyboardData>| async move {
                let Some(move_event) = MoveEvent::from_keyboard(&evt, (area_ctx.step)(), direction)
                else {
                    return;
                };
                evt.prevent_default();

                let new_value =
                    (area_ctx.value)() + Size2D::new(move_event.delta_x, move_event.delta_y);
                set_area_value(picker_ctx, clamp_area_value(new_value, (area_ctx.step)()));

                if move_event.delta_y != 0.0 {
                    if let Some(target) = (thumb_ctx.value_input_ref)() {
                        _ = target.set_focus(true).await;
                    }
                }
            },
            // Voice-control / direct-manipulation: a programmatic value
            // change on the input feeds the new saturation through.
            oninput: move |evt: Event<FormData>| {
                if let Ok(s) = evt.value().parse::<f64>() {
                    let v = picker_ctx.color().value;
                    let scaled = s.clamp(COLOR_AREA_MIN, COLOR_AREA_MAX) / COLOR_AREA_RANGE;
                    picker_ctx.set_sv(scaled, v);
                }
            },
            ..props.attributes,
        }
    }
}

/// The value axis input for [`AreaThumb`].
#[component]
pub fn AreaThumbValueInput(props: AreaThumbValueInputProps) -> Element {
    let picker_ctx = use_context::<ColorPickerContext>();
    let area_ctx = use_context::<ColorAreaContext>();
    let mut thumb_ctx = use_context::<AreaThumbContext>();
    let direction = use_direction();

    let percent = area_percent((area_ctx.value)());
    let current = (area_ctx.value)();
    let min = COLOR_AREA_MIN;
    let max = COLOR_AREA_MAX;
    let step = (area_ctx.step)();
    let color_label = color_name(Srgb::<f64>::from_color(picker_ctx.color()).into_format());

    rsx! {
        input {
            r#type: "range",
            aria_label: "Value",
            aria_roledescription: "2D Slider",
            aria_valuetext: format!("Value {:.0}%, {color_label}", percent.height),
            aria_orientation: "vertical",
            tabindex: "-1",
            min: "{min}",
            max: "{max}",
            step: "{step}",
            value: format!("{}", current.y),
            onmounted: move |evt| {
                thumb_ctx.value_input_ref.set(Some(evt.data()));
            },
            onkeydown: move |evt: Event<KeyboardData>| async move {
                let Some(move_event) = MoveEvent::from_keyboard(&evt, (area_ctx.step)(), direction)
                else {
                    return;
                };
                evt.prevent_default();

                let new_value =
                    (area_ctx.value)() + Size2D::new(move_event.delta_x, move_event.delta_y);
                set_area_value(picker_ctx, clamp_area_value(new_value, (area_ctx.step)()));

                if move_event.delta_x != 0.0 {
                    if let Some(target) = (thumb_ctx.saturation_input_ref)() {
                        _ = target.set_focus(true).await;
                    }
                }
            },
            oninput: move |evt: Event<FormData>| {
                if let Ok(v) = evt.value().parse::<f64>() {
                    let s = picker_ctx.color().saturation;
                    let scaled = v.clamp(COLOR_AREA_MIN, COLOR_AREA_MAX) / COLOR_AREA_RANGE;
                    picker_ctx.set_sv(s, scaled);
                }
            },
            ..props.attributes,
        }
    }
}

#[derive(Copy, Clone)]
struct ColorAreaContext {
    value: Memo<ClientPoint>,
    step: ReadSignal<f64>,
    dragging: ReadSignal<bool>,
}

#[derive(Clone, Copy)]
struct AreaThumbContext {
    saturation_input_ref: Signal<Option<Rc<MountedData>>>,
    value_input_ref: Signal<Option<Rc<MountedData>>>,
}
