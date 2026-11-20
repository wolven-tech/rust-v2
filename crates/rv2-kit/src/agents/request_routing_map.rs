//! Request routing map: incoming requests, the router that reads each one and
//! picks who handles it, and every handler's score and reason. A person can open
//! any request to see why it went where it went, and reassign it.
//!
//! Design choice: one markup serves both layouts. Stacked and side-by-side
//! differ only in CSS placement, so a container query can switch layouts
//! without changing which elements exist.
//! Design choice: the map has one roving tab stop. The stop follows the arrow
//! walk: top requests, the router, each handler, then that handler's queue.
//! Design choice: a request leaves the top list when it is assigned and joins
//! the queue of its handler. A person handler has no card; its waiting count
//! is a tag on the router.
//! Design choice: the reassign list offers every handler, the person included.
//! Design choice: connectors are curves measured from the rendered boxes, so
//! they follow the cards through resizes and re-renders. Side-by-side only;
//! stacked has no connectors.
//! Not implemented: the traveling light along each link (CSS-only motion), and
//! the icon element for each channel (the channel name is the tile's text).
//! Browser checks: the marker travels and the certainty bar fills as the router
//! decides; Arrow, Home and End walk the map; Enter on a handler isolates its
//! requests; Escape closes the dialog and clears the isolation; the dialog's
//! reassign list moves the marker to the new handler.

use std::collections::BTreeMap;

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, focus_after_frame};

const CSS: Asset = asset!("/assets/request-routing-map.css");

/// Tags shown on a request card; further tags are dropped.
const TAG_LIMIT: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    Mail,
    Chat,
    Ticket,
    Phone,
    TeamChat,
}

impl Channel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mail => "mail",
            Self::Chat => "chat",
            Self::Ticket => "ticket",
            Self::Phone => "phone",
            Self::TeamChat => "team-chat",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Mail => "Mail",
            Self::Chat => "Chat",
            Self::Ticket => "Ticket",
            Self::Phone => "Phone",
            Self::TeamChat => "Team chat",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestState {
    Incoming,
    Sorting,
    Assigned,
    NeedsPerson,
    Closed,
}

impl RequestState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Incoming => "incoming",
            Self::Sorting => "sorting",
            Self::Assigned => "assigned",
            Self::NeedsPerson => "needs-person",
            Self::Closed => "closed",
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::Incoming => "Incoming",
            Self::Sorting => "Sorting",
            Self::Assigned => "Assigned",
            Self::NeedsPerson => "Waiting for you",
            Self::Closed => "Closed",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayoutMode {
    #[default]
    Stacked,
    SideBySide,
}

impl LayoutMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stacked => "stacked",
            Self::SideBySide => "side-by-side",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RouterNode {
    pub node_key: String,
    pub node_label: String,
    pub model_label: Option<String>,
    pub reading_text: Option<String>,
    pub model_mark: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HandlerNode {
    pub handler_key: String,
    pub handler_label: String,
    pub model_label: Option<String>,
    pub handler_state: Option<String>,
    pub activity_text: Option<String>,
    pub load_count: Option<u32>,
    pub queued_request_keys: Vec<String>,
    pub is_person: bool,
    /// 1 to 6; out-of-range values are clamped.
    pub accent_slot: u8,
    pub handler_mark: Option<String>,
    pub model_mark: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RequestTag {
    pub label: String,
    pub accent_slot: Option<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RequestRow {
    pub request_key: String,
    pub request_title: String,
    pub sender_label: Option<String>,
    pub channel_kind: Channel,
    pub preview_text: Option<String>,
    pub request_state: RequestState,
    pub assigned_to_key: Option<String>,
    /// 0 to 1; out-of-range values are clamped.
    pub certainty_fraction: f64,
    pub score_map: BTreeMap<String, f64>,
    pub reason_text: Option<String>,
    pub tag_list: Vec<RequestTag>,
}

/// A stop in the keyboard walk over the map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavTarget {
    Request(String),
    Router,
    Handler(String),
}

/// Certainty as a whole percent, clamped to 0..=100. NaN reads as 0.
pub fn percent_sure(certainty: f64) -> u32 {
    if certainty.is_nan() {
        0
    } else {
        (certainty.clamp(0.0, 1.0) * 100.0).round() as u32
    }
}

pub fn accent_class(slot: u8) -> String {
    format!("slot-{}", slot.clamp(1, 6))
}

/// The sentence a request card reads as: title, sender, state, handler, certainty.
pub fn sentence(row: &RequestRow, handler_label: Option<&str>) -> String {
    let mut out = row.request_title.clone();
    if let Some(sender) = row.sender_label.as_deref().filter(|s| !s.is_empty()) {
        out.push_str(&format!(", from {sender}"));
    }
    out.push_str(&format!(": {}", row.request_state.word()));
    if let Some(handler) = handler_label {
        out.push_str(&format!(", to {handler}"));
    }
    out.push_str(&format!(", {}% sure", percent_sure(row.certainty_fraction)));
    out
}

/// The states a request passes through, in order, for the route it takes.
pub fn flow_for(needs_person: bool) -> Vec<RequestState> {
    let last = if needs_person {
        RequestState::NeedsPerson
    } else {
        RequestState::Assigned
    };
    vec![RequestState::Incoming, RequestState::Sorting, last]
}

/// How many cards a list shows, and how many it hides.
pub fn visible_split(count: usize, max: usize) -> (usize, usize) {
    let shown = count.min(max);
    (shown, count - shown)
}

/// The request cards on the top list: every request not yet assigned to a handler.
pub fn top_requests(requests: &[RequestRow]) -> Vec<&RequestRow> {
    requests
        .iter()
        .filter(|r| r.request_state != RequestState::Assigned)
        .collect()
}

/// The requests in a handler's queue.
pub fn queue_for<'a>(handler_key: &str, requests: &'a [RequestRow]) -> Vec<&'a RequestRow> {
    requests
        .iter()
        .filter(|r| {
            r.request_state == RequestState::Assigned
                && r.assigned_to_key.as_deref() == Some(handler_key)
        })
        .collect()
}

/// Requests waiting on the person.
pub fn waiting_on_person(requests: &[RequestRow]) -> usize {
    requests
        .iter()
        .filter(|r| r.request_state == RequestState::NeedsPerson)
        .count()
}

/// Counts for the header: sorting, waiting on the person, assigned.
pub fn header_counts(requests: &[RequestRow]) -> (usize, usize, usize) {
    let count = |state: RequestState| requests.iter().filter(|r| r.request_state == state).count();
    (
        count(RequestState::Sorting),
        count(RequestState::NeedsPerson),
        count(RequestState::Assigned),
    )
}

/// The keyboard walk: the visible top requests, the router, then each card handler
/// followed by its queue.
pub fn nav_order(
    top: &[&RequestRow],
    requests: &[RequestRow],
    handlers: &[HandlerNode],
) -> Vec<NavTarget> {
    let mut order: Vec<NavTarget> = top
        .iter()
        .map(|r| NavTarget::Request(r.request_key.clone()))
        .collect();
    order.push(NavTarget::Router);
    for handler in handlers.iter().filter(|h| !h.is_person) {
        order.push(NavTarget::Handler(handler.handler_key.clone()));
        for row in queue_for(&handler.handler_key, requests) {
            order.push(NavTarget::Request(row.request_key.clone()));
        }
    }
    order
}

/// The walk step for a key: Down and Right forward, Up and Left back, Home and End to the ends.
pub fn nav_step(current: Option<usize>, len: usize, key: NavKey) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let last = len - 1;
    Some(match (current, key) {
        (_, NavKey::Home) => 0,
        (_, NavKey::End) => last,
        (None, NavKey::Forward) => 0,
        (None, NavKey::Back) => last,
        (Some(i), NavKey::Forward) => (i + 1).min(last),
        (Some(i), NavKey::Back) => i.saturating_sub(1),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavKey {
    Forward,
    Back,
    Home,
    End,
}

/// Whether an element is dimmed while a handler is isolated. `owner` is the
/// handler a request is queued under; top requests have none.
pub fn is_dimmed(focus: Option<&str>, owner: Option<&str>) -> bool {
    match focus {
        Some(focused) => owner != Some(focused),
        None => false,
    }
}

/// Each handler's score for a request, in handler order.
pub fn score_rows(row: &RequestRow, handlers: &[HandlerNode]) -> Vec<(String, u32)> {
    handlers
        .iter()
        .map(|h| {
            let score = row.score_map.get(&h.handler_key).copied().unwrap_or(0.0);
            (h.handler_label.clone(), percent_sure(score))
        })
        .collect()
}

/// The polite line for a state change.
pub fn announcement(row: &RequestRow, handler_label: Option<&str>) -> String {
    match (row.request_state, handler_label) {
        (RequestState::NeedsPerson, _) => format!("{} needs you.", row.request_title),
        (RequestState::Assigned, Some(handler)) => {
            format!("{} assigned to {handler}.", row.request_title)
        }
        (RequestState::Assigned, None) => format!("{} assigned.", row.request_title),
        (RequestState::Sorting, _) => format!("{} is being sorted.", row.request_title),
        (RequestState::Closed, _) => format!("{} closed.", row.request_title),
        (RequestState::Incoming, _) => format!("{} arrived.", row.request_title),
    }
}

/// Requests whose state differs from the one seen at mount.
pub fn changed_requests<'a>(
    known: &[(String, RequestState)],
    requests: &'a [RequestRow],
) -> Vec<&'a RequestRow> {
    requests
        .iter()
        .filter(|r| {
            known
                .iter()
                .find(|(key, _)| *key == r.request_key)
                .is_none_or(|(_, state)| *state != r.request_state)
        })
        .collect()
}

fn handler_name(handlers: &[HandlerNode], key: Option<&str>) -> Option<String> {
    let key = key?;
    handlers
        .iter()
        .find(|h| h.handler_key == key)
        .map(|h| h.handler_label.clone())
}

fn dom_id(kind: &str, key: &str) -> String {
    format!("kit-rrm-{kind}-{key}")
}

fn focus_element(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

const ROUTER_DOM_ID: &str = "kit-rrm-router";
const LINKS_DOM_ID: &str = "kit-rrm-links";

/// Sets each `path` in the links SVG to a curve from the right edge of its
/// `data-from` box to the left edge of its `data-to` box, in map coordinates.
/// Re-runs on resize and on any change to the map's subtree that can move a box.
/// The path's own `d` is not observed, so the script never re-triggers itself.
const LINK_SCRIPT: &str = r#"
let svg = null;
for (let i = 0; i < 600 && !(svg = document.getElementById("__ID__")); i++)
    await new Promise((resolve) => requestAnimationFrame(resolve));
if (!svg) return false;
const root = svg.parentElement;
const layout = () => {
    if (svg.getClientRects().length === 0) return;
    const box = svg.getBoundingClientRect();
    for (const path of svg.querySelectorAll("path")) {
        const from = document.getElementById(path.dataset.from);
        const to = document.getElementById(path.dataset.to);
        if (!from || !to) {
            path.removeAttribute("d");
            continue;
        }
        const a = from.getBoundingClientRect();
        const b = to.getBoundingClientRect();
        const x1 = a.right - box.left;
        const y1 = a.top + a.height / 2 - box.top;
        const x2 = b.left - box.left;
        const y2 = b.top + b.height / 2 - box.top;
        const reach = Math.max(24, (x2 - x1) / 2);
        path.setAttribute("d", `M${x1} ${y1} C${x1 + reach} ${y1} ${x2 - reach} ${y2} ${x2} ${y2}`);
    }
};
let frame = 0;
const schedule = () => {
    if (!frame) frame = requestAnimationFrame(() => { frame = 0; layout(); });
};
const observers = (window.__kitRrmLinks ||= new Map());
observers.get("__ID__")?.disconnect();
const mutations = new MutationObserver(schedule);
mutations.observe(root, {
    subtree: true,
    childList: true,
    attributes: true,
    attributeFilter: ["class", "style", "data-state", "data-dim", "data-from", "data-to", "data-layout"],
});
const resizes = new ResizeObserver(schedule);
resizes.observe(root);
observers.set("__ID__", { disconnect() { mutations.disconnect(); resizes.disconnect(); } });
layout();
return true;
"#;

fn link_script() -> String {
    LINK_SCRIPT.replace("__ID__", LINKS_DOM_ID)
}

struct Ctx<'a> {
    handlers: &'a [HandlerNode],
    nav_order: &'a [NavTarget],
    current: Option<&'a NavTarget>,
    focus: Option<String>,
    focus_sig: Signal<Option<String>>,
    open: Signal<Option<String>>,
    nav: Signal<Option<usize>>,
    on_open: Option<EventHandler<RequestRow>>,
}

fn tab_stop(ctx: &Ctx<'_>, target: &NavTarget) -> &'static str {
    if ctx.current == Some(target) {
        "0"
    } else {
        "-1"
    }
}

fn nav_position(ctx: &Ctx<'_>, target: &NavTarget) -> Option<usize> {
    ctx.nav_order.iter().position(|t| t == target)
}

fn request_card(ctx: &Ctx<'_>, row: &RequestRow, owner: Option<&str>) -> Element {
    let handler = handler_name(ctx.handlers, row.assigned_to_key.as_deref());
    let line = sentence(row, handler.as_deref());
    let target = NavTarget::Request(row.request_key.clone());
    let tab = tab_stop(ctx, &target);
    let position = nav_position(ctx, &target);
    let dimmed = is_dimmed(ctx.focus.as_deref(), owner).to_string();
    let percent = percent_sure(row.certainty_fraction);
    let state = row.request_state.as_str();
    let channel = row.channel_kind.as_str();
    let channel_label = row.channel_kind.label();
    let sender = row.sender_label.clone().unwrap_or_default();
    let preview = row.preview_text.clone().unwrap_or_default();
    let tags: Vec<RequestTag> = row.tag_list.iter().take(TAG_LIMIT).cloned().collect();
    let id = dom_id("request", &row.request_key);
    let mut open = ctx.open;
    let mut nav = ctx.nav;
    let on_open = ctx.on_open;
    let open_key = row.request_key.clone();
    let open_row = row.clone();
    rsx! {
        button {
            id: id.clone(),
            r#type: "button",
            class: "kit-rrm__request",
            "data-state": state,
            "data-channel": channel,
            "data-dim": dimmed,
            tabindex: tab,
            "aria-label": line,
            "aria-haspopup": "dialog",
            onfocus: move |_| nav.set(position),
            onclick: move |_| {
                open.set(Some(open_key.clone()));
                if let Some(handler) = on_open {
                    handler.call(open_row.clone());
                }
            },
            span { class: "kit-rrm__tile", "aria-hidden": "true", {channel_label} }
            span { class: "kit-rrm__body",
                span { class: "kit-rrm__title", {row.request_title.clone()} }
                if !sender.is_empty() {
                    span { class: "kit-rrm__sender", {sender} }
                }
                span { class: "kit-rrm__state", {row.request_state.word()} }
                if !preview.is_empty() {
                    span { class: "kit-rrm__preview", {preview} }
                }
                span { class: "kit-rrm__certainty", role: "meter", "aria-valuemin": "0",
                    "aria-valuemax": "100", "aria-valuenow": percent.to_string(), "aria-hidden": "true",
                    span { class: "kit-rrm__fill", style: format!("width: {percent}%") }
                }
                span { class: "kit-rrm__percent", {format!("{percent}% sure")} }
                if !tags.is_empty() {
                    span { class: "kit-rrm__tags",
                        for tag in tags {
                            span { class: "kit-rrm__tag", "data-accent": tag.accent_slot.map(accent_class).unwrap_or_default(), {tag.label} }
                        }
                    }
                }
            }
        }
    }
}

fn handler_card(ctx: &Ctx<'_>, handler: &HandlerNode, requests: &[RequestRow]) -> Element {
    let target = NavTarget::Handler(handler.handler_key.clone());
    let tab = tab_stop(ctx, &target);
    let position = nav_position(ctx, &target);
    let pressed = ctx.focus.as_deref() == Some(handler.handler_key.as_str());
    let dimmed = is_dimmed(ctx.focus.as_deref(), Some(handler.handler_key.as_str())).to_string();
    let accent = accent_class(handler.accent_slot);
    let load = handler
        .load_count
        .map(|n| format!("{n} in progress"))
        .unwrap_or_default();
    let activity = handler.activity_text.clone().unwrap_or_default();
    let model = handler.model_label.clone().unwrap_or_default();
    let state = handler.handler_state.clone().unwrap_or_default();
    let queue: Vec<RequestRow> = queue_for(&handler.handler_key, requests)
        .into_iter()
        .cloned()
        .collect();
    let key = handler.handler_key.clone();
    let mut nav = ctx.nav;
    let mut focus_sig = ctx.focus_sig;
    let focus_key = handler.handler_key.clone();
    rsx! {
        section { class: "kit-rrm__handler", "data-accent": accent, "data-dim": dimmed,
            "aria-label": handler.handler_label.clone(),
            button {
                id: dom_id("handler", &key),
                r#type: "button",
                class: "kit-rrm__handler-head",
                tabindex: tab,
                "aria-pressed": pressed.to_string(),
                onfocus: move |_| nav.set(position),
                onclick: move |_| {
                    let next = if pressed { None } else { Some(focus_key.clone()) };
                    focus_sig.set(next);
                },
                span { class: "kit-rrm__handler-name", {handler.handler_label.clone()} }
                if !model.is_empty() {
                    span { class: "kit-rrm__handler-model", {model} }
                }
                if !state.is_empty() {
                    span { class: "kit-rrm__handler-state", {state} }
                }
                if !activity.is_empty() {
                    span { class: "kit-rrm__handler-activity", {activity} }
                }
                if !load.is_empty() {
                    span { class: "kit-rrm__handler-load", {load} }
                }
            }
            ul { class: "kit-rrm__queue", "aria-label": "Queue",
                for row in queue {
                    li {
                        {request_card(ctx, &row, Some(key.as_str()))}
                    }
                }
            }
        }
    }
}

/// A map of incoming requests, the router that reads each one, and the handlers
/// that take them, with every score and reason one open away.
#[component]
pub fn RequestRoutingMap(
    router_node: RouterNode,
    handler_nodes: Vec<HandlerNode>,
    request_rows: Vec<RequestRow>,
    #[props(default)] layout_mode: LayoutMode,
    #[props(default = 4)] max_visible_requests: usize,
    #[props(default = "Request routing".to_string(), into)] heading: String,
    #[props(default)] on_reassign: Option<EventHandler<(RequestRow, String)>>,
    #[props(default)] on_open: Option<EventHandler<RequestRow>>,
    #[props(default)] extra_class: String,
) -> Element {
    let mut open = use_signal(|| None::<String>);
    let mut focus = use_signal(|| None::<String>);
    let mut nav = use_signal(|| None::<usize>);
    let known = use_hook(|| {
        request_rows
            .iter()
            .map(|r| (r.request_key.clone(), r.request_state))
            .collect::<Vec<(String, RequestState)>>()
    });
    use_hook(|| {
        let script = link_script();
        spawn(async move {
            let _ = document::eval(&script).await;
        });
    });
    use_drop(|| {
        document::eval(&format!(
            "window.__kitRrmLinks?.get({LINKS_DOM_ID:?})?.disconnect(); window.__kitRrmLinks?.delete({LINKS_DOM_ID:?});"
        ));
    });

    let top_all = top_requests(&request_rows);
    let (shown, hidden) = visible_split(top_all.len(), max_visible_requests);
    let top: Vec<&RequestRow> = top_all.into_iter().take(shown).collect();
    let order = nav_order(&top, &request_rows, &handler_nodes);
    let current_index = nav().unwrap_or(0).min(order.len().saturating_sub(1));
    let current = order.get(current_index);
    let (sorting, waiting, assigned) = header_counts(&request_rows);
    let counts_line = format!("{sorting} sorting · {waiting} waiting on you · {assigned} assigned");
    let changed = changed_requests(&known, &request_rows);
    let live = changed
        .last()
        .map(|row| {
            announcement(
                row,
                handler_name(&handler_nodes, row.assigned_to_key.as_deref()).as_deref(),
            )
        })
        .unwrap_or_default();
    let sorting_row = request_rows
        .iter()
        .find(|r| r.request_state == RequestState::Sorting)
        .cloned();
    let person = handler_nodes.iter().find(|h| h.is_person).cloned();
    let card_handlers: Vec<HandlerNode> = handler_nodes
        .iter()
        .filter(|h| !h.is_person)
        .cloned()
        .collect();
    let focus_now = focus();
    let open_row: Option<RequestRow> =
        open().and_then(|key| request_rows.iter().find(|r| r.request_key == key).cloned());
    let reassign_targets: Vec<(String, String)> = handler_nodes
        .iter()
        .map(|h| (h.handler_key.clone(), h.handler_label.clone()))
        .collect();
    let class = class_list("kit-rrm", &extra_class);
    let layout = layout_mode.as_str();
    let hidden_line = format!("{hidden} more requests");
    let reading = router_reading(&router_node, sorting_row.as_ref());
    let router_title = router_node.node_label.clone();
    let router_label = router_node.node_label.clone();
    let router_model = router_node.model_label.clone().unwrap_or_default();
    let router_mark = router_node.model_mark.clone().unwrap_or_default();
    let order_for_nav = order.clone();

    let ctx = Ctx {
        handlers: &handler_nodes,
        nav_order: &order,
        current,
        focus: focus_now.clone(),
        focus_sig: focus,
        open,
        nav,
        on_open,
    };
    let top_cards: Vec<Element> = top
        .iter()
        .map(|row| request_card(&ctx, row, None))
        .collect();
    let handler_cards: Vec<Element> = card_handlers
        .iter()
        .map(|h| handler_card(&ctx, h, &request_rows))
        .collect();
    let router_tab = tab_stop(&ctx, &NavTarget::Router);
    let router_position = nav_position(&ctx, &NavTarget::Router);
    let router_dimmed = is_dimmed(focus_now.as_deref(), None).to_string();
    let person_waiting = waiting_on_person(&request_rows);
    let person_line = person
        .as_ref()
        .map(|p| format!("{} · {person_waiting} waiting", p.handler_label));
    let mut links: Vec<(String, String, bool)> = top
        .iter()
        .map(|row| {
            (
                dom_id("request", &row.request_key),
                ROUTER_DOM_ID.to_string(),
                is_dimmed(focus_now.as_deref(), None),
            )
        })
        .collect();
    links.extend(card_handlers.iter().map(|h| {
        (
            ROUTER_DOM_ID.to_string(),
            dom_id("handler", &h.handler_key),
            is_dimmed(focus_now.as_deref(), Some(h.handler_key.as_str())),
        )
    }));

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class, "aria-label": "{heading}", "data-layout": layout,
            header { class: "kit-rrm__head",
                h2 { class: "kit-rrm__title", {heading.clone()} }
                p { class: "kit-rrm__counts", {counts_line} }
            }
            div { class: "kit-rrm__sr", role: "status", "aria-live": "polite", {live} }
            div {
                class: "kit-rrm__map",
                onkeydown: move |event: KeyboardEvent| {
                    if open().is_some() {
                        if event.key() == Key::Escape {
                            event.prevent_default();
                            let key = open().unwrap_or_default();
                            open.set(None);
                            focus_after_frame(&dom_id("request", &key));
                        }
                        return;
                    }
                    if event.key() == Key::Escape {
                        event.prevent_default();
                        focus.set(None);
                        return;
                    }
                    let key = match event.key() {
                        Key::ArrowDown | Key::ArrowRight => NavKey::Forward,
                        Key::ArrowUp | Key::ArrowLeft => NavKey::Back,
                        Key::Home => NavKey::Home,
                        Key::End => NavKey::End,
                        _ => return,
                    };
                    event.prevent_default();
                    if let Some(index) = nav_step(Some(current_index), order_for_nav.len(), key) {
                        nav.set(Some(index));
                        if let Some(target) = order_for_nav.get(index) {
                            focus_element(&target_dom_id(target));
                        }
                    }
                },
                svg { id: LINKS_DOM_ID, class: "kit-rrm__links", "aria-hidden": "true",
                    for (from, to, dim) in links {
                        path {
                            class: "kit-rrm__link",
                            "data-from": from,
                            "data-to": to,
                            "data-dim": dim.to_string(),
                        }
                    }
                }
                div { class: "kit-rrm__col kit-rrm__col--requests",
                    ul { class: "kit-rrm__requests",
                        for (index, card) in top_cards.into_iter().enumerate() {
                            li { key: "{index}", {card} }
                        }
                    }
                    if hidden > 0 {
                        p { class: "kit-rrm__more", {hidden_line} }
                    }
                }
                div { class: "kit-rrm__col kit-rrm__col--router",
                    article {
                        id: ROUTER_DOM_ID,
                        class: "kit-rrm__router",
                        tabindex: router_tab,
                        "data-dim": router_dimmed,
                        "aria-label": router_title.clone(),
                        onfocus: move |_| nav.set(router_position),
                        span { class: "kit-rrm__router-name", {router_label.clone()} }
                        if !router_model.is_empty() {
                            span { class: "kit-rrm__router-model", {router_model} }
                        }
                        if !router_mark.is_empty() {
                            span { class: "kit-rrm__router-mark", "aria-hidden": "true", {router_mark} }
                        }
                        if let Some(text) = reading.clone() {
                            p { class: "kit-rrm__reading", {text} }
                        }
                        if let Some(row) = sorting_row.clone() {
                            span { class: "kit-rrm__sorting-bar", role: "meter", "aria-valuemin": "0",
                                "aria-valuemax": "100", "aria-valuenow": percent_sure(row.certainty_fraction).to_string(),
                                span { class: "kit-rrm__fill", style: format!("width: {}%", percent_sure(row.certainty_fraction)) }
                            }
                        }
                    }
                    if let Some(line) = person_line {
                        div { class: "kit-rrm__person", "data-count": person_waiting.to_string(),
                            span { class: "kit-rrm__person-tag", {line} }
                        }
                    }
                }
                div { class: "kit-rrm__col kit-rrm__col--handlers",
                    for card in handler_cards {
                        {card}
                    }
                }
            }
            if let Some(row) = open_row {
                {reassign_dialog(&ctx, &row, &reassign_targets, on_reassign, open)}
            }
        }
    }
}

fn target_dom_id(target: &NavTarget) -> String {
    match target {
        NavTarget::Request(key) => dom_id("request", key),
        NavTarget::Router => ROUTER_DOM_ID.to_string(),
        NavTarget::Handler(key) => dom_id("handler", key),
    }
}

fn router_reading(router: &RouterNode, sorting: Option<&RequestRow>) -> Option<String> {
    match (
        router.reading_text.as_deref().filter(|t| !t.is_empty()),
        sorting,
    ) {
        (Some(text), _) => Some(text.to_string()),
        (None, Some(row)) => Some(format!("Reading {}", row.request_title)),
        (None, None) => None,
    }
}

fn reassign_dialog(
    ctx: &Ctx<'_>,
    row: &RequestRow,
    targets: &[(String, String)],
    on_reassign: Option<EventHandler<(RequestRow, String)>>,
    open: Signal<Option<String>>,
) -> Element {
    let mut open = open;
    let scores = score_rows(row, ctx.handlers);
    let handler = handler_name(ctx.handlers, row.assigned_to_key.as_deref());
    let line = sentence(row, handler.as_deref());
    let percent = percent_sure(row.certainty_fraction);
    let reason = row.reason_text.clone().unwrap_or_default();
    let controls: Vec<(String, String, String, RequestRow)> = targets
        .iter()
        .map(|(key, label)| {
            (
                key.clone(),
                label.clone(),
                format!("Reassign to {label}"),
                row.clone(),
            )
        })
        .collect();
    rsx! {
        div { class: "kit-rrm__dialog", role: "dialog", "aria-label": line.clone(),
            h3 { class: "kit-rrm__dialog-title", {row.request_title.clone()} }
            p { class: "kit-rrm__dialog-certainty", {format!("{percent}% sure")} }
            ul { class: "kit-rrm__scores", "aria-label": "Scores",
                for (label, score) in scores {
                    li { {format!("{label}: {score}%")} }
                }
            }
            if !reason.is_empty() {
                p { class: "kit-rrm__reason", {reason} }
            }
            if on_reassign.is_some() {
                div { class: "kit-rrm__reassign", role: "group", "aria-label": "Reassign",
                    for (key, label, aria, request) in controls {
                        button {
                            r#type: "button",
                            class: "kit-rrm__reassign-item",
                            "data-handler": key.clone(),
                            "aria-label": aria,
                            onclick: move |_| {
                                if let Some(handler) = on_reassign {
                                    handler.call((request.clone(), key.clone()));
                                }
                                open.set(None);
                            },
                            {label}
                        }
                    }
                }
            }
        }
    }
}
