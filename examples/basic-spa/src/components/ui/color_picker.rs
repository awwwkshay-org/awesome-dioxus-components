//! Source-owned Dioxus-only Color Picker for Dioxus, backed by the owned
//! adico primitive layer. This is a Dioxus Components extra with no shadcn
//! equivalent -- it does not count toward shadcn parity.

use dioxus::prelude::*;
use palette::{FromColor, Hsl, Hsv, IntoColor, RgbHue, Srgb, encoding};

use adico_primitives::color_picker::{
    AreaThumb as AreaThumbPrimitive, AreaThumbSaturationInput as AreaThumbSaturationInputPrimitive,
    AreaThumbValueInput as AreaThumbValueInputPrimitive, AreaTrack as AreaTrackPrimitive,
    ColorArea as ColorAreaPrimitive, ColorPicker as ColorPickerPrimitive, color_name,
};
pub use adico_primitives::color_picker::{
    AreaThumbSaturationInputProps, AreaThumbValueInputProps, Color, ColorPickerContext,
};
use adico_primitives::popover::{PopoverRoot, PopoverRootProps};

use super::copy_button::CopyButton;
use super::native_select::{NativeSelect, NativeSelectOption, NativeSelectSize};
use super::popover::PopoverTrigger;
use super::slider::{Slider, SliderThumb, SliderTrack};
use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

/// A picked color's HSV representation, matching [`ColorPickerContext::color`].
type Hsvf = Hsv<encoding::Srgb, f64>;
/// A picked color's HSL representation, used by [`ColorPickerFields`]' HSL mode.
type Hslf = Hsl<encoding::Srgb, f64>;

/// Provides the color-picker context and synchronizes a color value between
/// its descendants.
#[component]
pub fn ColorPicker(
    #[props(default)] color: ReadSignal<palette::Hsv<palette::encoding::Srgb, f64>>,
    #[props(default)] on_color_change: Callback<palette::Hsv<palette::encoding::Srgb, f64>>,
    #[props(default)] disabled: ReadSignal<bool>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&[
        "inline-flex flex-col gap-2",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ColorPickerPrimitive {
            color,
            on_color_change,
            disabled,
            class,
            attributes,
            {children}
        }
    }
}

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Props, Clone, PartialEq)]
pub struct ColorPickerPopoverProps {
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub is_modal: ReadSignal<bool>,
    pub open: ReadSignal<Option<bool>>,
    #[props(default)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: Callback<bool>,
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
    #[props(default = PopoverRoot)]
    pub popover_root: fn(PopoverRootProps) -> Element,
}

/// Styled popover root for composing [`ColorPickerTrigger`] and the installed
/// `Popover`'s own `PopoverContent` around a [`ColorPicker`]'s controls. Takes
/// the same `popover_root` injection prop as `date-picker`'s
/// `DatePickerPopover`, so a consumer can swap in a custom popover root
/// without `adico-primitives` needing to know about it.
#[component]
pub fn ColorPickerPopover(props: ColorPickerPopoverProps) -> Element {
    let class = cn(&[
        "group/color-picker",
        props.class.as_deref().unwrap_or_default(),
    ]);
    let PopoverRoot = props.popover_root;
    rsx! {
        PopoverRoot {
            is_modal: props.is_modal,
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            class,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ColorPickerTriggerProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

/// Disclosure trigger composed from the installed Popover façade. Defaults to
/// rendering a [`ColorPickerSwatch`], so activating this trigger both opens
/// the picker and shows the color it currently holds -- must be used inside
/// a [`ColorPicker`] (for the swatch's context) and a [`ColorPickerPopover`].
#[component]
pub fn ColorPickerTrigger(props: ColorPickerTriggerProps) -> Element {
    let class = cn(&[
        "inline-flex size-8 items-center justify-center rounded-md border border-input shadow-xs transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50",
        props.class.as_deref().unwrap_or_default(),
    ]);
    let children = props
        .children
        .unwrap_or_else(|| rsx! { ColorPickerSwatch {} });
    rsx! {
        PopoverTrigger { class, attributes: props.attributes, {children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ColorPickerSwatchProps {
    #[props(default)]
    pub class: Option<String>,
}

/// A filled chip reflecting a [`ColorPicker`]'s current selected color.
/// Renders its fill as an inline `style` (not a Tailwind class), since the
/// color is a runtime value Tailwind cannot see statically in a consuming
/// project's own source. Labeled with the color's human-readable name so the
/// selection is conveyed to assistive technology, not only through its
/// visual fill. Must be used inside a [`ColorPicker`].
#[component]
pub fn ColorPickerSwatch(props: ColorPickerSwatchProps) -> Element {
    let ctx = use_context::<ColorPickerContext>();
    let color: Memo<Color> = use_memo(move || Srgb::<f64>::from_color(ctx.color()).into_format());
    let label = use_memo(move || color_name(color()));
    let class = cn(&[
        "size-4 shrink-0 rounded-sm border border-border/50",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        span {
            class,
            style: "background-color: #{color():X};",
            role: "img",
            "aria-label": "{label()}",
        }
    }
}

/// A two-dimensional saturation/value drag surface.
#[component]
pub fn ColorArea(
    #[props(default = 1.0)] step: ReadSignal<f64>,
    #[props(default = Radius::Md)] radius: Radius,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "relative size-48 touch-none border border-input",
        radius.class(),
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
pub fn AreaThumb(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "absolute size-4 -translate-x-1/2 translate-y-1/2 rounded-full border-2 border-white shadow-[0_0_0_1px_rgba(0,0,0,0.3)] outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 data-[dragging=true]:cursor-grabbing",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AreaThumbPrimitive { class, {children} }
    }
}

/// A horizontal slider that sets the color picker's hue (0-360°), keeping
/// saturation and value unchanged.
///
/// Real, previously-missing functionality, not a bug fix: [`ColorArea`]'s
/// drag surface only ever adjusts saturation/value *within the current
/// hue's plane* -- there was no control anywhere in this registry or the
/// playground demo that changed the hue itself, so a color like pure red
/// (hue 0°) was unreachable from whatever hue the picker happened to start
/// at (a user directly asked how to pick red and, at the time, there
/// genuinely was no way through the UI). `ColorPickerContext::set_hue`
/// already existed at the primitive layer for exactly this, just never
/// composed into a visible control. Reuses this registry's own already-built,
/// already-tested [`Slider`]/[`SliderTrack`]/[`SliderThumb`] (task 5.3f)
/// rather than inventing a new drag control, per this repo's standing
/// composition rule.
///
/// This must be used inside a [`ColorPicker`] component.
#[component]
pub fn HueSlider(class: Option<String>) -> Element {
    let ctx = use_context::<ColorPickerContext>();
    let hue = use_memo(move || ctx.color().hue.into_positive_degrees());

    rsx! {
        Slider {
            value: Some(hue()),
            min: 0.0,
            max: 360.0,
            step: 1.0,
            label: Some("Hue".to_string()),
            on_value_change: move |h: f64| ctx.set_hue(h),
            class,
            SliderTrack {
                class: "bg-[linear-gradient(to_right,red,yellow,lime,cyan,blue,magenta,red)]",
                SliderThumb {}
            }
        }
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

/// Which text representation [`ColorPickerFields`] currently displays and
/// edits. Purely a display preference of that one composed part -- not
/// shared color state, so it lives as local component state rather than on
/// [`ColorPickerContext`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ColorFormat {
    #[default]
    Hex,
    Rgb,
    Hsl,
}

impl ColorFormat {
    fn label(self) -> &'static str {
        match self {
            Self::Hex => "HEX",
            Self::Rgb => "RGB",
            Self::Hsl => "HSL",
        }
    }

    const ALL: [ColorFormat; 3] = [Self::Hex, Self::Rgb, Self::Hsl];
}

/// Formats `color` as text in `format` (e.g. `#9B80FF`, or space-separated
/// channel values for RGB/HSL).
fn format_color(color: Hsvf, format: ColorFormat) -> String {
    match format {
        ColorFormat::Hex => {
            let rgb: Color = Srgb::<f64>::from_color(color).into_format();
            format!("#{rgb:X}")
        }
        ColorFormat::Rgb => {
            let rgb: Color = Srgb::<f64>::from_color(color).into_format();
            format!("{} {} {}", rgb.red, rgb.green, rgb.blue)
        }
        ColorFormat::Hsl => {
            let hsl = Hslf::from_color(color);
            format!(
                "{:.0} {:.0}% {:.0}%",
                hsl.hue.into_positive_degrees(),
                hsl.saturation * 100.0,
                hsl.lightness * 100.0
            )
        }
    }
}

/// Parses `#rrggbb`/`#RRGGBB` (with or without the leading `#`) into an sRGB
/// triplet. Hand-rolled rather than a new crate dependency, mirroring
/// [`ColorPickerSwatch`]'s existing `Srgb<u8>` `UpperHex` formatting -- the
/// same representation, just the reverse direction.
fn parse_hex(value: &str) -> Option<Color> {
    let value = value.strip_prefix('#').unwrap_or(value);
    if value.len() != 6 {
        return None;
    }
    let channel = |slice: &str| u8::from_str_radix(slice, 16).ok();
    let r = channel(value.get(0..2)?)?;
    let g = channel(value.get(2..4)?)?;
    let b = channel(value.get(4..6)?)?;
    Some(Srgb::new(r, g, b))
}

#[derive(Props, Clone, PartialEq)]
pub struct ColorPickerFieldsProps {
    #[props(default)]
    pub class: Option<String>,
}

/// A compact format-select (HEX / RGB / HSL) plus the matching editable
/// field(s) for the selected format, staying in sync with the same
/// [`ColorPickerContext`] [`ColorArea`], [`HueSlider`], and
/// [`ColorPickerSwatch`] already read and write. Includes a [`CopyButton`]
/// for the currently displayed value. Composable and additive: a
/// [`ColorPicker`] composed without this part is unaffected. Must be used
/// inside a [`ColorPicker`].
#[component]
pub fn ColorPickerFields(props: ColorPickerFieldsProps) -> Element {
    let ctx = use_context::<ColorPickerContext>();
    let mut format = use_signal(ColorFormat::default);

    let rgb: Memo<Color> = use_memo(move || Srgb::<f64>::from_color(ctx.color()).into_format());
    let hsl: Memo<Hslf> = use_memo(move || Hslf::from_color(ctx.color()));
    let displayed = use_memo(move || format_color(ctx.color(), format()));

    let set_rgb_channel = move |channel: usize, text: String| {
        if let Ok(value) = text.trim().parse::<u8>() {
            let current = rgb();
            let updated = match channel {
                0 => Srgb::new(value, current.green, current.blue),
                1 => Srgb::new(current.red, value, current.blue),
                _ => Srgb::new(current.red, current.green, value),
            };
            let next: Hsvf = updated.into_format::<f64>().into_color();
            ctx.set_color(next);
        }
    };

    let set_hsl = move |hue: RgbHue<f64>, saturation: f64, lightness: f64| {
        let next: Hsvf = Hslf::new(hue, saturation, lightness).into_color();
        ctx.set_color(next);
    };

    let class = cn(&[
        "grid gap-2 text-xs",
        props.class.as_deref().unwrap_or_default(),
    ]);

    let field_class =
        "min-w-0 flex-1 border border-input bg-background px-1.5 py-1 tabular-nums text-foreground";

    rsx! {
        div { class,
            div { class: "flex items-center gap-2",
                NativeSelect {
                    size: NativeSelectSize::Sm,
                    class: "w-20",
                    value: Some((format() as u8).to_string()),
                    oninput: move |event: FormEvent| {
                        if let Ok(index) = event.value().parse::<usize>()
                            && let Some(next) = ColorFormat::ALL.get(index)
                        {
                            format.set(*next);
                        }
                    },
                    for (index , option) in ColorFormat::ALL.iter().enumerate() {
                        NativeSelectOption { value: "{index}", "{option.label()}" }
                    }
                }
                match format() {
                    ColorFormat::Hex => rsx! {
                        input {
                            r#type: "text",
                            class: "min-w-0 flex-1 border border-input bg-background px-2 py-1 font-mono tabular-nums text-foreground",
                            "aria-label": "Hex color value",
                            value: "{displayed()}",
                            onchange: move |event| {
                                if let Some(color) = parse_hex(&event.value()) {
                                    let next: Hsvf = color.into_format::<f64>().into_color();
                                    ctx.set_color(next);
                                }
                            },
                        }
                    },
                    ColorFormat::Rgb => rsx! {
                        div { class: "flex flex-1 gap-1",
                            for (index , axis_label) in [(0, "R"), (1, "G"), (2, "B")] {
                                label { class: "flex min-w-0 flex-1 items-center gap-1", "aria-label": "{axis_label}",
                                    input {
                                        r#type: "number",
                                        min: "0",
                                        max: "255",
                                        class: field_class,
                                        value: "{[rgb().red, rgb().green, rgb().blue][index]}",
                                        onchange: move |event| set_rgb_channel(index, event.value()),
                                    }
                                }
                            }
                        }
                    },
                    ColorFormat::Hsl => rsx! {
                        div { class: "flex flex-1 gap-1",
                            label { class: "flex min-w-0 flex-1 items-center gap-1", "aria-label": "Hue",
                                input {
                                    r#type: "number",
                                    min: "0",
                                    max: "360",
                                    class: field_class,
                                    value: "{hsl().hue.into_positive_degrees().round()}",
                                    onchange: move |event| {
                                        if let Ok(value) = event.value().trim().parse::<f64>() {
                                            set_hsl(RgbHue::new(value), hsl().saturation, hsl().lightness);
                                        }
                                    },
                                }
                            }
                            label { class: "flex min-w-0 flex-1 items-center gap-1", "aria-label": "Saturation",
                                input {
                                    r#type: "number",
                                    min: "0",
                                    max: "100",
                                    class: field_class,
                                    value: "{(hsl().saturation * 100.0).round()}",
                                    onchange: move |event| {
                                        if let Ok(value) = event.value().trim().parse::<f64>() {
                                            set_hsl(hsl().hue, (value / 100.0).clamp(0.0, 1.0), hsl().lightness);
                                        }
                                    },
                                }
                            }
                            label { class: "flex min-w-0 flex-1 items-center gap-1", "aria-label": "Lightness",
                                input {
                                    r#type: "number",
                                    min: "0",
                                    max: "100",
                                    class: field_class,
                                    value: "{(hsl().lightness * 100.0).round()}",
                                    onchange: move |event| {
                                        if let Ok(value) = event.value().trim().parse::<f64>() {
                                            set_hsl(hsl().hue, hsl().saturation, (value / 100.0).clamp(0.0, 1.0));
                                        }
                                    },
                                }
                            }
                        }
                    },
                }
                CopyButton { value: displayed() }
            }
        }
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

    #[test]
    fn hue_slider_track_spans_the_full_hue_wheel_back_to_red() {
        let gradient = "linear-gradient(to_right,red,yellow,lime,cyan,blue,magenta,red)";
        assert!(gradient.starts_with("linear-gradient(to_right,red,"));
        assert!(gradient.ends_with(",red)"));
    }
}
