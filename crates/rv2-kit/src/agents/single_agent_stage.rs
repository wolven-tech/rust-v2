//! Single agent stage: one assistant on one request, with who asked, what it is doing
//! now, the files it was handed, and the tools it may call with each tool's latest result.
//!
//! Design choice: one grid holds three bands. The requester, assistant and files sit in
//! the top band, the three-row work area below them, and the tool cards at the bottom.
//! DOM order is assistant, tools, files (the Tab order the spec gives); grid areas place them.
//! Design choice: the joins between requester, assistant and files are pseudo-elements in
//! the 12px column gap; the joins to tool cards are SVG curves measured from the rendered
//! boxes and sit behind the cards, so they show in the gaps between bands.
//! Design choice: marks are image URLs; a missing mark shows the name's initial.
//! Design choice: each entity keeps only its hue as a variable; the tint tokens compose the fill and ink.
//! Design choice: the info card is positioned under its trigger by the same script that
//! draws the joins, so it follows the trigger on resize.
//! Browser checks: travelling highlight and sheen; info card grow; the reduced-motion
//! static frames; Esc returns focus to the trigger; phone single column.

use dioxus::prelude::*;

use crate::agents::delegation_card_tree::accent_hue;
use crate::agents::single_run_status_card::initial_of;
use crate::internal::{KIT_CSS, focus_after_frame, next_id};

const CSS: Asset = asset!("/assets/single-agent-stage.css");

/// Sets each join path in the stage's SVG to a curve from the bottom centre of its
/// `data-from` card to the top centre of its `data-to` card, and places the info card
/// under its trigger. Runs again on resize and on any subtree change that can move a
/// card; the path's `d` is not observed. Positions are written only when they change,
/// because the observer watches `style`.
const JOIN_SCRIPT: &str = r#"
let svg = null;
for (let i = 0; i < 600 && !(svg = document.getElementById("__ID__")); i++)
    await new Promise((resolve) => requestAnimationFrame(resolve));
if (!svg) return false;
const root = svg.parentElement;
const layout = () => {
    if (svg.getClientRects().length === 0) return;
    const box = svg.getBoundingClientRect();
    for (const path of svg.querySelectorAll("path")) {
        const from = document.getElementById(path.dataset.from);
        const to = document.getElementById(path.dataset.to);
        if (!from || !to) {
            path.removeAttribute("d");
            continue;
        }
        const a = from.getBoundingClientRect();
        const b = to.getBoundingClientRect();
        const x1 = a.left + a.width / 2 - box.left;
        const y1 = a.bottom - box.top;
        const x2 = b.left + b.width / 2 - box.left;
        const y2 = b.top - box.top;
        const mid = (y1 + y2) / 2;
        path.setAttribute("d", `M${x1} ${y1} C${x1} ${mid} ${x2} ${mid} ${x2} ${y2}`);
    }
    const info = root.querySelector(".kit-sas__info");
    const trigger = info && document.getElementById(info.dataset.for);
    if (trigger) {
        const t = trigger.getBoundingClientRect();
        const r = root.getBoundingClientRect();
        const top = `${t.bottom - r.top - root.clientTop + 6}px`;
        const left = `${Math.max(0, Math.min(t.left - r.left - root.clientLeft, root.clientWidth - info.offsetWidth))}px`;
        if (info.style.top !== top) info.style.top = top;
        if (info.style.left !== left) info.style.left = left;
    }
};
let frame = 0;
const schedule = () => {
    if (!frame) frame = requestAnimationFrame(() => { frame = 0; layout(); });
};
const observers = (window.__kitSasJoins ||= new Map());
observers.get("__ID__")?.disconnect();
const mutations = new MutationObserver(schedule);
mutations.observe(root, {
    subtree: true,
    childList: true,
    attributes: true,
    attributeFilter: ["class", "style", "data-stage", "data-route", "aria-expanded", "data-live"],
});
const resizes = new ResizeObserver(schedule);
resizes.observe(root);
observers.set("__ID__", { disconnect() { mutations.disconnect(); resizes.disconnect(); } });
layout();
return true;
"#;

pub const MAX_TOOL_CARDS: usize = 4;
pub const MAX_FILE_ROWS: usize = 4;
pub const MAX_WORK_ROWS: usize = 3;

#[derive(Clone, PartialEq, Debug)]
pub struct DetailRow {
    pub field_label: String,
    pub field_value: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Requester {
    pub display_name: String,
    pub message_text: String,
    pub photo_url: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Worker {
    pub worker_key: String,
    pub display_name: String,
    pub subtitle_text: Option<String>,
    pub detail_rows: Vec<DetailRow>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerStage {
    Waiting,
    Working,
    Finished,
}

impl WorkerStage {
    fn as_str(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Working => "working",
            Self::Finished => "finished",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ToolGrant {
    pub tool_key: String,
    pub mark_source: Option<String>,
    pub display_name: String,
    pub grant_text: String,
    pub previous_action: Option<String>,
    pub previous_when: Option<String>,
    pub live_action: Option<String>,
    pub detail_rows: Vec<DetailRow>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FileItem {
    pub file_key: String,
    pub display_name: String,
    pub mark_source: Option<String>,
    pub meta_text: Option<String>,
    pub detail_rows: Vec<DetailRow>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct WorkStep {
    pub step_key: String,
    pub text: String,
    /// A tool key, `"files"`, or `"requester"`; picks the connector the step travels along.
    pub route_key: String,
    pub in_progress: bool,
}

/// The newest work steps, newest first.
pub fn newest_steps(work_log: &[WorkStep]) -> Vec<WorkStep> {
    work_log.iter().rev().take(MAX_WORK_ROWS).cloned().collect()
}

/// The work area's rows, newest first. Always `MAX_WORK_ROWS` long: unused rows are
/// `None`, so the band keeps its height whether or not steps are present.
pub fn work_slots(work_log: &[WorkStep]) -> Vec<Option<WorkStep>> {
    let mut slots: Vec<Option<WorkStep>> = newest_steps(work_log).into_iter().map(Some).collect();
    slots.resize(MAX_WORK_ROWS, None);
    slots
}

/// State word: Waiting, Done once finished, and Working while a step is in progress
/// (or before any step has been logged). A working worker whose steps are all done reads Done.
pub fn state_word(stage: WorkerStage, work_log: &[WorkStep]) -> &'static str {
    match stage {
        WorkerStage::Waiting => "Waiting",
        WorkerStage::Finished => "Done",
        WorkerStage::Working => {
            if work_log.is_empty() || work_log.iter().any(|step| step.in_progress) {
                "Working"
            } else {
                "Done"
            }
        }
    }
}

/// Tool cards shown, and how many tools are folded into the summary card.
pub fn split_tools(grants: &[ToolGrant]) -> (Vec<ToolGrant>, usize) {
    if grants.len() > MAX_TOOL_CARDS {
        (
            grants[..MAX_TOOL_CARDS].to_vec(),
            grants.len() - MAX_TOOL_CARDS,
        )
    } else {
        (grants.to_vec(), 0)
    }
}

/// File rows shown, and how many are counted in the "+N more" line.
pub fn split_files(files: &[FileItem]) -> (Vec<FileItem>, usize) {
    if files.len() > MAX_FILE_ROWS {
        (files[..MAX_FILE_ROWS].to_vec(), files.len() - MAX_FILE_ROWS)
    } else {
        (files.to_vec(), 0)
    }
}

/// Status word and line for a tool card: a live action wins, then the last action, then "Nothing yet".
pub fn tool_status(grant: &ToolGrant) -> (&'static str, String) {
    if let Some(live) = &grant.live_action {
        ("Calling", live.clone())
    } else if let Some(previous) = &grant.previous_action {
        let line = match &grant.previous_when {
            Some(when) => format!("{previous} · {when}"),
            None => previous.clone(),
        };
        ("Used", line)
    } else {
        ("Ready", "Nothing yet".to_string())
    }
}

/// Text of a work step; an empty one reads "Working…" while underway and "Done" otherwise.
pub fn step_text(step: &WorkStep) -> String {
    match (step.text.is_empty(), step.in_progress) {
        (false, _) => step.text.clone(),
        (true, true) => "Working…".to_string(),
        (true, false) => "Done".to_string(),
    }
}

/// Info card rows for a trigger key (`worker`, `tool:<key>` or `file:<key>`).
pub fn info_rows(
    target: &str,
    worker: &Worker,
    grants: &[ToolGrant],
    files: &[FileItem],
) -> Option<Vec<DetailRow>> {
    if target == "worker" {
        return Some(worker.detail_rows.clone());
    }
    if let Some(key) = target.strip_prefix("tool:") {
        let grant = grants.iter().find(|g| g.tool_key == key)?;
        let mut rows = grant.detail_rows.clone();
        let (_, line) = tool_status(grant);
        rows.push(DetailRow {
            field_label: "This request".to_string(),
            field_value: line,
        });
        return Some(rows);
    }
    if let Some(key) = target.strip_prefix("file:") {
        let file = files.iter().find(|f| f.file_key == key)?;
        return Some(file.detail_rows.clone());
    }
    None
}

/// Connector route for the step in progress, if any.
pub fn active_route(work_log: &[WorkStep]) -> Option<String> {
    work_log
        .iter()
        .rev()
        .find(|step| step.in_progress)
        .map(|step| step.route_key.clone())
}

#[derive(Clone, PartialEq, Debug)]
struct WorkView {
    key: String,
    text: String,
    route: String,
    in_progress: bool,
    filled: bool,
    placeholder: bool,
}

#[derive(Clone, PartialEq, Debug)]
struct JoinView {
    to: String,
    carrying: bool,
}

#[derive(Clone, PartialEq, Debug)]
struct ToolView {
    trigger: String,
    trigger_id: String,
    name: String,
    initial: String,
    hue: u32,
    mark: Option<String>,
    grant: String,
    status_word: &'static str,
    status_line: String,
    live: bool,
    expanded: bool,
}

#[derive(Clone, PartialEq, Debug)]
struct FileView {
    trigger: String,
    trigger_id: String,
    name: String,
    initial: String,
    hue: u32,
    mark: Option<String>,
    meta: Option<String>,
    reading: bool,
    expanded: bool,
}

#[component]
pub fn SingleAgentStage(
    requester: Requester,
    worker: Worker,
    worker_stage: WorkerStage,
    #[props(default)] tool_grants: Vec<ToolGrant>,
    #[props(default)] file_items: Vec<FileItem>,
    #[props(default)] reading_file_key: Option<String>,
    #[props(default)] work_log: Vec<WorkStep>,
    #[props(default = "Agent stage".to_string(), into)] accessible_name: String,
    #[props(default)] on_open_change: Option<EventHandler<Option<String>>>,
    #[props(default)] class_name: String,
) -> Element {
    let id = use_hook(|| format!("kit-sas-{}", next_id()));
    let mut open = use_signal(|| None::<String>);
    let mut photo_failed = use_signal(|| false);
    let joins_id = format!("{id}-joins");
    let script_id = joins_id.clone();
    use_hook(move || {
        let script = JOIN_SCRIPT.replace("__ID__", &script_id);
        spawn(async move {
            let _ = document::eval(&script).await;
        });
    });
    let drop_id = joins_id.clone();
    use_drop(move || {
        document::eval(&format!(
            "window.__kitSasJoins?.get({drop_id:?})?.disconnect(); window.__kitSasJoins?.delete({drop_id:?});"
        ));
    });

    let mut set_open = move |next: Option<String>| {
        open.set(next.clone());
        if let Some(handler) = &on_open_change {
            handler.call(next);
        }
    };
    let id_prefix = id.clone();
    let trigger_id = move |target: &str| format!("{id_prefix}-{}", target.replace(':', "-"));

    let worker_target = "worker".to_string();
    let worker_expanded = open().as_deref() == Some("worker");
    let route = active_route(&work_log);
    let stage_word = state_word(worker_stage, &work_log);
    let (tools, tools_more) = split_tools(&tool_grants);
    let (files, files_more) = split_files(&file_items);
    let tool_views: Vec<(String, ToolView)> = tools
        .iter()
        .map(|grant| {
            let trigger = format!("tool:{}", grant.tool_key);
            let (status_word, status_line) = tool_status(grant);
            (
                grant.tool_key.clone(),
                ToolView {
                    trigger_id: trigger_id(&trigger),
                    expanded: open().as_deref() == Some(trigger.as_str()),
                    hue: accent_hue(&trigger),
                    trigger,
                    name: grant.display_name.clone(),
                    initial: initial_of(&grant.display_name),
                    mark: grant.mark_source.clone(),
                    grant: grant.grant_text.clone(),
                    status_word,
                    status_line,
                    live: grant.live_action.is_some(),
                },
            )
        })
        .collect();
    let file_views: Vec<(String, FileView)> = files
        .iter()
        .map(|file| {
            let trigger = format!("file:{}", file.file_key);
            (
                file.file_key.clone(),
                FileView {
                    trigger_id: trigger_id(&trigger),
                    expanded: open().as_deref() == Some(trigger.as_str()),
                    reading: reading_file_key.as_deref() == Some(file.file_key.as_str()),
                    hue: accent_hue(&trigger),
                    trigger,
                    name: file.display_name.clone(),
                    initial: initial_of(&file.display_name),
                    mark: file.mark_source.clone(),
                    meta: file.meta_text.clone(),
                },
            )
        })
        .collect();
    let joins: Vec<JoinView> = tool_views
        .iter()
        .map(|(key, view)| JoinView {
            to: view.trigger_id.clone(),
            carrying: route.as_deref() == Some(key.as_str()),
        })
        .collect();
    let info = open().and_then(|target| {
        info_rows(&target, &worker, &tool_grants, &file_items).map(|rows| (target, rows))
    });
    let info_title = info
        .as_ref()
        .map(|(target, _)| target.clone())
        .unwrap_or_default();
    let info_trigger = info.as_ref().map(|(target, _)| trigger_id(target));
    let info_rows_view = info.map(|(_, rows)| rows).unwrap_or_default();
    let info_id = format!("{id}-info");
    let worker_trigger_id = trigger_id(&worker_target);
    let worker_mark = initial_of(&worker.display_name);
    let worker_hue = accent_hue(&worker.worker_key);
    let work_views: Vec<WorkView> = work_slots(&work_log)
        .into_iter()
        .enumerate()
        .map(|(index, slot)| match slot {
            Some(step) => WorkView {
                key: step.step_key.clone(),
                text: step_text(&step),
                route: step.route_key.clone(),
                in_progress: step.in_progress,
                filled: true,
                placeholder: false,
            },
            None => WorkView {
                key: format!("empty-{index}"),
                text: String::new(),
                route: String::new(),
                in_progress: false,
                filled: false,
                placeholder: index == 0 && work_log.is_empty(),
            },
        })
        .collect();
    let requester_mark = initial_of(&requester.display_name);
    let requester_photo = requester.photo_url.clone().filter(|_| !photo_failed());
    let files_count_text = format!("+{files_more} more");
    let tools_count_text = format!("+{tools_more} more tools");
    let route_attr = route.clone().unwrap_or_default();
    let class = format!("kit-sas {class_name}");
    let join_from = worker_trigger_id.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            class: "{class}",
            role: "region",
            "aria-label": "{accessible_name}",
            "data-stage": worker_stage.as_str(),
            "data-route": route_attr,
            onkeydown: move |event: KeyboardEvent| {
                if event.key() != Key::Escape {
                    return;
                }
                if let Some(target) = open() {
                    set_open(None);
                    focus_after_frame(&trigger_id(&target));
                }
            },
            svg { id: "{joins_id}", class: "kit-sas__joins", "aria-hidden": "true",
                for join in joins {
                    path {
                        class: "kit-sas__join",
                        "data-from": "{join_from}",
                        "data-to": "{join.to}",
                        "data-carrying": join.carrying.then_some("true"),
                    }
                }
            }
            article { class: "kit-sas__requester",
                span { class: "kit-sas__avatar", "aria-hidden": "true",
                    if let Some(src) = requester_photo {
                        img {
                            src: "{src}",
                            alt: "",
                            onerror: move |_| photo_failed.set(true),
                        }
                    } else {
                        {requester_mark}
                    }
                }
                div { class: "kit-sas__requester-text",
                    span { class: "kit-sas__name", {requester.display_name.clone()} }
                    p {
                        class: "kit-sas__message",
                        title: "{requester.message_text}",
                        {requester.message_text.clone()}
                    }
                }
            }
            button {
                id: "{worker_trigger_id}",
                class: "kit-sas__worker",
                r#type: "button",
                style: "--kit-sas-hue: {worker_hue}",
                "data-stage": worker_stage.as_str(),
                "aria-expanded": if worker_expanded { "true" } else { "false" },
                "aria-label": format!("{} details", worker.display_name),
                onclick: move |_| {
                    let next = if open().as_deref() == Some("worker") { None } else { Some("worker".to_string()) };
                    set_open(next);
                },
                span { class: "kit-sas__mark", "aria-hidden": "true", {worker_mark} }
                span { class: "kit-sas__worker-name", {worker.display_name.clone()} }
                if let Some(subtitle) = worker.subtitle_text.clone() {
                    span { class: "kit-sas__subtitle", {subtitle} }
                }
                span { class: "kit-sas__state", {stage_word} }
            }
            div { class: "kit-sas__work", role: "list", "aria-live": "polite",
                for view in work_views {
                    div {
                        key: "{view.key}",
                        class: "kit-sas__row",
                        role: view.filled.then_some("listitem"),
                        "aria-hidden": (!view.filled).then_some("true"),
                        "data-in-progress": view.in_progress.then_some("true"),
                        "data-route": view.route.clone(),
                        if view.placeholder {
                            span { class: "kit-sas__idle", "Waiting for work" }
                        }
                        {view.text.clone()}
                    }
                }
            }
            div { class: "kit-sas__tools",
                for (key, view) in tool_views {
                    button {
                        key: "{key}",
                        id: view.trigger_id.clone(),
                        class: "kit-sas__tool",
                        r#type: "button",
                        style: "--kit-sas-hue: {view.hue}",
                        "data-live": view.live.then_some("true"),
                        "aria-expanded": if view.expanded { "true" } else { "false" },
                        "aria-label": format!("{} details", view.name),
                        onclick: move |_| {
                            let next = if open().as_deref() == Some(view.trigger.as_str()) { None } else { Some(view.trigger.clone()) };
                            set_open(next);
                        },
                        span { class: "kit-sas__tool-head",
                            span { class: "kit-sas__mark", "aria-hidden": "true",
                                if let Some(src) = view.mark.clone() {
                                    img { src: src, alt: "" }
                                } else {
                                    {view.initial.clone()}
                                }
                            }
                            span { class: "kit-sas__tool-name", {view.name.clone()} }
                        }
                        span { class: "kit-sas__grant", {view.grant.clone()} }
                        span { class: "kit-sas__status", {view.status_word} }
                        span { class: "kit-sas__last", {view.status_line.clone()} }
                    }
                }
                if tools_more > 0 {
                    div { class: "kit-sas__tool kit-sas__tool--summary",
                        span { class: "kit-sas__tool-name", {tools_count_text} }
                    }
                }
            }
            div { class: "kit-sas__files",
                ul { class: "kit-sas__file-list",
                    for (key, view) in file_views {
                        li { key: "{key}",
                            button {
                                id: view.trigger_id.clone(),
                                class: "kit-sas__file",
                                r#type: "button",
                                style: "--kit-sas-hue: {view.hue}",
                                "data-reading": view.reading.then_some("true"),
                                "aria-expanded": if view.expanded { "true" } else { "false" },
                                "aria-label": format!("{} details", view.name),
                                onclick: move |_| {
                                    let next = if open().as_deref() == Some(view.trigger.as_str()) { None } else { Some(view.trigger.clone()) };
                                    set_open(next);
                                },
                                span { class: "kit-sas__file-mark", "aria-hidden": "true",
                                    if let Some(src) = view.mark.clone() {
                                        img { src: src, alt: "" }
                                    } else {
                                        {view.initial.clone()}
                                    }
                                }
                                span { class: "kit-sas__file-name", {view.name.clone()} }
                                if let Some(meta) = view.meta.clone() {
                                    span { class: "kit-sas__file-meta", {meta} }
                                }
                            }
                        }
                    }
                }
                if file_items.is_empty() {
                    p { class: "kit-sas__idle", "No files" }
                }
                if files_more > 0 {
                    p { class: "kit-sas__more", {files_count_text} }
                }
            }
            if let Some(trigger) = info_trigger {
                div {
                    id: "{info_id}",
                    class: "kit-sas__info",
                    role: "dialog",
                    "aria-label": "Details of {info_title}",
                    "data-for": trigger,
                    dl {
                        for (index, row) in info_rows_view.into_iter().enumerate() {
                            div { key: "{index}", class: "kit-sas__info-row",
                                dt { {row.field_label} }
                                dd { {row.field_value} }
                            }
                        }
                    }
                }
            }
        }
    }
}
