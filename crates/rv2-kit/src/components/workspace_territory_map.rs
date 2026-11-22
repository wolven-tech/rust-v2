//! Workspace territory map: a repository as folder groups of file tiles, with the agents working in it marked on their files.
//!
//! Design choice: each folder is a treemap rectangle sized by its line total, and its files tile that rectangle by line
//! count, so every file's area is its share of the whole map and the tiles leave no gap.
//! Design choice: files are laid out in map order (folder, then path), so the same file set always lays out the same way
//! and arrow keys walk the map in that order.
//! Design choice: marks are text glyphs (`agent_mark`), not nodes.
//! Browser checks: fill rising from the bottom on edit, conflict pulse, status line on agent move, file card on narrow widths.

use dioxus::prelude::*;

use crate::components::agent_context_map::{Rect, focus_on_mount, treemap};
use crate::internal::*;

const CSS: Asset = asset!("/assets/workspace-territory-map.css");

/// Design-unit canvas for the treemap; positions are percentages of it.
pub const MAP_W: f64 = 1000.0;
pub const MAP_H: f64 = 560.0;

#[derive(Clone, PartialEq, Debug)]
pub struct Change {
    pub agent_id: String,
    pub lines_added: Option<u32>,
    pub lines_removed: Option<u32>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct RepoFile {
    pub file_path: String,
    pub line_count: u32,
    pub change_log: Vec<Change>,
}

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum AgentAction {
    Edit,
    Read,
}

#[derive(Clone, PartialEq, Debug)]
pub struct RepoAgent {
    pub agent_id: String,
    pub agent_name: String,
    pub agent_mark: Option<String>,
    pub current_file: Option<String>,
    /// A file the agent reads while it is at `current_file`, so one agent can edit and read at once.
    pub read_file: Option<String>,
    pub agent_action: AgentAction,
    pub agent_task: Option<String>,
}

/// One folder group in map order: its path, the index of its first file and its file count.
#[derive(Clone, PartialEq, Debug)]
pub struct FolderRun {
    pub folder: String,
    pub start: usize,
    pub len: usize,
}

/// The file card's content for one file.
#[derive(Clone, PartialEq, Debug)]
pub struct FileCard {
    pub path: String,
    pub lines: u32,
    pub agents: Vec<String>,
    pub history: Vec<String>,
}

/// Folder part of a path, limited to `depth` segments. A file at the top level groups under "".
pub fn folder_of(path: &str, depth: usize) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= 1 {
        return String::new();
    }
    parts[..(parts.len() - 1).min(depth)].join("/")
}

/// Files with duplicate paths collapsed to the first occurrence, sorted by path.
pub fn normalised_files(files: &[RepoFile]) -> Vec<RepoFile> {
    let mut out: Vec<RepoFile> = Vec::new();
    for f in files {
        if !out.iter().any(|o| o.file_path == f.file_path) {
            out.push(f.clone());
        }
    }
    out.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    out
}

/// Files in map order: by folder, then by path. Arrow keys walk this order.
pub fn map_order(files: &[RepoFile], depth: usize) -> Vec<RepoFile> {
    let mut out = normalised_files(files);
    out.sort_by_cached_key(|f| (folder_of(&f.file_path, depth), f.file_path.clone()));
    out
}

/// Folder groups over files already in map order.
pub fn folder_runs(ordered: &[RepoFile], depth: usize) -> Vec<FolderRun> {
    let mut runs: Vec<FolderRun> = Vec::new();
    for (i, f) in ordered.iter().enumerate() {
        let folder = folder_of(&f.file_path, depth);
        match runs.last_mut() {
            Some(run) if run.folder == folder => run.len += 1,
            _ => runs.push(FolderRun {
                folder,
                start: i,
                len: 1,
            }),
        }
    }
    runs
}

/// Folder rectangles (one per run) and file tiles (one per file in map order). Folders split the map by line total;
/// files split their folder by line count, so a file's area is its share of the whole map.
pub fn layout(ordered: &[RepoFile], runs: &[FolderRun]) -> (Vec<Rect>, Vec<Rect>) {
    let lines = |run: &FolderRun| -> Vec<f64> {
        ordered[run.start..run.start + run.len]
            .iter()
            .map(|f| f.line_count as f64)
            .collect()
    };
    let totals: Vec<f64> = runs.iter().map(|r| lines(r).iter().sum::<f64>()).collect();
    let folders = treemap(&totals, 0.0, 0.0, MAP_W, MAP_H);
    let mut tiles = Vec::with_capacity(ordered.len());
    for (run, b) in runs.iter().zip(&folders) {
        tiles.extend(treemap(&lines(run), b.x, b.y, b.w, b.h));
    }
    (folders, tiles)
}

/// Tile rectangles in map order, area proportional to line count.
pub fn tile_rects(files: &[RepoFile], depth: usize) -> Vec<Rect> {
    let ordered = map_order(files, depth);
    layout(&ordered, &folder_runs(&ordered, depth)).1
}

/// Map index an arrow, Home or End key moves to. Arrows walk map order, so every file of a folder comes before the next folder's.
pub fn step_index(len: usize, from: usize, key: &Key) -> Option<usize> {
    match key {
        Key::ArrowRight | Key::ArrowDown => (from + 1 < len).then_some(from + 1),
        Key::ArrowLeft | Key::ArrowUp => from.checked_sub(1),
        Key::Home => (len > 0).then_some(0),
        Key::End => len.checked_sub(1),
        _ => None,
    }
}

/// Escape hides an open file card and keeps the agent focus; with no card open it clears the focus.
pub fn after_escape(open: Option<usize>, focus: Option<String>) -> (Option<usize>, Option<String>) {
    match open {
        Some(_) => (None, focus),
        None => (None, None),
    }
}

/// The agent whose change was most recent on a file, if any changed it.
pub fn last_editor(file: &RepoFile) -> Option<String> {
    file.change_log.last().map(|c| c.agent_id.clone())
}

/// Agents editing a file right now, in agent order. Two or more is a conflict.
pub fn editing_agents(file: &RepoFile, agents: &[RepoAgent]) -> Vec<String> {
    agents
        .iter()
        .filter(|a| {
            a.agent_action == AgentAction::Edit
                && a.current_file.as_deref() == Some(file.file_path.as_str())
        })
        .map(|a| a.agent_id.clone())
        .collect()
}

pub fn is_conflict(file: &RepoFile, agents: &[RepoAgent]) -> bool {
    editing_agents(file, agents).len() >= 2
}

/// Whether the agent reads this file, either as its read file or as a read-only current location.
pub fn is_reading(agent: &RepoAgent, path: &str) -> bool {
    agent.read_file.as_deref() == Some(path)
        || (agent.agent_action == AgentAction::Read && agent.current_file.as_deref() == Some(path))
}

/// Whether the agent is at this file, editing or reading it.
pub fn is_at(agent: &RepoAgent, path: &str) -> bool {
    agent.current_file.as_deref() == Some(path) || agent.read_file.as_deref() == Some(path)
}

/// Files the agent has a logged change on, plus the file it reads. The count is for the agent bar.
pub fn touched_count(agent: &RepoAgent, files: &[RepoFile]) -> usize {
    files
        .iter()
        .filter(|f| {
            f.change_log.iter().any(|c| c.agent_id == agent.agent_id)
                || is_reading(agent, &f.file_path)
        })
        .count()
}

fn sum_reported(file: &RepoFile, pick: impl Fn(&Change) -> Option<u32>) -> Option<u32> {
    file.change_log.iter().filter_map(pick).reduce(|a, b| a + b)
}

/// Added and removed lines over a file's logged changes, e.g. "+18 −6". A side no change reported is left out.
pub fn delta_text(file: &RepoFile) -> Option<String> {
    let added = sum_reported(file, |c| c.lines_added).map(|n| format!("+{n}"));
    let removed = sum_reported(file, |c| c.lines_removed).map(|n| format!("\u{2212}{n}"));
    match (added, removed) {
        (None, None) => None,
        (added, removed) => Some(
            [added, removed]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" "),
        ),
    }
}

/// Spelled-out accessible name for a tile.
pub fn tile_label(file: &RepoFile, agents: &[RepoAgent]) -> String {
    let editors = editing_agents(file, agents).len();
    let conflict = if editors >= 2 {
        ", conflict, two agents editing"
    } else {
        ""
    };
    format!("{}, {} lines{conflict}", file.file_path, file.line_count)
}

/// Status line: the most recently listed agent that is editing a file.
pub fn status_line(agents: &[RepoAgent]) -> Option<String> {
    agents.iter().rev().find_map(|a| {
        let file = a.current_file.as_deref()?;
        (a.agent_action == AgentAction::Edit).then(|| format!("{} is editing {file}", a.agent_name))
    })
}

pub fn file_card(file: &RepoFile, agents: &[RepoAgent]) -> FileCard {
    FileCard {
        path: file.file_path.clone(),
        lines: file.line_count,
        agents: agents
            .iter()
            .filter(|a| is_at(a, &file.file_path))
            .map(|a| a.agent_name.clone())
            .collect(),
        history: file
            .change_log
            .iter()
            .map(|c| {
                let name = agents
                    .iter()
                    .find(|a| a.agent_id == c.agent_id)
                    .map_or_else(|| c.agent_id.clone(), |a| a.agent_name.clone());
                let added = c.lines_added.map(|n| format!(" +{n}")).unwrap_or_default();
                let removed = c
                    .lines_removed
                    .map(|n| format!(" -{n}"))
                    .unwrap_or_default();
                format!("{name}{added}{removed}")
            })
            .collect(),
    }
}

fn files_noun(count: usize) -> &'static str {
    if count == 1 { "file" } else { "files" }
}

fn agent_index(agents: &[RepoAgent], id: &str) -> Option<usize> {
    agents.iter().position(|a| a.agent_id == id)
}

/// Identity colour for the agent at `index`; the six identity tokens cycle.
fn identity(index: usize) -> String {
    format!("var(--kit-id-{})", index % 6 + 1)
}

fn mark_of(agent: &RepoAgent) -> String {
    agent
        .agent_mark
        .clone()
        .unwrap_or_else(|| agent.agent_name.chars().take(1).collect())
}

fn place(r: Rect) -> String {
    format!(
        "left: {:.2}%; top: {:.2}%; width: {:.2}%; height: {:.2}%;",
        r.x / MAP_W * 100.0,
        r.y / MAP_H * 100.0,
        r.w / MAP_W * 100.0,
        r.h / MAP_H * 100.0
    )
}

/// What both forms of a file show: the wide tile and the narrow row read the same facts.
struct FileView {
    id: String,
    row_id: String,
    name: String,
    label: String,
    lines: String,
    delta: Option<String>,
    changes: String,
    editor: Option<String>,
    /// `--wtm-id` declaration for the last editor's identity, if anyone changed the file.
    tint: Option<String>,
    conflict: bool,
    read_only: bool,
    dimmed: bool,
    marks: Vec<(String, String)>,
}

fn file_view(
    base: &str,
    index: usize,
    file: &RepoFile,
    agents: &[RepoAgent],
    focus: Option<&str>,
) -> FileView {
    let editor = last_editor(file);
    let tint = editor
        .as_deref()
        .and_then(|id| agent_index(agents, id))
        .map(|n| format!("--wtm-id: {};", identity(n)));
    let changes = file.change_log.len();
    FileView {
        id: format!("{base}-tile-{index}"),
        row_id: format!("{base}-row-{index}"),
        name: file.file_path.rsplit('/').next().unwrap_or("").to_string(),
        label: tile_label(file, agents),
        lines: format!("{} lines", file.line_count),
        delta: delta_text(file),
        changes: format!(
            "{changes} {}",
            if changes == 1 { "change" } else { "changes" }
        ),
        editor,
        tint,
        conflict: is_conflict(file, agents),
        read_only: editing_agents(file, agents).is_empty()
            && agents.iter().any(|a| is_reading(a, &file.file_path)),
        dimmed: focus.is_some_and(|f| {
            !agents
                .iter()
                .any(|a| a.agent_id == f && is_at(a, &file.file_path))
        }),
        marks: agents
            .iter()
            .enumerate()
            .filter(|(_, a)| is_at(a, &file.file_path))
            .map(|(n, a)| (mark_of(a), identity(n)))
            .collect(),
    }
}

/// Repository map: folder groups of file tiles sized by lines, with each agent's mark on the file it is in.
/// Below the map's container breakpoint the same files render as folder rows (see `.kit-wtm__rows`).
#[component]
pub fn WorkspaceTerritoryMap(
    file_list: Vec<RepoFile>,
    agent_list: Vec<RepoAgent>,
    repo_name: String,
    #[props(default)] branch_name: Option<String>,
    #[props(default = 2)] folder_depth: usize,
    #[props(default)] focused_agent_id: Option<String>,
    #[props(default)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-wtm-{}", next_id()));
    let mut selected = use_signal(|| None::<usize>);
    let mut agent_focus = use_signal(|| focused_agent_id.clone());
    let mut cursor = use_signal(|| 0usize);

    let files = map_order(&file_list, folder_depth);
    let runs = folder_runs(&files, folder_depth);
    let (folder_rects, rects) = layout(&files, &runs);
    let total = files.len();
    let changed = files.iter().filter(|f| !f.change_log.is_empty()).count();
    let header = match &branch_name {
        Some(b) => format!("{repo_name} · {b}"),
        None => repo_name.clone(),
    };
    let counts = format!("{changed} of {total} files changed");
    let status = status_line(&agent_list).unwrap_or_default();
    let class = class_list("kit-wtm", &extra_class);
    let focus = agent_focus();
    let open = selected().and_then(|i| files.get(i).cloned());
    let stop = cursor().min(total.saturating_sub(1));
    let views: Vec<FileView> = files
        .iter()
        .enumerate()
        .map(|(i, file)| file_view(&base, i, file, &agent_list, focus.as_deref()))
        .collect();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", "aria-labelledby": "{base}-title",
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape {
                    let (open, focus) = after_escape(selected(), agent_focus());
                    selected.set(open);
                    agent_focus.set(focus);
                }
            },
            header { class: "kit-wtm__head",
                h2 { id: "{base}-title", class: "kit-wtm__title", "{header}" }
                p { class: "kit-wtm__counts", "{counts}" }
            }
            div { class: "kit-wtm__agents", role: "group", "aria-label": "Agents",
                for (n, a) in agent_list.iter().enumerate() {
                    {
                        let pressed = focus.as_deref() == Some(a.agent_id.as_str());
                        let touched = touched_count(a, &files);
                        let label = format!("{}, {touched} {} touched", a.agent_name, files_noun(touched));
                        let mark = mark_of(a);
                        let tint = identity(n);
                        let files_text = format!("{touched} {}", files_noun(touched));
                        let id = a.agent_id.clone();
                        rsx! {
                            button { key: "{a.agent_id}", r#type: "button", class: "kit-wtm__agent", style: "--wtm-id: {tint};", "aria-pressed": "{pressed}", "aria-label": "{label}",
                                onclick: move |_| agent_focus.set(if agent_focus() == Some(id.clone()) { None } else { Some(id.clone()) }),
                                span { class: "kit-wtm__mark", "aria-hidden": "true", "{mark}" }
                                span { class: "kit-wtm__agent-name", "{a.agent_name}" }
                                span { class: "kit-wtm__meta", "{files_text}" }
                            }
                        }
                    }
                }
            }
            div { class: "kit-wtm__map", role: "group", "aria-label": "Files", style: "aspect-ratio: {MAP_W} / {MAP_H};",
                for (run, rect) in runs.iter().zip(folder_rects.iter()) {
                    {
                        let label = if run.folder.is_empty() { "root".to_string() } else { run.folder.clone() };
                        let group = format!("{label} folder, {} files", run.len);
                        let style = place(*rect);
                        rsx! {
                            div { key: "{label}", class: "kit-wtm__group", role: "group", "aria-label": "{group}", style: "{style}",
                                span { class: "kit-wtm__group-label", "aria-hidden": "true", "{label}" }
                            }
                        }
                    }
                }
                for (i, view) in views.iter().enumerate() {
                    {
                        let rect = rects[i];
                        let style = match &view.tint {
                            Some(tint) => format!("{} {tint}", place(rect)),
                            None => place(rect),
                        };
                        let open_here = selected() == Some(i);
                        let tab = if i == stop { "0" } else { "-1" };
                        let nav_base = base.clone();
                        rsx! {
                            button {
                                id: "{view.id}", key: "{view.id}", r#type: "button", class: "kit-wtm__tile", style: "{style}",
                                tabindex: "{tab}", "aria-label": "{view.label}", "aria-expanded": "{open_here}",
                                "data-editor": view.editor.clone(), "data-read": view.read_only.then_some("true"),
                                "data-conflict": view.conflict.then_some("true"), "data-dim": view.dimmed.then_some("true"),
                                onclick: move |_| selected.set(if selected() == Some(i) { None } else { Some(i) }),
                                onfocus: move |_| cursor.set(i),
                                onkeydown: move |event: KeyboardEvent| {
                                    if let Some(next) = step_index(total, i, &event.key()) {
                                        event.prevent_default();
                                        cursor.set(next);
                                        let target = format!("{nav_base}-tile-{next}");
                                        document::eval(&format!("document.getElementById({target:?})?.focus();"));
                                    }
                                },
                                span { class: "kit-wtm__name", "{view.name}" }
                                div { class: "kit-wtm__sizes",
                                    span { class: "kit-wtm__lines", "{view.lines}" }
                                    if let Some(text) = view.delta.clone() { span { class: "kit-wtm__delta", "{text}" } }
                                }
                                div { class: "kit-wtm__foot",
                                    if view.conflict { span { class: "kit-wtm__tag", "Conflict" } }
                                    if view.read_only { span { class: "kit-wtm__tag", "Read only" } }
                                    span { class: "kit-wtm__marks", "aria-hidden": "true",
                                        for (m, tint) in view.marks.iter() {
                                            span { class: "kit-wtm__mark", style: "--wtm-id: {tint};", "{m}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "kit-wtm__rows", role: "group", "aria-label": "Files by folder",
                for run in runs.iter() {
                    {
                        let label = if run.folder.is_empty() { "root".to_string() } else { run.folder.clone() };
                        let group = format!("{label} folder, {} files", run.len);
                        let folder_views = &views[run.start..run.start + run.len];
                        rsx! {
                            div { key: "{label}", class: "kit-wtm__row-group", role: "group", "aria-label": "{group}",
                                span { class: "kit-wtm__row-head", "aria-hidden": "true", "{label}" }
                                for (k, view) in folder_views.iter().enumerate() {
                                    {
                                        let i = run.start + k;
                                        let row_style = view.tint.clone().unwrap_or_default();
                                        let open_here = selected() == Some(i);
                                        let tab = if i == stop { "0" } else { "-1" };
                                        let nav_base = base.clone();
                                        rsx! {
                                            button {
                                                id: "{view.row_id}", key: "{view.row_id}", r#type: "button", class: "kit-wtm__row", style: "{row_style}",
                                                tabindex: "{tab}", "aria-label": "{view.label}", "aria-expanded": "{open_here}",
                                                "data-editor": view.editor.clone(), "data-read": view.read_only.then_some("true"),
                                                "data-conflict": view.conflict.then_some("true"), "data-dim": view.dimmed.then_some("true"),
                                                onclick: move |_| selected.set(if selected() == Some(i) { None } else { Some(i) }),
                                                onfocus: move |_| cursor.set(i),
                                                onkeydown: move |event: KeyboardEvent| {
                                                    if let Some(next) = step_index(total, i, &event.key()) {
                                                        event.prevent_default();
                                                        cursor.set(next);
                                                        let target = format!("{nav_base}-row-{next}");
                                                        document::eval(&format!("document.getElementById({target:?})?.focus();"));
                                                    }
                                                },
                                                span { class: "kit-wtm__row-name", "{view.name}" }
                                                span { class: "kit-wtm__row-meta",
                                                    span { "{view.lines}" }
                                                    if let Some(text) = view.delta.clone() { span { "{text}" } }
                                                    span { "{view.changes}" }
                                                    if view.conflict { span { class: "kit-wtm__tag", "Conflict" } }
                                                    if view.read_only { span { class: "kit-wtm__tag", "Read only" } }
                                                    span { class: "kit-wtm__marks", "aria-hidden": "true",
                                                        for (m, tint) in view.marks.iter() {
                                                            span { class: "kit-wtm__mark", style: "--wtm-id: {tint};", "{m}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "kit-wtm__status", role: "status", "aria-live": "polite", "{status}" }
            if let (Some(open_i), Some(file)) = (selected(), open) {
                {
                    let card = file_card(&file, &agent_list);
                    let close_to = format!("{base}-tile-{open_i}");
                    let present = if card.agents.is_empty() { "No agent here".to_string() } else { card.agents.join(", ") };
                    let lines = format!("{} lines", card.lines);
                    let title = card.path.clone();
                    rsx! {
                        aside { class: "kit-wtm__card", "aria-label": "{title}", tabindex: "-1",
                            onmounted: focus_on_mount,
                            onkeydown: move |event: KeyboardEvent| {
                                if event.key() == Key::Escape {
                                    event.prevent_default();
                                    event.stop_propagation();
                                    selected.set(None);
                                    document::eval(&format!("document.getElementById({close_to:?})?.focus();"));
                                }
                            },
                            h3 { class: "kit-wtm__card-title", "{title}" }
                            p { class: "kit-wtm__meta", "{lines}" }
                            p { class: "kit-wtm__present", "Agents here: {present}" }
                            ol { class: "kit-wtm__history", "aria-label": "Change history",
                                for entry in card.history.iter() {
                                    li { "{entry}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
