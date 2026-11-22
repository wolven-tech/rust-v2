//! Agent tool explorer: one agent, the tools it may call, and every call it has
//! made. A person sets each tool to run freely, to ask first, or to stay off; they
//! approve or reject a waiting call, connect or disconnect tools, and pause the
//! agent.
//!
//! Design choice: the chart has three layouts. Org stacks the agent over its tools
//! with connector lines; compact drops the connectors and tiles the tools; canvas
//! keeps the org structure on a surface that pans and zooms (Alt with the arrow
//! keys pans, plus and minus zoom, 0 or F fits the view).
//! Design choice: a tool's access is changed with a segmented control. Allowed,
//! Ask first and Paused map to auto, ask and off.
//! Design choice: Disconnect needs a second press on the same button. Calls the
//! tool made stay in the table.
//! Design choice: arrow keys move focus between cards in display order.
//! Design choice: a running call draws a flowing highlight on its connector; the
//! card's label says "running" so reduced motion loses no information.
//! Design choice: the call table is resized from its top edge by drag or by the
//! arrow keys on the grip, between 120 px and 640 px and never past half the
//! viewport. Columns drop with the table's width: run first, then input and tokens.
//! Design choice: below 480 px the open tool is a sheet from below with a Back button.
//! Not implemented: a single tab stop for the chart. Every card is a tab stop.
//! Browser checks: a paused agent shows Paused and no new rows appear; a changed
//! access updates the badge; Approve on a waiting call calls `on_resolve`; the
//! second Disconnect press removes the card and keeps its calls; the failed-calls
//! tile equals the failed rows in range; right arrow moves from the agent card to
//! the nearest tool; canvas pans and zooms; the grip resizes the table; no
//! horizontal scroll at 320 px.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, focus_after_frame, next_id};
use crate::screens::agent_task_list::day_label;

const CSS: Asset = asset!("/assets/agent-tool-explorer.css");

const DAY_MS: u64 = 86_400_000;
const TABLE_MIN_PX: f64 = 120.0;
const TABLE_MAX_PX: f64 = 640.0;
const TABLE_STEP_PX: f64 = 16.0;
const PAN_STEP_PX: f64 = 40.0;
const ZOOM_MIN: f64 = 0.5;
const ZOOM_MAX: f64 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Auto,
    Ask,
    Off,
}

impl Access {
    pub fn key(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Ask => "ask",
            Self::Off => "off",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Auto => "Allowed",
            Self::Ask => "Ask first",
            Self::Off => "Paused",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallState {
    Running,
    Waiting,
    Done,
    Failed,
    Rejected,
}

impl CallState {
    fn word(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeRange {
    Today,
    #[default]
    SevenDays,
    ThirtyDays,
}

impl TimeRange {
    fn window_ms(self) -> u64 {
        match self {
            Self::Today => DAY_MS,
            Self::SevenDays => 7 * DAY_MS,
            Self::ThirtyDays => 30 * DAY_MS,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Today => "Today",
            Self::SevenDays => "Last 7 days",
            Self::ThirtyDays => "Last 30 days",
        }
    }
}

/// How the agent and its tools are drawn in the chart.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartLayout {
    #[default]
    Org,
    Compact,
    Canvas,
}

impl ChartLayout {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Org => "org",
            Self::Compact => "compact",
            Self::Canvas => "canvas",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentInfo {
    pub id: String,
    pub entry_name: String,
    pub task_name: String,
    pub model_name: String,
    pub is_paused: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub entry_name: String,
    pub type_name: String,
    pub is_required: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolInfo {
    pub id: String,
    pub entry_name: String,
    pub function_name: String,
    pub source_label: String,
    pub entry_description: String,
    pub access_mode: Access,
    pub parameter_list: Vec<Param>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallRecord {
    pub id: String,
    pub tool_id: String,
    pub started_at: u64,
    pub duration_ms: Option<u64>,
    pub call_state: CallState,
    pub input_data: String,
    pub output_data: Option<String>,
    pub error_text: Option<String>,
    pub token_count: Option<u64>,
    pub run_id: Option<String>,
}

/// Calls whose start falls in the range ending at `now_at`, newest first.
pub fn calls_in_range(calls: &[CallRecord], now_at: u64, range: TimeRange) -> Vec<CallRecord> {
    let from = now_at.saturating_sub(range.window_ms());
    let mut list: Vec<CallRecord> = calls
        .iter()
        .filter(|c| c.started_at >= from && c.started_at <= now_at)
        .cloned()
        .collect();
    list.sort_by_key(|call| std::cmp::Reverse(call.started_at));
    list
}

/// Share of finished calls that succeeded, in percent; zero with none finished.
pub fn success_share(calls: &[CallRecord]) -> f64 {
    let finished = calls
        .iter()
        .filter(|c| matches!(c.call_state, CallState::Done | CallState::Failed))
        .count();
    if finished == 0 {
        return 0.0;
    }
    let ok = calls
        .iter()
        .filter(|c| c.call_state == CallState::Done)
        .count();
    ok as f64 / finished as f64 * 100.0
}

/// Mean duration of finished calls in milliseconds, zero with none.
pub fn average_duration_ms(calls: &[CallRecord]) -> u64 {
    let durations: Vec<u64> = calls.iter().filter_map(|c| c.duration_ms).collect();
    if durations.is_empty() {
        0
    } else {
        durations.iter().sum::<u64>() / durations.len() as u64
    }
}

/// Access an access control maps to, from its label.
pub fn access_from_label(label: &str) -> Access {
    match label {
        "Allowed" => Access::Auto,
        "Ask first" => Access::Ask,
        _ => Access::Off,
    }
}

/// Access badge text for a tool card.
pub fn access_word(access: Access) -> &'static str {
    access.label()
}

/// Tools in card order: the agent card first, then each tool.
pub fn card_order(tools: &[ToolInfo]) -> Vec<String> {
    tools.iter().map(|t| t.id.clone()).collect()
}

/// The id `delta` cards from `current`, clamped at both ends.
pub fn move_card(order: &[String], current: Option<&str>, delta: isize) -> Option<String> {
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

/// `19 Nov 2026 14:03`, in UTC.
pub fn clock_text(ms: u64) -> String {
    format!(
        "{} {:02}:{:02}",
        day_label(ms),
        (ms / 3_600_000) % 24,
        (ms / 60_000) % 60
    )
}

/// Call table height after a drag or key step, kept between the bounds.
pub fn table_height(from: f64, delta: f64) -> f64 {
    (from + delta).clamp(TABLE_MIN_PX, TABLE_MAX_PX)
}

/// Canvas zoom one step in or out, kept between the bounds.
pub fn zoom_step(zoom: f64, zoom_in: bool) -> f64 {
    let next = if zoom_in { zoom * 1.2 } else { zoom / 1.2 };
    next.clamp(ZOOM_MIN, ZOOM_MAX)
}

/// Chart of one agent and its tools, with a panel and a table of calls.
#[component]
pub fn AgentToolExplorer(
    #[props(default)] agent_info: Option<AgentInfo>,
    #[props(default)] tool_list: Vec<ToolInfo>,
    #[props(default)] call_log: Vec<CallRecord>,
    #[props(default)] now_at: u64,
    #[props(default)] is_live: bool,
    #[props(default)] time_range: TimeRange,
    #[props(default)] chart_layout: ChartLayout,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_select: Option<EventHandler<Option<String>>>,
    #[props(default)] on_resolve: Option<EventHandler<(CallRecord, bool)>>,
    #[props(default)] on_access_change: Option<EventHandler<(String, Access)>>,
    #[props(default)] on_disconnect: Option<EventHandler<String>>,
    #[props(default)] on_add_tool: Option<EventHandler<()>>,
    #[props(default)] on_pause: Option<EventHandler<bool>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
) -> Element {
    let uid = use_hook(next_id);
    let mut range = use_signal(move || time_range);
    let mut selected = use_signal(|| None::<String>);
    let mut disconnect_armed = use_signal(|| None::<String>);
    let mut open_call = use_signal(|| None::<String>);
    let mut access_local = use_signal(Vec::<(String, Access)>::new);
    let mut paused_local = use_signal(|| None::<bool>);
    let mut announce = use_signal(String::new);
    let mut zoom = use_signal(|| 1.0_f64);
    let mut pan = use_signal(|| (0.0_f64, 0.0_f64));
    let mut table_px = use_signal(|| 260.0_f64);
    let mut grip_drag = use_signal(|| None::<(f64, f64)>);
    let mut focus_card = use_signal(|| "agent".to_string());

    let access_now = access_local();
    let tools: Vec<ToolInfo> = tool_list
        .iter()
        .map(|t| {
            let mut t = t.clone();
            if let Some((_, a)) = access_now.iter().find(|(id, _)| *id == t.id) {
                t.access_mode = *a;
            }
            t
        })
        .collect();
    let calls = calls_in_range(&call_log, now_at, range());
    let failed = calls
        .iter()
        .filter(|c| c.call_state == CallState::Failed)
        .count();
    let share = success_share(&calls);
    let average = average_duration_ms(&calls);
    let tokens: u64 = calls.iter().filter_map(|c| c.token_count).sum();
    let total_text = format!("{}", calls.len());
    let failed_text = format!("{failed}");
    let share_text = format!("{share:.0}%");
    let average_text = format!("{average} ms");
    let tokens_text = format!("{tokens}");
    let order = card_order(&tools);
    let key_order = order.clone();
    let paused = paused_local().unwrap_or_else(|| agent_info.as_ref().is_some_and(|a| a.is_paused));
    let selected_tool = selected().and_then(|id| tools.iter().find(|t| t.id == id).cloned());
    let range_now = range();
    let range_label = range_now.label();
    let agent_title = agent_info
        .as_ref()
        .map_or_else(|| "No agent".to_string(), |a| a.entry_name.clone());
    let agent_model = agent_info
        .as_ref()
        .map_or_else(String::new, |a| a.model_name.clone());
    let agent_task = agent_info
        .as_ref()
        .map_or_else(String::new, |a| a.task_name.clone());
    let agent_id = agent_info.as_ref().map(|a| a.id.clone());
    let pause_label = if paused {
        "Resume agent"
    } else {
        "Pause agent"
    };
    let state_word = if paused {
        "Paused"
    } else if is_live {
        "Working"
    } else {
        "Idle"
    };
    let state_key = if paused {
        "paused"
    } else if is_live {
        "working"
    } else {
        "idle"
    };
    let armed = disconnect_armed();
    let open_now = open_call();
    let agent_card_id = format!("kit-ate-{uid}-agent");
    let focus_base = format!("kit-ate-{uid}");
    let active_card = {
        let wanted = focus_card();
        if wanted == "agent" || tools.iter().any(|t| t.id == wanted) {
            wanted
        } else {
            "agent".to_string()
        }
    };
    let layout_now = chart_layout;
    let canvas = layout_now == ChartLayout::Canvas;
    let (pan_x, pan_y) = pan();
    let zoom_value = zoom();
    let stage_style = if canvas {
        format!("transform: translate({pan_x}px, {pan_y}px) scale({zoom_value});")
    } else {
        String::new()
    };
    let table_now = table_px();
    let table_style = format!("height: {table_now}px;");
    let table_value = table_now.round() as i64;

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: "kit-ate",
            "data-loading": is_loading.then_some("true"),
            "aria-busy": is_loading.then_some("true"),
            onpointermove: move |event: PointerEvent| {
                if let Some((start_y, start_h)) = grip_drag() {
                    table_px.set(table_height(start_h, start_y - event.client_coordinates().y));
                }
            },
            onpointerup: move |_| grip_drag.set(None),
            onpointercancel: move |_| grip_drag.set(None),
            onpointerleave: move |_| grip_drag.set(None),
            header { class: "kit-ate__head",
                div { class: "kit-ate__heading",
                    h2 { class: "kit-ate__title", "{agent_title}" }
                    span { class: "kit-ate__state", "data-state": "{state_key}", "{state_word}" }
                }
                div { class: "kit-ate__tools",
                    select {
                        class: "kit-ate__input",
                        "aria-label": "Time range",
                        value: "{range_now:?}",
                        onchange: move |event| {
                            range.set(match event.value().as_str() {
                                "Today" => TimeRange::Today,
                                "ThirtyDays" => TimeRange::ThirtyDays,
                                _ => TimeRange::SevenDays,
                            });
                        },
                        option { value: "Today", "Today" }
                        option { value: "SevenDays", "Last 7 days" }
                        option { value: "ThirtyDays", "Last 30 days" }
                    }
                    button {
                        r#type: "button",
                        class: "kit-ate__ghost",
                        onclick: move |_| {
                            if let Some(handler) = on_add_tool {
                                handler.call(());
                            }
                        },
                        "Connect tool"
                    }
                    button {
                        r#type: "button",
                        class: "kit-ate__primary",
                        onclick: move |_| {
                            let next = !paused;
                            paused_local.set(Some(next));
                            announce.set(if next { "Agent paused".to_string() } else { "Agent resumed".to_string() });
                            if let Some(handler) = on_pause {
                                handler.call(next);
                            }
                        },
                        "{pause_label}"
                    }
                }
            }

            section { class: "kit-ate__tiles", "aria-label": "Figures",
                div { class: "kit-ate__tile",
                    span { class: "kit-ate__label", "Failed" }
                    span { class: "kit-ate__value", "{failed_text}" }
                }
                div { class: "kit-ate__tile",
                    span { class: "kit-ate__label", "Calls" }
                    span { class: "kit-ate__value", "{total_text}" }
                }
                div { class: "kit-ate__tile",
                    span { class: "kit-ate__label", "Success" }
                    span { class: "kit-ate__value", "{share_text}" }
                }
                div { class: "kit-ate__tile",
                    span { class: "kit-ate__label", "Average" }
                    span { class: "kit-ate__value", "{average_text}" }
                }
                div { class: "kit-ate__tile",
                    span { class: "kit-ate__label", "Tokens" }
                    span { class: "kit-ate__value", "{tokens_text}" }
                }
            }

            if is_loading {
                div { class: "kit-ate__ph", "aria-hidden": "true" }
                div { class: "kit-ate__ph kit-ate__ph--table", "aria-hidden": "true" }
            } else {
                div {
                    class: "kit-ate__body",
                    "data-open": selected_tool.is_some().then_some("true"),
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() != Key::Escape {
                            return;
                        }
                        if let Some(id) = selected() {
                            event.prevent_default();
                            selected.set(None);
                            focus_after_frame(&format!("kit-ate-{uid}-tool-{id}"));
                        }
                    },
                    div {
                        class: "kit-ate__chart",
                        "data-layout": layout_now.as_str(),
                        onkeydown: move |event: KeyboardEvent| {
                            if !canvas {
                                return;
                            }
                            let alt = event.modifiers().alt();
                            let handled = match event.key() {
                                Key::ArrowLeft if alt => {
                                    pan.with_mut(|p| p.0 -= PAN_STEP_PX);
                                    true
                                }
                                Key::ArrowRight if alt => {
                                    pan.with_mut(|p| p.0 += PAN_STEP_PX);
                                    true
                                }
                                Key::ArrowUp if alt => {
                                    pan.with_mut(|p| p.1 -= PAN_STEP_PX);
                                    true
                                }
                                Key::ArrowDown if alt => {
                                    pan.with_mut(|p| p.1 += PAN_STEP_PX);
                                    true
                                }
                                Key::Character(c) => match c.as_str() {
                                    "+" | "=" => {
                                        zoom.set(zoom_step(zoom(), true));
                                        true
                                    }
                                    "-" | "_" => {
                                        zoom.set(zoom_step(zoom(), false));
                                        true
                                    }
                                    "0" | "f" => {
                                        zoom.set(1.0);
                                        pan.set((0.0, 0.0));
                                        true
                                    }
                                    _ => false,
                                },
                                _ => false,
                            };
                            if handled {
                                event.prevent_default();
                            }
                        },
                        if canvas {
                            div { class: "kit-ate__canvas-tools", role: "group", "aria-label": "Canvas view",
                                button {
                                    r#type: "button",
                                    class: "kit-ate__ghost",
                                    "aria-label": "Zoom in",
                                    onclick: move |_| zoom.set(zoom_step(zoom(), true)),
                                    "+"
                                }
                                button {
                                    r#type: "button",
                                    class: "kit-ate__ghost",
                                    "aria-label": "Zoom out",
                                    onclick: move |_| zoom.set(zoom_step(zoom(), false)),
                                    "−"
                                }
                                button {
                                    r#type: "button",
                                    class: "kit-ate__ghost",
                                    onclick: move |_| {
                                        zoom.set(1.0);
                                        pan.set((0.0, 0.0));
                                    },
                                    "Fit view"
                                }
                            }
                        }
                        div { class: "kit-ate__stage", style: "{stage_style}",
                            div { class: "kit-ate__agent-row",
                                button {
                                    id: "{agent_card_id}",
                                    r#type: "button",
                                    class: "kit-ate__card kit-ate__card--agent",
                                    "data-state": "{state_key}",
                                    "aria-label": "{agent_title}, {state_word}",
                                    tabindex: if active_card == "agent" { "0" } else { "-1" },
                                    onfocus: move |_| focus_card.set("agent".to_string()),
                                    onclick: move |_| selected.set(None),
                                    onkeydown: {
                                        let order_nav = key_order.clone();
                                        let base = focus_base.clone();
                                        move |event: KeyboardEvent| {
                                            if event.modifiers().alt() {
                                                return;
                                            }
                                            let delta = match event.key() {
                                                Key::ArrowRight | Key::ArrowDown => 1,
                                                Key::ArrowLeft | Key::ArrowUp => -1,
                                                _ => return,
                                            };
                                            if let Some(target) = move_card(&order_nav, None, delta) {
                                                focus_after_frame(&format!("{base}-tool-{target}"));
                                            }
                                        }
                                    },
                                    span { class: "kit-ate__name", "{agent_title}" }
                                    span { class: "kit-ate__meta", "{agent_model}" }
                                    span { class: "kit-ate__meta", "{agent_task}" }
                                }
                            }
                            if let Some(agent) = agent_id.clone() {
                                span { class: "kit-ate__stem", "aria-hidden": "true", "data-agent": "{agent}" }
                            }
                            if tools.is_empty() {
                                div { class: "kit-ate__slot", "aria-label": "No tools connected", "Connect a tool to give the agent a capability." }
                            }
                            ul { class: "kit-ate__tool-row", role: "list",
                                for tool in tools.iter().cloned() {
                                    {
                                        let tool_key = tool.id.clone();
                                        let tool_name = tool.entry_name.clone();
                                        let tool_source = tool.source_label.clone();
                                        let access_word = access_word(tool.access_mode);
                                        let access_key = tool.access_mode.key();
                                        let is_selected = selected().as_deref() == Some(tool.id.as_str());
                                        let card_id = format!("{focus_base}-tool-{}", tool.id);
                                        let in_use = calls.iter().any(|c| c.tool_id == tool.id && c.call_state == CallState::Running);
                                        let pick = tool.id.clone();
                                        let card_label = if in_use {
                                            format!("{tool_name}, {tool_source}, {access_word}, running")
                                        } else {
                                            format!("{tool_name}, {tool_source}, {access_word}")
                                        };
                                        let order_nav = key_order.clone();
                                        let here = tool.id.clone();
                                        let is_stop = active_card == here;
                                        let focus_id = tool.id.clone();
                                        let base = focus_base.clone();
                                        rsx! {
                                            li { key: "{tool_key}", class: "kit-ate__item",
                                                span { class: "kit-ate__link-line", "aria-hidden": "true", "data-flow": in_use.then_some("true") }
                                                button {
                                                    id: "{card_id}",
                                                    r#type: "button",
                                                    class: "kit-ate__card",
                                                    "data-access": "{access_key}",
                                                    "aria-label": "{card_label}",
                                                    "aria-current": is_selected.then_some("true"),
                                                    tabindex: if is_stop { "0" } else { "-1" },
                                                    onfocus: move |_| focus_card.set(focus_id.clone()),
                                                    onclick: move |_| {
                                                        selected.set(Some(pick.clone()));
                                                        disconnect_armed.set(None);
                                                        if let Some(handler) = on_select {
                                                            handler.call(Some(pick.clone()));
                                                        }
                                                    },
                                                    onkeydown: move |event: KeyboardEvent| {
                                                        if event.modifiers().alt() {
                                                            return;
                                                        }
                                                        let delta = match event.key() {
                                                            Key::ArrowRight | Key::ArrowDown => 1,
                                                            Key::ArrowLeft | Key::ArrowUp => -1,
                                                            _ => return,
                                                        };
                                                        if let Some(target) = move_card(&order_nav, Some(here.as_str()), delta) {
                                                            focus_after_frame(&format!("{base}-tool-{target}"));
                                                        }
                                                    },
                                                    span { class: "kit-ate__name", "{tool_name}" }
                                                    span { class: "kit-ate__meta", "{tool_source}" }
                                                    span { class: "kit-ate__access", "{access_word}" }
                                                    if in_use {
                                                        span { class: "kit-ate__live", "aria-hidden": "true" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(tool) = selected_tool.clone() {
                        {
                            let tool_id = tool.id.clone();
                            let access_now_key = tool.access_mode;
                            let access_options = [Access::Auto, Access::Ask, Access::Off];
                            let description = tool.entry_description.clone();
                            let title = tool.entry_name.clone();
                            let params: Vec<Param> = tool.parameter_list.clone();
                            let armed_here = armed.as_deref() == Some(tool_id.as_str());
                            let recent: Vec<CallRecord> = calls.iter().filter(|c| c.tool_id == tool_id).take(5).cloned().collect();
                            let disconnect_target = tool_id.clone();
                            let panel_label = format!("Tool: {title}");
                            let armed_id = tool_id.clone();
                            let back_card = format!("{focus_base}-tool-{tool_id}");
                            rsx! {
                                aside { class: "kit-ate__panel", "aria-label": "{panel_label}",
                                    button {
                                        r#type: "button",
                                        class: "kit-ate__back",
                                        onclick: move |_| {
                                            selected.set(None);
                                            focus_after_frame(&back_card);
                                        },
                                        "Back"
                                    }
                                    h3 { class: "kit-ate__panel-title", "{title}" }
                                    div { class: "kit-ate__segmented", role: "radiogroup", "aria-label": "Access",
                                        for option in access_options {
                                            {
                                                let option_label = option.label();
                                                let option_checked = (access_now_key == option).to_string();
                                                let change_id = tool_id.clone();
                                                rsx! {
                                                    button {
                                                        key: "{option_label}",
                                                        r#type: "button",
                                                        role: "radio",
                                                        class: "kit-ate__seg",
                                                        "aria-checked": "{option_checked}",
                                                        onclick: move |_| {
                                                            access_local.with_mut(|list| {
                                                                list.retain(|(id, _)| *id != change_id);
                                                                list.push((change_id.clone(), option));
                                                            });
                                                            announce.set(format!("Access set to {option_label}"));
                                                            if let Some(handler) = on_access_change {
                                                                handler.call((change_id.clone(), option));
                                                            }
                                                        },
                                                        "{option_label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    p { class: "kit-ate__meta", "{description}" }
                                    ul { class: "kit-ate__params",
                                        for param in params.iter().cloned() {
                                            {
                                                let param_key = param.entry_name.clone();
                                                let param_line = format!(
                                                    "{}: {}{}",
                                                    param.entry_name,
                                                    param.type_name,
                                                    if param.is_required { " (required)" } else { "" }
                                                );
                                                rsx! { li { key: "{param_key}", "{param_line}" } }
                                            }
                                        }
                                    }
                                    section { class: "kit-ate__block", "aria-label": "Recent calls",
                                        h4 { class: "kit-ate__section", "Recent calls" }
                                        if recent.is_empty() {
                                            p { class: "kit-ate__meta", "No calls in this range." }
                                        }
                                        ul { class: "kit-ate__params",
                                            for call in recent.iter().cloned() {
                                                {
                                                    let call_key = call.id.clone();
                                                    let call_line = format!("{} · {}", call.input_data, call.call_state.word());
                                                    rsx! { li { key: "{call_key}", "{call_line}" } }
                                                }
                                            }
                                        }
                                    }
                                    div { class: "kit-ate__danger-row",
                                        if armed_here {
                                            span { "Disconnect {title}? Its calls stay in the table." }
                                            button {
                                                r#type: "button",
                                                class: "kit-ate__danger",
                                                onclick: move |_| {
                                                    disconnect_armed.set(None);
                                                    selected.set(None);
                                                    if let Some(handler) = on_disconnect {
                                                        handler.call(disconnect_target.clone());
                                                    }
                                                },
                                                "Disconnect"
                                            }
                                        } else {
                                            button {
                                                r#type: "button",
                                                class: "kit-ate__ghost",
                                                onclick: move |_| disconnect_armed.set(Some(armed_id.clone())),
                                                "Disconnect"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                section { class: "kit-ate__table-wrap", "aria-label": "Calls", style: "{table_style}",
                    div {
                        class: "kit-ate__grip",
                        role: "separator",
                        tabindex: "0",
                        "aria-orientation": "horizontal",
                        "aria-label": "Resize call table",
                        "aria-valuemin": "120",
                        "aria-valuemax": "640",
                        "aria-valuenow": "{table_value}",
                        onpointerdown: move |event: PointerEvent| {
                            grip_drag.set(Some((event.client_coordinates().y, table_px())));
                        },
                        onkeydown: move |event: KeyboardEvent| {
                            let step = match event.key() {
                                Key::ArrowUp => TABLE_STEP_PX,
                                Key::ArrowDown => -TABLE_STEP_PX,
                                _ => return,
                            };
                            event.prevent_default();
                            table_px.set(table_height(table_px(), step));
                        },
                    }
                    div { class: "kit-ate__range", "{range_label}" }
                    table { class: "kit-ate__table",
                        thead {
                            tr {
                                th { "Time" }
                                th { "Tool" }
                                th { class: "kit-ate__col-input", "Input" }
                                th { "State" }
                                th { "Duration" }
                                th { class: "kit-ate__col-tokens", "Tokens" }
                                th { class: "kit-ate__col-run", "Run" }
                            }
                        }
                        tbody {
                            if is_empty {
                                tr { class: "kit-ate__empty-row",
                                    td { colspan: "7",
                                        span { "No tools connected, so the agent has no calls yet." }
                                        button {
                                            r#type: "button",
                                            class: "kit-ate__primary",
                                            onclick: move |_| {
                                                if let Some(handler) = on_add_tool {
                                                    handler.call(());
                                                }
                                            },
                                            "Connect tool"
                                        }
                                    }
                                }
                            }
                            for call in calls.iter().cloned() {
                                {
                                    let call_key = call.id.clone();
                                    let tool_name = tools.iter().find(|t| t.id == call.tool_id).map_or_else(|| "Unknown tool".to_string(), |t| t.entry_name.clone());
                                    let state_word_call = call.call_state.word();
                                    let duration = call.duration_ms.map_or_else(|| "—".to_string(), |d| format!("{d} ms"));
                                    let tokens_cell = call.token_count.map_or_else(|| "—".to_string(), |t| t.to_string());
                                    let time_cell = clock_text(call.started_at);
                                    let run_cell = call.run_id.clone().unwrap_or_else(|| "—".to_string());
                                    let input_cell = call.input_data.clone();
                                    let is_open_row = open_now.as_deref() == Some(call.id.as_str());
                                    let approve_id = call.id.clone();
                                    let reject_id = call.id.clone();
                                    let waiting = call.call_state == CallState::Waiting;
                                    let toggle_id = call.id.clone();
                                    let approve_call = call.clone();
                                    let reject_call = call.clone();
                                    let input_line = call.input_data.clone();
                                    let output_line = call.output_data.clone().or(call.error_text.clone()).unwrap_or_default();
                                    rsx! {
                                        tr { key: "{call_key}", "data-state": "{state_word_call}",
                                            td {
                                                button {
                                                    r#type: "button",
                                                    class: "kit-ate__link",
                                                    "aria-expanded": "{is_open_row}",
                                                    onclick: move |_| {
                                                        let next = if open_call().as_deref() == Some(toggle_id.as_str()) { None } else { Some(toggle_id.clone()) };
                                                        open_call.set(next);
                                                    },
                                                    "{time_cell}"
                                                }
                                            }
                                            td { "{tool_name}" }
                                            td { class: "kit-ate__col-input",
                                                span { class: "kit-ate__input-cell", "{input_cell}" }
                                            }
                                            td {
                                                span { class: "kit-ate__state-word", "{state_word_call}" }
                                                if waiting {
                                                    span { class: "kit-ate__row-actions",
                                                        button {
                                                            r#type: "button",
                                                            class: "kit-ate__primary",
                                                            "aria-label": "Approve call {approve_id}",
                                                            onclick: move |_| {
                                                                if let Some(handler) = on_resolve {
                                                                    handler.call((approve_call.clone(), true));
                                                                }
                                                            },
                                                            "Approve"
                                                        }
                                                        button {
                                                            r#type: "button",
                                                            class: "kit-ate__ghost",
                                                            "aria-label": "Reject call {reject_id}",
                                                            onclick: move |_| {
                                                                if let Some(handler) = on_resolve {
                                                                    handler.call((reject_call.clone(), false));
                                                                }
                                                            },
                                                            "Reject"
                                                        }
                                                    }
                                                }
                                            }
                                            td { "{duration}" }
                                            td { class: "kit-ate__col-tokens", "{tokens_cell}" }
                                            td { class: "kit-ate__col-run", "{run_cell}" }
                                        }
                                        if is_open_row {
                                            tr { class: "kit-ate__detail",
                                                td { colspan: "7",
                                                    pre { class: "kit-ate__pre", "{input_line}" }
                                                    pre { class: "kit-ate__pre", "{output_line}" }
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

            if paused {
                p { class: "kit-ate__meta", "The agent is paused. No new calls start until it resumes." }
            }
            if let Some(agent) = agent_id {
                input { r#type: "hidden", value: "{agent}" }
            }
            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
