//! Inline review of an AI rewrite: each changed span is accepted or rejected in place, with undo.
//!
//! Design choice: the spec's control handle (`highlight`, `accept`, `reject`, `undo`, ...) is driven by
//! keys and buttons inside the component. A host that needs to drive it passes `segments` and reads the
//! text through `on_text_change`.
//! Design choice: the diff is word-level LCS. Each token carries the whitespace before its word, so
//! segment text concatenates back to the exact source text, and punctuation stays attached to its word.
//! Design choice: undo restores the most recent decision. A pending change keeps the original words.
//! Browser checks: accepting shrinks the removed words away; the popover follows the highlighted change;
//! the arrows hide below narrow widths and Tab still reaches every pending change.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/inline-edit-review.css");

#[derive(Clone, PartialEq, Debug)]
pub enum Segment {
    Same(String),
    Change { removed: String, inserted: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Accepted,
    Rejected,
}

/// Splits `text` into tokens of whitespace-then-word. Leading whitespace stays on the first token, so
/// concatenating the tokens reproduces `text` exactly.
fn tokens(text: &str) -> Vec<&str> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut cuts = vec![0_usize];
    let mut prev_ws = false;
    let mut run_start = 0;
    for (at, c) in text.char_indices() {
        let ws = c.is_whitespace();
        if ws && !prev_ws {
            run_start = at;
        }
        if !ws && prev_ws && run_start > 0 {
            cuts.push(run_start);
        }
        prev_ws = ws;
    }
    cuts.push(text.len());
    cuts.windows(2)
        .map(|pair| &text[pair[0]..pair[1]])
        .collect()
}

/// Word-level diff of `before` against `after`, changes merged where removed and inserted words meet.
pub fn word_diff(before: &str, after: &str) -> Vec<Segment> {
    let a = tokens(before);
    let b = tokens(after);
    let (n, m) = (a.len(), b.len());
    let mut table = vec![vec![0_usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            table[i][j] = if a[i].trim_start() == b[j].trim_start() {
                table[i + 1][j + 1] + 1
            } else {
                table[i + 1][j].max(table[i][j + 1])
            };
        }
    }
    let mut ops: Vec<(bool, Option<&str>, Option<&str>)> = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && a[i].trim_start() == b[j].trim_start() {
            ops.push((true, Some(a[i]), Some(b[j])));
            i += 1;
            j += 1;
        } else if j < m && (i == n || table[i][j + 1] >= table[i + 1][j]) {
            ops.push((false, None, Some(b[j])));
            j += 1;
        } else {
            ops.push((false, Some(a[i]), None));
            i += 1;
        }
    }
    let mut out: Vec<Segment> = Vec::new();
    let mut removed: Vec<&str> = Vec::new();
    let mut inserted: Vec<&str> = Vec::new();
    let flush = |out: &mut Vec<Segment>, removed: &mut Vec<&str>, inserted: &mut Vec<&str>| {
        if !removed.is_empty() || !inserted.is_empty() {
            out.push(Segment::Change {
                removed: removed.concat(),
                inserted: inserted.concat(),
            });
            removed.clear();
            inserted.clear();
        }
    };
    for (same, old, new) in ops {
        if same {
            flush(&mut out, &mut removed, &mut inserted);
            out.push(Segment::Same(old.unwrap_or_default().to_string()));
        } else {
            if let Some(word) = old {
                removed.push(word);
            }
            if let Some(word) = new {
                inserted.push(word);
            }
        }
    }
    flush(&mut out, &mut removed, &mut inserted);
    out
}

/// Changes in order, by index into `segments`.
pub fn change_indices(segments: &[Segment]) -> Vec<usize> {
    segments
        .iter()
        .enumerate()
        .filter(|(_, segment)| matches!(segment, Segment::Change { .. }))
        .map(|(index, _)| index)
        .collect()
}

/// Text after decisions: accepted changes take their inserted text, rejected and pending ones keep the original.
/// Segment text carries its own whitespace and is concatenated as-is.
pub fn apply_decisions(segments: &[Segment], decisions: &[Option<Decision>]) -> String {
    let mut text = String::new();
    let mut change_at = 0;
    for segment in segments {
        match segment {
            Segment::Same(same) => text.push_str(same),
            Segment::Change { removed, inserted } => {
                let decision = decisions.get(change_at).copied().flatten();
                let chosen = match decision {
                    Some(Decision::Accepted) => inserted,
                    _ => removed,
                };
                text.push_str(chosen);
                change_at += 1;
            }
        }
    }
    text
}

pub fn pending_count(decisions: &[Option<Decision>]) -> usize {
    decisions
        .iter()
        .filter(|decision| decision.is_none())
        .count()
}

/// Inline diff review over `before_text` and `after_text`, or over precomputed `segments`.
#[component]
pub fn InlineEditReview(
    #[props(default, into)] before_text: String,
    #[props(default, into)] after_text: String,
    #[props(default)] segments: Option<Vec<Segment>>,
    #[props(default)] on_text_change: Option<EventHandler<String>>,
    #[props(default)] on_complete: Option<EventHandler<String>>,
    #[props(default, into)] caption_text: String,
    #[props(default, into)] extra_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-ier-{}", next_id()));
    let segments: Vec<Segment> = segments.unwrap_or_else(|| word_diff(&before_text, &after_text));
    let change_ids = change_indices(&segments);
    let change_count = change_ids.len();
    let mut decisions = use_signal(|| vec![None::<Decision>; change_count]);
    let mut highlight = use_signal(|| Option::<usize>::None);
    let mut history = use_signal(Vec::<(usize, Option<Decision>)>::new);
    let mut announce = use_signal(String::new);

    let current = decisions();
    let pending = pending_count(&current);
    let _text_now = apply_decisions(&segments, &current);
    let class = if extra_class.is_empty() {
        "kit-ier".to_string()
    } else {
        format!("kit-ier {extra_class}")
    };
    let toolbar_text = if pending == 0 {
        "All changes reviewed".to_string()
    } else {
        format!("{pending} pending")
    };

    let decide = {
        let on_change = on_text_change.clone();
        let on_done = on_complete.clone();
        let segments = segments.clone();
        move |change: usize, next: Option<Decision>| {
            let mut updated = decisions();
            if change >= updated.len() {
                return;
            }
            let previous = updated[change];
            updated[change] = next;
            history.with_mut(|list| list.push((change, previous)));
            let remaining = pending_count(&updated);
            let text = apply_decisions(&segments, &updated);
            decisions.set(updated);
            let verb = match next {
                Some(Decision::Accepted) => "Accepted",
                Some(Decision::Rejected) => "Rejected",
                None => "Reset",
            };
            announce.set(format!("{verb}. {remaining} remaining."));
            if let Some(h) = &on_change {
                h.call(text.clone());
            }
            if remaining == 0 {
                if let Some(h) = &on_done {
                    h.call(text);
                }
            }
        }
    };
    let decide_key = decide.clone();
    let mut decide_all_accept = decide.clone();
    let mut decide_all_reject = decide.clone();

    let undo = {
        let on_change = on_text_change.clone();
        let segments = segments.clone();
        move |_: ()| {
            let Some((change, previous)) = history.with_mut(|list| list.pop()) else {
                return;
            };
            let mut updated = decisions();
            updated[change] = previous;
            let text = apply_decisions(&segments, &updated);
            decisions.set(updated);
            announce.set("Undone.".to_string());
            if let Some(h) = &on_change {
                h.call(text);
            }
        }
    };
    let mut undo_key = undo.clone();
    let mut undo_button = undo.clone();

    let step = move |step: isize| {
        let count = change_count;
        if count == 0 {
            return;
        }
        let next = match highlight() {
            Some(at) => (at as isize + step).rem_euclid(count as isize) as usize,
            None => {
                if step >= 0 {
                    0
                } else {
                    count - 1
                }
            }
        };
        highlight.set(Some(next));
    };
    let mut step_key = step;

    let nodes: Vec<Element> = segments
        .iter()
        .enumerate()
        .map(|(index, segment)| match segment {
            Segment::Same(text) => {
                let key = format!("{id}-{index}");
                rsx! { span { key: "{key}", "{text}" } }
            }
            Segment::Change { removed, inserted } => {
                let change = change_ids.iter().position(|at| *at == index).unwrap_or(0);
                let state = current.get(change).copied().flatten();
                let state_attr = match state {
                    Some(Decision::Accepted) => "accepted",
                    Some(Decision::Rejected) => "rejected",
                    None => "pending",
                };
                let active = highlight() == Some(change);
                let label = format!(
                    "Change {} of {}: replace \"{}\" with \"{}\". Y accepts, N rejects.",
                    change + 1,
                    change_count,
                    removed.trim(),
                    inserted.trim()
                );
                let key = format!("{id}-c{index}");
                let mut accept_decide = decide_key.clone();
                let mut reject_decide = decide_key.clone();
                let has_removed = !removed.is_empty();
                let has_inserted = !inserted.is_empty();
                rsx! {
                    span { key: "{key}", class: "kit-ier__change", "data-state": state_attr, "data-active": if active { "true" } else { "false" },
                        button {
                            r#type: "button",
                            class: "kit-ier__hit",
                            aria_label: "{label}",
                            onclick: move |_| highlight.set(Some(change)),
                            onkeydown: move |event: KeyboardEvent| {
                                match event.key() {
                                    Key::Character(ref c) if c.eq_ignore_ascii_case("y") => {
                                        event.prevent_default();
                                        accept_decide(change, Some(Decision::Accepted));
                                    }
                                    Key::Enter => {
                                        event.prevent_default();
                                        accept_decide(change, Some(Decision::Accepted));
                                    }
                                    Key::Character(ref c) if c.eq_ignore_ascii_case("n") => {
                                        event.prevent_default();
                                        reject_decide(change, Some(Decision::Rejected));
                                    }
                                    Key::Backspace => {
                                        event.prevent_default();
                                        reject_decide(change, Some(Decision::Rejected));
                                    }
                                    _ => {}
                                }
                            },
                            if has_removed && state != Some(Decision::Accepted) {
                                del { class: "kit-ier__removed", "{removed}" }
                            }
                            if has_inserted && state != Some(Decision::Rejected) {
                                ins { class: "kit-ier__inserted", "{inserted}" }
                            }
                        }
                    }
                }
            }
        })
        .collect();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { id: "{id}", class: "{class}", aria_label: "Proposed edit",
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape {
                    highlight.set(None);
                } else if matches!(event.key(), Key::ArrowRight) {
                    event.prevent_default();
                    step_key(1);
                } else if matches!(event.key(), Key::ArrowLeft) {
                    event.prevent_default();
                    step_key(-1);
                } else if (event.modifiers().ctrl() || event.modifiers().meta()) && matches!(event.key(), Key::Character(ref c) if c.eq_ignore_ascii_case("z")) {
                    event.prevent_default();
                    undo_key(());
                }
            },
            if !caption_text.is_empty() {
                p { class: "kit-ier__caption", "{caption_text}" }
            }
            p { class: "kit-ier__body",
                for node in nodes {
                    {node}
                }
            }
            div { class: "kit-ier__toolbar", role: "toolbar", aria_label: "Review changes",
                span { class: "kit-ier__count", "{toolbar_text}" }
                button {
                    r#type: "button",
                    class: "kit-ier__reject-all",
                    disabled: pending == 0,
                    onclick: move |_| {
                        for change in 0..change_count {
                            if decisions()[change].is_none() {
                                decide_all_reject(change, Some(Decision::Rejected));
                            }
                        }
                    },
                    "Reject all"
                }
                button {
                    r#type: "button",
                    class: "kit-ier__accept-all",
                    disabled: pending == 0,
                    onclick: move |_| {
                        for change in 0..change_count {
                            if decisions()[change].is_none() {
                                decide_all_accept(change, Some(Decision::Accepted));
                            }
                        }
                    },
                    "Accept all"
                }
                button {
                    r#type: "button",
                    class: "kit-ier__undo",
                    disabled: history().is_empty(),
                    onclick: move |_| undo_button(()),
                    "Undo"
                }
            }
            p { class: "kit-ier__live", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
