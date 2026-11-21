//! Baton pass log: a task travels along a relay of agents. Each hand-off is
//! drawn as a transfer card between two agents, with the payload that went
//! along and whatever was dropped on the way.
//!
//! The component only draws. The host performs the hand-over in response to
//! `on_hand_over` and pushes transfers as the work moves.
//!
//! Design choice: no pictures. Each agent shows its initial in a mark.
//! Design choice: transfers are listed newest first; the slice given to the
//! component is taken as oldest first.
//! Design choice: "Ready for a task" means the agent holds a `todo` assignment.
//! Design choice: Escape cancels a pick-up from anywhere in the log.
//! Browser checks: HTML drag and drop between columns; the transit animation
//! plays once per new transfer id; the narrow single-column relay; the
//! keyboard pick-up and hand-over path.

use std::rc::Rc;

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/baton-pass-log.css");

/// Payload items shown on a collapsed transfer card.
const PREVIEW_ITEMS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentState {
    Idle,
    Working,
    Ready,
}

impl AgentState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::Ready => "ready",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Working => "Working",
            Self::Ready => "Ready for a task",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatonStage {
    Todo,
    Working,
    Done,
    Warning,
}

impl BatonStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::Working => "working",
            Self::Done => "done",
            Self::Warning => "warning",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Todo => "Waiting",
            Self::Working => "Working",
            Self::Done => "Done",
            Self::Warning => "Needs a look",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryType {
    File,
    Source,
    Fact,
    Decision,
    Note,
}

impl EntryType {
    fn label(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Source => "Source",
            Self::Fact => "Fact",
            Self::Decision => "Decision",
            Self::Note => "Note",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RelayAgent {
    pub id: String,
    pub name: String,
    pub model_label: Option<String>,
    pub role_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    pub id: String,
    pub caption: String,
    pub stage: BatonStage,
    pub holder_id: Option<String>,
    pub memo: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PayloadItem {
    pub id: String,
    pub text: String,
    pub entry_type: Option<EntryType>,
    pub dropped: bool,
    pub drop_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transfer {
    pub id: String,
    pub from_agent_id: String,
    pub to_agent_id: String,
    pub task_ref: String,
    pub payload: Vec<PayloadItem>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransferStatus {
    With(String),
    LooksGood,
    Warning(String),
}

impl TransferStatus {
    fn text(&self) -> String {
        match self {
            Self::With(name) => format!("With {name}"),
            Self::LooksGood => "Looks good".to_string(),
            Self::Warning(text) => text.clone(),
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Self::With(_) => "with",
            Self::LooksGood => "good",
            Self::Warning(_) => "warning",
        }
    }
}

pub fn position_of(relay: &[RelayAgent], id: &str) -> Option<usize> {
    relay.iter().position(|agent| agent.id == id)
}

/// The relay position an assignment is at, or `None` while it is pending.
pub fn column_of(relay: &[RelayAgent], assignment: &Assignment) -> Option<usize> {
    assignment
        .holder_id
        .as_deref()
        .and_then(|holder| position_of(relay, holder))
}

/// Cards only move forward: a pending card (`None`) may go anywhere; a card
/// at column `c` may go only to a column past `c`.
pub fn can_hand_over(current: Option<usize>, target: usize) -> bool {
    current.is_none_or(|column| target > column)
}

/// The first pending assignment, shown as "Up next" by the first agent.
pub fn up_next(assignments: &[Assignment]) -> Option<&Assignment> {
    assignments
        .iter()
        .find(|assignment| assignment.stage == BatonStage::Todo && assignment.holder_id.is_none())
}

/// The assignment an agent holds: its unfinished one first, else any held one.
pub fn held_by<'a>(assignments: &'a [Assignment], agent_id: &str) -> Option<&'a Assignment> {
    let held = || {
        assignments
            .iter()
            .filter(move |assignment| assignment.holder_id.as_deref() == Some(agent_id))
    };
    held()
        .find(|assignment| assignment.stage != BatonStage::Done)
        .or_else(|| held().next())
}

pub fn agent_state(agent_id: &str, assignments: &[Assignment]) -> AgentState {
    let mut state = AgentState::Idle;
    for assignment in assignments
        .iter()
        .filter(|assignment| assignment.holder_id.as_deref() == Some(agent_id))
    {
        match assignment.stage {
            BatonStage::Working => return AgentState::Working,
            BatonStage::Todo => state = AgentState::Ready,
            BatonStage::Done | BatonStage::Warning => {}
        }
    }
    state
}

pub fn transfer_status(
    transfer: &Transfer,
    assignments: &[Assignment],
    relay: &[RelayAgent],
) -> TransferStatus {
    let receiver = relay
        .iter()
        .find(|agent| agent.id == transfer.to_agent_id)
        .map_or_else(|| transfer.to_agent_id.clone(), |agent| agent.name.clone());
    let stage = assignments
        .iter()
        .find(|assignment| assignment.caption == transfer.task_ref)
        .map(|assignment| assignment.stage);
    match stage {
        Some(BatonStage::Done) => TransferStatus::LooksGood,
        Some(BatonStage::Warning) => {
            TransferStatus::Warning(BatonStage::Warning.label().to_string())
        }
        _ => TransferStatus::With(receiver),
    }
}

/// Transfers between relay positions `index` and `index + 1`, newest first.
pub fn transfers_between<'a>(
    transfers: &'a [Transfer],
    relay: &[RelayAgent],
    index: usize,
) -> Vec<&'a Transfer> {
    let (Some(from), Some(to)) = (relay.get(index), relay.get(index + 1)) else {
        return Vec::new();
    };
    transfers
        .iter()
        .rev()
        .filter(|transfer| transfer.from_agent_id == from.id && transfer.to_agent_id == to.id)
        .collect()
}

/// The first few payload texts, for a collapsed transfer card.
pub fn preview_texts(payload: &[PayloadItem]) -> Vec<String> {
    payload
        .iter()
        .take(PREVIEW_ITEMS)
        .map(|item| item.text.clone())
        .collect()
}

fn initial_of(name: &str) -> String {
    name.chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default()
}

fn toggle(list: &mut Vec<String>, key: &str) {
    match list.iter().position(|item| item == key) {
        Some(index) => {
            list.remove(index);
        }
        None => list.push(key.to_string()),
    }
}

/// One agent's card, with what it holds and whether a picked card may land here.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentView {
    pub index: usize,
    pub agent: RelayAgent,
    pub state: AgentState,
    pub slot_label: &'static str,
    pub slot_text: Option<String>,
    pub can_take: bool,
}

/// A relay of agents with the transfers between them and the pending work.
///
/// Hand-over is by drag, or by the keyboard path: activate a pending card to
/// pick it up, then activate an agent to its right.
#[component]
pub fn BatonPassLog(
    relay_agents: Vec<RelayAgent>,
    assignments: Vec<Assignment>,
    transfers: Vec<Transfer>,
    #[props(default)] on_hand_over: Option<EventHandler<(Assignment, RelayAgent)>>,
    #[props(default)] on_reinstate: Option<EventHandler<(Transfer, PayloadItem)>>,
    #[props(default)] extra_class: String,
) -> Element {
    let mut picked = use_signal(|| None::<String>);
    let mut dragging = use_signal(|| None::<String>);
    let mut open_transfers = use_signal(Vec::<String>::new);
    let mut open_items = use_signal(Vec::<String>::new);
    let mount_ids = use_hook(|| transfers.iter().map(|t| t.id.clone()).collect::<Vec<_>>());

    let hand = Rc::new({
        let relay = relay_agents.clone();
        let cards = assignments.clone();
        move |assignment_id: String, index: usize| {
            let (Some(card), Some(target)) = (
                cards.iter().find(|card| card.id == assignment_id),
                relay.get(index),
            ) else {
                return;
            };
            if can_hand_over(column_of(&relay, card), index) {
                if let Some(handler) = on_hand_over {
                    handler.call((card.clone(), target.clone()));
                }
            }
        }
    });

    let class = class_list("kit-baton", &extra_class);
    let picked_id = picked();
    let picked_caption = picked_id
        .as_ref()
        .and_then(|id| assignments.iter().find(|a| &a.id == id))
        .map(|a| a.caption.clone());
    let pick_line = picked_caption.map(|caption| {
        format!(
            "Holding {caption}. Activate an agent to its right to hand it over, or press Escape."
        )
    });
    let next = up_next(&assignments).cloned();
    let pending: Vec<Assignment> = assignments
        .iter()
        .filter(|a| a.holder_id.is_none())
        .cloned()
        .collect();
    let last = relay_agents.len().saturating_sub(1);
    let open_transfer_ids = open_transfers();
    let open_item_keys = open_items();

    let agent_views: Vec<AgentView> = relay_agents
        .iter()
        .enumerate()
        .map(|(index, agent)| {
            let state = agent_state(&agent.id, &assignments);
            let (slot_label, slot_card) = match (index, held_by(&assignments, &agent.id)) {
                (0, None) => ("Up next", next.as_ref()),
                (_, held) => ("Holding", held),
            };
            let picked_card = picked_id
                .as_ref()
                .and_then(|id| assignments.iter().find(|a| &a.id == id));
            AgentView {
                index,
                agent: agent.clone(),
                state,
                slot_label,
                slot_text: slot_card
                    .map(|card| format!("{} · {}", card.caption, card.stage.label())),
                can_take: picked_card
                    .is_some_and(|card| can_hand_over(column_of(&relay_agents, card), index)),
            }
        })
        .collect();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            class,
            "aria-label": "Baton pass log",
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape && picked().is_some() {
                    picked.set(None);
                }
            },
            span { class: "kit-baton__sr", role: "status", "aria-live": "polite", {pick_line.clone().unwrap_or_default()} }
            if let Some(line) = pick_line {
                p { class: "kit-baton__pick", "{line}" }
            }
            div { class: "kit-baton__relay",
                for view in agent_views {
                    AgentCard {
                        view: view.clone(),
                        on_activate: {
                            let hand = hand.clone();
                            let index = view.index;
                            move |_| {
                                if let Some(id) = picked() {
                                    hand(id, index);
                                    picked.set(None);
                                }
                            }
                        },
                        on_drop: {
                            let hand = hand.clone();
                            let index = view.index;
                            move |_| {
                                if let Some(id) = dragging() {
                                    hand(id, index);
                                }
                                dragging.set(None);
                            }
                        },
                    }
                    if view.index < last {
                        div { class: "kit-baton__lane",
                            for transfer in transfers_between(&transfers, &relay_agents, view.index).into_iter().cloned() {
                                TransferCard {
                                    status: transfer_status(&transfer, &assignments, &relay_agents),
                                    transit: !mount_ids.contains(&transfer.id),
                                    is_open: open_transfer_ids.contains(&transfer.id),
                                    open_items: open_item_keys.clone(),
                                    on_toggle: move |id: String| open_transfers.with_mut(|list| toggle(list, &id)),
                                    on_toggle_item: move |key: String| open_items.with_mut(|list| toggle(list, &key)),
                                    on_reinstate,
                                    transfer,
                                }
                            }
                        }
                    }
                }
            }
            section { class: "kit-baton__pending", "aria-labelledby": "kit-baton-pending",
                h3 { id: "kit-baton-pending", class: "kit-baton__pending-title", "Assignments" }
                if pending.is_empty() {
                    p { class: "kit-baton__empty", "No pending assignments." }
                }
                ul { class: "kit-baton__pending-list",
                    for card in pending {
                        li {
                            PendingCard {
                                is_picked: picked_id.as_deref() == Some(card.id.as_str()),
                                on_pick: move |id: String| {
                                    let next_pick = if picked() == Some(id.clone()) { None } else { Some(id) };
                                    picked.set(next_pick);
                                },
                                on_drag: move |id: String| dragging.set(Some(id)),
                                assignment: card,
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AgentCard(view: AgentView, on_activate: EventHandler<()>, on_drop: EventHandler<()>) -> Element {
    let initial = initial_of(&view.agent.name);
    let name = view.agent.name.clone();
    let state_label = view.state.label();
    let aria = format!("{name}, {state_label}");
    let slot_label = view.slot_label;
    let slot_text = view.slot_text.clone();
    let model = view.agent.model_label.clone();
    let role = view.agent.role_text.clone();
    rsx! {
        article {
            class: "kit-baton__agent",
            "data-state": view.state.as_str(),
            "data-target": view.can_take.then_some("true"),
            "aria-label": "{aria}",
            ondragover: move |event: DragEvent| event.prevent_default(),
            ondrop: move |event: DragEvent| {
                event.prevent_default();
                on_drop.call(());
            },
            button {
                r#type: "button",
                class: "kit-baton__agent-head",
                onclick: move |_| on_activate.call(()),
                span { class: "kit-baton__mark", "aria-hidden": "true", "{initial}" }
                span { class: "kit-baton__name", "{name}" }
                if let Some(model) = model {
                    span { class: "kit-baton__model", "{model}" }
                }
                span { class: "kit-baton__state", "{state_label}" }
            }
            if let Some(role) = role {
                p { class: "kit-baton__role", "{role}" }
            }
            if let Some(text) = slot_text {
                p { class: "kit-baton__slot", "{slot_label}: {text}" }
            }
        }
    }
}

#[component]
fn PendingCard(
    assignment: Assignment,
    is_picked: bool,
    on_pick: EventHandler<String>,
    on_drag: EventHandler<String>,
) -> Element {
    let id = assignment.id.clone();
    let drag_id = id.clone();
    let caption = assignment.caption.clone();
    let stage_label = assignment.stage.label();
    let memo = assignment.memo.clone();
    rsx! {
        button {
            r#type: "button",
            class: "kit-baton__card-pick",
            draggable: "true",
            "aria-pressed": is_picked.to_string(),
            ondragstart: move |_| on_drag.call(drag_id.clone()),
            onclick: move |_| on_pick.call(id.clone()),
            span { class: "kit-baton__caption", "{caption}" }
            span { class: "kit-baton__stage", "{stage_label}" }
            if let Some(memo) = memo {
                span { class: "kit-baton__memo", "{memo}" }
            }
        }
    }
}

#[component]
fn TransferCard(
    transfer: Transfer,
    status: TransferStatus,
    transit: bool,
    is_open: bool,
    open_items: Vec<String>,
    on_toggle: EventHandler<String>,
    on_toggle_item: EventHandler<String>,
    on_reinstate: Option<EventHandler<(Transfer, PayloadItem)>>,
) -> Element {
    let id = transfer.id.clone();
    let body_id = format!("kit-baton-transfer-{id}");
    let count = transfer.payload.len();
    let caption = transfer.task_ref.clone();
    let status_text = status.text();
    let status_kind = status.kind();
    let shown: Vec<PayloadItem> = if is_open {
        transfer.payload.clone()
    } else {
        transfer
            .payload
            .iter()
            .take(PREVIEW_ITEMS)
            .cloned()
            .collect()
    };
    let preview = preview_texts(&transfer.payload).join(", ");
    let toggle_id = id.clone();
    rsx! {
        article {
            class: "kit-baton__transfer",
            "data-transit": transit.then_some("true"),
            "data-open": is_open.then_some("true"),
            button {
                r#type: "button",
                class: "kit-baton__transfer-head",
                "aria-expanded": is_open.to_string(),
                "aria-controls": "{body_id}",
                onclick: move |_| on_toggle.call(toggle_id.clone()),
                span { class: "kit-baton__route", "{count} items" }
                span { class: "kit-baton__caption", "{caption}" }
                if !is_open {
                    span { class: "kit-baton__preview", "{preview}" }
                }
                span { class: "kit-baton__status", "data-status": status_kind, "{status_text}" }
            }
            ol { id: "{body_id}", class: "kit-baton__items", "aria-label": "Payload",
                for item in shown {
                    PayloadRow {
                        item_key: format!("{id}:{}", item.id),
                        is_open: open_items.contains(&format!("{id}:{}", item.id)),
                        on_toggle: on_toggle_item,
                        on_reinstate,
                        transfer: transfer.clone(),
                        item,
                    }
                }
            }
        }
    }
}

#[component]
fn PayloadRow(
    transfer: Transfer,
    item: PayloadItem,
    item_key: String,
    is_open: bool,
    on_toggle: EventHandler<String>,
    on_reinstate: Option<EventHandler<(Transfer, PayloadItem)>>,
) -> Element {
    let text = item.text.clone();
    let dropped = item.dropped;
    let dropped_label = if dropped { "dropped" } else { "" };
    let kind = item.entry_type.map(|kind| kind.label());
    let reason = item.drop_reason.clone();
    let toggle_key = item_key.clone();
    let reinstated = item.clone();
    rsx! {
        li { class: "kit-baton__item", "data-dropped": dropped.then_some("true"),
            button {
                r#type: "button",
                class: "kit-baton__item-head",
                "aria-expanded": is_open.to_string(),
                onclick: move |_| on_toggle.call(toggle_key.clone()),
                span { class: "kit-baton__item-text", "{text}" }
                if let Some(label) = kind {
                    span { class: "kit-baton__kind", "{label}" }
                }
                if dropped {
                    span { class: "kit-baton__dropped", "{dropped_label}" }
                }
            }
            if dropped && is_open {
                div { class: "kit-baton__reason",
                    if let Some(text) = reason {
                        p { "Dropped: {text}" }
                    }
                    if let Some(handler) = on_reinstate {
                        button {
                            r#type: "button",
                            class: "kit-baton__add-back",
                            onclick: move |_| handler.call((transfer.clone(), reinstated.clone())),
                            "Add back"
                        }
                    }
                }
            }
        }
    }
}
