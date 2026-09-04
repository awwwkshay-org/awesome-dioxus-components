//! Source-owned shadcn-style Input OTP for Dioxus, styling the owned OTP
//! field primitive.
//!
//! Upstream shadcn's `input-otp` wraps the `input-otp` npm package, which
//! renders one hidden real `<input>` plus `length` purely-visual `<div>`
//! "fake caret" slots kept in sync with it. `adico_primitives::otp_field`
//! took a different, simpler approach from the start (task 7.9): each slot
//! *is* its own real, individually focusable `<input maxlength="1">` (see
//! that module's own doc comment for why). This facade styles that real
//! input directly as a bordered box instead of simulating one -- visually
//! equivalent, and a real input's own native caret/selection means there is
//! no `data-[active=true]` fake-caret state to reproduce.

use dioxus::prelude::*;

use adico_primitives::otp_field::{
    OtpFieldInput as OtpFieldPrimitiveInput, OtpFieldRoot as OtpFieldPrimitiveRoot,
    OtpFieldSeparator as OtpFieldPrimitiveSeparator,
};

use crate::adico_lib::cn::cn;
use adico_primitives::icons::Dot;

/// The root of a one-time-passcode input: owns the combined value across a
/// run of [`InputOTPGroup`]/[`InputOTPSlot`]/[`InputOTPSeparator`] children.
/// Unlike `sheet.rs`/`drawer.rs` (which re-export their primitive root
/// directly), `OtpFieldRoot` renders its own container `<div>` with no
/// layout classes of its own -- this wrapper exists specifically to give
/// that container the `flex` layout its children (each already
/// `flex items-center`) need to sit inline instead of stacking.
#[component]
pub fn InputOTP(
    value: ReadSignal<Option<String>>,
    #[props(default)] default_value: String,
    #[props(default)] on_value_change: Callback<String>,
    #[props(default)] on_value_complete: Callback<String>,
    length: ReadSignal<usize>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] read_only: ReadSignal<bool>,
    #[props(default)] name: ReadSignal<String>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "flex items-center gap-2 has-[:disabled]:opacity-50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        OtpFieldPrimitiveRoot {
            value,
            default_value,
            on_value_change,
            on_value_complete,
            length,
            disabled,
            read_only,
            name,
            class,
            {children}
        }
    }
}

/// Groups a run of [`InputOTPSlot`]s (optionally split by
/// [`InputOTPSeparator`] into visually distinct groups, e.g. 3 + 3).
#[component]
pub fn InputOTPGroup(children: Element, class: Option<String>) -> Element {
    let class = cn(&["flex items-center", class.as_deref().unwrap_or_default()]);
    rsx! {
        div { class, {children} }
    }
}

/// A single character slot within an [`InputOTP`]. Styled as a real,
/// individually focusable bordered input box -- see this module's own doc
/// comment for why there is no simulated caret.
#[component]
pub fn InputOTPSlot(index: ReadSignal<usize>, class: Option<String>) -> Element {
    let class = cn(&[
        "relative flex h-9 w-9 items-center justify-center border-y border-r border-input text-center text-sm shadow-xs outline-none transition-all first:rounded-l-md first:border-l last:rounded-r-md focus:z-10 focus:border-ring focus:ring-[3px] focus:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        OtpFieldPrimitiveInput { index, class }
    }
}

/// A visual divider between [`InputOTPGroup`]s, rendered as a dot (matching
/// upstream's own default separator glyph).
#[component]
pub fn InputOTPSeparator() -> Element {
    rsx! {
        OtpFieldPrimitiveSeparator {
            Dot { class: "size-4" }
        }
    }
}
