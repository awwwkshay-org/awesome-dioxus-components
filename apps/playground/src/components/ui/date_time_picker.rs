//! Styled Date+Time Picker, composing the installed `date-picker` and
//! `time-picker` registry items behind one trigger showing the full
//! formatted value. Manages its own single popover (not `DatePickerPopover`
//! or a `TimePickerPopover`) so one popup surface holds both the calendar
//! and the time-selection UI -- `DatePicker`/`TimePicker` (bare, unwrapped
//! by either item's own popover) still work standalone this way, since each
//! primitive's own popover-adjacent state (`BaseDatePickerContext.open`) is
//! a harmless no-op signal when nothing reads it.
//!
//! No shared roving-focus sequence between the date and time segments is
//! needed for correct Tab order: every segment renders a plain, unmanaged
//! `tabindex="0"` (see `segment::NumericSegment`), so native browser Tab
//! order already flows correctly from the date's segments into the time's
//! regardless of which `CollectionState` registered each one -- only
//! ArrowLeft/ArrowRight (which stay within one composed field's own
//! registered segments) are scoped per-picker, which is the same behavior a
//! user gets from two adjacent composite fields in any other form.

use dioxus::prelude::*;
use time::{Date, PrimitiveDateTime, Time};

use super::date_picker::DatePicker;
use super::popover::{PopoverContent, PopoverTrigger};
use super::time_picker::{TimePicker, TimePickerView};
use crate::adico_lib::cn::cn;
use adico_primitives::icons::ChevronDown;
use adico_primitives::popover::{PopoverRoot, PopoverRootProps};
use adico_primitives::time_picker::format_time;

/// Props for the styled date-and-time picker root.
#[derive(Props, Clone, PartialEq)]
pub struct DateTimePickerProps {
    #[props(default)]
    pub on_value_change: Callback<Option<PrimitiveDateTime>>,
    #[props(default)]
    pub selected_datetime: ReadSignal<Option<PrimitiveDateTime>>,
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub read_only: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub is_12h: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub show_seconds: ReadSignal<bool>,
    /// Which popup body [`TimePickerBody`](super::time_picker::TimePickerBody)
    /// renders inside this `DateTimePicker` -- see
    /// [`TimePickerView`](super::time_picker::TimePickerView).
    #[props(default = ReadSignal::new(Signal::new(TimePickerView::Digital)))]
    pub view: ReadSignal<TimePickerView>,
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Pure reconciliation rule shared by [`DateTimePicker`] and its tests: the
/// combined value is `Some` only once both halves are known.
fn combine(date: Option<Date>, time: Option<Time>) -> Option<PrimitiveDateTime> {
    match (date, time) {
        (Some(d), Some(t)) => Some(PrimitiveDateTime::new(d, t)),
        _ => None,
    }
}

/// A registry-only display context so [`DateTimePickerTrigger`] can show the
/// combined value without the consumer threading a second copy of
/// `selected_datetime`/`is_12h`/`show_seconds` through
/// [`DateTimePickerPopover`] -- [`DateTimePicker`] already owns all three,
/// having just combined `last_date`/`last_time` itself.
#[derive(Clone, Copy)]
struct DateTimePickerDisplayContext {
    value: Memo<Option<PrimitiveDateTime>>,
    is_12h: ReadSignal<bool>,
    show_seconds: ReadSignal<bool>,
}

/// A styled date-and-time picker: composes the installed `DatePicker` and
/// `TimePicker` over one combined value. The combined value is `Some` only
/// once both a date and a time are set.
///
/// `last_date`/`last_time` buffer each half internally rather than deriving
/// both from `props.selected_datetime` on every change: completing the date
/// first computes against a still-unset time and so reports `None` back to
/// the caller, which means the external `selected_datetime` prop never
/// becomes `Some` from that call alone -- deriving the "other half" purely
/// from that still-`None` prop would then discard the date the user just
/// entered the moment they moved on to the time, and the two halves could
/// never combine. The buffers persist whichever half completes first; an
/// effect still resyncs them from `props.selected_datetime`, but only when
/// that prop is `Some` -- resyncing on every `None` too would risk wiping a
/// buffered half if a controlled consumer's `on_value_change` handler ever
/// re-set the prop to the same `None` it already held (Dioxus signal-change
/// notification on an equal value isn't a contract this file relies on).
/// This means an external caller cannot clear a fully-set value back to
/// `None` by resetting the prop alone; remounting via a `key` is the escape
/// hatch for that case.
#[component]
pub fn DateTimePicker(props: DateTimePickerProps) -> Element {
    let mut last_date = use_signal(move || props.selected_datetime.cloned().map(|dt| dt.date()));
    let mut last_time = use_signal(move || props.selected_datetime.cloned().map(|dt| dt.time()));

    use_effect(move || {
        if let Some(value) = props.selected_datetime.cloned() {
            last_date.set(Some(value.date()));
            last_time.set(Some(value.time()));
        }
    });

    let on_date_change = move |date: Option<Date>| {
        last_date.set(date);
        props.on_value_change.call(combine(date, last_time()));
    };
    let on_time_change = move |time: Option<Time>| {
        last_time.set(time);
        props.on_value_change.call(combine(last_date(), time));
    };

    let value = use_memo(move || combine(last_date(), last_time()));
    use_context_provider(|| DateTimePickerDisplayContext {
        value,
        is_12h: props.is_12h,
        show_seconds: props.show_seconds,
    });

    let class = cn(&[
        "relative inline-block",
        props.class.as_deref().unwrap_or_default(),
    ]);

    rsx! {
        DatePicker {
            selected_date: last_date(),
            on_value_change: on_date_change,
            disabled: props.disabled,
            read_only: props.read_only,
            TimePicker {
                selected_time: last_time(),
                on_value_change: on_time_change,
                disabled: props.disabled,
                read_only: props.read_only,
                is_12h: props.is_12h,
                show_seconds: props.show_seconds,
                view: props.view,
                class,
                attributes: props.attributes,
                {props.children}
            }
        }
    }
}

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Props, Clone, PartialEq)]
pub struct DateTimePickerPopoverProps {
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

/// The single popover both the calendar and time-selection UI render into.
/// Takes the same `popover_root` injection prop as `date-picker`'s
/// `DatePickerPopover`, `color-picker`'s `ColorPickerPopover`, and
/// `time-picker`'s `TimePickerPopover`. Must be used inside a
/// [`DateTimePicker`] (for [`DateTimePickerDisplayContext`], read by
/// [`DateTimePickerTrigger`]).
#[component]
pub fn DateTimePickerPopover(props: DateTimePickerPopoverProps) -> Element {
    let class = cn(&[
        "group/date-time-picker",
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
pub struct DateTimePickerTriggerProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

/// Disclosure trigger showing the combined date-and-time value (via
/// [`DateTimePickerDisplayContext`]) -- falling back to "Pick date & time".
/// Must be used inside a [`DateTimePicker`] (for the context).
#[component]
pub fn DateTimePickerTrigger(props: DateTimePickerTriggerProps) -> Element {
    let ctx = use_context::<DateTimePickerDisplayContext>();
    let label = use_memo(move || match ctx.value.cloned() {
        Some(dt) => format!(
            "{} {}",
            dt.date(),
            format_time(dt.time(), (ctx.is_12h)(), (ctx.show_seconds)())
        ),
        None => "Pick date & time".to_string(),
    });
    let class = cn(&[
        "inline-flex h-9 items-center gap-2 rounded-md border border-input bg-background px-3 text-sm shadow-xs transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50",
        props.class.as_deref().unwrap_or_default(),
    ]);
    let children = props.children.unwrap_or_else(|| {
        rsx! {
            "{label()}"
            ChevronDown {
                class: "size-4 shrink-0 text-muted-foreground transition-transform duration-200 group-data-[state=open]/date-time-picker:rotate-180",
                size: 16,
            }
        }
    });
    rsx! {
        PopoverTrigger { class, attributes: props.attributes, {children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DateTimePickerContentProps {
    #[props(default)]
    pub class: Option<String>,
    pub children: Element,
}

/// Popup shell with no duplicate frame -- the composed calendar/columns own
/// the surface, matching `DatePickerContent`/`TimePickerContent`'s
/// equivalent role.
#[component]
pub fn DateTimePickerContent(props: DateTimePickerContentProps) -> Element {
    // `!` overrides win regardless of Tailwind's compiled rule order (`cn` is a plain
    // join, not a tailwind-merge dedupe -- see `registry/lib/cn.rs`), which matters here:
    // `PopoverContent`'s base `w-72`/`overflow-y-auto` must lose. `overflow-y-auto` alone
    // forces `overflow-x` to compute `auto` too (CSS Overflow §3), which would clip the
    // `sm:flex-row` calendar+time-columns row that's deliberately wider than `w-72` -- the
    // time column already owns its own internal scroll (`time_picker.rs`'s
    // `scroll_area_visibility_class`), so this popover doesn't need an outer scroll cap.
    let class = cn(&[
        "flex w-auto! max-h-none! flex-col gap-3 overflow-visible! border-0 bg-transparent p-0 shadow-none sm:flex-row",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! { PopoverContent { class, {props.children} } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combined_value_is_none_unless_both_halves_are_set() {
        let date = Date::from_calendar_date(2026, time::Month::September, 6).unwrap();
        let time = Time::from_hms(14, 30, 0).unwrap();
        assert_eq!(
            combine(Some(date), Some(time)),
            Some(PrimitiveDateTime::new(date, time))
        );
        assert_eq!(combine(Some(date), None), None);
        assert_eq!(combine(None, Some(time)), None);
        assert_eq!(combine(None, None), None);
    }

    #[test]
    fn sequential_partial_updates_still_combine_once_both_halves_are_known() {
        // Regression test for a bug where DateTimePicker derived both halves
        // from `props.selected_datetime` on every change: completing the
        // date first computes against a still-unset time and reports `None`,
        // so the external prop never becomes `Some` from that call alone --
        // deriving the "other half" purely from that still-`None` prop would
        // then discard the date the moment the user moved on to the time,
        // and the two halves could never combine. `last_date`/`last_time`
        // must instead buffer each half internally so whichever completes
        // first survives until the other one does too.
        let date = Date::from_calendar_date(2026, time::Month::September, 6).unwrap();
        let time = Time::from_hms(14, 30, 0).unwrap();

        // Date completes first; time is still unset.
        let last_date = Some(date);
        let mut last_time = None;
        assert_eq!(combine(last_date, last_time), None);

        // Time completes second -- the date buffered above must still be
        // remembered, not re-derived from the still-`None` combined value.
        last_time = Some(time);
        assert_eq!(
            combine(last_date, last_time),
            Some(PrimitiveDateTime::new(date, time))
        );
    }
}
