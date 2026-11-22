//! Agent task board: every background task the agents work on, grouped by what
//! each one needs from the person. Waiting and failed tasks sit under Needs you,
//! running tasks under Running, and done or stopped tasks under Done. The open
//! task shows in a panel beside the board with the actions its state allows.
//!
//! Design choice: `actions_for` gives each state its own actions, so Approve and
//! Reject never appear on a running task and Retry only on a failed one.
//! Design choice: the clock is `clock_ms` when the host supplies one. A running
//! task with `started_at` counts from that clock; otherwise `elapsed_seconds` is
//! shown. Mount time is never used.
//! Design choice: below 900 px the columns stack and the open task's panel is a
//! sheet over the board with a Back button; wider, the panel sits beside them.
//! Design choice: a host state change shows in place first. The card keeps its
//! column and displays the new state for `SETTLE_MS`, then moves. Reduced motion
//! skips the wait. A card first rendered is placed at its given state.
//! Design choice: J and K move the open task through the board in column order,
//! so the panel always shows the focused card.
//! Design choice: `on_navigate` is not wired; the page has no navigation entries
//! of its own, the sidebar belongs to the host.
//! Browser checks: card moves column on a state change; J/K and N; Ctrl/Cmd+Enter
//! creates; Escape closes the form and returns focus to New task; agent filter
//! hides cards in every column; Back returns from the sheet; no horizontal scroll
//! at 320 px.

use std::collections::HashMap;

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, focus_after_frame, next_id, use_now_ms};

/// Milliseconds a changed card shows its new state in its old column.
const SETTLE_MS: u32 = 700;

/// Waits out the in-place step; zero under reduced motion.
async fn settle() {
    let _ = document::eval(&format!(
        "const ms = window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 0 : {SETTLE_MS}; \
         await new Promise((resolve) => setTimeout(resolve, ms)); return true;"
    ))
    .await;
}

fn card_dom_id(uid: usize, task_id: &str) -> String {
    format!("kit-tb-{uid}-t-{task_id}")
}

const CSS: Asset = asset!("/assets/agent-task-board.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    Waiting,
    Failed,
    Running,
    Done,
    Stopped,
}

impl TaskState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Failed => "failed",
            Self::Running => "running",
            Self::Done => "done",
            Self::Stopped => "stopped",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Waiting => "Waiting for you",
            Self::Failed => "Failed",
            Self::Running => "Running",
            Self::Done => "Done",
            Self::Stopped => "Stopped",
        }
    }

    pub fn column(self) -> TaskColumn {
        match self {
            Self::Waiting | Self::Failed => TaskColumn::NeedsYou,
            Self::Running => TaskColumn::Running,
            Self::Done | Self::Stopped => TaskColumn::Done,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskColumn {
    NeedsYou,
    Running,
    Done,
}

impl TaskColumn {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NeedsYou => "needs-you",
            Self::Running => "running",
            Self::Done => "done",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::NeedsYou => "Needs you",
            Self::Running => "Running",
            Self::Done => "Done",
        }
    }

    fn empty_text(self) -> &'static str {
        match self {
            Self::NeedsYou => "Nothing waiting on you.",
            Self::Running => "Nothing running.",
            Self::Done => "Nothing finished yet.",
        }
    }
}

const COLUMNS: [TaskColumn; 3] = [TaskColumn::NeedsYou, TaskColumn::Running, TaskColumn::Done];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TaskTab {
    #[default]
    All,
    NeedsYou,
    Running,
    Done,
}

impl TaskTab {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::NeedsYou => "needs-you",
            Self::Running => "running",
            Self::Done => "done",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::NeedsYou => "Needs you",
            Self::Running => "Running",
            Self::Done => "Done",
        }
    }

    pub fn admits(self, state: TaskState) -> bool {
        match self {
            Self::All => true,
            Self::NeedsYou => state.column() == TaskColumn::NeedsYou,
            Self::Running => state.column() == TaskColumn::Running,
            Self::Done => state.column() == TaskColumn::Done,
        }
    }

    fn all() -> [TaskTab; 4] {
        [Self::All, Self::NeedsYou, Self::Running, Self::Done]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BoardView {
    #[default]
    Board,
    List,
}

impl BoardView {
    fn as_str(self) -> &'static str {
        match self {
            Self::Board => "board",
            Self::List => "list",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskAction {
    Approve,
    Reject,
    Retry,
    Open,
}

impl TaskAction {
    fn label(self) -> &'static str {
        match self {
            Self::Approve => "Approve",
            Self::Reject => "Reject",
            Self::Retry => "Retry",
            Self::Open => "Open",
        }
    }
}

/// The actions a task in this state offers, in the order the panel shows them.
pub fn actions_for(state: TaskState) -> Vec<TaskAction> {
    match state {
        TaskState::Waiting => vec![TaskAction::Approve, TaskAction::Reject],
        TaskState::Failed => vec![TaskAction::Retry],
        TaskState::Running | TaskState::Done | TaskState::Stopped => vec![TaskAction::Open],
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StepRecord {
    pub entry_label: String,
    pub time_label: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TaskRecord {
    pub id: String,
    pub entry_title: String,
    pub entry_description: Option<String>,
    pub agent_id: String,
    pub task_state: TaskState,
    /// Epoch milliseconds on the host clock; read against `clock_ms`.
    pub started_at: Option<u64>,
    pub elapsed_seconds: Option<u64>,
    pub source_count: Option<u32>,
    pub tool_count: Option<u32>,
    pub step_records: Vec<StepRecord>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentRecord {
    pub id: String,
    pub entry_name: String,
    /// One of `ok`, `warn`, `fault`, `accent`; anything else draws the neutral mark.
    pub identity_hue_token: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewTask {
    pub description: String,
    pub agent_id: String,
}

/// Seconds a task has run at `now_ms`, or its stored elapsed time when it has no
/// start on the host clock (or no clock is supplied).
pub fn live_seconds(task: &TaskRecord, now_ms: u64) -> u64 {
    match task.started_at {
        Some(start) if now_ms > start => (now_ms - start) / 1000,
        _ => task.elapsed_seconds.unwrap_or(0),
    }
}

/// `42s`, `4m 12s`, `1h 03m`.
pub fn duration_text(seconds: u64) -> String {
    if seconds < 60 {
        format!("{seconds}s")
    } else if seconds < 3600 {
        format!("{}m {:02}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {:02}m", seconds / 3600, (seconds % 3600) / 60)
    }
}

/// Tasks whose agent is in `agent_ids`; an empty selection keeps every task.
pub fn filter_by_agent(tasks: &[TaskRecord], agent_ids: &[String]) -> Vec<TaskRecord> {
    tasks
        .iter()
        .filter(|task| agent_ids.is_empty() || agent_ids.contains(&task.agent_id))
        .cloned()
        .collect()
}

/// Counts for the tabs, in tab order: all, needs you, running, done.
pub fn tab_counts(tasks: &[TaskRecord]) -> [usize; 4] {
    let count = |tab: TaskTab| tasks.iter().filter(|t| tab.admits(t.task_state)).count();
    [
        count(TaskTab::All),
        count(TaskTab::NeedsYou),
        count(TaskTab::Running),
        count(TaskTab::Done),
    ]
}

/// Task ids in board order: each column in turn, list order kept within it.
pub fn visible_order(tasks: &[TaskRecord]) -> Vec<String> {
    COLUMNS
        .iter()
        .flat_map(|column| {
            tasks
                .iter()
                .filter(move |t| t.task_state.column() == *column)
                .map(|t| t.id.clone())
        })
        .collect()
}

/// The id `delta` steps from `current` in `order`, clamped at both ends.
pub fn step_index(order: &[String], current: Option<&str>, delta: isize) -> Option<String> {
    if order.is_empty() {
        return None;
    }
    let last = order.len() as isize - 1;
    let next = match current.and_then(|id| order.iter().position(|o| o == id)) {
        Some(index) => (index as isize + delta).clamp(0, last),
        None if delta >= 0 => 0,
        None => last,
    };
    Some(order[next as usize].clone())
}

/// Live-region text after a count changes.
pub fn announcement(needs_you: usize, running: usize) -> String {
    format!(
        "{} need you, {} running",
        plural(needs_you, "task"),
        running
    )
}

fn plural(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

/// A new task needs a description and an agent.
pub fn new_task_ready(description: &str, agent_id: &str) -> bool {
    !description.trim().is_empty() && !agent_id.is_empty()
}

fn agent_name(agents: &[AgentRecord], id: &str) -> String {
    agents
        .iter()
        .find(|a| a.id == id)
        .map_or_else(|| "Unassigned".to_string(), |a| a.entry_name.clone())
}

fn agent_hue(agents: &[AgentRecord], id: &str) -> Option<String> {
    agents
        .iter()
        .find(|a| a.id == id)
        .and_then(|a| a.identity_hue_token.clone())
}

fn initial_mark(name: &str) -> String {
    name.chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().collect())
}

fn counts_in(tasks: &[TaskRecord], column: TaskColumn) -> usize {
    tasks
        .iter()
        .filter(|t| t.task_state.column() == column)
        .count()
}

/// A state in `column`, so the tab filter can be asked about a whole column.
fn column_probe(column: TaskColumn) -> TaskState {
    match column {
        TaskColumn::NeedsYou => TaskState::Waiting,
        TaskColumn::Running => TaskState::Running,
        TaskColumn::Done => TaskState::Done,
    }
}

/// Board of background tasks in three columns, or a list, with an open-task
/// panel. Actions call back with the task; the host changes its state and the
/// card moves when the new state arrives.
#[component]
pub fn AgentTaskBoard(
    #[props(default)] tasks: Vec<TaskRecord>,
    #[props(default)] agents: Vec<AgentRecord>,
    #[props(default)] initial_view: BoardView,
    #[props(default)] initial_task_id: Option<String>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] clock_ms: Option<u64>,
    #[props(default)] on_approve: Option<EventHandler<TaskRecord>>,
    #[props(default)] on_reject: Option<EventHandler<TaskRecord>>,
    #[props(default)] on_retry: Option<EventHandler<TaskRecord>>,
    #[props(default)] on_open: Option<EventHandler<TaskRecord>>,
    #[props(default)] on_create: Option<EventHandler<NewTask>>,
) -> Element {
    let uid = use_hook(next_id);
    let now = use_now_ms(clock_ms, 1000);
    let default_agent = agents.first().map(|a| a.id.clone()).unwrap_or_default();
    let mut view = use_signal(|| initial_view);
    let mut tab = use_signal(TaskTab::default);
    let mut agent_filter = use_signal(Vec::<String>::new);
    let mut open_id = use_signal(|| initial_task_id.clone());
    let mut picked = use_signal(|| false);
    let mut form_open = use_signal(|| false);
    let mut draft_text = use_signal(String::new);
    let mut draft_agent = use_signal(|| default_agent.clone());

    let mut placed = use_signal(HashMap::<String, TaskState>::new);
    let mut latest = use_signal(HashMap::<String, TaskState>::new);
    use_effect(use_reactive!(|tasks| {
        let fresh: Vec<(String, TaskState)> = tasks
            .iter()
            .filter(|t| !placed.peek().contains_key(&t.id))
            .map(|t| (t.id.clone(), t.task_state))
            .collect();
        let settling: Vec<String> = tasks
            .iter()
            .filter(|t| placed.peek().get(&t.id).is_some_and(|s| *s != t.task_state))
            .map(|t| t.id.clone())
            .collect();
        latest.set(tasks.iter().map(|t| (t.id.clone(), t.task_state)).collect());
        if !fresh.is_empty() {
            placed.with_mut(|m| m.extend(fresh));
        }
        if !settling.is_empty() {
            spawn(async move {
                settle().await;
                let target = latest.peek().clone();
                placed.with_mut(|m| {
                    for id in &settling {
                        if let Some(state) = target.get(id) {
                            m.insert(id.clone(), *state);
                        }
                    }
                });
            });
        }
    }));

    let filtered = filter_by_agent(&tasks, &agent_filter());
    let placement = placed();
    let shown: Vec<(TaskRecord, TaskState)> = filtered
        .iter()
        .map(|t| {
            (
                t.clone(),
                placement.get(&t.id).copied().unwrap_or(t.task_state),
            )
        })
        .collect();
    let board: Vec<TaskRecord> = shown
        .iter()
        .map(|(t, state)| TaskRecord {
            task_state: *state,
            ..t.clone()
        })
        .collect();
    let counts = tab_counts(&board);
    let order = visible_order(&board);
    let open_task = open_id().and_then(|id| tasks.iter().find(|t| t.id == id).cloned());
    let form_id = format!("kit-tb-{uid}-form");
    let new_button_id = format!("kit-tb-{uid}-new");
    let announce = announcement(counts[1], counts[2]);
    let draft_text_now = draft_text();
    let draft_agent_now = draft_agent();
    let ready = new_task_ready(&draft_text_now, &draft_agent_now);
    let is_running_open = open_task
        .as_ref()
        .is_some_and(|t| t.task_state == TaskState::Running);
    let agent_options = agents.clone();

    let mut submit = move || {
        let text = draft_text();
        let agent = draft_agent();
        if !new_task_ready(&text, &agent) {
            return;
        }
        if let Some(handler) = on_create {
            handler.call(NewTask {
                description: text.trim().to_string(),
                agent_id: agent,
            });
        }
        draft_text.set(String::new());
        form_open.set(false);
        focus_after_frame(&format!("kit-tb-{uid}-new"));
    };

    let key_order = order.clone();
    let counts_all = counts[0];
    let counts_needs = counts[1];
    let counts_running = counts[2];
    let counts_done = counts[3];
    let tab_counts_list = [counts_all, counts_needs, counts_running, counts_done];

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: "kit-tb",
            "data-loading": is_loading.then_some("true"),
            "aria-busy": is_loading.then_some("true"),
            onkeydown: move |event: KeyboardEvent| {
                if form_open() {
                    if event.key() == Key::Escape {
                        form_open.set(false);
                        focus_after_frame(&format!("kit-tb-{uid}-new"));
                    }
                    return;
                }
                if let Key::Character(c) = event.key() {
                    match c.as_str() {
                        "j" | "k" => {
                            let delta = if c == "j" { 1 } else { -1 };
                            if let Some(next) = step_index(&key_order, open_id().as_deref(), delta) {
                                open_id.set(Some(next));
                                picked.set(true);
                            }
                        }
                        "n" if !is_loading => form_open.set(true),
                        _ => {}
                    }
                }
            },
            header { class: "kit-tb__top",
                h2 { class: "kit-tb__title", "Agent tasks" }
                div { class: "kit-tb__tools",
                    div {
                        class: "kit-tb__views",
                        role: "radiogroup",
                        "aria-label": "View",
                        for option in [BoardView::Board, BoardView::List] {
                            {
                                let option_key = option.as_str();
                                let option_label = if option == BoardView::Board { "Board" } else { "List" };
                                rsx! {
                                    button {
                                        key: "{option_key}",
                                        r#type: "button",
                                        class: "kit-tb__view",
                                        role: "radio",
                                        "aria-checked": (view() == option).to_string(),
                                        tabindex: if view() == option { "0" } else { "-1" },
                                        onclick: move |_| view.set(option),
                                        onkeydown: move |event: KeyboardEvent| {
                                            if matches!(event.key(), Key::ArrowLeft | Key::ArrowRight) {
                                                view.set(if view() == BoardView::Board {
                                                    BoardView::List
                                                } else {
                                                    BoardView::Board
                                                });
                                            }
                                        },
                                        ViewGlyph { view: option }
                                        span { class: "kit-tb__label", "{option_label}" }
                                    }
                                }
                            }
                        }
                    }
                    button {
                        id: "{new_button_id}",
                        r#type: "button",
                        class: "kit-tb__new",
                        "aria-expanded": form_open().to_string(),
                        "aria-controls": "{form_id}",
                        onclick: move |_| form_open.toggle(),
                        svg { class: "kit-tb__glyph", "aria-hidden": "true", view_box: "0 0 16 16", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                            path { d: "M8 3v10M3 8h10" }
                        }
                        span { class: "kit-tb__label", "New task" }
                    }
                }
            }

            if is_empty && !is_loading {
                div { class: "kit-tb__empty",
                    p { "No tasks yet. Start one and an agent works on it in the background." }
                    button {
                        r#type: "button",
                        class: "kit-tb__new",
                        onclick: move |_| form_open.set(true),
                        "New task"
                    }
                }
            }

            if form_open() {
                form {
                    id: "{form_id}",
                    class: "kit-tb__form",
                    "aria-label": "New task",
                    onsubmit: move |event: FormEvent| {
                        event.prevent_default();
                        submit();
                    },
                    label { class: "kit-tb__field",
                        span { "Describe the task" }
                        textarea {
                            class: "kit-tb__input",
                            rows: "3",
                            value: "{draft_text_now}",
                            oninput: move |event| draft_text.set(event.value()),
                            onkeydown: move |event: KeyboardEvent| {
                                if event.key() == Key::Enter
                                    && (event.modifiers().ctrl() || event.modifiers().meta())
                                {
                                    event.prevent_default();
                                    submit();
                                }
                            },
                        }
                    }
                    label { class: "kit-tb__field",
                        span { "Agent" }
                        select {
                            class: "kit-tb__input",
                            value: "{draft_agent_now}",
                            onchange: move |event| draft_agent.set(event.value()),
                            for agent in agent_options.iter() {
                                {
                                    let option_id = agent.id.clone();
                                    let option_name = agent.entry_name.clone();
                                    rsx! {
                                        option { key: "{option_id}", value: "{option_id}", "{option_name}" }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "kit-tb__form-actions",
                        button {
                            r#type: "button",
                            class: "kit-tb__ghost",
                            onclick: move |_| {
                                form_open.set(false);
                                focus_after_frame(&format!("kit-tb-{uid}-new"));
                            },
                            "Cancel"
                        }
                        button {
                            r#type: "submit",
                            class: "kit-tb__new",
                            disabled: !ready,
                            "Start task"
                        }
                    }
                }
            }

            if !is_empty || is_loading {
                div { class: "kit-tb__bar",
                    div { class: "kit-tb__tabs", role: "tablist", "aria-label": "Show",
                        for option in TaskTab::all() {
                            {
                                let option_key = option.as_str();
                                let option_label = option.label();
                                let option_count = tab_counts_list[option as usize];
                                rsx! {
                                    button {
                                        key: "{option_key}",
                                        r#type: "button",
                                        role: "tab",
                                        class: "kit-tb__tab",
                                        "aria-selected": (tab() == option).to_string(),
                                        tabindex: if tab() == option { "0" } else { "-1" },
                                        "data-tab": "{option_key}",
                                        onclick: move |_| tab.set(option),
                                        "{option_label} "
                                        span { class: "kit-tb__count", "{option_count}" }
                                    }
                                }
                            }
                        }
                    }
                    details { class: "kit-tb__filter",
                        summary { "Agents" }
                        div { class: "kit-tb__filter-list",
                            for agent in agents.iter() {
                                {
                                    let agent_id = agent.id.clone();
                                    let agent_label = agent.entry_name.clone();
                                    let checked = agent_filter().contains(&agent_id);
                                    rsx! {
                                        label { key: "{agent_id}", class: "kit-tb__check",
                                            input {
                                                r#type: "checkbox",
                                                checked,
                                                onchange: move |event: FormEvent| {
                                                    let mut next = agent_filter();
                                                    next.retain(|v| v != &agent_id);
                                                    if event.checked() {
                                                        next.push(agent_id.clone());
                                                    }
                                                    agent_filter.set(next);
                                                },
                                            }
                                            "{agent_label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if is_loading {
                div { class: "kit-tb__columns", "data-view": "board",
                    for n in 0..3 {
                        div { key: "{n}", class: "kit-tb__col",
                            span { class: "kit-tb__ph kit-tb__ph--head", "aria-hidden": "true" }
                            span { class: "kit-tb__ph kit-tb__ph--card", "aria-hidden": "true" }
                            span { class: "kit-tb__ph kit-tb__ph--card", "aria-hidden": "true" }
                        }
                    }
                }
            } else if !is_empty {
                if view() == BoardView::Board {
                    div { class: "kit-tb__columns", "data-view": "board",
                        for column in COLUMNS {
                            if tab().admits(column_probe(column)) {
                                {
                                    let column_key = column.as_str();
                                    let heading_id = format!("kit-tb-{uid}-{column_key}");
                                    let column_title = column.title();
                                    let column_count = counts_in(&board, column);
                                    let column_empty = column.empty_text();
                                    let cards: Vec<(TaskRecord, TaskState)> = shown
                                        .iter()
                                        .filter(|(_, state)| state.column() == column)
                                        .cloned()
                                        .collect();
                                    rsx! {
                                        section {
                                            key: "{column_key}",
                                            class: "kit-tb__col",
                                            "data-column": "{column_key}",
                                            "aria-labelledby": "{heading_id}",
                                            h3 { class: "kit-tb__col-head", id: "{heading_id}",
                                                "{column_title} "
                                                span { class: "kit-tb__count", "{column_count}" }
                                            }
                                            for (task, shown_state) in cards {
                                                {
                                                    let card_key = task.id.clone();
                                                    let card_open = open_id().as_deref() == Some(task.id.as_str());
                                                    let card_agent = agent_name(&agents, &task.agent_id);
                                                    let card_hue = agent_hue(&agents, &task.agent_id);
                                                    let card_changing = shown_state != task.task_state;
                                                    let card_dom = card_dom_id(uid, &task.id);
                                                    rsx! {
                                                        TaskCard {
                                                            key: "{card_key}",
                                                            agent_name: card_agent,
                                                            agent_hue: card_hue,
                                                            now_ms: now,
                                                            is_open: card_open,
                                                            changing: card_changing,
                                                            dom_id: card_dom,
                                                            on_pick: move |id: String| {
                                                                open_id.set(Some(id));
                                                                picked.set(true);
                                                            },
                                                            task,
                                                        }
                                                    }
                                                }
                                            }
                                            if column_count == 0 {
                                                p { class: "kit-tb__none", "{column_empty}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    ul { class: "kit-tb__list", "data-view": "list",
                        for (task, shown_state) in shown.iter().filter(|(_, state)| tab().admits(*state)).cloned() {
                            {
                                let row_key = task.id.clone();
                                let row_open = open_id().as_deref() == Some(task.id.as_str());
                                let row_agent = agent_name(&agents, &task.agent_id);
                                let row_hue = agent_hue(&agents, &task.agent_id);
                                let row_changing = shown_state != task.task_state;
                                let row_dom = card_dom_id(uid, &task.id);
                                rsx! {
                                    li { key: "{row_key}", class: "kit-tb__row",
                                        TaskCard {
                                            agent_name: row_agent,
                                            agent_hue: row_hue,
                                            now_ms: now,
                                            is_open: row_open,
                                            changing: row_changing,
                                            dom_id: row_dom,
                                            on_pick: move |id: String| {
                                                open_id.set(Some(id));
                                                picked.set(true);
                                            },
                                            task,
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(task) = open_task.clone() {
                    {
                        let state_key = task.task_state.as_str();
                        let state_label = task.task_state.label();
                        let back_target = card_dom_id(uid, &task.id);
                        let title = task.entry_title.clone();
                        let owner = agent_name(&agents, &task.agent_id);
                        let sources = task.source_count.unwrap_or(0);
                        let tools = task.tool_count.unwrap_or(0);
                        let time = duration_text(live_seconds(&task, now));
                        let description = task.entry_description.clone().unwrap_or_default();
                        let actions = actions_for(task.task_state);
                        rsx! {
                            aside { class: "kit-tb__panel", "aria-label": "Open task", "data-picked": picked().then_some("true"),
                                div { class: "kit-tb__panel-head",
                                    button {
                                        r#type: "button",
                                        class: "kit-tb__back",
                                        onclick: move |_| {
                                            picked.set(false);
                                            open_id.set(None);
                                            focus_after_frame(&back_target);
                                        },
                                        "Back"
                                    }
                                    span { class: "kit-tb__state", "data-state": "{state_key}", "{state_label}" }
                                    h3 { class: "kit-tb__panel-title", "{title}" }
                                    p { class: "kit-tb__meta", "{owner}" }
                                }
                                if !description.is_empty() {
                                    p { class: "kit-tb__desc", "{description}" }
                                }
                                dl { class: "kit-tb__facts",
                                    dt { "Sources" }
                                    dd { "{sources}" }
                                    dt { "Tools" }
                                    dd { "{tools}" }
                                    dt { "Time" }
                                    dd { "{time}" }
                                }
                                if !task.step_records.is_empty() {
                                    ol { class: "kit-tb__steps",
                                        for (index, step) in task.step_records.iter().enumerate() {
                                            {
                                                let step_label = step.entry_label.clone();
                                                let step_time = step.time_label.clone().unwrap_or_default();
                                                rsx! {
                                                    li { key: "{index}",
                                                        span { "{step_label}" }
                                                        if !step_time.is_empty() {
                                                            span { class: "kit-tb__meta", "{step_time}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                div { class: "kit-tb__panel-actions",
                                    for action in actions {
                                        {
                                            let action_key = action.label();
                                            rsx! {
                                                PanelAction {
                                                    key: "{action_key}",
                                                    action,
                                                    target: task.clone(),
                                                    on_approve,
                                                    on_reject,
                                                    on_retry,
                                                    on_open,
                                                }
                                            }
                                        }
                                    }
                                }
                                if is_running_open {
                                    p { class: "kit-tb__meta", "The agent is still working. Open the run to follow it." }
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

#[component]
fn ViewGlyph(view: BoardView) -> Element {
    rsx! {
        svg { class: "kit-tb__glyph", "aria-hidden": "true", view_box: "0 0 16 16", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "1.5",
            if view == BoardView::Board {
                rect { x: "2", y: "2", width: "4", height: "12" }
                rect { x: "10", y: "2", width: "4", height: "12" }
            } else {
                path { d: "M3 4h10M3 8h10M3 12h10" }
            }
        }
    }
}

#[component]
fn TaskCard(
    task: TaskRecord,
    agent_name: String,
    agent_hue: Option<String>,
    now_ms: u64,
    is_open: bool,
    changing: bool,
    dom_id: String,
    on_pick: EventHandler<String>,
) -> Element {
    let time = duration_text(live_seconds(&task, now_ms));
    let state = task.task_state;
    let state_key = state.as_str();
    let state_label = state.label();
    let hue = match agent_hue.as_deref() {
        Some(token @ ("ok" | "warn" | "fault" | "accent")) => token.to_string(),
        _ => "neutral".to_string(),
    };
    let name = format!("{}, {}, {}", task.entry_title, state_label, agent_name);
    let mark = initial_mark(&agent_name);
    let title = task.entry_title.clone();
    let description = task.entry_description.clone().unwrap_or_default();
    let id = task.id.clone();
    let is_live = state == TaskState::Running;
    rsx! {
        button {
            id: "{dom_id}",
            r#type: "button",
            class: "kit-tb__card",
            "data-state": "{state_key}",
            "data-changing": changing.then_some("true"),
            "aria-label": "{name}",
            "aria-current": is_open.then_some("true"),
            onclick: move |_| on_pick.call(id.clone()),
            span {
                class: "kit-tb__mark",
                "data-hue": "{hue}",
                "data-live": is_live.then_some("true"),
                "aria-hidden": "true",
                "{mark}"
            }
            span { class: "kit-tb__card-body",
                span { class: "kit-tb__card-title", "{title}" }
                if !description.is_empty() {
                    span { class: "kit-tb__card-desc", "{description}" }
                }
                span { class: "kit-tb__card-foot",
                    span { class: "kit-tb__state", "data-state": "{state_key}", "{state_label}" }
                    span { class: "kit-tb__meta", "{time}" }
                }
            }
        }
    }
}

#[component]
fn PanelAction(
    action: TaskAction,
    target: TaskRecord,
    on_approve: Option<EventHandler<TaskRecord>>,
    on_reject: Option<EventHandler<TaskRecord>>,
    on_retry: Option<EventHandler<TaskRecord>>,
    on_open: Option<EventHandler<TaskRecord>>,
) -> Element {
    let handler = match action {
        TaskAction::Approve => on_approve,
        TaskAction::Reject => on_reject,
        TaskAction::Retry => on_retry,
        TaskAction::Open => on_open,
    };
    let class = match action {
        TaskAction::Reject => "kit-tb__ghost",
        _ => "kit-tb__new",
    };
    let attr = action.label().to_lowercase();
    let label = action.label();
    rsx! {
        button {
            r#type: "button",
            class: "{class}",
            "data-action": "{attr}",
            onclick: move |_| {
                if let Some(h) = handler {
                    h.call(target.clone());
                }
            },
            "{label}"
        }
    }
}
