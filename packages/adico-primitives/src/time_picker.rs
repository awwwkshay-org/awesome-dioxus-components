// No dedicated WAI-ARIA APG "Time Picker" pattern exists, and no upstream
// this project mirrors (shadcn, Base UI, dioxus-components, dioxus-primitives)
// has a time picker either -- this is a net-new `ADICO_ONLY_EXTRA`. Follows
// the same shape `date_picker.rs` already established for exactly this
// composed-segmented-field problem: a `role="group"` field containing one
// `role="spinbutton"` segment per hour/minute(/second) component (via the
// shared `segment` module's `NumericSegment`, extracted from `date_picker.rs`
// for this reuse), plus a non-numeric meridiem segment
// (`segment::MeridiemSegment`) when displaying 12-hour time.
//
// The clock-dial view has no ARIA pattern of its own (an analog dial has no
// APG equivalent) -- this is precisely why the columns view and the
// segmented input are the accessible, keyboard/AT-operable paths, and the
// dial is pointer-only enhancement over the same value, never the only way
// to set it. See `add-time-picker-components`'s design.md for the full
// rationale.

//! Defines the [`TimePicker`] component and its subcomponents: a segmented
//! text field (hour/minute/second/meridiem, via `segment`), a scrollable
//! columns view, and an analog clock dial -- three ways to set the same
//! underlying time value.

use crate::{
    LocalDateExt as _,
    date_picker::DatePickerSeparator,
    move_interaction::use_move_interaction,
    segment::{MeridiemSegment, NumericSegment, use_segment_field_provider},
};

use dioxus::html::geometry::ClientPoint;
use dioxus::prelude::*;
use time::{OffsetDateTime, Time};

/// The context provided by the [`TimePicker`] component to its children.
/// `pub`, with read-only accessors, following the same shape
/// `color_picker::ColorPickerContext` already established: a registry
/// facade's trigger needs to read the live selected value (to show it as
/// the trigger's own label) without the primitive exposing mutable access
/// to its internals.
#[derive(Copy, Clone)]
pub struct TimePickerContext {
    on_value_change: Callback<Option<Time>>,
    selected_time: ReadSignal<Option<Time>>,
    is_12h: ReadSignal<bool>,
    show_seconds: ReadSignal<bool>,
}

impl TimePickerContext {
    fn set_time(&mut self, time: Option<Time>) {
        if self.selected_time.peek().cloned() != time {
            self.on_value_change.call(time);
        }
    }

    /// Replace the entire selected time -- e.g. a columns or clock-dial view
    /// setting a whole new hour/minute pair at once, rather than a single
    /// segment's own value changing.
    pub fn set_selected_time(&mut self, time: Option<Time>) {
        self.set_time(time);
    }

    /// The currently selected time, if any.
    pub fn selected_time(&self) -> Option<Time> {
        (self.selected_time)()
    }

    /// Whether this picker displays/accepts 12-hour time with a meridiem
    /// segment, rather than 24-hour time.
    pub fn is_12h(&self) -> bool {
        (self.is_12h)()
    }

    /// Whether this picker includes a seconds segment/column.
    pub fn show_seconds(&self) -> bool {
        (self.show_seconds)()
    }
}

/// The props for the [`TimePicker`] component.
#[derive(Props, Clone, PartialEq)]
pub struct TimePickerProps {
    /// Callback when value changes.
    #[props(default)]
    pub on_value_change: Callback<Option<Time>>,

    /// The selected time.
    #[props(default)]
    pub selected_time: ReadSignal<Option<Time>>,

    /// Whether the time picker is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether the time picker allows user input.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub read_only: ReadSignal<bool>,

    /// Whether to display (and accept typed input as) 12-hour time with a
    /// meridiem segment, rather than 24-hour time.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub is_12h: ReadSignal<bool>,

    /// Whether to include a seconds segment/column.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub show_seconds: ReadSignal<bool>,

    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub roving_loop: ReadSignal<bool>,

    /// Additional attributes to extend the time picker element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the time picker element.
    pub children: Element,
}

/// # TimePicker
///
/// The [`TimePicker`] component provides an accessible time input interface:
/// a segmented text field is always available; a scrollable columns view and
/// an analog clock dial (see [`TimePickerColumns`]/[`TimePickerClock`]) are
/// additional, pointer-friendly ways to set the same value.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::time_picker::*;
/// use time::Time;
/// #[component]
/// fn Demo() -> Element {
///     let mut selected_time = use_signal(|| None::<Time>);
///     rsx! {
///         TimePicker {
///             selected_time: selected_time(),
///             on_value_change: move |time| selected_time.set(time),
///             TimePickerInput {}
///         }
///     }
/// }
/// ```
///
/// # Styling
///
/// The [`TimePicker`] component defines the following data attributes you
/// can use to control styling:
/// - `data-disabled`: Indicates if the TimePicker is disabled. Possible
///   values are `true` or `false`.
#[component]
pub fn TimePicker(props: TimePickerProps) -> Element {
    use_segment_field_provider(props.roving_loop, props.disabled, props.read_only);

    use_context_provider(|| TimePickerContext {
        on_value_change: props.on_value_change,
        selected_time: props.selected_time,
        is_12h: props.is_12h,
        show_seconds: props.show_seconds,
    });

    rsx! {
        div {
            role: "group",
            aria_label: "Time",
            "data-disabled": (props.disabled)(),
            ..props.attributes,
            {props.children}
        }
    }
}

/// Formats `time` as `"H:MM"`/`"HH:MM"` (24-hour) or `"H:MM AM/PM"` (12-hour),
/// with an additional `:SS` when `show_seconds`. A registry trigger uses
/// this to show the currently selected time as its own label -- pure and
/// context-free, so it needs only the same `is_12h`/`show_seconds` values
/// already passed to the styled `TimePicker` wrapper, not a second read of
/// [`TimePickerContext`].
pub fn format_time(time: Time, is_12h: bool, show_seconds: bool) -> String {
    let (hour, suffix) = if is_12h {
        let (display, is_pm) = to_display_hour(time.hour());
        (display, if is_pm { " PM" } else { " AM" })
    } else {
        (time.hour(), "")
    };
    if show_seconds {
        format!("{hour}:{:02}:{:02}{suffix}", time.minute(), time.second())
    } else {
        format!("{hour}:{:02}{suffix}", time.minute())
    }
}

/// Converts a canonical 24-hour `hour` (`0..=23`) into the value and
/// AM/PM-ness a 12-hour segment displays (`1..=12`, `false` = AM).
pub fn to_display_hour(hour24: u8) -> (u8, bool) {
    let is_pm = hour24 >= 12;
    let display = match hour24 % 12 {
        0 => 12,
        h => h,
    };
    (display, is_pm)
}

/// The inverse of [`to_display_hour`]: reconstructs a canonical 24-hour hour
/// from a 12-hour display value and AM/PM-ness.
pub fn from_display_hour(display: u8, is_pm: bool) -> u8 {
    let base = display % 12;
    if is_pm { base + 12 } else { base }
}

#[derive(Clone, Copy)]
struct TimeElementContext {
    hour_index: usize,
    minute_index: usize,
    second_index: Memo<Option<usize>>,
    meridiem_index: Memo<Option<usize>>,
    hour_value: Signal<Option<u8>>,
    minute_value: Signal<Option<u8>>,
    second_value: Signal<Option<u8>>,
    is_pm_value: Signal<Option<bool>>,
    is_12h: ReadSignal<bool>,
    on_format_hour_placeholder: Callback<(), String>,
    on_format_minute_placeholder: Callback<(), String>,
    on_format_second_placeholder: Callback<(), String>,
}

#[derive(Props, Clone, PartialEq)]
struct TimeElementProps {
    /// The start index (used for focus), so a future composed field (e.g. a
    /// date-and-time field) can lay time segments out after its own.
    #[props(default = 0)]
    pub start_index: usize,

    /// The selected time.
    pub selected_time: ReadSignal<Option<Time>>,

    /// Callback when selected time changes.
    #[props(default)]
    pub on_time_change: Callback<Option<Time>>,

    #[props(default = Callback::new(|_| "H".to_string()))]
    pub on_format_hour_placeholder: Callback<(), String>,

    #[props(default = Callback::new(|_| "M".to_string()))]
    pub on_format_minute_placeholder: Callback<(), String>,

    #[props(default = Callback::new(|_| "S".to_string()))]
    pub on_format_second_placeholder: Callback<(), String>,

    #[props(default)]
    pub children: Option<Element>,
}

#[component]
fn TimeElement(props: TimeElementProps) -> Element {
    let ctx = use_context::<TimePickerContext>();
    let is_12h = ctx.is_12h;
    let show_seconds = ctx.show_seconds;
    let selected = props.selected_time.peek().cloned();

    // `hour_value` is always in the *display* domain: 24-hour (0..=23) when
    // `!is_12h`, matching `Time::hour()` directly; 12-hour (1..=12) when
    // `is_12h`, via `to_display_hour`. Getting this branch wrong here is
    // exactly the bug this comment is guarding against -- `to_display_hour`
    // must never run unconditionally, or a 24-hour picker would display and
    // reconstruct the wrong hour (found live: a 14:30 selection rendered as
    // hour "2" instead of "14" before this was gated).
    let to_hour_value = move |hour24: u8| -> (u8, Option<bool>) {
        if is_12h() {
            let (display, pm) = to_display_hour(hour24);
            (display, Some(pm))
        } else {
            (hour24, None)
        }
    };

    let initial_hour = selected.map(|t| to_hour_value(t.hour()));
    let mut hour_value = use_signal(move || initial_hour.map(|(h, _)| h));
    let mut is_pm_value = use_signal(move || initial_hour.and_then(|(_, pm)| pm));
    let mut minute_value = use_signal(move || selected.map(|t| t.minute()));
    let mut second_value = use_signal(move || selected.map(|t| t.second()));

    // External value changes overwrite the segments -- mirrors
    // `DateElement`'s identical sync effect.
    use_effect(move || match (props.selected_time)() {
        Some(t) => {
            let (display, pm) = to_hour_value(t.hour());
            hour_value.set(Some(display));
            is_pm_value.set(pm);
            minute_value.set(Some(t.minute()));
            second_value.set(Some(t.second()));
        }
        None => {
            hour_value.set(None);
            is_pm_value.set(None);
            minute_value.set(None);
            second_value.set(None);
        }
    });

    // Segments reassemble into a canonical `Time` -- mirrors `DateElement`'s
    // identical reconciliation effect (there: `Date::from_calendar_date`).
    use_effect(move || {
        if let (Some(hour_display), Some(minute)) = (hour_value(), minute_value()) {
            let second = if show_seconds() {
                second_value().unwrap_or(0)
            } else {
                0
            };
            let hour24 = if is_12h() {
                from_display_hour(hour_display, is_pm_value().unwrap_or(false))
            } else {
                hour_display
            };
            if let Ok(time) = Time::from_hms(hour24, minute, second) {
                props.on_time_change.call(Some(time));
            }
        }
    });

    let second_index = use_memo(move || show_seconds().then_some(props.start_index + 2));
    let meridiem_index =
        use_memo(move || is_12h().then_some(props.start_index + 2 + show_seconds() as usize));

    use_context_provider(|| TimeElementContext {
        hour_index: props.start_index,
        minute_index: props.start_index + 1,
        second_index,
        meridiem_index,
        hour_value,
        minute_value,
        second_value,
        is_pm_value,
        is_12h,
        on_format_hour_placeholder: props.on_format_hour_placeholder,
        on_format_minute_placeholder: props.on_format_minute_placeholder,
        on_format_second_placeholder: props.on_format_second_placeholder,
    });

    let children = props.children.unwrap_or_else(|| {
        rsx! {
            TimePickerHourSegment {}
            DatePickerSeparator { symbol: ':' }
            TimePickerMinuteSegment {}
            if show_seconds() {
                DatePickerSeparator { symbol: ':' }
                TimePickerSecondSegment {}
            }
            if is_12h() {
                span { aria_hidden: "true", tabindex: "-1", "is-separator": true, " " }
                TimePickerMeridiemSegment {}
            }
        }
    });

    rsx! {
        {children}
    }
}

/// The props for the [`TimePickerHourSegment`] component.
#[derive(Props, Clone, PartialEq)]
pub struct TimePickerHourSegmentProps {
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// An hour segment in a time input. Its range is `1..=12` when the
/// enclosing [`TimePicker`] is `is_12h`, otherwise `0..=23`.
#[component]
pub fn TimePickerHourSegment(props: TimePickerHourSegmentProps) -> Element {
    let mut ctx = use_context::<TimeElementContext>();
    let is_12h = ctx.is_12h;
    let min = use_memo(move || if is_12h() { 1u8 } else { 0u8 });
    let max = use_memo(move || if is_12h() { 12u8 } else { 23u8 });
    let default_value = use_memo(move || if is_12h() { 12u8 } else { 0u8 });

    rsx! {
        NumericSegment {
            aria_label: "hour",
            index: ctx.hour_index,
            value: ctx.hour_value,
            default: default_value(),
            on_value_change: move |value: Option<u8>| ctx.hour_value.set(value),
            min: min(),
            max: max(),
            max_length: 2,
            on_format_placeholder: ctx.on_format_hour_placeholder,
            attributes: props.attributes,
        }
    }
}

/// The props for the [`TimePickerMinuteSegment`] component.
#[derive(Props, Clone, PartialEq)]
pub struct TimePickerMinuteSegmentProps {
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// A minute segment in a time input (`0..=59`).
#[component]
pub fn TimePickerMinuteSegment(props: TimePickerMinuteSegmentProps) -> Element {
    let mut ctx = use_context::<TimeElementContext>();

    rsx! {
        NumericSegment {
            aria_label: "minute",
            index: ctx.minute_index,
            value: ctx.minute_value,
            default: 0u8,
            on_value_change: move |value: Option<u8>| ctx.minute_value.set(value),
            min: 0u8,
            max: 59u8,
            max_length: 2,
            on_format_placeholder: ctx.on_format_minute_placeholder,
            attributes: props.attributes,
        }
    }
}

/// The props for the [`TimePickerSecondSegment`] component.
#[derive(Props, Clone, PartialEq)]
pub struct TimePickerSecondSegmentProps {
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// A second segment in a time input (`0..=59`). Meaningful only while the
/// enclosing [`TimePicker`] has `show_seconds` set -- see
/// [`TimeElementContext::second_index`]'s fallback for the (unusual) case of
/// rendering this without `show_seconds`.
#[component]
pub fn TimePickerSecondSegment(props: TimePickerSecondSegmentProps) -> Element {
    let mut ctx = use_context::<TimeElementContext>();
    let index = use_memo(move || (ctx.second_index)().unwrap_or(ctx.minute_index + 1));

    rsx! {
        NumericSegment {
            aria_label: "second",
            index: index(),
            value: ctx.second_value,
            default: 0u8,
            on_value_change: move |value: Option<u8>| ctx.second_value.set(value),
            min: 0u8,
            max: 59u8,
            max_length: 2,
            on_format_placeholder: ctx.on_format_second_placeholder,
            attributes: props.attributes,
        }
    }
}

/// The props for the [`TimePickerMeridiemSegment`] component.
#[derive(Props, Clone, PartialEq)]
pub struct TimePickerMeridiemSegmentProps {
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// An AM/PM segment in a time input. Meaningful only while the enclosing
/// [`TimePicker`] has `is_12h` set.
#[component]
pub fn TimePickerMeridiemSegment(props: TimePickerMeridiemSegmentProps) -> Element {
    let mut ctx = use_context::<TimeElementContext>();
    let index = use_memo(move || (ctx.meridiem_index)().unwrap_or(ctx.minute_index + 1));

    rsx! {
        MeridiemSegment {
            index: index(),
            value: ctx.is_pm_value,
            on_value_change: move |value: bool| ctx.is_pm_value.set(Some(value)),
            attributes: props.attributes,
        }
    }
}

/// The props for the [`TimePickerInputValue`] component.
#[derive(Props, Clone, PartialEq)]
pub struct TimePickerInputValueProps {
    #[props(default = Callback::new(|_| "H".to_string()))]
    pub on_format_hour_placeholder: Callback<(), String>,

    #[props(default = Callback::new(|_| "M".to_string()))]
    pub on_format_minute_placeholder: Callback<(), String>,

    #[props(default = Callback::new(|_| "S".to_string()))]
    pub on_format_second_placeholder: Callback<(), String>,

    /// The children of the time value. Defaults to hour/minute(/second)(/meridiem)
    /// segments, matching the enclosing [`TimePicker`]'s configuration.
    #[props(default)]
    pub children: Option<Element>,
}

/// The editable time value for a [`TimePicker`] input.
#[component]
pub fn TimePickerInputValue(props: TimePickerInputValueProps) -> Element {
    let mut ctx = use_context::<TimePickerContext>();

    rsx! {
        TimeElement {
            selected_time: ctx.selected_time,
            on_time_change: move |time| ctx.set_time(time),
            on_format_hour_placeholder: props.on_format_hour_placeholder,
            on_format_minute_placeholder: props.on_format_minute_placeholder,
            on_format_second_placeholder: props.on_format_second_placeholder,
            children: props.children,
        }
    }
}

/// The props for the [`TimePickerInput`] component.
#[derive(Props, Clone, PartialEq)]
pub struct TimePickerInputProps {
    #[props(default = Callback::new(|_| "H".to_string()))]
    pub on_format_hour_placeholder: Callback<(), String>,

    #[props(default = Callback::new(|_| "M".to_string()))]
    pub on_format_minute_placeholder: Callback<(), String>,

    #[props(default = Callback::new(|_| "S".to_string()))]
    pub on_format_second_placeholder: Callback<(), String>,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    #[props(default)]
    pub children: Option<Element>,
}

/// # TimePickerInput
///
/// The input element for the [`TimePicker`] component, rendering the
/// segmented time value by default.
#[component]
pub fn TimePickerInput(props: TimePickerInputProps) -> Element {
    let children = props.children.unwrap_or_else(|| {
        rsx! {
            TimePickerInputValue {
                on_format_hour_placeholder: props.on_format_hour_placeholder,
                on_format_minute_placeholder: props.on_format_minute_placeholder,
                on_format_second_placeholder: props.on_format_second_placeholder,
            }
        }
    });

    rsx! {
        div { ..props.attributes,
            {children}
        }
    }
}

/// Converts a pointer position into the nearest whole value in `0..range`
/// around a dial centered at `center`, with `0` at the top (12 o'clock)
/// increasing clockwise -- matching a real clock face. Pure and independent
/// of any DOM/event machinery, so it's directly unit-testable. `pub`: the
/// registry's `TimePickerClock` calls this directly with a click/tap event's
/// own coordinates (click-to-set) rather than through [`ClockDial`]'s
/// continuous-drag tracking, which depends on `pointer.rs`'s global
/// pointer-position registry -- documented there as unverified-in-a-live-browser
/// (the same caveat `slider`/`color_picker`'s dragging already carries).
/// Click-to-set needs none of that: only the event's own coordinates and the
/// dial's rect. [`ClockDial`] remains available for a future consumer that
/// wants full continuous-drag tracking once that registry is confirmed
/// working live.
pub fn angle_to_value(center: ClientPoint, pointer: ClientPoint, range: u32) -> u32 {
    let dx = pointer.x - center.x;
    let dy = pointer.y - center.y;
    // `atan2(x, -y)` rather than the usual `atan2(y, x)`: this measures the
    // angle clockwise from the top (screen `y` grows downward, so negating
    // it before the standard counterclockwise-from-positive-x atan2 flips
    // the result to clockwise-from-the-top).
    let degrees = dx.atan2(-dy).to_degrees();
    let normalized = if degrees < 0.0 {
        degrees + 360.0
    } else {
        degrees
    };
    let step = 360.0 / range as f64;
    (normalized / step).round() as u32 % range
}

/// Headless state for [`TimePickerClock`]'s dial: converts pointer
/// interaction into hour/minute values via [`angle_to_value`], built on
/// [`crate::move_interaction::use_move_interaction`] for element-rect and
/// active-pointer tracking rather than a new pointer primitive.
#[derive(Copy, Clone)]
pub struct ClockDial {
    movement: crate::move_interaction::MoveInteraction,
}

/// A dial's current selection mode: which value a pointer interaction
/// currently sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockDialUnit {
    /// The dial sets the hour (range 12 if `is_12h`, else 24).
    Hour,
    /// The dial sets the minute (range 60).
    Minute,
}

impl ClockDialUnit {
    fn range(self, is_12h: bool) -> u32 {
        match self {
            Self::Hour if is_12h => 12,
            Self::Hour => 24,
            Self::Minute => 60,
        }
    }
}

/// Builds a fresh [`ClockDial`] for [`TimePickerClock`] to drive.
pub fn use_clock_dial() -> ClockDial {
    let dragging = use_signal(|| false);
    ClockDial {
        movement: use_move_interaction(dragging),
    }
}

impl ClockDial {
    /// The dial's own [`MoveInteraction`](crate::move_interaction::MoveInteraction),
    /// for wiring `onmounted`/`onresize`/`onpointerdown`/`onpointerup`
    /// exactly like any other `move_interaction`-based control (see
    /// `color_picker.rs`'s `ColorArea` for the established wiring pattern).
    pub fn movement(&mut self) -> &mut crate::move_interaction::MoveInteraction {
        &mut self.movement
    }

    /// Reads the current pointer position and this dial's element rect,
    /// returning the value in `0..unit.range()` the pointer is currently
    /// over -- `None` if a drag isn't active or the rect isn't known yet.
    pub fn value_at_pointer(&self, unit: ClockDialUnit, is_12h: bool) -> Option<u32> {
        let rect = self.movement.rect()?;
        let pointer = self.movement.pointer_position()?;
        let center = ClientPoint::new(
            rect.origin.x + rect.size.width / 2.0,
            rect.origin.y + rect.size.height / 2.0,
        );
        Some(angle_to_value(center, pointer, unit.range(is_12h)))
    }
}

/// Resolves "now" in the device's local timezone for a [`TimePicker`]
/// default value, e.g. `TimePicker { selected_time: selected_time().or_else(|| Some(default_local_time())), ... }`.
/// A thin, discoverable wrapper over [`crate::LocalDateExt::now_local_time`]
/// so a consumer doesn't need to import the trait themselves just for this.
pub fn default_local_time() -> Time {
    OffsetDateTime::now_local_time()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::time;

    #[test]
    fn format_time_matches_expected_shapes() {
        assert_eq!(format_time(time!(14:05:00), false, false), "14:05");
        assert_eq!(format_time(time!(14:05:00), true, false), "2:05 PM");
        assert_eq!(format_time(time!(00:05:09), true, false), "12:05 AM");
        assert_eq!(format_time(time!(14:05:09), false, true), "14:05:09");
        assert_eq!(format_time(time!(14:05:09), true, true), "2:05:09 PM");
    }

    #[test]
    fn to_display_hour_and_back_round_trip() {
        for hour24 in 0..24u8 {
            let (display, is_pm) = to_display_hour(hour24);
            assert!((1..=12).contains(&display), "{hour24} -> {display}");
            assert_eq!(from_display_hour(display, is_pm), hour24, "{hour24}");
        }
    }

    #[test]
    fn to_display_hour_matches_known_values() {
        assert_eq!(to_display_hour(0), (12, false)); // midnight -> 12 AM
        assert_eq!(to_display_hour(12), (12, true)); // noon -> 12 PM
        assert_eq!(to_display_hour(13), (1, true)); // 1 PM
        assert_eq!(to_display_hour(23), (11, true)); // 11 PM
    }

    #[test]
    fn angle_to_value_snaps_to_cardinal_points_of_an_hour_dial() {
        let center = ClientPoint::new(100.0, 100.0);
        // Straight up from center -> 0 (12 o'clock).
        assert_eq!(angle_to_value(center, ClientPoint::new(100.0, 0.0), 12), 0);
        // Straight right -> quarter turn clockwise -> 3 (of 12).
        assert_eq!(
            angle_to_value(center, ClientPoint::new(200.0, 100.0), 12),
            3
        );
        // Straight down -> half turn -> 6.
        assert_eq!(
            angle_to_value(center, ClientPoint::new(100.0, 200.0), 12),
            6
        );
        // Straight left -> three-quarter turn -> 9.
        assert_eq!(angle_to_value(center, ClientPoint::new(0.0, 100.0), 12), 9);
    }

    #[test]
    fn angle_to_value_snaps_to_nearest_minute() {
        let center = ClientPoint::new(0.0, 0.0);
        // A few degrees past straight up should still snap to 0, not 1.
        let angle = 2.0_f64.to_radians();
        let x = angle.sin() * 100.0;
        let y = -angle.cos() * 100.0;
        assert_eq!(angle_to_value(center, ClientPoint::new(x, y), 60), 0);
    }
}
