// The WAI-ARIA APG "Spin Button" pattern applied per-segment -- the same
// pattern `date_picker.rs`'s year/month/day segments already followed before
// this module was extracted from it (see that file's own top-of-file
// comment). `MeridiemSegment` extends the same composed-segmented-field
// shape to a non-numeric value: it participates in the same roving-focus
// sequence but exposes `aria-valuetext` ("AM"/"PM") instead of a numeric
// `aria-valuenow`, since there is no meaningful numeric value to announce.

//! Shared segmented-field primitives: a generic numeric segment
//! ([`NumericSegment`]) and a two-value meridiem segment ([`MeridiemSegment`]),
//! both operating over a minimal [`SegmentFieldContext`] (roving focus,
//! disabled, read-only) rather than any one composed field's full state.
//! Originally private to `date_picker.rs`'s year/month/day segments;
//! extracted so `time_picker.rs`'s hour/minute/second/meridiem segments can
//! reuse the same roving-focus, auto-advance, and spinbutton-ARIA behavior
//! instead of duplicating it. Composed fields that need more than this
//! (date's enabled-range, popover-closes-on-focus) layer their own context
//! and an `on_focus` hook on top -- see `date_picker.rs`'s
//! `DatePickerYearSegment` for the pattern.

use crate::collection::{CollectionState, collection_item, use_collection_provider, use_item};
use crate::use_unique_id;

use dioxus::prelude::*;
use num_integer::Integer;
use std::fmt::Display;
use std::str::FromStr;

/// Shared roving-focus/disabled/read-only state for a composed segmented
/// field. Deliberately minimal: it carries nothing specific to any one kind
/// of composed field (no date-only concepts like an enabled date range, no
/// popover-open state) so both date and time segments -- and any future
/// segmented field -- can share it.
#[derive(Copy, Clone)]
pub(crate) struct SegmentFieldContext {
    pub focus: CollectionState,
    pub disabled: ReadSignal<bool>,
    pub read_only: ReadSignal<bool>,
}

/// Provide a fresh [`SegmentFieldContext`] for a composed segmented field
/// rooted at this component, returning it so the caller can also read
/// `focus`/`disabled`/`read_only` for its own purposes (e.g. a picker root
/// that also needs `focus` for a calendar grid).
pub(crate) fn use_segment_field_provider(
    roving_loop: ReadSignal<bool>,
    disabled: ReadSignal<bool>,
    read_only: ReadSignal<bool>,
) -> SegmentFieldContext {
    let focus = use_collection_provider(roving_loop);
    use_context_provider(|| SegmentFieldContext {
        focus,
        disabled,
        read_only,
    })
}

/// The props for [`NumericSegment`].
#[derive(Props, Clone, PartialEq)]
pub(crate) struct NumericSegmentProps<T: Clone + Integer + 'static> {
    /// The index of this segment within its enclosing field's roving-focus
    /// sequence.
    pub index: ReadSignal<usize>,

    /// The controlled value.
    pub value: ReadSignal<Option<T>>,

    /// Default value used when a new value is typed with nothing previously set.
    pub default: T,

    /// Callback when the value changes.
    #[props(default)]
    pub on_value_change: Callback<Option<T>>,

    /// The minimum value.
    pub min: ReadSignal<T>,

    /// The maximum value.
    pub max: ReadSignal<T>,

    /// Max field length (digit count).
    pub max_length: usize,

    /// Callback to format the placeholder text shown when there is no value.
    pub on_format_placeholder: Callback<(), String>,

    /// Callback to format `Some(value)` for display. `None` (the default)
    /// zero-pads the raw value to `max_length` -- the primitive's original,
    /// pre-extraction behavior. Set this to render a value differently from
    /// its raw number (e.g. a 12-hour clock hour displaying `13` as `01`).
    /// Not a `#[props(default = ...)]` closure because that expression can't
    /// see this struct's own `max_length` field to close over it.
    #[props(default)]
    pub on_format_value: Option<Callback<T, String>>,

    /// Called when this segment receives focus, after the roving-focus
    /// bookkeeping. A composed field that needs additional focus behavior
    /// (for example, a date field closing its enclosing popover once the
    /// user starts typing) wires that here rather than this primitive
    /// knowing about it.
    #[props(default)]
    pub on_focus: Callback<()>,

    /// Additional attributes for the segment element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub(crate) fn NumericSegment<T: Clone + Copy + Integer + FromStr + Display + 'static>(
    props: NumericSegmentProps<T>,
) -> Element {
    let mut text_value = use_signal(|| "".to_string());
    use_effect(move || {
        let text = match (props.value)() {
            Some(value) => value.to_string(),
            None => String::default(),
        };
        text_value.set(text);
    });

    let mut reset_value = use_signal(|| false);

    // The formatted text for the segment
    let display_value = use_memo(move || {
        let value = (props.value)();
        match value {
            Some(value) => match props.on_format_value {
                Some(format) => format.call(value),
                None => format!("{:0>width$}", value, width = props.max_length),
            },
            None => props
                .on_format_placeholder
                .call(())
                .repeat(props.max_length),
        }
    });

    let now_value = use_memo(move || (props.value)().unwrap_or(props.default));

    let mut ctx = use_context::<SegmentFieldContext>();

    let mut set_value = move |text: String| {
        if text.is_empty() {
            props.on_value_change.call(None);
            ctx.focus.focus_prev();
            return;
        }
        let min = props.min.cloned();
        let max = props.max.cloned();

        let value = text.parse::<T>().map(|v| v.min(max)).ok();
        if let Some(value) = value {
            let in_range = value >= min && value <= max;

            // If adding a new digit would exceed max, move to next segment
            let new_value = (text + "0").parse::<T>().unwrap_or(value);
            if in_range && new_value > max {
                ctx.focus.focus_next();
            }
        };

        props.on_value_change.call(value);
    };
    use_effect(move || {
        // If this item is not focused, always keep the value clamped
        if !ctx.focus.is_focused(props.index.cloned())
            && let Some(value) = (props.value)()
        {
            let clamped_value = value.clamp(props.min.cloned(), props.max.cloned());
            if clamped_value != value {
                props.on_value_change.call(Some(clamped_value));
            }
        }
    });

    let roll_value = move |value: T| {
        let min = props.min.cloned();
        let max = props.max.cloned();
        if value < min {
            max
        } else if value > max {
            min
        } else {
            value
        }
    };

    let handle_keydown = move |event: Event<KeyboardData>| {
        if (ctx.disabled)() {
            return;
        }
        let read_only = (ctx.read_only)();
        let key = event.key();
        match key {
            Key::Character(actual_char) => {
                if read_only {
                    return;
                }
                // Don't block keyboard shortcuts
                if event.modifiers().ctrl() || event.modifiers().meta() || event.modifiers().alt() {
                    return;
                }
                if actual_char.parse::<T>().is_ok() {
                    let mut text = text_value();
                    if text.len() == props.max_length || reset_value() {
                        text = String::default();
                        reset_value.set(false);
                    };
                    text.push_str(&actual_char);
                    set_value(text);
                }
                event.prevent_default();
                event.stop_propagation();
            }
            Key::Backspace => {
                if read_only {
                    return;
                }
                let mut text = text_value();
                if event.modifiers().ctrl() || event.modifiers().meta() {
                    text.clear();
                } else {
                    text.pop();
                }
                set_value(text);
            }
            Key::Delete => {
                if read_only {
                    return;
                }
                let mut text = text_value();
                text.remove(0);
                set_value(text);
            }
            Key::ArrowLeft => {
                ctx.focus.focus_prev();
            }
            Key::ArrowRight => {
                ctx.focus.focus_next();
            }
            Key::Enter => {
                ctx.focus.focus_next();
                event.prevent_default();
                event.stop_propagation();
            }
            Key::ArrowUp => {
                if read_only {
                    return;
                }
                let value = match (props.value)() {
                    Some(mut value) => {
                        value.inc();
                        roll_value(value)
                    }
                    None => props.default,
                };
                props.on_value_change.call(Some(value));
            }
            Key::ArrowDown => {
                if read_only {
                    return;
                }
                let value = match (props.value)() {
                    Some(mut value) => {
                        value.dec();
                        roll_value(value)
                    }
                    None => props.default,
                };
                props.on_value_change.call(Some(value));
            }
            _ => (),
        }
    };

    let disabled = move || (ctx.disabled)();
    let onmounted =
        use_item(collection_item(ctx.focus, props.index).disabled(disabled)).onmounted();

    let span_id = use_unique_id();
    let id = use_memo(move || format!("span-{span_id}"));
    let label_id = format!("{id}-label");

    rsx! {
        span {
            id,
            role: "spinbutton",
            aria_valuemin: props.min.to_string(),
            aria_valuemax: props.max.to_string(),
            aria_valuenow: now_value.to_string(),
            aria_labelledby: "{label_id}",
            inputmode: "numeric",
            contenteditable: !(ctx.read_only)(),
            spellcheck: false,
            tabindex: "0",
            enterkeyhint: "next",
            onkeydown: handle_keydown,
            onmounted,
            onfocus: move |_| {
                reset_value.set(true);
                ctx.focus.set_focus(Some(props.index.cloned()));
                props.on_focus.call(());
            },
            // `no-date`, not `no-value`: kept exactly as this attribute was
            // named before this primitive was extracted out of
            // `date_picker.rs`, since `registry/ui/date_picker.rs`'s own
            // styling already selects on `[role=spinbutton][no-date=true]`
            // for the placeholder-text appearance -- renaming it would
            // silently break that CSS with no compile-time signal. A time
            // segment carrying this date-shaped name is a cosmetic wart, not
            // a behavior bug; `data-empty` below is the generically-named
            // equivalent for new styling (e.g. `time-picker`'s) to use
            // instead.
            "no-date": (props.value)().is_none(),
            "data-empty": (props.value)().is_none(),
            "data-disabled": (ctx.disabled)(),
            ..props.attributes,
            {display_value}
        }
    }
}

/// The props for [`MeridiemSegment`].
#[derive(Props, Clone, PartialEq)]
pub(crate) struct MeridiemSegmentProps {
    /// The index of this segment within its enclosing field's roving-focus
    /// sequence.
    pub index: ReadSignal<usize>,

    /// Whether the segment currently reads PM (`true`) or AM (`false`).
    /// `None` when no time value is set yet.
    pub value: ReadSignal<Option<bool>>,

    /// Callback when the value changes.
    #[props(default)]
    pub on_value_change: Callback<bool>,

    /// Called when this segment receives focus, after the roving-focus
    /// bookkeeping.
    #[props(default)]
    pub on_focus: Callback<()>,

    /// Additional attributes for the segment element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// A two-value (AM/PM) segment, sharing [`NumericSegment`]'s roving-focus
/// sequence and keyboard shape (arrow keys move between segments, ArrowUp/
/// ArrowDown and `A`/`P` toggle the value) but exposing `aria-valuetext`
/// instead of a numeric ARIA value, since there is nothing numeric to
/// announce.
#[component]
pub(crate) fn MeridiemSegment(props: MeridiemSegmentProps) -> Element {
    let mut ctx = use_context::<SegmentFieldContext>();

    let is_pm = use_memo(move || (props.value)().unwrap_or(false));
    let value_text = use_memo(move || match (props.value)() {
        Some(true) => "PM",
        Some(false) => "AM",
        None => "--",
    });

    let handle_keydown = move |event: Event<KeyboardData>| {
        if (ctx.disabled)() {
            return;
        }
        let read_only = (ctx.read_only)();
        match event.key() {
            Key::ArrowLeft => ctx.focus.focus_prev(),
            Key::ArrowRight => ctx.focus.focus_next(),
            Key::ArrowUp | Key::ArrowDown => {
                if read_only {
                    return;
                }
                props.on_value_change.call(!is_pm());
                event.prevent_default();
            }
            Key::Character(actual_char) => {
                if read_only {
                    return;
                }
                match actual_char.to_ascii_lowercase().as_str() {
                    "a" => props.on_value_change.call(false),
                    "p" => props.on_value_change.call(true),
                    _ => return,
                }
                event.prevent_default();
                event.stop_propagation();
            }
            Key::Enter => {
                ctx.focus.focus_next();
                event.prevent_default();
                event.stop_propagation();
            }
            _ => (),
        }
    };

    let disabled = move || (ctx.disabled)();
    let onmounted =
        use_item(collection_item(ctx.focus, props.index).disabled(disabled)).onmounted();

    rsx! {
        span {
            role: "spinbutton",
            "aria-valuetext": "{value_text}",
            aria_label: "meridiem",
            contenteditable: !(ctx.read_only)(),
            spellcheck: false,
            tabindex: "0",
            enterkeyhint: "next",
            onkeydown: handle_keydown,
            onmounted,
            onfocus: move |_| {
                ctx.focus.set_focus(Some(props.index.cloned()));
                props.on_focus.call(());
            },
            "data-empty": (props.value)().is_none(),
            "data-disabled": (ctx.disabled)(),
            ..props.attributes,
            "{value_text}"
        }
    }
}
