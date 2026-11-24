//! Autonomous run workspace: one autonomous run, its plan and activity, the approval it waits
//! on, the draft it wrote and the files it touched. The agent pauses at every approval, so the
//! person decides before anything is sent, paid or published.
//!
//! Design choice: the plan is `EditableStepPlan` and the draft is `InlineEditReview`, both from
//! the Phase 1 sets. The approval card is this module's own, because its wording is Approve and Deny.
//! Design choice: the approval card puts Deny before Approve in tab order, so a stray Enter denies
//! rather than approves.
//! Design choice: Stop asks for confirmation inline, not in a dialog.
//! Design choice: the right pane holds one fixed tab, Needs your input. The approval card belongs
//! to the step in `Waiting` phase and shows there, whichever file tab was open.
//! Design choice: `on_done` fires once, when every step is done, from an effect guarded by a flag.
//! Design choice: the plan's rename and move edits stay in the plan component; `on_edit` is not
//! forwarded, so the host's step list is unchanged by them.
//! Design choice: `auto_play` runs the simulation inside the component. One step finishes every
//! `SIM_STEP_MS`; the run stops at each approval until it is answered. Deny and Stop end the run.
//! Without `auto_play` the host sets the phases.
//! Design choice: "Accept all" and "Reject all" above the draft settle it in place; the diff is
//! replaced by the chosen text.
//! Browser checks: Stop asks first and calls `on_stop` only after confirming; Approve calls
//! `on_approve` and Deny calls `on_deny`; a file opens as its own tab; the simulation pauses at the
//! approval; no horizontal scroll at 320 px.

use std::cmp::Ordering;

use dioxus::prelude::*;

use crate::agents::editable_step_plan::{EditableStepPlan, PlanPhase, PlanStep, StepState};
use crate::components::inline_edit_review::{Decision, InlineEditReview};
use crate::internal::{KIT_CSS, focus_after_frame, sleep, use_now_ms};
use crate::screens::agent_task_list::ago_text;

const CSS: Asset = asset!("/assets/autonomous-run-workspace.css");

/// Time one simulated step takes, in milliseconds.
const SIM_STEP_MS: u32 = 1200;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunState {
    Running,
    Waiting,
    Done,
    Stopped,
    Failed,
}

impl RunState {
    fn label(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Waiting => "Waiting for you",
            Self::Done => "Done",
            Self::Stopped => "Stopped",
            Self::Failed => "Failed",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Done => "done",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
        }
    }

    fn is_live(self) -> bool {
        matches!(self, Self::Running | Self::Waiting)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepPhase {
    Done,
    Running,
    Pending,
    Waiting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallState {
    Running,
    Done,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Risk {
    Low,
    Medium,
    High,
}

impl Risk {
    fn label(self) -> &'static str {
        match self {
            Self::Low => "Low risk",
            Self::Medium => "Medium risk",
            Self::High => "High risk",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActivityEntry {
    pub at_ms: u64,
    pub entry_text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunRecord {
    pub id: String,
    pub entry_title: String,
    pub agent_name: String,
    pub started_at_ms: u64,
    pub run_state: RunState,
    pub elapsed_seconds: u64,
    pub use_count: u32,
    pub cost_label: String,
    pub activity_list: Vec<ActivityEntry>,
}

/// Activity entries, newest first.
pub fn activity_newest_first(entries: &[ActivityEntry]) -> Vec<ActivityEntry> {
    let mut sorted = entries.to_vec();
    sorted.sort_by_key(|e| std::cmp::Reverse(e.at_ms));
    sorted
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolCall {
    pub entry_name: String,
    pub summary: String,
    pub args_data: String,
    pub result_data: Option<String>,
    pub duration_ms: u64,
    pub call_state: CallState,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FactRow {
    pub entry_label: String,
    pub entry_value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Approval {
    pub entry_title: String,
    pub entry_description: String,
    pub risk_level: Risk,
    pub fact_rows: Vec<FactRow>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunStep {
    pub id: String,
    pub entry_title: String,
    pub detail_text: Option<String>,
    pub tool_name: String,
    pub step_phase: StepPhase,
    pub tool_call_list: Vec<ToolCall>,
    pub approval_request: Option<Approval>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DraftRecord {
    pub original_text: String,
    pub suggested_text: String,
    /// Caption above the diff, for example "Reminder email · 9 changes".
    pub entry_label: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    Document,
    Sheet,
    Code,
}

impl FileKind {
    fn word(self) -> &'static str {
        match self {
            Self::Document => "Document",
            Self::Sheet => "Sheet",
            Self::Code => "Code",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FileEntry {
    pub name: String,
    pub kind: FileKind,
    pub body: String,
}

/// The plan component's step for a run step.
pub fn plan_step(step: &RunStep) -> PlanStep {
    let state = match step.step_phase {
        StepPhase::Done => StepState::Succeeded,
        StepPhase::Running | StepPhase::Waiting => StepState::InProgress,
        StepPhase::Pending => StepState::Pending,
    };
    PlanStep {
        step_key: step.id.clone(),
        step_title: step.entry_title.clone(),
        step_detail: step.detail_text.clone(),
        tool_label: Some(step.tool_name.clone()),
        step_state: state,
        measured_ms: None,
        error_text: None,
    }
}

/// Steps that have not started can be moved, renamed or removed.
pub fn is_movable(step: &RunStep) -> bool {
    step.step_phase == StepPhase::Pending
}

/// The steps in the order the person last set: `keys` first, then any step the
/// order does not name, in host order.
pub fn in_plan_order(steps: &[RunStep], keys: Option<&[String]>) -> Vec<RunStep> {
    let Some(keys) = keys else {
        return steps.to_vec();
    };
    let mut out: Vec<RunStep> = keys
        .iter()
        .filter_map(|key| steps.iter().find(|s| &s.id == key).cloned())
        .collect();
    out.extend(steps.iter().filter(|s| !keys.contains(&s.id)).cloned());
    out
}

/// `1 of 5` for the progress tile.
pub fn progress_text(steps: &[RunStep]) -> String {
    let done = steps
        .iter()
        .filter(|s| s.step_phase == StepPhase::Done)
        .count();
    format!("{done} of {}", steps.len())
}

/// Every step is done, so the run is complete.
pub fn all_done(steps: &[RunStep]) -> bool {
    !steps.is_empty() && steps.iter().all(|s| s.step_phase == StepPhase::Done)
}

/// Applies the plan component's edits to the host's steps. The order and the set
/// of steps follow `edited`; a title changes only while a step has not started;
/// steps missing from `edited` were removed.
pub fn apply_plan_edit(steps: &[RunStep], edited: &[PlanStep]) -> Vec<RunStep> {
    edited
        .iter()
        .filter_map(|plan| {
            let original = steps.iter().find(|s| s.id == plan.step_key)?;
            let mut step = original.clone();
            if is_movable(original) {
                step.entry_title = plan.step_title.clone();
            }
            Some(step)
        })
        .collect()
}

/// Live-region text while an approval is pending.
pub fn waiting_text(steps: &[RunStep]) -> String {
    steps
        .iter()
        .filter(|s| s.step_phase == StepPhase::Waiting)
        .find_map(|s| s.approval_request.as_ref())
        .map(|a| format!("Waiting for your approval: {}", a.entry_title))
        .unwrap_or_default()
}

/// Step phases while the simulation plays: the first `done` steps are done, the
/// step at `done` has phase `at_done`, and the rest are pending.
pub fn simulate(steps: &[RunStep], done: usize, at_done: StepPhase) -> Vec<RunStep> {
    steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            let phase = match index.cmp(&done) {
                Ordering::Less => StepPhase::Done,
                Ordering::Equal => at_done,
                Ordering::Greater => StepPhase::Pending,
            };
            RunStep {
                step_phase: phase,
                ..step.clone()
            }
        })
        .collect()
}

/// Elapsed time as `m:ss`. While live, counts from the host start on `clock_ms`.
/// A `started_at_ms` of 0 means the host gave no start, so the reported seconds stand.
pub fn elapsed_text(run: &RunRecord, now_ms: u64) -> String {
    let seconds = if run.run_state.is_live() && run.started_at_ms > 0 && now_ms > run.started_at_ms
    {
        (now_ms - run.started_at_ms) / 1000
    } else {
        run.elapsed_seconds
    };
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// The run's plan, approval, draft and files. Approve and Deny call back with the
/// step id; Stop asks first and then calls `on_stop`.
#[component]
pub fn AutonomousRunWorkspace(
    #[props(default)] run_record: Option<RunRecord>,
    #[props(default)] step_list: Vec<RunStep>,
    #[props(default)] draft_record: Option<DraftRecord>,
    #[props(default)] file_map: Vec<FileEntry>,
    #[props(default)] app_mark_list: Vec<String>,
    #[props(default)] auto_play: bool,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] clock_ms: Option<u64>,
    #[props(default)] on_approve: Option<EventHandler<String>>,
    #[props(default)] on_deny: Option<EventHandler<String>>,
    #[props(default)] on_accept_draft: Option<EventHandler<()>>,
    #[props(default)] on_reject_draft: Option<EventHandler<()>>,
    #[props(default)] on_stop: Option<EventHandler<()>>,
    #[props(default)] on_done: Option<EventHandler<()>>,
    #[props(default)] on_plan_edit: Option<EventHandler<Vec<RunStep>>>,
) -> Element {
    let now = use_now_ms(clock_ms, 1000);
    let mut stop_asked = use_signal(|| false);
    let mut stopped = use_signal(|| false);
    let mut open_files = use_signal(Vec::<String>::new);
    let mut active_file = use_signal(|| None::<String>);
    let mut finished = use_signal(|| false);
    let mut fired = use_signal(|| false);
    let mut plan_order = use_signal(|| None::<Vec<String>>);
    let mut sim_tick = use_signal(|| 0usize);
    let mut sim_blocked = use_signal(|| false);
    let mut approved = use_signal(Vec::<String>::new);
    let mut draft_choice = use_signal(|| None::<Decision>);

    use_hook(move || {
        if auto_play {
            spawn(async move {
                loop {
                    sleep(SIM_STEP_MS).await;
                    if !*sim_blocked.peek() {
                        sim_tick += 1;
                    }
                }
            });
        }
    });

    let len = step_list.len();
    let sim_done = if auto_play { sim_tick().min(len) } else { 0 };
    let blocker = step_list
        .iter()
        .position(|s| s.approval_request.is_some() && !approved().contains(&s.id));
    let sim_waiting = auto_play && !stopped() && blocker == Some(sim_done);
    let sim_blocked_now = auto_play && (stopped() || sim_done >= len || sim_waiting);
    if *sim_blocked.peek() != sim_blocked_now {
        sim_blocked.set(sim_blocked_now);
    }
    let steps: Vec<RunStep> = if auto_play {
        let at_done = if stopped() {
            StepPhase::Pending
        } else if sim_waiting {
            StepPhase::Waiting
        } else {
            StepPhase::Running
        };
        simulate(&step_list, sim_done, at_done)
    } else {
        step_list.clone()
    };
    let edit_base = steps.clone();

    let done_now = all_done(&steps);
    if *finished.peek() != done_now {
        finished.set(done_now);
    }
    use_effect(move || {
        if finished() && !*fired.peek() {
            fired.set(true);
            if let Some(handler) = on_done {
                handler.call(());
            }
        }
    });

    let run_state = if stopped() {
        RunState::Stopped
    } else if auto_play {
        if len > 0 && sim_done >= len {
            RunState::Done
        } else if sim_waiting {
            RunState::Waiting
        } else {
            RunState::Running
        }
    } else {
        match &run_record {
            Some(run) => run.run_state,
            None => RunState::Running,
        }
    };
    let ordered = in_plan_order(&steps, plan_order().as_deref());
    let movable_keys: Vec<String> = ordered
        .iter()
        .filter(|s| is_movable(s))
        .map(|s| s.id.clone())
        .collect();
    let plan: Vec<PlanStep> = ordered
        .iter()
        .map(|step| {
            let mut entry = plan_step(step);
            if !step.tool_call_list.is_empty() {
                let calls: Vec<String> = step
                    .tool_call_list
                    .iter()
                    .map(|call| {
                        let word = match call.call_state {
                            CallState::Running => "running",
                            CallState::Done => "done",
                            CallState::Failed => "failed",
                        };
                        format!("{} {word} {} ms", call.entry_name, call.duration_ms)
                    })
                    .collect();
                entry.step_detail = Some(match entry.step_detail.take() {
                    Some(detail) => format!("{detail} · {}", calls.join(" · ")),
                    None => calls.join(" · "),
                });
            }
            entry
        })
        .collect();
    let approval_line = if stopped() {
        String::new()
    } else {
        waiting_text(&steps)
    };
    let progress = progress_text(&steps);
    let call_total: usize = steps.iter().map(|s| s.tool_call_list.len()).sum();
    let pending = steps
        .iter()
        .find(|s| s.step_phase == StepPhase::Waiting && s.approval_request.is_some())
        .filter(|_| !stopped())
        .cloned();
    let plan_phase = if run_state.is_live() {
        PlanPhase::Executing
    } else {
        PlanPhase::Completed
    };
    let state_key = run_state.key();
    let state_label = run_state.label();
    let draft_before = draft_record
        .as_ref()
        .map_or_else(String::new, |d| d.original_text.clone());
    let draft_after = draft_record
        .as_ref()
        .map_or_else(String::new, |d| d.suggested_text.clone());
    let draft_caption = draft_record
        .as_ref()
        .and_then(|d| d.entry_label.clone())
        .unwrap_or_else(|| "Draft changes".to_string());
    let pending_id = pending.as_ref().map_or_else(String::new, |s| s.id.clone());
    let pending_request = pending.as_ref().and_then(|s| s.approval_request.clone());
    let open_file_now = active_file();
    let needs_count = usize::from(pending_request.is_some());
    let activity = run_record
        .as_ref()
        .map_or_else(Vec::new, |run| activity_newest_first(&run.activity_list));

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: "kit-arw",
            "data-loading": is_loading.then_some("true"),
            "aria-busy": is_loading.then_some("true"),
            if is_loading {
                div { class: "kit-arw__ph kit-arw__ph--head", "aria-hidden": "true" }
                div { class: "kit-arw__ph kit-arw__ph--tiles", "aria-hidden": "true" }
                div { class: "kit-arw__ph kit-arw__ph--plan", "aria-hidden": "true" }
            } else if is_empty || run_record.is_none() {
                div { class: "kit-arw__empty",
                    p { "This run has no steps yet." }
                }
            } else if let Some(run) = run_record.clone() {
                {
                    let run_title = run.entry_title.clone();
                    let run_agent = run.agent_name.clone();
                    let run_cost = run.cost_label.clone();
                    let run_elapsed = elapsed_text(&run, now);
                    let run_started = if run.started_at_ms > 0 {
                        format!("{run_agent} · started {}", ago_text(now, run.started_at_ms))
                    } else {
                        run_agent.clone()
                    };
                    let call_text = call_total.to_string();
                    rsx! {
                        header { class: "kit-arw__head",
                            div { class: "kit-arw__title-group",
                                h2 { class: "kit-arw__title", "{run_title}" }
                                p { class: "kit-arw__meta", "{run_started}" }
                                if !app_mark_list.is_empty() {
                                    ul { class: "kit-arw__apps", "aria-label": "Apps this run uses",
                                        for mark in app_mark_list.iter() {
                                            li { key: "{mark}", class: "kit-arw__app", "{mark}" }
                                        }
                                    }
                                }
                            }
                            div { class: "kit-arw__head-actions",
                                span { class: "kit-arw__state", "data-state": "{state_key}", "{state_label}" }
                                if run_state.is_live() && !*stop_asked.read() {
                                    button {
                                        r#type: "button",
                                        class: "kit-arw__ghost",
                                        onclick: move |_| stop_asked.set(true),
                                        "Stop run"
                                    }
                                }
                                if stop_asked() {
                                    span { class: "kit-arw__confirm", role: "group", "aria-label": "Confirm stop",
                                        span { "Stop this run?" }
                                        button {
                                            r#type: "button",
                                            class: "kit-arw__danger",
                                            onclick: move |_| {
                                                stop_asked.set(false);
                                                stopped.set(true);
                                                if let Some(handler) = on_stop {
                                                    handler.call(());
                                                }
                                            },
                                            "Stop"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "kit-arw__ghost",
                                            onclick: move |_| {
                                                stop_asked.set(false);
                                                focus_after_frame("kit-arw-stop");
                                            },
                                            "Keep running"
                                        }
                                    }
                                }
                            }
                        }

                        section { class: "kit-arw__tiles", "aria-label": "Run figures",
                            div { class: "kit-arw__tile",
                                span { class: "kit-arw__label", "Steps done" }
                                span { class: "kit-arw__value", "{progress}" }
                            }
                            div { class: "kit-arw__tile",
                                span { class: "kit-arw__label", "Tool calls" }
                                span { class: "kit-arw__value", "{call_text}" }
                            }
                            div { class: "kit-arw__tile",
                                span { class: "kit-arw__label", "Elapsed" }
                                span { class: "kit-arw__value", "{run_elapsed}" }
                            }
                            div { class: "kit-arw__tile",
                                span { class: "kit-arw__label", "Cost" }
                                span { class: "kit-arw__value", "{run_cost}" }
                            }
                        }

                        div { class: "kit-arw__main",
                            div { class: "kit-arw__col",
                                section { class: "kit-arw__plan", "aria-label": "Plan",
                                    if plan.is_empty() {
                                        p { class: "kit-arw__meta", "No steps yet." }
                                    } else {
                                        p { class: "kit-arw__hint", "Steps that have not started can be renamed, moved or removed." }
                                        EditableStepPlan {
                                            plan_heading: "Plan",
                                            initial_steps: plan.clone(),
                                            plan_steps: Some(plan.clone()),
                                            plan_phase,
                                            movable_keys: Some(movable_keys.clone()),
                                            on_start: move |_: Vec<PlanStep>| {},
                                            on_edit: move |edited: Vec<PlanStep>| {
                                                plan_order.set(Some(edited.iter().map(|s| s.step_key.clone()).collect()));
                                                if let Some(handler) = on_plan_edit {
                                                    handler.call(apply_plan_edit(&edit_base, &edited));
                                                }
                                            },
                                        }
                                    }
                                }

                                section { class: "kit-arw__activity", "aria-label": "Activity",
                                    h3 { class: "kit-arw__section", "Activity" }
                                    if activity.is_empty() {
                                        p { class: "kit-arw__meta", "No activity yet." }
                                    }
                                    ol { class: "kit-arw__activity-list",
                                        for (index, entry) in activity.iter().enumerate() {
                                            {
                                                let entry_key = format!("{index}");
                                                let entry_time = ago_text(now, entry.at_ms);
                                                let entry_text = entry.entry_text.clone();
                                                rsx! { li { key: "{entry_key}", span { class: "kit-arw__meta", "{entry_time}" } span { "{entry_text}" } } }
                                            }
                                        }
                                    }
                                }
                            }

                            aside { class: "kit-arw__pane", "aria-label": "Needs your input and files",
                                div { class: "kit-arw__tabs", role: "tablist", "aria-label": "Work area",
                                    button {
                                        r#type: "button",
                                        role: "tab",
                                        class: "kit-arw__tab",
                                        "aria-selected": (open_file_now.is_none()).to_string(),
                                        onclick: move |_| active_file.set(None),
                                        "Needs your input "
                                        span { class: "kit-arw__count", "{needs_count}" }
                                    }
                                    for name in open_files().into_iter() {
                                        {
                                            let tab_key = name.clone();
                                            let tab_name = name.clone();
                                            let tab_selected = (open_file_now.as_deref() == Some(name.as_str())).to_string();
                                            rsx! {
                                                button {
                                                    key: "{tab_key}",
                                                    r#type: "button",
                                                    role: "tab",
                                                    class: "kit-arw__tab",
                                                    "aria-selected": "{tab_selected}",
                                                    onclick: move |_| active_file.set(Some(tab_name.clone())),
                                                    "{tab_key}"
                                                }
                                            }
                                        }
                                    }
                                    button {
                                        r#type: "button",
                                        class: "kit-arw__tab-add",
                                        disabled: true,
                                        "aria-label": "Add a tab",
                                        "+"
                                    }
                                }

                                if let Some(name) = open_file_now.clone() {
                                    {
                                        let body = file_map
                                            .iter()
                                            .find(|f| f.name == name)
                                            .map(|f| f.body.clone())
                                            .unwrap_or_default();
                                        rsx! {
                                            pre { class: "kit-arw__viewer", "{body}" }
                                        }
                                    }
                                } else {
                                    if let Some(request) = pending_request.clone() {
                                        {
                                            let approval_title = request.entry_title.clone();
                                            let approval_body = request.entry_description.clone();
                                            let risk_key = request.risk_level.key();
                                            let risk_label = request.risk_level.label();
                                            let deny_id = pending_id.clone();
                                            let approve_id = pending_id.clone();
                                            let deny_label = format!("Deny: {approval_title}");
                                            let approve_label = format!("Approve: {approval_title}");
                                            let region_id = format!("kit-arw-approval-{pending_id}");
                                            let heading_id = format!("{region_id}-title");
                                            rsx! {
                                                section { class: "kit-arw__approval", role: "region", "aria-labelledby": "{heading_id}",
                                                    h3 { id: "{heading_id}", class: "kit-arw__section", "{approval_title}" }
                                                    p { class: "kit-arw__meta", "{approval_body}" }
                                                    span { class: "kit-arw__risk", "data-risk": "{risk_key}", "{risk_label}" }
                                                    dl { class: "kit-arw__facts",
                                                        for fact in request.fact_rows.iter().cloned() {
                                                            {
                                                                let fact_key = fact.entry_label.clone();
                                                                let fact_value = fact.entry_value.clone();
                                                                rsx! {
                                                                    dt { key: "{fact_key}", "{fact_key}" }
                                                                    dd { "{fact_value}" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    div { class: "kit-arw__approval-actions",
                                                        button {
                                                            r#type: "button",
                                                            class: "kit-arw__ghost",
                                                            "aria-label": "{deny_label}",
                                                            onclick: move |_| {
                                                                stopped.set(true);
                                                                if let Some(handler) = on_deny {
                                                                    handler.call(deny_id.clone());
                                                                }
                                                            },
                                                            "Deny"
                                                        }
                                                        button {
                                                            r#type: "button",
                                                            class: "kit-arw__primary",
                                                            "aria-label": "{approve_label}",
                                                            onclick: move |_| {
                                                                approved.with_mut(|list| list.push(approve_id.clone()));
                                                                if let Some(handler) = on_approve {
                                                                    handler.call(approve_id.clone());
                                                                }
                                                            },
                                                            "Approve"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        p { class: "kit-arw__meta", "Nothing needs you yet. The run asks here when a step needs your decision." }
                                    }

                                    if draft_record.is_some() {
                                        section { class: "kit-arw__draft", "aria-label": "Draft",
                                            div { class: "kit-arw__block-head",
                                                h3 { class: "kit-arw__section", "Draft" }
                                                p { class: "kit-arw__meta", "{draft_caption}" }
                                            }
                                            div { class: "kit-arw__row-actions",
                                                button {
                                                    r#type: "button",
                                                    class: "kit-arw__ghost",
                                                    disabled: draft_choice().is_some(),
                                                    onclick: move |_| {
                                                        draft_choice.set(Some(Decision::Accepted));
                                                        if let Some(handler) = on_accept_draft {
                                                            handler.call(());
                                                        }
                                                    },
                                                    "Accept all"
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "kit-arw__ghost",
                                                    disabled: draft_choice().is_some(),
                                                    onclick: move |_| {
                                                        draft_choice.set(Some(Decision::Rejected));
                                                        if let Some(handler) = on_reject_draft {
                                                            handler.call(());
                                                        }
                                                    },
                                                    "Reject all"
                                                }
                                            }
                                            if let Some(choice) = draft_choice() {
                                                p { class: "kit-arw__draft-result", "data-choice": if choice == Decision::Accepted { "accepted" } else { "rejected" },
                                                    if choice == Decision::Accepted { "Suggested text accepted." } else { "Original text kept." }
                                                }
                                                p { class: "kit-arw__draft-text",
                                                    {if choice == Decision::Accepted { draft_after.clone() } else { draft_before.clone() }}
                                                }
                                            } else {
                                                InlineEditReview {
                                                    before_text: draft_before.clone(),
                                                    after_text: draft_after.clone(),
                                                    caption_text: draft_caption.clone(),
                                                }
                                            }
                                        }
                                    }

                                    section { class: "kit-arw__files", "aria-label": "Files",
                                        h3 { class: "kit-arw__section", "Files" }
                                        ul { class: "kit-arw__file-list",
                                            for file in file_map.iter().cloned() {
                                                {
                                                    let file_key = file.name.clone();
                                                    let file_kind = file.kind.word();
                                                    let open_label = format!("Open {}", file.name);
                                                    let open_name = file.name.clone();
                                                    rsx! {
                                                        li { key: "{file_key}",
                                                            button {
                                                                r#type: "button",
                                                                class: "kit-arw__file",
                                                                "aria-label": "{open_label}",
                                                                onclick: move |_| {
                                                                    let name = open_name.clone();
                                                                    if !open_files().contains(&name) {
                                                                        open_files.with_mut(|list| list.push(name.clone()));
                                                                    }
                                                                    active_file.set(Some(name));
                                                                },
                                                                span { class: "kit-arw__file-name", "{file_key}" }
                                                                span { class: "kit-arw__meta", "{file_kind}" }
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
            }

            p { class: "kit-sr kit-arw__live", role: "status", "aria-live": "polite",
                span { "{approval_line}" }
                if done_now { span { "Run complete" } }
                if stopped() { span { "Run stopped" } }
            }
        }
    }
}
