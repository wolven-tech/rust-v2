//! Change replay: what a coding agent changed across a run, one edit at a time,
//! as a two-series chart and a list of the files it touched. It follows a run
//! while it writes, or replays a finished one up to `shown_until_offset`.
//!
//! Design choice: offsets are seconds from the start of the run; `axis_length`
//! defaults to the run length rounded up to a whole minute.
//! Design choice: scales are taken from every event, so the axis does not
//! rescale during replay.
//! Design choice: the identity mark is a letter plus one of six palette slots
//! chosen from the author id, so it is stable across renders.
//! Design choice: the empty state shows the axis and an empty file list.
//! Browser checks: the pulse and row rise, the glow on the file being written,
//! the hover mirror on the baseline, and the folder names hiding at narrow width.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/change-replay.css");

#[derive(Clone, PartialEq, Debug)]
pub struct ReplayAuthor {
    pub author_id: String,
    pub author_name: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ChangeEvent {
    /// Seconds from the start of the run.
    pub at_offset: f64,
    pub file_path: String,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub line_details: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Totals {
    pub added: u64,
    pub removed: u64,
    pub net: i64,
    pub files: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FileRow {
    pub path: String,
    pub added: u64,
    pub removed: u64,
    pub first_offset: f64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReplayKey {
    Left,
    Right,
    Home,
    End,
}

/// Non-numeric and negative offsets count as zero.
pub fn sanitize(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}

/// Events at or before `shown_until`; all events when it is `None`.
pub fn shown_events(
    events: &[ChangeEvent],
    shown_until: Option<f64>,
) -> Vec<(usize, &ChangeEvent)> {
    events
        .iter()
        .enumerate()
        .filter(|(_, e)| shown_until.is_none_or(|limit| sanitize(e.at_offset) <= limit))
        .collect()
}

pub fn totals(events: &[ChangeEvent], shown_until: Option<f64>) -> Totals {
    let visible = shown_events(events, shown_until);
    let added: u64 = visible.iter().map(|(_, e)| u64::from(e.lines_added)).sum();
    let removed: u64 = visible
        .iter()
        .map(|(_, e)| u64::from(e.lines_removed))
        .sum();
    let mut files: Vec<&str> = visible.iter().map(|(_, e)| e.file_path.as_str()).collect();
    files.sort_unstable();
    files.dedup();
    Totals {
        added,
        removed,
        net: added as i64 - removed as i64,
        files: files.len(),
    }
}

/// Run length rounded up to a whole minute, at least one minute.
pub fn default_axis(events: &[ChangeEvent]) -> f64 {
    let last = events
        .iter()
        .map(|e| sanitize(e.at_offset))
        .fold(0.0_f64, f64::max);
    ((last / 60.0).ceil().max(1.0)) * 60.0
}

/// One row per file, in the order the agent first touched it. A file appears
/// only once one of its events is visible, and keeps its position.
pub fn file_rows(events: &[ChangeEvent], shown_until: Option<f64>) -> Vec<FileRow> {
    let mut rows: Vec<FileRow> = Vec::new();
    for (_, event) in shown_events(events, shown_until) {
        let index = match rows.iter().position(|r| r.path == event.file_path) {
            Some(index) => index,
            None => {
                rows.push(FileRow {
                    path: event.file_path.clone(),
                    added: 0,
                    removed: 0,
                    first_offset: sanitize(event.at_offset),
                });
                rows.len() - 1
            }
        };
        rows[index].added += u64::from(event.lines_added);
        rows[index].removed += u64::from(event.lines_removed);
    }
    rows
}

/// Offsets at which `path` was edited, among visible events.
pub fn file_marks(events: &[ChangeEvent], shown_until: Option<f64>, path: &str) -> Vec<f64> {
    shown_events(events, shown_until)
        .into_iter()
        .filter(|(_, e)| e.file_path == path)
        .map(|(_, e)| sanitize(e.at_offset))
        .collect()
}

/// The largest single-event count across all events, at least one.
pub fn scale_max(events: &[ChangeEvent]) -> f64 {
    events
        .iter()
        .map(|e| f64::from(e.lines_added.max(e.lines_removed)))
        .fold(1.0, f64::max)
}

/// Polyline points for the added and removed series in a `width` by `height` box.
pub fn series_points(
    events: &[ChangeEvent],
    shown_until: Option<f64>,
    axis: f64,
    width: f64,
    height: f64,
    scale: f64,
) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
    let axis = if axis > 0.0 { axis } else { 1.0 };
    let mut added = Vec::new();
    let mut removed = Vec::new();
    for (_, event) in shown_events(events, shown_until) {
        let x = (sanitize(event.at_offset) / axis * width).min(width);
        let a = height - f64::from(event.lines_added) / scale * height;
        let r = height - f64::from(event.lines_removed) / scale * height;
        added.push((x, a));
        removed.push((x, r));
    }
    (added, removed)
}

/// Index among `visible` after a key press; `None` when there is nothing to move to.
pub fn step_index(len: usize, current: Option<usize>, key: ReplayKey) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match key {
        ReplayKey::Home => 0,
        ReplayKey::End => len - 1,
        ReplayKey::Left => current.map_or(len - 1, |i| i.saturating_sub(1)),
        ReplayKey::Right => current.map_or(0, |i| (i + 1).min(len - 1)),
    })
}

/// `4800` as `4.8k`, `2_400_000` as `2.4M`; small values unchanged.
pub fn compact(value: u64) -> String {
    if value >= 1_000_000 {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    } else if value >= 1_000 {
        format!("{:.1}k", value as f64 / 1_000.0)
    } else {
        value.to_string()
    }
}

/// Splits a path into its folder part (with trailing slash) and its file name.
pub fn split_path(path: &str) -> (String, String) {
    match path.rfind('/') {
        Some(index) => (path[..=index].to_string(), path[index + 1..].to_string()),
        None => (String::new(), path.to_string()),
    }
}

/// `m:ss` for an offset in seconds.
pub fn clock(seconds: f64) -> String {
    let total = sanitize(seconds).round() as u64;
    format!("{}:{:02}", total / 60, total % 60)
}

/// Stable palette slot for an author id, 0 to 5.
pub fn mark_index(author_id: &str) -> usize {
    let hash = author_id.bytes().fold(5381_u64, |h, b| {
        h.wrapping_mul(33).wrapping_add(u64::from(b))
    });
    (hash % 6) as usize
}

fn focus_id(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

/// Replays an agent's edits across a run.
#[component]
pub fn ChangeReplay(
    author: ReplayAuthor,
    #[props(default)] context_text: Option<String>,
    #[props(default)] change_events: Vec<ChangeEvent>,
    #[props(default)] shown_until_offset: Option<f64>,
    #[props(default)] in_progress: bool,
    #[props(default)] axis_length: Option<f64>,
    #[props(default = 7)] max_visible_files: usize,
    #[props(default, into)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-replay-{}", next_id()));
    let mut open = use_signal(|| Option::<usize>::None);
    let mut hover_file = use_signal(|| Option::<String>::None);

    let axis = axis_length.unwrap_or_else(|| default_axis(&change_events));
    let scale = scale_max(&change_events);
    let visible = shown_events(&change_events, shown_until_offset);
    let visible_indexes: Vec<usize> = visible.iter().map(|(i, _)| *i).collect();
    let current_totals = totals(&change_events, shown_until_offset);
    let rows = file_rows(&change_events, shown_until_offset);
    let open_index = open().filter(|i| visible_indexes.contains(i));
    let open_event = open_index.and_then(|i| change_events.get(i).cloned());
    let hovered = hover_file();
    let marks: Vec<f64> = hovered
        .as_deref()
        .map(|path| file_marks(&change_events, shown_until_offset, path))
        .unwrap_or_default();
    let (added_pts, removed_pts) =
        series_points(&change_events, shown_until_offset, axis, 100.0, 40.0, scale);
    let to_polyline = |pts: &[(f64, f64)]| {
        pts.iter()
            .map(|(x, y)| format!("{x:.2},{y:.2}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let added_line = to_polyline(&added_pts);
    let removed_line = to_polyline(&removed_pts);
    let last_file = visible.last().map(|(_, e)| e.file_path.clone());
    let status = if in_progress {
        match &last_file {
            Some(path) => format!("Editing {path}"),
            None => "Waiting for the first edit".to_string(),
        }
    } else {
        format!(
            "{} files · +{} −{}",
            current_totals.files,
            compact(current_totals.added),
            compact(current_totals.removed)
        )
    };
    let net_text = format!(
        "{}{}",
        if current_totals.net >= 0 { "+" } else { "−" },
        compact(current_totals.net.unsigned_abs())
    );
    let added_text = format!("+{}", compact(current_totals.added));
    let removed_text = format!("−{}", compact(current_totals.removed));
    let files_text = current_totals.files.to_string();
    let axis_end = clock(axis);
    let announce = match &open_event {
        Some(event) => format!(
            "{} at {}: +{} −{}. {}",
            event.file_path,
            clock(event.at_offset),
            event.lines_added,
            event.lines_removed,
            event.line_details.join("; ")
        ),
        None => String::new(),
    };
    let mark_slot = format!("is-mark-{}", mark_index(&author.author_id));
    let initial = author.author_name.chars().take(1).collect::<String>();
    let author_name = author.author_name.clone();
    let context = context_text.clone().unwrap_or_default();
    let root_cls = class_list("kit-replay", &root_class);
    let rows_visible = rows.len();
    let overflow = rows_visible > max_visible_files;
    let list_class = if in_progress {
        "kit-replay__files is-live"
    } else {
        "kit-replay__files"
    };
    let list_max = format!("max-height: {}rem;", max_visible_files as f64 * 2.75);
    let chart_label = format!("Edits over {axis_end}");
    let rows_for_keys = visible_indexes.clone();
    let chart_id = format!("{base}-chart");
    let event_count = change_events.len();
    let empty = event_count == 0;

    let key_handler = {
        let keys = rows_for_keys.clone();
        let chart = chart_id.clone();
        move |event: KeyboardEvent| {
            let key = match event.key() {
                Key::ArrowRight => ReplayKey::Right,
                Key::ArrowLeft => ReplayKey::Left,
                Key::Home => ReplayKey::Home,
                Key::End => ReplayKey::End,
                Key::Escape => {
                    event.prevent_default();
                    open.set(None);
                    focus_id(&chart);
                    return;
                }
                _ => return,
            };
            event.prevent_default();
            let current = open().and_then(|i| keys.iter().position(|k| *k == i));
            if let Some(next) = step_index(keys.len(), current, key) {
                open.set(Some(keys[next]));
            }
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: root_cls,
            header { class: "kit-replay__head",
                span { class: "kit-replay__mark {mark_slot}", "aria-hidden": "true",
                    "{initial}"
                    if in_progress {
                        span { class: "kit-replay__pulse" }
                    }
                }
                div { class: "kit-replay__who",
                    span { class: "kit-replay__name", "{author_name}" }
                    if !context.is_empty() {
                        span { class: "kit-replay__context", "{context}" }
                    }
                }
            }
            p { class: "kit-replay__status", role: "status", "aria-live": "off", "{status}" }
            dl { class: "kit-replay__totals",
                div { dt { "Added" } dd { class: "kit-replay__add", "{added_text}" } }
                div { dt { "Removed" } dd { class: "kit-replay__remove", "{removed_text}" } }
                div { dt { "Net" } dd { "{net_text}" } }
                div { dt { "Files" } dd { "{files_text}" } }
            }
            if empty {
                p { class: "kit-replay__empty", "No edits yet" }
            }
            div {
                id: "{chart_id}",
                class: "kit-replay__chart",
                role: "group",
                tabindex: "0",
                "aria-label": "{chart_label}",
                onkeydown: key_handler,
                svg { class: "kit-replay__svg", view_box: "0 0 100 40", "preserveAspectRatio": "none", "aria-hidden": "true",
                    line { class: "kit-replay__baseline", x1: "0", y1: "40", x2: "100", y2: "40" }
                    for offset in marks.iter().copied() {
                        {
                            let x = (offset / axis * 100.0).min(100.0);
                            rsx! { line { class: "kit-replay__mark-line", x1: "{x:.2}", y1: "0", x2: "{x:.2}", y2: "40" } }
                        }
                    }
                    polyline { class: "kit-replay__series is-added", points: "{added_line}", fill: "none" }
                    polyline { class: "kit-replay__series is-removed", points: "{removed_line}", fill: "none" }
                    for (position, (index, event)) in visible.iter().enumerate() {
                        {
                            let x = (sanitize(event.at_offset) / axis * 100.0).min(100.0);
                            let index = *index;
                            let selected = if open_index == Some(index) { "true" } else { "false" };
                            let label = format!("{} at {}", event.file_path, clock(event.at_offset));
                            let key = format!("{base}-ev-{position}");
                            rsx! {
                                rect {
                                    key: "{key}",
                                    class: "kit-replay__hit",
                                    x: "{x:.2}",
                                    y: "0",
                                    width: "2",
                                    height: "40",
                                    "data-selected": "{selected}",
                                    "aria-label": "{label}",
                                    onclick: move |_| {
                                        if open() == Some(index) {
                                            open.set(None);
                                        } else {
                                            open.set(Some(index));
                                        }
                                    },
                                    onmouseenter: move |_| open.set(Some(index)),
                                }
                            }
                        }
                    }
                }
                div { class: "kit-replay__axis",
                    span { "0:00" }
                    span { "{axis_end}" }
                }
            }
            if overflow {
                p { class: "kit-replay__overflow", "{rows_visible} files" }
            }
            ol { class: "{list_class}", style: "{list_max}",
                for row in rows {
                    {
                        let (folder, name) = split_path(&row.path);
                        let width = if current_totals.added == 0 {
                            0.0
                        } else {
                            ((row.added + row.removed) as f64 / (current_totals.added + current_totals.removed).max(1) as f64 * 100.0).min(100.0)
                        };
                        let bar = format!("width: {width:.1}%;");
                        let first = clock(row.first_offset);
                        let path = row.path.clone();
                        let enter_path = path.clone();
                        let row_added = format!("+{}", compact(row.added));
                        let row_removed = format!("−{}", compact(row.removed));
                        rsx! {
                            li {
                                key: "{path}",
                                class: "kit-replay__file",
                                onmouseenter: move |_| hover_file.set(Some(enter_path.clone())),
                                onmouseleave: move |_| hover_file.set(None),
                                span { class: "kit-replay__path",
                                    span { class: "kit-replay__folder", "{folder}" }
                                    span { class: "kit-replay__name-file", "{name}" }
                                }
                                span { class: "kit-replay__bar", style: "{bar}", "aria-hidden": "true" }
                                span { class: "kit-replay__counts",
                                    span { class: "kit-replay__add", "{row_added}" }
                                    span { class: "kit-replay__remove", "{row_removed}" }
                                }
                                time { class: "kit-replay__first", "{first}" }
                            }
                        }
                    }
                }
            }
            if let Some(event) = open_event {
                div { class: "kit-replay__detail",
                    p { class: "kit-replay__detail-head",
                        span { "{event.file_path}" }
                        span { class: "kit-replay__first", "{clock(event.at_offset)}" }
                    }
                    ul { class: "kit-replay__lines",
                        for line in event.line_details.iter().cloned() {
                            li { "{line}" }
                        }
                    }
                }
            }
            div { class: "kit-replay__sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
