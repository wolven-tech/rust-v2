//! Review backlog: one agent proposal at a time, decided by approve, edit, reject or skip, with undo.
//!
//! Design choice: flagged passages are listed as notes under the body rather than highlighted inline;
//! the body is a plain string, so each flag carries its concern without a rich-text model.
//! Design choice: the edit is a plain textarea over `body_text`. Ctrl or Cmd+Enter approves the edit.
//! Design choice: decided items leave the queue and their results stay on an undo stack. Undo
//! restores the most recent decision; the sort order puts the item back where it was.
//! Design choice: `current_time` is epoch seconds supplied by the host; the component reads no clock.
//! Design choice: skip moves the selection to the next visible item and marks the row; it does not
//! remove the item from the queue.
//! Browser checks: the queue scrolls inside a fixed-height region; the decision bar stays reachable;
//! the container query stacks the queue above the item on narrow widths.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/decision-review-queue.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReviewRisk {
    High,
    #[default]
    Medium,
    Low,
}

impl ReviewRisk {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    fn rank(self) -> u8 {
        match self {
            Self::High => 0,
            Self::Medium => 1,
            Self::Low => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortOrder {
    #[default]
    Risk,
    Oldest,
    Given,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum QueueFilter {
    #[default]
    All,
    High,
    Unsure,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BodyFormat {
    #[default]
    Text,
    Code,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewOutcome {
    Approve,
    Edited,
    Reject,
}

impl ReviewOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::Edited => "edited",
            Self::Reject => "reject",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitState {
    Fresh,
    Warning,
    Overdue,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FileChange {
    pub path: String,
    pub lines_added: Option<u32>,
    pub lines_removed: Option<u32>,
    pub diff_text: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PlanStep {
    pub step_title: String,
    pub tool_name: Option<String>,
    pub step_detail: Option<String>,
    pub flagged: bool,
}

/// A flagged passage in the body, with the reason it was flagged.
#[derive(Clone, PartialEq, Debug)]
pub struct FlagNote {
    pub phrase: String,
    pub note: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ReviewItem {
    pub item_id: String,
    pub item_title: String,
    pub agent_name: String,
    pub model_name: Option<String>,
    pub item_kind: String,
    pub risk_level: ReviewRisk,
    /// 0.0 to 1.0.
    pub confidence_score: f64,
    /// Epoch seconds.
    pub created_at: i64,
    pub agent_reasoning: Option<String>,
    pub body_heading: Option<String>,
    pub body_text: Option<String>,
    pub body_format: BodyFormat,
    pub flags: Vec<FlagNote>,
    pub file_changes: Vec<FileChange>,
    pub plan_steps: Vec<PlanStep>,
    pub cited_sources: Vec<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ReviewResult {
    pub outcome: ReviewOutcome,
    pub reason: Option<String>,
    pub edited_text: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DecideEvent {
    pub item: ReviewItem,
    pub result: ReviewResult,
}

/// Queue order: risk then age, age alone, or as given.
pub fn sort_items(items: &[ReviewItem], order: SortOrder) -> Vec<ReviewItem> {
    let mut out = items.to_vec();
    match order {
        SortOrder::Risk => out.sort_by_key(|item| (item.risk_level.rank(), item.created_at)),
        SortOrder::Oldest => out.sort_by_key(|item| item.created_at),
        SortOrder::Given => {}
    }
    out
}

pub fn passes_filter(item: &ReviewItem, filter: QueueFilter, unsure_below: f64) -> bool {
    match filter {
        QueueFilter::All => true,
        QueueFilter::High => item.risk_level == ReviewRisk::High,
        QueueFilter::Unsure => item.confidence_score < unsure_below,
    }
}

/// Fresh until three quarters of the limit, a warning until it is passed, overdue after.
pub fn wait_state(waited_secs: i64, wait_limit_secs: i64) -> WaitState {
    if waited_secs > wait_limit_secs {
        WaitState::Overdue
    } else if waited_secs * 4 > wait_limit_secs * 3 {
        WaitState::Warning
    } else {
        WaitState::Fresh
    }
}

pub fn format_wait(secs: i64) -> String {
    let secs = secs.max(0);
    if secs >= 3600 {
        format!("{}h {:02}m", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}m", secs / 60)
    } else {
        format!("{secs}s")
    }
}

/// Reason chosen by a digit key, 1-based, from the first four reasons.
pub fn reason_for_digit(digit: char, reasons: &[String]) -> Option<String> {
    let index = digit.to_digit(10)?.checked_sub(1)? as usize;
    if index >= 4 {
        return None;
    }
    reasons.get(index).cloned()
}

/// One entry per line of a unified diff: the sign that marks it and the text after the sign.
pub fn diff_lines(diff: &str) -> Vec<(char, String)> {
    diff.lines()
        .map(|line| {
            if let Some(rest) = line.strip_prefix('+') {
                ('+', rest.to_string())
            } else if let Some(rest) = line.strip_prefix('-') {
                ('-', rest.to_string())
            } else {
                (' ', line.strip_prefix(' ').unwrap_or(line).to_string())
            }
        })
        .collect()
}

/// Opens the flag after the focused one and moves focus to its summary.
fn focus_next_flag(root: &str) {
    crate::internal::focus_next_summary(root, ".kit-drq__flag > summary");
}

/// Progress: decided before this session plus decided here, out of that plus what is still queued.
pub fn progress(reviewed_before: usize, decided_here: usize, remaining: usize) -> (usize, usize) {
    let done = reviewed_before + decided_here;
    (done, done + remaining)
}

/// Review workspace: queue on the left, the current item and its decision bar on the right.
#[component]
pub fn DecisionReviewQueue(
    review_items: Vec<ReviewItem>,
    #[props(default)] sort_order: SortOrder,
    #[props(default)] default_queue_filter: QueueFilter,
    #[props(default = 0.7)] unsure_below: f64,
    #[props(default)] reviewed_before: usize,
    #[props(default = 3600)] wait_limit_secs: i64,
    #[props(default)] current_time: i64,
    #[props(default = vec!["Off-policy".to_string(), "Wrong facts".to_string(), "Too risky".to_string(), "Not needed".to_string()])]
    reject_reasons: Vec<String>,
    #[props(default)] on_decide: Option<EventHandler<DecideEvent>>,
    #[props(default)] on_undo: Option<EventHandler<DecideEvent>>,
    #[props(default)] on_skip: Option<EventHandler<ReviewItem>>,
    #[props(default = "Review".to_string(), into)] board_title: String,
    #[props(default, into)] root_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-drq-{}", next_id()));
    let mut filter = use_signal(|| default_queue_filter);
    let mut gone = use_signal(Vec::<DecideEvent>::new);
    let mut skipped = use_signal(Vec::<String>::new);
    let mut selected = use_signal(|| Option::<String>::None);
    let mut pane_open = use_signal(|| false);
    let mut editing = use_signal(|| false);
    let mut draft = use_signal(String::new);
    let mut reason_open = use_signal(|| false);
    let mut announce = use_signal(String::new);

    let decided_here = gone().len();
    let decided_ids: Vec<String> = gone()
        .iter()
        .map(|event| event.item.item_id.clone())
        .collect();
    let ordered = sort_items(&review_items, sort_order);
    let remaining = ordered
        .iter()
        .filter(|item| !decided_ids.contains(&item.item_id))
        .count();
    let visible: Vec<ReviewItem> = ordered
        .into_iter()
        .filter(|item| !decided_ids.contains(&item.item_id))
        .filter(|item| passes_filter(item, filter(), unsure_below))
        .collect();
    let (done, total) = progress(reviewed_before, decided_here, remaining);
    let progress_text = format!("{done} of {total} reviewed");
    let current_index = selected()
        .and_then(|sel| visible.iter().position(|item| item.item_id == sel))
        .unwrap_or(0);
    let current = visible.get(current_index).cloned();
    let oldest_wait = visible
        .iter()
        .map(|item| current_time - item.created_at)
        .max()
        .unwrap_or(0);
    let oldest_text = format!("Oldest waiting {}", format_wait(oldest_wait));
    let count_for = |wanted: QueueFilter| {
        review_items
            .iter()
            .filter(|item| !decided_ids.contains(&item.item_id))
            .filter(|item| passes_filter(item, wanted, unsure_below))
            .count()
    };
    let (count_all, count_high, count_unsure) = (
        count_for(QueueFilter::All),
        count_for(QueueFilter::High),
        count_for(QueueFilter::Unsure),
    );
    let flag_root = id.clone();
    let root_class = if root_class.is_empty() {
        "kit-drq".to_string()
    } else {
        format!("kit-drq {root_class}")
    };
    let reasons: Vec<String> = reject_reasons.iter().take(4).cloned().collect();
    let reason_items: Vec<(usize, String)> = reasons
        .iter()
        .enumerate()
        .map(|(index, reason)| (index + 1, reason.clone()))
        .collect();
    let visible_ids: Vec<String> = visible.iter().map(|item| item.item_id.clone()).collect();
    let skipped_ids = skipped();

    let decide = {
        let on_decide = on_decide.clone();
        move |item: ReviewItem, result: ReviewResult| {
            let verb = match result.outcome {
                ReviewOutcome::Approve => "Approved",
                ReviewOutcome::Edited => "Approved with edits",
                ReviewOutcome::Reject => "Rejected",
            };
            announce.set(format!("{verb}: {}", item.item_title));
            let event = DecideEvent { item, result };
            gone.with_mut(|list| list.push(event.clone()));
            selected.set(None);
            editing.set(false);
            reason_open.set(false);
            if let Some(handler) = &on_decide {
                handler.call(event);
            }
        }
    };
    let mut undo = {
        let on_undo = on_undo.clone();
        move |_: ()| {
            let last = gone.with_mut(|list| list.pop());
            if let Some(event) = last {
                announce.set(format!("Undone: {}", event.item.item_title));
                selected.set(Some(event.item.item_id.clone()));
                if let Some(handler) = &on_undo {
                    handler.call(event);
                }
            }
        }
    };
    let skip = {
        let on_skip = on_skip.clone();
        let order = visible_ids.clone();
        move |item: ReviewItem| {
            announce.set(format!("Skipped: {}", item.item_title));
            skipped.with_mut(|list| {
                if !list.contains(&item.item_id) {
                    list.push(item.item_id.clone());
                }
            });
            let next = order
                .iter()
                .position(|known| *known == item.item_id)
                .and_then(|at| order.get((at + 1) % order.len()))
                .cloned();
            selected.set(next);
            editing.set(false);
            reason_open.set(false);
            if let Some(handler) = &on_skip {
                handler.call(item);
            }
        }
    };
    let step_to = {
        let order = visible_ids.clone();
        move |step: isize| {
            let count = order.len();
            if count == 0 {
                return;
            }
            let next = (current_index as isize + step).rem_euclid(count as isize) as usize;
            selected.set(order.get(next).cloned());
        }
    };

    let queue_rows: Vec<(
        String,
        String,
        String,
        String,
        bool,
        bool,
        &'static str,
        String,
    )> = visible
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let wait = format_wait(current_time - item.created_at);
            let percent = (item.confidence_score * 100.0).round() as i64;
            let label = format!(
                "{}, {}, {} risk, {percent}% confidence, waiting {wait}",
                item.item_title,
                item.item_kind,
                item.risk_level.as_str(),
            );
            (
                item.item_id.clone(),
                label,
                item.item_title.clone(),
                wait,
                index == current_index,
                skipped_ids.contains(&item.item_id),
                item.risk_level.as_str(),
                item.item_kind.clone(),
            )
        })
        .collect();

    let key_current = current.clone();
    let mut key_decide = decide.clone();
    let mut key_undo = undo.clone();
    let mut key_skip = skip.clone();
    let mut key_step = step_to.clone();
    let key_reasons = reasons.clone();
    let has_undo = !gone().is_empty();

    let detail_view = current.clone().map(|item| {
        let whole = item.clone();
        let ReviewItem {
            item_title,
            agent_name,
            model_name,
            risk_level,
            confidence_score,
            agent_reasoning,
            body_heading,
            body_text,
            body_format,
            flags,
            file_changes,
            plan_steps,
            cited_sources,
            ..
        } = item;
        let risk_key = risk_level.as_str();
        let percent = (confidence_score * 100.0).round() as i64;
        let confidence_text = format!("{percent}% confident");
        let risk_text = format!("{risk_key} risk");
        let is_code = body_format == BodyFormat::Code;
        let body_format_attr = if is_code { "code" } else { "text" };
        let body_plain = body_text.clone().unwrap_or_default();
        let has_body = body_text.is_some();
        let flag_items: Vec<(String, String)> = flags
            .into_iter()
            .map(|flag| (flag.phrase, flag.note))
            .collect();
        let change_items: Vec<(String, u32, u32, Option<String>)> = file_changes
            .into_iter()
            .map(|change| {
                (
                    change.path,
                    change.lines_added.unwrap_or(0),
                    change.lines_removed.unwrap_or(0),
                    change.diff_text,
                )
            })
            .collect();
        let step_items: Vec<(String, Option<String>, Option<String>, bool)> = plan_steps
            .into_iter()
            .map(|step| (step.step_title, step.tool_name, step.step_detail, step.flagged))
            .collect();
        let has_steps = !step_items.is_empty();
        let has_sources = !cited_sources.is_empty();
        let approve_item = whole.clone();
        let mut approve = decide.clone();
        let skip_item = whole.clone();
        let mut skip_now = skip.clone();
        let reject_item = whole.clone();
        let reject_reasons = reason_items.clone();
        let reject_decide = decide.clone();
        let seed_text = body_plain.clone();
        let edit_seed = body_plain.clone();
        let heading_text = body_heading.unwrap_or_default();
        let has_heading = !heading_text.is_empty();
        let reasoning_text = agent_reasoning.unwrap_or_default();
        let has_reasoning = !reasoning_text.is_empty();
        let model_text = model_name.unwrap_or_default();
        let has_model = !model_text.is_empty();
        let sources_list = cited_sources;
        let _edit_text = seed_text.clone();
        let flagged_changes: Vec<(String, u32, u32, Option<String>)> = change_items;
        rsx! {
            article { class: "kit-drq__pane", "data-risk": risk_key,
                button { r#type: "button", class: "kit-drq__back", onclick: move |_| pane_open.set(false), "Back to queue" }
                div { class: "kit-drq__content",
                h3 { class: "kit-drq__item-title", "{item_title}" }
                p { class: "kit-drq__meta",
                    span { "{agent_name}" }
                    if has_model {
                        span { "{model_text}" }
                    }
                    span { class: "kit-drq__tag", "data-risk": risk_key, "{risk_text}" }
                    span { class: "kit-drq__tag", "{confidence_text}" }
                }
                if has_heading {
                    h4 { class: "kit-drq__body-title", "{heading_text}" }
                }
                if editing() {
                    textarea {
                        class: "kit-drq__editor",
                        aria_label: "Edit proposed text",
                        value: "{draft}",
                        oninput: move |event: FormEvent| draft.set(event.value()),
                        onkeydown: move |event: KeyboardEvent| {
                            let mods = event.modifiers();
                            if event.key() == Key::Enter && (mods.ctrl() || mods.meta()) {
                                event.prevent_default();
                                let text = draft();
                                key_decide(whole.clone(), ReviewResult { outcome: ReviewOutcome::Edited, reason: None, edited_text: Some(text) });
                            }
                        },
                    }
                } else if has_body {
                    pre { class: "kit-drq__text", "data-format": body_format_attr, "{body_plain}" }
                }
                for (phrase, note) in flag_items {
                    details { key: "{phrase}", class: "kit-drq__flag",
                        summary { "Flagged: {phrase}" }
                        p { "{note}" }
                    }
                }
                for (path, added, removed, diff) in flagged_changes {
                    section { key: "{path}", class: "kit-drq__file",
                        header { class: "kit-drq__file-head",
                            code { "{path}" }
                            span { class: "kit-drq__add", "+{added}" }
                            span { class: "kit-drq__del", "-{removed}" }
                        }
                        if let Some(diff_text) = diff {
                            ol { class: "kit-drq__diff", "aria-label": "Diff of {path}",
                                for (number, (mark, line)) in diff_lines(&diff_text).into_iter().enumerate() {
                                    li { key: "{number}", class: "kit-drq__line", "data-mark": "{mark}",
                                        span { class: "kit-drq__ln", "aria-hidden": "true", "{number + 1}" }
                                        span { class: "kit-drq__sign", "aria-hidden": "true", "{mark}" }
                                        span { class: "kit-drq__code", "{line}" }
                                    }
                                }
                            }
                        }
                    }
                }
                if has_reasoning {
                    blockquote { class: "kit-drq__reasoning", "{reasoning_text}" }
                }
                if has_steps {
                    ol { class: "kit-drq__plan",
                        for (title, tool, detail_text, flagged) in step_items {
                            li { "data-flagged": flagged.then_some("true"),
                                span { "{title}" }
                                if let Some(tool_name) = tool {
                                    " "
                                    code { "{tool_name}" }
                                }
                                if let Some(text) = detail_text {
                                    p { "{text}" }
                                }
                            }
                        }
                    }
                }
                if has_sources {
                    ul { class: "kit-drq__sources",
                        for source in sources_list {
                            li { "{source}" }
                        }
                    }
                }
                }
                div { class: "kit-drq__bar", role: "group", aria_label: "Decisions",
                    button {
                        r#type: "button",
                        class: "kit-drq__approve",
                        "aria-keyshortcuts": "A",
                        onclick: move |_| approve(approve_item.clone(), ReviewResult { outcome: ReviewOutcome::Approve, reason: None, edited_text: None }),
                        "Approve "
                        span { class: "kit-drq__key", "A" }
                    }
                    button {
                        r#type: "button",
                        class: "kit-drq__edit",
                        "aria-keyshortcuts": "E",
                        onclick: move |_| {
                            draft.set(edit_seed.clone());
                            editing.set(true);
                        },
                        "Edit "
                        span { class: "kit-drq__key", "E" }
                    }
                    button {
                        r#type: "button",
                        class: "kit-drq__reject",
                        "aria-keyshortcuts": "R",
                        onclick: move |_| reason_open.set(true),
                        "Reject "
                        span { class: "kit-drq__key", "R" }
                    }
                    button {
                        r#type: "button",
                        class: "kit-drq__skip",
                        "aria-keyshortcuts": "S",
                        onclick: move |_| skip_now(skip_item.clone()),
                        "Skip "
                        span { class: "kit-drq__key", "S" }
                    }
                    button {
                        r#type: "button",
                        class: "kit-drq__undo",
                        "aria-keyshortcuts": "Z",
                        disabled: !has_undo,
                        onclick: move |_| undo(()),
                        "Undo "
                        span { class: "kit-drq__key", "Z" }
                    }
                }
                if reason_open() {
                    div { class: "kit-drq__reasons", role: "group", aria_label: "Reject reason",
                        for (number, reason) in reject_reasons {
                            button {
                                key: "{number}",
                                r#type: "button",
                                class: "kit-drq__reason",
                                onclick: {
                                    let picked = reason.clone();
                                    let target = reject_item.clone();
                                    let mut reject = reject_decide.clone();
                                    move |_| reject(target.clone(), ReviewResult { outcome: ReviewOutcome::Reject, reason: Some(picked.clone()), edited_text: None })
                                },
                                span { class: "kit-drq__key", "{number}" }
                                "{reason}"
                            }
                        }
                    }
                }
            }
        }
    });

    let pane_attr = if pane_open() && detail_view.is_some() {
        "open"
    } else {
        "list"
    };
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            id: "{id}",
            class: "{root_class}",
            "data-pane": pane_attr,
            aria_label: "{board_title}",
            tabindex: "0",
            onkeydown: move |event: KeyboardEvent| {
                let mods = event.modifiers();
                let modified = mods.ctrl() || mods.meta();
                if editing() {
                    if event.key() == Key::Escape {
                        event.prevent_default();
                        editing.set(false);
                    }
                    return;
                }
                if modified && matches!(event.key(), Key::Character(ref c) if c.eq_ignore_ascii_case("z")) {
                    event.prevent_default();
                    key_undo(());
                    return;
                }
                if reason_open() {
                    if event.key() == Key::Escape {
                        event.prevent_default();
                        reason_open.set(false);
                    } else if let Key::Character(text) = event.key() {
                        if let Some(first) = text.chars().next() {
                            if let Some(reason) = reason_for_digit(first, &key_reasons) {
                                if let Some(item) = key_current.clone() {
                                    event.prevent_default();
                                    key_decide(item, ReviewResult { outcome: ReviewOutcome::Reject, reason: Some(reason), edited_text: None });
                                }
                            }
                        }
                    }
                    return;
                }
                if event.key() == Key::ArrowDown || event.key() == Key::ArrowUp {
                    event.prevent_default();
                    key_step(if event.key() == Key::ArrowDown { 1 } else { -1 });
                    return;
                }
                let Key::Character(text) = event.key() else {
                    return;
                };
                let Some(item) = key_current.clone() else {
                    return;
                };
                match text.as_str() {
                    "a" => {
                        event.prevent_default();
                        key_decide(item, ReviewResult { outcome: ReviewOutcome::Approve, reason: None, edited_text: None });
                    }
                    "e" => {
                        event.prevent_default();
                        editing.set(true);
                    }
                    "r" => {
                        event.prevent_default();
                        reason_open.set(true);
                    }
                    "s" => {
                        event.prevent_default();
                        key_skip(item);
                    }
                    "j" => {
                        event.prevent_default();
                        key_step(1);
                    }
                    "k" => {
                        event.prevent_default();
                        key_step(-1);
                    }
                    "z" => {
                        event.prevent_default();
                        key_undo(());
                    }
                    "f" => {
                        event.prevent_default();
                        focus_next_flag(&flag_root);
                    }
                    _ => {}
                }
            },
            header { class: "kit-drq__head",
                h2 { class: "kit-drq__title", "{board_title}" }
                span { class: "kit-drq__progress", "{progress_text}" }
                span { class: "kit-drq__oldest", "{oldest_text}" }
            }
            div { class: "kit-drq__body",
                div { class: "kit-drq__side",
                    div { class: "kit-drq__filters", role: "group", aria_label: "Queue filter",
                        button {
                            r#type: "button",
                            class: "kit-drq__filter",
                            "aria-pressed": if filter() == QueueFilter::All { "true" } else { "false" },
                            onclick: move |_| filter.set(QueueFilter::All),
                            "All "
                            span { class: "kit-drq__count", "{count_all}" }
                        }
                        button {
                            r#type: "button",
                            class: "kit-drq__filter",
                            "aria-pressed": if filter() == QueueFilter::High { "true" } else { "false" },
                            onclick: move |_| filter.set(QueueFilter::High),
                            "High risk "
                            span { class: "kit-drq__count", "{count_high}" }
                        }
                        button {
                            r#type: "button",
                            class: "kit-drq__filter",
                            "aria-pressed": if filter() == QueueFilter::Unsure { "true" } else { "false" },
                            onclick: move |_| filter.set(QueueFilter::Unsure),
                            "Unsure "
                            span { class: "kit-drq__count", "{count_unsure}" }
                        }
                    }
                    ol { class: "kit-drq__queue", aria_label: "Queue",
                        for (row_id, label, title, wait, active, was_skipped, row_risk, kind) in queue_rows {
                            li { key: "{row_id}", class: "kit-drq__row-item",
                                button {
                                    r#type: "button",
                                    class: "kit-drq__row",
                                    aria_label: "{label}",
                                    "aria-current": if active { "true" } else { "false" },
                                    "data-skipped": was_skipped.then_some("true"),
                                    "data-risk": row_risk,
                                    onclick: {
                                        let pick = row_id.clone();
                                        move |_| {
                                            selected.set(Some(pick.clone()));
                                            pane_open.set(true);
                                        }
                                    },
                                    span { class: "kit-drq__row-mark", "aria-hidden": "true" }
                                    span { class: "kit-drq__row-text",
                                        span { class: "kit-drq__row-title", "{title}" }
                                        span { class: "kit-drq__row-kind", "{kind}" }
                                        span { class: "kit-drq__row-wait", "{wait}" }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(pane) = detail_view {
                    {pane}
                } else {
                    p { class: "kit-drq__empty", role: "status", "Nothing left to review." }
                    div { class: "kit-drq__bar", role: "group", aria_label: "Decisions",
                        button { r#type: "button", class: "kit-drq__approve", disabled: true, "Approve " span { class: "kit-drq__key", "A" } }
                        button { r#type: "button", class: "kit-drq__edit", disabled: true, "Edit " span { class: "kit-drq__key", "E" } }
                        button { r#type: "button", class: "kit-drq__reject", disabled: true, "Reject " span { class: "kit-drq__key", "R" } }
                        button { r#type: "button", class: "kit-drq__skip", disabled: true, "Skip " span { class: "kit-drq__key", "S" } }
                        button { r#type: "button", class: "kit-drq__undo", disabled: !has_undo, "Undo " span { class: "kit-drq__key", "Z" } }
                    }
                }
            }
            p { class: "kit-drq__live", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
