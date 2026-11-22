//! The AI workspace's landing page: a composer at the top, then the person's own figures in four cards
//! (activity, recent chats, usage, files made) and this month against last month.
//!
//! Design choice: the composer is `PromptInputBar`, with the two shortcuts in its footer.
//! Design choice: the activity grid is a roving-tabindex set of day buttons. Left and right move a
//! week, up and down move a day, as the spec's interactions say (its acceptance check reads "day by
//! day", which the up and down keys satisfy).
//! Design choice: the day tip lives in a slot with reserved height, so showing it cannot move the
//! grid under the pointer.
//! Design choice: the eyebrow above the composer carries the workspace and account names, joined.
//! Browser checks: arrow keys move the focused day and show its tip; Escape hides the tip; the empty
//! state's button focuses the composer; long file names keep their extension.

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonVariant};
use rv2_affect::skeleton::Skeleton;

use crate::components::metric_tile::{MetricTile, MetricValue};
use crate::components::prompt_input_bar::{Model, PromptInputBar, Submission};
use crate::components::series_trend_chart::group_digits;
use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/ai-workspace-home.css");

/// Days per column in the activity grid.
const WEEK: usize = 7;

/// Characters a file name keeps in the files card. Sized so the shortened name fits the card's
/// narrowest supported width without the row's end ellipsis cutting off the extension.
pub const FILE_NAME_CHARS: usize = 18;

#[derive(Clone, Debug, PartialEq)]
pub struct ActivityDay {
    pub date: String,
    pub chat_count: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentChat {
    pub id: String,
    pub entry_title: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UsageMeter {
    pub used_units: u32,
    pub unit_limit: u32,
    pub resets_on: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FileEntry {
    pub entry_name: String,
    pub extension: String,
    pub created_at: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonthStat {
    pub entry_value: u32,
    pub change_vs_last: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonthStats {
    pub chats: MonthStat,
    pub files_made: MonthStat,
}

/// Total chats across the displayed days.
pub fn activity_total(days: &[ActivityDay]) -> u32 {
    days.iter().map(|d| d.chat_count).sum()
}

/// First and last date of the displayed period.
pub fn activity_range(days: &[ActivityDay]) -> Option<(String, String)> {
    match (days.first(), days.last()) {
        (Some(first), Some(last)) => Some((first.date.clone(), last.date.clone())),
        _ => None,
    }
}

/// Shade of a day square, 0 for none and 4 for the busiest days.
pub fn activity_level(count: u32, max: u32) -> u8 {
    if count == 0 || max == 0 {
        0
    } else {
        (((count as u64 * 4).div_ceil(max as u64)) as u8).min(4)
    }
}

/// The day that focus moves to. Left and right move a week, up and down a day; Home and End go to the
/// first and last day. Other keys keep the index.
pub fn move_index(index: usize, count: usize, key: &Key) -> usize {
    if count == 0 {
        return 0;
    }
    let last = count - 1;
    match key {
        Key::ArrowLeft => index.saturating_sub(WEEK),
        Key::ArrowRight => (index + WEEK).min(last),
        Key::ArrowUp => index.saturating_sub(1),
        Key::ArrowDown => (index + 1).min(last),
        Key::Home => 0,
        Key::End => last,
        _ => index,
    }
}

/// Used units as a share of the limit, from 0 to 1.
pub fn usage_ratio(used: u32, limit: u32) -> f64 {
    if limit == 0 {
        return 0.0;
    }
    (used as f64 / limit as f64).clamp(0.0, 1.0)
}

/// The meter's words, which say so in full once the limit is passed.
pub fn usage_words(used: u32, limit: u32) -> String {
    let used_text = group_digits(used as f64);
    let limit_text = group_digits(limit as f64);
    if used > limit {
        format!("Limit reached: {used_text} of {limit_text} units used.")
    } else {
        format!("{used_text} of {limit_text} units used.")
    }
}

/// A change figure with its direction and its comparison, in words.
pub fn change_words(change: i32, period: &str) -> String {
    match change.cmp(&0) {
        std::cmp::Ordering::Greater => format!("up {change} on {period}"),
        std::cmp::Ordering::Less => format!("down {} on {period}", change.unsigned_abs()),
        std::cmp::Ordering::Equal => format!("same as {period}"),
    }
}

fn count_change(magnitude: f64) -> String {
    format!("{magnitude:.0}")
}

/// Shortens a file name in the middle to `max` characters, keeping its suffix (and so its extension).
pub fn middle_truncate(name: &str, max: usize) -> String {
    let chars: Vec<char> = name.chars().collect();
    if chars.len() <= max || max < 4 {
        return name.to_string();
    }
    let ext_len = name
        .rfind('.')
        .map(|dot| name[dot..].chars().count())
        .unwrap_or(0);
    let tail_len = ext_len.max(max / 3).min(max - 2);
    let head_len = max - tail_len - 1;
    let head: String = chars[..head_len].iter().collect();
    let tail: String = chars[chars.len() - tail_len..].iter().collect();
    format!("{head}…{tail}")
}

/// Home page for the AI workspace.
#[component]
pub fn AiWorkspaceHome(
    #[props(default, into)] workspace_name: String,
    #[props(default, into)] account_name: String,
    #[props(default)] activity_days: Vec<ActivityDay>,
    #[props(default)] recent_chat_list: Vec<RecentChat>,
    #[props(default)] hidden_chat_count: usize,
    usage_meter: UsageMeter,
    #[props(default)] file_list: Vec<FileEntry>,
    #[props(default)] hidden_file_count: usize,
    month_stats: MonthStats,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    on_send: EventHandler<String>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] on_open_chat: Option<EventHandler<String>>,
    #[props(default)] on_open_file: Option<EventHandler<String>>,
) -> Element {
    let id = use_hook(|| format!("kit-home-{}", next_id()));
    let eyebrow = [workspace_name.as_str(), account_name.as_str()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    let composer_class = "kit-home__composer";
    let mut focus = use_signal(|| 0usize);
    let mut tip = use_signal(|| None::<usize>);
    let total = activity_total(&activity_days);
    let peak = activity_days
        .iter()
        .map(|d| d.chat_count)
        .max()
        .unwrap_or(0);
    let range = activity_range(&activity_days);
    let tip_id = format!("{id}-tip");
    let grid_id = format!("{id}-grid");
    let key_id = id.clone();
    let models = vec![Model {
        model_id: "standard".to_string(),
        model_name: "Standard".to_string(),
        model_mark: None,
        model_note: None,
    }];
    let recent_shown: Vec<RecentChat> = recent_chat_list.iter().take(3).cloned().collect();
    let chats_more = hidden_chat_count + recent_chat_list.len().saturating_sub(3);
    let files_shown: Vec<FileEntry> = file_list.iter().take(3).cloned().collect();
    let files_more = hidden_file_count + file_list.len().saturating_sub(3);
    let used = usage_meter.used_units;
    let limit = usage_meter.unit_limit;
    let fill = format!("{:.1}%", usage_ratio(used, limit) * 100.0);
    let meter_text = usage_words(used, limit);
    let reset_text = format!("Resets {}", usage_meter.resets_on);
    let focus_composer = move |_| {
        document::eval(&format!(
            "document.querySelector('.{composer_class} textarea, .{composer_class} input')?.focus();"
        ));
    };
    let footer = rsx! {
        div { class: "kit-home__shortcuts",
            Button {
                variant: ButtonVariant::Ghost,
                label: "Work in a folder",
                onclick: move |_| {
                    if let Some(nav) = on_navigate {
                        nav.call("Folders".to_string());
                    }
                },
            }
            Button {
                variant: ButtonVariant::Ghost,
                label: "Connect apps",
                onclick: move |_| {
                    if let Some(nav) = on_navigate {
                        nav.call("Connect apps".to_string());
                    }
                },
            }
        }
    };
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "kit-home", "data-empty": is_empty.then_some("true"),
            div { class: "kit-home__head",
                if !eyebrow.is_empty() {
                    p { class: "kit-home__eyebrow", "{eyebrow}" }
                }
                PromptInputBar {
                    root_class: composer_class,
                    empty_hint: "Ask for a document, a table or an answer from your files",
                    model_options: models,
                    on_submit: move |submission: Submission| on_send.call(submission.text),
                    custom_footer: footer,
                }
            }
            if is_empty {
                section { class: "kit-home__empty",
                    h2 { class: "kit-home__empty-title", "Nothing here yet" }
                    p { class: "kit-home__empty-line", "Chats, usage and files you make will appear here." }
                    Button {
                        variant: ButtonVariant::Primary,
                        label: "Start a chat",
                        onclick: focus_composer,
                    }
                }
            } else if is_loading {
                div { class: "kit-home__board", "aria-busy": "true",
                    Skeleton { width: "100%", height: "180px" }
                    Skeleton { width: "100%", height: "180px" }
                    Skeleton { width: "100%", height: "180px" }
                    Skeleton { width: "100%", height: "180px" }
                }
            } else {
                div { class: "kit-home__board",
                    section { class: "kit-home__card kit-home__card--activity", "aria-labelledby": "{id}-activity",
                        header { class: "kit-home__card-head",
                            h2 { id: "{id}-activity", class: "kit-home__card-title", "Activity" }
                            if let Some((first, last)) = range.as_ref() {
                                p { class: "kit-home__card-note", "{total} chats, {first} to {last}" }
                            }
                        }
                        div { class: "kit-home__tip-slot",
                            if let Some(day) = tip().and_then(|index| activity_days.get(index)) {
                                {
                                    let text = format!("{}: {} chats", day.date, day.chat_count);
                                    rsx! {
                                        div { id: "{tip_id}", role: "tooltip", class: "kit-home__tip", "{text}" }
                                    }
                                }
                            }
                        }
                        div { id: "{grid_id}", class: "kit-home__grid", role: "group",
                            "aria-label": "Chats per day: {total} in total",
                            onkeydown: move |event: KeyboardEvent| {
                                if event.key() == Key::Escape {
                                    tip.set(None);
                                    return;
                                }
                                let next = move_index(focus(), activity_days.len(), &event.key());
                                if next != focus() {
                                    event.prevent_default();
                                    focus.set(next);
                                    tip.set(Some(next));
                                    crate::internal::focus_after_frame(&format!("{key_id}-day-{next}"));
                                }
                            },
                            for (week, column) in activity_days.chunks(WEEK).enumerate() {
                                div { key: "{week}", class: "kit-home__week",
                                    for (offset, day) in column.iter().enumerate() {
                                        {
                                            let index = week * WEEK + offset;
                                            let level = activity_level(day.chat_count, peak);
                                            let label = format!("{}: {} chats", day.date, day.chat_count);
                                            let dom = format!("{id}-day-{index}");
                                            let described = (tip() == Some(index)).then(|| tip_id.clone());
                                            rsx! {
                                                button {
                                                    key: "{index}",
                                                    id: "{dom}",
                                                    r#type: "button",
                                                    class: "kit-home__day",
                                                    "data-level": "{level}",
                                                    "aria-label": "{label}",
                                                    "aria-describedby": described,
                                                    tabindex: if focus() == index { "0" } else { "-1" },
                                                    onfocus: move |_| {
                                                        focus.set(index);
                                                        tip.set(Some(index));
                                                    },
                                                    onblur: move |_| {
                                                        if tip() == Some(index) {
                                                            tip.set(None);
                                                        }
                                                    },
                                                    onmouseenter: move |_| tip.set(Some(index)),
                                                    onmouseleave: move |_| tip.set(None),
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    section { class: "kit-home__card", "aria-labelledby": "{id}-recent",
                        h2 { id: "{id}-recent", class: "kit-home__card-title", "Recent chats" }
                        ul { class: "kit-home__rows",
                            for chat in recent_shown.iter() {
                                {
                                    let chat_id = chat.id.clone();
                                    let row_key = chat.id.clone();
                                    let title = chat.entry_title.clone();
                                    let when = chat.updated_at.clone();
                                    rsx! {
                                        li { key: "{row_key}",
                                            button {
                                                r#type: "button",
                                                class: "kit-home__row",
                                                onclick: move |_| {
                                                    if let Some(open) = on_open_chat {
                                                        open.call(chat_id.clone());
                                                    }
                                                },
                                                span { class: "kit-home__row-main", "{title}" }
                                                span { class: "kit-home__row-time", "{when}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if chats_more > 0 {
                            button {
                                r#type: "button",
                                class: "kit-home__more",
                                onclick: move |_| {
                                    if let Some(nav) = on_navigate {
                                        nav.call("Chat".to_string());
                                    }
                                },
                                "{chats_more} more"
                            }
                        }
                    }
                    section { class: "kit-home__card", "aria-labelledby": "{id}-usage",
                        h2 { id: "{id}-usage", class: "kit-home__card-title", "Usage" }
                        div { class: "kit-home__meter", role: "meter",
                            "aria-label": "Plan usage",
                            "aria-valuemin": "0",
                            "aria-valuemax": "{limit}",
                            "aria-valuenow": "{used}",
                            "aria-valuetext": "{meter_text}",
                            span { class: "kit-home__meter-fill", style: "width: {fill}" }
                        }
                        p { class: "kit-home__card-note", "{meter_text}" }
                        p { class: "kit-home__card-note", "{reset_text}" }
                    }
                    section { class: "kit-home__card", "aria-labelledby": "{id}-files",
                        h2 { id: "{id}-files", class: "kit-home__card-title", "Files made" }
                        ul { class: "kit-home__rows",
                            for file in files_shown.iter() {
                                {
                                    let name = file.entry_name.clone();
                                    let row_key = file.entry_name.clone();
                                    let short = middle_truncate(&file.entry_name, FILE_NAME_CHARS);
                                    let kind = file.extension.to_ascii_uppercase();
                                    let when = file.created_at.clone();
                                    rsx! {
                                        li { key: "{row_key}",
                                            button {
                                                r#type: "button",
                                                class: "kit-home__row",
                                                title: "{name}",
                                                onclick: move |_| {
                                                    if let Some(open) = on_open_file {
                                                        open.call(name.clone());
                                                    }
                                                },
                                                span { class: "kit-home__row-main", "{short}" }
                                                span { class: "kit-home__row-kind", "{kind}" }
                                                span { class: "kit-home__row-time", "{when}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if files_more > 0 {
                            button {
                                r#type: "button",
                                class: "kit-home__more",
                                onclick: move |_| {
                                    if let Some(nav) = on_navigate {
                                        nav.call("Folders".to_string());
                                    }
                                },
                                "{files_more} more"
                            }
                        }
                    }
                    section { class: "kit-home__card kit-home__card--month", "aria-labelledby": "{id}-month",
                        h2 { id: "{id}-month", class: "kit-home__card-title", "This month" }
                        div { class: "kit-home__month",
                            MetricTile {
                                metric_label: "Chats",
                                metric_value: MetricValue::Number(month_stats.chats.entry_value as f64),
                                change_percent: Some(month_stats.chats.change_vs_last as f64),
                                format_change: count_change as fn(f64) -> String,
                                change_note: Some("vs last month".to_string()),
                            }
                            MetricTile {
                                metric_label: "Files made",
                                metric_value: MetricValue::Number(month_stats.files_made.entry_value as f64),
                                change_percent: Some(month_stats.files_made.change_vs_last as f64),
                                format_change: count_change as fn(f64) -> String,
                                change_note: Some("vs last month".to_string()),
                            }
                        }
                    }
                }
            }
        }
    }
}
