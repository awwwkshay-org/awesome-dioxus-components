// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Implements the WAI-ARIA APG's typeahead pattern: keystrokes typed in rapid succession
// accumulate in a buffer, the buffer auto-clears after a pause, and focus moves to the
// best-matching available item. Neither `statics/catalogs/base-ui.json`'s Autocomplete entry
// nor `statics/primitive_compatibility.json` tracks a dedicated typeahead capability on
// either reference axis, so the matching layer below — a weighted edit distance favoring
// prefix/recency, adaptive keyboard-layout learning from observed key events, and a small
// cross-script phonetic fallback (Latin/Cyrillic/Greek only, by deliberate scope choice) —
// is an adico-only capability, not a ported one. It exists so typeahead keeps working
// through typos and non-QWERTY layouts, one input away from the APG's plain prefix baseline.

//! Typeahead search: matching algorithms plus a buffered, auto-clearing
//! [`Typeahead`] handle any list/menu-shaped widget can use to let users type
//! to focus a matching item, adapting to the user's keyboard layout.

use crate::selectable::OptionState;
use core::f32;
use dioxus::prelude::*;
use dioxus_core::Task;
use std::collections::HashMap;
use std::time::Duration;

/// Find the best matching option based on typeahead input
pub fn best_match(
    keyboard: &AdaptiveKeyboard,
    typeahead: &str,
    options: &[OptionState],
    is_available: impl Fn(usize) -> bool,
) -> Option<usize> {
    if typeahead.is_empty() {
        return None;
    }

    let typeahead_characters: Box<[_]> = typeahead.chars().collect();

    options
        .iter()
        .filter(|opt| is_available(opt.index))
        .map(|opt| {
            let value = &opt.text_value;
            let value_characters: Box<[_]> = value.chars().collect();
            let distance = normalized_distance(&typeahead_characters, &value_characters, keyboard);
            (distance, opt.index)
        })
        .min_by(|(d1, _), (d2, _)| f32::total_cmp(d1, d2))
        .map(|(_, value)| value)
}

/// Calculate normalized distance between typeahead and value characters
pub fn normalized_distance(
    typeahead_characters: &[char],
    value_characters: &[char],
    keyboard: &AdaptiveKeyboard,
) -> f32 {
    // Only compare against as much of the value as the typeahead could plausibly cover.
    let value_characters =
        &value_characters[..value_characters.len().min(typeahead_characters.len())];
    // Only the most recently typed characters matter once the buffer outgrows the value.
    let typeahead_characters = &typeahead_characters[typeahead_characters
        .len()
        .saturating_sub(value_characters.len())..];

    weighted_edit_distance(typeahead_characters, value_characters, |a, b| {
        keyboard.substitution_cost(a, b)
    })
}

/// Weight in `[0.02, 1.0]` for the `index`-th (1-indexed) position out of `length`: later
/// positions weigh more heavily. Used both for how much a typed character should count (more
/// recently typed matters more) and, inverted, for how forgiving an unmatched trailing value
/// character is (an option's tail mattering less than its head, since typeahead is
/// prefix-oriented).
pub fn position_weight(index: usize, length: usize) -> f32 {
    if length == 0 {
        return 0.0;
    }
    let ratio = (index.min(length) as f32) / (length as f32);
    ratio.powi(3).max(0.02)
}

/// A weighted edit distance between a typeahead buffer and a candidate value: unlike a plain
/// Levenshtein distance, character weight grows with position, so mismatches near the end of
/// the typed buffer (the most recently typed keystrokes) and near the start of the value (its
/// prefix) cost more than mismatches elsewhere — modeling "the user is typing a prefix, and
/// their latest keystroke is their most deliberate one." `substitution_cost` supplies the
/// per-character-pair cost when two characters don't match exactly (see
/// [`AdaptiveKeyboard::substitution_cost`]). `pub` for direct testing from
/// `packages/adico-primitives/tests/`; [`normalized_distance`] is the intended entry point.
pub fn weighted_edit_distance(
    typeahead: &[char],
    value: &[char],
    substitution_cost: impl Fn(char, char) -> f32,
) -> f32 {
    let (rows, cols) = (typeahead.len(), value.len());
    let mut cost = vec![vec![0.0_f32; cols + 1]; rows + 1];

    for j in 1..=cols {
        // Skipping a trailing (unmatched) value character is cheap; skipping one near the
        // start of the value is expensive, since a real prefix match should cover the start.
        cost[0][j] = cost[0][j - 1] + 0.5 * (1.0 - position_weight(j, cols));
    }
    for i in 1..=rows {
        // Dropping an early typed character is cheap; dropping a recently typed one is not.
        cost[i][0] = cost[i - 1][0] + 0.5 * position_weight(i, rows);
    }

    for i in 1..=rows {
        for j in 1..=cols {
            let mismatch_cost = if typeahead[i - 1] == value[j - 1] {
                0.0
            } else {
                substitution_cost(typeahead[i - 1], value[j - 1])
            };
            let drop_typed_char = cost[i - 1][j] + position_weight(i, rows);
            let skip_value_char = cost[i][j - 1] + (1.0 - position_weight(j, cols));
            let substitute = cost[i - 1][j - 1] + mismatch_cost * 2.0 * position_weight(i, rows);
            cost[i][j] = drop_typed_char.min(skip_value_char).min(substitute);
        }
    }

    let raw = cost[rows][cols];
    let worst_case = cost[rows][0].max(cost[0][cols]);
    if worst_case <= 0.0 {
        0.0
    } else {
        raw / worst_case
    }
}

/// Adaptive keyboard learning system for multi-language support
#[derive(Debug, Clone)]
pub struct AdaptiveKeyboard {
    /// Physical key position mappings learned from events
    physical_mappings: HashMap<String, char>,
    /// Our current best guess of the keyboard layout based on learned mappings
    layout: KeyboardLayout,
}

impl Default for AdaptiveKeyboard {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveKeyboard {
    /// Create a new adaptive keyboard system
    pub fn new() -> Self {
        Self {
            physical_mappings: HashMap::new(),
            layout: KeyboardLayout::Qwerty,
        }
    }

    /// Learn from a keyboard event mapping physical key to logical character
    pub fn learn_from_event(&mut self, physical_code: &str, logical_char: char) {
        self.physical_mappings
            .insert(physical_code.to_string(), logical_char);
        self.layout = KeyboardLayout::guess(&self.physical_mappings);
    }

    /// Cost of substituting `a` for `b`, taking the cheapest of three independent signals:
    /// physical keyboard-key proximity under the learned layout, Unicode codepoint proximity,
    /// and cross-script phonetic similarity. Each signal alone is a weak proxy for "these
    /// characters are easy to confuse"; taking their minimum lets any one of them justify a
    /// low cost without the other two false-negatively dragging it up.
    pub fn substitution_cost(&self, a: char, b: char) -> f32 {
        if a == b {
            return 0.0;
        }

        let (a_lower, b_lower) = (
            a.to_lowercase().next().unwrap_or(a),
            b.to_lowercase().next().unwrap_or(b),
        );
        if a_lower == b_lower {
            // Case-only difference: cheap, but not literally free — a typeahead buffer is
            // still not a case-insensitive text field, just forgiving of casing mistakes.
            return 0.05;
        }

        let physical_cost = self
            .layout
            .distance_cost(a_lower, b_lower)
            .map(|cost| cost * 0.35)
            .unwrap_or(f32::INFINITY);
        let codepoint_cost = Self::codepoint_proximity_cost(a, b);
        let phoneme_cost = Self::phoneme_proximity_cost(a_lower, b_lower);

        physical_cost.min(codepoint_cost).min(phoneme_cost)
    }

    /// Characters with nearby Unicode codepoints are often visually or historically related
    /// (accented variants of a base letter, for example); scale that proximity into a cost.
    fn codepoint_proximity_cost(a: char, b: char) -> f32 {
        let delta = (a as i64 - b as i64).unsigned_abs() as f32;
        (delta / 128.0).clamp(0.15, 1.0)
    }

    /// Cost from sharing a phoneme class across scripts (see [`phoneme_class`]) — deliberately
    /// scoped to Latin, Cyrillic, and Greek, the three scripts this crate's authors can
    /// verify the letter correspondences for directly, rather than risk transcribing
    /// codepoints for scripts they'd only be copying from a reference.
    fn phoneme_proximity_cost(a: char, b: char) -> f32 {
        match (phoneme_class(a), phoneme_class(b)) {
            (Some(class_a), Some(class_b)) if class_a == class_b => 0.25,
            _ => 1.0,
        }
    }
}

/// A coarse phoneme class shared by roughly-equivalent letters across Latin, Cyrillic, and
/// Greek — enough to let, say, a Cyrillic "б" and a Latin "b" cost less to substitute for one
/// another than two unrelated characters, without claiming linguistic precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PhonemeClass {
    A,
    B,
    D,
    E,
    F,
    G,
    H,
    I,
    K,
    L,
    M,
    N,
    O,
    P,
    R,
    S,
    T,
    U,
    V,
    Y,
    Z,
}

fn phoneme_class(c: char) -> Option<PhonemeClass> {
    use PhonemeClass::*;
    match c {
        // Latin
        'a' => Some(A),
        'b' => Some(B),
        'd' => Some(D),
        'e' => Some(E),
        'f' => Some(F),
        'g' => Some(G),
        'h' => Some(H),
        'i' => Some(I),
        'k' => Some(K),
        'l' => Some(L),
        'm' => Some(M),
        'n' => Some(N),
        'o' => Some(O),
        'p' => Some(P),
        'r' => Some(R),
        's' => Some(S),
        't' => Some(T),
        'u' => Some(U),
        'v' => Some(V),
        'y' => Some(Y),
        'z' => Some(Z),
        // Cyrillic
        'а' => Some(A),
        'б' => Some(B),
        'в' => Some(V),
        'г' => Some(G),
        'д' => Some(D),
        'е' | 'э' => Some(E),
        'з' => Some(Z),
        'и' => Some(I),
        'й' => Some(Y),
        'к' => Some(K),
        'л' => Some(L),
        'м' => Some(M),
        'н' => Some(N),
        'о' => Some(O),
        'п' => Some(P),
        'р' => Some(R),
        'с' => Some(S),
        'т' => Some(T),
        'у' => Some(U),
        'ф' => Some(F),
        'х' => Some(H),
        // Greek
        'α' => Some(A),
        'β' => Some(V),
        'γ' => Some(G),
        'δ' => Some(D),
        'ε' => Some(E),
        'ζ' => Some(Z),
        'ι' => Some(I),
        'κ' => Some(K),
        'λ' => Some(L),
        'μ' => Some(M),
        'ν' => Some(N),
        'ο' | 'ω' => Some(O),
        'π' => Some(P),
        'ρ' => Some(R),
        'σ' => Some(S),
        'τ' => Some(T),
        'υ' => Some(U),
        'φ' => Some(F),
        _ => None,
    }
}

/// Supported keyboard layouts for optimized text matching
#[derive(Debug, Clone, Copy, Default, PartialEq)]

pub enum KeyboardLayout {
    Qwerty,
    ColemakDH,
    Colemak,
    Dvorak,
    Workman,
    Azerty,
    Qwertz,
    #[default]
    Unknown,
}

impl KeyboardLayout {
    const KNOWN_KEYBOARD_LAYOUTS: &'static [KeyboardLayout] = &[
        KeyboardLayout::Qwerty,
        KeyboardLayout::ColemakDH,
        KeyboardLayout::Colemak,
        KeyboardLayout::Dvorak,
        KeyboardLayout::Workman,
        KeyboardLayout::Azerty,
        KeyboardLayout::Qwertz,
    ];

    /// Guess the keyboard layout based on observed key positions
    pub fn guess(known_key_positions: &HashMap<String, char>) -> KeyboardLayout {
        Self::KNOWN_KEYBOARD_LAYOUTS
            .iter()
            .copied()
            .find(|layout| {
                known_key_positions.iter().all(|(from, to)| {
                    let Some(from_char) = code_to_char(from) else {
                        return false;
                    };
                    match (
                        Self::Qwerty.char_position(from_char),
                        layout.char_position(*to),
                    ) {
                        (Some(from_pos), Some(to_pos)) => from_pos == to_pos,
                        _ => false,
                    }
                })
            })
            .unwrap_or_default()
    }

    /// Euclidean distance between two keys on this layout, scaled to a `[0.05, 1.0]`
    /// substitution cost — physically adjacent keys (a common source of typos) cost little,
    /// keys on opposite sides of the board cost close to the maximum.
    pub fn distance_cost(&self, a: char, b: char) -> Option<f32> {
        let (a_pos, b_pos) = match (self.char_position(a), self.char_position(b)) {
            (Some(a_pos), Some(b_pos)) => (a_pos, b_pos),
            _ => return None,
        };

        let dx = (a_pos.0 as f32 - b_pos.0 as f32).abs();
        let dy = (a_pos.1 as f32 - b_pos.1 as f32).abs();
        let distance = (dx * dx + dy * dy).sqrt();

        Some((distance / 8.0).clamp(0.05, 1.0))
    }

    /// Get the position of a character on the keyboard layout
    fn char_position(&self, c: char) -> Option<(usize, usize)> {
        let layout = match self {
            KeyboardLayout::Qwerty => &QWERTY_KEYBOARD_LAYOUT,
            KeyboardLayout::ColemakDH => &COLEMACK_DH_KEYBOARD_LAYOUT,
            KeyboardLayout::Colemak => &COLEMAK_KEYBOARD_LAYOUT,
            KeyboardLayout::Dvorak => &DVORAK_KEYBOARD_LAYOUT,
            KeyboardLayout::Workman => &WORKMAN_KEYBOARD_LAYOUT,
            KeyboardLayout::Azerty => &AZERTY_KEYBOARD_LAYOUT,
            KeyboardLayout::Qwertz => &QWERTZ_KEYBOARD_LAYOUT,
            KeyboardLayout::Unknown => return None,
        };

        for (row_idx, row) in layout.iter().enumerate() {
            for (col_idx, &ch) in row.iter().enumerate() {
                if ch == c {
                    return Some((col_idx, row_idx));
                }
            }
        }
        None
    }
}

/// Convert a key code to a character
pub fn code_to_char(code: &str) -> Option<char> {
    match code {
        "KeyA" => Some('a'),
        "KeyB" => Some('b'),
        "KeyC" => Some('c'),
        "KeyD" => Some('d'),
        "KeyE" => Some('e'),
        "KeyF" => Some('f'),
        "KeyG" => Some('g'),
        "KeyH" => Some('h'),
        "KeyI" => Some('i'),
        "KeyJ" => Some('j'),
        "KeyK" => Some('k'),
        "KeyL" => Some('l'),
        "KeyM" => Some('m'),
        "KeyN" => Some('n'),
        "KeyO" => Some('o'),
        "KeyP" => Some('p'),
        "KeyQ" => Some('q'),
        "KeyR" => Some('r'),
        "KeyS" => Some('s'),
        "KeyT" => Some('t'),
        "KeyU" => Some('u'),
        "KeyV" => Some('v'),
        "KeyW" => Some('w'),
        "KeyX" => Some('x'),
        "KeyY" => Some('y'),
        "KeyZ" => Some('z'),
        "Digit0" => Some('0'),
        "Digit1" => Some('1'),
        "Digit2" => Some('2'),
        "Digit3" => Some('3'),
        "Digit4" => Some('4'),
        "Digit5" => Some('5'),
        "Digit6" => Some('6'),
        "Digit7" => Some('7'),
        "Digit8" => Some('8'),
        "Digit9" => Some('9'),
        _ => None,
    }
}

// Keyboard layout definitions
static QWERTY_KEYBOARD_LAYOUT: [[char; 10]; 4] = [
    ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
    ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
    ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';'],
    ['z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/'],
];

static COLEMACK_DH_KEYBOARD_LAYOUT: [[char; 10]; 4] = [
    ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
    ['q', 'w', 'f', 'p', 'b', 'j', 'l', 'u', 'y', ';'],
    ['a', 'r', 's', 't', 'g', 'm', 'n', 'e', 'i', 'o'],
    ['x', 'c', 'd', 'v', 'z', 'k', 'h', ',', '.', '/'],
];

static COLEMAK_KEYBOARD_LAYOUT: [[char; 10]; 4] = [
    ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
    ['q', 'w', 'f', 'p', 'g', 'j', 'l', 'u', 'y', ';'],
    ['a', 'r', 's', 't', 'd', 'h', 'n', 'e', 'i', 'o'],
    ['z', 'x', 'c', 'v', 'b', 'k', 'm', ',', '.', '/'],
];

static DVORAK_KEYBOARD_LAYOUT: [[char; 10]; 4] = [
    ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
    ['\'', ',', '.', 'p', 'y', 'f', 'g', 'c', 'r', 'l'],
    ['a', 'o', 'e', 'u', 'i', 'd', 'h', 't', 'n', 's'],
    [';', 'q', 'j', 'k', 'x', 'b', 'm', 'w', 'v', 'z'],
];

static WORKMAN_KEYBOARD_LAYOUT: [[char; 10]; 4] = [
    ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
    ['q', 'd', 'r', 'w', 'b', 'j', 'f', 'u', 'p', ';'],
    ['a', 's', 'h', 't', 'g', 'y', 'n', 'e', 'o', 'i'],
    ['z', 'x', 'm', 'c', 'v', 'k', 'l', ',', '.', '/'],
];

static AZERTY_KEYBOARD_LAYOUT: [[char; 10]; 4] = [
    ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
    ['a', 'z', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
    ['q', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'm'],
    ['w', 'x', 'c', 'v', 'b', 'n', ',', ';', ':', '!'],
];

static QWERTZ_KEYBOARD_LAYOUT: [[char; 10]; 4] = [
    ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
    ['q', 'w', 'e', 'r', 't', 'z', 'u', 'i', 'o', 'p'],
    ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'ö'],
    ['y', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '-'],
];

/// A buffered, auto-clearing typeahead handle: append keystrokes with
/// [`Typeahead::on_input`], get back the best-matching available option's
/// index, and the buffer clears itself if the caller stops typing for the
/// configured timeout. Also adapts to the user's keyboard layout via
/// [`Typeahead::learn_from_keyboard_event`].
#[derive(Clone, Copy)]
pub struct Typeahead {
    adaptive_keyboard: Signal<AdaptiveKeyboard>,
    buffer: Signal<String>,
    clear_task: Signal<Option<Task>>,
    timeout: ReadSignal<Duration>,
}

/// Create a [`Typeahead`] handle. `timeout` is how long to wait after the
/// last keystroke before clearing the buffer.
pub fn use_typeahead(timeout: ReadSignal<Duration>) -> Typeahead {
    Typeahead {
        adaptive_keyboard: use_signal(AdaptiveKeyboard::new),
        buffer: use_signal(String::new),
        clear_task: use_signal(|| None),
        timeout,
    }
}

impl Typeahead {
    /// Learn from a keyboard event mapping physical key to logical character,
    /// so later matches account for non-QWERTY layouts.
    pub fn learn_from_keyboard_event(&mut self, physical_code: &str, logical_char: char) {
        let mut adaptive = self.adaptive_keyboard.write();
        let logical_char = logical_char.to_lowercase().next().unwrap_or(logical_char);
        adaptive.learn_from_event(physical_code, logical_char);
    }

    /// Append `text` to the buffer and return the index of the best-matching
    /// available option, if any. Schedules the buffer to clear itself after
    /// the configured timeout, canceling any previously scheduled clear.
    pub fn on_input(
        &mut self,
        text: &str,
        options: &[OptionState],
        is_available: impl Fn(usize) -> bool,
    ) -> Option<usize> {
        if let Some(existing_task) = self.clear_task.write().take() {
            existing_task.cancel();
        }

        let typeahead = {
            let mut buffer = self.buffer.write();
            buffer.push_str(text);
            buffer.clone()
        };

        let mut buffer_signal = self.buffer;
        let mut clear_task_signal = self.clear_task;
        let timeout = self.timeout.cloned();
        let new_task = spawn(async move {
            crate::time::sleep(timeout).await;
            buffer_signal.write().clear();
            clear_task_signal.write().take();
        });
        self.clear_task.write().replace(new_task);

        let keyboard = self.adaptive_keyboard.read();
        best_match(&keyboard, &typeahead, options, is_available)
    }

    /// Clear the buffer and cancel any pending auto-clear task, e.g. when the
    /// consuming widget closes.
    pub fn clear(&mut self) {
        if let Some(task) = self.clear_task.write().take() {
            task.cancel();
        }
        self.buffer.take();
    }
}
