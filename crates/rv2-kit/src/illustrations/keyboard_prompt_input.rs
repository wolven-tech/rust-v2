//! Keyboard prompt input: a decorative scene for an AI chat input. A prompt field types itself above a
//! stylised keyboard; each key press lifts its key and sends a wave through a glowing floor, so the
//! scene reads as a live chat. In interactive mode the field is a real prompt. The keyboard is `aria-hidden`
//! in both modes: its key labels carry nothing a reader needs, and the field is the accessible input.
//!
//! Design choice: the typing, key lift and wave state live in three signals driven by one timer loop
//! (`lit`, `strike`, `typed`); the wave is re-keyed by `strike`, so each press replays it.
//! Design choice: the floor is the Phase 1 `GlowFloorBackdrop`; its light spot is a child that moves by
//! transform toward the most recent key.
//! Design choice: the function and modifier rows are drawn but never light; only character keys do.
//! The function row is hidden by a container query when the keyboard is narrow.
//! Design choice: the typing demo stops in interactive mode; the field becomes a textarea that
//! submits on Enter and keeps Shift + Enter for a line break.
//! Browser checks: decorative mode ignores pointer and focus; Enter submits once and Shift + Enter adds
//! a line; each typed letter lifts its key in order; the spot follows the latest key; reduced motion
//! shows the full text at once with no lift and no waves; the keyboard hides at 390 and 320.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id, prefers_motion_script, sleep};
use crate::motion::glow_floor_backdrop::{GlowAnchor, GlowFloorBackdrop, GlowIntensity};

const CSS: Asset = asset!("/assets/keyboard-prompt-input.css");

/// Function row, drawn only; it is hidden by a container query at narrow widths.
pub const FUNCTION_ROW: [&str; 13] = [
    "esc", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
];
/// Character rows, top to bottom below the function row; each key lights when its character is typed.
pub const ROWS: [&str; 4] = ["1234567890", "qwertyuiop", "asdfghjkl", "zxcvbnm"];
/// Modifier row, drawn only, below the letter rows.
pub const MODIFIER_ROW: [&str; 4] = ["shift", "ctrl", "alt", "cmd"];
/// Horizontal offset of each character row in key widths, so the rows stagger like a real keyboard.
const ROW_OFFSETS: [f64; 4] = [0.0, 0.0, 0.5, 1.0];
/// Drawn rows: function, number, three letter rows, modifiers, space bar.
const DRAWN_ROWS: f64 = 7.0;
/// Index of the space bar among the drawn rows.
const SPACE_ROW: usize = 6;
/// Pause after the last character before the field clears, ms.
pub const HOLD_MS: u32 = 1600;
/// Fade before the field clears, ms.
pub const FADE_MS: u32 = 260;
/// Pause between prompts, ms.
pub const GAP_MS: u32 = 500;
/// How long the sending state shows after a submit, ms.
pub const SEND_MS: u32 = 900;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TypingMode {
    #[default]
    Steady,
    Bursty,
}

/// Vertical centre of a drawn row, in percent of the keyboard.
fn row_centre_y(drawn_row: usize) -> f64 {
    (drawn_row as f64 + 0.5) / DRAWN_ROWS * 100.0
}

/// Centre of a key in percent of the keyboard, `None` for a character the keyboard does not draw.
pub fn key_center(c: char) -> Option<(f64, f64)> {
    let c = c.to_ascii_lowercase();
    if c == ' ' {
        return Some((50.0, row_centre_y(SPACE_ROW)));
    }
    for (row, keys) in ROWS.iter().enumerate() {
        if let Some(col) = keys.chars().position(|k| k == c) {
            let x = (col as f64 + 0.5 + ROW_OFFSETS[row]) / 10.0 * 100.0;
            return Some((x, row_centre_y(row + 1)));
        }
    }
    None
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
    let mut typed = use_signal(|| 0usize);
    let mut prompt_index = use_signal(|| 0usize);
    let mut lit = use_signal(|| None::<char>);
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
    let (spot_x, spot_y) = lit().and_then(key_center).unwrap_or((50.0, 50.0));
    let wave_style = format!("--wx: {spot_x:.1}%; --wy: {spot_y:.1}%;");
    let spot_style = format!("--gx: {spot_x:.1};");
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
                            if let Key::Character(key) = event.key() {
                                if let Some(c) = key.chars().next().filter(|_| key.chars().count() == 1) {
                                    lit.set(Some(c));
                                    strike += 1;
                                }
                            }
                            let enter = event.key() == Key::Enter;
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
                div { class: "kit-kbp__keyboard", style: "{spot_style}", "aria-hidden": "true",
                    span { class: "kit-kbp__spot", "aria-hidden": "true" }
                    if !interactive && lit().is_some() {
                        span { class: "kit-kbp__wave", key: "{strike_key}", style: "{wave_style}", "aria-hidden": "true" }
                    }
                    div { class: "kit-kbp__row", "data-row": "0",
                        for label in FUNCTION_ROW {
                            span { class: "kit-kbp__key kit-kbp__key--fn", "{label}" }
                        }
                    }
                    for (row, keys) in ROWS.iter().enumerate() {
                        div { class: "kit-kbp__row", "data-row": (row + 1).to_string(),
                            for c in keys.chars() {
                                {
                                    let is_lit = !interactive && lit_key == Some(c);
                                    let label = c.to_ascii_uppercase().to_string();
                                    rsx! {
                                        span {
                                            class: "kit-kbp__key",
                                            "data-key": "{c}",
                                            "data-lit": is_lit.then_some("true"),
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "kit-kbp__row", "data-row": "5",
                        for label in MODIFIER_ROW {
                            span { class: "kit-kbp__key kit-kbp__key--mod", "{label}" }
                        }
                    }
                    div { class: "kit-kbp__row", "data-row": "6",
                        span {
                            class: "kit-kbp__key kit-kbp__key--space",
                            "data-key": " ",
                            "data-lit": (!interactive && lit_key == Some(' ')).then_some("true"),
                            "space"
                        }
                    }
                }
            }
        }
    }
}
