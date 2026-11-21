//! Agent tool reach map: one agent at the centre of every tool it can reach, with each tool's calls one click away.
//!
//! Design choice: the replay input is checked by a small JSON validator (no serde dependency). An object
//! input is replayed as `ReplayInput::Json` only when the edit is one complete JSON object.
//! Design choice: `format_time` defaults to seconds from the run start, because no wall clock is read.
//! Browser checks: connector spark, agent-tab arrow focus, calls-view focus hand-off, narrow-width reflow.

use dioxus::prelude::*;

use crate::components::agent_context_map::{arc_positions, focus_on_mount};
use crate::internal::*;

const CSS: Asset = asset!("/assets/agent-tool-reach-map.css");

/// Design-unit canvas the map is drawn in; tool positions are percentages of it.
pub const MAP_W: f64 = 1000.0;
pub const MAP_H: f64 = 600.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentState {
    Queued,
    Running,
    Waiting,
    Done,
    Failed,
}

impl AgentState {
    pub fn word(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Working",
            Self::Waiting => "Needs you",
            Self::Done => "Done",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallState {
    Running,
    Done,
    Failed,
}

impl CallState {
    pub fn word(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Done => "Done",
            Self::Failed => "Failed",
        }
    }
}

/// What an agent's access to one tool looks like on the map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    NoAccess,
    NotUsed,
    Running,
    Failed,
    Used,
}

impl Reach {
    pub fn word(self) -> &'static str {
        match self {
            Self::NoAccess => "No access",
            Self::NotUsed => "Not used yet",
            Self::Running => "Running",
            Self::Failed => "Failed",
            Self::Used => "Used",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ToolCall {
    pub call_id: String,
    pub call_state: CallState,
    pub call_input: String,
    pub call_output: Option<String>,
    pub call_error: Option<String>,
    pub called_at: f64,
    pub duration: Option<f64>,
    pub is_replay: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Grant {
    pub tool_id: String,
    pub call_list: Vec<ToolCall>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct MapAgent {
    pub agent_id: String,
    pub agent_name: String,
    pub model_name: Option<String>,
    pub agent_mark: Option<String>,
    pub agent_state: AgentState,
    pub agent_task: Option<String>,
    pub granted_tools: Vec<Grant>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct AccessTool {
    pub tool_id: String,
    pub tool_name: String,
    pub tool_mark: Option<String>,
    pub tool_summary: Option<String>,
}

/// The replay input: an object edited as valid JSON, or free text for anything else.
#[derive(Clone, PartialEq, Debug)]
pub enum ReplayInput {
    Text(String),
    Json(String),
}

/// The grant an agent holds for a tool, if any.
pub fn grant_of<'a>(agent: &'a MapAgent, tool_id: &str) -> Option<&'a Grant> {
    agent.granted_tools.iter().find(|g| g.tool_id == tool_id)
}

/// Reach of one tool. A running call outranks a failed last call, which outranks plain use.
pub fn tool_reach(grant: Option<&Grant>) -> Reach {
    let Some(grant) = grant else {
        return Reach::NoAccess;
    };
    if grant.call_list.is_empty() {
        return Reach::NotUsed;
    }
    if grant
        .call_list
        .iter()
        .any(|c| c.call_state == CallState::Running)
    {
        return Reach::Running;
    }
    if sorted_calls(&grant.call_list)
        .last()
        .is_some_and(|c| c.call_state == CallState::Failed)
    {
        return Reach::Failed;
    }
    Reach::Used
}

/// Calls oldest first, so a new call always lands at the end of its tool's list.
pub fn sorted_calls(calls: &[ToolCall]) -> Vec<ToolCall> {
    let mut sorted = calls.to_vec();
    sorted.sort_by(|a, b| a.called_at.total_cmp(&b.called_at));
    sorted
}

/// Heading counts for one agent: (running tools, used tools, catalog size, calls).
pub fn map_counts(agent: &MapAgent, catalog: &[AccessTool]) -> (usize, usize, usize, usize) {
    let reaches: Vec<Reach> = catalog
        .iter()
        .map(|t| tool_reach(grant_of(agent, &t.tool_id)))
        .collect();
    let running = reaches.iter().filter(|r| **r == Reach::Running).count();
    let used = reaches
        .iter()
        .filter(|r| matches!(r, Reach::Used | Reach::Running | Reach::Failed))
        .count();
    let calls = agent.granted_tools.iter().map(|g| g.call_list.len()).sum();
    (running, used, catalog.len(), calls)
}

/// Replay input for an edited call. An object original must stay an object.
pub fn replay_input(original: &str, edited: &str) -> Result<ReplayInput, String> {
    if original.trim_start().starts_with('{') {
        if is_json_object(edited) {
            Ok(ReplayInput::Json(edited.trim().to_string()))
        } else {
            Err("Input must be a complete JSON object".to_string())
        }
    } else {
        Ok(ReplayInput::Text(edited.to_string()))
    }
}

/// True when `text` is one complete JSON object, surrounding whitespace allowed.
pub fn is_json_object(text: &str) -> bool {
    let bytes = text.trim().as_bytes();
    if bytes.first() != Some(&b'{') {
        return false;
    }
    json_value(bytes, 0).is_some_and(|end| skip_ws(bytes, end) == bytes.len())
}

fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

/// Index just past one JSON value starting at `i`, or `None` when it does not parse.
fn json_value(b: &[u8], i: usize) -> Option<usize> {
    let i = skip_ws(b, i);
    match *b.get(i)? {
        b'{' => json_container(b, i + 1, b'}', true),
        b'[' => json_container(b, i + 1, b']', false),
        b'"' => json_string(b, i),
        b't' => literal(b, i, b"true"),
        b'f' => literal(b, i, b"false"),
        b'n' => literal(b, i, b"null"),
        b'-' | b'0'..=b'9' => {
            let mut j = i;
            while j < b.len() && matches!(b[j], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
                j += 1;
            }
            std::str::from_utf8(&b[i..j]).ok()?.parse::<f64>().ok()?;
            Some(j)
        }
        _ => None,
    }
}

fn json_container(b: &[u8], mut i: usize, close: u8, keyed: bool) -> Option<usize> {
    i = skip_ws(b, i);
    if b.get(i) == Some(&close) {
        return Some(i + 1);
    }
    loop {
        if keyed {
            i = json_string(b, skip_ws(b, i))?;
            i = skip_ws(b, i);
            if b.get(i) != Some(&b':') {
                return None;
            }
            i += 1;
        }
        i = json_value(b, i)?;
        i = skip_ws(b, i);
        match b.get(i)? {
            b',' => i += 1,
            c if *c == close => return Some(i + 1),
            _ => return None,
        }
    }
}

fn json_string(b: &[u8], i: usize) -> Option<usize> {
    if b.get(i) != Some(&b'"') {
        return None;
    }
    let mut j = i + 1;
    while j < b.len() {
        match b[j] {
            b'\\' => j += 2,
            b'"' => return Some(j + 1),
            _ => j += 1,
        }
    }
    None
}

fn literal(b: &[u8], i: usize, lit: &[u8]) -> Option<usize> {
    (b.get(i..i + lit.len())? == lit).then_some(i + lit.len())
}

/// DOM id of the tool card at `index`; the calls panel returns focus here on close.
pub fn tool_card_id(base: &str, index: usize) -> String {
    format!("{base}-tool-{index}")
}

/// Runs a replay from the edit text, falling back to the call's own input when the edit is empty.
fn submit_replay(
    call: &ToolCall,
    edit: &str,
    tool: Option<&AccessTool>,
    agent: &MapAgent,
    on_replay: Option<EventHandler<(ToolCall, ReplayInput, AccessTool, MapAgent)>>,
    mut error: Signal<Option<String>>,
) {
    let text = if edit.trim().is_empty() {
        call.call_input.as_str()
    } else {
        edit
    };
    match replay_input(&call.call_input, text) {
        Ok(input) => {
            error.set(None);
            if let (Some(h), Some(tool)) = (on_replay, tool) {
                h.call((call.clone(), input, tool.clone(), agent.clone()));
            }
        }
        Err(message) => error.set(Some(message)),
    }
}

#[derive(Clone, PartialEq, Debug)]
enum Open {
    Map,
    Calls(String),
}

/// Agent tabs over a map of one agent's tool access. Granted tools show a solid connector, the rest a dashed one.
/// Opening a granted tool lists its calls; a replay sends `on_replay` with the parsed input.
#[component]
pub fn AgentToolReachMap(
    agent_list: Vec<MapAgent>,
    tool_catalog: Vec<AccessTool>,
    #[props(default)] selected_agent_id: Option<String>,
    #[props(default)] default_agent_id: Option<String>,
    #[props(default)] on_agent_pick: Option<EventHandler<String>>,
    #[props(default)] on_replay: Option<
        EventHandler<(ToolCall, ReplayInput, AccessTool, MapAgent)>,
    >,
    #[props(default)] format_time: Option<Callback<f64, String>>,
    #[props(default = "Tool access".to_string(), into)] board_title: String,
    #[props(default)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-atr-{}", next_id()));
    let first_id = agent_list
        .first()
        .map(|a| a.agent_id.clone())
        .unwrap_or_default();
    let mut local_pick = use_signal(|| default_agent_id.clone().unwrap_or(first_id));
    let mut open = use_signal(|| Open::Map);
    let mut call_index = use_signal(|| 0usize);
    let mut edit = use_signal(String::new);
    let mut replay_error = use_signal(|| None::<String>);

    let current_id = selected_agent_id.clone().unwrap_or_else(|| local_pick());
    let Some(agent) = agent_list
        .iter()
        .find(|a| a.agent_id == current_id)
        .or(agent_list.first())
        .cloned()
    else {
        return rsx! {
            document::Link { rel: "stylesheet", href: KIT_CSS }
            document::Link { rel: "stylesheet", href: CSS }
            section { class: "kit-atr kit-atr--empty", "aria-label": "{board_title}", p { "No agents." } }
        };
    };
    let (running, used, total, calls_total) = map_counts(&agent, &tool_catalog);
    let left_n = tool_catalog.len().div_ceil(2);
    let left = arc_positions(left_n, true, MAP_W / 2.0, MAP_H / 2.0, 380.0, 230.0);
    let right = arc_positions(
        tool_catalog.len() - left_n,
        false,
        MAP_W / 2.0,
        MAP_H / 2.0,
        380.0,
        230.0,
    );
    let positions: Vec<(f64, f64)> = left.into_iter().chain(right).collect();
    let class = class_list("kit-atr", &extra_class);
    let blocked = matches!(open(), Open::Calls(_));
    let tab_ids: Vec<String> = agent_list.iter().map(|a| a.agent_id.clone()).collect();
    let tab_index = tab_ids
        .iter()
        .position(|id| *id == agent.agent_id)
        .unwrap_or(0);
    let counts_line =
        format!("{running} running · {used} of {total} tools used · {calls_total} calls");
    let fmt = move |t: f64| match format_time {
        Some(f) => f.call(t),
        None => format!("{:.1}s", t / 1000.0),
    };
    let pick = use_callback(move |id: String| {
        local_pick.set(id.clone());
        if let Some(h) = on_agent_pick {
            h.call(id);
        }
    });
    let agent_mark = agent
        .agent_mark
        .clone()
        .unwrap_or_else(|| agent.agent_name.chars().take(1).collect());
    let map_label = format!("Tools that {} can reach", agent.agent_name);

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", "aria-labelledby": "{base}-title",
            header { class: "kit-atr__head",
                h2 { id: "{base}-title", class: "kit-atr__title", "{board_title}" }
                p { class: "kit-atr__counts", role: "status", "aria-live": "polite", "{counts_line}" }
            }
            div { class: "kit-atr__tabs", role: "tablist", "aria-label": "Agents",
                for (i, a) in agent_list.iter().enumerate() {
                    {
                        let selected = a.agent_id == agent.agent_id;
                        let id = a.agent_id.clone();
                        let tab_dom = format!("{base}-tab-{i}");
                        let label = format!("{} · {}", a.agent_name, a.agent_state.word());
                        let ids = tab_ids.clone();
                        let base_keys = base.clone();
                        rsx! {
                            button {
                                key: "{a.agent_id}",
                                id: "{tab_dom}", r#type: "button", role: "tab", class: "kit-atr__tab",
                                "aria-selected": "{selected}", tabindex: if selected { "0" } else { "-1" },
                                onclick: move |_| pick.call(id.clone()),
                                onkeydown: move |event| {
                                    let len = ids.len();
                                    let next = match event.key() {
                                        Key::ArrowRight => Some((tab_index + 1) % len),
                                        Key::ArrowLeft => Some((tab_index + len - 1) % len),
                                        _ => None,
                                    };
                                    if let Some(n) = next {
                                        event.prevent_default();
                                        pick.call(ids[n].clone());
                                        document::eval(&format!("document.getElementById('{base_keys}-tab-{n}')?.focus();"));
                                    }
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }
            div {
                class: "kit-atr__map", role: "tabpanel", "aria-label": "{map_label}",
                "data-blocked": blocked.then_some("true"),
                inert: blocked.then_some("true"),
                svg { class: "kit-atr__wires", view_box: "0 0 {MAP_W} {MAP_H}", preserve_aspect_ratio: "none", "aria-hidden": "true",
                    for (i, t) in tool_catalog.iter().enumerate() {
                        {
                            let (x, y) = positions[i];
                            let granted = grant_of(&agent, &t.tool_id).is_some();
                            let running_call = tool_reach(grant_of(&agent, &t.tool_id)) == Reach::Running;
                            rsx! {
                                line {
                                    key: "{t.tool_id}", class: "kit-atr__wire",
                                    "data-granted": "{granted}", "data-running": running_call.then_some("true"),
                                    x1: "{MAP_W / 2.0}", y1: "{MAP_H / 2.0}", x2: "{x}", y2: "{y}",
                                }
                            }
                        }
                    }
                }
                ul { class: "kit-atr__tools", "aria-label": "Tools",
                    for (i, t) in tool_catalog.iter().enumerate() {
                        {
                            let (x, y) = positions[i];
                            let grant = grant_of(&agent, &t.tool_id);
                            let reach = tool_reach(grant);
                            let calls_here = grant.map(|g| g.call_list.len()).unwrap_or(0);
                            let last_time = grant
                                .and_then(|g| sorted_calls(&g.call_list).last().map(|c| c.called_at))
                                .map(fmt)
                                .unwrap_or_default();
                            let style = format!("--atr-x: {:.2}%; --atr-y: {:.2}%;", x / MAP_W * 100.0, y / MAP_H * 100.0);
                            let dom_id = format!("{base}-tool-{i}");
                            let granted = grant.is_some();
                            let label = if granted {
                                format!("{}, {}, {calls_here} calls", t.tool_name, reach.word())
                            } else {
                                format!("{}, No access", t.tool_name)
                            };
                            let open_id = t.tool_id.clone();
                            let mark = t.tool_mark.clone().unwrap_or_else(|| t.tool_name.chars().take(1).collect());
                            let summary = t.tool_summary.clone().unwrap_or_default();
                            let name = t.tool_name.clone();
                            rsx! {
                                li { key: "{t.tool_id}", class: "kit-atr__slot", style: "{style}", "data-reach": "{reach.word()}",
                                    button {
                                        id: "{dom_id}", r#type: "button", class: "kit-atr__card", "data-granted": "{granted}",
                                        "aria-label": "{label}", "aria-disabled": (!granted).then_some("true"),
                                        onclick: move |_| {
                                            if granted {
                                                call_index.set(0);
                                                edit.set(String::new());
                                                replay_error.set(None);
                                                open.set(Open::Calls(open_id.clone()));
                                            }
                                        },
                                        span { class: "kit-atr__mark", "aria-hidden": "true", "{mark}" }
                                        span { class: "kit-atr__name", "{name}" }
                                        span { class: "kit-atr__state", "{reach.word()}" }
                                        if granted && calls_here > 0 {
                                            span { class: "kit-atr__meta", "{calls_here} calls · last {last_time}" }
                                        }
                                        if !summary.is_empty() { span { class: "kit-atr__meta", "{summary}" } }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "kit-atr__agent",
                    span { class: "kit-atr__mark", "aria-hidden": "true", "{agent_mark}" }
                    span { class: "kit-atr__name", "{agent.agent_name}" }
                    if let Some(model) = agent.model_name.clone() { span { class: "kit-atr__meta", "{model}" } }
                    span { class: "kit-atr__state", "{agent.agent_state.word()}" }
                    if let Some(task) = agent.agent_task.clone() { span { class: "kit-atr__meta", "{task}" } }
                }
            }
            if let Open::Calls(tool_id) = open() {
                {
                    let tool = tool_catalog.iter().find(|t| t.tool_id == tool_id).cloned();
                    let tool_index = tool_catalog.iter().position(|t| t.tool_id == tool_id).unwrap_or(0);
                    let calls = grant_of(&agent, &tool_id).map(|g| sorted_calls(&g.call_list)).unwrap_or_default();
                    let idx = call_index().min(calls.len().saturating_sub(1));
                    let selected_call = calls.get(idx).cloned();
                    let panel_title = format!("{} calls", tool.as_ref().map(|t| t.tool_name.as_str()).unwrap_or(""));
                    let focus_back = tool_card_id(&base, tool_index);
                    let focus_on_close = focus_back.clone();
                    let agent_for_list = agent.clone();
                    let tool_for_list = tool.clone();
                    let call_count = calls.len();
                    rsx! {
                        section {
                            class: "kit-atr__calls", role: "dialog", "aria-label": "{panel_title}", tabindex: "-1",
                            onmounted: focus_on_mount,
                            onkeydown: move |event| {
                                match event.key() {
                                    Key::Escape => {
                                        event.prevent_default();
                                        open.set(Open::Map);
                                        focus_after_frame(&focus_back);
                                    }
                                    Key::ArrowDown => {
                                        event.prevent_default();
                                        call_index.set((idx + 1).min(call_count.saturating_sub(1)));
                                    }
                                    Key::ArrowUp => {
                                        event.prevent_default();
                                        call_index.set(idx.saturating_sub(1));
                                    }
                                    _ => {}
                                }
                            },
                            header { class: "kit-atr__calls-head",
                                h3 { class: "kit-atr__calls-title", "{panel_title}" }
                                button {
                                    r#type: "button", class: "kit-atr__close",
                                    onclick: move |_| {
                                        open.set(Open::Map);
                                        focus_after_frame(&focus_on_close);
                                    },
                                    "Close"
                                }
                            }
                            if calls.is_empty() {
                                p { class: "kit-atr__meta", "No calls yet." }
                            }
                            ol { class: "kit-atr__call-list", "aria-label": "Calls, oldest first",
                                for (i, c) in calls.iter().enumerate() {
                                    li { key: "{c.call_id}", class: "kit-atr__call",
                                        button {
                                            r#type: "button", "aria-pressed": "{i == idx}",
                                            onclick: move |_| call_index.set(i),
                                            span { "{fmt(c.called_at)}" }
                                            span { class: "kit-atr__tag", "{c.call_state.word()}" }
                                            if c.is_replay { span { class: "kit-atr__tag", "Replay" } }
                                        }
                                    }
                                }
                            }
                            if let Some(c) = selected_call {
                                {
                                    let on_run = c.clone();
                                    let on_key = c.clone();
                                    let tool_key = tool_for_list.clone();
                                    let tool_click = tool_for_list.clone();
                                    let agent_key = agent_for_list.clone();
                                    let agent_click = agent_for_list.clone();
                                    rsx! {
                                        div { class: "kit-atr__call-body",
                                            p { class: "kit-atr__label", "Input" }
                                            pre { class: "kit-atr__code", "{c.call_input}" }
                                            if let Some(out) = c.call_output.clone() {
                                                p { class: "kit-atr__label", "Output" }
                                                pre { class: "kit-atr__code", "{out}" }
                                            }
                                            if let Some(err) = c.call_error.clone() {
                                                p { class: "kit-atr__label", "Error" }
                                                pre { class: "kit-atr__code", "data-tone": "fault", "{err}" }
                                            }
                                            label { class: "kit-atr__label", r#for: "{base}-edit", "Replay with edited input" }
                                            textarea {
                                                id: "{base}-edit", class: "kit-atr__edit", rows: "4", value: "{edit}",
                                                oninput: move |event| edit.set(event.value()),
                                                onkeydown: move |event| {
                                                    if event.key() == Key::Enter && (event.modifiers().meta() || event.modifiers().ctrl()) {
                                                        event.prevent_default();
                                                        submit_replay(&on_key, &edit(), tool_key.as_ref(), &agent_key, on_replay, replay_error);
                                                    }
                                                },
                                            }
                                            if let Some(message) = replay_error() {
                                                p { class: "kit-atr__error", role: "alert", "{message}" }
                                            }
                                            button {
                                                r#type: "button", class: "kit-atr__replay",
                                                onclick: move |_| submit_replay(&on_run, &edit(), tool_click.as_ref(), &agent_click, on_replay, replay_error),
                                                "Replay"
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
