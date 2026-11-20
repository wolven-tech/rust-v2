//! Background task inbox: every background job in one place, grouped by what
//! the person has to do about it. A board of three columns, or a list with
//! filter tabs.
//!
//! Design choice: the clock is `clock_ms` when the host supplies one, else the
//! milliseconds since mount. `started_at` is read on that clock. `SystemTime`
//! is not used because it panics on `wasm32-unknown-unknown`.
//! Design choice: a running job without `started_at` counts from mount.
//! Design choice: the container width is measured. Below 720 px the list replaces
//! the board and the layout switch is not rendered, so no card is in the DOM twice.
//! Design choice: stage marks are text and CSS, not icon elements, so the data
//! types derive `Debug`.
//! Browser checks: the clock ticks once a second; a card glides to its new
//! column on a stage change; Escape returns focus; the list filters update;
//! reduced motion removes the glide and the spinner.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, focus_after_frame, next_id, sleep, use_element_width};

/// Below this container width the board is replaced by the list and the switch is hidden.
const NARROW_WIDTH_PX: f64 = 720.0;

const CSS: Asset = asset!("/assets/background-task-inbox.css");

/// Steps shown on an open card: the latest ones, oldest first.
const TIMELINE_LIMIT: usize = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InboxLayout {
    #[default]
    Board,
    List,
}

impl InboxLayout {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Board => "board",
            Self::List => "list",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobStage {
    Running,
    Waiting,
    Done,
    Failed,
}

impl JobStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }

    pub fn column(self) -> JobColumn {
        match self {
            Self::Waiting | Self::Failed => JobColumn::NeedsYou,
            Self::Running => JobColumn::Running,
            Self::Done => JobColumn::Done,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Waiting => "Waiting for you",
            Self::Done => "Done",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobColumn {
    NeedsYou,
    Running,
    Done,
}

impl JobColumn {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InboxFilter {
    All,
    NeedsYou,
    Running,
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobAction {
    Confirm,
    Decline,
    Restart,
    Inspect,
}

impl JobAction {
    fn label(self) -> &'static str {
        match self {
            Self::Confirm => "Confirm",
            Self::Decline => "Decline",
            Self::Restart => "Restart",
            Self::Inspect => "Inspect",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineStep {
    pub label: String,
    pub at: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Job {
    pub id: String,
    pub heading_text: String,
    pub summary_text: Option<String>,
    pub owner_id: String,
    pub stage: JobStage,
    pub started_at: Option<u64>,
    pub elapsed: Option<u64>,
    pub tool_count: Option<u32>,
    pub source_count: Option<u32>,
    /// Oldest first.
    pub timeline: Vec<TimelineStep>,
}

/// A stage change made inside the component, used only when the matching
/// handler is absent. It replaces the job's own clock fields.
#[derive(Clone, Debug, PartialEq)]
pub struct StageOverride {
    pub id: String,
    pub stage: JobStage,
    pub started_at: Option<u64>,
    pub elapsed: Option<u64>,
}

/// A job with its effective stage and its clock in whole seconds.
#[derive(Clone, Debug, PartialEq)]
pub struct InboxRow {
    pub job: Job,
    pub stage: JobStage,
    pub seconds: u64,
}

/// The stage and whole-second clock a job shows at `now_ms`. A running or
/// waiting job counts from `started_at`; a finished one keeps its `elapsed`.
pub fn resolve(job: &Job, overrides: &[StageOverride], now_ms: u64) -> (JobStage, u64) {
    let (stage, started_at, elapsed) = match overrides.iter().find(|o| o.id == job.id) {
        Some(o) => (o.stage, o.started_at, o.elapsed),
        None => (job.stage, job.started_at, job.elapsed),
    };
    let ms = match stage {
        JobStage::Running | JobStage::Waiting => now_ms.saturating_sub(started_at.unwrap_or(0)),
        JobStage::Done | JobStage::Failed => elapsed.unwrap_or(0),
    };
    (stage, ms / 1000)
}

/// The override a card action leaves behind when no handler is supplied.
/// `elapsed_seconds` is the clock the job shows now; decline freezes it.
pub fn override_for(
    job_id: &str,
    elapsed_seconds: u64,
    action: JobAction,
    now_ms: u64,
) -> Option<StageOverride> {
    match action {
        JobAction::Confirm | JobAction::Restart => Some(StageOverride {
            id: job_id.to_string(),
            stage: JobStage::Running,
            started_at: Some(now_ms),
            elapsed: None,
        }),
        JobAction::Decline => Some(StageOverride {
            id: job_id.to_string(),
            stage: JobStage::Failed,
            started_at: None,
            elapsed: Some(elapsed_seconds * 1000),
        }),
        JobAction::Inspect => None,
    }
}

/// Actions a card offers for its stage. Inspect needs `on_inspect` and a
/// running or finished job.
pub fn actions_for(stage: JobStage, has_inspect: bool) -> Vec<JobAction> {
    match stage {
        JobStage::Waiting => vec![JobAction::Confirm, JobAction::Decline],
        JobStage::Failed => vec![JobAction::Restart],
        JobStage::Running | JobStage::Done if has_inspect => vec![JobAction::Inspect],
        JobStage::Running | JobStage::Done => Vec::new(),
    }
}

pub fn in_filter(stage: JobStage, filter: InboxFilter) -> bool {
    match filter {
        InboxFilter::All => true,
        InboxFilter::NeedsYou => stage.column() == JobColumn::NeedsYou,
        InboxFilter::Running => stage.column() == JobColumn::Running,
        InboxFilter::Done => stage.column() == JobColumn::Done,
    }
}

/// `m:ss`, with the hours in front past one hour.
pub fn clock_text(total_seconds: u64) -> String {
    let (h, m, s) = (
        total_seconds / 3600,
        total_seconds / 60 % 60,
        total_seconds % 60,
    );
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// The polite live line: how many jobs need the person and how many run.
pub fn announcement(needs_you: usize, running: usize) -> String {
    format!("{needs_you} waiting on you, {running} running.")
}

/// The last `TIMELINE_LIMIT` steps, oldest first.
pub fn recent_steps(timeline: &[TimelineStep]) -> &[TimelineStep] {
    let start = timeline.len().saturating_sub(TIMELINE_LIMIT);
    &timeline[start..]
}

fn stage_line(stage: JobStage, seconds: u64) -> String {
    match stage {
        JobStage::Failed => stage.label().to_string(),
        _ => format!("{} · {}", stage.label(), clock_text(seconds)),
    }
}

fn toggle_open(list: &mut Vec<String>, id: &str) {
    match list.iter().position(|open| open == id) {
        Some(index) => {
            list.remove(index);
        }
        None => list.push(id.to_string()),
    }
}

/// A set of background jobs, grouped into Needs you, Running and Done.
///
/// With `on_confirm`, `on_decline` or `on_restart` supplied, the component
/// never alters a stage; the host updates the job and the card follows.
/// Without one, the component moves the job itself: confirm runs it, decline
/// fails it, restart runs it again from zero.
#[component]
pub fn BackgroundTaskInbox(
    jobs: Vec<Job>,
    #[props(default = "Agent inbox".to_string(), into)] heading: String,
    #[props(default)] layout: Option<InboxLayout>,
    #[props(default)] initial_layout: InboxLayout,
    #[props(default)] on_layout_change: Option<EventHandler<InboxLayout>>,
    #[props(default)] on_confirm: Option<EventHandler<Job>>,
    #[props(default)] on_decline: Option<EventHandler<Job>>,
    #[props(default)] on_restart: Option<EventHandler<Job>>,
    #[props(default)] on_inspect: Option<EventHandler<Job>>,
    #[props(default)] clock_ms: Option<u64>,
    #[props(default)] extra_class: String,
) -> Element {
    let mut local_layout = use_signal(|| initial_layout);
    let mut overrides = use_signal(Vec::<StageOverride>::new);
    let mut open = use_signal(Vec::<String>::new);
    let mut filter = use_signal(|| InboxFilter::All);
    let mut local_ms = use_signal(|| 0u64);
    use_future(move || async move {
        loop {
            sleep(1000).await;
            local_ms += 1000;
        }
    });

    let now = clock_ms.unwrap_or_else(|| local_ms());
    let current = overrides();
    let rows: Vec<InboxRow> = jobs
        .iter()
        .map(|job| {
            let (stage, seconds) = resolve(job, &current, now);
            InboxRow {
                job: job.clone(),
                stage,
                seconds,
            }
        })
        .collect();
    let members = |column: JobColumn| -> Vec<InboxRow> {
        rows.iter()
            .filter(|row| row.stage.column() == column)
            .cloned()
            .collect()
    };
    let needs_rows = members(JobColumn::NeedsYou);
    let running_rows = members(JobColumn::Running);
    let done_rows = members(JobColumn::Done);
    let needs_count = needs_rows.len();
    let running_count = running_rows.len();
    let done_count = done_rows.len();
    let total = rows.len();
    let active = layout.unwrap_or_else(|| local_layout());
    let has_inspect = on_inspect.is_some();
    let open_ids = open();
    let active_filter = filter();
    let summary = announcement(needs_count, running_count);

    let on_action = move |(job, action): (Job, JobAction)| {
        let handler = match action {
            JobAction::Confirm => on_confirm,
            JobAction::Decline => on_decline,
            JobAction::Restart => on_restart,
            JobAction::Inspect => on_inspect,
        };
        match handler {
            Some(handler) => handler.call(job),
            None => {
                let (_, seconds) = resolve(&job, &overrides.peek(), now);
                if let Some(next) = override_for(&job.id, seconds, action, now) {
                    overrides.with_mut(|list| {
                        list.retain(|o| o.id != next.id);
                        list.push(next);
                    });
                }
            }
        }
    };
    let on_toggle = move |id: String| open.with_mut(|list| toggle_open(list, &id));
    let class = class_list("kit-inbox", &extra_class);
    let root_id = use_hook(|| format!("kit-inbox-root-{}", next_id()));
    let narrow = use_element_width(root_id.clone()).is_some_and(|w| w < NARROW_WIDTH_PX);
    let shown = if narrow { InboxLayout::List } else { active };
    let list_rows: Vec<InboxRow> = rows
        .iter()
        .filter(|row| in_filter(row.stage, active_filter))
        .cloned()
        .collect();
    let tabs = [
        (InboxFilter::All, "All", total),
        (InboxFilter::NeedsYou, "Needs you", needs_count),
        (InboxFilter::Running, "Running", running_count),
        (InboxFilter::Done, "Done", done_count),
    ];

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class, id: "{root_id}", "data-layout": shown.as_str(), "aria-label": "{heading}",
            header { class: "kit-inbox__head",
                p { class: "kit-inbox__crumb", "Agents / {heading}" }
                if !narrow {
                    div { class: "kit-inbox__switch", role: "radiogroup", "aria-label": "Layout",
                        for option in [InboxLayout::Board, InboxLayout::List] {
                            button {
                                r#type: "button",
                                role: "radio",
                                class: "kit-inbox__switch-item",
                                "aria-checked": (active == option).to_string(),
                                onclick: move |_| {
                                    local_layout.set(option);
                                    if let Some(handler) = on_layout_change {
                                        handler.call(option);
                                    }
                                },
                                if option == InboxLayout::Board { "Board" } else { "List" }
                            }
                        }
                    }
                }
            }
            div { class: "kit-inbox__sr", role: "status", "aria-live": "polite", "{summary}" }
            if shown == InboxLayout::Board {
                div { class: "kit-inbox__board",
                    InboxColumn {
                        column: JobColumn::NeedsYou,
                        rows: needs_rows,
                        open_ids: open_ids.clone(),
                        has_inspect,
                        on_toggle,
                        on_action,
                    }
                    InboxColumn {
                        column: JobColumn::Running,
                        rows: running_rows,
                        open_ids: open_ids.clone(),
                        has_inspect,
                        on_toggle,
                        on_action,
                    }
                    InboxColumn {
                        column: JobColumn::Done,
                        rows: done_rows,
                        open_ids: open_ids.clone(),
                        has_inspect,
                        on_toggle,
                        on_action,
                    }
                }
            } else {
                div { class: "kit-inbox__list-wrap",
                    div { class: "kit-inbox__tabs", role: "group", "aria-label": "Filter jobs",
                        for (tab, label, count) in tabs {
                            button {
                                r#type: "button",
                                class: "kit-inbox__tab",
                                "aria-pressed": (active_filter == tab).to_string(),
                                onclick: move |_| filter.set(tab),
                                "{label}"
                                span { class: "kit-inbox__count", "{count}" }
                            }
                        }
                    }
                    ul { class: "kit-inbox__list",
                        for row in list_rows {
                            li {
                                InboxCard {
                                    is_open: open_ids.contains(&row.job.id),
                                    has_inspect,
                                    on_toggle,
                                    on_action,
                                    row,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn InboxColumn(
    column: JobColumn,
    rows: Vec<InboxRow>,
    open_ids: Vec<String>,
    has_inspect: bool,
    on_toggle: EventHandler<String>,
    on_action: EventHandler<(Job, JobAction)>,
) -> Element {
    let key = column.as_str();
    let heading_id = format!("kit-inbox-col-{key}");
    let title = column.title();
    let count = rows.len();
    let empty = column.empty_text();
    rsx! {
        section { class: "kit-inbox__col", "data-column": key, "aria-labelledby": "{heading_id}",
            header { class: "kit-inbox__col-head",
                h3 { id: "{heading_id}", class: "kit-inbox__col-title", "{title}" }
                span { class: "kit-inbox__count", "{count}" }
            }
            if rows.is_empty() {
                p { class: "kit-inbox__empty", "{empty}" }
            }
            for row in rows {
                InboxCard {
                    is_open: open_ids.contains(&row.job.id),
                    has_inspect,
                    on_toggle,
                    on_action,
                    row,
                }
            }
        }
    }
}

#[component]
fn InboxCard(
    row: InboxRow,
    is_open: bool,
    has_inspect: bool,
    on_toggle: EventHandler<String>,
    on_action: EventHandler<(Job, JobAction)>,
) -> Element {
    let job_id = row.job.id.clone();
    let card_id = format!("kit-inbox-card-{job_id}");
    let body_id = format!("{card_id}-body");
    let stage = row.stage;
    let stage_key = stage.as_str();
    let line = stage_line(stage, row.seconds);
    let title = row.job.heading_text.clone();
    let summary = row.job.summary_text.clone();
    let counts: Vec<String> = [
        row.job.tool_count.map(|n| format!("{n} tools")),
        row.job.source_count.map(|n| format!("{n} sources")),
    ]
    .into_iter()
    .flatten()
    .collect();
    let steps: Vec<String> = recent_steps(&row.job.timeline)
        .iter()
        .map(|step| step.label.clone())
        .collect();
    let actions: Vec<(JobAction, &'static str)> = actions_for(stage, has_inspect)
        .into_iter()
        .map(|action| (action, action.label()))
        .collect();
    let esc_id = job_id.clone();
    let esc_card = card_id.clone();
    let toggle_id = job_id.clone();
    let action_job = row.job.clone();
    rsx! {
        article {
            class: "kit-inbox__card",
            "data-stage": stage_key,
            "data-open": is_open.then_some("true"),
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape && is_open {
                    event.prevent_default();
                    on_toggle.call(esc_id.clone());
                    focus_after_frame(&esc_card);
                }
            },
            button {
                id: "{card_id}",
                r#type: "button",
                class: "kit-inbox__toggle",
                "aria-expanded": is_open.to_string(),
                "aria-controls": "{body_id}",
                onclick: move |_| on_toggle.call(toggle_id.clone()),
                span { class: "kit-inbox__title", "{title}" }
                if let Some(text) = summary {
                    span { class: "kit-inbox__summary", "{text}" }
                }
                span { class: "kit-inbox__stage", "data-stage": stage_key,
                    if stage == JobStage::Running {
                        span { class: "kit-inbox__spin", "aria-hidden": "true" }
                    }
                    "{line}"
                }
                if !counts.is_empty() {
                    span { class: "kit-inbox__meta",
                        for text in counts {
                            span { "{text}" }
                        }
                    }
                }
            }
            if is_open {
                div { id: "{body_id}", class: "kit-inbox__body",
                    if !steps.is_empty() {
                        ol { class: "kit-inbox__steps", "aria-label": "Steps",
                            for label in steps {
                                li { "{label}" }
                            }
                        }
                    }
                    if !actions.is_empty() {
                        div { class: "kit-inbox__actions", role: "group", "aria-label": "Actions",
                            for (action, label) in actions {
                                button {
                                    r#type: "button",
                                    class: "kit-inbox__action",
                                    "data-action": label,
                                    onclick: {
                                        let job = action_job.clone();
                                        move |_| on_action.call((job.clone(), action))
                                    },
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
