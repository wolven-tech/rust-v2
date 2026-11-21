//! "Since your last visit": what waits on the person, what broke, and what completed, with one action per row.
//!
//! Design choice: `on_entry_action` is a plain callback. A promise cannot cross into Rust, so the host
//! marks rows busy through `busy_entry_ids`.
//! Design choice: `button_text` in the spec is three states, so `EntryButton` replaces it: `Default`
//! (state wording), `Label` (custom wording) and `Hidden`.
//! Design choice: row order is fixed when the digest first renders. A row whose state changes keeps its
//! index; rows that arrive later are placed by state at the end.
//! Design choice: times are epoch seconds. Clock format is UTC `HH:MM`.
//! Browser checks: the awaiting marker pulses; collapse and reopen animate height; focus stays on the
//! divider after Acknowledge; the counts stay on one line at desktop width.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/absence-digest.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EntryState {
    #[default]
    AwaitingInput,
    Failed,
    Completed,
}

impl EntryState {
    pub fn rank(self) -> u8 {
        match self {
            Self::AwaitingInput => 0,
            Self::Failed => 1,
            Self::Completed => 2,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::AwaitingInput => "awaiting_input",
            Self::Failed => "failed",
            Self::Completed => "completed",
        }
    }

    pub fn default_button(self) -> &'static str {
        match self {
            Self::AwaitingInput => "Review",
            Self::Failed => "Retry",
            Self::Completed => "Open",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeFormat {
    #[default]
    Elapsed,
    Clock,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub enum EntryButton {
    #[default]
    Default,
    Label(String),
    Hidden,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DigestEntry {
    pub entry_id: String,
    pub entry_name: String,
    pub entry_state: EntryState,
    pub entry_note: Option<String>,
    pub button_text: EntryButton,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DigestText {
    pub heading: String,
    pub acknowledge: String,
    pub show_more: String,
    pub show_less: String,
    pub nothing_new: String,
    pub divider_expand: String,
}

impl Default for DigestText {
    fn default() -> Self {
        Self {
            heading: "Since your last visit".into(),
            acknowledge: "Acknowledge".into(),
            show_more: "Show more".into(),
            show_less: "Show less".into(),
            nothing_new: "Nothing new".into(),
            divider_expand: "Show digest".into(),
        }
    }
}

/// Orders entries by state, keeping arrival order inside a state.
pub fn order_entries(entries: &[DigestEntry]) -> Vec<DigestEntry> {
    let mut out = entries.to_vec();
    out.sort_by_key(|entry| entry.entry_state.rank());
    out
}

/// Keeps the previous order for entries still present, then appends new ones in state order.
pub fn merge_order(previous: &[String], entries: &[DigestEntry]) -> Vec<String> {
    let mut out: Vec<String> = previous
        .iter()
        .filter(|id| entries.iter().any(|entry| entry.entry_id == **id))
        .cloned()
        .collect();
    for entry in order_entries(entries) {
        if !out.contains(&entry.entry_id) {
            out.push(entry.entry_id);
        }
    }
    out
}

pub fn counts(entries: &[DigestEntry]) -> (usize, usize, usize) {
    let count = |state: EntryState| {
        entries
            .iter()
            .filter(|entry| entry.entry_state == state)
            .count()
    };
    (
        count(EntryState::AwaitingInput),
        count(EntryState::Failed),
        count(EntryState::Completed),
    )
}

pub fn format_elapsed(secs: i64) -> String {
    let secs = secs.max(0);
    if secs >= 86_400 {
        format!("{}d {}h", secs / 86_400, (secs % 86_400) / 3600)
    } else if secs >= 3600 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}m", secs / 60)
    }
}

pub fn format_clock(epoch_secs: i64) -> String {
    let day_secs = epoch_secs.rem_euclid(86_400);
    format!("{:02}:{:02}", day_secs / 3600, (day_secs % 3600) / 60)
}

/// Heading time: the elapsed span or the wall-clock time of leaving.
pub fn time_label(format: TimeFormat, away_since: i64, current_time: i64) -> String {
    match format {
        TimeFormat::Elapsed => format_elapsed(current_time - away_since),
        TimeFormat::Clock => format_clock(away_since),
    }
}

/// Digest card with a collapsible body and an acknowledge control.
#[component]
pub fn AbsenceDigest(
    away_since: i64,
    digest_entries: Vec<DigestEntry>,
    #[props(default)] on_entry_action: Option<EventHandler<String>>,
    #[props(default)] busy_entry_ids: Vec<String>,
    #[props(default)] on_acknowledge: Option<EventHandler<()>>,
    #[props(default)] initially_folded: bool,
    #[props(default)] folded: Option<bool>,
    #[props(default)] on_fold_change: Option<EventHandler<bool>>,
    #[props(default = 5)] row_limit: usize,
    #[props(default)] time_format: TimeFormat,
    #[props(default)] current_time: i64,
    #[props(default)] text_overrides: DigestText,
    #[props(default, into)] root_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-ad-{}", next_id()));
    let mut internal_folded = use_signal(|| initially_folded);
    let mut expanded_all = use_signal(|| false);
    let mut order = use_signal(Vec::<String>::new);
    let mut announce = use_signal(String::new);

    let is_folded = folded.unwrap_or_else(|| internal_folded());
    let merged = merge_order(&order.read(), &digest_entries);
    if *order.peek() != merged {
        order.set(merged.clone());
    }
    let ordered: Vec<DigestEntry> = merged
        .iter()
        .filter_map(|entry_id| {
            digest_entries
                .iter()
                .find(|entry| entry.entry_id == *entry_id)
                .cloned()
        })
        .collect();
    let (awaiting, failed, completed) = counts(&digest_entries);
    let total = digest_entries.len();
    let hidden = total.saturating_sub(row_limit);
    let showing_all = expanded_all();
    let visible: Vec<DigestEntry> = if showing_all {
        ordered.clone()
    } else {
        ordered.iter().take(row_limit).cloned().collect()
    };
    let elapsed = time_label(time_format, away_since, current_time);
    let heading = match time_format {
        TimeFormat::Elapsed => format!("{} · {elapsed}", text_overrides.heading),
        TimeFormat::Clock => format!("{} · left at {elapsed}", text_overrides.heading),
    };
    let counts_text =
        format!("{awaiting} awaiting input · {failed} failed · {completed} completed");
    let show_more_text = format!(
        "{} ({hidden})",
        if showing_all {
            text_overrides.show_less.clone()
        } else {
            text_overrides.show_more.clone()
        }
    );
    let divider_text = format!("{} · {elapsed}", text_overrides.divider_expand);
    let acknowledge_text = text_overrides.acknowledge.clone();
    let nothing_new = text_overrides.nothing_new.clone();
    let root_class = if root_class.is_empty() {
        "kit-ad".to_string()
    } else {
        format!("kit-ad {root_class}")
    };
    let body_id = format!("{id}-body");
    let divider_id = format!("{id}-divider");
    let region_label = heading.clone();
    let set_folded = {
        let handler = on_fold_change.clone();
        move |next: bool| {
            internal_folded.set(next);
            if let Some(h) = &handler {
                h.call(next);
            }
        }
    };
    let mut fold_open = set_folded.clone();
    let mut fold_close = set_folded.clone();
    let ack_handler = on_acknowledge.clone();
    let mut ack_fold = set_folded.clone();

    if is_folded {
        return rsx! {
            document::Link { rel: "stylesheet", href: KIT_CSS }
            document::Link { rel: "stylesheet", href: CSS }
            div { class: "{root_class}", "data-folded": "true",
                button {
                    r#type: "button",
                    id: "{divider_id}",
                    class: "kit-ad__divider",
                    "aria-expanded": "false",
                    "aria-controls": "{body_id}",
                    onclick: move |_| fold_open(false),
                    span { class: "kit-ad__rule", aria_hidden: "true" }
                    span { class: "kit-ad__divider-text", "{divider_text}" }
                    span { class: "kit-ad__rule", aria_hidden: "true" }
                }
            }
        };
    }

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            id: "{id}",
            class: "{root_class}",
            role: "region",
            aria_label: "{region_label}",
            "data-folded": "false",
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape {
                    event.prevent_default();
                    fold_close(true);
                }
            },
            header { class: "kit-ad__head",
                h2 { class: "kit-ad__title", "{heading}" }
                p { class: "kit-ad__counts", "{counts_text}" }
            }
            div { id: "{body_id}", class: "kit-ad__body",
                if total == 0 {
                    p { class: "kit-ad__empty", "{nothing_new}" }
                } else {
                    ul { class: "kit-ad__rows",
                        for entry in visible {
                            DigestRow {
                                key: "{entry.entry_id}",
                                entry: entry.clone(),
                                busy: busy_entry_ids.contains(&entry.entry_id),
                                on_action: on_entry_action.clone(),
                            }
                        }
                    }
                    if hidden > 0 {
                        button {
                            r#type: "button",
                            class: "kit-ad__more",
                            onclick: move |_| expanded_all.set(!showing_all),
                            "{show_more_text}"
                        }
                    }
                }
                div { class: "kit-ad__foot",
                    button {
                        r#type: "button",
                        class: "kit-ad__ack",
                        onclick: move |_| {
                            if let Some(h) = &ack_handler {
                                h.call(());
                            }
                            ack_fold(true);
                            announce.set(format!("Acknowledged. {awaiting} awaiting, {failed} failed, {completed} completed."));
                        },
                        "{acknowledge_text}"
                    }
                }
            }
            p { class: "kit-ad__live", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}

#[component]
fn DigestRow(entry: DigestEntry, busy: bool, on_action: Option<EventHandler<String>>) -> Element {
    let entry_id = entry.entry_id.clone();
    let name = entry.entry_name.clone();
    let state = entry.entry_state;
    let note = entry.entry_note.clone().unwrap_or_default();
    let has_note = entry.entry_note.is_some();
    let button_label = match &entry.button_text {
        EntryButton::Default => Some(state.default_button().to_string()),
        EntryButton::Label(text) => Some(text.clone()),
        EntryButton::Hidden => None,
    };
    let action_label = button_label.map(|label| format!("{label}: {name}"));
    let button_text = button_label_text(&entry.button_text, state);
    let state_word = state.as_str().replace('_', " ");
    rsx! {
        li { class: "kit-ad__row", "data-state": state.as_str(), "aria-busy": busy.then_some("true"),
            span { class: "kit-ad__marker", "data-state": state.as_str(), aria_hidden: "true" }
            div { class: "kit-ad__text",
                span { class: "kit-ad__name", title: "{name}", "{name}" }
                span { class: "kit-ad__state", "{state_word}" }
                if has_note {
                    span { class: "kit-ad__note", "{note}" }
                }
            }
            if let Some(action) = action_label {
                button {
                    r#type: "button",
                    class: "kit-ad__action",
                    aria_label: "{action}",
                    disabled: busy,
                    onclick: {
                        let target = entry_id.clone();
                        let handler = on_action.clone();
                        move |_| {
                            if let Some(h) = &handler {
                                h.call(target.clone());
                            }
                        }
                    },
                    "{button_text}"
                }
            }
        }
    }
}

fn button_label_text(button: &EntryButton, state: EntryState) -> String {
    match button {
        EntryButton::Default => state.default_button().to_string(),
        EntryButton::Label(text) => text.clone(),
        EntryButton::Hidden => String::new(),
    }
}
