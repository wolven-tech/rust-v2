//! Session packing planner: fits a backlog of agent tasks into a week of
//! sessions, each with a usage budget. Oversized tasks are divided across
//! sessions; pinned tasks stay where they are booked; pending tasks wait in a
//! holding pile until auto slotting or a drop places them.
//!
//! Design choice: placement is greedy in list order, day-major. A fragment is
//! never smaller than `smallest_fragment`; a task that cannot be fully placed
//! stays in the pile and is reported as unable to fit.
//! Design choice: auto mode reveals the computed placements one by one at
//! `auto_delay` ms; manual mode returns unpinned placements to the pile.
//! Design choice: keyboard pick-up is Space to lift, arrows to aim (one day or
//! session per press), Space to drop, Escape to put back. Drag and drop uses
//! the HTML5 drag events, so a drag onto a session is the same drop.
//! Design choice: a refused drop keeps the card lifted and says why in the
//! live region.
//! Browser checks: the travel of cards from the pile, the lit destination while
//! dragging, the sideways scroll of the week grid, and the card panel.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/session-packing-planner.css");

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

/// Places `tasks` into free room, day-major, skipping sessions before `current`.
/// Returns the parts placed and the ids still unplaced.
#[allow(clippy::needless_range_loop)] // day and session are also values in the placed parts.
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
        let mut remaining = task.usage_share.max(0.0);
        let mut pieces: Vec<PlacedPart> = Vec::new();
        'sessions: for day in 0..grid.0 {
            for session in 0..grid.1 {
                if remaining <= 1e-9 {
                    break 'sessions;
                }
                if let Some(cur) = current {
                    if (day, session) < (cur.day_index, cur.session_index) {
                        continue;
                    }
                }
                let free = (budget - load[day][session]).max(0.0);
                if free < smallest.max(1e-9) {
                    continue;
                }
                let mut piece = remaining.min(free);
                let leftover = remaining - piece;
                if leftover > 1e-9 && leftover < smallest {
                    piece = remaining - smallest;
                }
                if piece < smallest.max(1e-9) {
                    continue;
                }
                load[day][session] += piece;
                remaining -= piece;
                pieces.push(PlacedPart {
                    planned_task: task.task_id.clone(),
                    day_index: day,
                    session_index: session,
                    usage_share: piece,
                    part_number: 0,
                    part_count: 0,
                });
            }
        }
        if remaining > 1e-9 {
            unplaced.push(task.task_id.clone());
        }
        let count = pieces.len();
        for (i, mut p) in pieces.into_iter().enumerate() {
            p.part_number = i + 1;
            p.part_count = count;
            parts.push(p);
        }
    }
    (parts, unplaced)
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

fn slot_text(slot: Slot, days: &[String], sessions: &[String]) -> String {
    let day = days.get(slot.day_index).cloned().unwrap_or_default();
    let row = sessions
        .get(slot.session_index)
        .cloned()
        .unwrap_or_default();
    format!("{day} {row}").trim().to_string()
}

/// A week of sessions with a pile of pending tasks and drag or keyboard placement.
#[component]
pub fn SessionPackingPlanner(
    #[props(default)] task_queue: Vec<ScheduledTask>,
    #[props(default)] day_headers: Vec<String>,
    #[props(default)] row_headers: Vec<String>,
    #[props(default)] current_slot: Option<Slot>,
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
    let mut revealed = use_signal(|| 0_usize);
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
    let (auto_parts, _) = if auto_on {
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

    let settled_parts = auto_parts.clone();
    use_effect(use_reactive(
        (&auto_on, &total_parts),
        move |(on, total)| {
            if on && total > 0 && !*settled_once.peek() {
                let parts = settled_parts.clone();
                spawn(async move {
                    for step in 1..=total {
                        sleep(auto_delay).await;
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

    // Drops a card into a slot: checks the room, then records a relocation or a placement.
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

    let session_cards = |day: usize, session: usize| -> Vec<(String, f64)> {
        let mut out: Vec<(String, f64)> = pinned
            .iter()
            .filter(|(_, s)| s.day_index == day && s.session_index == session)
            .map(|(t, _)| (t.task_id.clone(), t.usage_share))
            .collect();
        out.extend(
            drops
                .iter()
                .chain(shown_parts.iter())
                .filter(|p| p.day_index == day && p.session_index == session)
                .map(|p| (p.planned_task.clone(), p.usage_share)),
        );
        out
    };
    let mut cells: Vec<(usize, usize, Vec<(String, f64)>, bool)> = Vec::new();
    for day in 0..days {
        for session in 0..sessions {
            let elapsed = current.is_some_and(|c| (day, session) < (c.day_index, c.session_index));
            cells.push((day, session, session_cards(day, session), elapsed));
        }
    }
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
    let filter_now = agent_filter();
    let manual_checked = if auto_on { "false" } else { "true" };
    let auto_checked = if auto_on { "true" } else { "false" };
    let status_text = announce();
    let root_cls = class_list("kit-pack", &root_class);
    let open_now = open_task();
    let lifted_now = lifted();
    let pile_len = pile.len();
    let pile_names: Vec<(String, String)> = pile
        .iter()
        .map(|t| (t.task_id.clone(), t.task_title.clone()))
        .collect();
    let budget = session_budget.max(1e-9);
    let hint_id = format!("{base}-hint");

    // Arrow keys aim a lifted card; Space drops it; Escape puts it back.
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
        section { class: root_cls, onkeydown: key_root,
            header { class: "kit-pack__head",
                h2 { class: "kit-pack__title", "{title}" }
                if !note.is_empty() {
                    p { class: "kit-pack__note", "{note}" }
                }
                div { class: "kit-pack__mode", role: "radiogroup", "aria-label": "Placement mode",
                    button {
                        r#type: "button",
                        role: "radio",
                        "aria-checked": "{auto_checked}",
                        onclick: move |_| {
                            mode.set(PackMode::Auto);
                            revealed.set(0);
                            settled_once.set(false);
                        },
                        "Auto"
                    }
                    button {
                        r#type: "button",
                        role: "radio",
                        "aria-checked": "{manual_checked}",
                        onclick: move |_| {
                            mode.set(PackMode::Manual);
                            revealed.set(0);
                            dropped.set(Vec::new());
                            announce.set("Unpinned cards returned to the pile".to_string());
                        },
                        "Manual"
                    }
                }
            }
            div { class: "kit-pack__agents", role: "group", "aria-label": "Agents",
                for (name, logo) in agents.iter().cloned() {
                    {
                        let pressed = filter_now.as_deref() == Some(name.as_str());
                        let pressed_attr = if pressed { "true" } else { "false" };
                        let letter = logo.clone().unwrap_or_else(|| name.chars().take(1).collect());
                        let pick = name.clone();
                        let key = name.clone();
                        let label = format!("Filter by {name}");
                        rsx! {
                            button {
                                key: "{key}",
                                r#type: "button",
                                class: "kit-pack__agent",
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
            }
            div { class: "kit-pack__pile", "aria-label": "Holding pile",
                h3 { class: "kit-pack__pile-title", "Waiting ({pile_len})" }
                ul { class: "kit-pack__cards",
                    for (task_id, card_title) in pile_names.iter().cloned() {
                        {
                            let key = task_id.clone();
                            let dom = format!("{base}-pile-{task_id}");
                            let is_lifted = lifted_now.as_ref().is_some_and(|(id, _)| id == &task_id);
                            let lift_class = if is_lifted { "kit-pack__card is-lifted" } else { "kit-pack__card" };
                            let drag_id = task_id.clone();
                            let open_id = task_id.clone();
                            let enter_id = task_id.clone();
                            let space_id = task_id.clone();
                            let mut lift_space = lift.clone();
                            rsx! {
                                li {
                                    key: "{key}",
                                    id: "{dom}",
                                    class: lift_class,
                                    role: "button",
                                    tabindex: "0",
                                    draggable: "true",
                                    "aria-label": "{card_title}",
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
                                    "{card_title}"
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
                                        let cell = cells
                                            .iter()
                                            .find(|(d, s, _, _)| *d == day && *s == session)
                                            .cloned();
                                        let (cards, elapsed) = cell.map(|(_, _, c, e)| (c, e)).unwrap_or_default();
                                        let elapsed_attr = if elapsed { "true" } else { "false" };
                                        let cell_key = format!("{base}-cell-{day}-{session}");
                                        let slot = Slot { day_index: day, session_index: session };
                                        let used: f64 = cards.iter().map(|(_, u)| *u).sum();
                                        let fill = format!("width: {:.0}%;", (used / budget * 100.0).min(100.0));
                                        let slot_label = slot_text(slot, &day_names, &row_names);
                                        let mut to_cell = drop_at.clone();
                                        rsx! {
                                            div {
                                                key: "{cell_key}",
                                                class: "kit-pack__cell",
                                                role: "gridcell",
                                                "data-elapsed": "{elapsed_attr}",
                                                "aria-label": "{slot_label}",
                                                ondragover: move |event: DragEvent| event.prevent_default(),
                                                ondrop: move |event: DragEvent| {
                                                    event.prevent_default();
                                                    if let Some(id) = dragging() {
                                                        dragging.set(None);
                                                        to_cell(id, slot);
                                                    }
                                                },
                                                for (card_index, (task_id, usage)) in cards.iter().cloned().enumerate() {
                                                    {
                                                        let card_title = title_of(&task_queue, &task_id);
                                                        let agent = task_queue
                                                            .iter()
                                                            .find(|t| t.task_id == task_id)
                                                            .and_then(|t| t.agent_name.clone())
                                                            .unwrap_or_default();
                                                        let faded = if filter_now.as_ref().is_some_and(|f| f != &agent) { "true" } else { "false" };
                                                        let card_key = format!("{cell_key}-{card_index}-{task_id}");
                                                        let pct = format!("{:.0}%", usage / budget * 100.0);
                                                        let aria = format!("{card_title}, {pct} of session");
                                                        let drag_id = task_id.clone();
                                                        let open_id = task_id.clone();
                                                        let mut lift_cell = lift.clone();
                                                        let lift_id = task_id.clone();
                                                        rsx! {
                                                            button {
                                                                key: "{card_key}",
                                                                r#type: "button",
                                                                class: "kit-pack__card",
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
                                                                "{card_title}"
                                                            }
                                                        }
                                                    }
                                                }
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
            if let Some(id) = open_now {
                {
                    let panel_title = title_of(&task_queue, &id);
                    let detail = task_queue
                        .iter()
                        .find(|t| t.task_id == id)
                        .and_then(|t| t.task_detail.clone())
                        .unwrap_or_default();
                    let agent = task_queue
                        .iter()
                        .find(|t| t.task_id == id)
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
