//! Tool call network: agents, the tools they invoke and the person they report to, as
//! one live graph. Each invocation travels down its link; its reply travels back.
//!
//! Design choice: links are straight lines, not right-angle bends; parallel-link
//! ordering is not drawn.
//! Design choice: pulses are positioned spans animated by CSS from the request's
//! endpoints, so the component schedules no loop of its own. The clock reads only
//! `clock_now_ms` (the host's time); absent, per-minute counts read as zero.
//! Design choice: the visually hidden list is the text equivalent and is always present.
//! Browser checks: centre agent first on first draw, then agents, then outer tools; pulses
//! and return pulses; the error flash; node card focus and Escape; the neighbourhood
//! highlight on focus; the waiting and error ripple; reduced motion lights links instead.

use dioxus::prelude::*;

use crate::agents::single_run_status_card::{format_usd, initial_of};
use crate::internal::{KIT_CSS, focus_after_frame, next_id};

const CSS: Asset = asset!("/assets/tool-call-network.css");

/// Ring radii and the canvas extent, in the 0..1000 space every position uses.
const INNER_RADIUS: f64 = 230.0;
const OUTER_RADIUS: f64 = 430.0;
const CENTRE: f64 = 500.0;
/// Angular gap between outer nodes that face the same agent, so their cards do not stack.
const OUTER_SPREAD: f64 = 0.32;
const MINUTE_MS: u64 = 60_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeRole {
    Agent,
    Tool,
    User,
}

impl NodeRole {
    fn word(self) -> &'static str {
        match self {
            Self::Agent => "Agent",
            Self::Tool => "Tool",
            Self::User => "Person",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Condition {
    #[default]
    Idle,
    Working,
    Waiting,
    Error,
    Done,
}

impl Condition {
    pub fn word(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Working => "Working",
            Self::Waiting => "Needs you",
            Self::Error => "Failed",
            Self::Done => "Done",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::Waiting => "waiting",
            Self::Error => "error",
            Self::Done => "done",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ExchangeResult {
    #[default]
    Pending,
    Ok,
    Error,
}

impl ExchangeResult {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Ok => "ok",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub node_role: NodeRole,
    pub compact_label: Option<String>,
    pub glyph: Option<String>,
    pub summary_line: Option<String>,
    pub condition: Condition,
    pub activity: Option<String>,
    pub spend: Option<f64>,
    pub is_centre: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Exchange {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub sent_at_ms: u64,
    pub result: ExchangeResult,
    pub caption: Option<String>,
    pub latency_ms: Option<u64>,
}

/// Position of each entity in the 0..1000 square. The centre agent sits in the middle, other
/// agents on the inner ring, and tools and people on the outer ring facing their first linked agent.
pub fn layout(entities: &[Entity], links: &[(String, String)]) -> Vec<(String, f64, f64)> {
    let centre_index = entities
        .iter()
        .position(|e| e.node_role == NodeRole::Agent && e.is_centre)
        .or_else(|| entities.iter().position(|e| e.node_role == NodeRole::Agent));
    let inner: Vec<usize> = entities
        .iter()
        .enumerate()
        .filter(|(i, e)| e.node_role == NodeRole::Agent && Some(*i) != centre_index)
        .map(|(i, _)| i)
        .collect();
    let mut placed: Vec<(String, f64, f64)> = Vec::with_capacity(entities.len());
    let mut angle_of: Vec<(String, f64)> = Vec::new();
    if let Some(c) = centre_index {
        placed.push((entities[c].id.clone(), CENTRE, CENTRE));
    }
    let count = inner.len().max(1) as f64;
    for (slot, &i) in inner.iter().enumerate() {
        let angle = -std::f64::consts::FRAC_PI_2 + std::f64::consts::TAU * slot as f64 / count;
        placed.push((
            entities[i].id.clone(),
            CENTRE + INNER_RADIUS * angle.cos(),
            CENTRE + INNER_RADIUS * angle.sin(),
        ));
        angle_of.push((entities[i].id.clone(), angle));
    }
    let outer: Vec<usize> = entities
        .iter()
        .enumerate()
        .filter(|(_, e)| e.node_role != NodeRole::Agent)
        .map(|(i, _)| i)
        .collect();
    let outer_count = outer.len().max(1) as f64;
    let bases: Vec<f64> = outer
        .iter()
        .enumerate()
        .map(|(slot, &i)| {
            let id = &entities[i].id;
            let facing = links.iter().find_map(|(a, b)| {
                if a == id {
                    Some(b)
                } else if b == id {
                    Some(a)
                } else {
                    None
                }
            });
            facing
                .and_then(|partner| angle_of.iter().find(|(pid, _)| pid == partner))
                .map(|(_, angle)| *angle)
                .unwrap_or(
                    -std::f64::consts::FRAC_PI_2
                        + std::f64::consts::TAU * slot as f64 / outer_count,
                )
        })
        .collect();
    for (k, &i) in outer.iter().enumerate() {
        let base = bases[k];
        let group: Vec<usize> = (0..bases.len())
            .filter(|&j| (bases[j] - base).abs() < 1e-9)
            .collect();
        let rank = group.iter().position(|&j| j == k).unwrap_or(0) as f64;
        let angle = base + (rank - (group.len() as f64 - 1.0) / 2.0) * OUTER_SPREAD;
        placed.push((
            entities[i].id.clone(),
            CENTRE + OUTER_RADIUS * angle.cos(),
            CENTRE + OUTER_RADIUS * angle.sin(),
        ));
    }
    placed
}

/// Exchanges with this entity as either end, in the minute ending at `now_ms`.
pub fn exchanges_last_minute(exchanges: &[Exchange], id: &str, now_ms: u64) -> usize {
    let since = now_ms.saturating_sub(MINUTE_MS);
    exchanges
        .iter()
        .filter(|x| {
            (x.from_id == id || x.to_id == id) && x.sent_at_ms >= since && x.sent_at_ms <= now_ms
        })
        .count()
}

/// Entities linked to `id`, directly or by any exchange.
pub fn neighbourhood(links: &[(String, String)], exchanges: &[Exchange], id: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let pairs = links.iter().map(|(a, b)| (a.as_str(), b.as_str())).chain(
        exchanges
            .iter()
            .map(|x| (x.from_id.as_str(), x.to_id.as_str())),
    );
    for (a, b) in pairs {
        let other = if a == id {
            b
        } else if b == id {
            a
        } else {
            continue;
        };
        if !out.iter().any(|known| known == other) {
            out.push(other.to_string());
        }
    }
    out
}

/// Stroke width for a link: thicker as its recent exchange count rises.
pub fn link_width(recent: usize) -> f64 {
    1.0 + recent.min(6) as f64 * 0.5
}

/// Text equivalent: one line per entity with its condition and its exchanges in the last minute.
pub fn text_summary(entities: &[Entity], exchanges: &[Exchange], now_ms: u64) -> Vec<String> {
    entities
        .iter()
        .map(|e| {
            let count = exchanges_last_minute(exchanges, &e.id, now_ms);
            format!(
                "{}: {}, {} in the last minute",
                e.name,
                e.condition.word(),
                exchange_phrase(count)
            )
        })
        .collect()
}

/// `1 exchange` or `n exchanges`.
pub fn exchange_phrase(count: usize) -> String {
    if count == 1 {
        "1 exchange".to_string()
    } else {
        format!("{count} exchanges")
    }
}

/// Counts of entities that need the person and that have failed.
pub fn attention_counts(entities: &[Entity]) -> (usize, usize) {
    let waiting = entities
        .iter()
        .filter(|e| e.condition == Condition::Waiting)
        .count();
    let failed = entities
        .iter()
        .filter(|e| e.condition == Condition::Error)
        .count();
    (waiting, failed)
}

/// Accessible name of a node.
pub fn node_name(entity: &Entity, recent: usize) -> String {
    let mut parts = vec![
        entity.name.clone(),
        entity.node_role.word().to_string(),
        entity.condition.word().to_string(),
    ];
    if let Some(activity) = &entity.activity {
        parts.push(activity.clone());
    }
    parts.push(format!("{} in the last minute", exchange_phrase(recent)));
    parts.join(", ")
}

fn percent(v: f64) -> String {
    format!("{:.2}%", v / 10.0)
}

#[derive(Clone, PartialEq, Debug)]
struct NodeView {
    id: String,
    name: String,
    role: &'static str,
    condition: &'static str,
    condition_word: &'static str,
    glyph: String,
    summary: Option<String>,
    style: String,
    aria: String,
    neighbour: bool,
    expanded: bool,
    attention: bool,
}

#[derive(Clone, PartialEq, Debug)]
struct LinkView {
    key: String,
    x1: String,
    y1: String,
    x2: String,
    y2: String,
    width: String,
    busy: bool,
}

#[derive(Clone, PartialEq, Debug)]
struct PulseView {
    key: String,
    style: String,
    result: &'static str,
    phase: &'static str,
}

#[component]
pub fn ToolCallNetwork(
    entities: Vec<Entity>,
    #[props(default)] links: Vec<(String, String)>,
    #[props(default)] exchanges: Vec<Exchange>,
    #[props(default)] clock_now_ms: Option<u64>,
    #[props(default)] on_focus_node: Option<EventHandler<Option<Entity>>>,
    #[props(default = "Agent graph".to_string(), into)] heading: String,
    #[props(default)] format_spend: Option<Callback<f64, String>>,
    #[props(default)] class_name: String,
) -> Element {
    let id = use_hook(|| format!("kit-tcn-{}", next_id()));
    let mut selected = use_signal(|| None::<String>);
    let mut focused = use_signal(|| None::<String>);
    let now = clock_now_ms.unwrap_or(0);

    let placed = layout(&entities, &links);
    let position = |key: &str| {
        placed
            .iter()
            .find(|(pid, _, _)| pid == key)
            .map(|(_, x, y)| (*x, *y))
    };
    let agents = entities
        .iter()
        .filter(|e| e.node_role == NodeRole::Agent)
        .count();
    let tools = entities
        .iter()
        .filter(|e| e.node_role == NodeRole::Tool)
        .count();
    let per_minute = exchanges
        .iter()
        .filter(|x| x.sent_at_ms + MINUTE_MS >= now && x.sent_at_ms <= now)
        .count();
    let spend_total: f64 = entities.iter().filter_map(|e| e.spend).sum();
    let spend_text = match format_spend {
        Some(format) => format.call(spend_total),
        None => format_usd(spend_total),
    };
    let (waiting, failed) = attention_counts(&entities);
    let focus_ref = focused();
    let highlight: Vec<String> = focus_ref
        .as_deref()
        .map(|f| neighbourhood(&links, &exchanges, f))
        .unwrap_or_default();

    let nodes: Vec<(String, NodeView)> = entities
        .iter()
        .map(|e| {
            let (x, y) = position(&e.id).unwrap_or((CENTRE, CENTRE));
            let recent = exchanges_last_minute(&exchanges, &e.id, now);
            (
                e.id.clone(),
                NodeView {
                    id: e.id.clone(),
                    name: e.compact_label.clone().unwrap_or_else(|| e.name.clone()),
                    role: e.node_role.word(),
                    condition: e.condition.as_str(),
                    condition_word: e.condition.word(),
                    glyph: e.glyph.clone().unwrap_or_else(|| initial_of(&e.name)),
                    summary: e.summary_line.clone(),
                    style: format!("left: {}; top: {}", percent(x), percent(y)),
                    aria: node_name(e, recent),
                    neighbour: highlight.contains(&e.id),
                    expanded: selected().as_deref() == Some(e.id.as_str()),
                    attention: matches!(e.condition, Condition::Waiting | Condition::Error),
                },
            )
        })
        .collect();

    let mut link_pairs: Vec<(String, String)> = links.clone();
    for x in &exchanges {
        let known = link_pairs
            .iter()
            .any(|(a, b)| (a == &x.from_id && b == &x.to_id) || (a == &x.to_id && b == &x.from_id));
        if !known {
            link_pairs.push((x.from_id.clone(), x.to_id.clone()));
        }
    }
    let line_views: Vec<(String, LinkView)> = link_pairs
        .iter()
        .filter_map(|(a, b)| {
            let (x1, y1) = position(a)?;
            let (x2, y2) = position(b)?;
            let recent = exchanges
                .iter()
                .filter(|x| {
                    (x.from_id == *a && x.to_id == *b) || (x.from_id == *b && x.to_id == *a)
                })
                .filter(|x| x.sent_at_ms + MINUTE_MS >= now)
                .count();
            let key = format!("{a}|{b}");
            Some((
                key.clone(),
                LinkView {
                    key,
                    x1: format!("{x1:.1}"),
                    y1: format!("{y1:.1}"),
                    x2: format!("{x2:.1}"),
                    y2: format!("{y2:.1}"),
                    width: format!("{:.1}", link_width(recent)),
                    busy: recent > 0,
                },
            ))
        })
        .collect();

    let mut pulses: Vec<(String, PulseView)> = Vec::new();
    for x in &exchanges {
        let (Some((fx, fy)), Some((tx, ty))) = (position(&x.from_id), position(&x.to_id)) else {
            continue;
        };
        let style = format!(
            "--fx: {}; --fy: {}; --tx: {}; --ty: {}",
            percent(fx),
            percent(fy),
            percent(tx),
            percent(ty)
        );
        pulses.push((
            format!("{}-out", x.id),
            PulseView {
                key: format!("{}-out", x.id),
                style: style.clone(),
                result: x.result.as_str(),
                phase: "out",
            },
        ));
        if x.result != ExchangeResult::Pending {
            let back = format!(
                "--fx: {}; --fy: {}; --tx: {}; --ty: {}",
                percent(tx),
                percent(ty),
                percent(fx),
                percent(fy)
            );
            pulses.push((
                format!("{}-back", x.id),
                PulseView {
                    key: format!("{}-back", x.id),
                    style: back,
                    result: x.result.as_str(),
                    phase: "back",
                },
            ));
        }
    }

    let selected_entity = selected().and_then(|key| entities.iter().find(|e| e.id == key).cloned());
    let card_recent: Vec<(String, String, String)> = selected_entity
        .as_ref()
        .map(|entity| {
            exchanges
                .iter()
                .rev()
                .filter(|x| x.from_id == entity.id || x.to_id == entity.id)
                .take(3)
                .map(|x| {
                    let other = if x.from_id == entity.id {
                        &x.to_id
                    } else {
                        &x.from_id
                    };
                    let target = entities
                        .iter()
                        .find(|e| &e.id == other)
                        .map(|e| e.name.clone())
                        .unwrap_or_default();
                    let latency = x
                        .latency_ms
                        .map(|ms| format!("{:.1} s", ms as f64 / 1000.0))
                        .unwrap_or_default();
                    (x.id.clone(), target, latency)
                })
                .collect()
        })
        .unwrap_or_default();
    let card_spend = selected_entity
        .as_ref()
        .and_then(|e| e.spend)
        .map(|amount| format_usd(amount))
        .unwrap_or_default();
    let card_name = selected_entity
        .as_ref()
        .map(|e| e.name.clone())
        .unwrap_or_default();
    let card_activity = selected_entity.as_ref().and_then(|e| e.activity.clone());
    let card_condition = selected_entity
        .as_ref()
        .map(|e| e.condition.word())
        .unwrap_or_default();
    let card_id = format!("{id}-card");
    let close_id = format!("{id}-close");
    let text_lines = text_summary(&entities, &exchanges, now);
    let agent_count = format!("{agents} agents");
    let tool_count = format!("{tools} tools");
    let minute_count = format!("{per_minute} exchanges / min");
    let attention_text = format!("{waiting} needs you · {failed} failed");
    let live_text = attention_text.clone();
    let class = format!("kit-tcn {class_name}");
    let heading_id = format!("{id}-heading");
    let on_focus = on_focus_node;
    let entities_by_key = entities.clone();
    let notify = Callback::new(move |next: Option<String>| {
        if let Some(handler) = &on_focus {
            let entity = next.and_then(|k| entities_by_key.iter().find(|e| e.id == k).cloned());
            handler.call(entity);
        }
    });
    let entity_for_close = selected_entity.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", "aria-labelledby": "{heading_id}",
            header { class: "kit-tcn__head",
                h3 { id: "{heading_id}", class: "kit-tcn__heading", {heading} }
                span { class: "kit-tcn__stat", {agent_count} }
                span { class: "kit-tcn__stat", {tool_count} }
                span { class: "kit-tcn__stat", {minute_count} }
                span { class: "kit-tcn__spend", {spend_text} }
                if waiting > 0 {
                    span { class: "kit-tcn__chip", "data-tone": "waiting", tabindex: "0", "Needs you {waiting}" }
                }
                if failed > 0 {
                    span { class: "kit-tcn__chip", "data-tone": "error", tabindex: "0", "Failed {failed}" }
                }
            }
            span { class: "kit-tcn__live", role: "status", "aria-live": "polite", {live_text} }
            div { class: "kit-tcn__canvas",
                svg {
                    class: "kit-tcn__links",
                    view_box: "0 0 1000 1000",
                    preserve_aspect_ratio: "none",
                    "aria-hidden": "true",
                    for (key, link) in line_views {
                        line {
                            key: "{key}",
                            x1: link.x1,
                            y1: link.y1,
                            x2: link.x2,
                            y2: link.y2,
                            "stroke-width": link.width,
                            "data-busy": link.busy.then_some("true"),
                        }
                    }
                }
                for (key, pulse) in pulses {
                    span {
                        key: "{key}",
                        class: "kit-tcn__pulse",
                        style: pulse.style,
                        "data-result": pulse.result,
                        "data-phase": pulse.phase,
                    }
                }
                for (key, node) in nodes {
                    button {
                        key: "{key}",
                        id: format!("{id}-node-{}", node.id),
                        class: "kit-tcn__node",
                        r#type: "button",
                        style: node.style,
                        "data-condition": node.condition,
                        "data-neighbour": node.neighbour.then_some("true"),
                        "aria-expanded": if node.expanded { "true" } else { "false" },
                        "aria-label": node.aria,
                        onfocus: {
                            let key = node.id.clone();
                            move |_| focused.set(Some(key.clone()))
                        },
                        onblur: move |_| focused.set(None),
                        onclick: {
                            let key = node.id.clone();
                            move |_| {
                                let next = if selected().as_deref() == Some(key.as_str()) { None } else { Some(key.clone()) };
                                selected.set(next.clone());
                                notify.call(next);
                            }
                        },
                        span { class: "kit-tcn__glyph", "aria-hidden": "true", {node.glyph.clone()} }
                        span { class: "kit-tcn__name", {node.name.clone()} }
                        span { class: "kit-tcn__role", {node.role} }
                        if node.attention {
                            span { class: "kit-tcn__badge", "aria-hidden": "true", {node.condition_word} }
                        }
                        if let Some(summary) = node.summary.clone() {
                            span { class: "kit-tcn__summary", {summary} }
                        }
                    }
                }
            }
            if let Some(entity) = selected_entity {
                div {
                    id: "{card_id}",
                    class: "kit-tcn__card",
                    role: "dialog",
                    "aria-label": "{card_name}",
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape {
                            selected.set(None);
                            if let Some(handler) = &on_focus {
                                handler.call(None);
                            }
                            focus_after_frame(&format!("{id}-node-{}", entity_for_close.as_ref().map(|e| e.id.clone()).unwrap_or_default()));
                        }
                    },
                    header { class: "kit-tcn__card-head",
                        span { class: "kit-tcn__card-name", {card_name.clone()} }
                        span { class: "kit-tcn__card-role", {entity.node_role.word()} }
                        span { class: "kit-tcn__card-condition", "data-condition": entity.condition.as_str(), {card_condition} }
                        button {
                            id: "{close_id}",
                            class: "kit-tcn__close",
                            r#type: "button",
                            "aria-label": "Close",
                            onmounted: move |event: MountedEvent| {
                                let node = event.data();
                                spawn(async move {
                                    let _ = node.set_focus(true).await;
                                });
                            },
                            onclick: move |_| {
                                selected.set(None);
                                if let Some(handler) = &on_focus {
                                    handler.call(None);
                                }
                            },
                            "×"
                        }
                    }
                    if let Some(activity) = card_activity {
                        p { class: "kit-tcn__activity", {activity} }
                    }
                    ul { class: "kit-tcn__recent",
                        for (index, (exchange_key, target, latency)) in card_recent.into_iter().enumerate() {
                            li { key: "{exchange_key}-{index}",
                                span { {target} }
                                span { class: "kit-tcn__latency", {latency} }
                            }
                        }
                    }
                    if !card_spend.is_empty() {
                        p { class: "kit-tcn__spend-line", {card_spend} }
                    }
                }
            }
            ul { class: "kit-tcn__sr", "aria-label": "Graph as text",
                for (index, line) in text_lines.into_iter().enumerate() {
                    li { key: "{index}", {line} }
                }
            }
        }
    }
}
