//! Agent action log: each step an agent took inside the apps it is connected to,
//! written as a plain sentence, with who approved it and the steps that can
//! still be reversed. A detail panel shows the open step.
//!
//! Design choice: Undo is optimistic. The row shows Undone at once and `on_undo`
//! fires. The host confirms by setting `is_undone`. If the host then sets it back
//! to false after a confirmed undo, the row reverts and an error line appears in
//! the panel.
//! Design choice: the day headings and `HH:MM` times are UTC. Without `clock_ms`
//! nothing is marked expired.
//! Design choice: a row is a `div` holding a hit button and a sibling trailing
//! control, so Undo is never nested inside a button.
//! Design choice: at 640 px and below the open step replaces the list full width
//! with a Back control; the list returns and focus goes back to the row.
//! Browser checks: J/K and arrow keys move the focused row; Enter opens; U
//! reverses; "/" focuses search and Escape clears it; the live region reads
//! "Undone: <sentence>"; left and right arrows cycle the filters; no horizontal
//! scroll at 320 px.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, focus_after_frame, next_id, use_now_ms};

const CSS: Asset = asset!("/assets/agent-action-log.css");

const DAY_MS: u64 = 86_400_000;

#[derive(Clone, Debug, PartialEq)]
pub struct AppRecord {
    pub id: String,
    pub entry_name: String,
    pub account: Option<String>,
    /// Short text drawn in the app's tile; without one a letter tile is shown.
    pub mark: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Actor {
    Automatic,
    Approved,
}

impl Actor {
    fn word(self) -> &'static str {
        match self {
            Self::Automatic => "Automatic",
            Self::Approved => "You approved",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DetailRow {
    pub entry_label: String,
    pub entry_value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UndoWindow {
    pub available_until: u64,
    pub entry_description: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StepRecord {
    pub id: String,
    pub occurred_at: u64,
    pub app_id: String,
    pub sentence: String,
    pub task_name: Option<String>,
    pub actor: Actor,
    pub rule_label: Option<String>,
    pub approved_at: Option<u64>,
    pub detail_rows: Vec<DetailRow>,
    pub preview_lines: Vec<String>,
    pub undo_window: Option<UndoWindow>,
    pub final_reason: Option<String>,
    pub is_undone: bool,
    pub undone_at: Option<u64>,
    pub cost_label: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepFilter {
    #[default]
    All,
    Automatic,
    Approved,
    Undone,
}

impl StepFilter {
    fn all() -> [StepFilter; 4] {
        [Self::All, Self::Automatic, Self::Approved, Self::Undone]
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Automatic => "automatic",
            Self::Approved => "approved",
            Self::Undone => "undone",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Automatic => "Automatic",
            Self::Approved => "Approved",
            Self::Undone => "Undone",
        }
    }

    pub fn admits(self, step: &StepRecord) -> bool {
        match self {
            Self::All => true,
            Self::Automatic => step.actor == Actor::Automatic,
            Self::Approved => step.actor == Actor::Approved,
            Self::Undone => step.is_undone,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UndoState {
    Available,
    Expired,
    Final,
    Undone,
    None,
}

impl UndoState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Expired => "expired",
            Self::Final => "final",
            Self::Undone => "undone",
            Self::None => "none",
        }
    }
}

/// Whether a step can still be reversed at `now_ms`.
pub fn undo_state(step: &StepRecord, now_ms: u64) -> UndoState {
    if step.is_undone {
        UndoState::Undone
    } else if step.final_reason.is_some() {
        UndoState::Final
    } else {
        match &step.undo_window {
            Some(window) if now_ms < window.available_until => UndoState::Available,
            Some(_) => UndoState::Expired,
            None => UndoState::None,
        }
    }
}

/// `HH:MM` in UTC.
pub fn clock_text(ms: u64) -> String {
    let minutes = (ms / 60_000) % 1440;
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

/// `Today`, `Yesterday`, or `N days ago`, relative to `now_ms`.
pub fn day_heading(now_ms: u64, then_ms: u64) -> String {
    let days = (now_ms / DAY_MS).saturating_sub(then_ms / DAY_MS);
    match days {
        0 => "Today".to_string(),
        1 => "Yesterday".to_string(),
        n => format!("{n} days ago"),
    }
}

pub fn app_name(apps: &[AppRecord], id: &str) -> String {
    apps.iter()
        .find(|a| a.id == id)
        .map_or_else(|| "Unknown app".to_string(), |a| a.entry_name.clone())
}

/// Steps matching the filter and the search text, newest first.
pub fn visible_steps(
    steps: &[StepRecord],
    apps: &[AppRecord],
    filter: StepFilter,
    query: &str,
) -> Vec<StepRecord> {
    let needle = query.trim().to_lowercase();
    let mut list: Vec<StepRecord> = steps
        .iter()
        .filter(|step| filter.admits(step))
        .filter(|step| {
            needle.is_empty()
                || step.sentence.to_lowercase().contains(&needle)
                || step
                    .task_name
                    .as_deref()
                    .is_some_and(|task| task.to_lowercase().contains(&needle))
                || app_name(apps, &step.app_id)
                    .to_lowercase()
                    .contains(&needle)
        })
        .cloned()
        .collect();
    list.sort_by_key(|step| std::cmp::Reverse(step.occurred_at));
    list
}

/// Counts for the filters, in filter order: all, automatic, approved, undone.
pub fn filter_counts(steps: &[StepRecord]) -> [usize; 4] {
    [
        steps.len(),
        steps
            .iter()
            .filter(|s| StepFilter::Automatic.admits(s))
            .count(),
        steps
            .iter()
            .filter(|s| StepFilter::Approved.admits(s))
            .count(),
        steps
            .iter()
            .filter(|s| StepFilter::Undone.admits(s))
            .count(),
    ]
}

/// The id `delta` rows from `current` in `order`, clamped at both ends.
pub fn move_index(order: &[String], current: Option<&str>, delta: isize) -> Option<String> {
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

/// Live-region text after a reversal.
pub fn undone_announcement(sentence: &str) -> String {
    format!("Undone: {sentence}")
}

/// Live-region and panel text when a reversal is refused.
pub fn undo_failed_text(sentence: &str) -> String {
    format!("Could not undo: {sentence}")
}

fn tile_letter(name: &str) -> String {
    name.chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().collect())
}

/// Everything one row shows, computed before the markup.
#[derive(Clone, Debug, PartialEq)]
struct RowView {
    key: String,
    day: Option<String>,
    time: String,
    sentence: String,
    task: String,
    actor: &'static str,
    tile: String,
    app: String,
    hit_label: String,
    undo_label: String,
    undone: bool,
    state: UndoState,
    is_open: bool,
    step: StepRecord,
}

/// Lists the steps an agent took, newest first, under day headings, with an
/// open-step panel. Undo calls back with the step; the host confirms by setting
/// `is_undone`.
#[component]
pub fn AgentActionLog(
    #[props(default)] app_list: Vec<AppRecord>,
    #[props(default)] step_list: Vec<StepRecord>,
    #[props(default)] selected_step_id: Option<String>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] clock_ms: Option<u64>,
    #[props(default)] on_undo: Option<EventHandler<StepRecord>>,
    #[props(default)] on_open_task: Option<EventHandler<String>>,
    #[props(default)] on_select: Option<EventHandler<String>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
) -> Element {
    let now = use_now_ms(clock_ms, 1000);
    let uid = use_hook(next_id);
    let mut filter = use_signal(StepFilter::default);
    let mut query = use_signal(String::new);
    let initial_open = selected_step_id
        .clone()
        .or_else(|| step_list.first().map(|s| s.id.clone()));
    let mut open_id = use_signal(move || initial_open.clone());
    let mut show_all = use_signal(|| false);
    let mut optimistic = use_signal(Vec::<String>::new);
    let mut failed = use_signal(Vec::<String>::new);
    let mut seen = use_signal(Vec::<(String, bool)>::new);
    let mut announce = use_signal(String::new);
    let mut detail_open = use_signal(|| false);

    // The host's `is_undone` is authoritative. A step that goes from undone back to
    // not undone is a refused reversal: report it and show the row as it was.
    let current: Vec<(String, bool)> = step_list
        .iter()
        .map(|s| (s.id.clone(), s.is_undone))
        .collect();
    if seen.peek().as_slice() != current.as_slice() {
        let previous = seen.peek().clone();
        for (id, undone) in &current {
            if *undone {
                optimistic.with_mut(|list| list.retain(|v| v != id));
                continue;
            }
            let was_undone = previous.iter().any(|(p, u)| p == id && *u);
            if was_undone {
                if let Some(step) = step_list.iter().find(|s| &s.id == id) {
                    failed.with_mut(|list| list.push(id.clone()));
                    announce.set(undo_failed_text(&step.sentence));
                }
            }
        }
        seen.set(current);
    }

    let visible = visible_steps(&step_list, &app_list, filter(), &query());
    let order: Vec<String> = visible.iter().map(|s| s.id.clone()).collect();
    let counts = filter_counts(&step_list);
    let open_step = open_id().and_then(|id| step_list.iter().find(|s| s.id == id).cloned());
    let open_now = open_id();
    let pending = optimistic();
    let failed_now = failed();
    let search_id = format!("kit-al-{uid}-search");
    let list_id = format!("kit-al-{uid}-list");
    let hit_prefix = format!("kit-al-{uid}-hit-");

    let rows: Vec<RowView> = visible
        .iter()
        .enumerate()
        .map(|(index, step)| {
            let undone = step.is_undone || pending.contains(&step.id);
            let state = if undone {
                UndoState::Undone
            } else {
                undo_state(step, now)
            };
            let app = app_name(&app_list, &step.app_id);
            let task = step
                .task_name
                .clone()
                .unwrap_or_else(|| "No task".to_string());
            let time = clock_text(step.occurred_at);
            let day = match index {
                0 => Some(day_heading(now, step.occurred_at)),
                _ => {
                    let today = day_heading(now, step.occurred_at);
                    let before = day_heading(now, visible[index - 1].occurred_at);
                    (today != before).then_some(today)
                }
            };
            let state_word = match state {
                UndoState::Undone => "undone",
                UndoState::Available => "undo available",
                UndoState::Expired => "undo expired",
                UndoState::Final => "final",
                UndoState::None => "no undo",
            };
            RowView {
                key: step.id.clone(),
                day,
                time: time.clone(),
                sentence: step.sentence.clone(),
                task: task.clone(),
                actor: step.actor.word(),
                tile: tile_letter(&app),
                app: app.clone(),
                hit_label: format!(
                    "{time}, {}, {app}, {task}, {}, {state_word}",
                    step.sentence,
                    step.actor.word()
                ),
                undo_label: format!("Undo: {}", step.sentence),
                undone,
                state,
                is_open: open_now.as_deref() == Some(step.id.as_str()),
                step: step.clone(),
            }
        })
        .collect();

    let key_order = order.clone();
    let search_focus = search_id.clone();
    let list_focus = list_id.clone();
    let filter_base = format!("kit-al-{uid}-filter");

    let mut undo_step = move |step: StepRecord| {
        if undo_state(&step, now) != UndoState::Available {
            return;
        }
        optimistic.with_mut(|list| list.push(step.id.clone()));
        failed.with_mut(|list| list.retain(|v| v != &step.id));
        announce.set(undone_announcement(&step.sentence));
        if let Some(handler) = on_undo {
            handler.call(step);
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: "kit-al",
            "data-loading": is_loading.then_some("true"),
            "aria-busy": is_loading.then_some("true"),
            "data-detail": detail_open().then_some("open"),
            onkeydown: move |event: KeyboardEvent| {
                let Key::Character(c) = event.key() else {
                    return;
                };
                match c.as_str() {
                    "/" => {
                        event.prevent_default();
                        focus_after_frame(&search_focus);
                    }
                    "j" | "k" => {
                        let delta = if c == "j" { 1 } else { -1 };
                        if let Some(next) = move_index(&key_order, open_id().as_deref(), delta) {
                            open_id.set(Some(next.clone()));
                            if let Some(handler) = on_select {
                                handler.call(next);
                            }
                        }
                    }
                    "u" => {
                        let target = open_id()
                            .and_then(|id| step_list.iter().find(|s| s.id == id).cloned());
                        if let Some(step) = target {
                            undo_step(step);
                        }
                    }
                    _ => {}
                }
            },
            header { class: "kit-al__head",
                h2 { class: "kit-al__title", "Activity" }
                div { class: "kit-al__tools",
                    div { class: "kit-al__filters", role: "radiogroup", "aria-label": "Filter steps",
                        for option in StepFilter::all() {
                            {
                                let option_key = option.as_str();
                                let option_label = option.label();
                                let option_count = counts[option as usize];
                                let option_dom = format!("{filter_base}-{option_key}");
                                let option_checked = (filter() == option).to_string();
                                let option_tab = if filter() == option { "0" } else { "-1" };
                                let filter_focus = filter_base.clone();
                                rsx! {
                                    button {
                                        key: "{option_key}",
                                        id: "{option_dom}",
                                        r#type: "button",
                                        role: "radio",
                                        class: "kit-al__filter",
                                        "aria-checked": "{option_checked}",
                                        tabindex: "{option_tab}",
                                        "data-filter": "{option_key}",
                                        onclick: move |_| filter.set(option),
                                        onkeydown: move |event: KeyboardEvent| {
                                            let all = StepFilter::all();
                                            let index = all.iter().position(|f| *f == filter()).unwrap_or(0);
                                            let next = match event.key() {
                                                Key::ArrowRight => (index + 1) % all.len(),
                                                Key::ArrowLeft => (index + all.len() - 1) % all.len(),
                                                _ => return,
                                            };
                                            filter.set(all[next]);
                                            focus_after_frame(&format!("{filter_focus}-{}", all[next].as_str()));
                                        },
                                        "{option_label} "
                                        span { class: "kit-al__count", "{option_count}" }
                                    }
                                }
                            }
                        }
                    }
                    input {
                        id: "{search_id}",
                        class: "kit-al__search",
                        r#type: "search",
                        placeholder: "Search steps",
                        "aria-label": "Search steps",
                        value: "{query}",
                        oninput: move |event| query.set(event.value()),
                        onkeydown: move |event: KeyboardEvent| {
                            if event.key() == Key::Escape {
                                query.set(String::new());
                                focus_after_frame(&list_focus);
                            }
                        },
                    }
                }
            }

            if is_empty && !is_loading {
                div { class: "kit-al__empty",
                    p { "Steps appear here as agents act in your apps." }
                    if on_navigate.is_some() {
                        button {
                            r#type: "button",
                            class: "kit-al__link",
                            onclick: move |_| {
                                if let Some(handler) = on_navigate {
                                    handler.call("Permissions".to_string());
                                }
                            },
                            "Review permissions"
                        }
                    }
                }
            } else if is_loading {
                ul { class: "kit-al__list", "aria-hidden": "true",
                    for n in 0..4 {
                        li { key: "{n}", class: "kit-al__ph-row",
                            span { class: "kit-al__ph" }
                            span { class: "kit-al__ph kit-al__ph--wide" }
                        }
                    }
                }
            } else {
                div { class: "kit-al__body",
                    ul { id: "{list_id}", class: "kit-al__list", role: "list", tabindex: "-1",
                        if rows.is_empty() {
                            li { class: "kit-al__none", "No steps match this view." }
                        }
                        for row in rows.iter().cloned() {
                            {
                                let RowView {
                                    key,
                                    day,
                                    time,
                                    sentence,
                                    task,
                                    actor,
                                    tile,
                                    app,
                                    hit_label,
                                    undo_label,
                                    undone,
                                    state,
                                    is_open,
                                    step,
                                } = row;
                                let state_key = state.as_str();
                                let open_key = key.clone();
                                let row_step = step.clone();
                                let show_day = day.is_some();
                                let day_text = day.unwrap_or_default();
                                let hit_dom = format!("{hit_prefix}{key}");
                                rsx! {
                                    li { key: "{key}", class: "kit-al__item",
                                        if show_day {
                                            div { class: "kit-al__day", role: "presentation", "{day_text}" }
                                        }
                                        div {
                                            class: "kit-al__row",
                                            "data-undone": if undone { "true" } else { "false" },
                                            "data-state": "{state_key}",
                                            "aria-current": is_open.then_some("true"),
                                            button {
                                                id: "{hit_dom}",
                                                r#type: "button",
                                                class: "kit-al__hit",
                                                "aria-label": "{hit_label}",
                                                onclick: move |_| {
                                                    open_id.set(Some(open_key.clone()));
                                                    detail_open.set(true);
                                                    if let Some(handler) = on_select {
                                                        handler.call(open_key.clone());
                                                    }
                                                },
                                                span { class: "kit-al__time", "{time}" }
                                                span { class: "kit-al__app", "aria-hidden": "true",
                                                    span { class: "kit-al__tile", "{tile}" }
                                                }
                                                span { class: "kit-al__sentence", title: "{sentence}", "{sentence}" }
                                                span { class: "kit-al__task", "{task}" }
                                                span { class: "kit-al__actor", "{actor}" }
                                                span { class: "kit-sr", "{app}" }
                                            }
                                            div { class: "kit-al__trail",
                                                if state == UndoState::Undone {
                                                    span { class: "kit-al__tag", "Undone" }
                                                } else if state == UndoState::Available {
                                                    button {
                                                        r#type: "button",
                                                        class: "kit-al__undo",
                                                        "aria-label": "{undo_label}",
                                                        onclick: move |_| undo_step(row_step.clone()),
                                                        "Undo"
                                                    }
                                                } else if state == UndoState::Final {
                                                    span { class: "kit-al__lock", "Final" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(step) = open_step.clone() {
                        {
                            let undone = step.is_undone || pending.contains(&step.id);
                            let state = if undone { UndoState::Undone } else { undo_state(&step, now) };
                            let state_key = state.as_str();
                            let app = app_name(&app_list, &step.app_id);
                            let tile = tile_letter(&app);
                            let when = clock_text(step.occurred_at);
                            let task = step.task_name.clone().unwrap_or_else(|| "No task".to_string());
                            let actor = step.actor.word();
                            let shown: Vec<String> = if show_all() {
                                step.preview_lines.clone()
                            } else {
                                step.preview_lines.iter().take(6).cloned().collect()
                            };
                            let has_more = step.preview_lines.len() > 6;
                            let more_label = if show_all() { "Show less" } else { "Show all" };
                            let why = match (&step.rule_label, step.approved_at) {
                                (Some(rule), _) => format!("Ran under the rule “{rule}”."),
                                (None, Some(at)) => format!("Approved at {}.", clock_text(at)),
                                (None, None) => "Ran automatically.".to_string(),
                            };
                            let window_text = match &step.undo_window {
                                Some(window) => format!(
                                    "You can undo this until {}. Undo will {}.",
                                    clock_text(window.available_until),
                                    window.entry_description
                                ),
                                None => String::new(),
                            };
                            let undone_text = format!(
                                "Undone at {}.",
                                clock_text(step.undone_at.unwrap_or(step.occurred_at))
                            );
                            let final_text = step.final_reason.clone().unwrap_or_default();
                            let refused = failed_now.contains(&step.id);
                            let error_text = undo_failed_text(&step.sentence);
                            let panel_sentence = step.sentence.clone();
                            let panel_when = format!("{when} · {app}");
                            let task_target = step.task_name.clone();
                            let open_task_label = format!("Open task {task}");
                            let task_text = format!("{task} →");
                            let undo_target = step.clone();
                            let facts: Vec<(String, String)> = step
                                .detail_rows
                                .iter()
                                .map(|row| (row.entry_label.clone(), row.entry_value.clone()))
                                .collect();
                            let cost = step.cost_label.clone();
                            let undo_state_key = state_key;
                            let back_target = format!("{hit_prefix}{}", step.id);
                            rsx! {
                                aside { class: "kit-al__panel", "aria-label": "Step detail", "data-state": "{undo_state_key}",
                                    button {
                                        r#type: "button",
                                        class: "kit-al__back",
                                        onclick: move |_| {
                                            detail_open.set(false);
                                            focus_after_frame(&back_target);
                                        },
                                        "Back to activity"
                                    }
                                    div { class: "kit-al__panel-head",
                                        span { class: "kit-al__tile kit-al__tile--lg", "aria-hidden": "true", "{tile}" }
                                        div { class: "kit-al__panel-meta",
                                            p { class: "kit-al__panel-sentence", "{panel_sentence}" }
                                            p { class: "kit-al__meta", "{panel_when}" }
                                        }
                                    }
                                    dl { class: "kit-al__facts",
                                        dt { "Task" }
                                        dd {
                                            if let Some(target) = task_target.clone() {
                                                button {
                                                    r#type: "button",
                                                    class: "kit-al__link",
                                                    "aria-label": "{open_task_label}",
                                                    onclick: move |_| {
                                                        if let Some(handler) = on_open_task {
                                                            handler.call(target.clone());
                                                        }
                                                    },
                                                    "{task_text}"
                                                }
                                            } else {
                                                "{task}"
                                            }
                                        }
                                        dt { "Actor" }
                                        dd { "{actor}" }
                                        if let Some(cost_text) = cost {
                                            dt { "Cost" }
                                            dd { "{cost_text}" }
                                        }
                                        for (label, value) in facts.iter().cloned() {
                                            dt { "{label}" }
                                            dd { "{value}" }
                                        }
                                    }
                                    section { class: "kit-al__block", "aria-label": "Why it could run",
                                        h3 { class: "kit-al__section-title", "Why it could" }
                                        p { "{why}" }
                                        if on_navigate.is_some() {
                                            button {
                                                r#type: "button",
                                                class: "kit-al__link",
                                                onclick: move |_| {
                                                    if let Some(handler) = on_navigate {
                                                        handler.call("Permissions".to_string());
                                                    }
                                                },
                                                "See permissions"
                                            }
                                        }
                                    }
                                    if !step.preview_lines.is_empty() {
                                        section { class: "kit-al__block",
                                            h3 { class: "kit-al__section-title", "Preview" }
                                            pre { class: "kit-al__preview",
                                                for (index, line) in shown.iter().enumerate() {
                                                    span { key: "{index}", class: "kit-al__line", "{line}" }
                                                }
                                            }
                                            if has_more {
                                                button {
                                                    r#type: "button",
                                                    class: "kit-al__link",
                                                    onclick: move |_| show_all.toggle(),
                                                    "{more_label}"
                                                }
                                            }
                                        }
                                    }
                                    section { class: "kit-al__block kit-al__undo-block",
                                        if state == UndoState::Available {
                                            p { "{window_text}" }
                                            button {
                                                r#type: "button",
                                                class: "kit-al__undo",
                                                onclick: move |_| undo_step(undo_target.clone()),
                                                "Undo"
                                            }
                                        } else if state == UndoState::Expired {
                                            p { "The window to reverse this step has closed." }
                                        } else if state == UndoState::Final {
                                            p { "{final_text}" }
                                        } else if state == UndoState::Undone {
                                            p { "{undone_text}" }
                                        } else {
                                            p { "This step cannot be reversed." }
                                        }
                                        if refused {
                                            p { class: "kit-al__error", role: "alert", "{error_text}" }
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
