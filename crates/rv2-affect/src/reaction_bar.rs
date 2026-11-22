//! Reaction bar: like, comment, repost and share as a row of quiet chips. A tap
//! toggles the action with a light blue tint, pops its icon and rolls the changed
//! digits of its count.
//!
//! Port losses: the React original sizes the count in a layout effect, before
//! paint. Here the width is measured by `document::eval` after the render, so a
//! count that changes can show one frame at the old width.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, prefers_motion_script};

const CSS: Asset = asset!("/assets/reaction-bar.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReactionKey {
    #[default]
    Like,
    Comment,
    Repost,
    Share,
}

impl ReactionKey {
    pub const ALL: [ReactionKey; 4] = [Self::Like, Self::Comment, Self::Repost, Self::Share];

    fn index(self) -> usize {
        self as usize
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Like => "like",
            Self::Comment => "comment",
            Self::Repost => "repost",
            Self::Share => "share",
        }
    }

    /// The verb for the button and the plural noun for the count.
    fn labels(self) -> (&'static str, &'static str) {
        match self {
            Self::Like => ("Like", "likes"),
            Self::Comment => ("Comment", "comments"),
            Self::Repost => ("Repost", "reposts"),
            Self::Share => ("Share", "shares"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReactionState {
    pub count: u64,
    pub active: bool,
}

/// Flips `active` and moves the count by one, never below zero.
pub fn toggled(state: ReactionState) -> ReactionState {
    if state.active {
        ReactionState {
            count: state.count.saturating_sub(1),
            active: false,
        }
    } else {
        ReactionState {
            count: state.count + 1,
            active: true,
        }
    }
}

/// Drops repeated actions, keeping the first occurrence of each.
pub fn unique_actions(actions: &[ReactionKey]) -> Vec<ReactionKey> {
    let mut out: Vec<ReactionKey> = Vec::with_capacity(actions.len());
    for &key in actions {
        if !out.contains(&key) {
            out.push(key);
        }
    }
    out
}

/// `1234` → `"1.2K"`. One decimal below ten, whole numbers above, half rounded up.
pub fn compact_count(n: u64) -> String {
    const UNITS: [(u64, &str); 3] = [(1_000_000_000, "B"), (1_000_000, "M"), (1_000, "K")];
    for (size, suffix) in UNITS {
        if n >= size {
            let scaled = n as f64 / size as f64;
            let number = if scaled < 10.0 {
                format!("{:.1}", (scaled * 10.0).round() / 10.0)
            } else {
                format!("{}", scaled.round())
            };
            return format!("{}{suffix}", number.trim_end_matches(".0"));
        }
    }
    n.to_string()
}

/// The count as shown in the pill. From 10,000 to 999,999 it rounds to whole
/// thousands first, so 999,950 reads `1M` and not `1000K`.
pub fn short_count(n: u64) -> String {
    if (10_000..1_000_000).contains(&n) {
        compact_count((n as f64 / 1000.0).round() as u64 * 1000)
    } else {
        compact_count(n)
    }
}

/// `1234` → `"1,234"`.
pub fn exact_count(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// One `(key, char)` per character of `text`. The key counts positions from the
/// end, so a digit that moves keeps its identity and only new positions roll in.
pub fn digit_slots(text: &str) -> Vec<(String, char)> {
    let len = text.chars().count();
    text.chars()
        .enumerate()
        .map(|(i, ch)| (format!("{}-{ch}", len - i), ch))
        .collect()
}

#[component]
pub fn ReactionBar(
    /// Starting counts and whether each action is already taken. Missing actions start at zero.
    #[props(default)]
    reactions: Vec<(ReactionKey, ReactionState)>,
    /// Called after an action was toggled, with its new state.
    #[props(default)]
    on_react: Option<EventHandler<(ReactionKey, ReactionState)>>,
    /// Hide actions you do not need.
    #[props(default = ReactionKey::ALL.to_vec())]
    actions: Vec<ReactionKey>,
    #[props(default)] class: String,
) -> Element {
    let actions = unique_actions(&actions);
    let initial = ReactionKey::ALL.map(|key| {
        reactions
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, state)| *state)
            .unwrap_or_default()
    });
    let mut state = use_signal(move || initial);
    let mut popped = use_signal(|| [0u32; 4]);

    let mut toggle = move |key: ReactionKey| {
        let next = toggled(state()[key.index()]);
        state.write()[key.index()] = next;
        popped.write()[key.index()] += 1;
        if let Some(handler) = &on_react {
            handler.call((key, next));
        }
    };

    if actions.is_empty() {
        return rsx! {};
    }

    let class = class_list("ui-rbc", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class, role: "group", aria_label: "Reactions",
            for key in actions {
                {
                    let ReactionState { count, active } = state()[key.index()];
                    let (verb, noun) = key.labels();
                    let singular = noun.trim_end_matches('s');
                    let aria = if count > 0 {
                        let word = if count == 1 { singular } else { noun };
                        format!("{verb}, {} {word}", exact_count(count))
                    } else {
                        verb.to_string()
                    };
                    let title = (count >= 1000).then(|| format!("{} {noun}", exact_count(count)));
                    let pop = popped()[key.index()];
                    let key_name = key.as_str();
                    rsx! {
                        button {
                            key: "{key_name}",
                            r#type: "button",
                            class: "ui-rbc__btn",
                            "data-kind": key_name,
                            aria_pressed: if active { "true" } else { "false" },
                            aria_label: "{aria}",
                            title,
                            onclick: move |_| toggle(key),
                            span {
                                class: "ui-rbc__icon",
                                key: "{pop}",
                                "data-pop": (pop > 0).then_some("true"),
                                {icon(key)}
                            }
                            Count { value: count }
                        }
                    }
                }
            }
        }
    }
}

/// The count glides to its new width; each changed digit rolls in from below.
#[component]
fn Count(value: u64) -> Element {
    let text = if value > 0 {
        short_count(value)
    } else {
        String::new()
    };
    let id = use_hook(|| format!("ui-rb-count-{}", next_id()));
    let inner_id = format!("{id}-digits");

    let (measure_id, measure_inner) = (id.clone(), inner_id.clone());
    use_effect(use_reactive((&text,), move |(_text,)| {
        let script = format!(
            "const o = document.getElementById({measure_id:?});
             const i = document.getElementById({measure_inner:?});
             if (!o || !i) return;
             o.style.width = i.offsetWidth + 'px';
             if (!o.hasAttribute('data-seen')) {{
               o.setAttribute('data-seen', '');
               if ({motion}) requestAnimationFrame(() => o.setAttribute('data-smooth', ''));
             }}",
            motion = prefers_motion_script(),
        );
        document::eval(&script);
    }));

    rsx! {
        span { id: "{id}", class: "ui-rbc__count", "aria-hidden": "true",
            span { id: "{inner_id}", class: "ui-rbc__digits",
                for (key, ch) in digit_slots(&text) {
                    span { key: "{key}", class: "ui-rbc__digit", "{ch}" }
                }
            }
        }
    }
}

fn icon(key: ReactionKey) -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            width: "16",
            height: "16",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.65",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            {match key {
                ReactionKey::Like => rsx! {
                    path { class: "ui-rbc__fill", d: "M19.46 4c-2.68-1.65-5.02-.99-6.43.07-.57.43-.86.65-1.03.65s-.46-.22-1.03-.65C9.56 3.01 7.22 2.35 4.54 4 1.02 6.15.22 13.27 8.34 19.28 9.89 20.43 10.66 21 12 21s2.11-.57 3.66-1.72C23.78 13.27 22.98 6.15 19.46 4Z" }
                },
                ReactionKey::Comment => rsx! {
                    path { class: "ui-rbc__fill", d: "M12 21.5c5.52 0 10-4.25 10-9.5S17.52 2.5 12 2.5 2 6.75 2 12c0 1.36.3 2.65.84 3.82.15.31.2.66.1 1l-.6 2.23a1.08 1.08 0 0 0 1.6 1.59l2.22-.6c.34-.09.69-.04 1 .1 1.17.55 2.46.86 4.84.86Z" }
                },
                ReactionKey::Repost => rsx! {
                    path { d: "M16.5 3.5 19 6l-2.5 2.5" }
                    path { d: "M19 6H8a4 4 0 0 0-4 4v1" }
                    path { d: "M7.5 20.5 5 18l2.5-2.5" }
                    path { d: "M5 18h11a4 4 0 0 0 4-4v-1" }
                },
                ReactionKey::Share => rsx! {
                    path { d: "M21 3 10.5 13.5" }
                    path { d: "M21 3 14.6 20.6a.5.5 0 0 1-.93.05L10.5 13.5 3.35 10.33a.5.5 0 0 1 .05-.93Z" }
                },
            }}
        }
    }
}
