//! Change log timeline: the recent changes on one branch as a vertical list,
//! with merges that fold away the changes they brought in and an in-progress
//! row for a change that is still being written.
//!
//! Design choice: `changed_at` is epoch seconds (or preformatted text), since
//! there is no `Date` on wasm32 and no date dependency. The full time is
//! formatted as UTC; the relative label needs the caller's `now_secs`.
//! Design choice: `text_overrides` is not implemented; the English strings are
//! constants. Localisation belongs with the caller's copy.
//! Design choice: name tooltips use the native `title` attribute.
//! Design choice: the flattened row list is the single source of keyboard order.
//! Browser checks: the rise of rows on first display, the saved check draw and
//! line fill, the travelling light on the in-progress marker, the merge side
//! branch, and the truncation of long chips.

use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, copy_to_clipboard, next_id};

const CSS: Asset = asset!("/assets/change-log-timeline.css");

pub const BUILT_IN_KINDS: [&str; 9] = [
    "new_capability",
    "correction",
    "restructure",
    "speed",
    "documentation",
    "verification",
    "housekeeping",
    "release",
    "merge",
];

#[derive(Clone, PartialEq, Debug)]
pub struct ChangeWriter {
    pub writer_name: String,
    pub is_agent: bool,
    pub lane_color: Option<String>,
    pub writer_mark: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Timestamp {
    /// Epoch seconds.
    Epoch(i64),
    /// Preformatted text, shown as given.
    Text(String),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Change {
    pub change_id: String,
    pub change_message: String,
    pub change_author: ChangeWriter,
    pub co_author: Option<ChangeWriter>,
    pub changed_at: Option<Timestamp>,
    pub change_kind: Option<String>,
    pub file_count: Option<u32>,
    pub lines_added: Option<u32>,
    pub lines_removed: Option<u32>,
    pub change_body: Option<String>,
    pub merged_changes: Vec<Change>,
    pub in_progress: bool,
}

/// Every visible word the timeline shows that is not a name or a time. Unset
/// fields keep the English defaults.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct ChangeLogText {
    pub empty: Option<String>,
    pub in_progress: Option<String>,
    pub with: Option<String>,
    pub copy: Option<String>,
    pub saved: Option<String>,
    pub changes: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct KindStyle {
    pub label: Option<String>,
    pub color: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KindTone {
    Neutral,
    BuiltIn,
    Custom,
}

impl KindTone {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::BuiltIn => "builtin",
            Self::Custom => "custom",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum MessagePart {
    Text(String),
    Name { name: String, is_file: bool },
}

#[derive(Clone, PartialEq, Debug)]
pub struct FlatRow {
    pub change: Change,
    pub depth: usize,
    pub is_merge: bool,
    pub expanded: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RowKey {
    Up,
    Down,
    Home,
    End,
}

/// Human label for a kind key: the override, else the key with underscores as spaces.
pub fn kind_label(key: &str, overrides: &BTreeMap<String, KindStyle>) -> String {
    if let Some(label) = overrides.get(key).and_then(|style| style.label.clone()) {
        return label;
    }
    let spaced = key.replace('_', " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// A kind is built in, customised with a colour, or neutral when it has neither.
pub fn kind_tone(key: &str, overrides: &BTreeMap<String, KindStyle>) -> KindTone {
    if overrides
        .get(key)
        .and_then(|style| style.color.as_ref())
        .is_some()
    {
        KindTone::Custom
    } else if BUILT_IN_KINDS.contains(&key) {
        KindTone::BuiltIn
    } else {
        KindTone::Neutral
    }
}

/// Splits a message on backticks. Odd segments are names; a name with an
/// extension is a file, anything else a branch.
pub fn message_parts(message: &str) -> Vec<MessagePart> {
    let mut parts = Vec::new();
    for (index, segment) in message.split('`').enumerate() {
        if segment.is_empty() {
            continue;
        }
        if index % 2 == 1 {
            parts.push(MessagePart::Name {
                name: segment.to_string(),
                is_file: looks_like_file(segment),
            });
        } else {
            parts.push(MessagePart::Text(segment.to_string()));
        }
    }
    parts
}

pub fn looks_like_file(name: &str) -> bool {
    if name.contains(' ') {
        return false;
    }
    match name.rsplit_once('.') {
        Some((stem, ext)) => {
            !stem.is_empty()
                && !ext.is_empty()
                && ext.len() <= 8
                && ext.chars().all(|c| c.is_ascii_alphanumeric())
        }
        None => false,
    }
}

/// Coarse age label in the unit that fits; the caller supplies `now_secs`.
pub fn relative_time(now_secs: i64, at_secs: i64) -> String {
    let delta = (now_secs - at_secs).max(0);
    if delta < 60 {
        "just now".to_string()
    } else if delta < 3_600 {
        format!("{} min ago", delta / 60)
    } else if delta < 86_400 {
        format!("{} h ago", delta / 3_600)
    } else if delta < 7 * 86_400 {
        format!("{} d ago", delta / 86_400)
    } else {
        format!("{} w ago", delta / (7 * 86_400))
    }
}

/// `2026-11-20 14:03 UTC`, from epoch seconds.
pub fn format_utc(epoch: i64) -> String {
    let days = epoch.div_euclid(86_400);
    let secs = epoch.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02} UTC",
        secs / 3_600,
        (secs % 3_600) / 60
    )
}

/// Days since 1970-01-01 to (year, month, day), proleptic Gregorian.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// Rows in display order: each change, then its merged changes when the merge is open.
pub fn flatten(changes: &[Change], expanded: &BTreeSet<String>) -> Vec<FlatRow> {
    let mut out = Vec::new();
    push_rows(changes, 0, expanded, &mut out);
    out
}

fn push_rows(
    changes: &[Change],
    depth: usize,
    expanded: &BTreeSet<String>,
    out: &mut Vec<FlatRow>,
) {
    for change in changes {
        let is_merge = !change.merged_changes.is_empty();
        let open = is_merge && expanded.contains(&change.change_id);
        out.push(FlatRow {
            change: change.clone(),
            depth,
            is_merge,
            expanded: open,
        });
        if open {
            push_rows(&change.merged_changes, depth + 1, expanded, out);
        }
    }
}

/// Moves focus through rows; `None` when there are no rows.
pub fn next_row(len: usize, current: Option<usize>, key: RowKey) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match key {
        RowKey::Home => 0,
        RowKey::End => len - 1,
        RowKey::Up => current.map_or(0, |i| i.saturating_sub(1)),
        RowKey::Down => current.map_or(0, |i| (i + 1).min(len - 1)),
    })
}

fn writers_label(change: &Change) -> String {
    match &change.co_author {
        Some(co) => format!(
            "{} with {}",
            change.change_author.writer_name, co.writer_name
        ),
        None => change.change_author.writer_name.clone(),
    }
}

fn time_text(at: &Option<Timestamp>, now_secs: i64) -> (String, String, String) {
    match at {
        Some(Timestamp::Epoch(secs)) => (
            relative_time(now_secs, *secs),
            secs.to_string(),
            format_utc(*secs),
        ),
        Some(Timestamp::Text(text)) => (text.clone(), text.clone(), text.clone()),
        None => (String::new(), String::new(), String::new()),
    }
}

fn focus_id(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

fn row_dom_id(base: &str, index: usize) -> String {
    format!("{base}-row-{index}")
}

/// The timeline of changes on a branch.
#[component]
pub fn ChangeLogTimeline(
    #[props(default)] change_list: Vec<Change>,
    #[props(default)] branch_names: Vec<String>,
    #[props(default, into)] current_branch: String,
    #[props(default)] on_branch_switch: Option<EventHandler<String>>,
    #[props(default)] open_change_id: Option<String>,
    #[props(default)] on_open_change: Option<EventHandler<Option<String>>>,
    #[props(default)] expanded_merge_ids: Option<Vec<String>>,
    #[props(default)] on_merge_expand_change: Option<EventHandler<Vec<String>>>,
    #[props(default)] has_older: bool,
    #[props(default = 420.0)] scroll_height: f64,
    #[props(default)] now_secs: i64,
    #[props(default)] name_tooltips: BTreeMap<String, String>,
    #[props(default)] kind_overrides: BTreeMap<String, KindStyle>,
    #[props(default)] release_tag: Option<String>,
    #[props(default)] text_overrides: Option<ChangeLogText>,
    #[props(default, into)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-log-{}", next_id()));
    let labels = text_overrides.unwrap_or_default();
    let saved_word = labels.saved.clone().unwrap_or_else(|| "Saved".to_string());
    let mut internal_open = use_signal(|| Option::<String>::None);
    let mut internal_expanded = use_signal(BTreeSet::<String>::new);
    let mut branch_menu = use_signal(|| false);
    let mut focused = use_signal(|| Option::<usize>::None);
    let mut seen_progress = use_signal(BTreeSet::<String>::new);
    let mut announce = use_signal(String::new);

    let open_id = if on_open_change.is_some() {
        open_change_id.clone()
    } else {
        internal_open()
    };
    let expanded_set: BTreeSet<String> = match &expanded_merge_ids {
        Some(ids) => ids.iter().cloned().collect(),
        None => internal_expanded(),
    };
    let rows = flatten(&change_list, &expanded_set);

    let progress_for_effect = change_list.clone();

    use_effect(use_reactive((&progress_for_effect,), move |(changes,)| {
        let before = seen_progress.peek().clone();
        let now: BTreeSet<String> = changes
            .iter()
            .filter(|c| c.in_progress)
            .map(|c| c.change_id.clone())
            .collect();
        let saved: Vec<String> = changes
            .iter()
            .filter(|c| before.contains(&c.change_id) && !c.in_progress)
            .map(|c| c.change_message.clone())
            .collect();
        if let Some(message) = saved.first() {
            announce.set(format!("{saved_word}: {message}"));
        }
        seen_progress.set(now);
    }));
    let mut set_open = move |value: Option<String>| {
        if on_open_change.is_none() {
            internal_open.set(value.clone());
        }
        if let Some(handler) = on_open_change {
            handler.call(value);
        }
    };

    let expanded_controlled = expanded_merge_ids.is_some();
    let mut set_expanded = move |next: BTreeSet<String>| {
        if !expanded_controlled {
            internal_expanded.set(next.clone());
        }
        if let Some(handler) = on_merge_expand_change {
            handler.call(next.into_iter().collect());
        }
    };

    let toggle_row = {
        let rows = rows.clone();
        let expanded_now = expanded_set.clone();
        let open_now = open_id.clone();
        move |index: usize| {
            let Some(row) = rows.get(index) else {
                return;
            };
            let id = row.change.change_id.clone();
            if row.is_merge {
                let mut next = expanded_now.clone();
                if !next.remove(&id) {
                    next.insert(id);
                }
                set_expanded(next);
            } else if open_now.as_deref() == Some(id.as_str()) {
                set_open(None);
            } else {
                set_open(Some(id));
            }
        }
    };

    let key_handler = {
        let rows = rows.clone();
        let open_for_keys = open_id.clone();
        let row_base = base.clone();
        let expanded_keys = expanded_set.clone();
        move |event: KeyboardEvent| {
            let key = match event.key() {
                Key::ArrowDown => RowKey::Down,
                Key::ArrowUp => RowKey::Up,
                Key::Home => RowKey::Home,
                Key::End => RowKey::End,
                Key::Escape => {
                    event.prevent_default();
                    if let Some(index) = focused() {
                        if let Some(row) = rows.get(index) {
                            if row.is_merge && row.expanded {
                                let mut next = expanded_keys.clone();
                                next.remove(&row.change.change_id);
                                set_expanded(next);
                                focus_id(&row_dom_id(&row_base, index));
                                return;
                            }
                            if open_for_keys.as_deref() == Some(row.change.change_id.as_str()) {
                                set_open(None);
                                focus_id(&row_dom_id(&row_base, index));
                            }
                        }
                    }
                    return;
                }
                _ => return,
            };
            event.prevent_default();
            if let Some(index) = next_row(rows.len(), focused(), key) {
                focus_id(&row_dom_id(&row_base, index));
            }
        }
    };

    let branch_label = current_branch.clone();
    let count = change_list.len();
    let count_text = format!(
        "{count} {}",
        labels
            .changes
            .clone()
            .unwrap_or_else(|| "changes".to_string())
    );
    let empty_text = labels
        .empty
        .clone()
        .unwrap_or_else(|| "No changes on this branch yet".to_string());
    let writing_text = labels
        .in_progress
        .clone()
        .unwrap_or_else(|| "Writing…".to_string());
    let with_word = labels.with.clone().unwrap_or_else(|| "with".to_string());
    let copy_text = labels.copy.clone().unwrap_or_else(|| "Copy id".to_string());
    let root_cls = class_list("kit-log", &root_class);
    let switcher = branch_names.len() >= 2;
    let menu_open = branch_menu();
    let menu_expanded = if menu_open { "true" } else { "false" };
    let max_height = format!("max-height: {scroll_height}px;");
    let fade_class = if has_older {
        "kit-log__fade is-visible"
    } else {
        "kit-log__fade"
    };
    let announcement = announce();
    let switcher_id = format!("{base}-switch");
    let list_label = format!("Changes on {current_branch}");
    let release = release_tag.clone();
    let empty = rows.is_empty();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: root_cls,
            header { class: "kit-log__head",
                if switcher {
                    div { class: "kit-log__switch",
                        button {
                            id: "{switcher_id}",
                            r#type: "button",
                            class: "kit-log__branch",
                            "aria-haspopup": "menu",
                            "aria-expanded": "{menu_expanded}",
                            onclick: move |_| branch_menu.set(!branch_menu()),
                            "{branch_label}"
                        }
                        if menu_open {
                            ul {
                                class: "kit-log__menu",
                                role: "menu",
                                onkeydown: {
                                    let switcher_id = switcher_id.clone();
                                    move |event: KeyboardEvent| {
                                        if matches!(event.key(), Key::Escape) {
                                            branch_menu.set(false);
                                            focus_id(&switcher_id);
                                        }
                                    }
                                },
                                for branch in branch_names.clone() {
                                    {
                                        let checked = if branch == current_branch { "true" } else { "false" };
                                        let picked = branch.clone();
                                        rsx! {
                                            li { key: "{branch}", role: "none",
                                                button {
                                                    r#type: "button",
                                                    role: "menuitemradio",
                                                    "aria-checked": "{checked}",
                                                    onclick: move |_| {
                                                        branch_menu.set(false);
                                                        set_open(None);
                                                        if let Some(handler) = on_branch_switch {
                                                            handler.call(picked.clone());
                                                        }
                                                    },
                                                    "{branch}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    span { class: "kit-log__branch is-static", "{branch_label}" }
                }
                span { class: "kit-log__count", "{count_text}" }
                if let Some(tag) = release {
                    span { class: "kit-log__release", "{tag}" }
                }
            }
            div { class: "kit-log__scroll", style: "{max_height}",
                if empty {
                    p { class: "kit-log__empty", "{empty_text}" }
                }
                ol {
                    class: "kit-log__list",
                    role: "list",
                    "aria-label": "{list_label}",
                    onkeydown: key_handler,
                    for (index, row) in rows.iter().enumerate() {
                        {
                            let change = row.change.clone();
                            let row_id = row_dom_id(&base, index);
                            let depth = row.depth.to_string();
                            let kind_key = change.change_kind.clone().unwrap_or_default();
                            let tone = if kind_key.is_empty() {
                                KindTone::Neutral
                            } else {
                                kind_tone(&kind_key, &kind_overrides)
                            };
                            let kind_text = if kind_key.is_empty() {
                                String::new()
                            } else {
                                kind_label(&kind_key, &kind_overrides)
                            };
                            let kind_style = kind_overrides
                                .get(&kind_key)
                                .and_then(|style| style.color.clone())
                                .map(|color| format!("--kit-kind: {color};"))
                                .unwrap_or_default();
                            let (relative, time_attr, full_time) = time_text(&change.changed_at, now_secs);
                            let is_open = open_id.as_deref() == Some(change.change_id.as_str());
                            let open_attr = if is_open { "true" } else { "false" };
                            let writers = writers_label(&change);
                            let parts = message_parts(&change.change_message);
                            let message_plain = change.change_message.clone();
                            let row_label = format!("{message_plain}, {writers}, {relative}");
                            let mut click = toggle_row.clone();
                            let merged_count = change.merged_changes.len();
                            let merge_expanded = if row.expanded { "true" } else { "false" };
                            let merge_label = if merged_count == 1 { "Merged 1 change".to_string() } else { format!("Merged {merged_count} changes") };
                            let merge_name = format!("{merge_label}: {message_plain}");
                            let id_text = change.change_id.clone();
                            let copy_id = change.change_id.clone();
                            let files = change.file_count.map(|n| format!("{n} files"));
                            let added = change.lines_added.map(|n| format!("+{n}"));
                            let removed = change.lines_removed.map(|n| format!("−{n}"));
                            let body = change.change_body.clone();
                            let author = change.change_author.clone();
                            let author_name = author.writer_name.clone();
                            let co_label = change
                                .co_author
                                .as_ref()
                                .map(|co| format!("{with_word} {}", co.writer_name));
                            let author_style = author
                                .lane_color
                                .clone()
                                .map(|c| format!("--kit-lane: {c};"))
                                .unwrap_or_default();
                            let author_mark = author
                                .writer_mark
                                .clone()
                                .unwrap_or_else(|| author.writer_name.chars().take(1).collect());
                            rsx! {
                                li {
                                    key: "{row_id}",
                                    class: "kit-log__item",
                                    "data-depth": "{depth}",
                                    "data-kind-tone": tone.as_str(),
                                    "data-change-id": "{id_text}",
                                    if change.in_progress {
                                        div { class: "kit-log__row is-progress", id: "{row_id}", tabindex: "-1",
                                            span { class: "kit-log__marker is-progress", "aria-hidden": "true" }
                                            span { class: "kit-log__message", "{message_plain}" }
                                            span { class: "kit-log__progress-label", "{writing_text}" }
                                        }
                                    } else if row.is_merge {
                                        button {
                                            id: "{row_id}",
                                            r#type: "button",
                                            class: "kit-log__row is-merge",
                                            "aria-expanded": "{merge_expanded}",
                                            "aria-label": "{merge_name}",
                                            onfocus: move |_| focused.set(Some(index)),
                                            onclick: move |_| click(index),
                                            span { class: "kit-log__marker is-merge", "aria-hidden": "true" }
                                            span { class: "kit-log__message", "{message_plain}" }
                                            span { class: "kit-log__merge-count", "{merge_label}" }
                                        }
                                    } else {
                                        button {
                                            id: "{row_id}",
                                            r#type: "button",
                                            class: "kit-log__row",
                                            "aria-expanded": "{open_attr}",
                                            "aria-label": "{row_label}",
                                            onfocus: move |_| focused.set(Some(index)),
                                            onclick: move |_| click(index),
                                            span { class: "kit-log__marker", "aria-hidden": "true" }
                                            span { class: "kit-log__message",
                                                for part in parts.clone() {
                                                    {
                                                        match part {
                                                            MessagePart::Text(text) => rsx! { span { class: "kit-log__text", "{text}" } },
                                                            MessagePart::Name { name, is_file } => {
                                                                let tip = name_tooltips.get(&name).cloned().unwrap_or_else(|| name.clone());
                                                                let kind = if is_file { "file" } else { "branch" };
                                                                rsx! {
                                                                    span {
                                                                        class: "kit-log__chip",
                                                                        "data-name-kind": kind,
                                                                        title: "{tip}",
                                                                        span { class: "kit-log__chip-text", "{name}" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            if !kind_text.is_empty() {
                                                span { class: "kit-log__kind", style: "{kind_style}", "{kind_text}" }
                                            }
                                            time { class: "kit-log__time", "datetime": "{time_attr}", "{relative}" }
                                        }
                                    }
                                    if is_open && !row.is_merge {
                                        div { class: "kit-log__details",
                                            div { class: "kit-log__writers",
                                                span { class: "kit-log__author", style: "{author_style}",
                                                    if author.is_agent {
                                                        span { class: "kit-log__agent", "aria-hidden": "true", "{author_mark}" }
                                                    }
                                                    "{author_name}"
                                                }
                                                if let Some(co_label) = co_label {
                                                    span { class: "kit-log__author is-co", "{co_label}" }
                                                }
                                            }
                                            p { class: "kit-log__meta",
                                                time { "datetime": "{time_attr}", "{full_time}" }
                                                code { class: "kit-log__id", "{id_text}" }
                                                if let Some(files) = files {
                                                    span { "{files}" }
                                                }
                                                if let Some(added) = added {
                                                    span { class: "kit-log__add", "{added}" }
                                                }
                                                if let Some(removed) = removed {
                                                    span { class: "kit-log__remove", "{removed}" }
                                                }
                                            }
                                            if let Some(body) = body {
                                                p { class: "kit-log__body", "{body}" }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "kit-log__copy",
                                                onclick: move |_| copy_to_clipboard(&copy_id),
                                                "{copy_text}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "{fade_class}", "aria-hidden": "true" }
            div { class: "kit-log__sr", role: "status", "aria-live": "polite", "{announcement}" }
        }
    }
}
