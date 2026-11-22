//! Reversible change log: what an agent changed in external systems, newest first.
//! A change reverts together with the changes built on it, where the system allows.
//!
//! Design choice: the component records intent and the host performs the external
//! revert. With no `on_revert` / `on_reapply` handler, the component updates the
//! row states itself.
//! Design choice: "also reverted" marks a reverted row built on another reverted row.
//! Design choice: the cascade is the breadth-first walk of `built_on_keys`, so the
//! selected row comes first and dependents follow in the order they depend on it.
//! Browser checks: cascade highlight on hover and focus; the cascade animation; new rows
//! entering at the top; container-query narrow layout; controls appear on hover or focus
//! and stay visible on touch.

use dioxus::prelude::*;

use crate::agents::delegation_card_tree::accent_hue;
use crate::agents::single_run_status_card::initial_of;
use crate::internal::{KIT_CSS, next_id, use_now_ms};

const CSS: Asset = asset!("/assets/reversible-change-log.css");

#[derive(Clone, PartialEq, Debug)]
pub struct ValueChange {
    pub from_text: String,
    pub to_text: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ChangeRow {
    pub row_key: String,
    pub verb_label: String,
    pub target_label: Option<String>,
    pub system_label: String,
    pub system_mark: Option<String>,
    pub happened_at_ms: u64,
    pub value_change: Option<ValueChange>,
    pub summary_line: Option<String>,
    pub reversible: bool,
    pub built_on_keys: Vec<String>,
}

impl ChangeRow {
    /// A reversible row with no dependencies and no value or summary text.
    pub fn new(row_key: &str, verb_label: &str, system_label: &str, happened_at_ms: u64) -> Self {
        Self {
            row_key: row_key.to_string(),
            verb_label: verb_label.to_string(),
            target_label: None,
            system_label: system_label.to_string(),
            system_mark: None,
            happened_at_ms,
            value_change: None,
            summary_line: None,
            reversible: true,
            built_on_keys: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowState {
    Applied,
    Reverted,
    NotReversible,
    Blocked,
    AlsoReverted,
}

impl RowState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Reverted => "reverted",
            Self::NotReversible => "not_reversible",
            Self::Blocked => "blocked",
            Self::AlsoReverted => "also_reverted",
        }
    }

    fn flag(self) -> Option<&'static str> {
        match self {
            Self::NotReversible | Self::Blocked => Some("Cannot revert"),
            Self::AlsoReverted => Some("Also reverted"),
            Self::Applied | Self::Reverted => None,
        }
    }
}

/// The row `key` followed by its reversible dependents, breadth first.
pub fn cascade_keys(rows: &[ChangeRow], key: &str) -> Vec<String> {
    let mut out = vec![key.to_string()];
    let mut next = 0;
    while next < out.len() {
        let current = out[next].clone();
        for row in rows {
            let depends = row.built_on_keys.contains(&current);
            if row.reversible && depends && !out.contains(&row.row_key) {
                out.push(row.row_key.clone());
            }
        }
        next += 1;
    }
    out
}

/// State of each row, in `rows` order. A row is blocked when it is built on a
/// reverted row that cannot be reverted, or on another blocked row.
pub fn row_states(rows: &[ChangeRow], reverted: &[String]) -> Vec<RowState> {
    let is_reverted = |key: &str| reverted.iter().any(|r| r == key);
    let position = |key: &str| rows.iter().position(|r| r.row_key == key);
    let mut blocked = vec![false; rows.len()];
    loop {
        let mut changed = false;
        for (i, row) in rows.iter().enumerate() {
            if blocked[i] {
                continue;
            }
            let hit = row.built_on_keys.iter().any(|dep| match position(dep) {
                Some(j) => blocked[j] || (is_reverted(dep) && !rows[j].reversible),
                None => false,
            });
            if hit {
                blocked[i] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            if is_reverted(&row.row_key) {
                if row.built_on_keys.iter().any(|dep| is_reverted(dep)) {
                    RowState::AlsoReverted
                } else {
                    RowState::Reverted
                }
            } else if !row.reversible {
                RowState::NotReversible
            } else if blocked[i] {
                RowState::Blocked
            } else {
                RowState::Applied
            }
        })
        .collect()
}

/// Indices into `rows`, newest first; equal times keep their input order.
pub fn newest_first(rows: &[ChangeRow]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(rows[i].happened_at_ms));
    order
}

/// Accessible name of a revert or reapply control.
pub fn control_name(verb: &str, target: &str, dependents: usize, reapply: bool) -> String {
    let prefix = if reapply { "Reapply" } else { "Revert" };
    let subject = if target.is_empty() {
        verb.to_string()
    } else {
        format!("{verb} {target}")
    };
    match dependents {
        0 => format!("{prefix}: {subject}"),
        1 => format!("{prefix}: {subject}, 1 dependent"),
        n => format!("{prefix}: {subject}, {n} dependents"),
    }
}

/// `just now`, `Nm ago`, `Nh ago`, `Nd ago`; a time after `now` reads as `just now`.
pub fn relative_time(now_ms: u64, then_ms: u64) -> String {
    let secs = now_ms.saturating_sub(then_ms) / 1000;
    match secs {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{}m ago", secs / 60),
        3600..=86_399 => format!("{}h ago", secs / 3600),
        _ => format!("{}d ago", secs / 86_400),
    }
}

/// Reverts `keys`, skipping any already reverted.
pub fn apply_revert(reverted: &mut Vec<String>, keys: &[String]) {
    for key in keys {
        if !reverted.contains(key) {
            reverted.push(key.clone());
        }
    }
}

/// Clears `keys` from the reverted set.
pub fn apply_reapply(reverted: &mut Vec<String>, keys: &[String]) {
    reverted.retain(|key| !keys.contains(key));
}

#[derive(Clone, PartialEq, Debug)]
struct Control {
    name: String,
    keys: Vec<String>,
    reapply: bool,
}

#[derive(Clone, PartialEq, Debug)]
struct RowView {
    key: String,
    verb: String,
    target: String,
    system: String,
    mark: Option<String>,
    values: Option<(String, String)>,
    summary: Option<String>,
    when: String,
    state: RowState,
    lit: bool,
    control: Option<Control>,
}

#[component]
pub fn ReversibleChangeLog(
    change_rows: Vec<ChangeRow>,
    #[props(default)] actor_label: Option<String>,
    #[props(default)] clock_ms: Option<u64>,
    #[props(default)] reverted_keys: Option<Vec<String>>,
    #[props(default)] default_reverted_keys: Vec<String>,
    #[props(default)] on_revert: Option<EventHandler<Vec<String>>>,
    #[props(default)] on_reapply: Option<EventHandler<Vec<String>>>,
    #[props(default)] spotlight_key: Option<String>,
    #[props(default = "Change log".to_string(), into)] heading: String,
    #[props(default)] class_name: String,
) -> Element {
    let id = use_hook(|| format!("kit-rcl-{}", next_id()));
    let mut internal = use_signal(|| default_reverted_keys.clone());
    let lit_row = use_signal(|| None::<String>);

    let now = use_now_ms(clock_ms, 1000);
    let controlled = reverted_keys.is_some();
    let reverted = reverted_keys.unwrap_or_else(|| internal());
    let local_revert = !controlled && on_revert.is_none();
    let local_reapply = !controlled && on_reapply.is_none();

    let states = row_states(&change_rows, &reverted);
    let lit_keys = match lit_row().or_else(|| spotlight_key.clone()) {
        Some(key) => cascade_keys(&change_rows, &key),
        None => Vec::new(),
    };
    let views: Vec<(String, RowView)> = newest_first(&change_rows)
        .into_iter()
        .map(|i| {
            let row = &change_rows[i];
            let state = states[i];
            let keys = cascade_keys(&change_rows, &row.row_key);
            let target = row.target_label.clone().unwrap_or_default();
            let dependents = keys.len() - 1;
            let control = match state {
                RowState::Applied => Some(Control {
                    name: control_name(&row.verb_label, &target, dependents, false),
                    keys,
                    reapply: false,
                }),
                RowState::Reverted | RowState::AlsoReverted => Some(Control {
                    name: control_name(&row.verb_label, &target, dependents, true),
                    keys,
                    reapply: true,
                }),
                RowState::NotReversible | RowState::Blocked => None,
            };
            RowView {
                key: row.row_key.clone(),
                verb: row.verb_label.clone(),
                target,
                system: row.system_label.clone(),
                mark: row.system_mark.clone(),
                values: row
                    .value_change
                    .as_ref()
                    .map(|vc| (vc.from_text.clone(), vc.to_text.clone())),
                summary: row.summary_line.clone(),
                when: relative_time(now, row.happened_at_ms),
                state,
                lit: lit_keys.contains(&row.row_key),
                control,
            }
        })
        .map(|view| (view.key.clone(), view))
        .collect();

    let total = change_rows.len();
    let reverted_count = states
        .iter()
        .filter(|s| matches!(s, RowState::Reverted | RowState::AlsoReverted))
        .count();
    let title_id = format!("{id}-title");
    let counts = if reverted_count > 0 {
        format!("{total} changes · {reverted_count} reverted")
    } else {
        format!("{total} changes")
    };
    let on_control = use_callback(move |(keys, reapply): (Vec<String>, bool)| {
        if reapply {
            if local_reapply {
                internal.write().retain(|key| !keys.contains(key));
            }
            if let Some(handler) = &on_reapply {
                handler.call(keys);
            }
        } else {
            if local_revert {
                let mut reverted = internal.write();
                apply_revert(&mut reverted, &keys);
            }
            if let Some(handler) = &on_revert {
                handler.call(keys);
            }
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "kit-rcl {class_name}",
            header { class: "kit-rcl__head",
                h3 { id: "{title_id}", class: "kit-rcl__title", "{heading}" }
                span { class: "kit-rcl__counts", "{counts}" }
                if let Some(actor) = actor_label {
                    span { class: "kit-rcl__actor", "by {actor}" }
                }
            }
            if views.is_empty() {
                p { class: "kit-rcl__empty", "No changes yet" }
            } else {
                ol { class: "kit-rcl__list", "aria-labelledby": "{title_id}",
                    for (key, view) in views {
                        ChangeRowItem {
                            key: "{key}",
                            view: view,
                            lit_row: lit_row,
                            on_control: on_control,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ChangeRowItem(
    view: RowView,
    lit_row: Signal<Option<String>>,
    on_control: Callback<(Vec<String>, bool)>,
) -> Element {
    let mut lit_row = lit_row;
    let key_focus = view.key.clone();
    let key_hover = view.key.clone();
    let control = view.control.clone();
    let control_keys = control.as_ref().map(|c| c.keys.clone()).unwrap_or_default();
    let control_reapply = control.as_ref().is_some_and(|c| c.reapply);
    let control_name = control.as_ref().map(|c| c.name.clone()).unwrap_or_default();
    let flag = view.state.flag().unwrap_or_default();
    let hue = accent_hue(&view.system);
    let initial = initial_of(&view.system);
    let verb = view.verb.clone();
    let target_text = view.target.clone();
    let when = view.when.clone();
    let system = view.system.clone();
    let mark_src = view.mark.clone();
    let also_reverted = view.state == RowState::AlsoReverted;

    rsx! {
        li {
            class: "kit-rcl__row",
            "data-state": view.state.as_str(),
            "data-lit": view.lit.then_some("true"),
            onmouseenter: move |_| lit_row.set(Some(key_hover.clone())),
            onmouseleave: move |_| lit_row.set(None),
            span { class: "kit-rcl__mark", "aria-hidden": "true", style: "--kit-rcl-hue: {hue}",
                if let Some(src) = mark_src {
                    img { src: src, alt: "" }
                } else {
                    {initial}
                }
            }
            div { class: "kit-rcl__body",
                p { class: "kit-rcl__line",
                    span { class: "kit-rcl__verb", "{verb}" }
                    if !target_text.is_empty() {
                        span { class: "kit-rcl__target", "{target_text}" }
                    }
                }
                if also_reverted {
                    span { class: "kit-rcl__flag", "Also reverted" }
                }
                if let Some((from, to)) = view.values.clone() {
                    p { class: "kit-rcl__values", "aria-hidden": "true",
                        span { class: "kit-rcl__from", "{from}" }
                        span { class: "kit-rcl__arrow", "→" }
                        span { class: "kit-rcl__to", "{to}" }
                    }
                    span { class: "kit-rcl__sr", "from {from} to {to}" }
                }
                if let Some(summary) = view.summary.clone() {
                    p { class: "kit-rcl__summary", "{summary}" }
                }
            }
            div { class: "kit-rcl__meta",
                time { class: "kit-rcl__time", "{when}" }
                span { class: "kit-rcl__system", "{system}" }
            }
            div { class: "kit-rcl__slot",
                if !flag.is_empty() && !also_reverted {
                    span { class: "kit-rcl__flag", "{flag}" }
                }
                if control.is_some() {
                    button {
                        class: "kit-rcl__control",
                        r#type: "button",
                        "data-reapply": control_reapply.then_some("true"),
                        "aria-label": "{control_name}",
                        onfocus: move |_| lit_row.set(Some(key_focus.clone())),
                        onblur: move |_| lit_row.set(None),
                        onclick: move |_| on_control.call((control_keys.clone(), control_reapply)),
                        if control_reapply { "Reapply" } else { "Revert" }
                    }
                }
            }
        }
    }
}
