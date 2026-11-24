//! Agent schedule board: the recurring jobs an agent runs, when each next runs,
//! how the recent runs ended, and what they cost. A person can pause a schedule,
//! skip one upcoming run, or start a run now.
//!
//! Design choice: the timeline is centred on `now_ms`, which the host supplies.
//! Past runs sit left of the now line and upcoming runs right of it, within a
//! window of twelve hours either side. Nothing reads the browser clock.
//! Design choice: a skip is shown at once, and Restore calls `on_skip` with the
//! flag cleared. The host's `run_times` are not changed by the page.
//! Design choice: outcomes are words (Done, Needs you, Failed) with a colour tag;
//! the word is always there for screen readers.
//! Design choice: the switch is a checkbox with `role=switch`, named with the
//! schedule and the action it takes.
//! Design choice: the activity trace above the axis is a fixed-height SVG; running
//! runs lift it from their marker towards the now line, finished runs leave a blip.
//! Design choice: Run now adds a running marker at the now line for this session;
//! the host's `run_times` are not changed by the page.
//! Browser checks: markers left of the now line show outcomes; Run now adds a
//! running marker and calls `on_run_now`; Skip removes an upcoming marker and
//! Restore brings it back; the status line names the running run with its
//! percentage; Space on a switch pauses or resumes; no horizontal scroll at 320 px.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, focus_after_frame, next_id};

const CSS: Asset = asset!("/assets/agent-schedule-board.css");

/// Half the axis: the axis spans ten hours, five either side of now.
const WINDOW_MS: u64 = 5 * 3_600_000;
const TICK_MS: u64 = 2 * 3_600_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkState {
    Upcoming,
    Running,
    Done,
    NeedsYou,
    Failed,
}

impl MarkState {
    fn key(self) -> &'static str {
        match self {
            Self::Upcoming => "upcoming",
            Self::Running => "running",
            Self::Done => "done",
            Self::NeedsYou => "needs-you",
            Self::Failed => "failed",
        }
    }

    fn word(self) -> &'static str {
        match self {
            Self::Upcoming => "Upcoming",
            Self::Running => "Running",
            Self::Done => "Done",
            Self::NeedsYou => "Needs you",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Done,
    NeedsYou,
    Failed,
}

impl Outcome {
    fn key(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::NeedsYou => "needs-you",
            Self::Failed => "failed",
        }
    }

    fn word(self) -> &'static str {
        match self {
            Self::Done => "Done",
            Self::NeedsYou => "Needs you",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunTime {
    pub at_ms: u64,
    pub run_state: MarkState,
    pub progress_percent: Option<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HistoryEntry {
    pub at_ms: u64,
    pub outcome: Outcome,
    pub summary: String,
    pub cost_label: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScheduleRecord {
    pub id: String,
    pub entry_name: String,
    pub project_name: Option<String>,
    pub brief_summary: Option<String>,
    pub cadence_text: String,
    pub run_times: Vec<RunTime>,
    pub duration_seconds: Option<u64>,
    pub is_paused: bool,
    pub history_log: Vec<HistoryEntry>,
    pub cost_per_run_label: Option<String>,
}

/// Horizontal position of `at_ms` on the axis, 0 to 100 percent, centred on `now_ms`.
pub fn marker_position(at_ms: u64, now_ms: u64) -> f64 {
    let offset = at_ms as f64 - now_ms as f64;
    (50.0 + offset / WINDOW_MS as f64 * 50.0).clamp(0.0, 100.0)
}

/// Round two-hour ticks on the axis as `(percent, "HH:00 UTC")`.
pub fn axis_ticks(now_ms: u64) -> Vec<(f64, String)> {
    let first = now_ms.saturating_sub(WINDOW_MS).div_ceil(TICK_MS) * TICK_MS;
    let last = now_ms + WINDOW_MS;
    (first..=last)
        .step_by(TICK_MS as usize)
        .filter(|at| at.abs_diff(now_ms) < WINDOW_MS)
        .map(|at| {
            let hour = (at / 3_600_000) % 24;
            (marker_position(at, now_ms), format!("{hour:02}:00 UTC"))
        })
        .collect()
}

/// Upcoming runs in time order, skipping any the person has skipped.
pub fn next_runs(
    schedule: &ScheduleRecord,
    now_ms: u64,
    skipped: &[u64],
    count: usize,
) -> Vec<RunTime> {
    let mut upcoming: Vec<RunTime> = schedule
        .run_times
        .iter()
        .filter(|r| r.at_ms >= now_ms && !skipped.contains(&r.at_ms))
        .cloned()
        .collect();
    upcoming.sort_by_key(|r| r.at_ms);
    upcoming.truncate(count);
    upcoming
}

/// Names the run that is going now, or the next one to come.
pub fn status_line(schedules: &[ScheduleRecord], now_ms: u64) -> String {
    let running = schedules.iter().find_map(|s| {
        s.run_times
            .iter()
            .find(|r| r.run_state == MarkState::Running)
            .map(|r| (s, r.progress_percent.unwrap_or(0)))
    });
    if let Some((schedule, percent)) = running {
        return format!("Running now: {}, {} percent", schedule.entry_name, percent);
    }
    let next = schedules
        .iter()
        .filter(|s| !s.is_paused)
        .flat_map(|s| {
            s.run_times
                .iter()
                .filter(move |r| r.at_ms >= now_ms)
                .map(move |r| (s, r.at_ms))
        })
        .min_by_key(|(_, at)| *at);
    match next {
        Some((schedule, at)) => format!(
            "Next: {} in {} min",
            schedule.entry_name,
            (at.saturating_sub(now_ms)) / 60_000
        ),
        None => "Nothing scheduled.".to_string(),
    }
}

/// Activity trace across the axis, as SVG points in a 100 x 24 box. Each running
/// run climbs from its marker towards the now line, to a height set by its progress;
/// a finished or failed run leaves a short blip at its marker.
pub fn trace_points(schedules: &[ScheduleRecord], now_ms: u64) -> String {
    let runs: Vec<(f64, MarkState, u8)> = schedules
        .iter()
        .flat_map(|s| s.run_times.iter())
        .map(|r| {
            (
                marker_position(r.at_ms, now_ms),
                r.run_state,
                r.progress_percent.unwrap_or(0),
            )
        })
        .collect();
    (0..=50)
        .map(|step| {
            let x = f64::from(step) * 2.0;
            let mut level: f64 = 0.0;
            for (start, state, progress) in &runs {
                match state {
                    MarkState::Running if *start <= 50.0 && x >= *start => {
                        let ramp = ((x - start) / (50.0 - start).max(1.0)).clamp(0.0, 1.0);
                        let head = 0.3 + 0.7 * f64::from(*progress) / 100.0;
                        level = level.max(ramp * head);
                    }
                    MarkState::Done | MarkState::Failed | MarkState::NeedsYou => {
                        let distance = (x - start).abs();
                        if distance < 3.0 {
                            level = level.max(0.6 * (1.0 - distance / 3.0));
                        }
                    }
                    MarkState::Running | MarkState::Upcoming => {}
                }
            }
            format!("{x:.1},{:.1}", 23.0 - level * 21.0)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The accessible name of a marker: schedule, moment and state.
pub fn marker_label(
    schedule: &str,
    at_ms: u64,
    now_ms: u64,
    state: MarkState,
    percent: Option<u8>,
) -> String {
    let when = if at_ms < now_ms { "earlier" } else { "later" };
    match state {
        MarkState::Running => format!(
            "{schedule}, {when}, in progress, {} percent",
            percent.unwrap_or(0)
        ),
        other => format!("{schedule}, {when}, {}", other.word()),
    }
}

/// The schedules on a timeline, with a status line, a list and a panel.
#[component]
pub fn AgentScheduleBoard(
    #[props(default)] schedule_list: Vec<ScheduleRecord>,
    #[props(default)] now_ms: u64,
    #[props(default)] selected_schedule_id: Option<String>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_pause: Option<EventHandler<(String, bool)>>,
    #[props(default)] on_run_now: Option<EventHandler<String>>,
    #[props(default)] on_skip: Option<EventHandler<(String, u64, bool)>>,
    #[props(default)] on_select: Option<EventHandler<String>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
) -> Element {
    let uid = use_hook(next_id);
    let mut selected = use_signal(move || selected_schedule_id.clone());
    let mut skipped = use_signal(Vec::<(String, u64)>::new);
    let mut paused = use_signal(Vec::<(String, bool)>::new);
    let mut ran = use_signal(Vec::<String>::new);
    let mut announce = use_signal(String::new);

    let active = schedule_list.iter().filter(|s| !s.is_paused).count();
    let total = schedule_list.len();
    let count_text = format!("{active} of {total} active");
    let ran_now = ran();
    let effective: Vec<ScheduleRecord> = schedule_list
        .iter()
        .map(|s| {
            let mut s = s.clone();
            if ran_now.contains(&s.id) {
                s.run_times.push(RunTime {
                    at_ms: now_ms,
                    run_state: MarkState::Running,
                    progress_percent: Some(0),
                });
            }
            s
        })
        .collect();
    let status = status_line(&effective, now_ms);
    let trace = trace_points(&effective, now_ms);
    let open = selected().and_then(|id| schedule_list.iter().find(|s| s.id == id).cloned());
    let order: Vec<String> = schedule_list.iter().map(|s| s.id.clone()).collect();
    let skipped_now = skipped();
    let paused_now = paused();
    let key_order = order.clone();
    let now_line = marker_position(now_ms, now_ms);

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: "kit-asb",
            "data-loading": is_loading.then_some("true"),
            "aria-busy": is_loading.then_some("true"),
            onkeydown: move |event: KeyboardEvent| {
                let delta = match event.key() {
                    Key::ArrowDown => 1,
                    Key::ArrowUp => -1,
                    Key::Character(c) if c == "j" => 1,
                    Key::Character(c) if c == "k" => -1,
                    _ => return,
                };
                let current = selected();
                let at = current.as_deref().and_then(|id| key_order.iter().position(|o| o == id));
                let next = match at {
                    Some(i) => (i as isize + delta).clamp(0, key_order.len() as isize - 1) as usize,
                    None => 0,
                };
                if let Some(id) = key_order.get(next) {
                    selected.set(Some(id.clone()));
                    if let Some(handler) = on_select {
                        handler.call(id.clone());
                    }
                    focus_after_frame(&format!("kit-asb-{uid}-row-{id}"));
                }
            },
            header { class: "kit-asb__head",
                h2 { class: "kit-asb__title", "Schedules" }
                p { class: "kit-asb__count", "{count_text}" }
            }

            if is_empty && !is_loading {
                div { class: "kit-asb__empty",
                    p { "Schedules run an agent on a rhythm. Ask in Thread to set one up." }
                    button {
                        r#type: "button",
                        class: "kit-asb__primary",
                        onclick: move |_| {
                            if let Some(handler) = on_navigate {
                                handler.call("Thread".to_string());
                            }
                        },
                        "Ask in Thread"
                    }
                }
            } else if is_loading {
                div { class: "kit-asb__ph kit-asb__ph--chart", "aria-hidden": "true" }
                div { class: "kit-asb__ph kit-asb__ph--row", "aria-hidden": "true" }
            } else {
                section { class: "kit-asb__timeline", "aria-label": "Timeline",
                    svg {
                        class: "kit-asb__trace",
                        view_box: "0 0 100 24",
                        "preserveAspectRatio": "none",
                        "aria-hidden": "true",
                        polyline { class: "kit-asb__trace-line", points: "{trace}" }
                    }
                    div { class: "kit-asb__axis",
                        span { class: "kit-asb__now", style: "left: {now_line:.2}%;", "aria-hidden": "true" }
                        for (tick_pos, tick_label) in axis_ticks(now_ms) {
                            span { class: "kit-asb__tick", style: "left: {tick_pos:.2}%;", "{tick_label}" }
                        }
                        for schedule in effective.iter().cloned() {
                            {
                                let is_paused_now = paused_now.iter().find(|(id, _)| *id == schedule.id).map_or(schedule.is_paused, |(_, p)| *p);
                                let skip_list: Vec<u64> = skipped_now.iter().filter(|(id, _)| *id == schedule.id).map(|(_, at)| *at).collect();
                                let runs: Vec<RunTime> = schedule
                                    .run_times
                                    .iter()
                                    .filter(|r| !(is_paused_now && r.run_state == MarkState::Upcoming))
                                    .filter(|r| r.at_ms.abs_diff(now_ms) < WINDOW_MS)
                                    .cloned()
                                    .collect();
                                rsx! {
                                    for (index, run) in runs.into_iter().enumerate() {
                                        {
                                            let hidden = run.run_state == MarkState::Upcoming && skip_list.contains(&run.at_ms);
                                            let pos = marker_position(run.at_ms, now_ms);
                                            let label = marker_label(&schedule.entry_name, run.at_ms, now_ms, run.run_state, run.progress_percent);
                                            let key = format!("{}-{index}", schedule.id);
                                            let style = format!("left: {pos:.2}%;");
                                            let state_key = run.run_state.key();
                                            let pick = schedule.id.clone();
                                            if hidden {
                                                rsx! {}
                                            } else {
                                                rsx! {
                                                    button {
                                                        key: "{key}",
                                                        r#type: "button",
                                                        class: "kit-asb__marker",
                                                        "data-state": "{state_key}",
                                                        style: "{style}",
                                                        "aria-label": "{label}",
                                                        onclick: move |_| {
                                                            selected.set(Some(pick.clone()));
                                                            if let Some(handler) = on_select {
                                                                handler.call(pick.clone());
                                                            }
                                                        },
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    p { class: "kit-asb__status", role: "status", "{status}" }
                }

                div { class: "kit-asb__body",
                    ul { class: "kit-asb__list", role: "list",
                        for schedule in schedule_list.iter().cloned() {
                            {
                                let row_id = format!("kit-asb-{uid}-row-{}", schedule.id);
                                let row_key = schedule.id.clone();
                                let is_paused_now = paused_now.iter().find(|(id, _)| *id == schedule.id).map_or(schedule.is_paused, |(_, p)| *p);
                                let next_at = next_runs(&schedule, now_ms, &[], 1)
                                    .first()
                                    .map_or_else(|| "none".to_string(), |r| format!("in {} min", r.at_ms.saturating_sub(now_ms) / 60_000));
                                let last = schedule.history_log.last().map(|h| (h.outcome.key(), h.outcome.word()));
                                let (last_key, last_word) = last.unwrap_or(("none", "No runs"));
                                let avg = schedule.cost_per_run_label.clone().unwrap_or_else(|| "—".to_string());
                                let switch_label = format!("{} {}", if is_paused_now { "Resume" } else { "Pause" }, schedule.entry_name);
                                let switch_checked = !is_paused_now;
                                let switch_id = schedule.id.clone();
                                let row_name = schedule.entry_name.clone();
                                let cadence = schedule.cadence_text.clone();
                                let pick = schedule.id.clone();
                                rsx! {
                                    li { key: "{row_key}", class: "kit-asb__row", "data-paused": "{is_paused_now}",
                                        button {
                                            id: "{row_id}",
                                            r#type: "button",
                                            class: "kit-asb__row-hit",
                                            "aria-current": (selected().as_deref() == Some(schedule.id.as_str())).then_some("true"),
                                            onclick: move |_| {
                                                selected.set(Some(pick.clone()));
                                                if let Some(handler) = on_select {
                                                    handler.call(pick.clone());
                                                }
                                            },
                                            span { class: "kit-asb__name", "{row_name}" }
                                            span { class: "kit-asb__meta", "{cadence}" }
                                            span { class: "kit-asb__meta", "Next {next_at}" }
                                            span { class: "kit-asb__outcome", "data-outcome": "{last_key}", "{last_word}" }
                                            span { class: "kit-asb__meta", "{avg}" }
                                        }
                                        label { class: "kit-asb__switch",
                                            input {
                                                r#type: "checkbox",
                                                role: "switch",
                                                "aria-label": "{switch_label}",
                                                checked: switch_checked,
                                                onchange: move |event: FormEvent| {
                                                    let next_paused = !event.checked();
                                                    paused.with_mut(|list| {
                                                        list.retain(|(id, _)| *id != switch_id);
                                                        list.push((switch_id.clone(), next_paused));
                                                    });
                                                    announce.set(if next_paused { "Paused" } else { "Resumed" }.to_string());
                                                    if let Some(handler) = on_pause {
                                                        handler.call((switch_id.clone(), next_paused));
                                                    }
                                                },
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(schedule) = open.clone() {
                        {
                            let is_paused_now = paused_now.iter().find(|(id, _)| *id == schedule.id).map_or(schedule.is_paused, |(_, p)| *p);
                            let upcoming = next_runs(&schedule, now_ms, &[], 5);
                            let history = schedule.history_log.clone();
                            let title = schedule.entry_name.clone();
                            let brief = schedule.brief_summary.clone().unwrap_or_default();
                            let cadence = schedule.cadence_text.clone();
                            let cost_line = schedule.cost_per_run_label.clone().unwrap_or_else(|| "No runs yet".to_string());
                            let action_label = if is_paused_now { "Resume" } else { "Pause" };
                            let run_id = schedule.id.clone();
                            let pause_id = schedule.id.clone();
                            let skip_targets: Vec<(u64, String)> = upcoming
                                .iter()
                                .map(|r| (r.at_ms, format!("{} min", r.at_ms.saturating_sub(now_ms) / 60_000)))
                                .collect();
                            let skip_schedule = schedule.id.clone();
                            rsx! {
                                aside { class: "kit-asb__panel", "aria-label": "Schedule detail",
                                    h3 { class: "kit-asb__panel-title", "{title}" }
                                    if !brief.is_empty() {
                                        p { class: "kit-asb__meta", "{brief}" }
                                    }
                                    p { class: "kit-asb__meta", "{cadence} · {cost_line}" }
                                    section { class: "kit-asb__block", "aria-label": "Next runs",
                                        h4 { class: "kit-asb__section", "Next runs" }
                                        ul { class: "kit-asb__runs",
                                            for (at, when) in skip_targets.iter().cloned() {
                                                {
                                                    let is_skipped = skipped_now.iter().any(|(id, s)| *id == skip_schedule && *s == at);
                                                    let skip_key = format!("{at}");
                                                    let skip_label = if is_skipped { "Restore" } else { "Skip" };
                                                    let skip_name = format!("{} run {when}", skip_label);
                                                    let skip_owner = skip_schedule.clone();
                                                    rsx! {
                                                        li { key: "{skip_key}",
                                                            span { "{when}" }
                                                            button {
                                                                r#type: "button",
                                                                class: "kit-asb__ghost",
                                                                "aria-label": "{skip_name}",
                                                                onclick: move |_| {
                                                                    let restoring = is_skipped;
                                                                    skipped.with_mut(|list| {
                                                                        if restoring {
                                                                            list.retain(|(id, s)| !(*id == skip_owner && *s == at));
                                                                        } else {
                                                                            list.push((skip_owner.clone(), at));
                                                                        }
                                                                    });
                                                                    if let Some(handler) = on_skip {
                                                                        handler.call((skip_owner.clone(), at, !restoring));
                                                                    }
                                                                },
                                                                "{skip_label}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    section { class: "kit-asb__block", "aria-label": "Recent runs",
                                        h4 { class: "kit-asb__section", "Recent runs" }
                                        ul { class: "kit-asb__runs",
                                            for (index, entry) in history.iter().enumerate() {
                                                {
                                                    let entry_key = format!("{index}");
                                                    let entry_word = entry.outcome.word();
                                                    let entry_state = entry.outcome.key();
                                                    let entry_text = entry.summary.clone();
                                                    let entry_cost = entry.cost_label.clone().unwrap_or_default();
                                                    rsx! {
                                                        li { key: "{entry_key}",
                                                            span { class: "kit-asb__outcome", "data-outcome": "{entry_state}", "{entry_word}" }
                                                            span { "{entry_text}" }
                                                            span { class: "kit-asb__meta", "{entry_cost}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    div { class: "kit-asb__actions",
                                        button {
                                            r#type: "button",
                                            class: "kit-asb__primary",
                                            onclick: move |_| {
                                                ran.with_mut(|list| list.push(run_id.clone()));
                                                announce.set(format!("Started {title}"));
                                                if let Some(handler) = on_run_now {
                                                    handler.call(run_id.clone());
                                                }
                                            },
                                            "Run now"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "kit-asb__ghost",
                                            onclick: move |_| {
                                                if let Some(handler) = on_pause {
                                                    handler.call((pause_id.clone(), !is_paused_now));
                                                }
                                            },
                                            "{action_label}"
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
