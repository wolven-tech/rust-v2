//! Exchange diagram: the message traffic of one agent run (person, agents,
//! tools) in time order. Each actor is a column; each exchange is a row with an
//! arrow from sender to receiver. A reader can trace which request an answer
//! closes, spot failures, and open any row for its full text.
//!
//! Design choice: rows are a list, so each row is its own grid with the same
//! column template as the actor header. Lifelines are per-row cells that
//! join across rows because rows have a fixed height. Each arrow is absolutely
//! placed from its sender's lifeline centre to its receiver's, with no box.
//! Design choice: a work bar is drawn on the receiver's column within each
//! request row, not as one bar spanning to its answer. A spanning bar needs one
//! grid across all rows, which the list semantics rule out.
//! Design choice: cards open in flow under their anchor, not as floating
//! popovers, so the scroll container never clips them.
//! Design choice: the default clock is `m:ss` with hours past one hour, and a
//! negative time reads as zero. A host formatter replaces it through
//! `format_clock`.
//! Design choice: participant tints are assigned by role and order: person,
//! tool, the first agent as primary, and further agents cycle five slots.
//! Not implemented: the frame around a group draws per row (a single box across
//! rows would need the same spanning grid), and the traveling light and
//! ripple on a new exchange (CSS animation only).
//! Browser checks: Up, Down, Home and End move focus between rows; Enter opens a
//! card as a dialog; Escape closes it and returns focus to the row; at phone
//! width the diagram scrolls inside its container and the page does not.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, focus_after_frame, next_id};

const CSS: Asset = asset!("/assets/exchange-diagram.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorRole {
    Person,
    Agent,
    Tool,
}

impl ActorRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Agent => "agent",
            Self::Tool => "tool",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExchangeKind {
    Request,
    Answer,
}

impl ExchangeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Request => "request",
            Self::Answer => "answer",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExchangeState {
    Ok,
    Failed,
    AwaitingPerson,
}

impl ExchangeState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
            Self::AwaitingPerson => "awaiting-person",
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::Ok => "Complete",
            Self::Failed => "Failed",
            Self::AwaitingPerson => "Needs your reply",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Actor {
    pub actor_key: String,
    pub role: ActorRole,
    pub caption: String,
    pub mark_source: Option<String>,
    pub tint_key: Option<String>,
    pub model_tag: Option<String>,
    pub summary: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Exchange {
    pub exchange_key: String,
    pub from_key: String,
    pub to_key: String,
    pub kind: ExchangeKind,
    pub answers_key: Option<String>,
    pub state: ExchangeState,
    pub sent_at_ms: Option<i64>,
    pub group_tag: Option<String>,
    pub headline: String,
    pub detail_text: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavKey {
    Down,
    Up,
    Home,
    End,
}

/// `m:ss`, with hours in front past one hour. A negative time reads as zero.
pub fn clock_text(ms: i64) -> String {
    let total = ms.max(0) / 1000;
    let (h, m, s) = (total / 3600, total / 60 % 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// Column indices of an exchange's sender and receiver, or `None` when either
/// names an actor that is not in the diagram.
pub fn resolve_columns(exchange: &Exchange, actors: &[Actor]) -> Option<(usize, usize)> {
    let from = actors
        .iter()
        .position(|a| a.actor_key == exchange.from_key)?;
    let to = actors.iter().position(|a| a.actor_key == exchange.to_key)?;
    Some((from, to))
}

/// The tint slot of each actor, in order.
pub fn tint_for(actors: &[Actor]) -> Vec<String> {
    let mut agents_seen = 0usize;
    actors
        .iter()
        .map(|actor| {
            if let Some(tint) = actor.tint_key.as_ref().filter(|t| !t.is_empty()) {
                return tint.clone();
            }
            match actor.role {
                ActorRole::Person => "person".to_string(),
                ActorRole::Tool => "tool".to_string(),
                ActorRole::Agent => {
                    let slot = agents_seen;
                    agents_seen += 1;
                    if slot == 0 {
                        "primary".to_string()
                    } else {
                        format!("slot-{}", (slot - 1) % 5 + 1)
                    }
                }
            }
        })
        .collect()
}

/// Request keys that no answer closes.
pub fn open_request_keys(exchanges: &[Exchange]) -> Vec<String> {
    exchanges
        .iter()
        .filter(|e| e.kind == ExchangeKind::Request)
        .filter(|req| {
            !exchanges.iter().any(|e| {
                e.kind == ExchangeKind::Answer
                    && e.answers_key.as_deref() == Some(req.exchange_key.as_str())
            })
        })
        .map(|e| e.exchange_key.clone())
        .collect()
}

/// For each row, whether a labelled group starts there: the first row of a
/// run that carries a tag different from its predecessor.
pub fn group_label_starts(tags: &[Option<String>]) -> Vec<bool> {
    let mut previous: Option<&str> = None;
    tags.iter()
        .map(|tag| {
            let current = tag.as_deref().filter(|t| !t.is_empty());
            let starts = current.is_some() && current != previous;
            previous = current;
            starts
        })
        .collect()
}

/// Earliest and latest sent time, when any exchange carries one.
pub fn run_span_ms(exchanges: &[Exchange]) -> Option<(i64, i64)> {
    let times: Vec<i64> = exchanges.iter().filter_map(|e| e.sent_at_ms).collect();
    let min = times.iter().copied().min()?;
    let max = times.iter().copied().max()?;
    Some((min, max))
}

/// The row to move to for a navigation key. From nowhere, Down and Home go to
/// the first row, Up and End to the last.
pub fn next_index(current: Option<usize>, len: usize, key: NavKey) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let last = len - 1;
    Some(match (current, key) {
        (_, NavKey::Home) => 0,
        (_, NavKey::End) => last,
        (None, NavKey::Down) => 0,
        (None, NavKey::Up) => last,
        (Some(i), NavKey::Down) => (i + 1).min(last),
        (Some(i), NavKey::Up) => i.saturating_sub(1),
    })
}

/// The accessible name of a row: sender, receiver, headline, state and time.
pub fn exchange_name(
    from: &str,
    to: &str,
    headline: &str,
    state: ExchangeState,
    time: Option<&str>,
) -> String {
    let mut name = format!("{from} to {to}, {headline}, {}", state.word());
    if let Some(time) = time {
        name.push_str(&format!(", {time}"));
    }
    name.push_str(". Opens a card.");
    name
}

/// The polite line for a new exchange.
pub fn announce_exchange(from: &str, to: &str, headline: &str) -> String {
    format!("{from} to {to}: {headline}")
}

fn row_dom_id(scope: &str, key: &str) -> String {
    format!("{scope}-row-{key}")
}

fn chip_dom_id(scope: &str, key: &str) -> String {
    format!("{scope}-chip-{key}")
}

fn focus_element(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

#[derive(Clone, Debug, PartialEq)]
enum Pick {
    Actor(String),
    Exchange(String),
}

struct RowView {
    index: usize,
    key: String,
    dom_id: String,
    exchange: Exchange,
    name: String,
    from_name: String,
    to_name: String,
    time: Option<String>,
    group_label: Option<String>,
    span_style: String,
    tint: String,
    bar_style: Option<String>,
    bar_state: &'static str,
    to_col: usize,
    cross: bool,
    answer: bool,
    direction: &'static str,
    self_loop: bool,
    kind_key: &'static str,
    state_key: &'static str,
    state_word: &'static str,
}

fn caption_of(actors: &[Actor], key: &str) -> String {
    actors
        .iter()
        .find(|a| a.actor_key == key)
        .map_or_else(|| key.to_string(), |a| a.caption.clone())
}

/// A request/answer diagram of one agent run, with a card for every actor and
/// exchange.
#[component]
pub fn ExchangeDiagram(
    actors: Vec<Actor>,
    exchanges: Vec<Exchange>,
    #[props(default = "Run".to_string(), into)] run_heading: String,
    #[props(default = true)] run_is_live: bool,
    #[props(default)] show_time_gutter: Option<bool>,
    #[props(default)] format_clock: Option<Callback<i64, String>>,
    #[props(default)] on_pick: Option<EventHandler<Option<Exchange>>>,
    #[props(default)] extra_class: String,
) -> Element {
    let mut pick = use_signal(|| None::<Pick>);
    let mut focused = use_signal(|| None::<usize>);
    let scope = use_hook(|| format!("kit-agsq-{}", next_id()));
    let known = use_hook(|| {
        exchanges
            .iter()
            .map(|e| e.exchange_key.clone())
            .collect::<Vec<String>>()
    });

    let format_time = |ms: i64| match format_clock {
        Some(callback) => callback.call(ms),
        None => clock_text(ms),
    };
    let gutter_on =
        show_time_gutter.unwrap_or_else(|| exchanges.iter().any(|e| e.sent_at_ms.is_some()));
    let gutter = if gutter_on { "3.5rem" } else { "0px" };
    let actor_count = actors.len();
    let tints = tint_for(&actors);
    let open_requests = open_request_keys(&exchanges);

    let drawn: Vec<(&Exchange, usize, usize)> = exchanges
        .iter()
        .filter_map(|e| resolve_columns(e, &actors).map(|(from, to)| (e, from, to)))
        .collect();
    let tags: Vec<Option<String>> = drawn.iter().map(|(e, _, _)| e.group_tag.clone()).collect();
    let starts = group_label_starts(&tags);
    let rows: Vec<RowView> = drawn
        .iter()
        .enumerate()
        .map(|(index, &(exchange, from, to))| {
            let from_name = caption_of(&actors, &exchange.from_key);
            let to_name = caption_of(&actors, &exchange.to_key);
            let time = exchange.sent_at_ms.map(&format_time);
            let name = exchange_name(
                &from_name,
                &to_name,
                &exchange.headline,
                exchange.state,
                time.as_deref(),
            );
            let (lo, hi) = (from.min(to), from.max(to));
            let width = actor_count as f64;
            let span_style = format!(
                "--kit-agsq-from: {:.4}; --kit-agsq-to: {:.4}",
                (lo as f64 + 0.5) / width,
                (hi as f64 + 0.5) / width
            );
            let is_open_request = open_requests.contains(&exchange.exchange_key);
            let bar_state = match (exchange.kind, is_open_request, run_is_live, exchange.state) {
                (ExchangeKind::Request, true, true, ExchangeState::Ok) => "working",
                (ExchangeKind::Request, true, _, _) => "stopped",
                (ExchangeKind::Request, false, _, _) => "closed",
                (ExchangeKind::Answer, _, _, _) => "",
            };
            let is_request = exchange.kind == ExchangeKind::Request;
            RowView {
                index,
                key: exchange.exchange_key.clone(),
                dom_id: row_dom_id(&scope, &exchange.exchange_key),
                exchange: exchange.clone(),
                name,
                from_name,
                to_name,
                time,
                group_label: if starts[index] {
                    exchange.group_tag.clone()
                } else {
                    None
                },
                span_style,
                tint: tints[from].clone(),
                bar_style: is_request.then(|| format!("grid-column: {}", to + 2)),
                bar_state,
                to_col: to + 2,
                cross: exchange.state == ExchangeState::Failed,
                answer: !is_request,
                direction: if from <= to { "forward" } else { "backward" },
                self_loop: from == to,
                kind_key: exchange.kind.as_str(),
                state_key: exchange.state.as_str(),
                state_word: exchange.state.word(),
            }
        })
        .collect();

    let nav_keys: Vec<String> = rows.iter().map(|r| r.key.clone()).collect();
    let fresh_live = rows
        .iter()
        .rev()
        .find(|r| !known.contains(&r.key))
        .map(|r| announce_exchange(&r.from_name, &r.to_name, &r.exchange.headline))
        .unwrap_or_default();
    let span_text = run_span_ms(&exchanges)
        .map(|(lo, hi)| format!("{} span", format_time(hi - lo)))
        .unwrap_or_default();
    let status = if run_is_live {
        let working = rows.iter().filter(|r| r.bar_state == "working").count();
        format!("{working} working")
    } else {
        "Run over".to_string()
    };
    let meta = format!(
        "{status} · {} messages{}",
        rows.len(),
        if span_text.is_empty() {
            String::new()
        } else {
            format!(" · {span_text}")
        }
    );
    let class = class_list("kit-agsq", &extra_class);
    let cols_style = format!("--kit-agsq-cols: {actor_count}; --kit-agsq-gutter: {gutter}");
    let life_cols: Vec<usize> = (2..actor_count + 2).collect();
    let open_actor = match pick() {
        Some(Pick::Actor(key)) => actors.iter().find(|a| a.actor_key == key).cloned(),
        _ => None,
    };
    let open_exchange = match pick() {
        Some(Pick::Exchange(key)) => Some(key),
        _ => None,
    };
    let chips: Vec<(usize, Actor, String, String, bool)> = actors
        .iter()
        .enumerate()
        .map(|(i, actor)| {
            let expanded = open_actor
                .as_ref()
                .is_some_and(|o| o.actor_key == actor.actor_key);
            (
                i + 2,
                actor.clone(),
                chip_dom_id(&scope, &actor.actor_key),
                tints[i].clone(),
                expanded,
            )
        })
        .collect();
    let on_pick_handler = on_pick;
    let actor_anchor = open_actor
        .as_ref()
        .map(|a| chip_dom_id(&scope, &a.actor_key))
        .unwrap_or_default();
    let actor_caption = open_actor
        .as_ref()
        .map(|a| a.caption.clone())
        .unwrap_or_default();
    let actor_model = open_actor.as_ref().and_then(|a| a.model_tag.clone());
    let actor_summary = open_actor.as_ref().and_then(|a| a.summary.clone());

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class, "aria-label": "{run_heading}", style: "{cols_style}",
            header { class: "kit-agsq__head",
                h2 { class: "kit-agsq__title", {run_heading.clone()} }
                p { class: "kit-agsq__meta", {meta} }
            }
            div { class: "kit-agsq__sr", role: "status", "aria-live": "polite", {fresh_live} }
            div { class: "kit-agsq__scroll",
                div { class: "kit-agsq__chips",
                    span { class: "kit-agsq__gutter-head", "aria-hidden": "true" }
                    for (col, actor, chip_id, tint, expanded) in chips {
                        button {
                            id: chip_id.clone(),
                            r#type: "button",
                            class: "kit-agsq__chip",
                            "data-role": actor.role.as_str(),
                            "data-tint": tint,
                            style: format!("grid-column: {col}"),
                            "aria-haspopup": "dialog",
                            "aria-expanded": expanded.to_string(),
                            onclick: {
                                let key = actor.actor_key.clone();
                                move |_| {
                                    let was_open = pick() == Some(Pick::Actor(key.clone()));
                                    if let (Some(Pick::Exchange(_)), Some(handler)) = (pick(), on_pick_handler) {
                                        handler.call(None);
                                    }
                                    pick.set(if was_open { None } else { Some(Pick::Actor(key.clone())) });
                                }
                            },
                            span { class: "kit-agsq__mark", "aria-hidden": "true",
                                {actor.caption.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default()}
                            }
                            span { class: "kit-agsq__chip-name", {actor.caption.clone()} }
                        }
                    }
                }
                if open_actor.is_some() {
                    div { class: "kit-agsq__card kit-agsq__card--actor", role: "dialog",
                        "aria-label": actor_caption.clone(),
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
                                pick.set(None);
                                focus_after_frame(&actor_anchor);
                            }
                        },
                        p { class: "kit-agsq__card-title", {actor_caption.clone()} }
                        if let Some(tag) = actor_model {
                            p { class: "kit-agsq__card-model", {tag} }
                        }
                        if let Some(summary) = actor_summary {
                            p { class: "kit-agsq__card-summary", {summary} }
                        }
                    }
                }
                if rows.is_empty() {
                    p { class: "kit-agsq__empty", "No exchanges yet." }
                }
                ol {
                    class: "kit-agsq__list",
                    onkeydown: move |event: KeyboardEvent| {
                        let key = match event.key() {
                            Key::ArrowDown => NavKey::Down,
                            Key::ArrowUp => NavKey::Up,
                            Key::Home => NavKey::Home,
                            Key::End => NavKey::End,
                            _ => return,
                        };
                        event.prevent_default();
                        if let Some(k) = next_index(focused(), nav_keys.len(), key).and_then(|target| nav_keys.get(target)) {
                            focus_element(&row_dom_id(&scope, k));
                        }
                    },
                    for row in rows {
                        li {
                            class: "kit-agsq__item",
                            "data-kind": row.kind_key,
                            "data-state": row.state_key,
                            "data-direction": row.direction,
                            "data-self": row.self_loop.to_string(),
                            "data-tint": row.tint.clone(),
                            "data-group": row.exchange.group_tag.clone(),
                            if let Some(label) = row.group_label.clone() {
                                span { class: "kit-agsq__group", {label} }
                            }
                            span { class: "kit-agsq__time", {row.time.clone().unwrap_or_default()} }
                            for col in life_cols.iter().copied() {
                                span { class: "kit-agsq__life", "aria-hidden": "true", style: format!("grid-column: {col}") }
                            }
                            if let Some(style) = row.bar_style.clone() {
                                span { class: "kit-agsq__bar", "data-state": row.bar_state, "aria-hidden": "true", style }
                            }
                            if row.cross {
                                span { class: "kit-agsq__cross", "aria-hidden": "true", style: format!("grid-column: {}", row.to_col) }
                            }
                            button {
                                id: row.dom_id.clone(),
                                r#type: "button",
                                class: "kit-agsq__row",
                                style: row.span_style.clone(),
                                "data-answer": row.answer.to_string(),
                                "aria-haspopup": "dialog",
                                "aria-expanded": (open_exchange.as_deref() == Some(row.key.as_str())).to_string(),
                                "aria-label": row.name.clone(),
                                onfocus: move |_| focused.set(Some(row.index)),
                                onclick: {
                                    let key = row.key.clone();
                                    let exchange = row.exchange.clone();
                                    move |_| {
                                        let opening = pick() != Some(Pick::Exchange(key.clone()));
                                        pick.set(if opening { Some(Pick::Exchange(key.clone())) } else { None });
                                        if let Some(handler) = on_pick_handler {
                                            handler.call(if opening { Some(exchange.clone()) } else { None });
                                        }
                                    }
                                },
                                span { class: "kit-agsq__line", "aria-hidden": "true" }
                                span { class: "kit-agsq__tag", {row.exchange.headline.clone()} }
                                span { class: "kit-agsq__state-word", {row.state_word} }
                            }
                            if open_exchange.as_deref() == Some(row.key.as_str()) {
                                div { class: "kit-agsq__card", role: "dialog", "aria-label": row.name.clone(),
                                    style: "grid-column: 1 / -1",
                                    tabindex: "-1",
                                    onmounted: move |event: MountedEvent| {
                                        let node = event.data();
                                        spawn(async move {
                                            let _ = node.set_focus(true).await;
                                        });
                                    },
                                    onkeydown: {
                                        let dom_id = row.dom_id.clone();
                                        move |event: KeyboardEvent| {
                                            if event.key() == Key::Escape {
                                                event.prevent_default();
                                                pick.set(None);
                                                if let Some(handler) = on_pick_handler {
                                                    handler.call(None);
                                                }
                                                focus_after_frame(&dom_id);
                                            }
                                        }
                                    },
                                    p { class: "kit-agsq__card-route", {format!("{} to {}", row.from_name, row.to_name)} }
                                    p { class: "kit-agsq__card-headline", {row.exchange.headline.clone()} }
                                    p { class: "kit-agsq__card-state", {row.state_word} }
                                    if let Some(time) = row.time.clone() {
                                        p { class: "kit-agsq__card-time", {time} }
                                    }
                                    if let Some(detail) = row.exchange.detail_text.clone() {
                                        p { class: "kit-agsq__card-detail", {detail} }
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
