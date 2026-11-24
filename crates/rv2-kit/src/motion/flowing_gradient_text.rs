//! Flowing gradient text: an emphasised span filled with a gradient that sweeps
//! once when first viewed, then drifts slowly through the letters.
//!
//! Design choice: `gradient_token` maps to the design-system variable
//! `--kit-gradient-<token>`; an undefined token falls back to the accent-to-identity
//! gradient, so the fill never uses a status colour.
//! Design choice: replay remounts the fill (keyed by a counter), so the sweep
//! restarts from its first frame even when the fill is already flowing.
//! Design choice: `play_on_view: false` starts the sweep at mount.
//! Design choice: reduced motion is CSS-only; the fill keeps its fixed gradient.
//!
//! Browser checks: the sweep runs once when the text enters view and not again on
//! scroll; replay restarts the sweep from the start; the replay button takes keyboard
//! focus and shows its ring; the gradient spans line breaks; phone width wraps.

use std::ops::Range;

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/flowing-gradient-text.css");

/// The text split around the emphasised range. Indices are characters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segments {
    pub before: String,
    pub emphasis: String,
    pub after: String,
}

/// Splits `text` at character indices. `None` emphasises the whole text; an
/// out-of-range end is clamped to the text's length.
pub fn split_emphasis(text: &str, range: Option<Range<usize>>) -> Segments {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let range = range.unwrap_or(0..len);
    let start = range.start.min(len);
    let end = range.end.clamp(start, len);
    Segments {
        before: chars[..start].iter().collect(),
        emphasis: chars[start..end].iter().collect(),
        after: chars[end..].iter().collect(),
    }
}

/// Character range covering the last `words` whitespace-separated words of `text`.
pub fn last_words_range(text: &str, words: usize) -> Range<usize> {
    let chars: Vec<char> = text.chars().collect();
    let mut start = chars.len();
    let mut seen = 0;
    let mut in_word = false;
    for (i, c) in chars.iter().enumerate().rev() {
        if c.is_whitespace() {
            if in_word {
                seen += 1;
                if seen == words {
                    break;
                }
            }
            in_word = false;
        } else {
            in_word = true;
            start = i;
        }
    }
    start..chars.len()
}

/// The token reduced to ASCII letters, digits and `-`, so caller input cannot
/// break out of a style attribute.
pub fn token_name(token: &str) -> String {
    token
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect()
}

/// CSS `background-image` for the fill.
pub fn gradient_fill(token: &str) -> String {
    let fallback = "linear-gradient(90deg, var(--kit-accent), var(--kit-id-1))";
    let name = token_name(token);
    if name.is_empty() {
        fallback.to_string()
    } else {
        format!("var(--kit-gradient-{name}, {fallback})")
    }
}

/// Where the fill is in its life. Waiting shows the plain ink fill.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GradientPhase {
    #[default]
    Waiting,
    Sweeping,
    Flowing,
}

impl GradientPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Sweeping => "sweeping",
            Self::Flowing => "flowing",
        }
    }
}

#[component]
pub fn FlowingGradientText(
    #[props(into)] text: String,
    #[props(default)] emphasis_range: Option<Range<usize>>,
    #[props(default)] replay_enabled: bool,
    #[props(default, into)] gradient_token: String,
    #[props(default = true)] play_on_view: bool,
    #[props(default)] class: String,
) -> Element {
    let id = use_hook(|| format!("kit-gradient-{}", next_id()));
    let initial = if play_on_view {
        GradientPhase::Waiting
    } else {
        GradientPhase::Sweeping
    };
    let mut phase = use_signal(|| initial);
    let mut replay = use_signal(|| 0u32);

    let observe_id = id.clone();
    use_effect(move || {
        if !play_on_view {
            return;
        }
        let script = format!(
            "const el = document.getElementById({observe_id:?});
             const go = () => dioxus.send(true);
             if (!el || typeof IntersectionObserver === 'undefined') {{ go(); return; }}
             const io = new IntersectionObserver(([e]) => {{
               if (!e.isIntersecting) return;
               io.disconnect();
               go();
             }}, {{ threshold: 0.5 }});
             io.observe(el);"
        );
        let mut channel = document::eval(&script);
        spawn(async move {
            if channel.recv::<bool>().await.is_ok() {
                phase.set(GradientPhase::Sweeping);
            }
        });
    });

    let parts = split_emphasis(&text, emphasis_range);
    let fill = gradient_fill(&gradient_token);
    let class = class_list("kit-gradient-text", &class);
    let fill_key = replay();
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        span { id: "{id}", class, "data-phase": phase().as_str(),
            style: "--kit-flow-fill: {fill};",
            if !parts.before.is_empty() {
                "{parts.before}"
            }
            span {
                key: "{fill_key}",
                class: "kit-gradient-text__fill",
                "data-phase": phase().as_str(),
                onanimationend: move |_| {
                    if phase() == GradientPhase::Sweeping {
                        phase.set(GradientPhase::Flowing);
                    }
                },
                "{parts.emphasis}"
            }
            if !parts.after.is_empty() {
                "{parts.after}"
            }
            if replay_enabled {
                button {
                    r#type: "button",
                    class: "kit-gradient-text__replay",
                    aria_label: "Replay animation",
                    onclick: move |_| {
                        replay += 1;
                        phase.set(GradientPhase::Sweeping);
                    },
                    "Replay"
                }
            }
        }
    }
}
