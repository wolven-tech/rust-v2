//! Agent team session: a team's shared conversation, its plan and its shared context on one
//! screen. Picking an agent narrows the stream to that agent's messages and the plan to its
//! steps.
//!
//! Design choice: the context panel is `ContextFillGrid`; its cells are the team's token
//! budget, one part per segment.
//! Design choice: a message stays visible when its author is the focused agent or it names
//! the focused agent as a recipient. Broadcasts to everyone hide under a focus.
//! Design choice: the plan keeps all steps and dims the ones the focused agent does not own,
//! so step numbers stay put.
//! Design choice: the plan's live region speaks the running step at minute resolution (step
//! changes, then each whole minute), so the per-second clock stays visual.
//! Browser checks: tab focus narrows the stream, Escape clears, Home/End and Left/Right move
//! between agent tabs, Up/Down move between messages, Enter sends and Shift+Enter inserts a
//! line, `@` opens the agent list, pause stops the step advance, phone stacking.

use dioxus::prelude::*;
use rv2_affect::badge::{Badge, BadgeTone};

use crate::components::context_fill_grid::{ContextFillGrid, ContextPart};
use crate::components::series_trend_chart::SeriesTone;
use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/agent-team-session.css");

#[derive(Clone, Debug, PartialEq)]
pub struct TeamAgent {
    pub id: String,
    pub name: String,
    pub model_label: String,
    pub role: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Recipients {
    Everyone,
    Ids(Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolCallCount {
    pub name: String,
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TeamMessage {
    pub id: String,
    pub author_id: String,
    pub to: Recipients,
    pub reply_to_id: Option<String>,
    pub at_label: String,
    pub text: String,
    pub tool_calls: Vec<ToolCallCount>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Done,
    Blocked,
}

impl StepStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Done => "done",
            Self::Blocked => "blocked",
        }
    }

    fn mark(self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::Running => "◐",
            Self::Done => "✓",
            Self::Blocked => "!",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlanStep {
    pub id: String,
    pub title: String,
    pub owner_id: Option<String>,
    pub status: StepStatus,
    pub started_at_ms: Option<u64>,
    pub finished_at_ms: Option<u64>,
    pub blocked_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContextSegment {
    pub label: String,
    pub tokens: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContextUsage {
    pub used_tokens: f64,
    pub limit_tokens: f64,
    pub segments: Vec<ContextSegment>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComposeSubmit {
    pub text: String,
    pub to_ids: Vec<String>,
    pub reply_to_id: Option<String>,
}

/// Whether a message shows under the focused agent. `None` shows everything.
pub fn message_visible(message: &TeamMessage, focus: Option<&str>) -> bool {
    let Some(focus) = focus else { return true };
    message.author_id == focus
        || matches!(&message.to, Recipients::Ids(ids) if ids.iter().any(|id| id == focus))
}

/// Whether a plan step is dimmed under the focused agent.
pub fn step_dimmed(step: &PlanStep, focus: Option<&str>) -> bool {
    focus.is_some_and(|f| step.owner_id.as_deref() != Some(f))
}

/// Share of the window in use, clamped to `0..=100`.
pub fn usage_percent(usage: &ContextUsage) -> f64 {
    if usage.limit_tokens <= 0.0 {
        return 0.0;
    }
    (usage.used_tokens / usage.limit_tokens * 100.0).clamp(0.0, 100.0)
}

/// Running time of the plan: from the first step that started to `now_ms`, or to the last
/// finish once every started step has finished. `h:mm:ss` past an hour, else `m:ss`.
pub fn plan_running_label(steps: &[PlanStep], now_ms: u64) -> String {
    let Some(start) = steps.iter().filter_map(|s| s.started_at_ms).min() else {
        return "0:00".to_string();
    };
    let all_finished = steps
        .iter()
        .filter(|s| s.started_at_ms.is_some())
        .all(|s| s.finished_at_ms.is_some());
    let end = if all_finished {
        steps
            .iter()
            .filter_map(|s| s.finished_at_ms)
            .max()
            .unwrap_or(now_ms)
    } else {
        now_ms
    };
    let secs = end.saturating_sub(start) / 1000;
    if secs >= 3600 {
        format!("{}:{:02}:{:02}", secs / 3600, secs % 3600 / 60, secs % 60)
    } else {
        format!("{}:{:02}", secs / 60, secs % 60)
    }
}

/// The plan's spoken status. Minute resolution: the text changes on a step change or a
/// whole minute, so the live region does not speak on every clock tick.
pub fn step_live_text(steps: &[PlanStep], now_ms: u64) -> String {
    let total = steps.len();
    let Some((i, step)) = steps
        .iter()
        .enumerate()
        .find(|(_, s)| s.status == StepStatus::Running)
    else {
        let all_done = total > 0 && steps.iter().all(|s| s.status == StepStatus::Done);
        return if all_done {
            format!("All {total} steps done")
        } else {
            String::new()
        };
    };
    let started = step.started_at_ms.unwrap_or(now_ms);
    let minutes = now_ms.saturating_sub(started) / 60_000;
    let elapsed = if minutes == 0 {
        "under a minute".to_string()
    } else {
        format!("{minutes} min")
    };
    format!(
        "Step {} of {total}, {}: running {elapsed}",
        i + 1,
        step.title
    )
}

/// The agent tab an arrow, Home or End key moves to among `count` tabs (Everyone first);
/// Left and Right wrap, Home and End stop at the ends.
pub fn tab_target(current: usize, count: usize, key: &Key) -> Option<usize> {
    if count == 0 {
        return None;
    }
    match key {
        Key::ArrowRight => Some((current + 1) % count),
        Key::ArrowLeft => Some((current + count - 1) % count),
        Key::Home => Some(0),
        Key::End => Some(count - 1),
        _ => None,
    }
}

/// The visible message that Up or Down moves focus to from `current`; stops at either end.
pub fn message_target(ids: &[String], current: &str, key: &Key) -> Option<String> {
    let at = ids.iter().position(|id| id == current)?;
    let next = match key {
        Key::ArrowDown => (at + 1).min(ids.len() - 1),
        Key::ArrowUp => at.saturating_sub(1),
        _ => return None,
    };
    Some(ids[next].clone())
}

/// The `@` trigger: the partial name after a trailing `@`, if the text ends in one.
pub fn mention_query(text: &str) -> Option<&str> {
    let at = text.rfind('@')?;
    let partial = &text[at + 1..];
    (!partial.contains(char::is_whitespace)).then_some(partial)
}

/// Agents whose name starts with `query`, case-insensitively, as owned copies so that
/// event handlers can hold them.
pub fn agents_matching(agents: &[TeamAgent], query: &str) -> Vec<TeamAgent> {
    let query = query.to_lowercase();
    agents
        .iter()
        .filter(|a| a.name.to_lowercase().starts_with(&query))
        .cloned()
        .collect()
}

fn recipients_text(to: &Recipients, agents: &[TeamAgent]) -> String {
    match to {
        Recipients::Everyone => "everyone".to_string(),
        Recipients::Ids(ids) => ids
            .iter()
            .map(|id| {
                agents
                    .iter()
                    .find(|a| &a.id == id)
                    .map_or(id.as_str(), |a| a.name.as_str())
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(", "),
    }
}

fn name_of(agents: &[TeamAgent], id: &str) -> String {
    agents
        .iter()
        .find(|a| a.id == id)
        .map_or_else(|| id.to_string(), |a| a.name.clone())
}

#[component]
pub fn AgentTeamSession(
    task_title: String,
    team_name: String,
    agents: Vec<TeamAgent>,
    messages: Vec<TeamMessage>,
    plan_steps: Vec<PlanStep>,
    context_usage: ContextUsage,
    clock_ms: u64,
    #[props(default)] is_paused: bool,
    #[props(default)] on_toggle_pause: Option<EventHandler<bool>>,
    #[props(default)] spotlight_agent_id: Option<String>,
    #[props(default)] on_spotlight_change: Option<EventHandler<Option<String>>>,
    #[props(default)] on_message_submit: Option<EventHandler<ComposeSubmit>>,
    #[props(default = 420.0)] stream_height_cap: f64,
    #[props(default, into)] extra_classes: String,
) -> Element {
    let mut picked = use_signal(|| None::<String>);
    let mut draft = use_signal(String::new);
    let mut mention_index = use_signal(|| 0_usize);
    let mut mention_closed = use_signal(|| false);
    let mut reply_to = use_signal(|| None::<String>);

    let focus = spotlight_agent_id.clone();
    let visible: Vec<&TeamMessage> = messages
        .iter()
        .filter(|m| message_visible(m, focus.as_deref()))
        .collect();
    let visible_ids: Vec<String> = visible.iter().map(|m| m.id.clone()).collect();
    let step_live = step_live_text(&plan_steps, clock_ms);
    let mention = mention_query(&draft()).map(str::to_string);
    let suggestions: Vec<TeamAgent> = match (&mention, mention_closed()) {
        (Some(q), false) => agents_matching(&agents, q),
        _ => Vec::new(),
    };
    let live_label = if is_paused { "Paused" } else { "Live" };
    let running = plan_running_label(&plan_steps, clock_ms);
    let active_step = plan_steps
        .iter()
        .position(|s| s.status == StepStatus::Running);
    let usage = usage_percent(&context_usage);
    let context_parts: Vec<ContextPart> = context_usage
        .segments
        .iter()
        .enumerate()
        .map(|(i, s)| ContextPart {
            part_key: format!("seg-{i}"),
            part_name: s.label.clone(),
            token_count: Some(s.tokens),
            sub_items: Vec::new(),
            is_locked: false,
            can_compact: false,
            part_color: Some(match i % 4 {
                0 => SeriesTone::Ink,
                1 => SeriesTone::Ok,
                2 => SeriesTone::Warn,
                _ => SeriesTone::Muted,
            }),
        })
        .collect();
    let warn_word = if usage >= 80.0 { " · near limit" } else { "" };
    let reply_name = reply_to()
        .map(|id| name_of(&agents, &id))
        .unwrap_or_default();
    let root_class = class_list("kit-team", &extra_classes);
    let tabs_focus = focus.clone().unwrap_or_else(|| "everyone".to_string());
    let stream_cap = format!("max-height: {stream_height_cap}px;");

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{root_class}", "aria-labelledby": "kit-team-title",
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape && focus_clear_allowed(mention_closed(), &draft()) {
                    if let Some(handler) = on_spotlight_change { handler.call(None); }
                }
            },
            header { class: "kit-team__head",
                div {
                    h2 { id: "kit-team-title", class: "kit-team__title", "{task_title}" }
                    p { class: "kit-team__team", "{team_name}" }
                }
                div { class: "kit-team__live",
                    Badge { tone: if is_paused { BadgeTone::Warning } else { BadgeTone::Success }, dot: true, live: !is_paused, "{live_label}" }
                    span { class: "kit-team__elapsed", "{running}" }
                }
            }
            div { class: "kit-team__tabs", role: "tablist", "aria-label": "Show messages from",
                for (index, tab) in std::iter::once(None).chain(agents.iter().map(Some)).enumerate() {
                    {
                        let key = tab.map_or("everyone".to_string(), |a| a.id.clone());
                        let selected = tabs_focus == key;
                        let label = tab.map_or("Everyone".to_string(), |a| a.name.clone());
                        let agents_for_keys = agents.clone();
                        let next_id = key.clone();
                        rsx! {
                            button {
                                key: "{key}",
                                id: "kit-team-tab-{index}",
                                r#type: "button",
                                role: "tab",
                                class: "kit-team__tab",
                                "aria-selected": if selected { "true" } else { "false" },
                                tabindex: if selected { "0" } else { "-1" },
                                onclick: move |_| {
                                    if let Some(handler) = on_spotlight_change {
                                        handler.call(if next_id == "everyone" { None } else { Some(next_id.clone()) });
                                    }
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    if let Some(target) = tab_target(index, agents_for_keys.len() + 1, &e.key()) {
                                        e.prevent_default();
                                        let target_key = if target == 0 { None } else { Some(agents_for_keys[target - 1].id.clone()) };
                                        if let Some(handler) = on_spotlight_change {
                                            handler.call(target_key);
                                        }
                                        focus_tab(target);
                                    }
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }
            div { class: "kit-team__grid",
                section { class: "kit-team__stream", role: "log", "aria-live": "polite", "aria-label": "Messages", style: "{stream_cap}",
                    if visible.is_empty() {
                        p { class: "kit-team__empty", "No messages yet. Send the first one below." }
                    }
                    for message in visible.iter().copied() {
                        {
                            let id = message.id.clone();
                            let author = name_of(&agents, &message.author_id);
                            let to_text = recipients_text(&message.to, &agents);
                            let open = picked() == Some(id.clone());
                            let reply = message.reply_to_id.clone().map(|r| name_of(&agents, &r));
                            let dimmed = focus.as_deref().is_some_and(|f| message.author_id != f);
                            let tools: u32 = message.tool_calls.iter().map(|t| t.count).sum();
                            let tool_line = message.tool_calls.iter().map(|t| format!("{} ×{}", t.name, t.count)).collect::<Vec<_>>().join(", ");
                            let accessible = format!("{author} to {to_text} at {}", message.at_label);
                            let pick_id = id.clone();
                            rsx! {
                                article { key: "{id}", id: "kit-msg-{id}", class: "kit-team__msg", tabindex: "0",
                                    "aria-label": "{accessible}", "data-dimmed": dimmed.then_some("true"),
                                    onkeydown: {
                                        let id = id.clone();
                                        let ids = visible_ids.clone();
                                        move |e: KeyboardEvent| {
                                            if e.key() == Key::Enter {
                                                picked.set(if picked() == Some(id.clone()) { None } else { Some(id.clone()) });
                                            } else if let Some(target) = message_target(&ids, &id, &e.key()) {
                                                e.prevent_default();
                                                focus_message(&target);
                                            }
                                        }
                                    },
                                    onclick: move |_| {
                                        picked.set(if picked() == Some(pick_id.clone()) { None } else { Some(pick_id.clone()) });
                                    },
                                    header { class: "kit-team__msg-head",
                                        span { class: "kit-team__author", "{author}" }
                                        span { class: "kit-team__time", "{message.at_label}" }
                                    }
                                    p { class: "kit-team__text", "{message.text}" }
                                    if tools > 0 {
                                        p { class: "kit-team__tools", "{tools} tool calls · {tool_line}" }
                                    }
                                    if open {
                                        dl { class: "kit-team__details",
                                            dt { "To" }
                                            dd { "{to_text}" }
                                            dt { "Time" }
                                            dd { "{message.at_label}" }
                                            if let Some(reply) = reply.clone() {
                                                dt { "Reply to" }
                                                dd { "{reply}" }
                                            }
                                            if !message.tool_calls.is_empty() {
                                                dt { "Tool calls" }
                                                dd { "{tool_line}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                aside { class: "kit-team__side", "aria-label": "Plan and context",
                    section { class: "kit-team__plan", "aria-labelledby": "kit-team-plan-title",
                        header { class: "kit-team__panel-head",
                            h3 { id: "kit-team-plan-title", "Plan" }
                            span { class: "kit-team__count", "{plan_steps.len()} steps" }
                            button {
                                r#type: "button",
                                class: "kit-team__pause",
                                "aria-pressed": if is_paused { "true" } else { "false" },
                                onclick: move |_| {
                                    if let Some(handler) = on_toggle_pause { handler.call(!is_paused); }
                                },
                                if is_paused { "Resume" } else { "Pause" }
                            }
                        }
                        p { class: "kit-sr", role: "status", "{step_live}" }
                        ol { class: "kit-team__steps",
                            for (i, step) in plan_steps.iter().enumerate() {
                                li { key: "{step.id}", class: "kit-team__step", "data-status": step.status.as_str(),
                                    "data-dimmed": step_dimmed(step, focus.as_deref()).then_some("true"),
                                    "aria-current": (active_step == Some(i)).then_some("step"),
                                    span { class: "kit-team__mark", "aria-hidden": "true", "{step.status.mark()}" }
                                    span { class: "kit-team__step-title",
                                        "{step.title}"
                                        if let Some(owner) = step.owner_id.clone() {
                                            span { class: "kit-team__owner", " · {name_of(&agents, &owner)}" }
                                        }
                                    }
                                    span { class: "kit-team__status-word", "{step.status.as_str()}" }
                                    if let Some(reason) = step.blocked_reason.clone() {
                                        span { class: "kit-team__reason", "{reason}" }
                                    }
                                }
                            }
                        }
                    }
                    section { class: "kit-team__context", "aria-labelledby": "kit-team-context-title",
                        h3 { id: "kit-team-context-title", class: "kit-team__panel-title", "Context" }
                        p { class: "kit-team__usage",
                            span { class: "kit-team__usage-figure", "{usage:.0}% used" }
                            span { class: "kit-team__usage-word", "{warn_word}" }
                        }
                        ContextFillGrid {
                            part_list: context_parts,
                            window_size: context_usage.limit_tokens,
                            panel_title: "Shared context",
                        }
                    }
                }
            }
            form { class: "kit-team__composer",
                onsubmit: move |e: FormEvent| e.prevent_default(),
                label { class: "kit-team__sr", r#for: "kit-team-compose", "Message the team" }
                if !suggestions.is_empty() {
                    ul { class: "kit-team__mentions", role: "listbox", "aria-label": "Agents",
                        for (i, agent) in suggestions.iter().enumerate() {
                            li { key: "{agent.id}", role: "option", "aria-selected": if i == mention_index() { "true" } else { "false" },
                                onclick: {
                                    let name = agent.name.clone();
                                    move |_| {
                                        let current = draft();
                                        let at = current.rfind('@').unwrap_or(current.len());
                                        draft.set(format!("{}@{} ", &current[..at], name));
                                        mention_closed.set(true);
                                    }
                                },
                                "{agent.name}"
                                span { class: "kit-team__role", " {agent.role}" }
                            }
                        }
                    }
                }
                if !reply_name.is_empty() {
                    p { class: "kit-team__replying",
                        "Replying to {reply_name} "
                        button { r#type: "button", class: "kit-team__action", onclick: move |_| reply_to.set(None), "Cancel reply" }
                    }
                }
                textarea {
                    id: "kit-team-compose",
                    class: "kit-team__input",
                    rows: "2",
                    placeholder: "Message the team. Type @ to pick an agent.",
                    value: "{draft}",
                    "aria-autocomplete": "list",
                    oninput: move |e: FormEvent| {
                        draft.set(e.value());
                        mention_closed.set(false);
                        mention_index.set(0);
                    },
                    onkeydown: {
                        let agents_for_mention = agents.clone();
                        move |e: KeyboardEvent| {
                            let open = !suggestions.is_empty();
                            if open {
                                match e.key() {
                                    Key::ArrowDown => { e.prevent_default(); mention_index.set((mention_index() + 1) % suggestions.len()); }
                                    Key::ArrowUp => { e.prevent_default(); mention_index.set((mention_index() + suggestions.len() - 1) % suggestions.len()); }
                                    Key::Enter if !e.modifiers().shift() => {
                                        e.prevent_default();
                                        let pick = suggestions[mention_index().min(suggestions.len() - 1)].name.clone();
                                        let current = draft();
                                        let at = current.rfind('@').unwrap_or(current.len());
                                        draft.set(format!("{}@{} ", &current[..at], pick));
                                        mention_closed.set(true);
                                    }
                                    Key::Escape => { e.prevent_default(); mention_closed.set(true); }
                                    _ => {}
                                }
                                return;
                            }
                            if e.key() == Key::Enter && !e.modifiers().shift() {
                                e.prevent_default();
                                let text = draft().trim().to_string();
                                if text.is_empty() { return; }
                                let to_ids = resolve_mentions(&text, &agents_for_mention);
                                draft.set(String::new());
                                if let Some(handler) = on_message_submit {
                                    handler.call(ComposeSubmit { text, to_ids, reply_to_id: reply_to() });
                                }
                                reply_to.set(None);
                            }
                        }
                    },
                }
            }
        }
    }
}

/// Escape clears the focused agent only when no `@` list is open and the composer is empty
/// of a mention in progress.
fn focus_clear_allowed(mention_closed: bool, draft: &str) -> bool {
    mention_closed || mention_query(draft).is_none()
}

/// Agents named by `@name` in the text, in the order they first appear.
pub fn resolve_mentions(text: &str, agents: &[TeamAgent]) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    for agent in agents {
        if text.contains(&format!("@{}", agent.name)) && !ids.contains(&agent.id) {
            ids.push(agent.id.clone());
        }
    }
    ids
}

fn focus_tab(index: usize) {
    document::eval(&format!(
        "document.getElementById('kit-team-tab-{index}')?.focus();"
    ));
}

fn focus_message(id: &str) {
    document::eval(&format!(
        "document.getElementById({:?})?.focus();",
        format!("kit-msg-{id}")
    ));
}
