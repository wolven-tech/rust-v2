//! Session packing planner: fits a backlog of agent tasks into a week of
//! sessions, each with a usage budget. Oversized tasks are divided across
//! sessions; pinned tasks stay where they are booked; pending tasks wait in a
//! holding pile until auto slotting or a drop places them.
//!
//! Design choice: placement is greedy in list order, day-major, and auto
//! slotting only uses sessions after the one in progress. A task goes whole
//! into the first session with room for it. A task larger than one session's
//! budget is divided across sessions; a division that cannot be completed, and
//! a task that fits one session but has no room, stays whole in the backlog.
//! A fragment is never smaller than `smallest_fragment`.
//! Design choice: the session in progress takes manual drops; sessions before
//! it refuse them.
//! Design choice: auto mode reveals the computed placements one by one at
//! `auto_delay` ms; a zero delay shows them at mount. Manual mode returns
//! unpinned placements to the pile.
//! Design choice: keyboard pick-up is Space to lift, arrows to aim (one day or
//! session per press), Space to drop, Escape to put back. Drag and drop uses
//! the HTML5 drag events, so a drag onto a session is the same drop.
//! Design choice: a refused drop keeps the card lifted and says why in the
//! live region.
//! Browser checks: the travel of cards from the pile, the lit destination while
//! dragging, the sideways scroll of the week grid, and the card panel.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, focus_after_frame, next_id, sleep};

const CSS: Asset = asset!("/assets/session-packing-planner.css");

/// Residue below this is floating-point noise, not usage.
const EPS: f64 = 1e-9;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Slot {
    pub day_index: usize,
    pub session_index: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ScheduledTask {
    pub task_id: String,
    pub task_title: String,
    /// Fraction of one session's budget.
    pub usage_share: f64,
    pub agent_name: Option<String>,
    pub task_detail: Option<String>,
    pub agent_logo: Option<String>,
    pub pinned_slot: Option<Slot>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PlacedPart {
    pub planned_task: String,
    pub day_index: usize,
    pub session_index: usize,
    pub usage_share: f64,
    pub part_number: usize,
    pub part_count: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PackMode {
    Auto,
    #[default]
    Manual,
}

/// The mode the other radio button selects.
pub fn other_mode(mode: PackMode) -> PackMode {
    match mode {
        PackMode::Auto => PackMode::Manual,
        PackMode::Manual => PackMode::Auto,
    }
}

/// What the live region says when the mode changes.
pub fn mode_announcement(mode: PackMode) -> &'static str {
    match mode {
        PackMode::Auto => "Auto: pending cards slot into sessions in order",
        PackMode::Manual => "Manual: unpinned cards returned to the pile",
    }
}

/// Places `tasks` into free room, day-major, in sessions after `current`.
/// Returns the parts placed and the ids still unplaced; an unplaced task leaves
/// no part and no load behind.
pub fn place(
    tasks: &[ScheduledTask],
    occupied: &[(Slot, f64)],
    grid: (usize, usize),
    budget: f64,
    smallest: f64,
    current: Option<Slot>,
) -> (Vec<PlacedPart>, Vec<String>) {
    let mut load: Vec<Vec<f64>> = vec![vec![0.0; grid.1]; grid.0];
    for (slot, used) in occupied {
        if slot.day_index < grid.0 && slot.session_index < grid.1 {
            load[slot.day_index][slot.session_index] += used;
        }
    }
    let mut parts = Vec::new();
    let mut unplaced = Vec::new();
    for task in tasks {
        match fit(task.usage_share, &load, grid, budget, smallest, current) {
            Some(pieces) => {
                let count = pieces.len();
                for (index, (day, session, usage)) in pieces.into_iter().enumerate() {
                    load[day][session] += usage;
                    parts.push(PlacedPart {
                        planned_task: task.task_id.clone(),
                        day_index: day,
                        session_index: session,
                        usage_share: usage,
                        part_number: index + 1,
                        part_count: count,
                    });
                }
            }
            None => unplaced.push(task.task_id.clone()),
        }
    }
    (parts, unplaced)
}

/// The (day, session, share) pieces for one task, or `None` when the task
/// cannot be fully placed against `load`.
fn fit(
    usage: f64,
    load: &[Vec<f64>],
    grid: (usize, usize),
    budget: f64,
    smallest: f64,
    current: Option<Slot>,
) -> Option<Vec<(usize, usize, f64)>> {
    let usage = usage.max(0.0);
    if usage <= EPS {
        return Some(Vec::new());
    }
    let min_piece = smallest.max(EPS);
    let mut open = Vec::new();
    for day in 0..grid.0 {
        for session in 0..grid.1 {
            let started = current.is_some_and(|c| (day, session) <= (c.day_index, c.session_index));
            if !started {
                open.push((day, session));
            }
        }
    }
    let room = |scratch: &[Vec<f64>], (day, session): (usize, usize)| {
        (budget - scratch[day][session]).max(0.0)
    };
    if let Some(&(day, session)) = open.iter().find(|&&slot| room(load, slot) + EPS >= usage) {
        return Some(vec![(day, session, usage)]);
    }
    if usage <= budget + EPS {
        return None;
    }
    let mut scratch = load.to_vec();
    let mut remaining = usage;
    let mut pieces = Vec::new();
    for (day, session) in open {
        if remaining <= EPS {
            break;
        }
        let free = room(&scratch, (day, session));
        if free < min_piece {
            continue;
        }
        let mut piece = remaining.min(free);
        let leftover = remaining - piece;
        if leftover > EPS && leftover < smallest {
            piece = remaining - smallest;
        }
        if piece < min_piece {
            continue;
        }
        scratch[day][session] += piece;
        remaining -= piece;
        pieces.push((day, session, piece));
    }
    (remaining <= EPS).then_some(pieces)
}

/// Why a placement of `usage` into `slot` is refused, if it is.
pub fn refusal(
    slot: Slot,
    usage: f64,
    occupied: &[(Slot, f64)],
    budget: f64,
    current: Option<Slot>,
) -> Option<String> {
    if let Some(cur) = current {
        if (slot.day_index, slot.session_index) < (cur.day_index, cur.session_index) {
            return Some("That session has already run".to_string());
        }
    }
    let used: f64 = occupied
        .iter()
        .filter(|(s, _)| *s == slot)
        .map(|(_, u)| u)
        .sum();
    let free = (budget - used).max(0.0);
    if usage > free + 1e-9 {
        Some(format!(
            "Not enough room: {:.0}% free, card needs {:.0}%",
            free * 100.0,
            usage * 100.0
        ))
    } else {
        None
    }
}

/// Checks a manual drop of task `id` into `slot`. `pins` are the booked tasks
/// and `drops` the manually placed parts; the task's own position is excluded.
pub fn try_place(
    tasks: &[ScheduledTask],
    pins: &[(String, Slot)],
    drops: &[PlacedPart],
    id: &str,
    slot: Slot,
    budget: f64,
    current: Option<Slot>,
) -> Result<PlacedPart, String> {
    let task = tasks
        .iter()
        .find(|t| t.task_id == id)
        .ok_or_else(|| "Unknown card".to_string())?;
    let mut occupied: Vec<(Slot, f64)> = pins
        .iter()
        .filter(|(p, _)| p.as_str() != id)
        .filter_map(|(p, s)| {
            tasks
                .iter()
                .find(|t| &t.task_id == p)
                .map(|t| (*s, t.usage_share))
        })
        .collect();
    occupied.extend(drops.iter().filter(|p| p.planned_task != id).map(|p| {
        (
            Slot {
                day_index: p.day_index,
                session_index: p.session_index,
            },
            p.usage_share,
        )
    }));
    if let Some(reason) = refusal(slot, task.usage_share, &occupied, budget, current) {
        return Err(reason);
    }
    Ok(PlacedPart {
        planned_task: id.to_string(),
        day_index: slot.day_index,
        session_index: slot.session_index,
        usage_share: task.usage_share,
        part_number: 1,
        part_count: 1,
    })
}

/// Splits a usage across sessions of `budget`, each part at or above `smallest`.
pub fn split_usage(usage: f64, budget: f64, smallest: f64) -> Vec<f64> {
    if budget <= 0.0 || usage <= budget {
        return vec![usage];
    }
    let mut n = (usage / budget).ceil() as usize;
    while n > 1 && usage / (n as f64) < smallest {
        n -= 1;
    }
    vec![usage / n as f64; n]
}

fn title_of(tasks: &[ScheduledTask], id: &str) -> String {
    tasks
        .iter()
        .find(|t| t.task_id == id)
        .map(|t| t.task_title.clone())
        .unwrap_or_else(|| id.to_string())
}

fn task_by_id<'a>(tasks: &'a [ScheduledTask], id: &str) -> Option<&'a ScheduledTask> {
    tasks.iter().find(|t| t.task_id == id)
}

fn slot_text(slot: Slot, days: &[String], sessions: &[String]) -> String {
    let day = days.get(slot.day_index).cloned().unwrap_or_default();
    let row = sessions
        .get(slot.session_index)
        .cloned()
        .unwrap_or_default();
    format!("{day} {row}").trim().to_string()
}

/// Inline custom property that colours an agent's marks. Agents take the
/// identity tokens in roster order and wrap after six; a card with no agent
/// takes the first.
fn identity_style(agents: &[(String, Option<String>)], agent: &str) -> String {
    let index = agents
        .iter()
        .position(|(name, _)| name == agent)
        .unwrap_or(0);
    format!("--gx: var(--kit-id-{});", index % 6 + 1)
}

fn mark_of(task: &ScheduledTask) -> String {
    match (&task.agent_name, &task.agent_logo) {
        (None, _) => String::new(),
        (Some(_), Some(logo)) => logo.clone(),
        (Some(name), None) => name.chars().take(1).collect(),
    }
}

#[derive(Clone, PartialEq, Debug)]
struct PileCard {
    task_id: String,
    title: String,
    identity: String,
    mark: String,
    share: String,
    unplaceable: bool,
}

#[derive(Clone, PartialEq, Debug)]
struct CellCard {
    task_id: String,
    title: String,
    agent: String,
    identity: String,
    mark: String,
    usage: f64,
    part: Option<(usize, usize)>,
    pinned: bool,
}

#[derive(Clone, PartialEq, Debug)]
struct CellView {
    cards: Vec<CellCard>,
    used: f64,
    elapsed: bool,
    current: bool,
}

/// A week of sessions with a pile of pending tasks and drag or keyboard placement.
#[component]
pub fn SessionPackingPlanner(
    #[props(default)] task_queue: Vec<ScheduledTask>,
    #[props(default)] day_headers: Vec<String>,
    #[props(default)] row_headers: Vec<String>,
    #[props(default)] current_slot: Option<Slot>,
    /// Where the current time falls inside the session in progress's cell: 0 at its top edge, 1 at its bottom.
    #[props(default = 0.5)]
    now_fraction: f64,
    #[props(default = 1.0)] session_budget: f64,
    #[props(default = 0.1)] smallest_fragment: f64,
    #[props(default)] initial_mode: PackMode,
    #[props(default = 800)] auto_delay: u32,
    #[props(default)] on_plan_settled: Option<EventHandler<Vec<PlacedPart>>>,
    #[props(default)] on_drop: Option<EventHandler<(String, Vec<PlacedPart>)>>,
    #[props(default)] on_relocate: Option<EventHandler<(PlacedPart, Slot)>>,
    #[props(default = "Session plan".to_string(), into)] title: String,
    #[props(default, into)] note: String,
    #[props(default, into)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-pack-{}", next_id()));
    let mut mode = use_signal(move || initial_mode);
    let mut revealed = use_signal(move || if auto_delay == 0 { usize::MAX } else { 0 });
    let mut dropped = use_signal(Vec::<PlacedPart>::new);
    let mut moved_pins = use_signal(Vec::<(String, Slot)>::new);
    let mut lifted = use_signal(|| Option::<(String, Slot)>::None);
    let mut dragging = use_signal(|| Option::<String>::None);
    let mut agent_filter = use_signal(|| Option::<String>::None);
    let mut open_task = use_signal(|| Option::<String>::None);
    let mut announce = use_signal(String::new);
    let mut settled_once = use_signal(|| false);

    let days = day_headers.len().max(1);
    let sessions = row_headers.len().max(1);
    let grid = (days, sessions);
    let current = current_slot;
    let budget = session_budget.max(EPS);

    let pinned: Vec<(ScheduledTask, Slot)> = task_queue
        .iter()
        .filter_map(|t| {
            let moved = moved_pins()
                .iter()
                .find(|(id, _)| id == &t.task_id)
                .map(|(_, s)| *s);
            moved.or(t.pinned_slot).map(|s| (t.clone(), s))
        })
        .collect();
    let pin_pairs: Vec<(String, Slot)> = pinned
        .iter()
        .map(|(t, s)| (t.task_id.clone(), *s))
        .collect();
    let pinned_occupied: Vec<(Slot, f64)> =
        pinned.iter().map(|(t, s)| (*s, t.usage_share)).collect();
    let unpinned: Vec<ScheduledTask> = task_queue
        .iter()
        .filter(|t| !pinned.iter().any(|(p, _)| p.task_id == t.task_id))
        .cloned()
        .collect();

    let drops = dropped();
    let mut occupied = pinned_occupied.clone();
    occupied.extend(drops.iter().map(|p| {
        (
            Slot {
                day_index: p.day_index,
                session_index: p.session_index,
            },
            p.usage_share,
        )
    }));
    let auto_on = mode() == PackMode::Auto;
    let (auto_parts, auto_unplaced) = if auto_on {
        place(
            &unpinned,
            &occupied,
            grid,
            session_budget,
            smallest_fragment,
            current,
        )
    } else {
        (Vec::new(), Vec::new())
    };
    let total_parts = auto_parts.len();
    let shown_parts: Vec<PlacedPart> = auto_parts.iter().take(revealed()).cloned().collect();
    let pile: Vec<ScheduledTask> = unpinned
        .iter()
        .filter(|t| {
            !drops.iter().any(|p| p.planned_task == t.task_id)
                && (!auto_on || !shown_parts.iter().any(|p| p.planned_task == t.task_id))
        })
        .cloned()
        .collect();

    let agents: Vec<(String, Option<String>)> = {
        let mut seen: Vec<(String, Option<String>)> = Vec::new();
        for t in &task_queue {
            if let Some(name) = &t.agent_name {
                if !seen.iter().any(|(n, _)| n == name) {
                    seen.push((name.clone(), t.agent_logo.clone()));
                }
            }
        }
        seen
    };

    let pile_cards: Vec<PileCard> = pile
        .iter()
        .map(|t| PileCard {
            task_id: t.task_id.clone(),
            title: t.task_title.clone(),
            identity: identity_style(&agents, t.agent_name.as_deref().unwrap_or_default()),
            mark: mark_of(t),
            share: format!("{:.0}%", t.usage_share / budget * 100.0),
            unplaceable: auto_on && auto_unplaced.contains(&t.task_id),
        })
        .collect();

    let settled_parts = auto_parts.clone();
    use_effect(use_reactive(
        (&auto_on, &total_parts),
        move |(on, total)| {
            if on && total > 0 && !*settled_once.peek() {
                let parts = settled_parts.clone();
                spawn(async move {
                    for step in 1..=total {
                        if auto_delay > 0 {
                            sleep(auto_delay).await;
                        }
                        revealed.set(step);
                    }
                    settled_once.set(true);
                    if let Some(handler) = on_plan_settled {
                        handler.call(parts);
                    }
                });
            }
            if !on {
                revealed.set(0);
                settled_once.set(false);
            }
        },
    ));

    let day_names: Vec<String> = day_headers.clone();
    let row_names: Vec<String> = row_headers.clone();

    let mut choose = move |next: PackMode| {
        mode.set(next);
        revealed.set(0);
        settled_once.set(false);
        if next == PackMode::Manual {
            dropped.set(Vec::new());
        }
        announce.set(mode_announcement(next).to_string());
    };

    let drop_at = {
        let tasks = task_queue.clone();
        let pins = pin_pairs.clone();
        let days_text = day_names.clone();
        let rows_text = row_names.clone();
        move |id: String, slot: Slot| {
            let drops_now = dropped();
            match try_place(
                &tasks,
                &pins,
                &drops_now,
                &id,
                slot,
                session_budget,
                current,
            ) {
                Err(reason) => announce.set(reason),
                Ok(part) => {
                    let place_label = slot_text(slot, &days_text, &rows_text);
                    let card = title_of(&tasks, &id);
                    if pins.iter().any(|(p, _)| p == &id) {
                        moved_pins.write().retain(|(p, _)| p != &id);
                        moved_pins.write().push((id.clone(), slot));
                        if let Some(handler) = on_relocate {
                            handler.call((part, slot));
                        }
                        announce.set(format!("{card} moved to {place_label}"));
                    } else {
                        dropped.write().retain(|p| p.planned_task != id);
                        dropped.write().push(part.clone());
                        if let Some(handler) = on_drop {
                            handler.call((id, vec![part]));
                        }
                        announce.set(format!("{card} placed in {place_label}"));
                    }
                    lifted.set(None);
                }
            }
        }
    };

    // Lifts a card for keyboard aiming; a card from a cell starts aiming at its own slot.
    let lift = {
        let tasks = task_queue.clone();
        let days_text = day_names.clone();
        let rows_text = row_names.clone();
        move |id: String, origin: Option<Slot>| {
            let aim = origin.unwrap_or_default();
            lifted.set(Some((id.clone(), aim)));
            announce.set(format!(
                "Lifted {}. Aim at {}",
                title_of(&tasks, &id),
                slot_text(aim, &days_text, &rows_text)
            ));
        }
    };

    let card_for = |id: &str, usage: f64, part: Option<(usize, usize)>, pinned: bool| {
        let task = task_by_id(&task_queue, id)?;
        let agent = task.agent_name.clone().unwrap_or_default();
        Some(CellCard {
            task_id: id.to_string(),
            title: task.task_title.clone(),
            identity: identity_style(&agents, &agent),
            mark: mark_of(task),
            agent,
            usage,
            part,
            pinned,
        })
    };
    let mut cell_views: Vec<CellView> = Vec::new();
    for session in 0..sessions {
        for day in 0..days {
            let mut cards: Vec<CellCard> = pinned
                .iter()
                .filter(|(_, s)| s.day_index == day && s.session_index == session)
                .filter_map(|(t, _)| card_for(&t.task_id, t.usage_share, None, true))
                .collect();
            cards.extend(
                drops
                    .iter()
                    .chain(shown_parts.iter())
                    .filter(|p| p.day_index == day && p.session_index == session)
                    .filter_map(|p| {
                        let part = (p.part_count > 1).then_some((p.part_number, p.part_count));
                        card_for(&p.planned_task, p.usage_share, part, false)
                    }),
            );
            let used: f64 = cards.iter().fold(0.0, |sum, c| sum + c.usage);
            cell_views.push(CellView {
                cards,
                used,
                elapsed: current.is_some_and(|c| (day, session) < (c.day_index, c.session_index)),
                current: current
                    == Some(Slot {
                        day_index: day,
                        session_index: session,
                    }),
            });
        }
    }
    let agent_names: Vec<(String, Option<String>)> = agents.clone();
    let filter_now = agent_filter();
    let auto_checked = if auto_on { "true" } else { "false" };
    let manual_checked = if auto_on { "false" } else { "true" };
    let auto_tab = if auto_on { "0" } else { "-1" };
    let manual_tab = if auto_on { "-1" } else { "0" };
    let auto_id = format!("{base}-mode-auto");
    let manual_id = format!("{base}-mode-manual");
    let focus_auto = auto_id.clone();
    let focus_manual = manual_id.clone();
    let mode_key = move |event: KeyboardEvent| {
        if matches!(event.key(), Key::ArrowLeft | Key::ArrowRight) {
            event.prevent_default();
            let next = other_mode(mode());
            choose(next);
            focus_after_frame(if next == PackMode::Auto {
                &focus_auto
            } else {
                &focus_manual
            });
        }
    };
    let status_text = announce();
    let root_cls = class_list("kit-pack", &root_class);
    let open_now = open_task();
    let lifted_now = lifted();
    let pile_len = pile_cards.len();
    let task_count = task_queue.len();
    let hint_id = format!("{base}-hint");

    let mut drop_key = drop_at.clone();
    let key_days = day_names.clone();
    let key_rows = row_names.clone();
    let key_root = move |event: KeyboardEvent| {
        let Some((id, aim)) = lifted() else {
            if matches!(event.key(), Key::Escape) {
                open_task.set(None);
            }
            return;
        };
        let moved = match event.key() {
            Key::ArrowLeft => Slot {
                day_index: aim.day_index.saturating_sub(1),
                ..aim
            },
            Key::ArrowRight => Slot {
                day_index: (aim.day_index + 1).min(days - 1),
                ..aim
            },
            Key::ArrowUp => Slot {
                session_index: aim.session_index.saturating_sub(1),
                ..aim
            },
            Key::ArrowDown => Slot {
                session_index: (aim.session_index + 1).min(sessions - 1),
                ..aim
            },
            Key::Character(c) if c == " " => {
                event.prevent_default();
                drop_key(id, aim);
                return;
            }
            Key::Escape => {
                event.prevent_default();
                lifted.set(None);
                announce.set("Put back".to_string());
                return;
            }
            _ => return,
        };
        event.prevent_default();
        lifted.set(Some((id, moved)));
        announce.set(format!("Aim at {}", slot_text(moved, &key_days, &key_rows)));
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            class: root_cls,
            style: "--kit-pack-days: {days};",
            onkeydown: key_root,
            header { class: "kit-pack__head",
                h2 { class: "kit-pack__title", "{title}" }
                if !note.is_empty() {
                    p { class: "kit-pack__note", "{note}" }
                }
                div {
                    class: "kit-pack__mode",
                    role: "radiogroup",
                    "aria-label": "Placement mode",
                    onkeydown: mode_key,
                    button {
                        id: "{auto_id}",
                        r#type: "button",
                        role: "radio",
                        "aria-checked": "{auto_checked}",
                        tabindex: "{auto_tab}",
                        onclick: move |_| choose(PackMode::Auto),
                        "Auto"
                    }
                    button {
                        id: "{manual_id}",
                        r#type: "button",
                        role: "radio",
                        "aria-checked": "{manual_checked}",
                        tabindex: "{manual_tab}",
                        onclick: move |_| choose(PackMode::Manual),
                        "Manual"
                    }
                }
            }
            div { class: "kit-pack__agents", role: "group", "aria-label": "Agents",
                for (name, logo) in agent_names.iter().cloned() {
                    {
                        let pressed = filter_now.as_deref() == Some(name.as_str());
                        let pressed_attr = if pressed { "true" } else { "false" };
                        let letter = logo.clone().unwrap_or_else(|| name.chars().take(1).collect());
                        let identity = identity_style(&agents, &name);
                        let pick = name.clone();
                        let key = name.clone();
                        let label = format!("Filter by {name}");
                        rsx! {
                            button {
                                key: "{key}",
                                r#type: "button",
                                class: "kit-pack__agent",
                                style: "{identity}",
                                "aria-pressed": "{pressed_attr}",
                                "aria-label": "{label}",
                                onclick: move |_| {
                                    if agent_filter() == Some(pick.clone()) {
                                        agent_filter.set(None);
                                    } else {
                                        agent_filter.set(Some(pick.clone()));
                                    }
                                },
                                "{letter}"
                            }
                        }
                    }
                }
                span { class: "kit-pack__count", "{task_count} tasks" }
            }
            ul { class: "kit-pack__key", "aria-label": "Agent colours",
                for (name, _) in agent_names.iter().cloned() {
                    {
                        let identity = identity_style(&agents, &name);
                        let key = name.clone();
                        rsx! {
                            li { key: "{key}", class: "kit-pack__key-item", style: "{identity}",
                                span { class: "kit-pack__swatch", "aria-hidden": "true" }
                                "{name}"
                            }
                        }
                    }
                }
            }
            div { class: "kit-pack__board",
            div { class: "kit-pack__pile", "aria-label": "Unplaced tasks",
                h3 { class: "kit-pack__pile-title", "Unplaced", span { class: "kit-pack__pile-count", "{pile_len}" } }
                ul { class: "kit-pack__cards",
                    for card in pile_cards.iter().cloned() {
                        {
                            let key = card.task_id.clone();
                            let dom = format!("{base}-pile-{}", card.task_id);
                            let is_lifted = lifted_now.as_ref().is_some_and(|(id, _)| id == &card.task_id);
                            let lift_class = if is_lifted { "kit-pack__card is-lifted" } else { "kit-pack__card" };
                            let drag_id = card.task_id.clone();
                            let open_id = card.task_id.clone();
                            let enter_id = card.task_id.clone();
                            let space_id = card.task_id.clone();
                            let mut lift_space = lift.clone();
                            let aria = if card.unplaceable {
                                format!("{}, no session has room for it", card.title)
                            } else {
                                card.title.clone()
                            };
                            rsx! {
                                li {
                                    key: "{key}",
                                    id: "{dom}",
                                    class: lift_class,
                                    style: "{card.identity}",
                                    role: "button",
                                    tabindex: "0",
                                    draggable: "true",
                                    "aria-label": "{aria}",
                                    "aria-describedby": "{hint_id}",
                                    ondragstart: move |_| dragging.set(Some(drag_id.clone())),
                                    ondragend: move |_| dragging.set(None),
                                    onclick: move |_| {
                                        if open_task().as_deref() == Some(open_id.as_str()) {
                                            open_task.set(None);
                                        } else {
                                            open_task.set(Some(open_id.clone()));
                                        }
                                    },
                                    onkeydown: move |event: KeyboardEvent| {
                                        match event.key() {
                                            Key::Enter => {
                                                event.prevent_default();
                                                if open_task().as_deref() == Some(enter_id.as_str()) {
                                                    open_task.set(None);
                                                } else {
                                                    open_task.set(Some(enter_id.clone()));
                                                }
                                            }
                                            Key::Character(c) if c == " " => {
                                                if lifted().is_none() {
                                                    event.prevent_default();
                                                    event.stop_propagation();
                                                    lift_space(space_id.clone(), None);
                                                }
                                            }
                                            _ => {}
                                        }
                                    },
                                    if !card.mark.is_empty() {
                                        span { class: "kit-pack__mark", "{card.mark}" }
                                    }
                                    span { class: "kit-pack__card-title", "{card.title}" }
                                    span { class: "kit-pack__share", "{card.share}" }
                                    if card.unplaceable {
                                        span { class: "kit-pack__none", "No session has room for this card" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "kit-pack__grid", role: "grid", "aria-label": "Week grid",
                div { class: "kit-pack__row kit-pack__row-head", role: "row",
                    span { role: "columnheader", "" }
                    for day_name in day_names.iter().cloned() {
                        span { key: "{day_name}", class: "kit-pack__day", role: "columnheader", "{day_name}" }
                    }
                }
                for session in 0..sessions {
                    {
                        let row_label = row_names.get(session).cloned().unwrap_or_default();
                        let key = format!("{base}-row-{session}");
                        rsx! {
                            div { key: "{key}", class: "kit-pack__row", role: "row",
                                span { class: "kit-pack__time", role: "rowheader", "{row_label}" }
                                for day in 0..days {
                                    {
                                        let cell = cell_views[session * days + day].clone();
                                        let elapsed_attr = if cell.elapsed { "true" } else { "false" };
                                        let current_attr = if cell.current { "true" } else { "false" };
                                        let cell_key = format!("{base}-cell-{day}-{session}");
                                        let slot = Slot { day_index: day, session_index: session };
                                        let fill = format!("width: {:.0}%;", (cell.used / budget * 100.0).min(100.0));
                                        let used_text = format!("{:.0}% used", cell.used / budget * 100.0);
                                        let slot_label = slot_text(slot, &day_names, &row_names);
                                        let now_style = format!("top: {:.2}%;", now_fraction.clamp(0.0, 1.0) * 100.0);
                                        let mut to_cell = drop_at.clone();
                                        rsx! {
                                            div {
                                                key: "{cell_key}",
                                                class: "kit-pack__cell",
                                                role: "gridcell",
                                                "data-elapsed": "{elapsed_attr}",
                                                "data-current": "{current_attr}",
                                                "aria-label": "{slot_label}",
                                                ondragover: move |event: DragEvent| event.prevent_default(),
                                                ondrop: move |event: DragEvent| {
                                                    event.prevent_default();
                                                    if let Some(id) = dragging() {
                                                        dragging.set(None);
                                                        to_cell(id, slot);
                                                    }
                                                },
                                                if cell.current {
                                                    span { class: "kit-pack__now", "In progress" }
                                                    span { class: "kit-pack__nowline", style: "{now_style}", "aria-hidden": "true" }
                                                }
                                                div { class: "kit-pack__stack",
                                                    for (card_index, card) in cell.cards.iter().cloned().enumerate() {
                                                        {
                                                            let faded = if filter_now.as_ref().is_some_and(|f| f != &card.agent) { "true" } else { "false" };
                                                            let card_key = format!("{cell_key}-{card_index}-{}", card.task_id);
                                                            let pct = format!("{:.0}%", card.usage / budget * 100.0);
                                                            let aria = match card.part {
                                                                Some((n, m)) => format!("{}, {pct} of session, part {n} of {m}", card.title),
                                                                None => format!("{}, {pct} of session", card.title),
                                                            };
                                                            let drag_id = card.task_id.clone();
                                                            let open_id = card.task_id.clone();
                                                            let mut lift_cell = lift.clone();
                                                            let lift_id = card.task_id.clone();
                                                            rsx! {
                                                                button {
                                                                    key: "{card_key}",
                                                                    r#type: "button",
                                                                    class: "kit-pack__card",
                                                                    style: "{card.identity}",
                                                                    draggable: "true",
                                                                    "data-faded": "{faded}",
                                                                    "aria-label": "{aria}",
                                                                    ondragstart: move |_| dragging.set(Some(drag_id.clone())),
                                                                    ondragend: move |_| dragging.set(None),
                                                                    onclick: move |_| open_task.set(Some(open_id.clone())),
                                                                    onkeydown: move |event: KeyboardEvent| {
                                                                        if let Key::Character(c) = event.key() {
                                                                            if c == " " && lifted().is_none() {
                                                                                event.prevent_default();
                                                                                event.stop_propagation();
                                                                                lift_cell(lift_id.clone(), Some(slot));
                                                                            }
                                                                        }
                                                                    },
                                                                    if !card.mark.is_empty() {
                                                                        span { class: "kit-pack__mark", "{card.mark}" }
                                                                    }
                                                                    span { class: "kit-pack__card-title", "{card.title}" }
                                                                    span { class: "kit-pack__share", "{pct}" }
                                                                    if let Some((n, m)) = card.part {
                                                                        span { class: "kit-pack__part", "part {n} of {m}" }
                                                                    }
                                                                    if card.pinned {
                                                                        span { class: "kit-pack__pin", "pinned" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                div { class: "kit-pack__meter",
                                                    span { class: "kit-pack__used", "{used_text}" }
                                                    span { class: "kit-pack__track",
                                                        span { class: "kit-pack__capacity", style: "{fill}" }
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
            if let Some(id) = open_now {
                {
                    let panel_title = title_of(&task_queue, &id);
                    let detail = task_by_id(&task_queue, &id)
                        .and_then(|t| t.task_detail.clone())
                        .unwrap_or_default();
                    let agent = task_by_id(&task_queue, &id)
                        .and_then(|t| t.agent_name.clone())
                        .unwrap_or_default();
                    rsx! {
                        aside { class: "kit-pack__panel", role: "region", "aria-label": "{panel_title}",
                            h3 { "{panel_title}" }
                            p { "{agent}" }
                            p { "{detail}" }
                            button {
                                r#type: "button",
                                class: "kit-pack__close",
                                onclick: move |_| open_task.set(None),
                                "Close"
                            }
                        }
                    }
                }
            }
            p { id: "{hint_id}", class: "kit-pack__hint", "Press Space to pick up a card, arrows to aim, Space to drop, Escape to cancel." }
            div { class: "kit-pack__sr", role: "status", "aria-live": "polite", "{status_text}" }
        }
    }
}
