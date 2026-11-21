//! Workspace territory map: a codebase as a treemap of folders and files, with the agents working in it placed on their files.
//!
//! Design choice: the treemap is `agent_context_map::treemap`, fed files sorted by path so the same file set always
//! lays out the same way, whatever order the input arrives in.
//! Design choice: marks are text glyphs (`agent_mark`), not nodes.
//! Design choice: `folder_depth` groups by the first N path segments.
//! Browser checks: mark glide between tiles, fill sheen, conflict pulse, file card placement on narrow widths.

use dioxus::prelude::*;

use crate::components::agent_context_map::{Rect, focus_on_mount, treemap};
use crate::internal::*;

const CSS: Asset = asset!("/assets/workspace-territory-map.css");

/// Design-unit canvas for the treemap; tile positions are percentages of it.
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
    pub agent_action: AgentAction,
    pub agent_task: Option<String>,
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

/// Tile rectangles for the files, area proportional to line count, in path order.
/// Tile an arrow key moves to: the nearest tile lying in that direction, by centre distance with
/// sideways offset counted double so a straight neighbour beats a diagonal one.
pub fn neighbour_tile(rects: &[Rect], from: usize, key: &Key) -> Option<usize> {
    let here = rects.get(from)?;
    let (hx, hy) = (here.x + here.w / 2.0, here.y + here.h / 2.0);
    rects
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != from)
        .filter_map(|(i, r)| {
            let (dx, dy) = (r.x + r.w / 2.0 - hx, r.y + r.h / 2.0 - hy);
            let (along, across) = match key {
                Key::ArrowRight => (dx, dy),
                Key::ArrowLeft => (-dx, dy),
                Key::ArrowDown => (dy, dx),
                Key::ArrowUp => (-dy, dx),
                _ => return None,
            };
            (along > 0.0).then_some((i, along + 2.0 * across.abs()))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

pub fn tile_rects(files: &[RepoFile]) -> Vec<Rect> {
    let weights: Vec<f64> = files.iter().map(|f| f.line_count as f64).collect();
    treemap(&weights, 0.0, 0.0, MAP_W, MAP_H)
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

/// Files touched by an agent (in its change log), for the agent bar count.
pub fn touched_count(agent_id: &str, files: &[RepoFile]) -> usize {
    files
        .iter()
        .filter(|f| f.change_log.iter().any(|c| c.agent_id == agent_id))
        .count()
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

/// Repository map: files grouped by folder and sized by lines, with each agent's mark on the file it is in.
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

    let files = normalised_files(&file_list);
    let rects = tile_rects(&files);
    let changed = files.iter().filter(|f| !f.change_log.is_empty()).count();
    let header = match &branch_name {
        Some(b) => format!("{repo_name} · {b}"),
        None => repo_name.clone(),
    };
    let counts = format!("{changed} of {} files changed", files.len());
    let class = class_list("kit-wtm", &extra_class);
    let focus = agent_focus();
    let open = selected().and_then(|i| files.get(i).cloned());

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", "aria-labelledby": "{base}-title",
            header { class: "kit-wtm__head",
                h2 { id: "{base}-title", class: "kit-wtm__title", "{header}" }
                p { class: "kit-wtm__counts", role: "status", "aria-live": "polite", "{counts}" }
            }
            div { class: "kit-wtm__agents", role: "group", "aria-label": "Agents",
                for a in agent_list.iter() {
                    {
                        let pressed = focus.as_deref() == Some(a.agent_id.as_str());
                        let touched = touched_count(&a.agent_id, &files);
                        let label = format!("{}, {touched} files touched", a.agent_name);
                        let mark = a.agent_mark.clone().unwrap_or_else(|| a.agent_name.chars().take(1).collect());
                        let id = a.agent_id.clone();
                        rsx! {
                            button { key: "{a.agent_id}", r#type: "button", class: "kit-wtm__agent", "aria-pressed": "{pressed}", "aria-label": "{label}",
                                onclick: move |_| agent_focus.set(if agent_focus() == Some(id.clone()) { None } else { Some(id.clone()) }),
                                span { class: "kit-wtm__mark", "aria-hidden": "true", "{mark}" }
                                span { "{a.agent_name}" }
                                span { class: "kit-wtm__meta", "{touched}" }
                            }
                        }
                    }
                }
            }
            div { class: "kit-wtm__map", role: "group", "aria-label": "Files", style: "aspect-ratio: {MAP_W} / {MAP_H};",
                for (i, file) in files.iter().enumerate() {
                    {
                        let rect = rects[i];
                        let style = format!(
                            "left: {:.2}%; top: {:.2}%; width: {:.2}%; height: {:.2}%;",
                            rect.x / MAP_W * 100.0, rect.y / MAP_H * 100.0, rect.w / MAP_W * 100.0, rect.h / MAP_H * 100.0
                        );
                        let folder = folder_of(&file.file_path, folder_depth);
                        let name = file.file_path.rsplit('/').next().unwrap_or("").to_string();
                        let label = tile_label(file, &agent_list);
                        let conflict = is_conflict(file, &agent_list);
                        let editor = last_editor(file);
                        let read_only = editing_agents(file, &agent_list).is_empty()
                            && agent_list.iter().any(|a| a.agent_action == AgentAction::Read && a.current_file.as_deref() == Some(file.file_path.as_str()));
                        let dimmed = focus.as_ref().is_some_and(|f| !file.change_log.iter().any(|c| c.agent_id == *f) && !agent_list.iter().any(|a| a.agent_id == *f && a.current_file.as_deref() == Some(file.file_path.as_str())));
                        let here: Vec<String> = agent_list
                            .iter()
                            .filter(|a| a.current_file.as_deref() == Some(file.file_path.as_str()))
                            .map(|a| a.agent_mark.clone().unwrap_or_else(|| a.agent_name.chars().take(1).collect()))
                            .collect();
                        let open_here = selected() == Some(i);
                        let lines = format!("{} lines", file.line_count);
                        let tile_id = format!("{base}-tile-{i}");
                        let nav_rects = rects.clone();
                        let nav_base = base.clone();
                        rsx! {
                            button {
                                id: "{tile_id}", key: "{file.file_path}", r#type: "button", class: "kit-wtm__tile", style: "{style}",
                                "aria-label": "{label}", "aria-expanded": "{open_here}",
                                "data-editor": editor.clone(), "data-read": read_only.then_some("true"),
                                "data-conflict": conflict.then_some("true"), "data-dim": dimmed.then_some("true"),
                                onclick: move |_| selected.set(if selected() == Some(i) { None } else { Some(i) }),
                                onkeydown: move |event: KeyboardEvent| {
                                    let key = event.key();
                                    if key == Key::Escape {
                                        selected.set(None);
                                        return;
                                    }
                                    if let Some(next) = neighbour_tile(&nav_rects, i, &key) {
                                        event.prevent_default();
                                        let target = format!("{nav_base}-tile-{next}");
                                        document::eval(&format!("document.getElementById({target:?})?.focus();"));
                                    }
                                },
                                span { class: "kit-wtm__folder", "{folder}" }
                                span { class: "kit-wtm__name", "{name}" }
                                span { class: "kit-wtm__lines", "{lines}" }
                                if conflict { span { class: "kit-wtm__tag", "Conflict" } }
                                if read_only { span { class: "kit-wtm__tag", "Read only" } }
                                span { class: "kit-wtm__marks", "aria-hidden": "true",
                                    for m in here.iter() { span { class: "kit-wtm__mark", "{m}" } }
                                }
                            }
                        }
                    }
                }
            }
            if let (Some(open_i), Some(file)) = (selected(), open) {
                {
                    let title = file.file_path.clone();
                    let lines = file.line_count;
                    let close_to = format!("{base}-tile-{open_i}");
                    rsx! {
                        aside { class: "kit-wtm__card", "aria-label": "{title}", tabindex: "-1",
                            onmounted: focus_on_mount,
                            onkeydown: move |event: KeyboardEvent| {
                                if event.key() == Key::Escape {
                                    event.prevent_default();
                                    selected.set(None);
                                    document::eval(&format!("document.getElementById({close_to:?})?.focus();"));
                                }
                            },
                            h3 { class: "kit-wtm__card-title", "{title}" }
                            p { class: "kit-wtm__meta", "{lines} lines" }
                            ol { class: "kit-wtm__history", "aria-label": "Change history",
                                for c in file.change_log.iter() {
                                    {
                                        let agent = agent_list.iter().find(|a| a.agent_id == c.agent_id).map(|a| a.agent_name.clone()).unwrap_or_else(|| c.agent_id.clone());
                                        let added = c.lines_added.map(|n| format!("+{n}")).unwrap_or_default();
                                        let removed = c.lines_removed.map(|n| format!("-{n}")).unwrap_or_default();
                                        rsx! { li { key: "{c.agent_id}-{added}-{removed}", "{agent} {added} {removed}" } }
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
