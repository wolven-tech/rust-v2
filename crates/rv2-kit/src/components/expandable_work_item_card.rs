//! Expandable work item card: a summary of a work item that unfolds to show its
//! subtasks, members and status. Subtasks can be ticked, members filter the
//! list, and the status changes in place.
//!
//! Design choice: the header never moves; the body unfolds beneath it.
//! Design choice: the ring is one arc per subtask, drawn as SVG paths by
//! `ring_segments`, so the segment count follows the subtask count.
//! Design choice: a member filter matches the assignee name exactly; a subtask
//! whose assignee matches no member is shown without an avatar.
//! Design choice: `on_subtasks_update` set means the host owns the ticks.
//! Browser checks: the unfold fade, the check draw on tick, the title truncation
//! and avatar spacing at narrow width, and hover-mode open and close.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, focus_id, next_id};

const CSS: Asset = asset!("/assets/expandable-work-item-card.css");

#[derive(Clone, PartialEq, Debug)]
pub struct Subtask {
    pub subtask_id: String,
    pub subtask_text: String,
    pub is_done: bool,
    pub assignee_name: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Member {
    pub member_name: String,
    pub avatar: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Tone {
    #[default]
    Neutral,
    Positive,
    Caution,
    Critical,
}

impl Tone {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Positive => "positive",
            Self::Caution => "caution",
            Self::Critical => "critical",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct StatusOption {
    pub option_id: String,
    pub option_label: String,
    pub emphasis: Tone,
    pub option_mark: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct PrimaryAction {
    pub action_label: String,
    pub on_activate: EventHandler<()>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum OpenOn {
    #[default]
    Hover,
    Activate,
}

/// Done and total subtask counts.
pub fn progress(subtasks: &[Subtask]) -> (usize, usize) {
    (
        subtasks.iter().filter(|s| s.is_done).count(),
        subtasks.len(),
    )
}

/// SVG arc from `start_deg` to `end_deg` on a circle, angles clockwise from 12 o'clock.
pub fn arc_path(cx: f64, cy: f64, radius: f64, start_deg: f64, end_deg: f64) -> String {
    let point = |deg: f64| {
        let rad = (deg - 90.0).to_radians();
        (cx + radius * rad.cos(), cy + radius * rad.sin())
    };
    let (x0, y0) = point(start_deg);
    let (x1, y1) = point(end_deg);
    let large = if end_deg - start_deg > 180.0 { 1 } else { 0 };
    format!("M{x0:.3} {y0:.3} A{radius:.3} {radius:.3} 0 {large} 1 {x1:.3} {y1:.3}")
}

/// One arc per subtask, with a 3 degree gap between arcs; `(path, done)` pairs.
pub fn ring_segments(subtasks: &[Subtask]) -> Vec<(String, bool)> {
    let count = subtasks.len();
    if count == 0 {
        return Vec::new();
    }
    let slice = 360.0 / count as f64;
    subtasks
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let start = i as f64 * slice + 1.5;
            let end = (i + 1) as f64 * slice - 1.5;
            (
                arc_path(12.0, 12.0, 9.0, start, end.max(start + 0.1)),
                s.is_done,
            )
        })
        .collect()
}

/// `2 open` / `1 open` / `all done`. Used when no footer hint is supplied.
pub fn default_footer_hint(done: usize, total: usize) -> String {
    let open = total - done;
    if open == 0 {
        "all done".to_string()
    } else if open == 1 {
        "1 open".to_string()
    } else {
        format!("{open} open")
    }
}

/// Whether a subtask is at full emphasis under a member filter.
pub fn member_matches(subtask: &Subtask, filter: Option<&str>) -> bool {
    match filter {
        None => true,
        Some(name) => subtask.assignee_name.as_deref() == Some(name),
    }
}

/// Returns the list with the subtask `id` ticked or unticked.
pub fn toggle_subtask(subtasks: &[Subtask], id: &str) -> Vec<Subtask> {
    subtasks
        .iter()
        .map(|s| {
            let mut next = s.clone();
            if s.subtask_id == id {
                next.is_done = !s.is_done;
            }
            next
        })
        .collect()
}

/// Up to two initials from a name.
pub fn initials(name: &str) -> String {
    name.split_whitespace()
        .take(2)
        .filter_map(|word| word.chars().next())
        .flat_map(|c| c.to_uppercase())
        .collect()
}

/// A work item card that opens to reveal its subtasks, members and status.
#[component]
pub fn ExpandableWorkItemCard(
    #[props(default, into)] item_title: String,
    #[props(default)] subtask_list: Vec<Subtask>,
    #[props(default)] on_subtasks_update: Option<EventHandler<Vec<Subtask>>>,
    #[props(default, into)] status_id: String,
    #[props(default)] status_options: Vec<StatusOption>,
    #[props(default)] on_status_pick: Option<EventHandler<String>>,
    #[props(default)] due_text: Option<String>,
    #[props(default)] member_list: Vec<Member>,
    #[props(default)] primary_action: Option<PrimaryAction>,
    #[props(default)] footer_hint: Option<String>,
    #[props(default)] open_on: OpenOn,
    #[props(default)] is_open: Option<bool>,
    #[props(default)] default_open: bool,
    #[props(default)] on_open_change: Option<EventHandler<bool>>,
    #[props(default)] read_only: bool,
    #[props(default, into)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-item-{}", next_id()));
    let mut internal_open = use_signal(|| default_open);
    let mut internal_list = use_signal(|| subtask_list.clone());
    let mut member_filter = use_signal(|| Option::<String>::None);
    let mut status_open = use_signal(|| false);

    let open = is_open.unwrap_or(internal_open());
    let controlled_list = on_subtasks_update.is_some();
    let subtasks: Vec<Subtask> = if controlled_list {
        subtask_list.clone()
    } else {
        internal_list()
    };
    let (done, total) = progress(&subtasks);
    let filter = member_filter();
    let status_label = status_options
        .iter()
        .find(|o| o.option_id == status_id)
        .map(|o| o.option_label.clone())
        .unwrap_or_default();
    let status_tone = status_options
        .iter()
        .find(|o| o.option_id == status_id)
        .map(|o| o.emphasis)
        .unwrap_or_default();
    let due = due_text.clone();
    let summary = {
        let mut parts = vec![
            item_title.clone(),
            format!("{done} of {total} done"),
            status_label.clone(),
        ];
        if let Some(due) = &due {
            parts.push(format!("Due {due}"));
        }
        parts.join(", ")
    };
    let hint = footer_hint
        .clone()
        .unwrap_or_else(|| default_footer_hint(done, total));
    let expanded = if open { "true" } else { "false" };
    let status_expanded = if status_open() { "true" } else { "false" };
    let root_cls = class_list("kit-item", &root_class);
    let header_id = format!("{base}-head");
    let body_id = format!("{base}-body");
    let segments = ring_segments(&subtasks);
    let status_menu_id = format!("{base}-status");
    let show_status = open && status_open();
    let is_hover = open_on == OpenOn::Hover;
    let frac_text = format!("{done}/{total}");

    let mut set_open = move |value: bool| {
        if is_open.is_none() {
            internal_open.set(value);
        }
        if let Some(handler) = on_open_change {
            handler.call(value);
        }
        if !value {
            status_open.set(false);
        }
    };

    let toggle_subtask_handler = move |id: String| {
        if read_only {
            return;
        }
        let next = toggle_subtask(
            &subtask_list_for_toggle(&internal_list, controlled_list, &subtask_list),
            &id,
        );
        if let Some(handler) = on_subtasks_update {
            handler.call(next);
        } else {
            internal_list.set(next);
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        article {
            class: root_cls,
            "data-open": "{expanded}",
            onmouseenter: move |_| {
                if is_hover {
                    set_open(true);
                }
            },
            onmouseleave: move |_| {
                if is_hover {
                    set_open(false);
                }
            },
            onkeydown: {
                let status_key = status_menu_id.clone();
                let header_key = header_id.clone();
                move |event: KeyboardEvent| {
                    if matches!(event.key(), Key::Escape) {
                        if status_open() {
                            status_open.set(false);
                            focus_id(&status_key);
                        } else if open {
                            set_open(false);
                            focus_id(&header_key);
                        }
                    }
                }
            },
            button {
                id: "{header_id}",
                r#type: "button",
                class: "kit-item__head",
                "aria-expanded": "{expanded}",
                "aria-controls": "{body_id}",
                "aria-label": "{summary}",
                onclick: move |_| set_open(!open),
                svg { class: "kit-item__ring", view_box: "0 0 24 24", "aria-hidden": "true",
                    circle { class: "kit-item__ring-track", cx: "12", cy: "12", r: "9", fill: "none" }
                    for (index, (path, is_done)) in segments.iter().cloned().enumerate() {
                        {
                            let class = if is_done { "kit-item__seg is-done" } else { "kit-item__seg" };
                            let key = format!("{base}-seg-{index}");
                            rsx! { path { key: "{key}", class: class, d: "{path}", fill: "none" } }
                        }
                    }
                }
                span { class: "kit-item__title", "{item_title}" }
                span { class: "kit-item__meta",
                    span { class: "kit-item__frac", "{frac_text}" }
                    if let Some(due) = due {
                        span { class: "kit-item__due",
                            span { class: "kit-item__due-word", "Due " }
                            "{due}"
                        }
                    }
                    span { class: "kit-item__avatars", "aria-hidden": "true",
                        for member in member_list.iter().take(4) {
                            {
                                let letters = initials(&member.member_name);
                                let key = member.member_name.clone();
                                rsx! { span { key: "{key}", class: "kit-item__avatar", "{letters}" } }
                            }
                        }
                    }
                    span { class: "kit-item__status", "data-tone": status_tone.as_str(), "{status_label}" }
                }
            }
            if open {
                div { id: "{body_id}", class: "kit-item__body",
                    ul { class: "kit-item__subtasks",
                        for subtask in subtasks.iter() {
                            {
                                let id = subtask.subtask_id.clone();
                                let text = subtask.subtask_text.clone();
                                let assignee = subtask.assignee_name.clone();
                                let checked = subtask.is_done;
                                let matched = member_matches(subtask, filter.as_deref());
                                let matched_attr = if matched { "true" } else { "false" };
                                let assignee_name = assignee.clone().unwrap_or_default();
                                let mut toggle = toggle_subtask_handler.clone();
                                rsx! {
                                    li { key: "{id}", class: "kit-item__subtask", "data-match": "{matched_attr}",
                                        label {
                                            input {
                                                r#type: "checkbox",
                                                checked: checked,
                                                disabled: read_only,
                                                onchange: move |_| toggle(id.clone()),
                                            }
                                            span { class: "kit-item__text", "{text}" }
                                        }
                                        if assignee.is_some() {
                                            span { class: "kit-item__assignee", "{assignee_name}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "kit-item__members", role: "group", "aria-label": "Filter by member",
                        for member in member_list.iter() {
                            {
                                let name = member.member_name.clone();
                                let pressed = filter.as_deref() == Some(name.as_str());
                                let pressed_attr = if pressed { "true" } else { "false" };
                                let letters = initials(&name);
                                let pick = name.clone();
                                let key = name.clone();
                                rsx! {
                                    button {
                                        key: "{key}",
                                        r#type: "button",
                                        class: "kit-item__member",
                                        "aria-pressed": "{pressed_attr}",
                                        disabled: read_only,
                                        onclick: move |_| {
                                            if member_filter() == Some(pick.clone()) {
                                                member_filter.set(None);
                                            } else {
                                                member_filter.set(Some(pick.clone()));
                                            }
                                        },
                                        span { class: "kit-item__avatar", "aria-hidden": "true", "{letters}" }
                                        span { class: "kit-item__member-name", "{name}" }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "kit-item__status-row",
                        button {
                            id: "{status_menu_id}",
                            r#type: "button",
                            class: "kit-item__status-btn",
                            "aria-haspopup": "radiogroup",
                            "aria-expanded": "{status_expanded}",
                            disabled: read_only,
                            onclick: move |_| status_open.set(!status_open()),
                            "{status_label}"
                        }
                        if show_status {
                            div { class: "kit-item__options", role: "radiogroup", "aria-label": "Status",
                                for option in status_options.iter() {
                                    {
                                        let option_id = option.option_id.clone();
                                        let checked = if option_id == status_id { "true" } else { "false" };
                                        let label = option.option_label.clone();
                                        let tone = option.emphasis.as_str();
                                        let mark = option.option_mark.clone().unwrap_or_default();
                                        let pick_id = option_id.clone();
                                        let key = option_id.clone();
                                        rsx! {
                                            button {
                                                key: "{key}",
                                                r#type: "button",
                                                role: "radio",
                                                "aria-checked": "{checked}",
                                                "data-tone": tone,
                                                onclick: move |_| {
                                                    if read_only {
                                                        return;
                                                    }
                                                    status_open.set(false);
                                                    if let Some(handler) = on_status_pick {
                                                        handler.call(pick_id.clone());
                                                    }
                                                },
                                                span { class: "kit-item__option-mark", "aria-hidden": "true", "{mark}" }
                                                "{label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    footer { class: "kit-item__foot",
                        p { class: "kit-item__hint", role: "status", "aria-live": "polite", "{hint}" }
                        if let Some(action) = primary_action {
                            button {
                                r#type: "button",
                                class: "kit-item__primary",
                                onclick: move |_| action.on_activate.call(()),
                                "{action.action_label}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The subtask list a toggle starts from: the host's list when it owns the ticks, else the panel's.
fn subtask_list_for_toggle(
    internal: &Signal<Vec<Subtask>>,
    controlled: bool,
    host: &[Subtask],
) -> Vec<Subtask> {
    if controlled {
        host.to_vec()
    } else {
        internal.peek().clone()
    }
}
