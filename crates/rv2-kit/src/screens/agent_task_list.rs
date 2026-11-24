//! Agent task list: the work an orchestrating agent carries out as tasks, grouped
//! by project. Each row shows how far the task has got, what it has produced and
//! whether it is waiting for a person. Opening a task shows its checklist, its
//! waiting question and its history.
//!
//! Design choice: tasks group under their project headings in the order the host
//! gives. Each heading can collapse. The tree takes one tab stop: the focused
//! heading or row carries tabindex 0 and the arrow keys move between stops.
//! Design choice: the progress ring is a conic gradient filled to steps done over
//! total. A planning task draws a dashed ring with no fill.
//! Design choice: the New task line is a single field. Enter creates a planning task
//! in the chosen project and the line stays open for the next one. The live region
//! announces the creation once the host's list has grown, not before.
//! Design choice: artifacts are listed by name. The shared artifact viewers are not
//! embedded here.
//! Design choice: "Decide" calls `on_open_decision` with the task and the ask id.
//! Without it the host's inbox is the target, which the host handles.
//! Design choice: below 480 px the open task is a sheet from the right with a Back
//! button; a row that the host adds settles in with a short grow.
//! Browser checks: each heading count equals its rows; a ring fills to steps done
//! over total; Enter in the New task line creates at the top of its project and
//! keeps the line open; the Done tab lists finished tasks with their day; arrows,
//! Home and End move through the tree; Left and Right fold a heading; no horizontal
//! scroll at 320 px, and each row takes two lines.

use std::collections::HashSet;

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, focus_after_frame, next_id, sleep};

const CSS: Asset = asset!("/assets/agent-task-list.css");

/// Milliseconds a newly added row keeps its settle animation.
const SETTLE_MS: u32 = 800;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    Planning,
    Running,
    Paused,
}

impl Progress {
    fn key(self) -> &'static str {
        match self {
            Self::Planning => "planning",
            Self::Running => "running",
            Self::Paused => "paused",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TaskTab {
    #[default]
    Active,
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AskKind {
    Decision,
    Pick,
    Approval,
    Question,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ask {
    pub id: String,
    pub question_text: String,
    pub ask_kind: AskKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TaskStep {
    pub entry_label: String,
    pub is_done: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionEntry {
    pub question_text: String,
    pub answer_text: String,
    pub at_ms: u64,
}

/// One line of "What happened so far".
#[derive(Clone, Debug, PartialEq)]
pub struct TaskEvent {
    pub at_ms: u64,
    pub entry_text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TaskRecord {
    pub id: String,
    pub entry_title: String,
    pub project_name: String,
    pub progress_state: Progress,
    pub status_phrase: Option<String>,
    pub step_records: Vec<TaskStep>,
    pub artifact_names: Vec<String>,
    pub pending_asks: Vec<Ask>,
    pub updated_at_ms: u64,
    pub is_done: bool,
    pub goal_text: Option<String>,
    pub started_at_ms: Option<u64>,
    pub decision_log: Vec<DecisionEntry>,
    pub event_log: Vec<TaskEvent>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewTaskLine {
    pub entry_title: String,
    pub project_name: String,
}

/// One stop in the tree: a project heading or a task row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeKey {
    Heading(String),
    Row(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeMove {
    Next,
    Prev,
    First,
    Last,
}

/// Steps done as a fraction of the total, 0 when there are none.
pub fn ring_fraction(steps: &[TaskStep]) -> f64 {
    if steps.is_empty() {
        return 0.0;
    }
    steps.iter().filter(|s| s.is_done).count() as f64 / steps.len() as f64
}

/// Asks waiting on the person, summed across tasks.
pub fn needs_you_total(tasks: &[TaskRecord]) -> usize {
    tasks.iter().map(|t| t.pending_asks.len()).sum()
}

/// Tasks in one group, in list order.
pub fn tasks_in_group<'a>(tasks: &'a [TaskRecord], project: &str) -> Vec<&'a TaskRecord> {
    tasks.iter().filter(|t| t.project_name == project).collect()
}

/// Whether a task belongs on the tab.
pub fn on_tab(task: &TaskRecord, tab: TaskTab) -> bool {
    match tab {
        TaskTab::Active => !task.is_done,
        TaskTab::Done => task.is_done,
    }
}

/// `planning`, `paused`, or the phrase the host gave, else `running`.
pub fn status_text(task: &TaskRecord) -> String {
    match &task.status_phrase {
        Some(phrase) => phrase.clone(),
        None => task.progress_state.key().to_string(),
    }
}

/// `3 min ago`, `2 h ago`, `4 d ago`, or `now`.
pub fn ago_text(now_ms: u64, then_ms: u64) -> String {
    let seconds = now_ms.saturating_sub(then_ms) / 1000;
    match seconds {
        0..=59 => "now".to_string(),
        60..=3599 => format!("{} min ago", seconds / 60),
        3600..=86_399 => format!("{} h ago", seconds / 3600),
        _ => format!("{} d ago", seconds / 86_400),
    }
}

/// `1 task`, `2 tasks`.
pub fn count_text(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("1 {singular}")
    } else {
        format!("{count} {plural}")
    }
}

/// `12 Nov 2026` for an epoch-millisecond instant, in UTC.
pub fn day_label(ms: u64) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let z = (ms / 86_400_000) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{day} {} {year}", MONTHS[(month - 1) as usize])
}

/// Visible tree stops in order: each group heading, then its rows while the
/// group is open and the row is on `tab`.
pub fn tree_keys(
    task_records: &[TaskRecord],
    project_names: &[String],
    tab: TaskTab,
    collapsed: &[String],
) -> Vec<TreeKey> {
    let mut keys = Vec::new();
    for project in project_names {
        keys.push(TreeKey::Heading(project.clone()));
        if !collapsed.contains(project) {
            keys.extend(
                tasks_in_group(task_records, project)
                    .into_iter()
                    .filter(|t| on_tab(t, tab))
                    .map(|t| TreeKey::Row(t.id.clone())),
            );
        }
    }
    keys
}

/// The stop `movement` reaches from `current`, clamped at both ends.
pub fn step_key(
    keys: &[TreeKey],
    current: Option<&TreeKey>,
    movement: TreeMove,
) -> Option<TreeKey> {
    if keys.is_empty() {
        return None;
    }
    let last = keys.len() - 1;
    let at = current.and_then(|c| keys.iter().position(|k| k == c));
    let index = match movement {
        TreeMove::First => 0,
        TreeMove::Last => last,
        TreeMove::Next => at.map_or(0, |i| (i + 1).min(last)),
        TreeMove::Prev => at.map_or(last, |i| i.saturating_sub(1)),
    };
    keys.get(index).cloned()
}

/// The heading a row sits under.
pub fn parent_heading(keys: &[TreeKey], row_id: &str) -> Option<TreeKey> {
    let at = keys
        .iter()
        .position(|k| *k == TreeKey::Row(row_id.to_string()))?;
    keys[..at]
        .iter()
        .rev()
        .find(|k| matches!(k, TreeKey::Heading(_)))
        .cloned()
}

fn tree_dom_id(uid: usize, key: &TreeKey) -> String {
    match key {
        TreeKey::Heading(project) => format!("kit-atl-{uid}-h-{project}"),
        TreeKey::Row(id) => format!("kit-atl-{uid}-r-{id}"),
    }
}

/// Task list grouped by project, with an Active and a Done tab, a New task line,
/// and a panel for the open task.
#[component]
pub fn AgentTaskList(
    #[props(default)] task_records: Vec<TaskRecord>,
    #[props(default)] project_names: Vec<String>,
    #[props(default)] initial_tab: TaskTab,
    #[props(default)] initial_task_id: Option<String>,
    #[props(default)] now_ms: u64,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_select: Option<EventHandler<String>>,
    #[props(default)] on_create: Option<EventHandler<NewTaskLine>>,
    #[props(default)] on_open_decision: Option<EventHandler<(String, Option<String>)>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
) -> Element {
    let uid = use_hook(next_id);
    let mut tab = use_signal(move || initial_tab);
    let mut selected = use_signal(move || initial_task_id.clone());
    let mut sheet_picked = use_signal(|| false);
    let mut artifact_tab = use_signal(|| 0_usize);
    let mut collapsed = use_signal(Vec::<String>::new);
    let mut line_open = use_signal(|| false);
    let mut line_text = use_signal(String::new);
    let first_project = project_names.first().cloned().unwrap_or_default();
    let mut line_project = use_signal(move || first_project.clone());
    let mut menu_open = use_signal(|| false);
    let mut menu_focus = use_signal(|| 0_usize);
    let mut announce = use_signal(String::new);
    let mut pending_create = use_signal(|| None::<(String, usize)>);
    let mut seen = use_signal(HashSet::<String>::new);
    let mut primed = use_signal(|| false);
    let mut fresh = use_signal(Vec::<String>::new);
    let mut focus_key = use_signal(|| None::<TreeKey>);

    use_effect(use_reactive!(|task_records| {
        let ids: HashSet<String> = task_records.iter().map(|t| t.id.clone()).collect();
        if *primed.peek() {
            let added: Vec<String> = ids
                .iter()
                .filter(|id| !seen.peek().contains(*id))
                .cloned()
                .collect();
            if !added.is_empty() {
                fresh.with_mut(|f| f.extend(added.iter().cloned()));
                spawn(async move {
                    sleep(SETTLE_MS).await;
                    fresh.with_mut(|f| f.retain(|id| !added.contains(id)));
                });
            }
        }
        let waiting = pending_create.peek().clone();
        if let Some((project, before)) = waiting {
            if tasks_in_group(&task_records, &project).len() > before {
                announce.set(format!("Task created in {project}"));
                pending_create.set(None);
            }
        }
        primed.set(true);
        seen.set(ids);
    }));

    let current_tab = tab();
    let active_count = task_records
        .iter()
        .filter(|t| on_tab(t, TaskTab::Active))
        .count();
    let done_count = task_records
        .iter()
        .filter(|t| on_tab(t, TaskTab::Done))
        .count();
    let needs = needs_you_total(&task_records);
    let needs_text = format!("{needs} waiting for you");
    let open_task = selected().and_then(|id| task_records.iter().find(|t| t.id == id).cloned());
    let collapsed_now = collapsed();
    let line_now = line_open();
    let line_value = line_text();
    let project_value = line_project();
    let menu_now = menu_open();
    let picked = project_names
        .iter()
        .position(|p| *p == project_value)
        .unwrap_or(0);
    let project_count = project_names.len();
    let project_mark: String = project_value
        .chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().collect());
    let names_for_menu = project_names.clone();
    let mut choose = move |name: String| {
        line_project.set(name);
        menu_open.set(false);
        focus_after_frame(&format!("kit-atl-{uid}-picker"));
    };
    let fresh_now = fresh();
    let keys = tree_keys(&task_records, &project_names, current_tab, &collapsed_now);
    let active_key = focus_key()
        .filter(|k| keys.contains(k))
        .or_else(|| selected().map(TreeKey::Row).filter(|k| keys.contains(k)))
        .or_else(|| keys.first().cloned());
    let row_order: Vec<String> = keys
        .iter()
        .filter_map(|k| match k {
            TreeKey::Row(id) => Some(id.clone()),
            TreeKey::Heading(_) => None,
        })
        .collect();
    let group_sizes: Vec<(String, usize)> = project_names
        .iter()
        .map(|p| (p.clone(), tasks_in_group(&task_records, p).len()))
        .collect();
    let tree_stops = keys.clone();

    let create = move || {
        let title = line_text().trim().to_string();
        if title.is_empty() {
            return;
        }
        let project = line_project();
        let before = group_sizes
            .iter()
            .find(|(p, _)| *p == project)
            .map_or(0, |(_, n)| *n);
        pending_create.set(Some((project.clone(), before)));
        if let Some(handler) = on_create {
            handler.call(NewTaskLine {
                entry_title: title,
                project_name: project,
            });
        }
        line_text.set(String::new());
    };
    let mut create = create;

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: "kit-atl",
            "data-loading": is_loading.then_some("true"),
            "aria-busy": is_loading.then_some("true"),
            onkeydown: move |event: KeyboardEvent| {
                if line_open() {
                    return;
                }
                let Key::Character(c) = event.key() else {
                    return;
                };
                if c == "c" || c == "n" {
                    line_open.set(true);
                    focus_after_frame(&format!("kit-atl-{uid}-line"));
                    return;
                }
                if c == "j" || c == "k" {
                    let delta: isize = if c == "j" { 1 } else { -1 };
                    let at = selected().as_deref().and_then(|id| row_order.iter().position(|o| o == id));
                    let next = match at {
                        Some(i) => (i as isize + delta).clamp(0, row_order.len() as isize - 1) as usize,
                        None => 0,
                    };
                    if let Some(id) = row_order.get(next) {
                        selected.set(Some(id.clone()));
                        sheet_picked.set(true);
                        if let Some(handler) = on_select {
                            handler.call(id.clone());
                        }
                    }
                }
            },
            header { class: "kit-atl__head",
                div { class: "kit-atl__heading",
                    h2 { class: "kit-atl__title", "Tasks" }
                    span { class: "kit-atl__count", "{active_count}" }
                    p { class: "kit-atl__meta", "{needs_text}" }
                }
                div { class: "kit-atl__tabs", role: "tablist", "aria-label": "Show",
                    button {
                        r#type: "button",
                        role: "tab",
                        class: "kit-atl__tab",
                        "aria-selected": (current_tab == TaskTab::Active).to_string(),
                        onclick: move |_| tab.set(TaskTab::Active),
                        "Active "
                        span { class: "kit-atl__count", "{active_count}" }
                    }
                    button {
                        r#type: "button",
                        role: "tab",
                        class: "kit-atl__tab",
                        "aria-selected": (current_tab == TaskTab::Done).to_string(),
                        onclick: move |_| tab.set(TaskTab::Done),
                        onkeydown: move |event: KeyboardEvent| {
                            if matches!(event.key(), Key::ArrowLeft | Key::ArrowRight) {
                                tab.set(TaskTab::Active);
                            }
                        },
                        "Done "
                        span { class: "kit-atl__count", "{done_count}" }
                    }
                }
                button {
                    id: "kit-atl-{uid}-new",
                    r#type: "button",
                    class: "kit-atl__primary",
                    "aria-expanded": line_now.to_string(),
                    onclick: move |_| line_open.toggle(),
                    "New task"
                }
            }

            if line_now {
                form {
                    class: "kit-atl__line",
                    "aria-label": "New task",
                    onsubmit: move |event: FormEvent| {
                        event.prevent_default();
                        create();
                    },
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape {
                            line_open.set(false);
                            focus_after_frame(&format!("kit-atl-{uid}-new"));
                        }
                    },
                    input {
                        id: "kit-atl-{uid}-line",
                        class: "kit-atl__input",
                        r#type: "text",
                        placeholder: "Describe a task",
                        "aria-label": "Task description",
                        value: "{line_value}",
                        oninput: move |event| line_text.set(event.value()),
                    }
                    div { class: "kit-atl__picker-wrap",
                        button {
                            id: "kit-atl-{uid}-picker",
                            r#type: "button",
                            class: "kit-atl__picker",
                            "aria-haspopup": "menu",
                            "aria-expanded": "{menu_now}",
                            "aria-label": "Project: {project_value}",
                            onclick: move |_| {
                                if menu_open() {
                                    menu_open.set(false);
                                } else {
                                    menu_focus.set(picked);
                                    menu_open.set(true);
                                    focus_after_frame(&format!("kit-atl-{uid}-pick-{picked}"));
                                }
                            },
                            onkeydown: move |event: KeyboardEvent| {
                                if matches!(event.key(), Key::ArrowDown | Key::ArrowUp) {
                                    event.prevent_default();
                                    menu_focus.set(picked);
                                    menu_open.set(true);
                                    focus_after_frame(&format!("kit-atl-{uid}-pick-{picked}"));
                                }
                            },
                            span { class: "kit-atl__mark", "aria-hidden": "true", "{project_mark}" }
                            span { class: "kit-atl__picker-name", "{project_value}" }
                            span { class: "kit-atl__chevron", "aria-hidden": "true", "▾" }
                        }
                        if menu_now {
                            ul {
                                class: "kit-atl__menu",
                                role: "menu",
                                "aria-label": "Projects",
                                onkeydown: move |event: KeyboardEvent| {
                                    let current = menu_focus();
                                    let next = match event.key() {
                                        Key::ArrowDown => (current + 1).min(project_count.saturating_sub(1)),
                                        Key::ArrowUp => current.saturating_sub(1),
                                        Key::Home => 0,
                                        Key::End => project_count.saturating_sub(1),
                                        Key::Escape => {
                                            event.stop_propagation();
                                            event.prevent_default();
                                            menu_open.set(false);
                                            focus_after_frame(&format!("kit-atl-{uid}-picker"));
                                            return;
                                        }
                                        Key::Tab => {
                                            menu_open.set(false);
                                            return;
                                        }
                                        _ => return,
                                    };
                                    event.stop_propagation();
                                    event.prevent_default();
                                    menu_focus.set(next);
                                    focus_after_frame(&format!("kit-atl-{uid}-pick-{next}"));
                                },
                                for (index, name) in names_for_menu.iter().cloned().enumerate() {
                                    {
                                        let item_id = format!("kit-atl-{uid}-pick-{index}");
                                        let is_checked = index == picked;
                                        let item_mark = name.chars().next().map_or_else(|| "?".to_string(), |c| c.to_uppercase().collect());
                                        let choose_name = name.clone();
                                        rsx! {
                                            li { key: "{name}", role: "none",
                                                button {
                                                    id: "{item_id}",
                                                    r#type: "button",
                                                    role: "menuitemradio",
                                                    class: "kit-atl__menu-item",
                                                    "aria-checked": "{is_checked}",
                                                    tabindex: if index == menu_focus() { "0" } else { "-1" },
                                                    onfocus: move |_| menu_focus.set(index),
                                                    onclick: move |_| choose(choose_name.clone()),
                                                    span { class: "kit-atl__mark", "aria-hidden": "true", "{item_mark}" }
                                                    span { class: "kit-atl__menu-name", "{name}" }
                                                    if is_checked {
                                                        svg { class: "kit-atl__check", "aria-hidden": "true", view_box: "0 0 16 16", width: "14", height: "14", fill: "none", stroke: "currentColor", stroke_width: "2",
                                                            path { d: "M3 8.5l3 3 7-7" }
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

            if is_empty && !is_loading {
                div { class: "kit-atl__empty",
                    if current_tab == TaskTab::Active {
                        p { "No active tasks. Start one and the agent plans it." }
                        button {
                            r#type: "button",
                            class: "kit-atl__primary",
                            onclick: move |_| line_open.set(true),
                            "New task"
                        }
                    } else {
                        p { "Nothing is finished yet." }
                    }
                }
            } else if is_loading {
                div { class: "kit-atl__ph", "aria-hidden": "true" }
                div { class: "kit-atl__ph", "aria-hidden": "true" }
            } else {
                div { class: "kit-atl__body",
                    ul {
                        class: "kit-atl__tree",
                        role: "tree",
                        "aria-label": "Tasks by project",
                        onkeydown: move |event: KeyboardEvent| {
                            let current = focus_key();
                            let movement = match event.key() {
                                Key::ArrowDown => Some(TreeMove::Next),
                                Key::ArrowUp => Some(TreeMove::Prev),
                                Key::Home => Some(TreeMove::First),
                                Key::End => Some(TreeMove::Last),
                                _ => None,
                            };
                            if let Some(movement) = movement {
                                event.prevent_default();
                                if let Some(next) = step_key(&tree_stops, current.as_ref(), movement) {
                                    focus_key.set(Some(next.clone()));
                                    focus_after_frame(&tree_dom_id(uid, &next));
                                }
                                return;
                            }
                            match (event.key(), current) {
                                (Key::ArrowRight, Some(TreeKey::Heading(project))) => {
                                    if collapsed().contains(&project) {
                                        event.prevent_default();
                                        collapsed.with_mut(|c| c.retain(|v| v != &project));
                                    }
                                }
                                (Key::ArrowLeft, Some(TreeKey::Heading(project))) => {
                                    if !collapsed().contains(&project) {
                                        event.prevent_default();
                                        collapsed.with_mut(|c| c.push(project));
                                    }
                                }
                                (Key::ArrowLeft, Some(TreeKey::Row(id))) => {
                                    if let Some(parent) = parent_heading(&tree_stops, &id) {
                                        event.prevent_default();
                                        focus_key.set(Some(parent.clone()));
                                        focus_after_frame(&tree_dom_id(uid, &parent));
                                    }
                                }
                                _ => {}
                            }
                        },
                        for project in project_names.iter().cloned() {
                            {
                                let group_open = !collapsed_now.contains(&project);
                                let rows: Vec<TaskRecord> = tasks_in_group(&task_records, &project)
                                    .into_iter()
                                    .filter(|t| on_tab(t, current_tab))
                                    .cloned()
                                    .collect();
                                let group_count = rows.len();
                                let group_label = format!("{project}, {}", count_text(group_count, "task", "tasks"));
                                let group_key = project.clone();
                                let toggle_name = project.clone();
                                let head_key = TreeKey::Heading(project.clone());
                                let head_dom = tree_dom_id(uid, &head_key);
                                let head_active = active_key.as_ref() == Some(&head_key);
                                let head_focus = head_key.clone();
                                rsx! {
                                    li { key: "g-{group_key}", role: "treeitem", "aria-expanded": "{group_open}", "aria-label": "{group_label}", class: "kit-atl__group",
                                        button {
                                            id: "{head_dom}",
                                            r#type: "button",
                                            class: "kit-atl__heading-btn",
                                            tabindex: if head_active { "0" } else { "-1" },
                                            onfocus: move |_| focus_key.set(Some(head_focus.clone())),
                                            onclick: move |_| {
                                                let mut next = collapsed();
                                                if next.contains(&toggle_name) {
                                                    next.retain(|v| v != &toggle_name);
                                                } else {
                                                    next.push(toggle_name.clone());
                                                }
                                                collapsed.set(next);
                                            },
                                            span { class: "kit-atl__chevron", "aria-hidden": "true", "▾" }
                                            span { "{project}" }
                                            span { class: "kit-atl__count", "{group_count}" }
                                        }
                                        if group_open {
                                            ul { class: "kit-atl__rows", role: "group",
                                                for task in rows {
                                                    {
                                                        let row_key = task.id.clone();
                                                        let row_open = selected().as_deref() == Some(task.id.as_str());
                                                        let stop = TreeKey::Row(task.id.clone());
                                                        let row_dom = tree_dom_id(uid, &stop);
                                                        let row_active = active_key.as_ref() == Some(&stop);
                                                        let row_focus = stop.clone();
                                                        let is_fresh = fresh_now.contains(&task.id);
                                                        let asks = task.pending_asks.len();
                                                        let steps_done = task.step_records.iter().filter(|s| s.is_done).count();
                                                        let steps_total = task.step_records.len();
                                                        let ring_pct = format!("{:.0}%", ring_fraction(&task.step_records) * 100.0);
                                                        let ring_style = format!("--kit-atl-ring: {ring_pct};");
                                                        let progress_key = task.progress_state.key();
                                                        let status = status_text(&task);
                                                        let when = if task.is_done {
                                                            format!("Done {}", day_label(task.updated_at_ms))
                                                        } else {
                                                            ago_text(now_ms, task.updated_at_ms)
                                                        };
                                                        let artifacts = task.artifact_names.len();
                                                        let title = task.entry_title.clone();
                                                        let row_label = format!(
                                                            "{title}, {project}, {asks} waiting for you, {steps_done} of {steps_total} steps, {}, {}",
                                                            count_text(artifacts, "artifact", "artifacts"),
                                                            if task.is_done { format!("done {}", day_label(task.updated_at_ms)) } else { format!("updated {when}") },
                                                        );
                                                        let pick = task.id.clone();
                                                        let progress_line = format!("{steps_done}/{steps_total}");
                                                        let files_line = count_text(artifacts, "artifact", "artifacts");
                                                        rsx! {
                                                            li { key: "{row_key}", role: "treeitem", class: "kit-atl__row", "data-new": is_fresh.then_some("true"),
                                                                button {
                                                                    id: "{row_dom}",
                                                                    r#type: "button",
                                                                    class: "kit-atl__row-hit",
                                                                    tabindex: if row_active { "0" } else { "-1" },
                                                                    "aria-label": "{row_label}",
                                                                    "aria-current": row_open.then_some("true"),
                                                                    onfocus: move |_| focus_key.set(Some(row_focus.clone())),
                                                                    onclick: move |_| {
                                                                        selected.set(Some(pick.clone()));
                                                                        sheet_picked.set(true);
                                                                        if let Some(handler) = on_select {
                                                                            handler.call(pick.clone());
                                                                        }
                                                                    },
                                                                    span { class: "kit-atl__ring", "data-progress": "{progress_key}", style: "{ring_style}", "aria-hidden": "true" }
                                                                    span { class: "kit-atl__row-title", "{title}" }
                                                                    span { class: "kit-atl__status", "{status}" }
                                                                    if asks > 0 {
                                                                        span { class: "kit-atl__needs", "Needs you {asks}" }
                                                                    }
                                                                    span { class: "kit-atl__steps", "{progress_line}" }
                                                                    span { class: "kit-atl__files", "{files_line}" }
                                                                    span { class: "kit-atl__ago", "{when}" }
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

                    if let Some(task) = open_task.clone() {
                        {
                            let title = task.entry_title.clone();
                            let project = task.project_name.clone();
                            let state_key = task.progress_state.key();
                            let state = status_text(&task);
                            let goal = task.goal_text.clone().unwrap_or_else(|| "No goal set.".to_string());
                            let steps = task.step_records.clone();
                            let artifacts = task.artifact_names.clone();
                            let asks = task.pending_asks.clone();
                            let first_ask = asks.first().cloned();
                            let decisions = task.decision_log.clone();
                            let task_id = task.id.clone();
                            let back_dom = tree_dom_id(uid, &TreeKey::Row(task.id.clone()));
                            let first_ask_id = first_ask.as_ref().map(|a| a.id.clone());
                            let ask_text = first_ask.as_ref().map(|a| a.question_text.clone()).unwrap_or_default();
                            let ask_kind = first_ask.as_ref().map(|a| match a.ask_kind {
                                AskKind::Decision => "Decision",
                                AskKind::Pick => "Pick",
                                AskKind::Approval => "Approval",
                                AskKind::Question => "Question",
                            });
                            let extra = asks.len().saturating_sub(1);
                            let decide_task = task_id.clone();
                            let mut events = task.event_log.clone();
                            events.sort_by_key(|e| std::cmp::Reverse(e.at_ms));
                            let artifact_now = artifact_tab().min(artifacts.len().saturating_sub(1));
                            let panel_label = format!("Task: {title}");
                            let kind_text = ask_kind.unwrap_or("Question");
                            rsx! {
                                aside { class: "kit-atl__panel", "aria-label": "{panel_label}", "data-picked": sheet_picked().then_some("true"),
                                    button {
                                        r#type: "button",
                                        class: "kit-atl__back",
                                        onclick: move |_| {
                                            sheet_picked.set(false);
                                            selected.set(None);
                                            focus_after_frame(&back_dom);
                                        },
                                        "Back"
                                    }
                                    h3 { class: "kit-atl__panel-title", "{title}" }
                                    p { class: "kit-atl__meta", "{project}" }
                                    p { class: "kit-atl__meta", "State: {state}" }
                                    p { class: "kit-atl__goal", "data-progress": "{state_key}", "{goal}" }
                                    section { class: "kit-atl__block", "aria-label": "Checklist",
                                        h4 { class: "kit-atl__section", "Steps" }
                                        ol { class: "kit-atl__checklist",
                                            for (index, step) in steps.iter().enumerate() {
                                                {
                                                    let step_key = format!("{index}");
                                                    let step_text = step.entry_label.clone();
                                                    let step_mark = if step.is_done { "✓" } else { "○" };
                                                    rsx! { li { key: "{step_key}", span { "aria-hidden": "true", "{step_mark}" } span { "{step_text}" } } }
                                                }
                                            }
                                        }
                                    }
                                    if let Some(ask_id) = first_ask_id.clone() {
                                        section { class: "kit-atl__needs-block", "aria-label": "Needs you",
                                            h4 { class: "kit-atl__section", "Needs you · {kind_text}" }
                                            p { "{ask_text}" }
                                            if extra > 0 {
                                                p { class: "kit-atl__meta", "{extra} more waiting" }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "kit-atl__primary",
                                                onclick: move |_| {
                                                    if let Some(handler) = on_open_decision {
                                                        handler.call((decide_task.clone(), Some(ask_id.clone())));
                                                    } else if let Some(nav) = on_navigate {
                                                        nav.call("Inbox".to_string());
                                                    }
                                                },
                                                "Decide"
                                            }
                                        }
                                    }
                                    section { class: "kit-atl__block", "aria-label": "Artifacts",
                                        h4 { class: "kit-atl__section", "Artifacts" }
                                        if artifacts.is_empty() {
                                            p { class: "kit-atl__meta", "None yet." }
                                        }
                                        if !artifacts.is_empty() {
                                            div { class: "kit-atl__artifact-tabs", role: "tablist", "aria-label": "Artifacts",
                                                for (index, name) in artifacts.iter().cloned().enumerate() {
                                                    {
                                                        let tab_key = format!("{index}");
                                                        let tab_selected = (index == artifact_now).to_string();
                                                        rsx! {
                                                            button {
                                                                key: "{tab_key}",
                                                                r#type: "button",
                                                                role: "tab",
                                                                class: "kit-atl__artifact-tab",
                                                                "aria-selected": "{tab_selected}",
                                                                onclick: move |_| artifact_tab.set(index),
                                                                "{name}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            if let Some(name) = artifacts.get(artifact_now).cloned() {
                                                {
                                                    let kind = if name.ends_with(".csv") || name.ends_with(".xlsx") { "Sheet" } else { "Document" };
                                                    let kind_line = format!("{kind} · {name}");
                                                    rsx! { p { class: "kit-atl__meta", role: "tabpanel", "{kind_line}" } }
                                                }
                                            }
                                        }
                                    }
                                    section { class: "kit-atl__block", "aria-label": "What happened so far",
                                        h4 { class: "kit-atl__section", "What happened so far" }
                                        if events.is_empty() {
                                            p { class: "kit-atl__meta", "Nothing yet." }
                                        }
                                        ul { class: "kit-atl__artifacts",
                                            for (index, event) in events.iter().enumerate() {
                                                {
                                                    let event_key = format!("{index}");
                                                    let event_line = format!("{} · {}", ago_text(now_ms, event.at_ms), event.entry_text);
                                                    rsx! { li { key: "{event_key}", "{event_line}" } }
                                                }
                                            }
                                        }
                                    }
                                    section { class: "kit-atl__block", "aria-label": "Decisions made",
                                        h4 { class: "kit-atl__section", "Decisions made" }
                                        if decisions.is_empty() {
                                            p { class: "kit-atl__meta", "Nothing decided yet." }
                                        }
                                        ul { class: "kit-atl__artifacts",
                                            for (index, entry) in decisions.iter().enumerate() {
                                                {
                                                    let decision_key = format!("{index}");
                                                    let decision_line = format!("{}: {}", entry.question_text, entry.answer_text);
                                                    rsx! { li { key: "{decision_key}", "{decision_line}" } }
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

            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
