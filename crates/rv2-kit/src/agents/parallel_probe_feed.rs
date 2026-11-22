//! Parallel probe feed: a swarm of checks running at once. One row per
//! worker shows its progress; the results they produce form a short feed,
//! newest first. A row filters the feed to that worker.
//!
//! Design choice: no wall clock, since `SystemTime` panics on
//! `wasm32-unknown-unknown`. Relative times count from `clock_ms` when the host
//! gives one, else from the newest result.
//! Design choice: the header's severity totals sum each worker's `counts`. The
//! feed is not recounted, so a host that trims results does not change them.
//! Design choice: a worker's state is `failed` only when the host says so; a
//! worker is `done` once `completed` reaches `planned`, else `running`.
//! Design choice: the empty-feed line is "No findings yet." The source has no
//! wording for it.
//! Design choice: Escape has no effect, as the source defines.
//! Design choice: below the narrow breakpoint the step text is hidden too,
//! so a row can stay on one line; the failure label stays.
//! Browser checks: progress bars grow on change; a new result slides in and
//! the rest shift down; the filter chip and worker toggle; narrow rows drop
//! counts and location; reduced motion removes the slide.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/parallel-probe-feed.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::Warning => "Warning",
            Self::Info => "Info",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerState {
    Running,
    Done,
    Failed,
}

impl WorkerState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SeverityCounts {
    pub critical: u32,
    pub warning: u32,
    pub info: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProbeWorker {
    pub id: String,
    pub name: String,
    pub current_step: Option<String>,
    pub completed: u32,
    pub planned: u32,
    pub state: Option<WorkerState>,
    pub counts: SeverityCounts,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FeedResult {
    pub id: String,
    pub worker_id: String,
    pub severity: Severity,
    pub title: String,
    pub location: Option<String>,
    pub found_ms: u64,
}

/// A worker's state: the one the host gave, else `done` once every planned
/// check is complete and `running` before that.
pub fn effective_state(worker: &ProbeWorker) -> WorkerState {
    match worker.state {
        Some(state) => state,
        None if worker.completed >= worker.planned => WorkerState::Done,
        None => WorkerState::Running,
    }
}

/// `(completed, planned)` summed across workers.
pub fn totals(workers: &[ProbeWorker]) -> (u32, u32) {
    workers.iter().fold((0, 0), |(done, planned), w| {
        (done + w.completed, planned + w.planned)
    })
}

/// Severity totals summed across workers.
pub fn severity_totals(workers: &[ProbeWorker]) -> SeverityCounts {
    workers
        .iter()
        .fold(SeverityCounts::default(), |acc, w| SeverityCounts {
            critical: acc.critical + w.counts.critical,
            warning: acc.warning + w.counts.warning,
            info: acc.info + w.counts.info,
        })
}

/// The header's "N of M checks".
pub fn summary_text(completed: u32, planned: u32) -> String {
    format!("{completed} of {planned} checks")
}

/// Fill of a worker's progress bar, 0 to 100.
pub fn progress_percent(completed: u32, planned: u32) -> f64 {
    if planned == 0 {
        return 100.0;
    }
    (f64::from(completed) / f64::from(planned) * 100.0).clamp(0.0, 100.0)
}

/// The feed after a worker filter, capped at `limit`. Input order is kept:
/// the host sends results newest first.
pub fn visible_results(
    results: &[FeedResult],
    filter: Option<&str>,
    limit: usize,
) -> Vec<FeedResult> {
    results
        .iter()
        .filter(|r| filter.is_none_or(|id| r.worker_id == id))
        .take(limit)
        .cloned()
        .collect()
}

/// Clicking the active worker clears the filter; clicking any other sets it.
pub fn toggle_filter(current: Option<&str>, clicked: &str) -> Option<String> {
    if current == Some(clicked) {
        None
    } else {
        Some(clicked.to_string())
    }
}

/// `"Ns ago"`, `"Nm ago"` or `"Nh ago"`; a result from after `clock_ms` reads as now.
pub fn relative_text(found_ms: u64, clock_ms: u64) -> String {
    let secs = clock_ms.saturating_sub(found_ms) / 1000;
    if secs < 60 {
        format!("{secs}s ago")
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else {
        format!("{}h ago", secs / 3600)
    }
}

/// The polite line for a result that has just appeared.
pub fn result_announcement(result: &FeedResult, worker_name: &str) -> String {
    format!(
        "{}: {}, {}.",
        result.severity.label(),
        result.title,
        worker_name
    )
}

fn worker_name(workers: &[ProbeWorker], id: &str) -> String {
    workers
        .iter()
        .find(|w| w.id == id)
        .map_or_else(|| id.to_string(), |w| w.name.clone())
}

/// Stable hue (0..360) per worker id, so a worker keeps its tint across renders.
fn worker_hue(id: &str) -> u32 {
    id.bytes().fold(0u32, |hash, b| {
        hash.wrapping_mul(31).wrapping_add(u32::from(b))
    }) % 360
}

fn monogram(name: &str) -> String {
    name.chars()
        .next()
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_else(|| "?".to_string())
}

#[derive(Clone, Debug, PartialEq)]
struct WorkerRow {
    id: String,
    name: String,
    initial: String,
    hue: u32,
    state_key: &'static str,
    is_failed: bool,
    is_done: bool,
    step: Option<String>,
    pressed: bool,
    muted: bool,
    completed: u32,
    planned: u32,
    percent: String,
    progress_id: String,
    critical: u32,
    warning: u32,
    info: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct ResultRow {
    result: FeedResult,
    title: String,
    worker: String,
    initial: String,
    hue: u32,
    severity_key: &'static str,
    severity_label: &'static str,
    time: String,
    location: Option<String>,
    fresh: bool,
}

/// Worker progress, the findings they produced, and a filter by worker.
#[component]
pub fn ParallelProbeFeed(
    #[props(default = "Verification".to_string(), into)] heading: String,
    workers: Vec<ProbeWorker>,
    results: Vec<FeedResult>,
    #[props(default = 4)] visible_count: usize,
    #[props(default)] clock_ms: Option<u64>,
    on_select_result: EventHandler<FeedResult>,
    #[props(default)] extra_class: String,
) -> Element {
    let uid = use_hook(next_id);
    let heading_id = format!("kit-probe-{uid}-head");
    let feed_id = format!("kit-probe-{uid}-feed");
    let mut filter = use_signal(|| None::<String>);
    let result_ids: Vec<String> = results.iter().map(|r| r.id.clone()).collect();
    let mut seen = use_signal(|| result_ids.clone());
    let mut announce = use_signal(String::new);

    use_effect(use_reactive!(|(result_ids,)| {
        let fresh: Vec<String> = result_ids
            .iter()
            .filter(|id| !seen.peek().contains(id))
            .cloned()
            .collect();
        if let Some(newest) = fresh.first() {
            announce.set(newest.clone());
            seen.with_mut(|ids| ids.extend(fresh.iter().cloned()));
        }
    }));

    let active = filter();
    let (done, planned) = totals(&workers);
    let summary = summary_text(done, planned);
    let severity = severity_totals(&workers);
    let (critical, warning, info) = (severity.critical, severity.warning, severity.info);
    let clock = clock_ms.unwrap_or_else(|| results.iter().map(|r| r.found_ms).max().unwrap_or(0));
    let class = class_list("kit-probe", &extra_class);
    let seen_now = seen();
    let announced_id = announce();
    let line = results
        .iter()
        .find(|r| r.id == announced_id)
        .map(|r| result_announcement(r, &worker_name(&workers, &r.worker_id)))
        .unwrap_or_default();

    let rows: Vec<WorkerRow> = workers
        .iter()
        .map(|w| {
            let state = effective_state(w);
            let pressed = active.as_deref() == Some(w.id.as_str());
            WorkerRow {
                id: w.id.clone(),
                name: w.name.clone(),
                initial: monogram(&w.name),
                hue: worker_hue(&w.id),
                state_key: state.as_str(),
                is_failed: state == WorkerState::Failed,
                is_done: state == WorkerState::Done,
                step: (state == WorkerState::Running)
                    .then(|| w.current_step.clone())
                    .flatten(),
                pressed,
                muted: active.is_some() && !pressed,
                completed: w.completed,
                planned: w.planned,
                percent: format!("{:.1}", progress_percent(w.completed, w.planned)),
                progress_id: format!("kit-probe-{uid}-{}-bar", w.id),
                critical: w.counts.critical,
                warning: w.counts.warning,
                info: w.counts.info,
            }
        })
        .collect();

    let feed: Vec<ResultRow> = visible_results(&results, active.as_deref(), visible_count)
        .into_iter()
        .map(|result| ResultRow {
            title: result.title.clone(),
            worker: worker_name(&workers, &result.worker_id),
            initial: monogram(&worker_name(&workers, &result.worker_id)),
            hue: worker_hue(&result.worker_id),
            severity_key: result.severity.as_str(),
            severity_label: result.severity.label(),
            time: relative_text(result.found_ms, clock),
            location: result.location.clone(),
            fresh: !seen_now.contains(&result.id),
            result,
        })
        .collect();

    let chip = active
        .as_ref()
        .map(|id| (id.clone(), worker_name(&workers, id)));
    let empty = feed.is_empty();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class, "aria-labelledby": "{heading_id}",
            header { class: "kit-probe__bar",
                h2 { id: "{heading_id}", class: "kit-probe__heading", "{heading}" }
                p { class: "kit-probe__summary", "{summary}" }
                ul { class: "kit-probe__totals", "aria-label": "Findings by severity",
                    li { class: "kit-probe__total", "data-severity": "critical",
                        span { class: "kit-probe__count", "{critical}" }
                        span { class: "kit-probe__total-label", "Critical" }
                    }
                    li { class: "kit-probe__total", "data-severity": "warning",
                        span { class: "kit-probe__count", "{warning}" }
                        span { class: "kit-probe__total-label", "Warning" }
                    }
                    li { class: "kit-probe__total", "data-severity": "info",
                        span { class: "kit-probe__count", "{info}" }
                        span { class: "kit-probe__total-label", "Info" }
                    }
                }
            }
            ul { class: "kit-probe__workers", "aria-label": "Workers",
                for WorkerRow {
                    id, name, initial, hue, state_key, is_failed, is_done, step, pressed, muted,
                    completed, planned, percent, progress_id, critical, warning, info,
                } in rows
                {
                    li { class: "kit-probe__worker-item",
                        button {
                            r#type: "button",
                            class: "kit-probe__worker",
                            style: "--kit-probe-hue: {hue};",
                            "data-state": state_key,
                            "data-muted": muted.then_some("true"),
                            "aria-pressed": pressed.to_string(),
                            onclick: move |_| {
                                let next = toggle_filter(filter.peek().as_deref(), &id);
                                filter.set(next);
                            },
                            span { class: "kit-probe__tile", "aria-hidden": "true", "{initial}" }
                            span { class: "kit-probe__name", "{name}" }
                            if is_failed {
                                span { class: "kit-probe__failed", "Failed" }
                            } else if is_done {
                                span { class: "kit-probe__done", "Done" }
                            } else if let Some(step) = step {
                                span { class: "kit-probe__step", "{step}" }
                            }
                            span {
                                class: "kit-probe__bar-track",
                                role: "progressbar",
                                id: "{progress_id}",
                                "aria-label": "{name} checks",
                                "aria-valuemin": "0",
                                "aria-valuemax": "{planned}",
                                "aria-valuenow": "{completed}",
                                span { class: "kit-probe__bar-fill", style: "width: {percent}%" }
                            }
                            span { class: "kit-probe__fraction", "{completed}/{planned}" }
                            span { class: "kit-probe__worker-counts",
                                span { class: "kit-probe__sev", "data-severity": "critical", "{critical} Critical" }
                                span { class: "kit-probe__sev", "data-severity": "warning", "{warning} Warning" }
                                span { class: "kit-probe__sev", "data-severity": "info", "{info} Info" }
                            }
                        }
                    }
                }
            }
            section { class: "kit-probe__feed-wrap", "aria-labelledby": "{feed_id}",
                div { class: "kit-probe__feed-head",
                    h3 { id: "{feed_id}", class: "kit-probe__feed-title", "Latest findings" }
                    if let Some((id, name)) = chip {
                        button {
                            r#type: "button",
                            class: "kit-probe__chip",
                            "data-worker": "{id}",
                            "aria-label": "Remove filter: {name}",
                            onclick: move |_| filter.set(None),
                            "{name}"
                            span { class: "kit-probe__chip-x", "aria-hidden": "true", "×" }
                        }
                    }
                }
                p { class: "kit-probe__sr", role: "status", "aria-live": "polite", "{line}" }
                if empty {
                    p { class: "kit-probe__empty", "No findings yet." }
                }
                ol { class: "kit-probe__feed", "aria-label": "Latest findings",
                    for ResultRow {
                        result, title, worker, initial, hue, severity_key, severity_label, time, location, fresh,
                    } in feed
                    {
                        li { class: "kit-probe__result-item", "data-new": fresh.then_some("true"),
                            button {
                                r#type: "button",
                                class: "kit-probe__result",
                                style: "--kit-probe-hue: {hue};",
                                onclick: move |_| on_select_result.call(result.clone()),
                                span { class: "kit-probe__tile", "aria-hidden": "true", "{initial}" }
                                span { class: "kit-probe__sev-label", "data-severity": severity_key, "{severity_label}" }
                                span { class: "kit-probe__result-title", "{title}" }
                                span { class: "kit-probe__result-worker", "{worker}" }
                                if let Some(location) = location {
                                    code { class: "kit-probe__location", "{location}" }
                                }
                                time { class: "kit-probe__time", "{time}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
