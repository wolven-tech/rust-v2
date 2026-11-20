//! Background jobs popover: a pill that summarises background agent work, and
//! a dialog that lists each job with its progress and a pause, resume or open control.
//!
//! Design choice: the constrained trigger keeps the counts in a short form
//! (`2 / 1`) that CSS shows when the full summary no longer fits.
//! Design choice: a row's open control carries the row's accessible name; pause
//! and resume sit beside it, not inside it, because nested buttons are invalid.
//! Design choice: the announcement is raised when a job starts to need the
//! person or finishes, not on every progress tick.
//! Browser checks: focus moves into the dialog after the keyboard open (one
//! frame after render), the ring fill and the crossfade of the step text, the
//! row heights across states, and the outside click that closes the dialog.

use std::collections::BTreeMap;

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/background-jobs-popover.css");

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JobState {
    Running,
    Paused,
    NeedsYou,
    Done,
}

impl JobState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Paused => "paused",
            Self::NeedsYou => "needs-you",
            Self::Done => "done",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct BackgroundJob {
    pub job_id: String,
    pub job_title: String,
    pub current_step: String,
    pub steps_done: u32,
    pub steps_total: u32,
    pub job_state: JobState,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Edge {
    Start,
    #[default]
    End,
}

/// Every word the component shows. Unset fields fall back to English.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct JobsLabels {
    pub heading: Option<String>,
    pub idle: Option<String>,
    pub running: Option<String>,
    pub needs_you: Option<String>,
    pub paused: Option<String>,
    pub open_all: Option<String>,
    pub empty: Option<String>,
    pub pause: Option<String>,
    pub resume: Option<String>,
    pub open: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct JobCounts {
    pub running: usize,
    pub needs_you: usize,
    pub paused: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NavKey {
    Up,
    Down,
    Home,
    End,
}

/// Paused jobs are not counted as running; done jobs are not counted at all.
pub fn job_counts(jobs: &[BackgroundJob]) -> JobCounts {
    let mut counts = JobCounts::default();
    for job in jobs {
        match job.job_state {
            JobState::Running => counts.running += 1,
            JobState::NeedsYou => counts.needs_you += 1,
            JobState::Paused => counts.paused += 1,
            JobState::Done => {}
        }
    }
    counts
}

/// `3 running, 1 needs you`; falls back to the paused count, then to `None` for idle.
pub fn summary_text(
    counts: JobCounts,
    running: &str,
    needs_you: &str,
    paused: &str,
) -> Option<String> {
    let mut parts = Vec::new();
    if counts.running > 0 {
        parts.push(format!("{} {running}", counts.running));
    }
    if counts.needs_you > 0 {
        parts.push(format!("{} {needs_you}", counts.needs_you));
    }
    if parts.is_empty() && counts.paused > 0 {
        parts.push(format!("{} {paused}", counts.paused));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

/// Fraction of steps complete, 0 when there are no steps.
pub fn progress(steps_done: u32, steps_total: u32) -> f64 {
    if steps_total == 0 {
        return 0.0;
    }
    (f64::from(steps_done) / f64::from(steps_total)).clamp(0.0, 1.0)
}

/// SVG path for a ring arc of `fraction`, starting at 12 o'clock on a 24-unit box.
pub fn ring_path(fraction: f64) -> String {
    let f = fraction.clamp(0.0, 1.0);
    if f <= 0.0 {
        return String::new();
    }
    let sweep = f.min(0.9999) * std::f64::consts::TAU;
    let x = 12.0 + 9.0 * sweep.sin();
    let y = 12.0 - 9.0 * sweep.cos();
    let large = if f > 0.5 { 1 } else { 0 };
    format!("M12 3 A9 9 0 {large} 1 {x:.3} {y:.3}")
}

/// The next row to focus for a key press, or `None` when the key does nothing.
pub fn next_index(len: usize, current: Option<usize>, key: NavKey) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match key {
        NavKey::Home => 0,
        NavKey::End => len - 1,
        NavKey::Up => match current {
            Some(i) if i > 0 => i - 1,
            Some(_) => 0,
            None => len - 1,
        },
        NavKey::Down => match current {
            Some(i) if i + 1 < len => i + 1,
            Some(i) => i,
            None => 0,
        },
    })
}

fn pick(value: &Option<String>, fallback: &str) -> String {
    value.clone().unwrap_or_else(|| fallback.to_string())
}

fn focus_id(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

fn focus_later(id: String) {
    spawn(async move {
        sleep(16).await;
        focus_id(&id);
    });
}

/// Shifts the panel horizontally so it stays inside an 8px margin of the viewport.
fn clamp_panel(id: &str) {
    document::eval(&format!(
        "const p = document.getElementById({id:?}); if (p) {{ p.style.translate = '0px 0px'; const r = p.getBoundingClientRect(); const margin = 8; let shift = 0; if (r.right > innerWidth - margin) shift = innerWidth - margin - r.right; else if (r.left < margin) shift = margin - r.left; p.style.translate = shift + 'px 0px'; }}"
    ));
}

/// A pill summarising background jobs, opening a dialog with one row per job.
#[component]
pub fn BackgroundJobsPopover(
    #[props(default)] job_list: Vec<BackgroundJob>,
    #[props(default)] on_pause: Option<EventHandler<String>>,
    #[props(default)] on_resume: Option<EventHandler<String>>,
    #[props(default)] on_open_job: Option<EventHandler<String>>,
    #[props(default)] on_open_all: Option<EventHandler<()>>,
    #[props(default)] is_open: Option<bool>,
    #[props(default)] default_open: bool,
    #[props(default)] on_open_change: Option<EventHandler<bool>>,
    #[props(default)] align_edge: Edge,
    #[props(default)] label_text: Option<JobsLabels>,
    #[props(default, into)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-jobs-{}", next_id()));
    let mut internal_open = use_signal(|| default_open);
    let mut seen = use_signal(BTreeMap::<String, JobState>::new);
    let mut announce = use_signal(String::new);
    let mut focused_row = use_signal(|| Option::<usize>::None);

    let labels = label_text.unwrap_or_default();
    let heading = pick(&labels.heading, "Background tasks");
    let idle_text = pick(&labels.idle, "No background tasks");
    let running_word = pick(&labels.running, "running");
    let needs_word = pick(&labels.needs_you, "needs you");
    let paused_word = pick(&labels.paused, "paused");
    let open_all_text = pick(&labels.open_all, "All tasks");
    let empty_text = pick(&labels.empty, "Nothing running in the background.");
    let pause_word = pick(&labels.pause, "Pause");
    let resume_word = pick(&labels.resume, "Resume");
    let open_word = pick(&labels.open, "Open");

    let open = is_open.unwrap_or(internal_open());
    let counts = job_counts(&job_list);
    let summary = summary_text(counts, &running_word, &needs_word, &paused_word);
    let full_summary = match &summary {
        Some(text) => format!("{heading}: {text}"),
        None => format!("{heading}: {idle_text}"),
    };
    let short_summary = format!("{} · {}", counts.running, counts.needs_you);
    let trigger_label = summary.clone().unwrap_or_else(|| idle_text.clone());
    let expanded = if open { "true" } else { "false" };
    let trigger_id = format!("{base}-trigger");
    let panel_id = format!("{base}-panel");
    let align = match align_edge {
        Edge::Start => "start",
        Edge::End => "end",
    };
    let root_cls = class_list("kit-jobs", &root_class);
    let row_ids: Vec<String> = (0..job_list.len())
        .map(|i| format!("{base}-row-{i}"))
        .collect();
    let announcement = announce();
    let jobs_for_effect = job_list.clone();

    use_effect(use_reactive((&jobs_for_effect,), move |(jobs,)| {
        let previous = seen.peek().clone();
        let mut next = BTreeMap::new();
        let mut messages = Vec::new();
        for job in &jobs {
            let changed = previous
                .get(&job.job_id)
                .is_some_and(|before| *before != job.job_state);
            if changed && matches!(job.job_state, JobState::NeedsYou | JobState::Done) {
                let verb = if job.job_state == JobState::NeedsYou {
                    "needs you"
                } else {
                    "finished"
                };
                messages.push(format!("{} {verb}", job.job_title));
            }
            next.insert(job.job_id.clone(), job.job_state);
        }
        if !messages.is_empty() {
            announce.set(messages.join(". "));
        }
        seen.set(next);
    }));

    let mut set_open = move |value: bool| {
        if is_open.is_none() {
            internal_open.set(value);
        }
        if let Some(handler) = on_open_change {
            handler.call(value);
        }
    };

    let trigger_key = {
        let row_ids = row_ids.clone();
        let jobs_len = job_list.len();
        move |event: KeyboardEvent| match event.key() {
            Key::ArrowDown | Key::ArrowUp => {
                event.prevent_default();
                set_open(true);
                if jobs_len > 0 {
                    let target = if matches!(event.key(), Key::ArrowDown) {
                        0
                    } else {
                        jobs_len - 1
                    };
                    focus_later(row_ids[target].clone());
                }
            }
            _ => {}
        }
    };

    let rows_key = {
        let row_ids = row_ids.clone();
        let trigger_id = trigger_id.clone();
        move |event: KeyboardEvent| {
            let nav = match event.key() {
                Key::ArrowDown => NavKey::Down,
                Key::ArrowUp => NavKey::Up,
                Key::Home => NavKey::Home,
                Key::End => NavKey::End,
                Key::Escape => {
                    event.prevent_default();
                    set_open(false);
                    focus_id(&trigger_id);
                    return;
                }
                _ => return,
            };
            event.prevent_default();
            if let Some(index) = next_index(row_ids.len(), focused_row(), nav) {
                focus_id(&row_ids[index]);
            }
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: root_cls, "data-align": align,
            button {
                id: "{trigger_id}",
                r#type: "button",
                class: "kit-jobs__trigger",
                "aria-label": "{full_summary}",
                "aria-haspopup": "dialog",
                "aria-expanded": "{expanded}",
                "aria-controls": "{panel_id}",
                onclick: {
                    let row_ids = row_ids.clone();
                    move |_| {
                        set_open(!open);
                        if !open {
                            if let Some(first) = row_ids.first() {
                                focus_later(first.clone());
                            }
                        }
                    }
                },
                onkeydown: trigger_key,
                span { class: "kit-jobs__full", "aria-hidden": "true", "{trigger_label}" }
                span { class: "kit-jobs__short", "aria-hidden": "true", "{short_summary}" }
            }
            if open {
                div {
                    id: "{panel_id}",
                    class: "kit-jobs__panel",
                    role: "dialog",
                    "aria-label": "{heading}",
                    onmounted: {
                        let panel_id = panel_id.clone();
                        move |_: MountedEvent| clamp_panel(&panel_id)
                    },
                    onkeydown: rows_key,
                    h2 { class: "kit-jobs__heading", "{heading}" }
                    if job_list.is_empty() {
                        p { class: "kit-jobs__empty", "{empty_text}" }
                    }
                    ul { class: "kit-jobs__list",
                        for (index, job) in job_list.iter().enumerate() {
                            {
                                let row_id = row_ids[index].clone();
                                let id = job.job_id.clone();
                                let state_word = job.job_state.as_str();
                                let fraction = progress(job.steps_done, job.steps_total);
                                let ring = ring_path(fraction);
                                let step_text = job.current_step.clone();
                                let title = job.job_title.clone();
                                let row_name = format!(
                                    "{title}, {state_word}, {step_text}, {} of {} steps",
                                    job.steps_done, job.steps_total
                                );
                                let steps_label = format!("{} of {}", job.steps_done, job.steps_total);
                                let needs = job.job_state == JobState::NeedsYou;
                                let control_name = format!("{} {title}", if job.job_state == JobState::Paused { &resume_word } else { &pause_word });
                                let can_pause = job.job_state == JobState::Running;
                                let can_resume = job.job_state == JobState::Paused;
                                let pause_id = id.clone();
                                let resume_id = id.clone();
                                let open_job_id = id.clone();
                                let open_name = format!("{open_word} {title}");
                                let row_class = format!("kit-jobs__row is-{state_word}");
                                let open_job = on_open_job;
                                rsx! {
                                    li { key: "{id}", class: row_class, "data-job-state": state_word,
                                        button {
                                            id: "{row_id}",
                                            r#type: "button",
                                            class: "kit-jobs__open",
                                            "aria-label": "{row_name}",
                                            onfocus: move |_| focused_row.set(Some(index)),
                                            onclick: move |_| {
                                                if let Some(handler) = open_job {
                                                    handler.call(open_job_id.clone());
                                                }
                                            },
                                            svg { class: "kit-jobs__ring", view_box: "0 0 24 24", "aria-hidden": "true",
                                                circle { class: "kit-jobs__ring-track", cx: "12", cy: "12", r: "9", fill: "none" }
                                                path { class: "kit-jobs__ring-fill", d: "{ring}", fill: "none" }
                                            }
                                            span { class: "kit-jobs__text",
                                                span { class: "kit-jobs__title", "{title}" }
                                                span { class: "kit-jobs__step", "{step_text}" }
                                            }
                                            span { class: "kit-jobs__steps", "{steps_label}" }
                                            if needs {
                                                span { class: "kit-jobs__badge", "{needs_word}" }
                                            }
                                        }
                                        if can_pause {
                                            button {
                                                r#type: "button",
                                                class: "kit-jobs__icon",
                                                "aria-label": "{control_name}",
                                                onclick: move |_| {
                                                    if let Some(handler) = on_pause {
                                                        handler.call(pause_id.clone());
                                                    }
                                                },
                                                "❚❚"
                                            }
                                        }
                                        if can_resume {
                                            button {
                                                r#type: "button",
                                                class: "kit-jobs__icon",
                                                "aria-label": "{control_name}",
                                                onclick: move |_| {
                                                    if let Some(handler) = on_resume {
                                                        handler.call(resume_id.clone());
                                                    }
                                                },
                                                "▶"
                                            }
                                        }
                                        if needs {
                                            button {
                                                r#type: "button",
                                                class: "kit-jobs__icon",
                                                "aria-label": "{open_name}",
                                                onclick: {
                                                    let open_again = id.clone();
                                                    move |_| {
                                                        if let Some(handler) = open_job {
                                                            handler.call(open_again.clone());
                                                        }
                                                    }
                                                },
                                                "↗"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(handler) = on_open_all {
                        button {
                            r#type: "button",
                            class: "kit-jobs__all",
                            onclick: move |_| handler.call(()),
                            "{open_all_text}"
                        }
                    }
                }
            }
            div { class: "kit-jobs__sr", role: "status", "aria-live": "polite", "{announcement}" }
        }
    }
}
