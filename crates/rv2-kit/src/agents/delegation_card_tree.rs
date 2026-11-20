//! Delegation card tree: a lead agent and the sub-agents it hands work to, as
//! one tree of cards. Each card shows what its agent is doing, how full its
//! context is, which tools it holds, and what it last received.
//!
//! Design choice: the clock is `clock_ms` when the host supplies one, else 0.
//! A start time after the clock reads as zero elapsed. `SystemTime` is not
//! used because it panics on `wasm32-unknown-unknown`.
//! Design choice: marks are text, not icon elements, so the data types derive
//! `Debug` and `PartialEq`.
//! Design choice: the tree has a roving tab stop. Tab enters on the last card
//! focused and leaves the tree from there.
//! Design choice: a hand-off pulse is mounted only for transfers that arrive
//! after mount, so the CSS animation plays once per transfer id.
//! Design choice: narrow containers indent levels through a container query;
//! the markup does not change.
//! Not implemented: the highlight that travels along intermediate connectors
//! (the pulse sits on the receiving card only), and the connector draw-in when
//! a workspace is shown.
//! Browser checks: pulses and load bars animate; arrow, Home and End move focus
//! between cards; a click outside closes the details; the workspace listbox
//! takes Up, Down, Enter and Escape; reduced motion removes all of it.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, focus_after_frame};

const CSS: Asset = asset!("/assets/delegation-card-tree.css");

/// Tools shown as tiles before the rest collapse into `+N`.
const TOOL_TILE_LIMIT: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentState {
    Active,
    Idle,
    Paused,
    Waiting,
    Done,
    Failed,
}

impl AgentState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Idle => "idle",
            Self::Paused => "paused",
            Self::Waiting => "waiting",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }

    fn word(self) -> &'static str {
        match self {
            Self::Active => "Working",
            Self::Idle => "Idle",
            Self::Paused => "Paused",
            Self::Waiting => "Needs you",
            Self::Done => "Done",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandAction {
    Message,
    Pause,
    Stop,
    Resume,
    Approve,
    Decline,
    Retry,
}

impl CommandAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Pause => "pause",
            Self::Stop => "stop",
            Self::Resume => "resume",
            Self::Approve => "approve",
            Self::Decline => "decline",
            Self::Retry => "retry",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Message => "Message",
            Self::Pause => "Pause",
            Self::Stop => "Stop",
            Self::Resume => "Resume",
            Self::Approve => "Approve",
            Self::Decline => "Decline",
            Self::Retry => "Retry",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadTone {
    Normal,
    Warn,
    Critical,
}

impl LoadTone {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Warn => "warn",
            Self::Critical => "critical",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolName {
    pub mark: Option<String>,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LastTransfer {
    pub from_name: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub state: AgentState,
    pub doing_text: Option<String>,
    /// Fraction of the context in use. Used when present, else the token
    /// figures.
    pub load_ratio: Option<f64>,
    pub load_capacity_tokens: Option<u64>,
    pub load_used_tokens: Option<u64>,
    pub model_label: Option<String>,
    pub model_mark: Option<String>,
    pub tool_names: Vec<ToolName>,
    pub last_transfer: Option<LastTransfer>,
    pub assignment: Option<String>,
    pub started_at: Option<u64>,
    pub ended_at: Option<u64>,
    pub spend: Option<f64>,
    pub tokens: Option<u64>,
    pub identity_mark: Option<String>,
    pub sub_agents: Vec<Agent>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transfer {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub active_count: Option<u32>,
    pub attention_count: Option<u32>,
}

/// Load as a whole percent, clamped to 0..=100. Token counts stand in when no
/// ratio is given. NaN reads as 0.
pub fn load_percent(agent: &Agent) -> u32 {
    let ratio = match (
        agent.load_ratio,
        agent.load_used_tokens,
        agent.load_capacity_tokens,
    ) {
        (Some(ratio), _, _) => ratio,
        (None, Some(used), Some(capacity)) if capacity > 0 => used as f64 / capacity as f64,
        _ => 0.0,
    };
    if ratio.is_nan() {
        0
    } else {
        (ratio.clamp(0.0, 1.0) * 100.0).round() as u32
    }
}

/// Warn above 80%, critical above 95%.
pub fn load_tone(percent: u32) -> LoadTone {
    if percent > 95 {
        LoadTone::Critical
    } else if percent > 80 {
        LoadTone::Warn
    } else {
        LoadTone::Normal
    }
}

/// Token counts in compact form: `999`, `44k`, `1.2M`.
pub fn compact_tokens(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{}k", count / 1_000)
    } else {
        count.to_string()
    }
}

/// `used / capacity` for the load bar, from whichever figures are present.
pub fn token_text(used: Option<u64>, capacity: Option<u64>) -> Option<String> {
    match (used, capacity) {
        (Some(used), Some(capacity)) => Some(format!(
            "{} / {}",
            compact_tokens(used),
            compact_tokens(capacity)
        )),
        (Some(used), None) => Some(compact_tokens(used)),
        _ => None,
    }
}

/// Dollars to two places; a positive amount under one cent reads as
/// "less than one cent".
pub fn format_spend(amount: f64) -> String {
    if !amount.is_finite() || amount <= 0.0 {
        "$0.00".to_string()
    } else if amount < 0.01 {
        "less than one cent".to_string()
    } else {
        format!("${amount:.2}")
    }
}

/// Whole seconds from `started` to `ended`, or to `now` while running. A
/// start after the end (or after `now`) reads as zero.
pub fn elapsed_seconds(started: Option<u64>, ended: Option<u64>, now: u64) -> u64 {
    let Some(start) = started else { return 0 };
    ended.unwrap_or(now).saturating_sub(start) / 1000
}

/// `m:ss`, with hours in front past one hour.
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

/// The one-line activity: what the agent is doing, or why it is resting.
pub fn activity_line(agent: &Agent) -> String {
    match agent.doing_text.as_deref().filter(|t| !t.is_empty()) {
        Some(text) => text.to_string(),
        None => match agent.state {
            AgentState::Active => "Working".to_string(),
            AgentState::Idle => "Waiting for work".to_string(),
            AgentState::Paused => "Paused".to_string(),
            AgentState::Waiting => "Waiting for you".to_string(),
            AgentState::Done => "Finished".to_string(),
            AgentState::Failed => "Stopped with an error".to_string(),
        },
    }
}

/// Command actions valid for a state. Approve and Decline only when waiting;
/// Retry only when failed.
pub fn commands_for(state: AgentState) -> Vec<CommandAction> {
    match state {
        AgentState::Active => vec![
            CommandAction::Message,
            CommandAction::Pause,
            CommandAction::Stop,
        ],
        AgentState::Idle => vec![CommandAction::Message, CommandAction::Stop],
        AgentState::Paused => vec![
            CommandAction::Message,
            CommandAction::Resume,
            CommandAction::Stop,
        ],
        AgentState::Waiting => {
            vec![
                CommandAction::Approve,
                CommandAction::Decline,
                CommandAction::Message,
            ]
        }
        AgentState::Done => vec![CommandAction::Message],
        AgentState::Failed => vec![CommandAction::Retry, CommandAction::Message],
    }
}

/// Tiles shown and the count collapsed into `+N`.
pub fn tool_tiles(count: usize) -> (usize, usize) {
    if count <= TOOL_TILE_LIMIT {
        (count, 0)
    } else {
        (TOOL_TILE_LIMIT - 1, count - (TOOL_TILE_LIMIT - 1))
    }
}

/// Agent ids in tree order: the lead, then each branch depth-first.
pub fn flatten_ids(agent: &Agent) -> Vec<String> {
    let mut out = vec![agent.id.clone()];
    for child in &agent.sub_agents {
        out.extend(flatten_ids(child));
    }
    out
}

/// Agents working and agents needing the person, across the whole tree.
pub fn tally(agent: &Agent) -> (usize, usize) {
    let own = match agent.state {
        AgentState::Active => (1, 0),
        AgentState::Waiting => (0, 1),
        _ => (0, 0),
    };
    agent
        .sub_agents
        .iter()
        .map(tally)
        .fold(own, |(w, a), (cw, ca)| (w + cw, a + ca))
}

pub fn agent_name(agent: &Agent, id: &str) -> Option<String> {
    if agent.id == id {
        return Some(agent.name.clone());
    }
    agent
        .sub_agents
        .iter()
        .find_map(|child| agent_name(child, id))
}

/// Transfers whose ids were not in `known`, in their order.
pub fn new_transfers<'a>(known: &[String], transfers: &'a [Transfer]) -> Vec<&'a Transfer> {
    transfers
        .iter()
        .filter(|t| !known.contains(&t.id))
        .collect()
}

/// The polite line for a hand-off, for example "Lead handed to Coder: Guard empty carts".
pub fn handoff_line(from: &str, to: &str, note: Option<&str>) -> String {
    match note.filter(|n| !n.is_empty()) {
        Some(note) => format!("{from} handed to {to}: {note}"),
        None => format!("{from} handed to {to}."),
    }
}

/// The polite line for a command, for example "Coder paused."
pub fn command_line(name: &str, action: CommandAction) -> String {
    match action {
        CommandAction::Message => format!("Message sent to {name}."),
        CommandAction::Pause => format!("{name} paused."),
        CommandAction::Stop => format!("{name} stopped."),
        CommandAction::Resume => format!("{name} resumed."),
        CommandAction::Approve => format!("{name} approved."),
        CommandAction::Decline => format!("{name} declined."),
        CommandAction::Retry => format!("{name} retrying."),
    }
}

/// The card's single accessible label: name, state, activity and load.
pub fn card_label(agent: &Agent) -> String {
    format!(
        "{}, {}, {}, load {}%, opens a dialog",
        agent.name,
        agent.state.word(),
        activity_line(agent),
        load_percent(agent)
    )
}

fn card_dom_id(agent_id: &str) -> String {
    format!("kit-delg-card-{agent_id}")
}

struct Ctx<'a> {
    open: Signal<Option<String>>,
    messaging: Signal<bool>,
    draft: Signal<String>,
    roving: Signal<Option<String>>,
    roving_id: Option<String>,
    fresh: &'a [&'a Transfer],
    notice: Signal<String>,
    on_open: Option<EventHandler<Agent>>,
    on_command: Option<EventHandler<(Agent, CommandAction, Option<String>)>>,
    now: u64,
}

fn details_panel(ctx: &Ctx<'_>, agent: &Agent) -> Element {
    let mut open = ctx.open;
    let mut messaging = ctx.messaging;
    let mut draft = ctx.draft;
    let mut notice = ctx.notice;
    let on_command = ctx.on_command;
    let name = agent.name.clone();
    let dialog_label = format!("{} details", agent.name);
    let message_label = format!("Message to {}", agent.name);
    let card_id = card_dom_id(&agent.id);
    let assignment = agent
        .assignment
        .clone()
        .unwrap_or_else(|| "No assignment".to_string());
    let model = agent
        .model_label
        .clone()
        .unwrap_or_else(|| "Model not set".to_string());
    let elapsed = clock_text(elapsed_seconds(agent.started_at, agent.ended_at, ctx.now));
    let spend_text = agent.spend.map(format_spend);
    let tokens_text = agent.tokens.map(compact_tokens);
    let tools: Vec<String> = agent.tool_names.iter().map(|t| t.name.clone()).collect();
    let last_line = agent
        .last_transfer
        .as_ref()
        .map(|t| format!("Last received from {}: {}", t.from_name, t.text));
    let actions = if on_command.is_some() {
        commands_for(agent.state)
    } else {
        Vec::new()
    };
    let action_nodes: Vec<Element> = actions
        .into_iter()
        .map(|action| {
            let target = agent.clone();
            let label = action.label();
            let line = command_line(&name, action);
            rsx! {
                button {
                    r#type: "button",
                    class: "kit-delg__action",
                    "data-action": label,
                    onclick: move |_| {
                        if action == CommandAction::Message {
                            messaging.set(true);
                            return;
                        }
                        if let Some(handler) = on_command {
                            handler.call((target.clone(), action, None));
                        }
                        notice.set(line.clone());
                    },
                    {label}
                }
            }
        })
        .collect();
    let show_message = messaging();
    let send_target = agent.clone();
    let send_line = command_line(&agent.name, CommandAction::Message);
    rsx! {
        div {
            class: "kit-delg__details",
            role: "dialog",
            "aria-label": "{dialog_label}",
            tabindex: "-1",
            onmounted: move |event: MountedEvent| {
                let node = event.data();
                spawn(async move {
                    let _ = node.set_focus(true).await;
                });
            },
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape {
                    event.prevent_default();
                    if messaging() {
                        messaging.set(false);
                    } else {
                        open.set(None);
                        focus_after_frame(&card_id);
                    }
                }
            },
            dl { class: "kit-delg__facts",
                div { dt { "Assignment" } dd { {assignment} } }
                div { dt { "Model" } dd { {model} } }
                div { dt { "Elapsed" } dd { {elapsed} } }
                if let Some(text) = spend_text {
                    div { dt { "Spend" } dd { {text} } }
                }
                if let Some(text) = tokens_text {
                    div { dt { "Tokens" } dd { {text} } }
                }
            }
            if !tools.is_empty() {
                ul { class: "kit-delg__tools-all", "aria-label": "Tools",
                    for tool in tools {
                        li { {tool} }
                    }
                }
            }
            if let Some(line) = last_line {
                p { class: "kit-delg__last", {line} }
            }
            if !action_nodes.is_empty() {
                div { class: "kit-delg__actions", role: "group", "aria-label": "Actions",
                    for node in action_nodes {
                        {node}
                    }
                }
            }
            if show_message {
                input {
                    r#type: "text",
                    class: "kit-delg__message",
                    "aria-label": "{message_label}",
                    placeholder: "Message",
                    value: "{draft}",
                    oninput: move |event: FormEvent| draft.set(event.value()),
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Enter {
                            let text = draft().trim().to_string();
                            if !text.is_empty() {
                                if let Some(handler) = on_command {
                                    handler.call((send_target.clone(), CommandAction::Message, Some(text)));
                                }
                                notice.set(send_line.clone());
                                draft.set(String::new());
                                messaging.set(false);
                            }
                        }
                    },
                }
            }
        }
    }
}

fn tree_node(ctx: &Ctx<'_>, agent: &Agent) -> Element {
    let card_id = card_dom_id(&agent.id);
    let percent = load_percent(agent);
    let tone = load_tone(percent).as_str();
    let activity = activity_line(agent);
    let label = card_label(agent);
    let word = agent.state.word();
    let state = agent.state.as_str();
    let is_open = (ctx.open)().as_deref() == Some(agent.id.as_str());
    let tab_stop = if ctx.roving_id.as_deref() == Some(agent.id.as_str()) {
        "0"
    } else {
        "-1"
    };
    let tokens = token_text(agent.load_used_tokens, agent.load_capacity_tokens);
    let (shown, extra) = tool_tiles(agent.tool_names.len());
    let tiles: Vec<String> = agent
        .tool_names
        .iter()
        .take(shown)
        .map(|t| t.name.clone())
        .collect();
    let more = format!("+{extra}");
    let received = agent
        .last_transfer
        .as_ref()
        .map(|t| format!("{} · {}", t.from_name, t.text));
    let has_received = received.is_some();
    let received_text = received.unwrap_or_default();
    let mark = agent.identity_mark.clone().unwrap_or_else(|| {
        agent
            .name
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default()
    });
    let model_label = agent.model_label.clone().unwrap_or_default();
    let spend_text = agent.spend.map(format_spend).unwrap_or_default();
    let pulses: Vec<Element> = ctx
        .fresh
        .iter()
        .filter(|t| t.target_id == agent.id)
        .map(|_| rsx! { span { class: "kit-delg__pulse", "aria-hidden": "true" } })
        .collect();
    let details = if is_open {
        details_panel(ctx, agent)
    } else {
        rsx! {}
    };
    let children: Vec<Element> = agent
        .sub_agents
        .iter()
        .map(|child| tree_node(ctx, child))
        .collect();

    let mut open = ctx.open;
    let mut messaging = ctx.messaging;
    let mut roving = ctx.roving;
    let on_open = ctx.on_open;
    let click_target = agent.clone();
    let click_id = agent.id.clone();
    let focus_id = agent.id.clone();
    rsx! {
        li { class: "kit-delg__node", role: "none",
            div { class: "kit-delg__card", "data-state": state, "data-load-tone": tone,
                "data-received": has_received.to_string(),
                button {
                    id: "{card_id}",
                    r#type: "button",
                    class: "kit-delg__toggle",
                    tabindex: tab_stop,
                    "aria-haspopup": "dialog",
                    "aria-expanded": is_open.to_string(),
                    "aria-label": "{label}",
                    onfocus: move |_| roving.set(Some(focus_id.clone())),
                    onclick: move |_| {
                        let next = if (open)().as_deref() == Some(click_id.as_str()) {
                            None
                        } else {
                            Some(click_id.clone())
                        };
                        open.set(next);
                        messaging.set(false);
                        if let Some(handler) = on_open {
                            handler.call(click_target.clone());
                        }
                    },
                    span { class: "kit-delg__mark", "aria-hidden": "true", {mark} }
                    span { class: "kit-delg__heading",
                        span { class: "kit-delg__name", {agent.name.clone()} }
                        span { class: "kit-delg__state", "data-state": state, {word} }
                        if !model_label.is_empty() {
                            span { class: "kit-delg__model", {model_label} }
                        }
                    }
                    span { class: "kit-delg__activity", {activity} }
                    span { class: "kit-delg__load", role: "meter", "aria-valuemin": "0",
                        "aria-valuemax": "100", "aria-valuenow": percent.to_string(),
                        span { class: "kit-delg__fill", style: "width: {percent}%" }
                    }
                    span { class: "kit-delg__figures",
                        span { class: "kit-delg__percent", {format!("{percent}%")} }
                        if let Some(text) = tokens {
                            span { class: "kit-delg__tokens", {text} }
                        }
                    }
                    if !tiles.is_empty() {
                        span { class: "kit-delg__tiles",
                            for tile in tiles {
                                span { class: "kit-delg__tile", {tile} }
                            }
                            if extra > 0 {
                                span { class: "kit-delg__tile kit-delg__tile--more", {more} }
                            }
                        }
                    }
                    if has_received {
                        span { class: "kit-delg__received", {received_text} }
                    }
                    if !spend_text.is_empty() {
                        span { class: "kit-delg__spend", {spend_text} }
                    }
                }
                for pulse in pulses {
                    {pulse}
                }
            }
            {details}
            if !children.is_empty() {
                ul { class: "kit-delg__children", role: "group",
                    for child in children {
                        {child}
                    }
                }
            }
        }
    }
}

/// A lead agent and its sub-agents as one tree of cards. Hand-offs and
/// commands are announced politely; the command set follows each agent's state.
#[component]
pub fn DelegationCardTree(
    top_agent: Agent,
    #[props(default)] transfers: Vec<Transfer>,
    #[props(default = "Agents".to_string(), into)] heading: String,
    #[props(default)] workspaces: Vec<Workspace>,
    #[props(default)] active_workspace: Option<String>,
    #[props(default)] on_workspace_change: Option<EventHandler<String>>,
    #[props(default)] on_open: Option<EventHandler<Agent>>,
    #[props(default)] on_command: Option<EventHandler<(Agent, CommandAction, Option<String>)>>,
    #[props(default)] clock_ms: Option<u64>,
    #[props(default)] extra_class: String,
) -> Element {
    let open = use_signal(|| None::<String>);
    let messaging = use_signal(|| false);
    let draft = use_signal(String::new);
    let mut roving = use_signal(|| None::<String>);
    let mut ws_open = use_signal(|| false);
    let mut ws_focus = use_signal(|| 0usize);
    let mut local_ws = use_signal(|| None::<String>);
    let notice = use_signal(String::new);
    let known = use_hook(|| {
        transfers
            .iter()
            .map(|t| t.id.clone())
            .collect::<Vec<String>>()
    });

    let order = flatten_ids(&top_agent);
    let roving_id = roving().or_else(|| order.first().cloned());
    let (working, attention) = tally(&top_agent);
    let fresh = new_transfers(&known, &transfers);
    let handoff = fresh
        .last()
        .map(|t| {
            let from = agent_name(&top_agent, &t.source_id).unwrap_or_else(|| t.source_id.clone());
            let to = agent_name(&top_agent, &t.target_id).unwrap_or_else(|| t.target_id.clone());
            handoff_line(&from, &to, t.note.as_deref())
        })
        .unwrap_or_default();
    let current_notice = notice();
    let live = if current_notice.is_empty() {
        handoff
    } else {
        current_notice
    };

    let ctx = Ctx {
        open,
        messaging,
        draft,
        roving,
        roving_id,
        fresh: &fresh,
        notice,
        on_open,
        on_command,
        now: clock_ms.unwrap_or(0),
    };
    let tree = tree_node(&ctx, &top_agent);

    let active = local_ws()
        .or(active_workspace)
        .or_else(|| workspaces.first().map(|w| w.id.clone()));
    let active_name = workspaces
        .iter()
        .find(|w| Some(&w.id) == active.as_ref())
        .map(|w| w.name.clone())
        .unwrap_or_else(|| heading.clone());
    let ws_ids: Vec<String> = workspaces.iter().map(|w| w.id.clone()).collect();
    let ws_rows: Vec<(String, String, bool)> = workspaces
        .iter()
        .map(|w| {
            let label = format!(
                "{} · {} working, {} need you",
                w.name,
                w.active_count.unwrap_or(0),
                w.attention_count.unwrap_or(0)
            );
            (w.id.clone(), label, Some(&w.id) == active.as_ref())
        })
        .collect();
    let class = class_list("kit-delg", &extra_class);
    let counts_line = format!("{working} working · {attention} need you");

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class, "aria-label": "{heading}",
            header { class: "kit-delg__bar",
                if workspaces.is_empty() {
                    p { class: "kit-delg__crumb", "Agents / {heading}" }
                } else {
                    div { class: "kit-delg__switcher",
                        button {
                            r#type: "button",
                            class: "kit-delg__switch",
                            "aria-haspopup": "listbox",
                            "aria-expanded": ws_open().to_string(),
                            "aria-controls": "kit-delg-ws-list",
                            onclick: move |_| ws_open.set(!ws_open()),
                            {active_name}
                        }
                        if ws_open() {
                            ul {
                                id: "kit-delg-ws-list",
                                class: "kit-delg__listbox",
                                role: "listbox",
                                "aria-label": "Workspaces",
                                tabindex: "-1",
                                onkeydown: move |event: KeyboardEvent| {
                                    let last = ws_ids.len().saturating_sub(1);
                                    match event.key() {
                                        Key::ArrowDown => {
                                            event.prevent_default();
                                            ws_focus.set((ws_focus() + 1).min(last));
                                        }
                                        Key::ArrowUp => {
                                            event.prevent_default();
                                            ws_focus.set(ws_focus().saturating_sub(1));
                                        }
                                        Key::Enter => {
                                            event.prevent_default();
                                            if let Some(id) = ws_ids.get(ws_focus()).cloned() {
                                                local_ws.set(Some(id.clone()));
                                                if let Some(handler) = on_workspace_change {
                                                    handler.call(id);
                                                }
                                            }
                                            ws_open.set(false);
                                        }
                                        Key::Escape => {
                                            event.prevent_default();
                                            ws_open.set(false);
                                        }
                                        _ => {}
                                    }
                                },
                                for (id, label, selected) in ws_rows {
                                    li {
                                        key: "{id}",
                                        role: "option",
                                        "aria-selected": selected.to_string(),
                                        onclick: {
                                            let id = id.clone();
                                            move |_| {
                                                local_ws.set(Some(id.clone()));
                                                if let Some(handler) = on_workspace_change {
                                                    handler.call(id.clone());
                                                }
                                                ws_open.set(false);
                                            }
                                        },
                                        {label}
                                    }
                                }
                            }
                        }
                    }
                }
                p { class: "kit-delg__counts", {counts_line} }
            }
            div { class: "kit-delg__sr", role: "status", "aria-live": "polite", {live} }
            div {
                class: "kit-delg__tree",
                onkeydown: move |event: KeyboardEvent| {
                    let Some(current) = roving() else { return };
                    let last = order.len().saturating_sub(1);
                    let index = order.iter().position(|id| *id == current).unwrap_or(0);
                    let target = match event.key() {
                        Key::ArrowDown | Key::ArrowRight => (index + 1).min(last),
                        Key::ArrowUp | Key::ArrowLeft => index.saturating_sub(1),
                        Key::Home => 0,
                        Key::End => last,
                        _ => return,
                    };
                    event.prevent_default();
                    if let Some(id) = order.get(target) {
                        roving.set(Some(id.clone()));
                        focus_after_frame(&card_dom_id(id));
                    }
                },
                ul { class: "kit-delg__root", role: "group",
                    {tree}
                }
            }
        }
    }
}
