//! Filter menu: active filters as removable chips, above collapsible groups of checkbox options.
//!
//! Port losses: the React original's `ref` is not forwarded. Group and option rows are keyed by position, so a row
//! that moves replays its enter animation.

use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};
use crate::pagination::group_digits;

const CSS: Asset = asset!("/assets/filter-menu.css");

/// Keeps the chip box as tall as its content, and animates that height from the second frame on.
const SMOOTH: &str = r#"
const outer = document.getElementById("__OUTER__");
const inner = document.getElementById("__INNER__");
if (outer && inner) {
  const sync = () => {
    outer.style.height = inner.offsetHeight + "px";
  };
  sync();
  new ResizeObserver(sync).observe(inner);
  requestAnimationFrame(() => outer.setAttribute("data-smooth", ""));
}
"#;

#[derive(Clone, PartialEq)]
pub struct FilterOption {
    pub id: String,
    pub label: String,
}

#[derive(Clone, PartialEq)]
pub struct FilterGroup {
    pub id: String,
    pub label: String,
    pub options: Vec<FilterOption>,
    /// Put in front of the option on its chip, e.g. "Score: " gives "Score: 80–90".
    pub chip_prefix: Option<String>,
}

/// Picked option ids per group id.
pub type FilterValue = BTreeMap<String, Vec<String>>;

/// An active filter shown as a chip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chip {
    pub group: String,
    pub option: String,
    pub label: String,
}

/// A row the arrow keys can reach: a group head, or an option of an open group. Indexes into the group list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    Group(usize),
    Option(usize, usize),
}

impl Row {
    pub fn key(self) -> String {
        match self {
            Self::Group(gi) => format!("g:{gi}"),
            Self::Option(gi, oi) => format!("o:{gi}:{oi}"),
        }
    }
}

/// `value` with `option` of `group` toggled on or off. Other groups are kept as they are.
pub fn toggle_option(value: &FilterValue, group: &str, option: &str) -> FilterValue {
    let mut next = value.clone();
    let mut picks = next.get(group).cloned().unwrap_or_default();
    if picks.iter().any(|id| id.as_str() == option) {
        picks.retain(|id| id.as_str() != option);
    } else {
        picks.push(option.to_string());
    }
    next.insert(group.to_string(), picks);
    next
}

/// Distinct picked ids of a group that name one of its options. Ids that match nothing count for nothing.
pub fn picked_count(group: &FilterGroup, value: &FilterValue) -> usize {
    let picks = value.get(&group.id).map(Vec::as_slice).unwrap_or_default();
    picks
        .iter()
        .filter(|id| group.options.iter().any(|o| &o.id == *id))
        .collect::<BTreeSet<_>>()
        .len()
}

/// One chip per picked option, in group order and then option order. A repeated pick shows once.
pub fn chips(groups: &[FilterGroup], value: &FilterValue) -> Vec<Chip> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for group in groups {
        let picks = value.get(&group.id).map(Vec::as_slice).unwrap_or_default();
        for option in &group.options {
            if picks.contains(&option.id) && seen.insert((group.id.as_str(), option.id.as_str())) {
                out.push(Chip {
                    group: group.id.clone(),
                    option: option.id.clone(),
                    label: format!(
                        "{}{}",
                        group.chip_prefix.as_deref().unwrap_or(""),
                        option.label
                    ),
                });
            }
        }
    }
    out
}

/// The keyboard-reachable rows, top to bottom: every group head, and the options of each open group under it.
pub fn rows(groups: &[FilterGroup], open: &BTreeSet<String>) -> Vec<Row> {
    let mut out = Vec::new();
    for (gi, group) in groups.iter().enumerate() {
        out.push(Row::Group(gi));
        if open.contains(&group.id) {
            out.extend((0..group.options.len()).map(|oi| Row::Option(gi, oi)));
        }
    }
    out
}

/// The groups open at the start: those named in `default_open`, or the first group when none is named.
pub fn initial_open(groups: &[FilterGroup], default_open: Option<&[String]>) -> BTreeSet<String> {
    match default_open {
        Some(ids) => ids.iter().cloned().collect(),
        None => groups.first().map(|g| g.id.clone()).into_iter().collect(),
    }
}

/// The index of the group a row belongs to.
fn group_of(row: Row) -> usize {
    match row {
        Row::Group(gi) | Row::Option(gi, _) => gi,
    }
}

/// Filters, grouped and shown as checkboxes. Controlled when `value` is set; otherwise `default_value` starts it.
#[component]
pub fn FilterMenu(
    groups: Vec<FilterGroup>,
    #[props(default)] value: Option<FilterValue>,
    #[props(default)] default_value: FilterValue,
    #[props(default)] on_change: Option<EventHandler<FilterValue>>,
    #[props(default)] default_open: Option<Vec<String>>,
    #[props(default = "Filters".to_string(), into)] title: String,
    #[props(default = true)] hints: bool,
    #[props(default)] class: String,
) -> Element {
    let uid = use_hook(next_id);
    let inner = use_signal(|| default_value);
    let open = use_signal(|| initial_open(&groups, default_open.as_deref()));
    let focus = use_signal(|| None::<String>);
    let ctx = Ctx {
        uid,
        open,
        focus,
        inner,
        controlled: value.is_some(),
        on_change,
    };

    let picked = match &value {
        Some(value) => value.clone(),
        None => inner(),
    };
    let open_now = open();
    let focused = focus();
    let chip_list = chips(&groups, &picked);
    let total_chips = chip_list.len();

    let outer_id = format!("ui-fm-{uid}-chips");
    let inner_id = format!("{outer_id}-inner");
    let groups_id = format!("ui-fm-{uid}-groups");
    let smooth_script = SMOOTH
        .replace("__OUTER__", &outer_id)
        .replace("__INNER__", &inner_id);
    use_effect(move || {
        document::eval(&smooth_script);
    });

    let key_groups = groups.clone();
    let key_picked = picked.clone();
    let on_keys = move |event: KeyboardEvent| {
        let keys = rows(&key_groups, &open());
        let at = keys
            .iter()
            .position(|r| Some(r.key()) == focus())
            .map(|i| i as isize)
            .unwrap_or(-1);
        let last = keys.len().saturating_sub(1) as isize;
        match event.key() {
            Key::ArrowDown | Key::ArrowUp => {
                event.prevent_default();
                let step = if event.key() == Key::ArrowDown { 1 } else { -1 };
                if let Some(row) = keys.get((at + step).clamp(0, last) as usize) {
                    ctx.move_to(row.key());
                }
            }
            Key::Home | Key::End => {
                event.prevent_default();
                let target = if event.key() == Key::Home {
                    keys.first()
                } else {
                    keys.last()
                };
                if let Some(row) = target {
                    ctx.move_to(row.key());
                }
            }
            Key::ArrowRight | Key::ArrowLeft => {
                let Some(row) = keys.get(at.max(0) as usize).copied().filter(|_| at >= 0) else {
                    return;
                };
                event.prevent_default();
                let gi = group_of(row);
                let gid = key_groups[gi].id.clone();
                let is_open = open().contains(&gid);
                if event.key() == Key::ArrowRight && !is_open {
                    ctx.toggle_group(gid);
                } else if event.key() == Key::ArrowLeft {
                    if is_open {
                        ctx.toggle_group(gid);
                    }
                    ctx.move_to(format!("g:{gi}"));
                }
            }
            Key::Enter => {
                let Some(row) = keys.get(at.max(0) as usize).copied().filter(|_| at >= 0) else {
                    return;
                };
                event.prevent_default();
                ctx.activate(&key_groups, &key_picked, row);
            }
            Key::Character(ref ch) if ch.as_str() == " " => {
                let Some(row) = keys.get(at.max(0) as usize).copied().filter(|_| at >= 0) else {
                    return;
                };
                event.prevent_default();
                ctx.activate(&key_groups, &key_picked, row);
            }
            _ => {}
        }
    };

    let title_text = title.clone();
    let clear_disabled = total_chips == 0;

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class: class_list("ui-fm", &class),
            div { class: "ui-fm__card",
                div { class: "ui-fm__head",
                    span { class: "ui-fm__title", dir: "auto", title: "{title_text}", "{title_text}" }
                    button {
                        "type": "button",
                        class: "ui-fm__clear",
                        disabled: clear_disabled,
                        onclick: move |_| ctx.commit(FilterValue::new()),
                        "Clear all"
                    }
                }

                div { id: "{outer_id}", class: "ui-fm__chips-outer",
                    div { id: "{inner_id}", class: "ui-fm__chips", aria_live: "polite",
                        for (i, chip) in chip_list.into_iter().enumerate() {
                            {chip_view(ctx, i, chip, picked.clone())}
                        }
                    }
                }

                div { id: "{groups_id}", class: "ui-fm__groups", onkeydown: on_keys,
                    for (gi, group) in groups.iter().enumerate() {
                        {group_view(ctx, gi, group.clone(), &picked, &open_now, focused.clone())}
                    }
                }
            }

            if hints {
                div { class: "ui-fm__hints", "aria-hidden": "true",
                    span { class: "ui-fm__hint",
                        kbd {
                            svg { view_box: "0 0 24 24", path { d: "M12 5v14m0 0-5-5m5 5 5-5" } }
                        }
                        kbd {
                            svg { view_box: "0 0 24 24", path { d: "M12 19V5m0 0-5 5m5-5 5 5" } }
                        }
                        "To navigate"
                    }
                    span { class: "ui-fm__hint",
                        kbd {
                            svg { view_box: "0 0 24 24", path { d: "M18 6v6.5a2 2 0 0 1-2 2H7m0 0 3.5-3.5M7 14.5 10.5 18" } }
                        }
                        "To select"
                    }
                }
            }
        }
    }
}

/// The state every row needs, as signals: copied into each handler.
#[derive(Clone, Copy)]
struct Ctx {
    uid: usize,
    open: Signal<BTreeSet<String>>,
    focus: Signal<Option<String>>,
    inner: Signal<FilterValue>,
    controlled: bool,
    on_change: Option<EventHandler<FilterValue>>,
}

impl Ctx {
    fn commit(self, next: FilterValue) {
        if !self.controlled {
            let mut inner = self.inner;
            inner.set(next.clone());
        }
        if let Some(handler) = self.on_change {
            handler.call(next);
        }
    }

    fn toggle_group(self, group: String) {
        let mut open = self.open;
        let mut set = open.write();
        if !set.remove(&group) {
            set.insert(group);
        }
    }

    fn set_focus(self, key: String) {
        let mut focus = self.focus;
        focus.set(Some(key));
    }

    fn move_to(self, key: String) {
        document::eval(&format!(
            "document.getElementById({:?})?.querySelector('[data-key=\"{key}\"]')?.focus();",
            format!("ui-fm-{}-groups", self.uid)
        ));
        self.set_focus(key);
    }

    /// Enter or Space on a row: a group opens or closes, an option toggles.
    fn activate(self, groups: &[FilterGroup], picked: &FilterValue, row: Row) {
        match row {
            Row::Group(gi) => self.toggle_group(groups[gi].id.clone()),
            Row::Option(gi, oi) => {
                let group = &groups[gi];
                let option = &group.options[oi];
                self.commit(toggle_option(picked, &group.id, &option.id));
            }
        }
    }
}

fn chip_view(ctx: Ctx, i: usize, chip: Chip, picked: FilterValue) -> Element {
    let label = chip.label.clone();
    rsx! {
        span { key: "{i}", class: "ui-fm__chip",
            span { class: "ui-fm__chip-label", dir: "auto", title: "{label}", "{label}" }
            button {
                "type": "button",
                aria_label: "Remove {label}",
                onclick: move |_| ctx.commit(toggle_option(&picked, &chip.group, &chip.option)),
                svg { view_box: "0 0 24 24", "aria-hidden": "true",
                    path { d: "M6.5 6.5l11 11M17.5 6.5l-11 11" }
                }
            }
        }
    }
}

fn group_view(
    ctx: Ctx,
    gi: usize,
    group: FilterGroup,
    picked: &FilterValue,
    open: &BTreeSet<String>,
    focused: Option<String>,
) -> Element {
    let is_open = open.contains(&group.id);
    let count = picked_count(&group, picked);
    let head_key = Row::Group(gi).key();
    let focus_key = head_key.clone();
    let head_tab =
        if focused.as_deref() == Some(head_key.as_str()) || (focused.is_none() && gi == 0) {
            "0"
        } else {
            "-1"
        };
    let panel_id = format!("ui-fm-{}-{gi}", ctx.uid);
    let picked_label = match count {
        0 => String::new(),
        1 => "1 filter".to_string(),
        n => format!("{n} filters"),
    };
    let total = group_digits(group.options.len());
    let gid = group.id.clone();
    let label = group.label.clone();
    let options: Vec<(usize, FilterOption, bool)> = group
        .options
        .iter()
        .enumerate()
        .map(|(oi, o)| {
            let on = picked.get(&group.id).is_some_and(|ids| ids.contains(&o.id));
            (oi, o.clone(), on)
        })
        .collect();

    rsx! {
        section { key: "{group.id}#{gi}", class: "ui-fm__group", "data-open": is_open.then_some(""),
            button {
                "type": "button",
                class: "ui-fm__group-head",
                "data-key": "{head_key}",
                tabindex: "{head_tab}",
                aria_expanded: if is_open { "true" } else { "false" },
                aria_controls: "{panel_id}",
                onfocus: move |_| ctx.set_focus(focus_key.clone()),
                onclick: move |_| ctx.toggle_group(gid.clone()),
                svg { class: "ui-fm__chevron", view_box: "0 0 24 24", "aria-hidden": "true",
                    path { d: "M6.5 9.5 12 15l5.5-5.5" }
                }
                span { class: "ui-fm__group-label", dir: "auto", title: "{label}", "{label}" }
                span { class: "ui-fm__group-count", "{total}" }
                span { class: "ui-fm__group-picked", aria_live: "polite", "{picked_label}" }
            }
            div { id: "{panel_id}", class: "ui-fm__panel", role: "group", aria_label: "{label}",
                div { class: "ui-fm__panel-inner",
                    for (oi, option, on) in options {
                        {option_view(ctx, gi, oi, &group.id, option, on, is_open, focused.clone(), picked.clone())}
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn option_view(
    ctx: Ctx,
    gi: usize,
    oi: usize,
    group_id: &str,
    option: FilterOption,
    on: bool,
    is_open: bool,
    focused: Option<String>,
    picked: FilterValue,
) -> Element {
    let key = Row::Option(gi, oi).key();
    let focus_key = key.clone();
    let tab = if is_open && focused.as_deref() == Some(key.as_str()) {
        "0"
    } else {
        "-1"
    };
    let group_id = group_id.to_string();
    let option_id = option.id.clone();
    let label = option.label.clone();
    rsx! {
        button {
            key: "{oi}-{option.id}",
            "type": "button",
            role: "checkbox",
            aria_checked: "{on}",
            class: "ui-fm__option",
            "data-key": "{key}",
            tabindex: "{tab}",
            onfocus: move |_| ctx.set_focus(focus_key.clone()),
            onclick: move |_| ctx.commit(toggle_option(&picked, &group_id, &option_id)),
            span { class: "ui-fm__box", "aria-hidden": "true",
                svg { view_box: "0 0 24 24",
                    path { d: "m6 12.5 4 4 8-9" }
                }
            }
            span { class: "ui-fm__option-label", dir: "auto", title: "{label}", "{label}" }
        }
    }
}
