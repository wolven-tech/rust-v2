//! Task assignment board: hands queued work to a team of agents, by drag,
//! by keyboard pick-up, by digit keys, or automatically to the least busy agent.
//!
//! Design choice: `clock_minutes` (minutes after midnight) replaces "now" for
//! "busy until", since wasm32 has no clock; default 0.
//! Design choice: the queue total is the cost of the minutes still owed:
//! remaining busy time plus the estimates of queued items, times the rate.
//! Design choice: `format_money` is a callback; the default is dollars.
//! Design choice: the "+N" overflow list is shown by CSS on hover or focus.
//! Design choice: keyboard pick-up is Enter to lift an item; Up and Down move
//! the highlighted worker; Enter again hands the item to it; Escape puts it back.
//! Browser checks: the drag lift and worker highlight, the queue move
//! animation, the automatic pace, and the overflow popover.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, sleep};

const CSS: Asset = asset!("/assets/task-assignment-board.css");

#[derive(Clone, PartialEq, Debug)]
pub struct Worker {
    pub worker_id: String,
    pub worker_name: String,
    pub worker_model_name: Option<String>,
    pub worker_avatar: Option<String>,
    /// Minutes left on the task running now.
    pub busy_minutes: f64,
    /// Charged per minute of hand-off and basis of the queue total.
    pub cost_per_minute: f64,
}

#[derive(Clone, PartialEq, Debug)]
pub struct WorkItem {
    pub work_id: String,
    pub work_title: String,
    pub estimate_minutes: f64,
    pub required_tools: Vec<String>,
    pub assigned_worker_id: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AssignMode {
    Automatic,
    #[default]
    Manual,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Proposal {
    pub work_id: String,
    pub worker_id: String,
}

fn minutes(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}

/// Items queued for a worker, in order.
pub fn queue_for<'a>(items: &'a [WorkItem], worker_id: &str) -> Vec<&'a WorkItem> {
    items
        .iter()
        .filter(|i| i.assigned_worker_id.as_deref() == Some(worker_id))
        .collect()
}

/// Items with no worker, or a worker that does not exist.
pub fn unassigned<'a>(items: &'a [WorkItem], workers: &[Worker]) -> Vec<&'a WorkItem> {
    items
        .iter()
        .filter(|i| match &i.assigned_worker_id {
            None => true,
            Some(id) => !workers.iter().any(|w| &w.worker_id == id),
        })
        .collect()
}

/// `now + busy + queued estimates`, in minutes after midnight.
pub fn busy_until(clock_minutes: f64, worker: &Worker, queue: &[&WorkItem]) -> f64 {
    clock_minutes
        + minutes(worker.busy_minutes)
        + queue
            .iter()
            .map(|i| minutes(i.estimate_minutes))
            .sum::<f64>()
}

/// Cost of the minutes a worker still owes: remaining busy time plus queued estimates, at its rate.
pub fn queued_cost(worker: &Worker, queue: &[&WorkItem]) -> f64 {
    let owed = minutes(worker.busy_minutes)
        + queue
            .iter()
            .map(|i| minutes(i.estimate_minutes))
            .sum::<f64>();
    owed * minutes(worker.cost_per_minute)
}

/// Assigns (or unassigns with `None`) a work item.
pub fn assign(items: &[WorkItem], work_id: &str, worker_id: Option<&str>) -> Vec<WorkItem> {
    items
        .iter()
        .map(|i| {
            let mut next = i.clone();
            if i.work_id == work_id {
                next.assigned_worker_id = worker_id.map(str::to_string);
            }
            next
        })
        .collect()
}

/// The least busy worker: smallest owed minutes, first in list on ties.
pub fn least_busy(workers: &[Worker], items: &[WorkItem]) -> Option<String> {
    workers
        .iter()
        .map(|w| {
            let owed = minutes(w.busy_minutes)
                + queue_for(items, &w.worker_id)
                    .iter()
                    .map(|i| minutes(i.estimate_minutes))
                    .sum::<f64>();
            (owed, w.worker_id.clone())
        })
        .fold(None::<(f64, String)>, |best, cur| match best {
            Some(b) if b.0 <= cur.0 => Some(b),
            _ => Some(cur),
        })
        .map(|(_, id)| id)
}

/// Next automatic hand-off: the first unassigned item to the least busy worker.
pub fn next_auto(workers: &[Worker], items: &[WorkItem]) -> Option<(String, String)> {
    let item = unassigned(items, workers)
        .first()
        .map(|i| i.work_id.clone())?;
    let worker = least_busy(workers, items)?;
    Some((item, worker))
}

/// Moves a highlight index among `len` workers, wrapping at both ends.
pub fn move_highlight(len: usize, current: usize, down: bool) -> usize {
    if len == 0 {
        return 0;
    }
    if down {
        (current + 1) % len
    } else {
        (current + len - 1) % len
    }
}

/// `09:05` for minutes after midnight.
pub fn clock(minute: f64) -> String {
    let total = minutes(minute).round() as u64 % 1440;
    format!("{:02}:{:02}", total / 60, total % 60)
}

fn default_money(amount: f64) -> String {
    format!("${amount:.2}")
}

/// Queue-based board; drag, keyboard or automatic assignment.
#[component]
pub fn TaskAssignmentBoard(
    #[props(default)] worker_list: Vec<Worker>,
    #[props(default)] work_items: Vec<WorkItem>,
    #[props(default)] on_assign: Option<EventHandler<(String, Option<String>)>>,
    #[props(default)] assignment_mode: Option<AssignMode>,
    #[props(default)] default_assignment_mode: AssignMode,
    #[props(default)] on_mode_change: Option<EventHandler<AssignMode>>,
    #[props(default)] proposed_assignment: Option<Proposal>,
    #[props(default)] clock_minutes: f64,
    #[props(default)] format_money: Option<Callback<f64, String>>,
    #[props(default = "Delegate".to_string(), into)] heading_text: String,
    #[props(default, into)] root_class: String,
) -> Element {
    let mut internal_items = use_signal(|| work_items.clone());
    let mut internal_mode = use_signal(move || default_assignment_mode);
    let mut picked = use_signal(|| Option::<String>::None);
    let mut highlight = use_signal(|| 0_usize);
    let mut dragging = use_signal(|| Option::<String>::None);
    let mut announce = use_signal(String::new);

    let controlled_items = on_assign.is_some();
    let items: Vec<WorkItem> = if controlled_items {
        work_items.clone()
    } else {
        internal_items()
    };
    let mode = assignment_mode.unwrap_or(internal_mode());
    let auto_on = mode == AssignMode::Automatic;
    let money = |v: f64| match format_money {
        Some(cb) => cb.call(v),
        None => default_money(v),
    };
    let root_cls = class_list("kit-board", &root_class);
    let announcement = announce();
    let pick_now = picked();
    let hl = highlight();
    let unassigned_items: Vec<WorkItem> = unassigned(&items, &worker_list)
        .into_iter()
        .cloned()
        .collect();
    let unassigned_count = unassigned_items.len();
    let total_cost: f64 = worker_list
        .iter()
        .map(|w| {
            let queue = queue_for(&items, &w.worker_id);
            queued_cost(w, &queue)
        })
        .sum();
    let total_text = money(total_cost);
    let auto_label = if auto_on {
        "Automatic on"
    } else {
        "Automatic off"
    };
    let auto_checked = if auto_on { "true" } else { "false" };
    let proposal = proposed_assignment.clone();
    let worker_ids: Vec<String> = worker_list.iter().map(|w| w.worker_id.clone()).collect();

    // Hands an item to a worker (or back to unassigned) and reports it.
    let apply = {
        let items = items.clone();
        move |work_id: String, worker_id: Option<String>| {
            let next = assign(&items, &work_id, worker_id.as_deref());
            let title = items
                .iter()
                .find(|i| i.work_id == work_id)
                .map(|i| i.work_title.clone())
                .unwrap_or_default();
            let target = worker_id
                .clone()
                .unwrap_or_else(|| "unassigned".to_string());
            announce.set(format!("{title} to {target}"));
            if let Some(handler) = on_assign {
                handler.call((work_id, worker_id));
            } else {
                internal_items.set(next);
            }
        }
    };

    let toggle_mode = move |_| {
        let next = if auto_on {
            AssignMode::Manual
        } else {
            AssignMode::Automatic
        };
        if assignment_mode.is_none() {
            internal_mode.set(next);
        }
        if let Some(handler) = on_mode_change {
            handler.call(next);
        }
    };

    let auto_items = items.clone();
    let auto_workers = worker_list.clone();
    let apply_auto = apply.clone();
    use_effect(use_reactive((&auto_on,), move |(on,)| {
        if !on {
            return;
        }
        let workers = auto_workers.clone();
        let current = auto_items.clone();
        let mut run = apply_auto.clone();
        spawn(async move {
            sleep(1200).await;
            if let Some((work, worker)) = next_auto(&workers, &current) {
                run(work, Some(worker));
            }
        });
    }));

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: root_cls,
            header { class: "kit-board__head",
                h2 { class: "kit-board__title", "{heading_text}" }
                label { class: "kit-board__auto",
                    input {
                        r#type: "checkbox",
                        role: "switch",
                        "aria-checked": "{auto_checked}",
                        checked: auto_on,
                        onchange: toggle_mode,
                    }
                    span { "{auto_label}" }
                }
            }
            div { class: "kit-board__body",
                div { class: "kit-board__unassigned", role: "region", "aria-label": "Unassigned",
                    div { class: "kit-board__col-head",
                        span { "Unassigned" }
                        span { class: "kit-board__count", "{unassigned_count}" }
                    }
                    ul { class: "kit-board__items",
                        for item in unassigned_items.iter().cloned() {
                            {
                                let id = item.work_id.clone();
                                let title = item.work_title.clone();
                                let est = format!("{:.0} min", item.estimate_minutes);
                                let tools = item.required_tools.join(", ");
                                let is_picked = pick_now.as_deref() == Some(id.as_str());
                                let is_proposed = proposal.as_ref().is_some_and(|p| p.work_id == id);
                                let class = if is_picked {
                                    "kit-board__item is-picked"
                                } else if is_proposed {
                                    "kit-board__item is-proposed"
                                } else {
                                    "kit-board__item"
                                };
                                let label = format!(
                                    "{title}, {est}. Enter to pick up, then Enter on a worker or a digit to assign"
                                );
                                let key = id.clone();
                                let pick_id = id.clone();
                                let digit_id = id.clone();
                                let enter_id = id.clone();
                                let drag_id = id.clone();
                                let mut apply_digit = apply.clone();
                                let mut apply_enter = apply.clone();
                                let digit_workers = worker_ids.clone();
                                let enter_workers = worker_ids.clone();
                                let hint = if is_picked { "Up or Down to choose a worker" } else { "" };
                                rsx! {
                                    li { key: "{key}",
                                        button {
                                            r#type: "button",
                                            class: class,
                                            draggable: "true",
                                            "aria-label": "{label}",
                                            ondragstart: move |_| dragging.set(Some(drag_id.clone())),
                                            ondragend: move |_| dragging.set(None),
                                            onclick: move |_| {
                                                if picked().as_deref() == Some(pick_id.as_str()) {
                                                    picked.set(None);
                                                } else {
                                                    picked.set(Some(pick_id.clone()));
                                                    highlight.set(0);
                                                }
                                            },
                                            onkeydown: move |event: KeyboardEvent| {
                                                match event.key() {
                                                    Key::Enter => {
                                                        event.prevent_default();
                                                        let held = picked().as_deref() == Some(enter_id.as_str());
                                                        if held {
                                                            if let Some(worker) = enter_workers.get(highlight()) {
                                                                picked.set(None);
                                                                apply_enter(enter_id.clone(), Some(worker.clone()));
                                                            }
                                                        } else {
                                                            picked.set(Some(enter_id.clone()));
                                                            highlight.set(0);
                                                        }
                                                    }
                                                    Key::ArrowDown | Key::ArrowUp => {
                                                        if picked().is_some() {
                                                            event.prevent_default();
                                                            let down = matches!(event.key(), Key::ArrowDown);
                                                            highlight.set(move_highlight(enter_workers.len(), highlight(), down));
                                                        }
                                                    }
                                                    Key::Escape => {
                                                        if picked().is_some() {
                                                            picked.set(None);
                                                            announce.set("Put back".to_string());
                                                        }
                                                    }
                                                    Key::Character(c) => {
                                                        if let Some(d) = c.chars().next().and_then(|ch| ch.to_digit(10)) {
                                                            if d >= 1 && (d as usize) <= digit_workers.len() {
                                                                event.prevent_default();
                                                                picked.set(None);
                                                                apply_digit(digit_id.clone(), Some(digit_workers[d as usize - 1].clone()));
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            },
                                            span { class: "kit-board__item-title", "{title}" }
                                            span { class: "kit-board__est", "{est}" }
                                            if !tools.is_empty() {
                                                span { class: "kit-board__tools", "{tools}" }
                                            }
                                            if is_proposed {
                                                span { class: "kit-board__mark", "proposed" }
                                            }
                                        }
                                        if is_picked {
                                            span { class: "kit-board__hint", "{hint}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "kit-board__workers", role: "group", "aria-label": "Workers",
                    div { class: "kit-board__col-head",
                        span { "Workers" }
                        span { class: "kit-board__total", "Total {total_text}" }
                    }
                    div { class: "kit-board__rows",
                        for (index, worker) in worker_list.iter().enumerate() {
                            {
                                let worker_id = worker.worker_id.clone();
                                let queue: Vec<WorkItem> = queue_for(&items, &worker_id).into_iter().cloned().collect();
                                let queue_refs: Vec<&WorkItem> = queue.iter().collect();
                                let until = clock(busy_until(clock_minutes, worker, &queue_refs));
                                let cost_value = queued_cost(worker, &queue_refs);
                                let cost = money(cost_value);
                                let rate = money(worker.cost_per_minute);
                                let model = worker.worker_model_name.clone().unwrap_or_default();
                                let name = worker.worker_name.clone();
                                let is_free = queue.is_empty() && worker.busy_minutes <= 0.0;
                                let free_text = if is_free {
                                    "Free".to_string()
                                } else {
                                    format!("Busy until {until}")
                                };
                                let highlighted = hl == index && pick_now.is_some();
                                let proposed_here = proposal.as_ref().is_some_and(|p| p.worker_id == worker_id);
                                let preview = proposal
                                    .as_ref()
                                    .filter(|p| p.worker_id == worker_id)
                                    .and_then(|p| items.iter().find(|i| i.work_id == p.work_id))
                                    .map(|item| {
                                        let mut with_item = queue_refs.clone();
                                        with_item.push(item);
                                        let badge = format!(
                                            "Busy until {}",
                                            clock(busy_until(clock_minutes, worker, &with_item))
                                        );
                                        let added = money(queued_cost(worker, &with_item) - cost_value);
                                        (badge, format!("+{added} with {}", item.work_title))
                                    });
                                let col_class = if highlighted || proposed_here { "kit-board__worker is-target" } else { "kit-board__worker" };
                                let status_class = if preview.is_some() {
                                    "kit-board__status is-preview"
                                } else if is_free {
                                    "kit-board__status is-free"
                                } else {
                                    "kit-board__status"
                                };
                                let (status_text, preview_line) = match preview {
                                    Some((badge, line)) => (badge, Some(line)),
                                    None => (free_text.clone(), None),
                                };
                                let number = (index + 1).to_string();
                                let overflow_count = queue.len().saturating_sub(3);
                                let overflow_label = queue
                                    .iter()
                                    .skip(3)
                                    .map(|i| i.work_title.clone())
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                let shown: Vec<WorkItem> = queue.iter().take(3).cloned().collect();
                                let drop_id = worker_id.clone();
                                let mut assign_drop = apply.clone();
                                let mut assign_pick = apply.clone();
                                let pick_worker_id = worker_id.clone();
                                let key = worker_id.clone();
                                let col_label = format!("{name}, {free_text}, queue total {cost}");
                                let queue_label = format!("Queue for {name}");
                                let hand_off = format!("Press Enter to hand the item to {name}");
                                let rate_line = format!("{rate} per min · {cost}");
                                rsx! {
                                    section { key: "{key}", class: col_class, "aria-label": "{col_label}",
                                        ondragover: move |event: DragEvent| event.prevent_default(),
                                        ondrop: move |event: DragEvent| {
                                            event.prevent_default();
                                            if let Some(work) = dragging() {
                                                dragging.set(None);
                                                assign_drop(work, Some(drop_id.clone()));
                                            }
                                        },
                                        onclick: move |_| {
                                            if let Some(work) = picked() {
                                                picked.set(None);
                                                assign_pick(work, Some(pick_worker_id.clone()));
                                            }
                                        },
                                        header { class: "kit-board__worker-head",
                                            span { class: "kit-board__digit", "aria-hidden": "true", "{number}" }
                                            span { class: "kit-board__id",
                                                span { class: "kit-board__name", "{name}" }
                                                if !model.is_empty() {
                                                    span { class: "kit-board__model", "{model}" }
                                                }
                                            }
                                            span { class: status_class,
                                                span { "{status_text}" }
                                                if let Some(line) = preview_line {
                                                    span { class: "kit-board__preview", "{line}" }
                                                }
                                            }
                                        }
                                        p { class: "kit-board__rate", "{rate_line}" }
                                        if highlighted {
                                            span { class: "kit-board__sr", "{hand_off}" }
                                        }
                                        ol { class: "kit-board__queue", "aria-label": "{queue_label}",
                                            for item in shown.iter().cloned() {
                                                {
                                                    let title = item.work_title.clone();
                                                    let est = format!("{:.0} min", item.estimate_minutes);
                                                    let tools = item.required_tools.join(", ");
                                                    let item_key = item.work_id.clone();
                                                    rsx! {
                                                        li { key: "{item_key}", class: "kit-board__queued",
                                                            span { "{title}" }
                                                            span { class: "kit-board__est", "{est}" }
                                                            if !tools.is_empty() {
                                                                span { class: "kit-board__tools", "{tools}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if overflow_count > 0 {
                                            span { class: "kit-board__overflow", tabindex: "0", "+{overflow_count}",
                                                span { class: "kit-board__overflow-list", role: "tooltip", "{overflow_label}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "kit-board__usage", "Drag to a worker, or Enter to pick up and Enter on a worker, or press its number." }
            p { class: "kit-board__sr", role: "status", "aria-live": "polite", "{announcement}" }
        }
    }
}
