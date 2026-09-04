// Implements the WAI-ARIA APG "Meter" pattern: `role="meter"` with a required
// `aria-valuenow` (unlike `progress.rs`'s `role="progressbar"`, a meter has no
// indeterminate state -- Base UI's own `Meter.Root` makes `value` a required prop,
// not `Option<f64>`), plus `aria-valuemin`/`aria-valuemax`/optional `aria-valuetext`.
// Structurally mirrors `progress.rs` (root/track/indicator + a `--meter-value` CSS
// variable), adding a `MeterLabel` and `MeterValue` render-prop part matching Base
// UI's own `label`/`value` parts, since Base UI names both explicitly where
// `progress.rs`'s upstream axis does not.

//! Defines the [`MeterRoot`] component and its sub-components.

use dioxus::prelude::*;

#[derive(Clone, Copy)]
struct MeterCtx {
    value: ReadSignal<f64>,
}

/// The props for the [`MeterRoot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MeterRootProps {
    /// The current value. Unlike `progress.rs`'s optional value, a meter
    /// always has a known reading -- there is no indeterminate meter state.
    pub value: ReadSignal<f64>,

    /// The minimum value. Defaults to 0.
    #[props(default = ReadSignal::new(Signal::new(0.0)))]
    pub min: ReadSignal<f64>,

    /// The maximum value. Defaults to 100.
    #[props(default = ReadSignal::new(Signal::new(100.0)))]
    pub max: ReadSignal<f64>,

    /// A human-readable alternative for `value`, exposed as `aria-valuetext`
    /// (e.g. `"6 of 10 disk slots used"` instead of the raw number).
    #[props(default)]
    pub value_text: ReadSignal<Option<String>>,

    /// Additional attributes to apply to the meter's root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the meter component.
    pub children: Element,
}

/// # MeterRoot
///
/// The `MeterRoot` component displays a numeric reading within a known
/// range, such as disk usage or a rating -- distinct from [`crate::progress::Progress`],
/// which tracks an in-progress operation and can be indeterminate.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::meter::{MeterRoot, MeterTrack, MeterIndicator};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         MeterRoot {
///             aria_label: "Disk usage",
///             value: 60.0,
///             MeterTrack {
///                 MeterIndicator {}
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`MeterRoot`] component defines the following data attributes you
/// can use to control styling:
/// - `data-value`: The current meter value.
/// - `data-min` / `data-max`: The meter's range.
///
/// The [`MeterRoot`] component defines the following CSS variable you can
/// use to control styling:
/// - `--meter-value`: A value between 0 and 100 representing the current
///   reading's percentage across `min..max`.
#[component]
pub fn MeterRoot(props: MeterRootProps) -> Element {
    use_context_provider(|| MeterCtx { value: props.value });

    let percentage = use_memo(move || {
        let min = (props.min)();
        let max = (props.max)();
        let span = max - min;
        if span <= 0.0 {
            0.0
        } else {
            (((props.value)() - min) / span * 100.0).clamp(0.0, 100.0)
        }
    });

    rsx! {
        div {
            role: "meter",
            "aria-valuemin": props.min,
            "aria-valuemax": props.max,
            "aria-valuenow": props.value,
            "aria-valuetext": props.value_text,
            "data-value": props.value.cloned().to_string(),
            "data-min": props.min,
            "data-max": props.max,
            style: "--meter-value: {percentage()}%",
            ..props.attributes,

            {props.children}
        }
    }
}

/// The props for the [`MeterTrack`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MeterTrackProps {
    /// Additional attributes to apply to the track element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the track, typically a [`MeterIndicator`].
    pub children: Element,
}

/// # MeterTrack
///
/// The background track a [`MeterIndicator`] fills. This must be used
/// inside a [`MeterRoot`] component.
#[component]
pub fn MeterTrack(props: MeterTrackProps) -> Element {
    rsx! {
        div { ..props.attributes, {props.children} }
    }
}

/// The props for the [`MeterIndicator`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MeterIndicatorProps {
    /// Additional attributes to apply to the indicator element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the indicator.
    pub children: Element,
}

/// # MeterIndicator
///
/// The visual fill representing the meter's current reading. This must be
/// used inside a [`MeterRoot`] component (typically nested in a [`MeterTrack`]).
///
/// ## Styling
///
/// Reads the same `--meter-value` CSS variable [`MeterRoot`] sets on its own
/// element (custom properties inherit), so a percentage-width fill can be
/// styled with `width: var(--meter-value)` without re-deriving it here.
#[component]
pub fn MeterIndicator(props: MeterIndicatorProps) -> Element {
    rsx! {
        div { ..props.attributes, {props.children} }
    }
}

/// The props for the [`MeterLabel`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MeterLabelProps {
    /// Additional attributes to apply to the label element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the label.
    pub children: Element,
}

/// # MeterLabel
///
/// A visible label for the [`MeterRoot`] it's nested in. Purely
/// presentational (does not wire `aria-labelledby` itself); pass an
/// explicit `id` and set `aria_labelledby` on [`MeterRoot`] to associate
/// them, matching this crate's other non-native-`<label>` labeling parts.
#[component]
pub fn MeterLabel(props: MeterLabelProps) -> Element {
    rsx! {
        span { ..props.attributes, {props.children} }
    }
}

/// The props for the [`MeterValue`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MeterValueProps {
    /// Additional attributes to apply to the value element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// A render function receiving the raw numeric value, producing the
    /// displayed content (e.g. formatted with a unit). Defaults to the raw
    /// value's `Display` output when not provided.
    #[props(default)]
    pub render: Option<Callback<f64, Element>>,
}

/// # MeterValue
///
/// Displays the ancestor [`MeterRoot`]'s current numeric value, optionally
/// formatted through a render callback (matching Base UI's `Meter.Value`
/// `children` render-prop, translated as `render` here since this crate's
/// `children` field is reserved for nested rsx content, not a callback).
/// Must be used inside a `MeterRoot`.
#[component]
pub fn MeterValue(props: MeterValueProps) -> Element {
    let ctx: MeterCtx = use_context();
    let value = (ctx.value)();

    rsx! {
        span { ..props.attributes,
            if let Some(render) = props.render {
                {render.call(value)}
            } else {
                "{value}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_required_aria_valuenow_unlike_indeterminate_progress() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                MeterRoot { aria_label: "Disk usage", value: 60.0,
                    MeterTrack {
                        MeterIndicator {}
                    }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains(r#"role="meter""#), "{html}");
        assert!(html.contains("aria-valuenow=60"), "{html}");
    }

    #[test]
    fn zero_span_range_does_not_divide_by_zero() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                MeterRoot { aria_label: "Degenerate", value: 5.0, min: 3.0, max: 3.0,
                    MeterTrack {
                        MeterIndicator {}
                    }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("--meter-value: 0%"), "{html}");
    }
}
