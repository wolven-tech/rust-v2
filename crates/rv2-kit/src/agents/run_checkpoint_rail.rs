//! Run checkpoint rail: a long task as an ordered list of saved points. Restarting
//! from a point starts a new attempt on its own branch; the abandoned attempt stays
//! in the list, muted.
//!
//! Design choice: the component is controlled. A restart calls `on_restart_from`
//! with the chosen entry, and the host appends the new entry (`append_restart`
//! builds that list) and moves `current_id`.
//! Design choice: an entry with no `follows_id` continues the entry before it.
//! Design choice: each attempt after the first is a branch line in a gutter left of
//! the main rail, one lane per attempt. No row prints an attempt number.
//! Design choice: the restart control is a pointer shortcut with `tabindex=-1`, so the
//! keyboard reaches restart only through Enter on the focused row.
//! Browser checks: arrow, Home and End focus moves; the rail draws down on arrival;
//! running labels shimmer; the list scrolls past 14 rows and keeps the current row in view.

use dioxus::prelude::*;

use crate::agents::single_run_status_card::{focus_by_id, format_mm_ss, format_usd};
use crate::internal::use_now_ms;
use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/run-checkpoint-rail.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RunState {
    Running,
    #[default]
    Done,
    Failed,
    Stopped,
}

impl RunState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Stopped => "stopped",
        }
    }

    fn word(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Done => "Done",
            Self::Failed => "Failed",
            Self::Stopped => "Stopped",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct CheckpointEntry {
    pub id: String,
    pub label: String,
    pub glyph: Option<String>,
    pub follows_id: Option<String>,
    pub run_state: RunState,
    pub started_ms: Option<u64>,
    pub file_count: Option<u32>,
    pub spend: Option<f64>,
}

impl CheckpointEntry {
    /// A finished entry on the main line, with no spend or file count.
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            glyph: None,
            follows_id: None,
            run_state: RunState::Done,
            started_ms: None,
            file_count: None,
            spend: None,
        }
    }
}

/// Index of the entry `i` continues from, if any.
fn predecessor(entries: &[CheckpointEntry], i: usize) -> Option<usize> {
    match &entries[i].follows_id {
        Some(parent) => entries.iter().position(|e| e.id == *parent),
        None => i.checked_sub(1),
    }
}

/// Attempt number of each entry. An entry that continues a predecessor which already
/// has an earlier successor opens a new attempt; everything else inherits its
/// predecessor's attempt.
pub fn attempt_numbers(entries: &[CheckpointEntry]) -> Vec<u32> {
    let mut attempts: Vec<u32> = Vec::with_capacity(entries.len());
    let mut branch_starts = 0_u32;
    for i in 0..entries.len() {
        let parent = predecessor(entries, i);
        let taken = parent.is_some_and(|p| (0..i).any(|j| predecessor(entries, j) == Some(p)));
        let attempt = if taken {
            branch_starts += 1;
            branch_starts + 1
        } else {
            parent.map_or(1, |p| attempts[p])
        };
        attempts.push(attempt);
    }
    attempts
}

/// Entries on the path from the first entry to `current`, marked `true`.
pub fn on_path(entries: &[CheckpointEntry], current: usize) -> Vec<bool> {
    let mut path = vec![false; entries.len()];
    let mut cursor = Some(current);
    while let Some(i) = cursor {
        path[i] = true;
        cursor = predecessor(entries, i);
    }
    path
}

/// Next focused row for a navigation key, or `None` for other keys and empty lists.
pub fn move_focus(len: usize, current: usize, key: &str) -> Option<usize> {
    if len == 0 {
        return None;
    }
    match key {
        "ArrowDown" => Some((current + 1).min(len - 1)),
        "ArrowUp" => Some(current.saturating_sub(1)),
        "Home" => Some(0),
        "End" => Some(len - 1),
        _ => None,
    }
}

/// The list with a new running entry continuing `from_id`, appended last.
pub fn append_restart(
    entries: &[CheckpointEntry],
    from_id: &str,
    new_id: &str,
    label: &str,
) -> Vec<CheckpointEntry> {
    let mut next = entries.to_vec();
    next.push(CheckpointEntry {
        follows_id: Some(from_id.to_string()),
        run_state: RunState::Running,
        ..CheckpointEntry::new(new_id, label)
    });
    next
}

const ROW_PX: usize = 40;
const LANE_FIRST_PX: usize = 8;
const LANE_STEP_PX: usize = 12;
const LANE_EDGE_PX: usize = 8;

/// Centre of branch lane `lane` in px from the card's left edge. Lane 0 is leftmost.
fn lane_px(lane: usize) -> usize {
    LANE_FIRST_PX + LANE_STEP_PX * lane
}

#[derive(Clone, PartialEq, Debug)]
struct RowView {
    index: usize,
    label: String,
    glyph: Option<String>,
    state: RunState,
    lane: Option<usize>,
    current: bool,
    left_behind: bool,
    tab: &'static str,
    file_text: Option<String>,
    elapsed: String,
    entry: CheckpointEntry,
}

#[component]
pub fn RunCheckpointRail(
    entries: Vec<CheckpointEntry>,
    #[props(default)] current_id: Option<String>,
    #[props(default)] goal_text: String,
    #[props(default = "Checkpoints".to_string(), into)] heading: String,
    #[props(default)] on_restart_from: Option<EventHandler<CheckpointEntry>>,
    #[props(default)] format_elapsed: Option<Callback<u64, String>>,
    #[props(default)] format_spend: Option<Callback<f64, String>>,
    #[props(default)] clock_ms: Option<u64>,
    #[props(default)] class_name: String,
) -> Element {
    let id = use_hook(|| format!("kit-rcr-{}", next_id()));
    let mut focus = use_signal(|| 0_usize);
    let now = use_now_ms(clock_ms, 1000);

    let current_index = current_id
        .as_deref()
        .and_then(|c| entries.iter().position(|e| e.id == c))
        .or(entries.len().checked_sub(1));
    let path = current_index
        .map(|c| on_path(&entries, c))
        .unwrap_or_else(|| vec![false; entries.len()]);
    let attempts = attempt_numbers(&entries);
    let attempt_count = attempts.iter().copied().max().unwrap_or(0);
    let spend_total: f64 = entries.iter().filter_map(|e| e.spend).sum();
    let spend_text = match format_spend {
        Some(format) => format.call(spend_total),
        None => format_usd(spend_total),
    };
    let entry_count = entries.len();
    let focused = focus().min(entry_count.saturating_sub(1));

    let lanes = attempt_count.saturating_sub(1) as usize;
    let gutter_px = if lanes == 0 {
        0
    } else {
        lane_px(lanes - 1) + LANE_EDGE_PX
    };
    let main_h = attempts.iter().rposition(|&a| a == 1).unwrap_or(0) * ROW_PX;
    let branch_lines: Vec<(usize, usize, usize)> = (2..=attempt_count)
        .filter_map(|attempt| {
            let first = attempts.iter().position(|&a| a == attempt)?;
            let last = attempts.iter().rposition(|&a| a == attempt)?;
            let start = predecessor(&entries, first).filter(|&s| s < first)?;
            Some((lane_px(attempt as usize - 2), start, last))
        })
        .collect();
    let lane_marks: Vec<(String, String)> = branch_lines
        .iter()
        .map(|&(x, start, last)| {
            (
                format!(
                    "left: {}px; top: {}px; height: {}px",
                    x - 1,
                    start * ROW_PX + ROW_PX / 2,
                    (last - start) * ROW_PX
                ),
                format!("left: {x}px; top: {}px", start * ROW_PX + ROW_PX / 2 - 1),
            )
        })
        .collect();
    let body_style = format!("--rcr-gutter: {gutter_px}px; --rcr-main-h: {main_h}px");

    let rows: Vec<(String, RowView)> = entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let elapsed = match entry.started_ms {
                Some(started) => {
                    let ms = now.saturating_sub(started);
                    match format_elapsed {
                        Some(format) => format.call(ms),
                        None => format_mm_ss(ms),
                    }
                }
                None => String::new(),
            };
            let lane = (attempts[index] > 1).then(|| lane_px(attempts[index] as usize - 2));
            let view = RowView {
                index,
                label: entry.label.clone(),
                glyph: entry.glyph.clone(),
                state: entry.run_state,
                lane,
                current: Some(index) == current_index,
                left_behind: !path[index],
                tab: if index == focused { "0" } else { "-1" },
                file_text: entry.file_count.map(|n| format!("{n} files")),
                elapsed,
                entry: entry.clone(),
            };
            (entry.id.clone(), view)
        })
        .collect();

    let count_text = format!("{entry_count} entries");
    let attempt_text = format!("{attempt_count} attempts");
    let show_attempts = attempt_count > 1;
    let row_ids: Vec<String> = (0..entry_count).map(|i| format!("{id}-row-{i}")).collect();
    let list_id = format!("{id}-list");
    let handled = entries.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "kit-rcr {class_name}",
            header { class: "kit-rcr__bar",
                h3 { id: "{id}-heading", class: "kit-rcr__heading", "{heading}" }
                if !goal_text.is_empty() {
                    span { class: "kit-rcr__goal", "{goal_text}" }
                }
                span { class: "kit-rcr__count", "{count_text}" }
                if show_attempts {
                    span { class: "kit-rcr__count", "{attempt_text}" }
                }
                span { class: "kit-rcr__spend", "{spend_text}" }
            }
            div { class: "kit-rcr__body", style: "{body_style}",
                for (lane_style, elbow_style) in lane_marks {
                    span { class: "kit-rcr__lane", "aria-hidden": "true", style: "{lane_style}" }
                    span { class: "kit-rcr__elbow", "aria-hidden": "true", style: "{elbow_style}" }
                }
                ol {
                    id: "{list_id}",
                    class: "kit-rcr__rail",
                    "aria-labelledby": "{id}-heading",
                    onkeydown: move |event: KeyboardEvent| {
                        let key = event.key();
                        if key == Key::Enter || key == Key::Character(" ".to_string()) {
                            event.prevent_default();
                            if let Some(entry) = handled.get(focused) {
                                if let Some(handler) = &on_restart_from {
                                    handler.call(entry.clone());
                                }
                            }
                            return;
                        }
                        let name = match key {
                            Key::ArrowDown => "ArrowDown",
                            Key::ArrowUp => "ArrowUp",
                            Key::Home => "Home",
                            Key::End => "End",
                            _ => "",
                        };
                        if let Some(next) = move_focus(entry_count, focused, name) {
                            event.prevent_default();
                            focus.set(next);
                            focus_by_id(&row_ids[next]);
                        }
                    },
                    for (key, view) in rows {
                        li {
                            key: "{key}",
                            id: row_ids[view.index].clone(),
                            class: "kit-rcr__row",
                            tabindex: view.tab,
                            "aria-current": view.current.then_some("step"),
                            "data-state": view.state.as_str(),
                            "data-current": view.current.then_some("true"),
                            "data-left-behind": view.left_behind.then_some("true"),
                            "data-branch": view.lane.is_some().then_some("true"),
                            style: view.lane.map(|x| format!("--rcr-x: {x}px")).unwrap_or_default(),
                            onfocus: move |_| focus.set(view.index),
                            span { class: "kit-rcr__node", "aria-hidden": "true" }
                            span { class: "kit-rcr__label",
                                if let Some(glyph) = view.glyph.clone() {
                                    span { class: "kit-rcr__glyph", "aria-hidden": "true", {glyph} }
                                }
                                span { class: "kit-rcr__name", {view.label.clone()} }
                                if view.state != RunState::Done {
                                    span { class: "kit-rcr__state", {view.state.word()} }
                                }
                            }
                            span { class: "kit-rcr__meta",
                                if let Some(files) = view.file_text.clone() {
                                    span { {files} }
                                }
                                span { class: "kit-rcr__elapsed", {view.elapsed.clone()} }
                            }
                            button {
                                class: "kit-rcr__restart",
                                r#type: "button",
                                tabindex: "-1",
                                "aria-hidden": "true",
                                onclick: move |_| {
                                    if let Some(handler) = &on_restart_from {
                                        handler.call(view.entry.clone());
                                    }
                                },
                                "Restart"
                            }
                        }
                    }
                }
            }
            span { class: "kit-rcr__live", role: "status", "aria-live": "polite", "{count_text}" }
        }
    }
}
