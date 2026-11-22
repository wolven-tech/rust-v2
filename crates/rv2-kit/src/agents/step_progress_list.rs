//! Step progress list: the ordered steps an agent works through on one task, with the
//! step in progress open to its sub-steps, and an approval gate where the person decides.
//!
//! Design choice: `wording` is one full struct; Rust has no partial struct, so callers
//! spread `..Default::default()` for the keys they leave alone.
//! Design choice: `open_keys` is controlled when given; otherwise the list keeps its own
//! set, seeded with the in-progress item.
//! Design choice: an item that needs the person opens its Confirm and Refuse panel from
//! its row. Sub-steps of that item open from the same row only when it has no pending
//! approval.
//! Design choice: the polite region announces the awaiting item if any, else the count.
//! Browser checks: the branch line animates with the rows it connects; the in-progress
//! mark turns, its note shimmers and its timer counts up; Enter on an awaiting row
//! focuses Confirm, Esc returns focus to the row; reduced motion applies state at once.

use dioxus::prelude::*;

use crate::agents::single_run_status_card::{format_mm_ss, initial_of};
use crate::internal::{KIT_CSS, focus_after_frame, next_id, use_now_ms};

const CSS: Asset = asset!("/assets/step-progress-list.css");

/// Sub-item rows shown when the list folds.
pub const FOLD_WINDOW: usize = 5;
/// Sub-item lists longer than this fold to `FOLD_WINDOW` rows.
pub const FOLD_THRESHOLD: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemState {
    Finished,
    InProgress,
    Queued,
    AwaitingApproval,
    AwaitingPerson,
    Errored,
    Dropped,
}

impl ItemState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Finished => "finished",
            Self::InProgress => "in_progress",
            Self::Queued => "queued",
            Self::AwaitingApproval => "awaiting_approval",
            Self::AwaitingPerson => "awaiting_person",
            Self::Errored => "errored",
            Self::Dropped => "dropped",
        }
    }

    fn mark(self) -> &'static str {
        match self {
            Self::Finished => "✓",
            Self::InProgress => "◌",
            Self::Queued => "○",
            Self::AwaitingApproval => "◇",
            Self::AwaitingPerson => "!",
            Self::Errored => "✕",
            Self::Dropped => "–",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubState {
    Finished,
    InProgress,
    Queued,
}

impl SubState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Finished => "finished",
            Self::InProgress => "in_progress",
            Self::Queued => "queued",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SubItem {
    pub sub_key: String,
    pub text: String,
    pub sub_state: Option<SubState>,
    pub fact_text: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ToolBadge {
    pub display_name: String,
    pub mark_source: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ProgressItem {
    pub item_key: String,
    pub title_text: String,
    pub item_state: ItemState,
    pub took_ms: Option<u64>,
    pub began_at_ms: Option<u64>,
    pub live_note: Option<String>,
    pub needs_approval: bool,
    pub tool_badge: Option<ToolBadge>,
    pub approval_request: Option<String>,
    pub sub_items: Vec<SubItem>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AgentBadge {
    pub display_name: String,
    pub mark_source: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ProgressWording {
    pub list_heading_fallback: String,
    pub in_progress: String,
    pub approval: String,
    pub awaiting: String,
    pub errored: String,
    pub dropped: String,
    pub finished: String,
    pub queued: String,
    pub empty: String,
    pub untitled_item: String,
    /// `{n}` is replaced with the count.
    pub earlier_count: String,
    /// `{n}` is replaced with the count.
    pub later_count: String,
    pub show_fewer: String,
    pub confirm: String,
    pub refuse: String,
}

impl Default for ProgressWording {
    fn default() -> Self {
        Self {
            list_heading_fallback: "Steps".to_string(),
            in_progress: "Running".to_string(),
            approval: "Needs you".to_string(),
            awaiting: "Needs you".to_string(),
            errored: "Failed".to_string(),
            dropped: "Skipped".to_string(),
            finished: "Done".to_string(),
            queued: "Queued".to_string(),
            empty: "No steps yet".to_string(),
            untitled_item: "Untitled step".to_string(),
            earlier_count: "{n} earlier".to_string(),
            later_count: "{n} later".to_string(),
            show_fewer: "Show fewer".to_string(),
            confirm: "Confirm".to_string(),
            refuse: "Refuse".to_string(),
        }
    }
}

/// "n of m steps finished".
pub fn count_line(items: &[ProgressItem]) -> String {
    let finished = items
        .iter()
        .filter(|item| item.item_state == ItemState::Finished)
        .count();
    format!("{finished} of {} steps finished", items.len())
}

/// The word for an item's state, when it has one. Finished items carry a duration instead.
pub fn state_word(state: ItemState, wording: &ProgressWording) -> Option<String> {
    match state {
        ItemState::Finished => None,
        ItemState::InProgress => Some(wording.in_progress.clone()),
        ItemState::Queued => Some(wording.queued.clone()),
        ItemState::AwaitingApproval => Some(wording.approval.clone()),
        ItemState::AwaitingPerson => Some(wording.awaiting.clone()),
        ItemState::Errored => Some(wording.errored.clone()),
        ItemState::Dropped => Some(wording.dropped.clone()),
    }
}

/// The sub-items drawn when folded: a window of `FOLD_WINDOW` around the in-progress one.
pub fn fold_window(len: usize, focus: Option<usize>) -> (usize, usize) {
    if len <= FOLD_THRESHOLD {
        return (0, len);
    }
    let center = focus.unwrap_or(0);
    let start = center
        .saturating_sub(FOLD_WINDOW / 2)
        .min(len - FOLD_WINDOW);
    (start, start + FOLD_WINDOW)
}

/// Items open by default: the one in progress.
pub fn default_open_keys(items: &[ProgressItem]) -> Vec<String> {
    items
        .iter()
        .filter(|item| item.item_state == ItemState::InProgress)
        .map(|item| item.item_key.clone())
        .collect()
}

/// Accessible name of an item: title, state and time in one string.
pub fn item_name(title: &str, state_text: &str, time: &str) -> String {
    [title, state_text, time]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Clone, PartialEq, Debug)]
struct SubView {
    key: String,
    text: String,
    state: &'static str,
    fact: Option<String>,
    current: bool,
}

#[component]
pub fn StepProgressList(
    #[props(default)] list_heading: Option<String>,
    #[props(default)] agent_badge: Option<AgentBadge>,
    #[props(default)] progress_items: Vec<ProgressItem>,
    #[props(default)] open_keys: Option<Vec<String>>,
    #[props(default)] on_open_change: Option<EventHandler<Vec<String>>>,
    #[props(default)] on_confirm: Option<EventHandler<ProgressItem>>,
    #[props(default)] on_refuse: Option<EventHandler<ProgressItem>>,
    #[props(default)] wording: ProgressWording,
    #[props(default)] clock_ms: Option<u64>,
    #[props(default)] class_name: String,
) -> Element {
    let id = use_hook(|| format!("kit-spl-{}", next_id()));
    let mut open_local = use_signal(|| default_open_keys(&progress_items));
    let mut expanded_subs = use_signal(Vec::<String>::new);
    let mut panel = use_signal(|| None::<String>);

    let now = use_now_ms(clock_ms, 1000);
    let controlled = open_keys.is_some();
    let open_now = open_keys.unwrap_or_else(|| open_local());
    let on_toggle = {
        let open_now = open_now.clone();
        let on_open_change = on_open_change;
        Callback::new(move |key: String| {
            let mut next = open_now.clone();
            match next.iter().position(|k| *k == key) {
                Some(at) => {
                    next.remove(at);
                }
                None => next.push(key),
            }
            if !controlled {
                open_local.set(next.clone());
            }
            if let Some(handler) = &on_open_change {
                handler.call(next);
            }
        })
    };
    let on_expand = Callback::new(move |key: String| {
        let mut next = expanded_subs();
        match next.iter().position(|k| *k == key) {
            Some(at) => {
                next.remove(at);
            }
            None => next.push(key),
        }
        expanded_subs.set(next);
    });
    let on_panel = Callback::new(move |key: String| {
        let next = if panel().as_deref() == Some(key.as_str()) {
            None
        } else {
            Some(key)
        };
        panel.set(next);
    });

    let count = count_line(&progress_items);
    let awaiting_title = progress_items
        .iter()
        .find(|item| item.item_state == ItemState::AwaitingPerson)
        .map(|item| {
            let title = if item.title_text.is_empty() {
                wording.untitled_item.clone()
            } else {
                item.title_text.clone()
            };
            format!("{title}: {}", wording.awaiting)
        });
    let announce = awaiting_title.unwrap_or_else(|| count.clone());
    let heading = list_heading.clone();
    let heading_label = heading
        .clone()
        .unwrap_or_else(|| wording.list_heading_fallback.clone());
    let heading_id = format!("{id}-heading");
    let rows: Vec<(String, String)> = progress_items
        .iter()
        .map(|item| {
            (
                item.item_key.clone(),
                format!("{id}-item-{}", item.item_key),
            )
        })
        .collect();
    let class = format!("kit-spl {class_name}");
    let empty_text = wording.empty.clone();
    let badge_name = agent_badge.as_ref().map(|badge| badge.display_name.clone());
    let badge_mark = agent_badge
        .as_ref()
        .map(|badge| initial_of(&badge.display_name));

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}",
            if heading.is_some() || badge_name.is_some() {
                header { class: "kit-spl__head",
                    if let Some(title) = heading {
                        h3 { id: "{heading_id}", class: "kit-spl__heading", {title} }
                    }
                    span { class: "kit-spl__count", {count} }
                    if let (Some(name), Some(mark)) = (badge_name, badge_mark) {
                        span { class: "kit-spl__badge",
                            span { class: "kit-spl__badge-mark", "aria-hidden": "true", {mark} }
                            span { {name} }
                        }
                    }
                }
            }
            if progress_items.is_empty() {
                p { class: "kit-spl__empty", {empty_text} }
            } else {
                ol {
                    class: "kit-spl__list",
                    "aria-label": "{heading_label}",
                    for (key, row_id) in rows {
                        ProgressStep {
                            key: "{key}",
                            row_id: row_id,
                            item: progress_items.iter().find(|item| item.item_key == key).cloned().unwrap_or(ProgressItem {
                                item_key: key.clone(),
                                title_text: String::new(),
                                item_state: ItemState::Queued,
                                took_ms: None,
                                began_at_ms: None,
                                live_note: None,
                                needs_approval: false,
                                tool_badge: None,
                                approval_request: None,
                                sub_items: Vec::new(),
                            }),
                            wording: wording.clone(),
                            open: open_now.contains(&key),
                            subs_all: expanded_subs().contains(&key),
                            panel_open: panel().as_deref() == Some(key.as_str()),
                            now_ms: now,
                            on_toggle: on_toggle,
                            on_expand: on_expand,
                            on_panel: on_panel,
                            on_confirm: on_confirm,
                            on_refuse: on_refuse,
                        }
                    }
                }
            }
            span { class: "kit-spl__live", role: "status", "aria-live": "polite", {announce} }
        }
    }
}

#[component]
fn ProgressStep(
    item: ProgressItem,
    row_id: String,
    wording: ProgressWording,
    open: bool,
    subs_all: bool,
    panel_open: bool,
    now_ms: u64,
    on_toggle: Callback<String>,
    on_expand: Callback<String>,
    on_panel: Callback<String>,
    on_confirm: Option<EventHandler<ProgressItem>>,
    on_refuse: Option<EventHandler<ProgressItem>>,
) -> Element {
    let key = item.item_key.clone();
    let title = if item.title_text.is_empty() {
        wording.untitled_item.clone()
    } else {
        item.title_text.clone()
    };
    let state_text = state_word(item.item_state, &wording).unwrap_or_default();
    let time = match item.item_state {
        ItemState::Finished => item.took_ms.map(format_mm_ss),
        ItemState::InProgress => item
            .began_at_ms
            .map(|began| format_mm_ss(now_ms.saturating_sub(began))),
        _ => None,
    }
    .unwrap_or_default();
    let name = item_name(&title, &state_text, &time);
    let awaiting = item.item_state == ItemState::AwaitingPerson;
    let in_progress = item.item_state == ItemState::InProgress;
    let has_subs = !item.sub_items.is_empty();
    let focus = item
        .sub_items
        .iter()
        .position(|sub| sub.sub_state == Some(SubState::InProgress));
    let (start, end) = if subs_all {
        (0, item.sub_items.len())
    } else {
        fold_window(item.sub_items.len(), focus)
    };
    let earlier = start;
    let later = item.sub_items.len() - end;
    let earlier_text = wording.earlier_count.replace("{n}", &earlier.to_string());
    let later_text = wording.later_count.replace("{n}", &later.to_string());
    let subs: Vec<(String, SubView)> = item.sub_items[start..end]
        .iter()
        .enumerate()
        .map(|(offset, sub)| {
            (
                sub.sub_key.clone(),
                SubView {
                    key: sub.sub_key.clone(),
                    text: sub.text.clone(),
                    state: sub.sub_state.map_or("", SubState::as_str),
                    fact: sub.fact_text.clone(),
                    current: focus == Some(start + offset),
                },
            )
        })
        .collect();
    let mark = item.item_state.mark();
    let state_name = item.item_state.as_str();
    let tool_name = item
        .tool_badge
        .as_ref()
        .map(|badge| badge.display_name.clone());
    let tool_mark = item
        .tool_badge
        .as_ref()
        .and_then(|badge| badge.mark_source.clone());
    let live_note = item.live_note.clone().filter(|_| in_progress);
    let approval_request = item.approval_request.clone();
    let gate = item.needs_approval && item.item_state == ItemState::AwaitingApproval;
    let confirm_wording = wording.confirm.clone();
    let refuse_wording = wording.refuse.clone();
    let show_fewer = wording.show_fewer.clone();
    let panel_key = key.clone();
    let escape_key = key.clone();
    let toggle_key = key.clone();
    let expand_key = key.clone();
    let subs_id = format!("{row_id}-subs");
    let confirm_item = item.clone();
    let refuse_item = item.clone();
    let panel_region = format!("Approval for {title}");
    let row_class = "kit-spl__row";
    let later_key = key.clone();
    let fewer_key = key.clone();
    let escape_row = row_id.clone();

    rsx! {
        li {
            class: "kit-spl__item",
            "data-state": state_name,
            "aria-current": in_progress.then_some("step"),
            span { class: "kit-spl__branch", "aria-hidden": "true" }
            button {
                id: "{row_id}",
                class: row_class,
                r#type: "button",
                "aria-label": "{name}",
                "aria-expanded": (has_subs && !awaiting).then_some(if open { "true" } else { "false" }),
                "aria-controls": (has_subs && !awaiting && open).then_some(subs_id.clone()),
                onclick: move |_| {
                    if awaiting {
                        on_panel.call(panel_key.clone());
                    } else if has_subs {
                        on_toggle.call(toggle_key.clone());
                    }
                },
                span { class: "kit-spl__mark", "aria-hidden": "true", {mark} }
                span { class: "kit-spl__name",
                    if let Some(tool) = tool_name {
                        span { class: "kit-spl__tool",
                            span { class: "kit-spl__tool-mark", "aria-hidden": "true",
                                if let Some(src) = tool_mark {
                                    img { src: src, alt: "" }
                                }
                            }
                            span { {tool} }
                        }
                    }
                    span { class: "kit-spl__title", {title.clone()} }
                }
                span { class: "kit-spl__right",
                    if let Some(note) = live_note {
                        span { class: "kit-spl__note", {note} }
                    }
                    if !state_text.is_empty() && !matches!(item.item_state, ItemState::InProgress) {
                        span { class: "kit-spl__state", {state_text.clone()} }
                    }
                    if !time.is_empty() {
                        span { class: "kit-spl__time", {time.clone()} }
                    }
                    if gate {
                        span { class: "kit-spl__gate", "aria-hidden": "true", "gate" }
                    }
                }
            }
            if open && has_subs && !awaiting {
                div { id: "{subs_id}", class: "kit-spl__subs",
                    if start > 0 && !subs_all {
                        button {
                            class: "kit-spl__fold",
                            r#type: "button",
                            onclick: move |_| on_expand.call(expand_key.clone()),
                            {earlier_text}
                        }
                    }
                    ul { class: "kit-spl__sub-list",
                        for (sub_key, sub) in subs {
                            li {
                                key: "{sub_key}",
                                class: "kit-spl__sub",
                                "data-state": sub.state,
                                "aria-current": sub.current.then_some("step"),
                                span { class: "kit-spl__sub-text", {sub.text.clone()} }
                                if let Some(fact) = sub.fact.clone() {
                                    span { class: "kit-spl__fact", {fact} }
                                }
                            }
                        }
                    }
                    if end < item.sub_items.len() && !subs_all {
                        button {
                            class: "kit-spl__fold",
                            r#type: "button",
                            onclick: move |_| on_expand.call(later_key.clone()),
                            {later_text}
                        }
                    }
                    if subs_all && item.sub_items.len() > FOLD_THRESHOLD {
                        button {
                            class: "kit-spl__fold",
                            r#type: "button",
                            onclick: move |_| on_expand.call(fewer_key.clone()),
                            {show_fewer}
                        }
                    }
                }
            }
            if awaiting && panel_open {
                div {
                    class: "kit-spl__panel",
                    role: "group",
                    "aria-label": "{panel_region}",
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape {
                            on_panel.call(escape_key.clone());
                            focus_after_frame(&escape_row);
                        }
                    },
                    if let Some(request) = approval_request {
                        p { class: "kit-spl__request", {request} }
                    }
                    div { class: "kit-spl__actions",
                        button {
                            class: "kit-spl__confirm",
                            r#type: "button",
                            onmounted: move |event: MountedEvent| {
                                let node = event.data();
                                spawn(async move {
                                    let _ = node.set_focus(true).await;
                                });
                            },
                            onclick: move |_| {
                                if let Some(handler) = &on_confirm {
                                    handler.call(confirm_item.clone());
                                }
                            },
                            {confirm_wording}
                        }
                        button {
                            class: "kit-spl__refuse",
                            r#type: "button",
                            onclick: move |_| {
                                if let Some(handler) = &on_refuse {
                                    handler.call(refuse_item.clone());
                                }
                            },
                            {refuse_wording}
                        }
                    }
                }
            }
        }
    }
}
