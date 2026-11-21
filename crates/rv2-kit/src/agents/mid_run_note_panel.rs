//! Mid-run note panel: a person adds a note to an agent that is already
//! working, queued for its next step or applied to the active step at once.
//! The plan shows which step is active and where each note landed.
//!
//! The component draws and reports. The host moves a queued note into a
//! step's `step_notes` when the agent takes it in.
//!
//! Design choice: the clock is `clock_ms` when the host supplies one, else the
//! browser clock. Elapsed is now minus `run_started_ms`, less the time spent
//! paused; a pause freezes it and resuming continues from the same value.
//! Design choice: a note already present in some step's `step_notes` is hidden
//! from the queue even if the host has not yet removed it from `queued_notes`.
//! Design choice: Ctrl+Enter and Cmd+Enter both apply now on every platform.
//! Telling Apple platforms apart would need a platform read.
//! Design choice: a taken-in note appears in its step with a brightening, not a
//! flight from the queue. The travel animation is not built.
//! Design choice: the queue region renders only while it holds notes.
//! Browser checks: the clock ticks and freezes on pause; Enter, Shift+Enter,
//! Ctrl/Cmd+Enter and Esc in the composer; queue open and close without row
//! jump; Tab order; the live region announces queued, applied and taken-in
//! notes; reduced motion stops the shimmer.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id, use_now_ms};

const CSS: Asset = asset!("/assets/mid-run-note-panel.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunState {
    Running,
    Paused,
    Finished,
}

impl RunState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Finished => "finished",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Paused => "Paused",
            Self::Finished => "Finished",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepPhase {
    Upcoming,
    Active,
    Done,
}

impl StepPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Upcoming => "upcoming",
            Self::Active => "active",
            Self::Done => "done",
        }
    }

    fn word(self) -> &'static str {
        match self {
            Self::Upcoming => "Upcoming",
            Self::Active => "Running",
            Self::Done => "Done",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotePlacement {
    Next,
    Now,
}

impl NotePlacement {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Next => "next",
            Self::Now => "now",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Next => "Next step",
            Self::Now => "Applied now",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub note_key: String,
    pub text: String,
    pub placed: Option<NotePlacement>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlanStep {
    pub step_key: String,
    pub label: String,
    pub phase: StepPhase,
    pub began_at_ms: Option<u64>,
    pub took_ms: Option<u64>,
    pub step_notes: Vec<Note>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComposerAction {
    Queue,
    ApplyNow,
    Newline,
    Clear,
    Ignore,
}

/// Advances the clock by one tick while running; any other state keeps it.
pub fn advance(now_ms: u64, state: RunState) -> u64 {
    match state {
        RunState::Running => now_ms + 1000,
        RunState::Paused | RunState::Finished => now_ms,
    }
}

/// `m:ss`, with the hours in front past one hour.
pub fn format_clock(ms: u64) -> String {
    let total = ms / 1000;
    let (h, m, s) = (total / 3600, total / 60 % 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// A done step shows its duration; the active step shows its live elapsed
/// time; an upcoming step shows nothing.
pub fn step_time_ms(step: &PlanStep, run_started_ms: u64, now_ms: u64) -> Option<u64> {
    match step.phase {
        StepPhase::Done => step.took_ms,
        StepPhase::Active => {
            Some(now_ms.saturating_sub(step.began_at_ms.unwrap_or(run_started_ms)))
        }
        StepPhase::Upcoming => None,
    }
}

/// `(position, total)` for "step n of m": the active step's number, or the
/// count of finished steps when none is active.
pub fn step_counter(steps: &[PlanStep]) -> (usize, usize) {
    let position = match steps
        .iter()
        .position(|step| step.phase == StepPhase::Active)
    {
        Some(index) => index + 1,
        None => steps
            .iter()
            .filter(|step| step.phase == StepPhase::Done)
            .count(),
    };
    (position, steps.len())
}

/// Every note the agent has taken in, in plan order.
pub fn taken_notes(steps: &[PlanStep]) -> Vec<Note> {
    steps
        .iter()
        .flat_map(|step| step.step_notes.iter().cloned())
        .collect()
}

/// The queue minus notes already taken into a step.
pub fn queue_visible(queued: &[Note], steps: &[PlanStep]) -> Vec<Note> {
    let taken = taken_notes(steps);
    queued
        .iter()
        .filter(|note| !taken.iter().any(|t| t.note_key == note.note_key))
        .cloned()
        .collect()
}

/// Notes whose key is not yet in `seen`: the ones to announce.
pub fn fresh_notes(notes: &[Note], seen: &[String]) -> Vec<Note> {
    notes
        .iter()
        .filter(|note| !seen.contains(&note.note_key))
        .cloned()
        .collect()
}

/// The composer text as submitted: trimmed, and `None` when blank.
pub fn submit_text(draft: &str) -> Option<String> {
    let trimmed = draft.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// What an Enter or Esc keypress in the composer does.
pub fn composer_action(key: &Key, chord: bool, shift: bool) -> ComposerAction {
    match key {
        Key::Enter if shift => ComposerAction::Newline,
        Key::Enter if chord => ComposerAction::ApplyNow,
        Key::Enter => ComposerAction::Queue,
        Key::Escape => ComposerAction::Clear,
        _ => ComposerAction::Ignore,
    }
}

/// The state the pause control moves to. A finished run has no control.
pub fn next_run_state(state: RunState) -> Option<RunState> {
    match state {
        RunState::Running => Some(RunState::Paused),
        RunState::Paused => Some(RunState::Running),
        RunState::Finished => None,
    }
}

/// The polite line for notes the agent has just taken in.
pub fn taken_announcement(notes: &[Note]) -> String {
    match notes {
        [note] => format!("Agent took in: {}", note.text),
        _ => format!("Agent took in {} notes.", notes.len()),
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TakenView {
    text: String,
    placed_key: Option<&'static str>,
    placed_label: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq)]
struct StepView {
    label: String,
    phase_key: &'static str,
    phase_word: &'static str,
    active: bool,
    time: Option<String>,
    notes: Vec<TakenView>,
}

/// One run's plan, queue and composer. Steps show their phase and time,
/// with the notes the agent has taken in beneath them.
///
/// Notes go in through `on_queue_note` (Enter) or `on_apply_note_now`
/// (Ctrl/Cmd+Enter, or a queued note's apply action). The component clears
/// the composer itself through `on_draft_change`.
#[component]
pub fn MidRunNotePanel(
    #[props(into)] run_heading: String,
    #[props(default, into)] run_agent_name: Option<String>,
    run_started_ms: u64,
    run_state: RunState,
    plan_steps: Vec<PlanStep>,
    queued_notes: Vec<Note>,
    #[props(into)] draft_text: String,
    #[props(default, into)] draft_hint: Option<String>,
    on_queue_note: EventHandler<String>,
    on_apply_note_now: EventHandler<(String, Option<String>)>,
    on_drop_note: EventHandler<String>,
    on_run_state_request: EventHandler<RunState>,
    on_draft_change: EventHandler<String>,
    #[props(default)] clock_ms: Option<u64>,
    #[props(default)] extra_class: String,
) -> Element {
    let uid = use_hook(next_id);
    let heading_id = format!("kit-mrnp-{uid}-head");
    let queue_id = format!("kit-mrnp-{uid}-queue");
    let label_id = format!("kit-mrnp-{uid}-label");
    let hint_id = format!("kit-mrnp-{uid}-hint");
    let field_id = format!("kit-mrnp-{uid}-field");

    let wall = use_now_ms(clock_ms, 1000);
    let mut paused_since = use_signal(|| None::<u64>);
    let mut paused_total = use_signal(|| 0u64);
    let mut announce = use_signal(String::new);
    let taken = taken_notes(&plan_steps);
    let mut seen = use_signal(|| {
        taken_notes(&plan_steps)
            .into_iter()
            .map(|note| note.note_key)
            .collect::<Vec<String>>()
    });

    let running = run_state == RunState::Running;
    use_effect(use_reactive!(|(running, wall)| {
        if wall == 0 {
            return;
        }
        let since = *paused_since.peek();
        match (running, since) {
            (false, None) => paused_since.set(Some(wall)),
            (true, Some(since)) => {
                paused_total.with_mut(|total| *total += wall.saturating_sub(since));
                paused_since.set(None);
            }
            _ => {}
        }
    }));
    use_effect(use_reactive!(|(taken,)| {
        let fresh = fresh_notes(&taken, &seen.peek());
        if !fresh.is_empty() {
            seen.with_mut(|keys| keys.extend(fresh.iter().map(|note| note.note_key.clone())));
            announce.set(taken_announcement(&fresh));
        }
    }));
    let reference = paused_since().unwrap_or(wall);
    let elapsed = reference
        .saturating_sub(run_started_ms)
        .saturating_sub(paused_total());
    let now = run_started_ms + elapsed;
    let clock = format_clock(elapsed);
    let (position, total) = step_counter(&plan_steps);
    let step_label = format!("Step {position} of {total}");
    let state_label = run_state.label();
    let finished = run_state == RunState::Finished;
    let next_state = next_run_state(run_state);
    let run_label = if run_state == RunState::Running {
        "Pause"
    } else {
        "Resume"
    };
    let agent_name = run_agent_name.unwrap_or_default();
    let has_agent = !agent_name.is_empty();
    let hint = draft_hint.unwrap_or_default();
    let send_disabled = submit_text(&draft_text).is_none();
    let class = class_list("kit-mrnp", &extra_class);
    let announcement = announce();

    let views: Vec<StepView> = plan_steps
        .iter()
        .map(|step| StepView {
            label: step.label.clone(),
            phase_key: step.phase.as_str(),
            phase_word: step.phase.word(),
            active: step.phase == StepPhase::Active,
            time: step_time_ms(step, run_started_ms, now).map(format_clock),
            notes: step
                .step_notes
                .iter()
                .map(|note| TakenView {
                    text: note.text.clone(),
                    placed_key: note.placed.map(NotePlacement::as_str),
                    placed_label: note.placed.map(NotePlacement::label),
                })
                .collect(),
        })
        .collect();
    let queue: Vec<(String, String, String, String)> = queue_visible(&queued_notes, &plan_steps)
        .into_iter()
        .map(|note| {
            let apply = format!("Apply now: {}", note.text);
            let remove = format!("Remove note: {}", note.text);
            (note.text, note.note_key, apply, remove)
        })
        .collect();
    let queue_title = format!("Queued notes ({})", queue.len());

    let draft_for_keys = draft_text.clone();
    let draft_for_send = draft_text.clone();
    let on_keydown = move |event: KeyboardEvent| {
        let mods = event.modifiers();
        let chord = mods.contains(Modifiers::CONTROL) || mods.contains(Modifiers::META);
        let shift = mods.contains(Modifiers::SHIFT);
        match composer_action(&event.key(), chord, shift) {
            ComposerAction::Queue => {
                event.prevent_default();
                if let Some(text) = submit_text(&draft_for_keys) {
                    on_queue_note.call(text);
                    on_draft_change.call(String::new());
                    announce.set("Note queued.".to_string());
                }
            }
            ComposerAction::ApplyNow => {
                event.prevent_default();
                if let Some(text) = submit_text(&draft_for_keys) {
                    on_apply_note_now.call((text, None));
                    on_draft_change.call(String::new());
                    announce.set("Applying note now.".to_string());
                }
            }
            ComposerAction::Clear => {
                event.prevent_default();
                on_draft_change.call(String::new());
            }
            ComposerAction::Newline | ComposerAction::Ignore => {}
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            class,
            "data-run-state": run_state.as_str(),
            "aria-labelledby": "{heading_id}",
            header { class: "kit-mrnp__head",
                div { class: "kit-mrnp__id",
                    h2 { id: "{heading_id}", class: "kit-mrnp__heading", "{run_heading}" }
                    p { class: "kit-mrnp__meta",
                        if has_agent {
                            span { class: "kit-mrnp__agent", "{agent_name}" }
                        }
                        span { class: "kit-mrnp__step", "{step_label}" }
                        span { class: "kit-mrnp__clock", "{clock}" }
                        span { class: "kit-mrnp__state", "{state_label}" }
                    }
                }
                if let Some(next) = next_state {
                    button {
                        r#type: "button",
                        class: "kit-mrnp__run",
                        onclick: move |_| on_run_state_request.call(next),
                        "{run_label}"
                    }
                }
            }
            p { class: "kit-mrnp__sr", role: "status", "aria-live": "polite", "{announcement}" }
            ol { class: "kit-mrnp__plan", "aria-label": "Plan",
                for StepView { label, phase_key, phase_word, active, time, notes } in views {
                    li {
                        class: "kit-mrnp__step-row",
                        "data-phase": phase_key,
                        "aria-current": active.then_some("step"),
                        div { class: "kit-mrnp__step-line",
                            span { class: "kit-mrnp__mark", "aria-hidden": "true" }
                            span { class: "kit-mrnp__sr", "{phase_word}:" }
                            span { class: "kit-mrnp__label", "{label}" }
                            if let Some(time) = time {
                                span { class: "kit-mrnp__time", "{time}" }
                            }
                        }
                        if !notes.is_empty() {
                            ul { class: "kit-mrnp__taken", "aria-label": "Notes taken in",
                                for TakenView { text, placed_key, placed_label } in notes {
                                    li { class: "kit-mrnp__taken-note",
                                        if let Some(label) = placed_label {
                                            span { class: "kit-mrnp__placed", "data-placed": placed_key, "{label}" }
                                        }
                                        span { class: "kit-mrnp__note-text", "{text}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !finished && !queue.is_empty() {
                section { class: "kit-mrnp__queue", "aria-labelledby": "{queue_id}",
                    h3 { id: "{queue_id}", class: "kit-mrnp__queue-title", "{queue_title}" }
                    ul { class: "kit-mrnp__queue-list",
                        for (text, key, apply_label, remove_label) in queue {
                            li { class: "kit-mrnp__queued",
                                p { class: "kit-mrnp__queued-text", "{text}" }
                                div { class: "kit-mrnp__queued-actions",
                                    button {
                                        r#type: "button",
                                        class: "kit-mrnp__apply",
                                        "aria-label": "{apply_label}",
                                        onclick: {
                                            let text = text.clone();
                                            let key = key.clone();
                                            move |_| {
                                                on_apply_note_now.call((text.clone(), Some(key.clone())));
                                                announce.set("Applying note now.".to_string());
                                            }
                                        },
                                        "Apply now"
                                    }
                                    button {
                                        r#type: "button",
                                        class: "kit-mrnp__remove",
                                        "aria-label": "{remove_label}",
                                        onclick: {
                                            let key = key.clone();
                                            move |_| {
                                                on_drop_note.call(key.clone());
                                                announce.set("Note removed.".to_string());
                                            }
                                        },
                                        "Remove"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !finished {
                div { class: "kit-mrnp__composer",
                    label { id: "{label_id}", class: "kit-mrnp__field-label", "for": "{field_id}", "Note for the agent" }
                    textarea {
                        id: "{field_id}",
                        class: "kit-mrnp__field",
                        rows: "2",
                        placeholder: "{hint}",
                        value: "{draft_text}",
                        "aria-labelledby": "{label_id}",
                        "aria-describedby": "{hint_id}",
                        oninput: move |event: FormEvent| on_draft_change.call(event.value()),
                        onkeydown: on_keydown,
                    }
                    div { class: "kit-mrnp__composer-foot",
                        p { id: "{hint_id}", class: "kit-mrnp__hint",
                            "Enter queues. Ctrl+Enter or Cmd+Enter applies now. Shift+Enter starts a line. Esc clears."
                        }
                        button {
                            r#type: "button",
                            class: "kit-mrnp__send",
                            disabled: send_disabled,
                            onclick: move |_| {
                                if let Some(text) = submit_text(&draft_for_send) {
                                    on_queue_note.call(text);
                                    on_draft_change.call(String::new());
                                    announce.set("Note queued.".to_string());
                                }
                            },
                            "Send"
                        }
                    }
                }
            }
        }
    }
}
