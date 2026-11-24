//! Inline review of an AI rewrite: each changed span is accepted or rejected in place, with undo.
//!
//! Design choice: the spec's control handle (`highlight`, `accept`, `reject`, `undo`, ...) is driven by
//! keys and buttons inside the component. A host that needs to drive it passes `segments` and reads the
//! text through `on_text_change`.
//! Design choice: the diff is word-level LCS. Each token carries the whitespace before its word, so
//! segment text concatenates back to the exact source text, and punctuation stays attached to its word.
//! Design choice: undo restores the most recent decision. A pending change keeps the original words.
//! Design choice: the popover hangs from the end of its change when the change sits past the midpoint
//! of the text, so it stays inside the panel.
//! Browser checks: accepting shrinks the removed words away; the popover follows the highlighted change;
//! the arrows hide below narrow widths and Tab still reaches every pending change.

use dioxus::prelude::*;

use crate::agents::single_run_status_card::focus_by_id;
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

/// The next pending change after `current`, or the first (step forward) or last (step back) when
/// nothing is highlighted. Decided changes are skipped and the search wraps.
pub fn next_pending(
    decisions: &[Option<Decision>],
    current: Option<usize>,
    step: isize,
) -> Option<usize> {
    let pending: Vec<usize> = decisions
        .iter()
        .enumerate()
        .filter(|(_, decision)| decision.is_none())
        .map(|(index, _)| index)
        .collect();
    let (first, last) = (*pending.first()?, *pending.last()?);
    let forward = step >= 0;
    Some(match current {
        None if forward => first,
        None => last,
        Some(at) if forward => pending.iter().copied().find(|&i| i > at).unwrap_or(first),
        Some(at) => pending
            .iter()
            .rev()
            .copied()
            .find(|&i| i < at)
            .unwrap_or(last),
    })
}

/// Accessible name of a change: its position, the words it replaces, and the keys that decide it.
pub fn change_label(index: usize, total: usize, removed: &str, inserted: &str) -> String {
    let (removed, inserted) = (removed.trim(), inserted.trim());
    let what = match (removed.is_empty(), inserted.is_empty()) {
        (false, false) => format!("replaces “{removed}” with “{inserted}”"),
        (false, true) => format!("removes “{removed}”"),
        _ => format!("adds “{inserted}”"),
    };
    format!(
        "Change {} of {total}: {what}. Y accepts, N rejects.",
        index + 1
    )
}

/// Whether the popover of change `change` hangs from the end of the change: the change sits past the
/// midpoint of the text, so a popover that hangs from its start would cross the panel's right edge.
pub fn popover_hangs_from_end(segments: &[Segment], change: usize) -> bool {
    let Some(&at) = change_indices(segments).get(change) else {
        return false;
    };
    let weight = |segment: &Segment| match segment {
        Segment::Same(text) => text.chars().count(),
        Segment::Change { removed, inserted } => removed.chars().count() + inserted.chars().count(),
    };
    let total: usize = segments.iter().map(weight).sum();
    let before: usize = segments[..at].iter().map(weight).sum();
    before * 2 > total
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
    let mut focus_request = use_signal(|| None::<String>);
    use_effect(move || {
        if let Some(target) = focus_request() {
            focus_request.set(None);
            focus_by_id(&target);
        }
    });
    let undo_reject = format!("{id}-undo");
    let undo_accept = undo_reject.clone();

    let current = decisions();
    let pending = pending_count(&current);
    let class = if extra_class.is_empty() {
        "kit-ier".to_string()
    } else {
        format!("kit-ier {extra_class}")
    };
    let review_words = if pending == 0 {
        "All changes reviewed"
    } else {
        "pending changes"
    };
    let review_state = if pending == 0 { "reviewed" } else { "pending" };

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
    let mut decide_all_accept = decide.clone();
    let mut decide_all_reject = decide.clone();

    // Keyboard and popover decisions move focus on: to the next pending change, or to Undo when none remain.
    let advance = {
        let mut decide = decide.clone();
        let id = id.clone();
        move |change: usize, next: Decision| {
            decide(change, Some(next));
            match next_pending(&decisions(), Some(change), 1) {
                Some(target) => {
                    highlight.set(Some(target));
                    focus_request.set(Some(format!("{id}-h{target}")));
                }
                None => {
                    highlight.set(None);
                    focus_request.set(Some(format!("{id}-undo")));
                }
            }
        }
    };

    let undo = {
        let on_change = on_text_change.clone();
        let segments = segments.clone();
        let id = id.clone();
        move |_: ()| {
            let Some((change, previous)) = history.with_mut(|list| list.pop()) else {
                return;
            };
            let mut updated = decisions();
            updated[change] = previous;
            let text = apply_decisions(&segments, &updated);
            decisions.set(updated);
            announce.set("Undone.".to_string());
            highlight.set(Some(change));
            if let Some(h) = &on_change {
                h.call(text);
            }
            if history().is_empty() {
                focus_request.set(Some(format!("{id}-h{change}")));
            }
        }
    };
    let mut undo_key = undo.clone();
    let mut undo_button = undo.clone();

    let step = move |step: isize| {
        if let Some(next) = next_pending(&decisions(), highlight(), step) {
            highlight.set(Some(next));
        }
    };
    let mut step_key = step;
    let mut step_prev = step;
    let mut step_next = step;

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
                let label = change_label(change, change_count, removed, inserted);
                let key = format!("{id}-c{index}");
                let hit = format!("{id}-h{change}");
                let show_removed = !removed.is_empty() && state != Some(Decision::Accepted);
                let show_inserted = !inserted.is_empty() && state != Some(Decision::Rejected);
                let spaced = show_removed && show_inserted;
                let show_popover = active && state.is_none();
                let side = if popover_hangs_from_end(&segments, change) {
                    "end"
                } else {
                    "start"
                };
                let mut accept_click = advance.clone();
                let mut reject_click = advance.clone();
                let mut accept_key = advance.clone();
                let mut reject_key = advance.clone();
                rsx! {
                    span { key: "{key}", class: "kit-ier__change", "data-state": state_attr, "data-active": if active { "true" } else { "false" },
                        button {
                            id: "{hit}",
                            r#type: "button",
                            class: "kit-ier__hit",
                            aria_label: "{label}",
                            onclick: move |_| highlight.set(Some(change)),
                            onfocus: move |_| highlight.set(Some(change)),
                            onkeydown: move |event: KeyboardEvent| {
                                match event.key() {
                                    Key::Character(ref c) if c.eq_ignore_ascii_case("y") => {
                                        event.prevent_default();
                                        accept_key(change, Decision::Accepted);
                                    }
                                    Key::Enter => {
                                        event.prevent_default();
                                        accept_key(change, Decision::Accepted);
                                    }
                                    Key::Character(ref c) if c.eq_ignore_ascii_case("n") => {
                                        event.prevent_default();
                                        reject_key(change, Decision::Rejected);
                                    }
                                    Key::Backspace => {
                                        event.prevent_default();
                                        reject_key(change, Decision::Rejected);
                                    }
                                    _ => {}
                                }
                            },
                            if show_removed {
                                del { class: "kit-ier__removed", "{removed}" }
                            }
                            if spaced {
                                " "
                            }
                            if show_inserted {
                                ins { class: "kit-ier__inserted", "{inserted}" }
                            }
                        }
                        if show_popover {
                            span { class: "kit-ier__pop", "data-side": side,
                                button {
                                    r#type: "button",
                                    class: "kit-ier__accept",
                                    onclick: move |_| accept_click(change, Decision::Accepted),
                                    "Accept"
                                    span { class: "kit-ier__key", "aria-hidden": "true", "Y" }
                                }
                                button {
                                    r#type: "button",
                                    class: "kit-ier__reject",
                                    onclick: move |_| reject_click(change, Decision::Rejected),
                                    "Reject"
                                    span { class: "kit-ier__key", "aria-hidden": "true", "N" }
                                }
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
        section { id: "{id}", class: "{class}", "data-state": review_state, aria_label: "Proposed edit",
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
                span { class: "kit-ier__count",
                    span { class: "kit-ier__badge", "{pending}" }
                    "{review_words}"
                }
                button {
                    r#type: "button",
                    id: "{id}-prev",
                    class: "kit-ier__arrow",
                    "aria-label": "Previous change",
                    disabled: pending == 0,
                    onclick: move |_| step_prev(-1),
                    svg { view_box: "0 0 16 16", width: "14", height: "14", fill: "none", stroke: "currentColor", "aria-hidden": "true",
                        path { d: "M10 4 6 8l4 4" }
                    }
                }
                button {
                    r#type: "button",
                    id: "{id}-next",
                    class: "kit-ier__arrow",
                    "aria-label": "Next change",
                    disabled: pending == 0,
                    onclick: move |_| step_next(1),
                    svg { view_box: "0 0 16 16", width: "14", height: "14", fill: "none", stroke: "currentColor", "aria-hidden": "true",
                        path { d: "M6 4l4 4-4 4" }
                    }
                }
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
                        focus_request.set(Some(undo_reject.clone()));
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
                        focus_request.set(Some(undo_accept.clone()));
                    },
                    "Accept all"
                }
                if !history().is_empty() {
                    button {
                        r#type: "button",
                        id: "{id}-undo",
                        class: "kit-ier__undo",
                        onclick: move |_| undo_button(()),
                        "Undo"
                    }
                }
            }
            p { class: "kit-ier__live", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
