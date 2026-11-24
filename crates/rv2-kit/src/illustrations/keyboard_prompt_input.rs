//! Keyboard prompt input: one prompt pill above one stylised keyboard, with a glow floor under the keys.
//! Each typed character lifts its key and sends a wave out of it; the floor's light follows the newest
//! key, so the scene reads as a live chat. In interactive mode the pill is a real prompt. The keyboard is
//! `aria-hidden` in both modes: the pill is the accessible input.
//!
//! Design choice: typing, key lift and wave state live in three signals driven by one timer loop
//! (`lit`, `strike`, `typed`); the wave is re-keyed by `strike`, so each press replays it.
//! Design choice: the floor is the `GlowFloorBackdrop`; its light spot is a sibling of the keyboard, moved
//! by `left` toward the lit key, so it reads as light on the floor under that key.
//! Design choice: a scene paused at mount rests on the first prompt about half typed, with its last key
//! lifted. A still capture of the gallery therefore shows the same frame every time.
//! Design choice: every drawn row is `ROW_UNITS` wide, so keys are placed by flex grow and a key's centre
//! is a plain sum of the widths before it.
//!
//! Browser checks: decorative mode ignores pointer and focus; Enter submits once and Shift + Enter adds
//! a line; each typed letter lifts its key in order; the spot follows the latest key; reduced motion
//! shows the full text at once with no lift and no waves; the keyboard hides at 390 and 320.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id, prefers_motion_script, sleep};
use crate::motion::glow_floor_backdrop::{GlowAnchor, GlowFloorBackdrop, GlowIntensity};

const CSS: Asset = asset!("/assets/keyboard-prompt-input.css");

/// Width of every drawn row, in key units.
pub const ROW_UNITS: f64 = 15.0;
/// Pause after the last character before the field clears, ms.
pub const HOLD_MS: u32 = 1600;
/// Fade before the field clears, ms.
pub const FADE_MS: u32 = 260;
/// Pause between prompts, ms.
pub const GAP_MS: u32 = 500;
/// How long the sending state shows after a submit, ms.
pub const SEND_MS: u32 = 900;

/// One drawn key. A key with `typed` set lights when that character is typed; `label` empty means the
/// label is the typed character in capitals.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Key {
    pub label: &'static str,
    pub units: f64,
    pub typed: Option<char>,
    pub modifier: bool,
}

const fn ch(typed: char, units: f64) -> Key {
    Key {
        label: "",
        units,
        typed: Some(typed),
        modifier: false,
    }
}

const fn named(label: &'static str, units: f64) -> Key {
    Key {
        label,
        units,
        typed: None,
        modifier: false,
    }
}

const fn modifier(label: &'static str, units: f64) -> Key {
    Key {
        label,
        units,
        typed: None,
        modifier: true,
    }
}

const fn space() -> Key {
    Key {
        label: "space",
        units: 5.0,
        typed: Some(' '),
        modifier: false,
    }
}

/// Function row: escape and twelve function keys. Drawn only.
pub const FUNCTION_ROW: &[Key] = &[
    named("esc", 1.5),
    named("F1", 1.125),
    named("F2", 1.125),
    named("F3", 1.125),
    named("F4", 1.125),
    named("F5", 1.125),
    named("F6", 1.125),
    named("F7", 1.125),
    named("F8", 1.125),
    named("F9", 1.125),
    named("F10", 1.125),
    named("F11", 1.125),
    named("F12", 1.125),
];

/// Number row: digits, symbols and backspace.
pub const NUMBER_ROW: &[Key] = &[
    ch('`', 1.0),
    ch('1', 1.0),
    ch('2', 1.0),
    ch('3', 1.0),
    ch('4', 1.0),
    ch('5', 1.0),
    ch('6', 1.0),
    ch('7', 1.0),
    ch('8', 1.0),
    ch('9', 1.0),
    ch('0', 1.0),
    ch('-', 1.0),
    ch('=', 1.0),
    modifier("back", 2.0),
];

/// First letter row: tab, the top letters, and the brackets.
pub const TAB_ROW: &[Key] = &[
    modifier("tab", 1.5),
    ch('q', 1.0),
    ch('w', 1.0),
    ch('e', 1.0),
    ch('r', 1.0),
    ch('t', 1.0),
    ch('y', 1.0),
    ch('u', 1.0),
    ch('i', 1.0),
    ch('o', 1.0),
    ch('p', 1.0),
    ch('[', 1.0),
    ch(']', 1.0),
    ch('\\', 1.5),
];

/// Second letter row: caps lock, the home letters, and return.
pub const CAPS_ROW: &[Key] = &[
    modifier("caps", 1.75),
    ch('a', 1.0),
    ch('s', 1.0),
    ch('d', 1.0),
    ch('f', 1.0),
    ch('g', 1.0),
    ch('h', 1.0),
    ch('j', 1.0),
    ch('k', 1.0),
    ch('l', 1.0),
    ch(';', 1.0),
    ch('\'', 1.0),
    modifier("return", 2.25),
];

/// Third letter row: shift, the bottom letters, punctuation, and shift.
pub const SHIFT_ROW: &[Key] = &[
    modifier("shift", 2.25),
    ch('z', 1.0),
    ch('x', 1.0),
    ch('c', 1.0),
    ch('v', 1.0),
    ch('b', 1.0),
    ch('n', 1.0),
    ch('m', 1.0),
    ch(',', 1.0),
    ch('.', 1.0),
    ch('/', 1.0),
    modifier("shift", 2.75),
];

/// Bottom row: modifiers, the long space key, and four arrows.
pub const BOTTOM_ROW: &[Key] = &[
    modifier("ctrl", 1.5),
    modifier("alt", 1.5),
    modifier("cmd", 1.5),
    space(),
    modifier("cmd", 1.5),
    named("←", 1.0),
    named("↑", 1.0),
    named("↓", 1.0),
    named("→", 1.0),
];

/// The drawn rows, top to bottom.
pub const KEY_ROWS: [&[Key]; 6] = [
    FUNCTION_ROW,
    NUMBER_ROW,
    TAB_ROW,
    CAPS_ROW,
    SHIFT_ROW,
    BOTTOM_ROW,
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TypingMode {
    #[default]
    Steady,
    Bursty,
}

/// Vertical centre of a drawn row, in percent of the keyboard.
fn row_centre_y(row: usize) -> f64 {
    (row as f64 + 0.5) / KEY_ROWS.len() as f64 * 100.0
}

/// Centre of the key that types `c`, in percent of the keyboard. `None` for a character the keyboard
/// does not draw. Capitals light the key of their lower-case letter.
pub fn key_center(c: char) -> Option<(f64, f64)> {
    let c = c.to_ascii_lowercase();
    for (row, keys) in KEY_ROWS.iter().enumerate() {
        let mut start = 0.0;
        for key in keys.iter() {
            if key.typed == Some(c) {
                return Some((
                    (start + key.units / 2.0) / ROW_UNITS * 100.0,
                    row_centre_y(row),
                ));
            }
            start += key.units;
        }
    }
    None
}

/// Horizontal position of the floor's light: the lit key's centre, or the middle with no key lit.
pub fn spot_x(lit: Option<char>) -> f64 {
    lit.and_then(key_center).map_or(50.0, |(x, _)| x)
}

/// Milliseconds before the character at `index` is typed.
pub fn char_delay_ms(mode: TypingMode, index: usize, c: char) -> u32 {
    match mode {
        TypingMode::Steady => 70,
        TypingMode::Bursty => {
            if c == ' ' || index % 5 == 4 {
                190
            } else {
                34
            }
        }
    }
}

/// The index of the next prompt, wrapping at `len`.
pub fn next_prompt(index: usize, len: usize) -> usize {
    if len == 0 { 0 } else { (index + 1) % len }
}

/// A submit happens on a plain Enter over non-blank text.
pub fn should_submit(enter: bool, shift: bool, text: &str) -> bool {
    enter && !shift && !text.trim().is_empty()
}

/// The resting frame of a paused scene: the first prompt about half typed, and its last typed key lit.
fn resting_state(prompt: Option<&str>, paused: bool) -> (usize, Option<char>) {
    match prompt.filter(|_| paused) {
        Some(prompt) => {
            let shown = prompt.chars().count() / 2;
            (
                shown,
                shown.checked_sub(1).and_then(|i| prompt.chars().nth(i)),
            )
        }
        None => (0, None),
    }
}

/// The drawn label of one key.
fn key_label(key: Key) -> String {
    if key.label.is_empty() {
        key.typed
            .map(|c| c.to_ascii_uppercase().to_string())
            .unwrap_or_default()
    } else {
        key.label.to_string()
    }
}

#[component]
pub fn KeyboardPromptInput(
    #[props(default)] sample_prompts: Vec<String>,
    #[props(default, into)] value: Option<String>,
    #[props(default)] on_submit: Option<EventHandler<String>>,
    #[props(default = "Ask anything".to_string(), into)] placeholder: String,
    #[props(default)] typing_speed_mode: TypingMode,
    #[props(default)] paused: bool,
    #[props(default)] class: String,
) -> Element {
    let interactive = on_submit.is_some();
    let uid = use_hook(|| format!("kit-kbp-{}", next_id()));
    let (rest_shown, rest_lit) = resting_state(
        sample_prompts.first().map(String::as_str),
        paused && !interactive,
    );
    let mut typed = use_signal(|| rest_shown);
    let mut prompt_index = use_signal(|| 0usize);
    let mut lit = use_signal(|| rest_lit);
    let mut strike = use_signal(|| 0u64);
    let mut fading = use_signal(|| false);
    let mut sending = use_signal(|| false);
    let mut text = use_signal(|| value.clone().unwrap_or_default());
    let mut paused_now = use_signal(|| paused);
    if *paused_now.peek() != paused {
        paused_now.set(paused);
    }

    let prompts = sample_prompts.clone();
    use_hook(move || {
        if interactive || prompts.is_empty() {
            return;
        }
        let prompts = prompts.clone();
        spawn(async move {
            let allowed = document::eval(&format!("return {};", prefers_motion_script()))
                .await
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allowed {
                let full = prompts[0].chars().count();
                typed.set(full);
                return;
            }
            loop {
                if *paused_now.peek() {
                    sleep(150).await;
                    continue;
                }
                let index = *prompt_index.peek();
                let chars: Vec<char> = prompts[index % prompts.len()].chars().collect();
                let shown = *typed.peek();
                if shown < chars.len() {
                    let c = chars[shown];
                    typed.set(shown + 1);
                    lit.set(Some(c));
                    strike += 1;
                    sleep(char_delay_ms(typing_speed_mode, shown, c)).await;
                } else {
                    sleep(HOLD_MS).await;
                    fading.set(true);
                    sleep(FADE_MS).await;
                    typed.set(0);
                    fading.set(false);
                    lit.set(None);
                    prompt_index.set(next_prompt(index, prompts.len()));
                    sleep(GAP_MS).await;
                }
            }
        });
    });

    let shown_chars: String = if interactive {
        text()
    } else {
        let index = prompt_index() % sample_prompts.len().max(1);
        match sample_prompts.get(index) {
            Some(prompt) => prompt.chars().take(typed()).collect(),
            None => String::new(),
        }
    };
    let (wave_x, wave_y) = lit().and_then(key_center).unwrap_or((50.0, 50.0));
    let wave_style = format!("--wx: {wave_x:.1}%; --wy: {wave_y:.1}%;");
    let stage_style = format!("--gx: {:.1};", spot_x(lit()));
    let strike_key = strike();
    let lit_key = lit().map(|c| c.to_ascii_lowercase());
    let class = class_list("kit-kbp", &class);
    let field_text_class = if fading() {
        "kit-kbp__text is-fading"
    } else {
        "kit-kbp__text"
    };
    let field_empty = shown_chars.is_empty() && !sending();
    let submit_text = text();
    let text_id = format!("{uid}-field");
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class, "data-mode": if interactive { "interactive" } else { "decorative" },
            "data-sending": sending().then_some("true"),
            "data-paused": paused.then_some("true"),
            "aria-hidden": (!interactive).then_some("true"),
            div { class: "kit-kbp__field",
                span { class: "kit-kbp__mark", "aria-hidden": "true" }
                if interactive {
                    textarea {
                        id: "{text_id}",
                        class: "kit-kbp__input",
                        "aria-label": "Prompt",
                        rows: "1",
                        placeholder: "{placeholder}",
                        value: "{submit_text}",
                        oninput: move |event: FormEvent| text.set(event.value()),
                        onkeydown: move |event: KeyboardEvent| {
                            if let dioxus::prelude::Key::Character(key) = event.key() {
                                if let Some(c) = key.chars().next().filter(|_| key.chars().count() == 1) {
                                    lit.set(Some(c));
                                    strike += 1;
                                }
                            }
                            let enter = event.key() == dioxus::prelude::Key::Enter;
                            if should_submit(enter, event.modifiers().shift(), &text.peek()) {
                                event.prevent_default();
                                let submitted = text.peek().clone();
                                text.set(String::new());
                                sending.set(true);
                                if let Some(handler) = on_submit {
                                    handler.call(submitted);
                                }
                                spawn(async move {
                                    sleep(SEND_MS).await;
                                    sending.set(false);
                                });
                            } else if enter && !event.modifiers().shift() {
                                event.prevent_default();
                            }
                        },
                    }
                } else {
                    div { class: field_text_class,
                        if field_empty {
                            span { class: "kit-kbp__placeholder", "{placeholder}" }
                        } else {
                            span { class: "kit-kbp__typed", "{shown_chars}" }
                            span { class: "kit-kbp__caret", "aria-hidden": "true" }
                        }
                    }
                }
                if sending() {
                    span { class: "kit-kbp__sending", "Sending" }
                }
            }
            GlowFloorBackdrop {
                intensity: GlowIntensity::Standard,
                anchor: GlowAnchor::Bottom,
                paused,
                class: "kit-kbp__floor-wrap",
                div { class: "kit-kbp__stage", style: "{stage_style}", "aria-hidden": "true",
                    div { class: "kit-kbp__keyboard", style: "{wave_style}",
                        if !interactive && lit().is_some() {
                            span { class: "kit-kbp__wave", key: "{strike_key}", "aria-hidden": "true" }
                        }
                        for (row, keys) in KEY_ROWS.iter().enumerate() {
                            div { class: "kit-kbp__row", "data-row": "{row}",
                                for key in keys.iter().copied() {
                                    {
                                        let is_lit = !interactive && key.typed.is_some() && key.typed == lit_key;
                                        let mut class = String::from("kit-kbp__key");
                                        if row == 0 {
                                            class.push_str(" kit-kbp__key--fn");
                                        }
                                        if key.modifier {
                                            class.push_str(" kit-kbp__key--mod");
                                        }
                                        if key.typed == Some(' ') {
                                            class.push_str(" kit-kbp__key--space");
                                        }
                                        let flex = format!("flex: {} 1 0;", key.units);
                                        let data_key = key.typed.map(|c| c.to_string());
                                        let label = key_label(key);
                                        rsx! {
                                            span {
                                                class: "{class}",
                                                style: "{flex}",
                                                "data-key": data_key,
                                                "data-lit": is_lit.then_some("true"),
                                                "{label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    span { class: "kit-kbp__spot", "aria-hidden": "true" }
                }
            }
        }
    }
}
