//! Work queue flow: jobs waiting for slots, running in them, and finishing or failing,
//! with the backlog over time. A person can pause intake and change the slot count.
//!
//! Design choice: the component reflects pause but does not enforce it; the host decides
//! which waiting job enters a slot. The slot count is the number of `slot_rows`, changed
//! through `on_slot_count_change`.
//! Design choice: the depth chart is derived from job timestamps (waiting depth at each
//! sample time), so it needs no separate history feed.
//! Design choice: the pinned card is seeded from `open_job_key` and then kept internally;
//! promote is offered only for enqueued jobs.
//! Browser checks: urgent jobs jump the line; bars grow; throttled bars stop with a
//! countdown; the chart scale glides; roving focus through slots then the line; Escape
//! closes the card and returns focus to its job; reduced motion shows everything in place.

use dioxus::prelude::*;

use crate::agents::single_run_status_card::{focus_by_id, format_mm_ss};
use crate::internal::{KIT_CSS, focus_after_frame, next_id, use_now_ms};

const CSS: Asset = asset!("/assets/work-queue-flow.css");

const MINUTE_MS: u64 = 60_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobKind {
    ModelCall,
    ToolCall,
    SubAgent,
    HumanReview,
}

impl JobKind {
    fn word(self) -> &'static str {
        match self {
            Self::ModelCall => "model call",
            Self::ToolCall => "tool call",
            Self::SubAgent => "sub-agent",
            Self::HumanReview => "human review",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobState {
    Enqueued,
    Active,
    Succeeded,
    Errored,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
pub enum Priority {
    Urgent,
    #[default]
    Standard,
    Deferred,
}

impl Priority {
    fn word(self) -> &'static str {
        match self {
            Self::Urgent => "urgent",
            Self::Standard => "standard",
            Self::Deferred => "deferred",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct JobRow {
    pub job_key: String,
    pub job_label: String,
    pub job_kind: JobKind,
    pub job_state: JobState,
    pub enqueued_at_ms: u64,
    pub started_at_ms: Option<u64>,
    pub ended_at_ms: Option<u64>,
    pub slot_key: Option<String>,
    pub lane_priority: Priority,
    pub progress_fraction: Option<f64>,
    pub expected_run_ms: Option<u64>,
    pub attempt_number: u32,
    pub max_attempts: Option<u32>,
    pub detail_text: Option<String>,
    pub job_mark: Option<String>,
    pub error_text: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SlotRow {
    pub slot_key: String,
    pub slot_label: String,
    pub slot_detail: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ThrottleWindow {
    pub from_ms: u64,
    pub until_ms: u64,
    pub reason_label: Option<String>,
}

/// Indices of waiting jobs in line order: priority first, then enqueue time.
pub fn line_order(jobs: &[JobRow]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..jobs.len())
        .filter(|&i| jobs[i].job_state == JobState::Enqueued)
        .collect();
    order.sort_by_key(|&i| (jobs[i].lane_priority, jobs[i].enqueued_at_ms));
    order
}

/// For each slot, the index of the active job in it, if any.
pub fn slot_occupancy(jobs: &[JobRow], slots: &[SlotRow]) -> Vec<Option<usize>> {
    slots
        .iter()
        .map(|slot| {
            jobs.iter().position(|job| {
                job.job_state == JobState::Active
                    && job.slot_key.as_deref() == Some(slot.slot_key.as_str())
            })
        })
        .collect()
}

/// Nearest-rank 95th percentile of waits (start minus enqueue) over started jobs.
pub fn p95_wait_ms(jobs: &[JobRow]) -> Option<u64> {
    let mut waits: Vec<u64> = jobs
        .iter()
        .filter_map(|job| {
            job.started_at_ms
                .map(|s| s.saturating_sub(job.enqueued_at_ms))
        })
        .collect();
    if waits.is_empty() {
        return None;
    }
    waits.sort_unstable();
    let rank = (waits.len() as f64 * 0.95).ceil() as usize;
    waits.get(rank.saturating_sub(1)).copied()
}

/// Succeeded jobs that ended within the minute ending at `now_ms`.
pub fn throughput_per_min(jobs: &[JobRow], now_ms: u64) -> usize {
    let since = now_ms.saturating_sub(MINUTE_MS);
    jobs.iter()
        .filter(|job| job.job_state == JobState::Succeeded)
        .filter(|job| {
            job.ended_at_ms
                .is_some_and(|end| end >= since && end <= now_ms)
        })
        .count()
}

/// The throttle window current at `now_ms`, if any.
pub fn active_throttle(windows: &[ThrottleWindow], now_ms: u64) -> Option<&ThrottleWindow> {
    windows
        .iter()
        .find(|w| w.from_ms <= now_ms && now_ms < w.until_ms)
}

/// Jobs waiting at time `t`: enqueued by then and not yet started.
pub fn depth_at(jobs: &[JobRow], t: u64) -> usize {
    jobs.iter()
        .filter(|job| job.enqueued_at_ms <= t && job.started_at_ms.is_none_or(|s| s > t))
        .count()
}

/// Waiting depth at `samples` evenly spaced times across `[from_ms, to_ms]`.
pub fn depth_series(
    jobs: &[JobRow],
    from_ms: u64,
    to_ms: u64,
    samples: usize,
) -> Vec<(u64, usize)> {
    if samples < 2 {
        return vec![(to_ms, depth_at(jobs, to_ms))];
    }
    let span = to_ms.saturating_sub(from_ms);
    (0..samples)
        .map(|i| {
            let t = from_ms + span * i as u64 / (samples as u64 - 1);
            (t, depth_at(jobs, t))
        })
        .collect()
}

/// Slot count after a step, within bounds.
pub fn step_slots(count: usize, delta: i32, min: usize, max: usize) -> usize {
    let next = count as i64 + delta as i64;
    next.clamp(min as i64, max as i64) as usize
}

/// Next roving-focus index for a navigation key over `len` items.
pub fn next_index(len: usize, current: usize, key: &str) -> Option<usize> {
    if len == 0 {
        return None;
    }
    match key {
        "ArrowRight" | "ArrowDown" => Some((current + 1).min(len - 1)),
        "ArrowLeft" | "ArrowUp" => Some(current.saturating_sub(1)),
        "Home" => Some(0),
        "End" => Some(len - 1),
        _ => None,
    }
}

/// Text equivalent of the depth chart.
pub fn chart_summary(
    series: &[(u64, usize)],
    backlog_line: Option<usize>,
    throttles: usize,
) -> String {
    let max = series.iter().map(|(_, d)| *d).max().unwrap_or(0);
    let mean = if series.is_empty() {
        0.0
    } else {
        series.iter().map(|(_, d)| *d as f64).sum::<f64>() / series.len() as f64
    };
    let now = series.last().map(|(_, d)| *d).unwrap_or(0);
    let line = backlog_line
        .map(|b| format!(", backlog line {b}"))
        .unwrap_or_default();
    format!("Backlog {now}, maximum {max}, mean {mean:.1}{line}, {throttles} throttled intervals")
}

#[derive(Clone, PartialEq, Debug)]
struct JobView {
    key: String,
    id: String,
    label: String,
    kind: &'static str,
    state: &'static str,
    time: String,
    position: Option<usize>,
    priority: &'static str,
    attempt: u32,
    fill: String,
    throttled: bool,
    mark: Option<String>,
    tab: &'static str,
    aria: String,
}

#[component]
pub fn WorkQueueFlow(
    #[props(default = "Queue".to_string(), into)] queue_title: String,
    #[props(default)] queue_name: Option<String>,
    #[props(default)] job_rows: Vec<JobRow>,
    #[props(default)] slot_rows: Vec<SlotRow>,
    #[props(default)] clock_ms: Option<u64>,
    #[props(default)] throttle_windows: Vec<ThrottleWindow>,
    #[props(default)] is_paused: Option<bool>,
    #[props(default)] on_paused_change: Option<EventHandler<bool>>,
    #[props(default)] on_slot_count_change: Option<EventHandler<usize>>,
    #[props(default = 1)] min_slots: usize,
    #[props(default = 8)] max_slots: usize,
    #[props(default)] backlog_line: Option<usize>,
    #[props(default = 300_000)] chart_span_ms: u64,
    #[props(default)] open_job_key: Option<String>,
    #[props(default)] on_open_change: Option<EventHandler<Option<String>>>,
    #[props(default)] on_promote: Option<EventHandler<JobRow>>,
    #[props(default)] on_withdraw: Option<EventHandler<JobRow>>,
    #[props(default)] class_name: String,
) -> Element {
    let id = use_hook(|| format!("kit-wqf-{}", next_id()));
    let now = use_now_ms(clock_ms, 1000);
    let mut pinned = use_signal(|| open_job_key.clone());
    let mut paused_local = use_signal(|| false);
    let mut focus = use_signal(|| 0_usize);

    let paused = is_paused.unwrap_or_else(|| paused_local());
    let occupancy = slot_occupancy(&job_rows, &slot_rows);
    let waiting_order = line_order(&job_rows);
    let active_jobs: Vec<usize> = occupancy.iter().flatten().copied().collect();
    let order: Vec<usize> = active_jobs
        .iter()
        .chain(waiting_order.iter())
        .copied()
        .collect();
    let focused = focus().min(order.len().saturating_sub(1));
    let throttle = active_throttle(&throttle_windows, now).cloned();
    let throttled = throttle.is_some();
    let completed = job_rows
        .iter()
        .filter(|j| j.job_state == JobState::Succeeded)
        .count();
    let errored = job_rows
        .iter()
        .filter(|j| j.job_state == JobState::Errored)
        .count();
    let throughput = throughput_per_min(&job_rows, now);
    let p95 = p95_wait_ms(&job_rows)
        .map(format_mm_ss)
        .unwrap_or_else(|| "—".to_string());
    let slots = slot_rows.len();
    let at_min = slots <= min_slots;
    let at_max = slots >= max_slots;

    let to = now;
    let from = now.saturating_sub(chart_span_ms);
    let series = depth_series(&job_rows, from, to, 48);
    let max_depth = series
        .iter()
        .map(|(_, d)| *d)
        .chain(backlog_line)
        .max()
        .unwrap_or(1)
        .max(1);
    let points: String = series
        .iter()
        .enumerate()
        .map(|(i, (_, d))| {
            let x = i as f64 / (series.len().max(2) - 1) as f64 * 100.0;
            let y = 100.0 - *d as f64 / max_depth as f64 * 90.0;
            format!("{x:.2},{y:.2}")
        })
        .collect::<Vec<_>>()
        .join(" ");
    let backlog_y = backlog_line.map(|b| 100.0 - b as f64 / max_depth as f64 * 90.0);
    let summary = chart_summary(
        &series,
        backlog_line,
        throttle_windows
            .iter()
            .filter(|w| w.until_ms > from && w.from_ms < to)
            .count(),
    );

    let job_view = |index: usize, position: Option<usize>| -> (String, JobView) {
        let job = &job_rows[index];
        let waited_or_ran = match job.job_state {
            JobState::Enqueued => now.saturating_sub(job.enqueued_at_ms),
            JobState::Active => now.saturating_sub(job.started_at_ms.unwrap_or(job.enqueued_at_ms)),
            _ => job
                .ended_at_ms
                .unwrap_or(now)
                .saturating_sub(job.started_at_ms.unwrap_or(job.enqueued_at_ms)),
        };
        let time = format_mm_ss(waited_or_ran);
        let pos_word = position
            .map(|p| format!(", position {}", p + 1))
            .unwrap_or_default();
        let state = match job.job_state {
            JobState::Enqueued => "enqueued",
            JobState::Active => "active",
            JobState::Succeeded => "succeeded",
            JobState::Errored => "errored",
        };
        let aria = format!(
            "{}, {}, {} {}{}, {} priority, attempt {}",
            job.job_label,
            job.job_kind.word(),
            time,
            if job.job_state == JobState::Enqueued {
                "waiting"
            } else {
                "running"
            },
            pos_word,
            job.lane_priority.word(),
            job.attempt_number
        );
        let fill = match (job.progress_fraction, job.expected_run_ms) {
            (Some(p), _) => format!("{:.1}%", p.clamp(0.0, 1.0) * 100.0),
            (None, Some(expected)) if expected > 0 => {
                let started = job.started_at_ms.unwrap_or(now);
                format!(
                    "{:.1}%",
                    (now.saturating_sub(started) as f64 / expected as f64).clamp(0.0, 1.0) * 100.0
                )
            }
            _ => "0%".to_string(),
        };
        let tab = if index == order.get(focused).copied().unwrap_or(usize::MAX) {
            "0"
        } else {
            "-1"
        };
        (
            job.job_key.clone(),
            JobView {
                key: job.job_key.clone(),
                id: format!("{id}-job-{}", job.job_key),
                label: job.job_label.clone(),
                kind: job.job_kind.word(),
                state,
                time,
                position,
                priority: job.lane_priority.word(),
                attempt: job.attempt_number,
                fill,
                throttled,
                mark: job.job_mark.clone(),
                tab,
                aria,
            },
        )
    };
    let slot_views: Vec<(String, String, String, Option<(String, JobView)>)> = slot_rows
        .iter()
        .enumerate()
        .map(|(s, slot)| {
            let job = occupancy[s].map(|index| job_view(index, None));
            (
                slot.slot_key.clone(),
                slot.slot_label.clone(),
                slot.slot_detail.clone().unwrap_or_default(),
                job,
            )
        })
        .collect();
    let line_views: Vec<(String, JobView)> = waiting_order
        .iter()
        .enumerate()
        .map(|(p, &index)| job_view(index, Some(p)))
        .collect();
    let pinned_key = pinned();
    let pinned_job: Option<JobRow> = pinned_key
        .as_ref()
        .and_then(|key| job_rows.iter().find(|j| &j.job_key == key).cloned());
    let pinned_promotable = pinned_job
        .as_ref()
        .is_some_and(|j| j.job_state == JobState::Enqueued);
    let class = format!("kit-wqf {class_name}");
    let root_id = id.clone();
    let flow_id = id.clone();
    let heading_id = format!("{id}-heading");
    let card_id = format!("{id}-card");
    let waiting_count = waiting_order.len();
    let active_count = active_jobs.len();
    let throughput_text = format!("{throughput}/min");
    let stat_waiting = format!("{waiting_count}");
    let stat_active = format!("{active_count}");
    let completed_text = format!("{completed} done");
    let errored_text = format!("{errored} failed");
    let throttle_text = throttle
        .as_ref()
        .map(|w| {
            let left = format_mm_ss(w.until_ms.saturating_sub(now));
            match &w.reason_label {
                Some(reason) => format!("{reason} · resumes in {left}"),
                None => format!("Throttled · resumes in {left}"),
            }
        })
        .unwrap_or_default();
    let pause_text = if paused {
        "Resume intake"
    } else {
        "Pause intake"
    };
    let slot_count_text = format!("{slots} slots");
    let chart_name = format!("Backlog, {queue_title}");
    let pinned_name = pinned_job
        .as_ref()
        .map(|j| j.job_label.clone())
        .unwrap_or_default();
    let pinned_detail = pinned_job.as_ref().and_then(|j| j.detail_text.clone());
    let pinned_error = pinned_job.as_ref().and_then(|j| j.error_text.clone());
    let pinned_attempt = pinned_job
        .as_ref()
        .map(|j| match j.max_attempts {
            Some(max) => format!("attempt {} of {max}", j.attempt_number),
            None => format!("attempt {}", j.attempt_number),
        })
        .unwrap_or_default();
    let pinned_priority = pinned_job
        .as_ref()
        .map(|j| j.lane_priority.word())
        .unwrap_or_default();
    let promote_job = pinned_job.clone();
    let withdraw_job = pinned_job.clone();
    let order_len = order.len();
    let live = format!(
        "{waiting_count} waiting · {active_count} active{}",
        if throttled { " · throttled" } else { "" }
    );

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", "aria-labelledby": "{heading_id}", "data-throttled": throttled.then_some("true"), "data-paused": paused.then_some("true"),
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape && pinned().is_some() {
                    let key = pinned().unwrap_or_default();
                    pinned.set(None);
                    if let Some(handler) = &on_open_change {
                        handler.call(None);
                    }
                    focus_after_frame(&format!("{root_id}-job-{key}"));
                }
            },
            header { class: "kit-wqf__head",
                h3 { id: "{heading_id}", class: "kit-wqf__title", {queue_title.clone()} }
                if let Some(name) = queue_name.clone() {
                    span { class: "kit-wqf__name", {name} }
                }
                span { class: "kit-wqf__sr", role: "status", "aria-live": "polite", {if throttled { throttle_text.clone() } else { String::new() }} }
                if throttled {
                    span { class: "kit-wqf__throttle", {throttle_text} }
                }
                button {
                    class: "kit-wqf__pause",
                    r#type: "button",
                    "aria-pressed": if paused { "true" } else { "false" },
                    onclick: move |_| {
                        let next = !paused;
                        if is_paused.is_none() {
                            paused_local.set(next);
                        }
                        if let Some(handler) = &on_paused_change {
                            handler.call(next);
                        }
                    },
                    {pause_text}
                }
                div { class: "kit-wqf__stepper", role: "group", "aria-label": "Slots",
                    button {
                        class: "kit-wqf__step",
                        r#type: "button",
                        disabled: at_min,
                        "aria-label": "Fewer slots",
                        onclick: move |_| {
                            if let Some(handler) = &on_slot_count_change {
                                handler.call(step_slots(slots, -1, min_slots, max_slots));
                            }
                        },
                        "−"
                    }
                    span { class: "kit-wqf__slot-count", {slot_count_text} }
                    button {
                        class: "kit-wqf__step",
                        r#type: "button",
                        disabled: at_max,
                        "aria-label": "More slots",
                        onclick: move |_| {
                            if let Some(handler) = &on_slot_count_change {
                                handler.call(step_slots(slots, 1, min_slots, max_slots));
                            }
                        },
                        "+"
                    }
                }
            }
            div { class: "kit-wqf__stats",
                div { class: "kit-wqf__tile", span { class: "kit-wqf__tile-value", {stat_waiting} } span { class: "kit-wqf__tile-label", "waiting" } }
                div { class: "kit-wqf__tile", span { class: "kit-wqf__tile-value", {stat_active} } span { class: "kit-wqf__tile-label", "active" } }
                div { class: "kit-wqf__tile", span { class: "kit-wqf__tile-value", {throughput_text} } span { class: "kit-wqf__tile-label", "throughput" } }
                div { class: "kit-wqf__tile", span { class: "kit-wqf__tile-value", {p95} } span { class: "kit-wqf__tile-label", "p95 wait" } }
            }
            div { class: "kit-wqf__flow", role: "group", "aria-label": "Job flow",
                onkeydown: move |event: KeyboardEvent| {
                    let name = match event.key() {
                        Key::ArrowRight => "ArrowRight",
                        Key::ArrowLeft => "ArrowLeft",
                        Key::ArrowDown => "ArrowDown",
                        Key::ArrowUp => "ArrowUp",
                        Key::Home => "Home",
                        Key::End => "End",
                        _ => return,
                    };
                    if let Some(next) = next_index(order_len, focused, name) {
                        event.prevent_default();
                        focus.set(next);
                        if let Some(&job) = order.get(next) {
                            focus_by_id(&format!("{flow_id}-job-{}", job_rows[job].job_key));
                        }
                    }
                },
                div { class: "kit-wqf__waiting",
                    span { class: "kit-wqf__col-title", "Waiting line" }
                    ol { class: "kit-wqf__line",
                        for (key, view) in line_views {
                            li { key: "{key}",
                                button {
                                    id: view.id.clone(),
                                    class: "kit-wqf__job",
                                    r#type: "button",
                                    "data-state": view.state,
                                    "data-throttled": view.throttled.then_some("true"),
                                    "aria-label": view.aria.clone(),
                                    tabindex: view.tab,
                                    onclick: {
                                        let job = key.clone();
                                        move |_| pinned_toggle(&mut pinned, &on_open_change, &job)
                                    },
                                    if let Some(src) = view.mark.clone() {
                                        img { class: "kit-wqf__mark", src: src, alt: "" }
                                    }
                                    span { class: "kit-wqf__label", {view.label.clone()} }
                                    span { class: "kit-wqf__meta", {view.kind} }
                                    span { class: "kit-wqf__meta", {view.time.clone()} }
                                    if let Some(p) = view.position {
                                        span { class: "kit-wqf__pos", {format!("#{}", p + 1)} }
                                    }
                                    span { class: "kit-wqf__badge", {view.priority} }
                                    span { class: "kit-wqf__badge", {format!("try {}", view.attempt)} }
                                }
                            }
                        }
                    }
                }
                div { class: "kit-wqf__slots",
                    span { class: "kit-wqf__col-title", "Slots" }
                    for (key, label, detail, job) in slot_views {
                        div { key: "{key}", class: "kit-wqf__slot",
                            span { class: "kit-wqf__slot-label", {label} }
                            if !detail.is_empty() {
                                span { class: "kit-wqf__slot-detail", {detail} }
                            }
                            if let Some((job_key, view)) = job {
                                button {
                                    id: view.id.clone(),
                                    class: "kit-wqf__job kit-wqf__job--active",
                                    r#type: "button",
                                    "data-state": view.state,
                                    "data-throttled": view.throttled.then_some("true"),
                                    "aria-label": view.aria.clone(),
                                    tabindex: view.tab,
                                    onclick: {
                                        let job = job_key.clone();
                                        move |_| pinned_toggle(&mut pinned, &on_open_change, &job)
                                    },
                                    span { class: "kit-wqf__label", {view.label.clone()} }
                                    span { class: "kit-wqf__bar", style: "width: {view.fill}" }
                                    span { class: "kit-wqf__meta", {view.time.clone()} }
                                }
                            }
                        }
                    }
                }
                div { class: "kit-wqf__done",
                    div { class: "kit-wqf__tile", span { class: "kit-wqf__tile-value", {completed_text} } }
                    div { class: "kit-wqf__tile", "data-tone": "error", span { class: "kit-wqf__tile-value", {errored_text} } }
                }
            }
            figure { class: "kit-wqf__chart", "aria-label": "{chart_name}",
                svg { view_box: "0 0 100 100", preserve_aspect_ratio: "none", "aria-hidden": "true",
                    polyline { class: "kit-wqf__depth", points: points }
                    if let Some(y) = backlog_y {
                        line { class: "kit-wqf__backlog", x1: "0", x2: "100", y1: format!("{y:.2}"), y2: format!("{y:.2}") }
                    }
                }
                figcaption { class: "kit-wqf__sr", {summary} }
            }
            if let Some(job) = pinned_job {
                div { id: "{card_id}", class: "kit-wqf__card", role: "dialog", "aria-label": "{pinned_name}",
                    p { class: "kit-wqf__card-line", {pinned_attempt} }
                    p { class: "kit-wqf__card-line", {format!("{pinned_priority} priority")} }
                    if let Some(detail) = pinned_detail {
                        p { class: "kit-wqf__card-line", {detail} }
                    }
                    if let Some(error) = pinned_error {
                        p { class: "kit-wqf__card-error", role: "alert", {error} }
                    }
                    div { class: "kit-wqf__card-actions",
                        if pinned_promotable {
                            button {
                                class: "kit-wqf__promote",
                                r#type: "button",
                                onclick: move |_| {
                                    if let (Some(handler), Some(job)) = (&on_promote, promote_job.clone()) {
                                        handler.call(job);
                                    }
                                },
                                "Promote"
                            }
                        }
                        button {
                            class: "kit-wqf__withdraw",
                            r#type: "button",
                            onclick: move |_| {
                                if let (Some(handler), Some(job)) = (&on_withdraw, withdraw_job.clone()) {
                                    handler.call(job);
                                }
                            },
                            "Withdraw"
                        }
                    }
                    span { class: "kit-wqf__sr", {job.job_key.clone()} }
                }
            }
            span { class: "kit-wqf__live", role: "status", "aria-live": "polite", {live} }
        }
    }
}

/// Pins a job's card, or closes it when it is already pinned.
fn pinned_toggle(
    pinned: &mut Signal<Option<String>>,
    on_open_change: &Option<EventHandler<Option<String>>>,
    job: &str,
) {
    let next = if pinned().as_deref() == Some(job) {
        None
    } else {
        Some(job.to_string())
    };
    pinned.set(next.clone());
    if let Some(handler) = on_open_change {
        handler.call(next);
    }
}
