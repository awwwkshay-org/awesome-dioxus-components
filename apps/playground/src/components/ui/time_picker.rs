//! Styled Time Picker roots composed from adico's owned time/popover
//! primitives. The primitives keep segment editing, focus, and ARIA
//! behavior; the roots supply the semantic layout hook. Mirrors
//! `registry/ui/date_picker.rs`'s composition shape throughout -- see that
//! file for the established conventions this one follows.

use dioxus::html::geometry::ElementPoint;
use dioxus::prelude::*;
use time::Time;

use super::native_select::{NativeSelect, NativeSelectOption, NativeSelectSize};
use super::popover::{PopoverContent, PopoverTrigger};
use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;
use adico_primitives::icons::ChevronDown;
use adico_primitives::popover::{PopoverRoot, PopoverRootProps};
use adico_primitives::scroll_area::scroll_area_visibility_class;
use adico_primitives::time_picker::{
    TimePicker as PrimitiveTimePicker, TimePickerInputValue as PrimitiveTimePickerInputValue,
    format_time, from_display_hour, to_display_hour,
};

pub use adico_primitives::time_picker::{
    TimePickerContext, TimePickerHourSegment, TimePickerMeridiemSegment, TimePickerMinuteSegment,
    TimePickerSecondSegment,
};

/// Which popup body view a [`TimePicker`] shows -- independent of `is_12h`
/// (12h/24h format is a value-formatting concern; this is purely which
/// registry-level part renders). Registry-only: no primitive behavior
/// depends on it, only whether [`TimePickerBody`] renders
/// [`TimePickerColumns`] or [`TimePickerClock`]. `TimePickerColumns` and
/// `TimePickerClock` both remain independently composable directly (as
/// `time-picker`'s own demo page does, showing both at once) for a consumer
/// who wants to hardcode one specific view without a prop switch.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum TimePickerView {
    #[default]
    Digital,
    Analog,
}

/// A registry-only context so [`TimePickerBody`] can read the enclosing
/// [`TimePicker`]'s `view` prop without it being threaded through every
/// intermediate layer by hand.
#[derive(Clone, Copy)]
struct TimePickerViewContext {
    view: ReadSignal<TimePickerView>,
}

/// Props for the styled time picker root.
#[derive(Props, Clone, PartialEq)]
pub struct TimePickerProps {
    #[props(default)]
    pub on_value_change: Callback<Option<Time>>,
    #[props(default)]
    pub selected_time: ReadSignal<Option<Time>>,
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub read_only: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub is_12h: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub show_seconds: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub roving_loop: ReadSignal<bool>,
    /// Which popup body [`TimePickerBody`] renders -- see [`TimePickerView`].
    #[props(default = ReadSignal::new(Signal::new(TimePickerView::Digital)))]
    pub view: ReadSignal<TimePickerView>,
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// A styled time picker retaining the primitive's full interaction API.
#[component]
pub fn TimePicker(props: TimePickerProps) -> Element {
    use_context_provider(|| TimePickerViewContext { view: props.view });
    let class = cn(&[
        "relative inline-block",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveTimePicker {
            on_value_change: props.on_value_change,
            selected_time: props.selected_time,
            disabled: props.disabled,
            read_only: props.read_only,
            is_12h: props.is_12h,
            show_seconds: props.show_seconds,
            roving_loop: props.roving_loop,
            class,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Props, Clone, PartialEq)]
pub struct TimePickerPopoverProps {
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

/// Styled popover root for composing [`TimePickerTrigger`] and
/// [`TimePickerContent`] around a [`TimePicker`]. Takes the same
/// `popover_root` injection prop as `date-picker`'s `DatePickerPopover` and
/// `color-picker`'s `ColorPickerPopover`.
#[component]
pub fn TimePickerPopover(props: TimePickerPopoverProps) -> Element {
    let class = cn(&[
        "group/time-picker",
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
pub struct TimePickerTriggerProps {
    /// Renders a compact icon-only disclosure button (the same shape
    /// `date-picker`'s `DatePickerTrigger` always uses) instead of the
    /// default labeled button that spells out the formatted time.
    ///
    /// Set this whenever the trigger sits *inside* a [`TimePickerInput`]
    /// alongside [`TimePickerInputValue`]: that field already displays the
    /// value, so the default labeled trigger renders the same time a second
    /// time, inside its own nested bordered box (e.g. `05 : 00` followed by
    /// `5:00 AM ⌄`). The labeled default remains correct for a standalone
    /// trigger with no separate input, which is how `date-time-picker`'s own
    /// `DateTimePickerTrigger` presents itself.
    #[props(default)]
    pub compact: bool,
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

/// Disclosure trigger composed from the installed Popover façade. Defaults
/// to showing the currently selected time (via [`TimePickerContext`]) --
/// falling back to "Pick a time" -- so activating this trigger both opens
/// the picker and shows the time it currently holds. Pass `compact: true`
/// when composing it inside a [`TimePickerInput`], which already shows the
/// value (see [`TimePickerTriggerProps::compact`]). Must be used inside a
/// [`TimePicker`] (for the context) and a [`TimePickerPopover`].
#[component]
pub fn TimePickerTrigger(props: TimePickerTriggerProps) -> Element {
    let ctx = use_context::<TimePickerContext>();
    let label = use_memo(move || match ctx.selected_time() {
        Some(time) => format_time(time, ctx.is_12h(), ctx.show_seconds()),
        None => "Pick a time".to_string(),
    });
    // Swaps the whole base class rather than layering overrides on top of
    // it: `cn` concatenates without resolving Tailwind conflicts, so a
    // caller-supplied `border-0`/`px-0` would not reliably beat the base
    // `border`/`px-3` (which utility wins is decided by stylesheet order).
    let class = cn(&[
        if props.compact {
            "ml-1 size-6 rounded-sm text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        } else {
            "inline-flex h-9 items-center gap-2 rounded-md border border-input bg-background px-3 text-sm shadow-xs transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
        },
        props.class.as_deref().unwrap_or_default(),
    ]);
    let children = props.children.unwrap_or_else(|| {
        if props.compact {
            rsx! {
                ChevronDown {
                    class: "size-4 shrink-0 transition-transform duration-200 group-data-[state=open]/time-picker:rotate-180",
                    size: 16,
                }
                span { class: "sr-only", "Toggle time picker" }
            }
        } else {
            rsx! {
                "{label()}"
                ChevronDown {
                    class: "size-4 shrink-0 text-muted-foreground transition-transform duration-200 group-data-[state=open]/time-picker:rotate-180",
                    size: 16,
                }
            }
        }
    });
    rsx! {
        PopoverTrigger { class, attributes: props.attributes, {children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TimePickerContentProps {
    #[props(default)]
    pub class: Option<String>,
    pub children: Element,
}

/// Popup shell with no duplicate frame -- the columns/clock view owns the
/// surface, matching `DatePickerContent`'s equivalent role for the calendar.
#[component]
pub fn TimePickerContent(props: TimePickerContentProps) -> Element {
    let class = cn(&[
        "w-auto border-0 bg-transparent p-0 shadow-none",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! { PopoverContent { class, {props.children} } }
}

#[derive(Props, Clone, PartialEq)]
pub struct TimePickerInputValueProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub children: Option<Element>,
}

/// Editable `HH : MM (: SS) (AM/PM)` segments; each segment keeps the
/// primitive's spinbutton keyboard semantics. Styling matches
/// `DatePickerInputValue`'s CSS-descendant-selector approach exactly --
/// `[data-empty=true]` is `segment::NumericSegment`/`MeridiemSegment`'s
/// generically-named empty-value hook (the same attribute `no-date` names
/// for backward compatibility with `date-picker`'s own styling).
#[component]
pub fn TimePickerInputValue(props: TimePickerInputValueProps) -> Element {
    let class = cn(&[
        "flex items-center gap-1 text-sm tabular-nums text-foreground [&_[role=spinbutton]]:rounded-sm [&_[role=spinbutton]]:px-0.5 [&_[role=spinbutton]]:outline-none [&_[role=spinbutton]:focus]:bg-accent [&_[role=spinbutton][data-empty=true]]:text-muted-foreground [&_[is-separator=true]]:text-muted-foreground",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        span { class,
            PrimitiveTimePickerInputValue { children: props.children }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TimePickerInputProps {
    #[props(default = Radius::Md)]
    pub radius: Radius,
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

/// A compact editable time field matching `DatePickerInput`'s composition.
#[component]
pub fn TimePickerInput(props: TimePickerInputProps) -> Element {
    let class = cn(&[
        "inline-flex h-9 items-center gap-1 border border-input bg-background px-2 text-sm shadow-xs transition-colors focus-within:border-ring focus-within:ring-2 focus-within:ring-ring/50 data-[disabled=true]:pointer-events-none data-[disabled=true]:opacity-50",
        props.radius.class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    let children = props
        .children
        .unwrap_or_else(|| rsx! { TimePickerInputValue {} });

    rsx! {
        div { class, ..props.attributes,
            {children}
        }
    }
}

fn current_or_midnight(ctx: TimePickerContext) -> Time {
    ctx.selected_time()
        .unwrap_or_else(|| Time::from_hms(0, 0, 0).expect("0:00:00 is always valid"))
}

/// Sets the AM/PM half of the current time, keeping the displayed hour and
/// the minute/second unchanged -- shared by [`TimePickerColumns`]'s and
/// [`TimePickerClock`]'s own AM/PM controls.
fn apply_meridiem(mut ctx: TimePickerContext, pm: bool) {
    let current = current_or_midnight(ctx);
    let (display, _) = to_display_hour(current.hour());
    let hour24 = from_display_hour(display, pm);
    if let Ok(next) = Time::from_hms(hour24, current.minute(), current.second()) {
        ctx.set_selected_time(Some(next));
    }
}

/// Shared selected/unselected styling for a `TimePickerColumns` list item and
/// a `TimePickerClock` AM/PM button.
fn time_option_class(selected: bool) -> String {
    cn(&[
        "shrink-0 rounded-sm px-2 py-1 text-center text-sm tabular-nums transition-colors hover:bg-accent",
        if selected {
            "bg-primary text-primary-foreground hover:bg-primary/90"
        } else {
            "text-foreground"
        },
    ])
}

#[derive(Props, Clone, PartialEq)]
pub struct TimePickerColumnsProps {
    #[props(default)]
    pub class: Option<String>,
    /// Stretches each column to the height of its flex container (e.g. a
    /// taller sibling like `DatePickerCalendar` in `date-time-picker`)
    /// instead of the default fixed `max-h-48`. Off by default because it
    /// relies on the immediate parent being a flex container that already
    /// has a definite cross-size to stretch into (a taller sibling, or an
    /// explicit height) -- turning it on with no such context collapses the
    /// columns to zero height. The caller is still responsible for growing
    /// this component's own box into that available space (e.g. `class:
    /// "flex-1 min-h-0"`); this prop only changes what the columns do with
    /// whatever height they're given.
    #[props(default)]
    pub fill_height: bool,
}

/// A scrollable hour/minute(/second)(/meridiem) column view -- the
/// accessible, keyboard-operable alternative to [`TimePickerClock`] (native
/// buttons, reachable and activatable the same way any other button is).
/// Must be used inside a [`TimePicker`].
#[component]
pub fn TimePickerColumns(props: TimePickerColumnsProps) -> Element {
    let mut ctx = use_context::<TimePickerContext>();
    let is_12h = ctx.is_12h();
    let show_seconds = ctx.show_seconds();
    let current = current_or_midnight(ctx);
    let (hour_display, is_pm) = to_display_hour(current.hour());

    let mut set_hour = move |display: u8| {
        let current = current_or_midnight(ctx);
        let hour24 = if is_12h {
            from_display_hour(display, is_pm)
        } else {
            display
        };
        if let Ok(next) = Time::from_hms(hour24, current.minute(), current.second()) {
            ctx.set_selected_time(Some(next));
        }
    };
    let mut set_minute = move |minute: u8| {
        let current = current_or_midnight(ctx);
        if let Ok(next) = Time::from_hms(current.hour(), minute, current.second()) {
            ctx.set_selected_time(Some(next));
        }
    };
    let mut set_second = move |second: u8| {
        let current = current_or_midnight(ctx);
        if let Ok(next) = Time::from_hms(current.hour(), current.minute(), second) {
            ctx.set_selected_time(Some(next));
        }
    };
    let hour_range: Vec<u8> = if is_12h {
        (1..=12).collect()
    } else {
        (0..24).collect()
    };

    let column_class = cn(&[
        "flex w-14 flex-col gap-0.5 overflow-y-auto p-1 [scrollbar-width:thin]",
        // `scrollbar-color` (this class) and `scrollbar-width: thin` (the Tailwind
        // arbitrary-value class above) are independent CSS properties and compose
        // cleanly -- the column keeps its intentionally slim native scrollbar, now
        // themed to match every other scroll surface.
        scroll_area_visibility_class(false),
        if props.fill_height {
            "h-full"
        } else {
            "max-h-48"
        },
    ]);
    let class = cn(&[
        // Owns its own popup surface, matching `calendar`'s `CalendarView`
        // (`border bg-popover p-3 text-popover-foreground shadow-sm`): the
        // enclosing `TimePickerContent` deliberately strips the popover's
        // frame (same as `DatePickerContent`), so the view part is what
        // supplies it. Without this the columns render bare on the page
        // background with no card, border, or padding, and their own scroll
        // clipping reads as a rendering glitch rather than a scrollable list.
        "flex gap-1 rounded-md border bg-popover p-2 text-popover-foreground shadow-sm",
        if props.fill_height {
            "items-stretch"
        } else {
            "items-start"
        },
        props.class.as_deref().unwrap_or_default(),
    ]);

    rsx! {
        div { class,
            div { class: column_class.clone(),
                for hour in hour_range {
                    button {
                        r#type: "button",
                        class: time_option_class(hour == hour_display),
                        onclick: move |_| set_hour(hour),
                        "{hour:02}"
                    }
                }
            }
            div { class: column_class.clone(),
                for minute in 0u8..60 {
                    button {
                        r#type: "button",
                        class: time_option_class(minute == current.minute()),
                        onclick: move |_| set_minute(minute),
                        "{minute:02}"
                    }
                }
            }
            if show_seconds {
                div { class: column_class.clone(),
                    for second in 0u8..60 {
                        button {
                            r#type: "button",
                            class: time_option_class(second == current.second()),
                            onclick: move |_| set_second(second),
                            "{second:02}"
                        }
                    }
                }
            }
            if is_12h {
                div { class: "flex flex-col gap-0.5 p-1",
                    button {
                        r#type: "button",
                        class: time_option_class(!is_pm),
                        onclick: move |_| apply_meridiem(ctx, false),
                        "AM"
                    }
                    button {
                        r#type: "button",
                        class: time_option_class(is_pm),
                        onclick: move |_| apply_meridiem(ctx, true),
                        "PM"
                    }
                }
            }
        }
    }
}

/// Which unit [`TimePickerClock`]'s face currently sets. A small format
/// select above the dial switches between them.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ClockUnit {
    Hour,
    Minute,
}

impl ClockUnit {
    const ALL: [ClockUnit; 2] = [Self::Hour, Self::Minute];

    fn label(self) -> &'static str {
        match self {
            Self::Hour => "Hour",
            Self::Minute => "Minute",
        }
    }

    fn range(self, is_12h: bool) -> u32 {
        match self {
            Self::Hour if is_12h => 12,
            Self::Hour => 24,
            Self::Minute => 60,
        }
    }
}

/// Computes `(left%, top%)` for a value's label around a circular dial face,
/// `0` at the top, increasing clockwise -- the inverse layout of
/// `adico_primitives::time_picker::angle_to_value`'s own convention, so a
/// label drawn here lines up with where a click at that position resolves.
fn dial_label_position(value: u32, range: u32) -> (f64, f64) {
    let angle = (value as f64 / range as f64) * std::f64::consts::TAU;
    let radius = 40.0;
    (50.0 + radius * angle.sin(), 50.0 - radius * angle.cos())
}

#[derive(Props, Clone, PartialEq)]
pub struct TimePickerClockProps {
    #[props(default)]
    pub class: Option<String>,
}

/// An analog clock-face view -- pointer-only enhancement over the same
/// value [`TimePickerColumns`]/the segmented input already provide; there is
/// no WAI-ARIA pattern for an analog dial, so this is deliberately not the
/// only way to set a value (see this item's own documentation and
/// `add-time-picker-components`'s design.md).
///
/// Renders a clock hand from the centre to the selected value and supports
/// dragging it around the face, as well as click/tap-to-set. Both read the
/// pointer event's *own* coordinates against the dial's measured rect and
/// listen on the dial element itself (`onpointerdown`/`onpointermove`/
/// `onpointerup`/`onpointerleave`) -- deliberately *not*
/// `adico-primitives`'s global document-level pointer-position registry,
/// which its own source documents as unverified in a live browser. Dragging
/// therefore needs no new primitive plumbing.
///
/// When `is_12h` is set, an AM/PM button pair always renders next to the
/// Hour/Minute select -- the dial face itself has no AM/PM affordance
/// (dragging it only ever sets the 1-12 position), so without this the clock
/// would offer no way to choose which half of the day is meant. Must be used
/// inside a [`TimePicker`].
#[component]
pub fn TimePickerClock(props: TimePickerClockProps) -> Element {
    let mut ctx = use_context::<TimePickerContext>();
    let mut unit = use_signal(|| ClockUnit::Hour);
    let mut face_size = use_signal(|| None::<(f64, f64)>);
    let mut is_dragging = use_signal(|| false);
    // The raw (unsnapped) pointer angle while a drag is in flight; `None`
    // when idle, so the hand rests on the committed value's exact angle.
    let mut drag_degrees = use_signal(|| None::<f64>);

    let is_12h = ctx.is_12h();
    let current = current_or_midnight(ctx);
    let (_, is_pm) = to_display_hour(current.hour());
    let range = unit().range(is_12h);
    // Read fresh rather than captured, so the drag handler can compare
    // against the value as it stands *now* instead of as it stood when this
    // render's closures were built.
    let current_dial_value = move || {
        let current = current_or_midnight(ctx);
        match unit() {
            ClockUnit::Hour if is_12h => (to_display_hour(current.hour()).0 % 12) as u32,
            ClockUnit::Hour => current.hour() as u32,
            ClockUnit::Minute => current.minute() as u32,
        }
    };
    let current_value = current_dial_value();

    let mut apply_value = move |value: u32| {
        let current = current_or_midnight(ctx);
        let next = match unit() {
            ClockUnit::Hour => {
                let hour24 = if is_12h {
                    let display = if value == 0 { 12 } else { value as u8 };
                    from_display_hour(display, is_pm)
                } else {
                    value as u8
                };
                Time::from_hms(hour24, current.minute(), current.second())
            }
            ClockUnit::Minute => Time::from_hms(current.hour(), value as u8, current.second()),
        };
        if let Ok(next) = next {
            ctx.set_selected_time(Some(next));
        }
    };

    // Shared by tap and drag: resolve a pointer position to the nearest
    // value on the face. `atan2(dx, -dy)` puts 0 at the top and grows
    // clockwise, matching `dial_label_position`'s layout convention.
    //
    // Works entirely in the dial's *own* coordinate space
    // (`element_coordinates`, i.e. offsetX/offsetY) rather than against a
    // viewport rect captured at mount: `Positioner` places the popup *after*
    // its content mounts, so a rect measured in `onmounted` describes where
    // the dial sat before it was positioned. Confirmed live -- that stale
    // centre was ~169px above the real one, and a drag released at 3
    // o'clock resolved to 5. Element coordinates are relative to the element
    // itself, so repositioning, scrolling, and the popover's zoom-in
    // entrance transform cannot desynchronise them.
    let mut apply_from_pointer = move |point: ElementPoint| {
        if let Some((width, height)) = face_size() {
            let dx = point.x - width / 2.0;
            let dy = point.y - height / 2.0;
            let degrees = dx.atan2(-dy).to_degrees();
            let normalized = if degrees < 0.0 {
                degrees + 360.0
            } else {
                degrees
            };
            // The hand follows the raw pointer angle so the sweep is
            // continuous; only the committed value snaps. Rendering the hand
            // from the snapped value instead makes it lurch a whole step at
            // a time (30 degrees per hour on a 12-hour face), which is what
            // "not smooth" looks like.
            drag_degrees.set(Some(normalized));
            let step = 360.0 / range as f64;
            let next = (normalized / step).round() as u32 % range;
            // A pointermove fires many times within one step, and every
            // committed value re-renders the whole picker. Writing only on an
            // actual change keeps the drag cheap.
            if next != current_dial_value() {
                apply_value(next);
            }
        }
    };

    // While dragging, follow the pointer exactly; otherwise rest on the
    // committed value. The easing is applied only when *not* dragging, so
    // the hand settles onto its snapped angle on release without lagging
    // behind the pointer mid-drag.
    let snapped_degrees = (current_value as f64 / range as f64) * 360.0;
    let hand_degrees = 180.0 + drag_degrees().unwrap_or(snapped_degrees);
    let hand_class = cn(&[
        "pointer-events-none absolute bg-primary",
        if is_dragging() {
            ""
        } else {
            "transition-transform duration-150 ease-out"
        },
    ]);

    let labels: Vec<u32> = match unit() {
        ClockUnit::Hour if is_12h => (0..12).collect(),
        ClockUnit::Hour => (0..24).step_by(2).collect(),
        ClockUnit::Minute => (0..60).step_by(5).collect(),
    };
    // The 12-hour hour dial's internal value domain is 0..12 (matching
    // `to_display_hour`/`from_display_hour`'s own `0 == 12 o'clock` mapping
    // used by `current_value`/`apply_value` above), but a real clock face
    // shows "12" at that position, never "0".
    let is_hour_12h = matches!(unit(), ClockUnit::Hour) && is_12h;
    let display_label = move |value: u32| {
        if is_hour_12h && value == 0 {
            "12".to_string()
        } else {
            value.to_string()
        }
    };

    let class = cn(&[
        // Owns its own popup surface for the same reason `TimePickerColumns`
        // does -- see that component's own note.
        "flex flex-col items-center gap-3 rounded-md border bg-popover p-3 text-popover-foreground shadow-sm",
        props.class.as_deref().unwrap_or_default(),
    ]);

    rsx! {
        div { class,
            div { class: "flex items-center gap-2",
                NativeSelect {
                    size: NativeSelectSize::Sm,
                    class: "w-24",
                    value: Some((unit() as u8).to_string()),
                    oninput: move |event: FormEvent| {
                        if let Ok(index) = event.value().parse::<usize>()
                            && let Some(next) = ClockUnit::ALL.get(index)
                        {
                            unit.set(*next);
                        }
                    },
                    for (index , option) in ClockUnit::ALL.iter().enumerate() {
                        NativeSelectOption { value: "{index}", "{option.label()}" }
                    }
                }
                if is_12h {
                    div { class: "flex gap-1",
                        button {
                            r#type: "button",
                            class: time_option_class(!is_pm),
                            onclick: move |_| apply_meridiem(ctx, false),
                            "AM"
                        }
                        button {
                            r#type: "button",
                            class: time_option_class(is_pm),
                            onclick: move |_| apply_meridiem(ctx, true),
                            "PM"
                        }
                    }
                }
            }
            div {
                // `select-none` stops a drag across the face from
                // highlighting the hour labels as text, and `touch-none`
                // stops a touch drag from scrolling the page instead of
                // turning the hand -- both are required for the drag to feel
                // like a dial rather than a text selection.
                class: "relative size-56 shrink-0 cursor-pointer touch-none select-none rounded-full border border-input bg-muted/30",
                role: "presentation",
                "aria-hidden": "true",
                // Only the dial's own size is measured, never its position --
                // size is what `apply_from_pointer` needs to find the centre
                // in element space, and unlike position it does not change
                // when `Positioner` places the popup. `get_scroll_size`
                // (scrollWidth/Height) is used rather than
                // `get_client_rect`, since the latter reports the popover's
                // zoom-in entrance transform mid-animation and would report
                // a 95%-scaled size.
                onmounted: move |event| async move {
                    if let Ok(size) = event.data().get_scroll_size().await {
                        face_size.set(Some((size.width, size.height)));
                    }
                },
                onpointerdown: move |event: Event<PointerData>| {
                    is_dragging.set(true);
                    apply_from_pointer(event.element_coordinates());
                },
                onpointermove: move |event: Event<PointerData>| {
                    if is_dragging() {
                        apply_from_pointer(event.element_coordinates());
                    }
                },
                // Clearing `drag_degrees` hands the angle back to the
                // committed value, so the hand eases onto its snapped
                // position instead of resting between two values.
                onpointerup: move |_| {
                    is_dragging.set(false);
                    drag_degrees.set(None);
                },
                // Without document-level tracking, a pointer released outside
                // the dial never reports its `pointerup` here -- ending the
                // drag on leave keeps the hand from continuing to follow an
                // already-released pointer when it re-enters the face.
                onpointerleave: move |_| {
                    is_dragging.set(false);
                    drag_degrees.set(None);
                },
                // The hand: a thin bar pivoting at the dial centre, pointing
                // at the selected value. `transform-origin: 50% 0` puts the
                // pivot at the bar's top edge and an unrotated bar hangs
                // straight down, so the base 180deg turns it to 12 o'clock
                // before `hand_degrees` sweeps it clockwise -- the same
                // convention `dial_label_position` lays the numbers out on,
                // so the hand always lands on its label.
                span {
                    class: hand_class,
                    style: "left: calc(50% - 1px); top: 50%; width: 2px; height: 40%; transform-origin: 50% 0; transform: rotate({hand_degrees}deg);",
                }
                for value in labels {
                    {
                        let (left, top) = dial_label_position(value, range);
                        let selected = value == current_value;
                        rsx! {
                            span {
                                class: cn(
                                    &[
                                        "pointer-events-none absolute flex size-7 -translate-x-1/2 -translate-y-1/2 items-center justify-center rounded-full text-xs tabular-nums",
                                        if selected {
                                            "bg-primary text-primary-foreground"
                                        } else {
                                            "text-foreground"
                                        },
                                    ],
                                ),
                                style: "left: {left}%; top: {top}%;",
                                "{display_label(value)}"
                            }
                        }
                    }
                }
                span { class: "absolute left-1/2 top-1/2 size-1.5 -translate-x-1/2 -translate-y-1/2 rounded-full bg-primary" }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TimePickerBodyProps {
    #[props(default)]
    pub class: Option<String>,
    /// Forwarded to [`TimePickerColumns`]'s own `fill_height` when the
    /// current view is `Digital`; has no effect for `Analog` (`TimePickerClock`
    /// is a fixed-size dial with no comparable height-matching concept).
    #[props(default)]
    pub fill_height: bool,
}

/// Renders [`TimePickerColumns`] or [`TimePickerClock`] depending on the
/// enclosing [`TimePicker`]'s `view` prop -- the prop-driven single-view
/// alternative to composing `TimePickerColumns`/`TimePickerClock` directly.
/// Must be used inside a [`TimePicker`] (for both [`TimePickerContext`] and
/// the view context [`TimePicker`]'s `view` prop provides).
#[component]
pub fn TimePickerBody(props: TimePickerBodyProps) -> Element {
    let ctx = use_context::<TimePickerViewContext>();
    match (ctx.view)() {
        TimePickerView::Digital => rsx! {
            TimePickerColumns { class: props.class.clone(), fill_height: props.fill_height }
        },
        TimePickerView::Analog => rsx! {
            TimePickerClock { class: props.class.clone() }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dial_label_position_places_zero_at_the_top_center() {
        let (left, top) = dial_label_position(0, 12);
        assert!((left - 50.0).abs() < 0.001, "{left}");
        assert!(top < 50.0, "expected top-of-circle, got {top}");
    }
}
