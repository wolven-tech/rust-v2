//! Agent roster: every agent a person runs, with its state and access, in a grid
//! or a list. From here a person pauses or resumes an agent, messages it, opens
//! it in a panel, or adds a new one to the team.
//!
//! Design choice: a card is a button whose name carries the display name, role
//! and state. The open card is marked `aria-current`.
//! Design choice: a pause is shown at once. The button flips to the requested
//! state; the badge follows the host's `run_state`, and a pending pause clears
//! when the host's state catches up.
//! Design choice: the runs chart is a row of bars sized by the daily counts with
//! CSS, not a chart component. The roster shows seven numbers, not a series.
//! Design choice: arrow keys move focus between cards in display order. Grid and
//! list share that order.
//! Design choice: the identity palette is seven hues (`ok`, `warn`, `fault`,
//! `accent`, `teal`, `plum`, `slate`) built from the kit tokens. Any other token
//! hashes onto the palette, so the neutral mark is only for an agent with no token.
//! Browser checks: Pause on a working agent shows Paused and calls `on_pause`;
//! Ctrl/Cmd+Enter in the form creates; the grid stays two columns and one column on
//! phones; no horizontal scroll at 320 px.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, focus_after_frame, next_id};

const CSS: Asset = asset!("/assets/agent-roster.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunState {
    Working,
    Idle,
    Paused,
}

impl RunState {
    pub fn key(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Idle => "idle",
            Self::Paused => "paused",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Working => "Working",
            Self::Idle => "Idle",
            Self::Paused => "Paused",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    Anthropic,
    OpenAi,
    Google,
    Xai,
    Other,
}

impl Provider {
    fn label(self) -> &'static str {
        match self {
            Self::Anthropic => "Anthropic",
            Self::OpenAi => "OpenAI",
            Self::Google => "Google",
            Self::Xai => "xAI",
            Self::Other => "Other",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RosterView {
    #[default]
    Grid,
    List,
}

impl RosterView {
    fn key(self) -> &'static str {
        match self {
            Self::Grid => "grid",
            Self::List => "list",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppRef {
    pub id: String,
    pub entry_name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentTask {
    pub entry_title: String,
    pub task_state: String,
    pub occurred_at: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RosterEntry {
    pub id: String,
    pub display_name: String,
    pub role_label: String,
    pub identity_hue_token: Option<String>,
    pub model_name: String,
    pub model_provider: Provider,
    pub run_state: RunState,
    pub current_activity: Option<String>,
    pub last_run_label: Option<String>,
    pub runs_this_week: u32,
    pub cost_this_week: f64,
    pub daily_run_counts: Vec<u32>,
    pub owner_name: Option<String>,
    pub instruction_text: Option<String>,
    pub skill_names: Vec<String>,
    pub app_refs: Vec<AppRef>,
    pub recent_tasks: Vec<RecentTask>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewAgent {
    pub name: String,
    pub purpose: String,
    pub model: String,
    pub tone: String,
}

/// Counts of agents by run state: working, idle, paused.
pub fn state_counts(entries: &[RosterEntry]) -> (usize, usize, usize) {
    let count = |state: RunState| entries.iter().filter(|e| e.run_state == state).count();
    (
        count(RunState::Working),
        count(RunState::Idle),
        count(RunState::Paused),
    )
}

/// The summary line: `2 working · 1 idle · 0 paused`.
pub fn summary_text(entries: &[RosterEntry]) -> String {
    let (working, idle, paused) = state_counts(entries);
    format!("{working} working · {idle} idle · {paused} paused")
}

/// The name a card is announced with.
pub fn card_name(entry: &RosterEntry, state: RunState) -> String {
    format!(
        "{}, {}, {}",
        entry.display_name,
        entry.role_label,
        state.label()
    )
}

/// The state an agent shows once a pause request is applied.
pub fn paused_after_request(current: RunState, pause: bool) -> RunState {
    match (pause, current) {
        (true, _) => RunState::Paused,
        (false, RunState::Paused) => RunState::Idle,
        (false, other) => other,
    }
}

/// The initial mark for an agent's name, or `?` when it has none.
pub fn name_mark(name: &str) -> String {
    name.chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().collect())
}

/// Whether the New agent form can create an agent.
pub fn form_ready(name: &str) -> bool {
    !name.trim().is_empty()
}

const HUE_PALETTE: [&str; 7] = ["ok", "warn", "fault", "accent", "teal", "plum", "slate"];

/// A palette name is used as it is; any other token hashes onto the palette.
fn hue_of(token: Option<&str>) -> &'static str {
    let Some(token) = token.map(str::trim).filter(|t| !t.is_empty()) else {
        return "neutral";
    };
    if let Some(known) = HUE_PALETTE.iter().find(|hue| **hue == token) {
        return known;
    }
    let hash = token
        .bytes()
        .fold(5381u32, |h, b| h.wrapping_mul(33) ^ u32::from(b));
    HUE_PALETTE[hash as usize % HUE_PALETTE.len()]
}

/// Grid or list of agents, with a panel for the open agent and a form that creates
/// a new one. Pause and Resume call back with the agent and the requested state.
#[component]
pub fn AgentRoster(
    #[props(default)] roster_entries: Vec<RosterEntry>,
    #[props(default)] initial_view: RosterView,
    #[props(default)] initial_open_id: Option<String>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_pause: Option<EventHandler<(RosterEntry, bool)>>,
    #[props(default)] on_message: Option<EventHandler<RosterEntry>>,
    #[props(default)] on_create: Option<EventHandler<NewAgent>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
) -> Element {
    let uid = use_hook(next_id);
    let mut view = use_signal(move || initial_view);
    let mut open_id = use_signal(move || initial_open_id.clone());
    let mut pending = use_signal(Vec::<(String, bool)>::new);
    let mut form_open = use_signal(|| false);
    let mut form_name = use_signal(String::new);
    let mut form_purpose = use_signal(String::new);
    let mut form_model = use_signal(String::new);
    let mut form_tone = use_signal(|| "Plain".to_string());
    let mut announce = use_signal(String::new);

    let summary = summary_text(&roster_entries);
    let order: Vec<String> = roster_entries.iter().map(|e| e.id.clone()).collect();
    let open_entry = open_id().and_then(|id| roster_entries.iter().find(|e| e.id == id).cloned());
    let pending_now = pending();
    let name_now = form_name();
    let mark_now = name_mark(&name_now);
    let ready = form_ready(&name_now);
    let current_view = view();
    let view_key = current_view.key();
    let form_model_now = form_model();
    let tone_now = form_tone();
    let form_id = format!("kit-ro-{uid}-form");
    let grid_base = format!("kit-ro-{uid}-card");

    let mut create = move || {
        let name = form_name().trim().to_string();
        if !form_ready(&name) {
            return;
        }
        let agent = NewAgent {
            name: name.clone(),
            purpose: form_purpose().trim().to_string(),
            model: form_model(),
            tone: form_tone(),
        };
        if let Some(handler) = on_create {
            handler.call(agent);
        }
        announce.set(format!("Created {name}"));
        form_name.set(String::new());
        form_purpose.set(String::new());
        form_open.set(false);
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: "kit-ro",
            "data-loading": is_loading.then_some("true"),
            "aria-busy": is_loading.then_some("true"),
            header { class: "kit-ro__head",
                div { class: "kit-ro__heading",
                    h2 { class: "kit-ro__title", "Agents" }
                    p { class: "kit-ro__summary", "{summary}" }
                }
                div { class: "kit-ro__tools",
                    div { class: "kit-ro__views", role: "radiogroup", "aria-label": "Layout",
                        for option in [RosterView::Grid, RosterView::List] {
                            {
                                let option_key = option.key();
                                let option_label = if option == RosterView::Grid { "Grid" } else { "List" };
                                let option_checked = (current_view == option).to_string();
                                let option_tab = if current_view == option { "0" } else { "-1" };
                                rsx! {
                                    button {
                                        key: "{option_key}",
                                        r#type: "button",
                                        role: "radio",
                                        class: "kit-ro__view",
                                        "aria-checked": "{option_checked}",
                                        tabindex: "{option_tab}",
                                        "aria-label": "{option_label} layout",
                                        onclick: move |_| view.set(option),
                                        "{option_label}"
                                    }
                                }
                            }
                        }
                    }
                    button {
                        id: "kit-ro-{uid}-new",
                        r#type: "button",
                        class: "kit-ro__primary",
                        "aria-expanded": form_open().to_string(),
                        "aria-controls": "{form_id}",
                        onclick: move |_| form_open.toggle(),
                        "New agent"
                    }
                }
            }

            if form_open() {
                form {
                    id: "{form_id}",
                    class: "kit-ro__form",
                    "aria-label": "New agent",
                    onsubmit: move |event: FormEvent| {
                        event.prevent_default();
                        create();
                    },
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape {
                            form_open.set(false);
                            focus_after_frame(&format!("kit-ro-{uid}-new"));
                        } else if event.key() == Key::Enter && (event.modifiers().ctrl() || event.modifiers().meta()) {
                            event.prevent_default();
                            create();
                        }
                    },
                    div { class: "kit-ro__preview", "aria-hidden": "true",
                        span { class: "kit-ro__mark", "data-hue": "accent", "{mark_now}" }
                        span { class: "kit-ro__meta", "Preview" }
                    }
                    label { class: "kit-ro__field",
                        span { "Name" }
                        input {
                            class: "kit-ro__input",
                            r#type: "text",
                            value: "{name_now}",
                            oninput: move |event| form_name.set(event.value()),
                        }
                    }
                    label { class: "kit-ro__field",
                        span { "Purpose" }
                        textarea {
                            class: "kit-ro__input",
                            rows: "3",
                            value: "{form_purpose}",
                            oninput: move |event| form_purpose.set(event.value()),
                        }
                    }
                    label { class: "kit-ro__field",
                        span { "Model" }
                        select {
                            class: "kit-ro__input",
                            value: "{form_model_now}",
                            onchange: move |event| form_model.set(event.value()),
                            optgroup { label: "Anthropic",
                                option { value: "claude-sonnet", "Claude Sonnet" }
                                option { value: "claude-haiku", "Claude Haiku" }
                            }
                            optgroup { label: "OpenAI",
                                option { value: "gpt-mini", "GPT mini" }
                            }
                        }
                    }
                    fieldset { class: "kit-ro__tones",
                        legend { "Tone" }
                        for tone in ["Plain", "Warm", "Formal"] {
                            {
                                let checked = (tone_now == tone).to_string();
                                rsx! {
                                    label { key: "{tone}", class: "kit-ro__tone",
                                        input {
                                            r#type: "radio",
                                            name: "kit-ro-{uid}-tone",
                                            checked: "{checked}",
                                            onchange: move |_| form_tone.set(tone.to_string()),
                                        }
                                        "{tone}"
                                    }
                                }
                            }
                        }
                    }
                    div { class: "kit-ro__form-actions",
                        button {
                            r#type: "button",
                            class: "kit-ro__ghost",
                            onclick: move |_| form_open.set(false),
                            "Cancel"
                        }
                        button {
                            r#type: "submit",
                            class: "kit-ro__primary",
                            disabled: !ready,
                            "Create agent"
                        }
                    }
                }
            }

            if is_empty && !is_loading {
                div { class: "kit-ro__empty",
                    p { "No agents yet. Add one to start your team." }
                    button {
                        r#type: "button",
                        class: "kit-ro__primary",
                        onclick: move |_| form_open.set(true),
                        "New agent"
                    }
                }
            } else if is_loading {
                div { class: "kit-ro__cards", "aria-hidden": "true",
                    for n in 0..4 {
                        div { key: "{n}", class: "kit-ro__ph" }
                    }
                }
            } else {
                div { class: "kit-ro__body",
                    ul {
                        class: "kit-ro__cards",
                        "data-view": "{view_key}",
                        role: "list",
                        for entry in roster_entries.iter().cloned() {
                            {
                                let card_key = entry.id.clone();
                                let requested = pending_now.iter().find(|(id, _)| *id == entry.id).map(|(_, p)| *p);
                                let state = match requested {
                                    Some(pause) => paused_after_request(entry.run_state, pause),
                                    None => entry.run_state,
                                };
                                let is_open = open_id().as_deref() == Some(entry.id.as_str());
                                let name = card_name(&entry, state);
                                let mark = name_mark(&entry.display_name);
                                let hue = hue_of(entry.identity_hue_token.as_deref());
                                let state_key = state.key();
                                let state_label = state.label();
                                let model = format!("{} · {}", entry.model_name, entry.model_provider.label());
                                let runs = entry.runs_this_week.to_string();
                                let cost = format!("${:.2}", entry.cost_this_week);
                                let display = entry.display_name.clone();
                                let role = entry.role_label.clone();
                                let card_id = format!("{grid_base}-{}", entry.id);
                                let focus_order = order.clone();
                                let self_id = entry.id.clone();
                                let grid_focus = grid_base.clone();
                                rsx! {
                                    li { key: "{card_key}", class: "kit-ro__item",
                                        button {
                                            id: "{card_id}",
                                            r#type: "button",
                                            class: "kit-ro__card",
                                            "data-state": "{state_key}",
                                            "aria-label": "{name}",
                                            "aria-current": is_open.then_some("true"),
                                            onclick: {
                                                let pick = self_id.clone();
                                                move |_| open_id.set(Some(pick.clone()))
                                            },
                                            onkeydown: move |event: KeyboardEvent| {
                                                let here = focus_order.iter().position(|id| *id == self_id).unwrap_or(0);
                                                let next = match event.key() {
                                                    Key::ArrowRight | Key::ArrowDown => here + 1,
                                                    Key::ArrowLeft | Key::ArrowUp => here.saturating_sub(1),
                                                    _ => return,
                                                };
                                                if let Some(target) = focus_order.get(next) {
                                                    focus_after_frame(&format!("{grid_focus}-{target}"));
                                                }
                                            },
                                            span { class: "kit-ro__mark", "data-hue": "{hue}", "aria-hidden": "true", "{mark}" }
                                            span { class: "kit-ro__body-text",
                                                span { class: "kit-ro__name", "{display}" }
                                                span { class: "kit-ro__role", "{role}" }
                                            }
                                            span { class: "kit-ro__badge", "data-state": "{state_key}", "{state_label}" }
                                            span { class: "kit-ro__figures",
                                                span { "{model}" }
                                                span { "{runs} runs · {cost}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(entry) = open_entry.clone() {
                        {
                            let requested = pending_now.iter().find(|(id, _)| *id == entry.id).map(|(_, p)| *p);
                            let state = match requested {
                                Some(pause) => paused_after_request(entry.run_state, pause),
                                None => entry.run_state,
                            };
                            let paused_now = state == RunState::Paused;
                            let pause_label = if paused_now { "Resume" } else { "Pause" };
                            let state_key = state.key();
                            let state_label = state.label();
                            let title = entry.display_name.clone();
                            let subtitle = format!("{} · {}", entry.role_label, entry.model_name);
                            let activity = entry.current_activity.clone().unwrap_or_else(|| "No current activity.".to_string());
                            let instructions = entry.instruction_text.clone().unwrap_or_default();
                            let instruction_lines: Vec<String> = instructions.lines().map(|l| l.to_string()).collect();
                            let short_lines: Vec<String> = instruction_lines.iter().take(3).cloned().collect();
                            let long_text = instruction_lines.len() > 3;
                            let bars: Vec<u32> = entry.daily_run_counts.clone();
                            let peak = bars.iter().copied().max().unwrap_or(1).max(1);
                            let pause_entry = entry.clone();
                            let message_entry = entry.clone();
                            let nav_skills = entry.skill_names.len();
                            let skills_line = entry.skill_names.join(", ");
                            let apps = entry.app_refs.clone();
                            let recent = entry.recent_tasks.clone();
                            let cost = format!("${:.2}", entry.cost_this_week);
                            let owner = entry.owner_name.clone().unwrap_or_else(|| "You".to_string());
                            let last_run = entry.last_run_label.clone().unwrap_or_else(|| "No runs yet".to_string());
                            let nav_skills_label = format!("Skills ({nav_skills})");
                            rsx! {
                                aside { class: "kit-ro__panel", "aria-label": "Agent details",
                                    div { class: "kit-ro__panel-head",
                                        span { class: "kit-ro__mark kit-ro__mark--lg", "aria-hidden": "true", "{title}" }
                                        div { class: "kit-ro__panel-meta",
                                            h3 { class: "kit-ro__panel-title", "{title}" }
                                            p { class: "kit-ro__meta", "{subtitle}" }
                                        }
                                    }
                                    div { class: "kit-ro__state-row",
                                        span { class: "kit-ro__badge", "data-state": "{state_key}", "{state_label}" }
                                        button {
                                            r#type: "button",
                                            class: "kit-ro__ghost",
                                            onclick: move |_| {
                                                let want = !paused_now;
                                                pending.with_mut(|list| {
                                                    list.retain(|(id, _)| *id != pause_entry.id);
                                                    list.push((pause_entry.id.clone(), want));
                                                });
                                                announce.set(format!("{}: {}", pause_entry.display_name, if want { "paused" } else { "resumed" }));
                                                if let Some(handler) = on_pause {
                                                    handler.call((pause_entry.clone(), want));
                                                }
                                            },
                                            "{pause_label}"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "kit-ro__primary",
                                            onclick: move |_| {
                                                if let Some(handler) = on_message {
                                                    handler.call(message_entry.clone());
                                                } else if let Some(nav) = on_navigate {
                                                    nav.call("Thread".to_string());
                                                }
                                            },
                                            "Message"
                                        }
                                    }
                                    section { class: "kit-ro__block", "aria-label": "Overview",
                                        h4 { class: "kit-ro__section", "Overview" }
                                        p { class: "kit-ro__meta", "{activity}" }
                                        p { class: "kit-ro__meta", "Last run: {last_run}" }
                                        div { class: "kit-ro__bars", role: "img", "aria-label": "Runs per day",
                                            for (index, value) in bars.iter().enumerate() {
                                                {
                                                    let bar_key = format!("{index}");
                                                    let height = format!("{:.0}%", f64::from(*value) / f64::from(peak) * 100.0);
                                                    rsx! { span { key: "{bar_key}", class: "kit-ro__bar", style: "height: {height};" } }
                                                }
                                            }
                                        }
                                        p { class: "kit-ro__meta", "{cost} this week, {owner} owns it" }
                                    }
                                    section { class: "kit-ro__block", "aria-label": "Instructions",
                                        h4 { class: "kit-ro__section", "Instructions" }
                                        if instruction_lines.is_empty() {
                                            p { class: "kit-ro__meta", "No instructions set." }
                                        }
                                        for (index, line) in short_lines.iter().enumerate() {
                                            p { key: "{index}", class: "kit-ro__meta", "{line}" }
                                        }
                                        if long_text {
                                            details {
                                                summary { "Show all" }
                                                for (index, line) in instruction_lines.iter().skip(3).enumerate() {
                                                    p { key: "{index}", class: "kit-ro__meta", "{line}" }
                                                }
                                            }
                                        }
                                    }
                                    section { class: "kit-ro__block", "aria-label": "Skills and apps",
                                        button {
                                            r#type: "button",
                                            class: "kit-ro__link",
                                            onclick: move |_| {
                                                if let Some(nav) = on_navigate {
                                                    nav.call("Skills".to_string());
                                                }
                                            },
                                            "{nav_skills_label}"
                                        }
                                        p { class: "kit-ro__meta", "{skills_line}" }
                                        button {
                                            r#type: "button",
                                            class: "kit-ro__link",
                                            onclick: move |_| {
                                                if let Some(nav) = on_navigate {
                                                    nav.call("Permissions".to_string());
                                                }
                                            },
                                            "Apps"
                                        }
                                        ul { class: "kit-ro__apps",
                                            for app in apps.iter().cloned() {
                                                {
                                                    let app_key = app.id.clone();
                                                    let app_name = app.entry_name.clone();
                                                    rsx! { li { key: "{app_key}", "{app_name}" } }
                                                }
                                            }
                                        }
                                    }
                                    section { class: "kit-ro__block", "aria-label": "Recent tasks",
                                        h4 { class: "kit-ro__section", "Recent tasks" }
                                        if recent.is_empty() {
                                            p { class: "kit-ro__meta", "No tasks yet." }
                                        }
                                        ul { class: "kit-ro__tasks",
                                            for (index, task) in recent.iter().enumerate() {
                                                {
                                                    let task_key = format!("{index}");
                                                    let task_title = task.entry_title.clone();
                                                    let task_state = task.task_state.clone();
                                                    rsx! {
                                                        li { key: "{task_key}", span { "{task_title}" } span { class: "kit-ro__meta", "{task_state}" } }
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

            p { class: "kit-sr", role: "status", "aria-live": "polite", "{summary}" }
            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
