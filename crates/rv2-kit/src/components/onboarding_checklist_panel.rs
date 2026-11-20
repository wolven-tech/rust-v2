//! Onboarding checklist panel: grouped setup tasks in which each task unlocks after the one before
//! it in its group. Finished groups fold away and the next group opens.
//!
//! Design choice: the first task of each group is open to tick; the chain runs within a group.
//! Unticking a task also clears the later tasks in its group, since they can no longer be locked
//! open. `is_locked` on a task adds a lock on top of the chain.
//!
//! Browser checks: a finished group collapses and focus moves to the next group; the progress bar
//! animates; reduced motion removes the transitions.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/onboarding-checklist-panel.css");

#[derive(Clone, Debug, PartialEq)]
pub struct CheckItem {
    pub item_id: String,
    pub item_text: String,
    pub is_done: bool,
    pub is_locked: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckGroup {
    pub group_id: String,
    pub group_title: String,
    pub task_list: Vec<CheckItem>,
}

/// Ids of tasks already marked done.
pub fn initial_done(groups: &[CheckGroup]) -> Vec<String> {
    groups
        .iter()
        .flat_map(|g| g.task_list.iter())
        .filter(|t| t.is_done)
        .map(|t| t.item_id.clone())
        .collect()
}

/// Per task of `group`: locked when it is explicitly locked or its predecessor is not done.
pub fn locked_flags(group: &CheckGroup, done: &[String]) -> Vec<bool> {
    group
        .task_list
        .iter()
        .enumerate()
        .map(|(index, task)| {
            task.is_locked || (index > 0 && !done.contains(&group.task_list[index - 1].item_id))
        })
        .collect()
}

/// Ticks a free task, or unticks a done one together with the later tasks of its group.
pub fn apply_tick(groups: &[CheckGroup], done: &[String], item_id: &str) -> Vec<String> {
    let mut next = done.to_vec();
    let Some(group) = groups
        .iter()
        .find(|g| g.task_list.iter().any(|t| t.item_id == item_id))
    else {
        return next;
    };
    let index = group
        .task_list
        .iter()
        .position(|t| t.item_id == item_id)
        .unwrap_or(0);
    if next.iter().any(|id| id == item_id) {
        for task in &group.task_list[index..] {
            next.retain(|id| *id != task.item_id);
        }
        return next;
    }
    if locked_flags(group, done)[index] {
        return next;
    }
    next.push(item_id.to_string());
    next
}

pub fn done_count(groups: &[CheckGroup], done: &[String]) -> usize {
    groups
        .iter()
        .flat_map(|g| g.task_list.iter())
        .filter(|t| done.contains(&t.item_id))
        .count()
}

pub fn total_count(groups: &[CheckGroup]) -> usize {
    groups.iter().map(|g| g.task_list.len()).sum()
}

/// Done tasks over total, rounded down.
pub fn progress_percent(done: usize, total: usize) -> usize {
    (done * 100).checked_div(total).unwrap_or(0)
}

pub fn group_done(group: &CheckGroup, done: &[String]) -> bool {
    !group.task_list.is_empty() && group.task_list.iter().all(|t| done.contains(&t.item_id))
}

#[component]
pub fn OnboardingChecklistPanel(
    group_list: Vec<CheckGroup>,
    #[props(default = "Setup guide".to_string(), into)] panel_title: String,
    #[props(default)] on_edit: Option<EventHandler<()>>,
    #[props(default)] on_dismiss: Option<EventHandler<()>>,
    #[props(default)] on_change: EventHandler<Vec<String>>,
    #[props(default, into)] extra_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-ocp-{}", next_id()));
    let mut done = use_signal(|| initial_done(&group_list));
    let mut open = use_signal(|| {
        let finished = initial_done(&group_list);
        group_list
            .iter()
            .map(|g| !group_done(g, &finished))
            .collect::<Vec<bool>>()
    });

    let finished = done();
    let total = total_count(&group_list);
    let completed = done_count(&group_list, &finished);
    let percent = progress_percent(completed, total);
    let percent_text = format!("{percent}%");
    let count_text = format!("{completed} of {total} done");
    let all_done = total > 0 && completed == total;
    let open_now = open();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class_list("kit-ocp", &extra_class), "aria-label": "{panel_title}",
            header { class: "kit-ocp__head",
                h3 { class: "kit-ocp__title", "{panel_title}" }
                if let Some(edit) = on_edit {
                    button { r#type: "button", class: "kit-ocp__icon", "aria-label": "Edit", onclick: move |_| edit.call(()), "✎" }
                }
                if let Some(dismiss) = on_dismiss {
                    button { r#type: "button", class: "kit-ocp__icon", "aria-label": "Close", onclick: move |_| dismiss.call(()), "×" }
                }
            }
            div { class: "kit-ocp__progress",
                div {
                    class: "kit-ocp__bar",
                    role: "progressbar",
                    "aria-label": "Setup progress",
                    "aria-valuemin": "0",
                    "aria-valuemax": "100",
                    "aria-valuenow": "{percent}",
                    span { class: "kit-ocp__fill", style: "width: {percent_text}" }
                }
                span { class: "kit-ocp__count", "{count_text}" }
            }
            p { class: "kit-ocp__complete", role: "status", "aria-live": "polite",
                if all_done { "Setup complete" }
            }
            for (group_index, group) in group_list.iter().enumerate() {
                {
                    let is_done_group = group_done(group, &finished);
                    let is_open = open_now.get(group_index).copied().unwrap_or(false);
                    let locks = locked_flags(group, &finished);
                    let panel_id = format!("{id}-group-{group_index}");
                    let group_name = group.group_title.clone();
                    let group_key = group.group_id.clone();
                    let group_count = format!(
                        "{} of {}",
                        group.task_list.iter().filter(|t| finished.contains(&t.item_id)).count(),
                        group.task_list.len()
                    );
                    rsx! {
                        div { key: "{group_key}", class: "kit-ocp__group", "data-done": is_done_group.then_some("true"),
                            button {
                                r#type: "button",
                                class: "kit-ocp__group-head",
                                "aria-expanded": if is_open { "true" } else { "false" },
                                "aria-controls": "{panel_id}",
                                onclick: move |_| {
                                    open.with_mut(|o| {
                                        if let Some(state) = o.get_mut(group_index) {
                                            *state = !*state;
                                        }
                                    });
                                },
                                span { class: "kit-ocp__group-title", "{group_name}" }
                                span { class: "kit-ocp__group-count", "{group_count}" }
                            }
                            if is_open {
                                ul { id: "{panel_id}", class: "kit-ocp__tasks",
                                    for (task_index, task) in group.task_list.iter().enumerate() {
                                        {
                                            let checked = finished.contains(&task.item_id);
                                            let locked = locks[task_index];
                                            let item_id = task.item_id.clone();
                                            let item_key = task.item_id.clone();
                                            let text = task.item_text.clone();
                                            let groups_for_tick = group_list.clone();
                                            rsx! {
                                                li { key: "{item_key}", class: "kit-ocp__task",
                                                    button {
                                                        r#type: "button",
                                                        class: "kit-ocp__toggle",
                                                        "aria-pressed": if checked { "true" } else { "false" },
                                                        disabled: locked,
                                                        "data-done": checked.then_some("true"),
                                                        onclick: move |_| {
                                                            let next = apply_tick(&groups_for_tick, &done(), &item_id);
                                                            let finished_now = groups_for_tick
                                                                .get(group_index)
                                                                .is_some_and(|g| group_done(g, &next));
                                                            done.set(next.clone());
                                                            if finished_now {
                                                                open.with_mut(|o| {
                                                                    if let Some(state) = o.get_mut(group_index) {
                                                                        *state = false;
                                                                    }
                                                                    if let Some(state) = o.get_mut(group_index + 1) {
                                                                        *state = true;
                                                                    }
                                                                });
                                                            }
                                                            on_change.call(next);
                                                        },
                                                        span { class: "kit-ocp__check", "aria-hidden": "true", if checked { "✓" } }
                                                        span { class: "kit-ocp__text", "{text}" }
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
    }
}
