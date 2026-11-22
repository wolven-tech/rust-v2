//! Client workspace: one client's headline numbers, its sections, and a shared timeline of
//! goals, notes, files and reports that everyone on the account can read and add to.
//!
//! Design choice: stat cards are `MetricTile`s with a `SeriesTrendChart` sparkline; at most
//! four are shown.
//! Design choice: the `@` list suggests reports only, since a reference points at a report.
//! Design choice: an attachment is a file name chip; the bytes stay with the host.
//! Design choice: while `is_loading` is set, the stat row and the timeline are replaced by
//! skeletons inside a `role="status"` region, and the sync label reads loading.
//! Browser checks: workspace switch keeps the section, `@` list with Enter and Escape, Enter
//! posts and Shift+Enter does not, kind and search combine, new item at the top, sync label,
//! report disclosure, phone stat grid, loading skeletons.

use dioxus::prelude::*;
use rv2_affect::skeleton::{Loading, Skeleton};
use rv2_affect::switch::Switch;

use crate::blocks::metric_overview_board::{ListPicker, PickOption};

use crate::components::metric_tile::{MetricTile, MetricValue};
use crate::components::series_trend_chart::{
    AxisMode, ChartVariant, SeriesTone, SeriesTrendChart, TrendSeries,
};
use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/client-workspace.css");

/// Stat cards shown at most.
pub const MAX_STAT_CARDS: usize = 4;

#[derive(Clone, Debug, PartialEq)]
pub struct WorkspaceInfo {
    pub id: String,
    pub name: String,
    pub meta: Vec<String>,
    pub synced_at_ms: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkspaceOption {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatCard {
    pub id: String,
    pub label: String,
    pub value: String,
    pub delta_percent: Option<f64>,
    pub series: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkspaceSection {
    pub id: String,
    pub label: String,
    pub count: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    Note,
    Report,
    File,
    Goal,
}

impl ItemKind {
    pub const ALL: [ItemKind; 4] = [Self::Goal, Self::Note, Self::Report, Self::File];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Report => "report",
            Self::File => "file",
            Self::Goal => "goal",
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::Note => "Note",
            Self::Report => "Report",
            Self::File => "File",
            Self::Goal => "Goal",
        }
    }

    pub fn action(self) -> &'static str {
        match self {
            Self::Note => "added a note",
            Self::Report => "generated a report",
            Self::File => "shared a file",
            Self::Goal => "set a goal",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reference {
    pub label: String,
    pub target: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineItem {
    pub id: String,
    pub kind: ItemKind,
    pub author: String,
    pub at_ms: u64,
    pub title: Option<String>,
    pub text: String,
    pub references: Vec<Reference>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NoteSubmit {
    pub text: String,
    pub references: Vec<Reference>,
    pub attachments: Vec<String>,
}

/// Items of `kind` (or any kind) whose text or reference labels contain `search`, newest
/// first. Both filters must pass.
pub fn visible_items(
    items: &[TimelineItem],
    kind: Option<ItemKind>,
    search: &str,
) -> Vec<TimelineItem> {
    let needle = search.trim().to_lowercase();
    let mut out: Vec<TimelineItem> = items
        .iter()
        .filter(|i| kind.is_none_or(|k| i.kind == k))
        .filter(|i| {
            needle.is_empty()
                || i.text.to_lowercase().contains(&needle)
                || i.references
                    .iter()
                    .any(|r| r.label.to_lowercase().contains(&needle))
        })
        .cloned()
        .collect();
    out.sort_by_key(|i| std::cmp::Reverse(i.at_ms));
    out
}

/// Reports whose title starts with the text typed after the last `@`, if the draft ends in
/// an open mention.
pub fn mention_matches(items: &[TimelineItem], draft: &str) -> Option<Vec<Reference>> {
    let at = draft.rfind('@')?;
    let partial = &draft[at + 1..];
    if partial.contains(char::is_whitespace) {
        return None;
    }
    let partial = partial.to_lowercase();
    Some(
        items
            .iter()
            .filter(|i| i.kind == ItemKind::Report)
            .map(|i| Reference {
                label: i.title.clone().unwrap_or_else(|| i.text.clone()),
                target: i.id.clone(),
            })
            .filter(|r| r.label.to_lowercase().starts_with(&partial))
            .collect(),
    )
}

/// `just now`, `4 min ago`, `3 h ago`, `2 days ago`.
pub fn relative_age(age_ms: u64) -> String {
    let secs = age_ms / 1000;
    match secs {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{} min ago", secs / 60),
        3600..=86_399 => format!("{} h ago", secs / 3600),
        _ => format!("{} days ago", secs / 86_400),
    }
}

/// Replaces `@partial` at the end of the draft with `@title `.
pub fn insert_reference(draft: &str, title: &str) -> String {
    match draft.rfind('@') {
        Some(at) => format!("{}@{title} ", &draft[..at]),
        None => draft.to_string(),
    }
}

#[component]
pub fn ClientWorkspace(
    workspace: WorkspaceInfo,
    workspace_options: Vec<WorkspaceOption>,
    stat_cards: Vec<StatCard>,
    sections: Vec<WorkspaceSection>,
    #[props(default)] active_section: Option<String>,
    #[props(default)] on_section_change: Option<EventHandler<String>>,
    timeline_items: Vec<TimelineItem>,
    kind_options: Vec<ItemKind>,
    #[props(default)] selected_kind: Option<ItemKind>,
    #[props(default)] on_kind_change: Option<EventHandler<Option<ItemKind>>>,
    now_ms: u64,
    #[props(default)] is_syncing: bool,
    #[props(default)] is_loading: bool,
    #[props(default)] on_switch_workspace: Option<EventHandler<String>>,
    #[props(default)] on_submit_note: Option<EventHandler<NoteSubmit>>,
    #[props(default)] on_open_reference: Option<EventHandler<Reference>>,
    #[props(default)] on_search_text: Option<EventHandler<String>>,
    #[props(default)] on_refresh: Option<EventHandler<()>>,
    #[props(default, into)] extra_classes: String,
) -> Element {
    let mut focused_stat = use_signal(|| None::<String>);
    let mut expanded_report = use_signal(|| None::<String>);
    let mut local_section = use_signal(|| None::<String>);
    let mut draft = use_signal(String::new);
    let mut mention_index = use_signal(|| 0_usize);
    let mut mention_closed = use_signal(|| false);
    let mut attachments = use_signal(Vec::<String>::new);
    let mut search = use_signal(String::new);

    let section_id = active_section
        .clone()
        .or_else(|| local_section())
        .or_else(|| sections.first().map(|s| s.id.clone()))
        .unwrap_or_default();
    let is_settings = section_id == "settings";
    let search_now = search();
    let shown = visible_items(&timeline_items, selected_kind, &search_now);
    let mentions: Vec<Reference> = if mention_closed() {
        Vec::new()
    } else {
        mention_matches(&timeline_items, &draft()).unwrap_or_default()
    };
    let sync_label = if is_loading {
        "Loading…".to_string()
    } else if is_syncing {
        "Syncing…".to_string()
    } else {
        format!(
            "Synced {}",
            relative_age(now_ms.saturating_sub(workspace.synced_at_ms))
        )
    };
    let root_class = class_list("kit-ws", &extra_classes);
    let cards: Vec<StatCard> = stat_cards.iter().take(MAX_STAT_CARDS).cloned().collect();
    let focus_key = focused_stat();
    let section_count = shown.len();
    let count_text = if is_loading {
        "Loading…".to_string()
    } else {
        format!("{section_count} items")
    };
    let current_section_label = sections
        .iter()
        .find(|s| s.id == section_id)
        .map(|s| s.label.clone())
        .unwrap_or_default();
    let has_draft = !draft().trim().is_empty();
    let meta_line = workspace.meta.join(" · ");

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{root_class}", "aria-label": "Client workspace",
            header { class: "kit-ws__head",
                div { class: "kit-ws__identity",
                    h2 { class: "kit-ws__name", "{workspace.name}" }
                    if !meta_line.is_empty() {
                        p { class: "kit-ws__meta", "{meta_line}" }
                    }
                }
                div { class: "kit-ws__tools",
                    span { class: "kit-ws__sync", role: "status", "{sync_label}" }
                    button {
                        r#type: "button",
                        class: "kit-ws__refresh",
                        "aria-busy": if is_syncing { "true" } else { "false" },
                        "aria-label": "Refresh",
                        onclick: move |_| {
                            if let Some(handler) = on_refresh { handler.call(()); }
                        },
                        span { class: "kit-ws__spin", "aria-hidden": "true", "↻" }
                    }
                    div { class: "kit-ws__switch",
                        ListPicker {
                            options: workspace_options.iter().map(|o| PickOption { id: o.id.clone(), label: o.name.clone() }).collect::<Vec<_>>(),
                            selected: Some(workspace.id.clone()),
                            label: "Switch client",
                            on_change: move |id: String| {
                                if let Some(handler) = on_switch_workspace { handler.call(id); }
                            },
                        }
                    }
                }
            }
            if is_loading {
                Loading { label: "Loading headline numbers", class: "kit-ws__stats",
                    for _ in 0..MAX_STAT_CARDS {
                        Skeleton { height: "84px" }
                    }
                }
            } else {
            div { class: "kit-ws__stats", role: "group", "aria-label": "Headline numbers",
                for card in cards.iter() {
                    {
                        let id = card.id.clone();
                        let chosen = focus_key.as_deref() == Some(card.id.as_str());
                        let dim = focus_key.is_some() && !chosen;
                        let x_labels: Vec<String> = (0..card.series.len()).map(|i| i.to_string()).collect();
                        let spark = TrendSeries { name: card.label.clone(), values: card.series.iter().map(|v| Some(*v)).collect(), tone: SeriesTone::Ink, emphasis: true, is_earlier: false };
                        rsx! {
                            div { key: "{card.id}", class: "kit-ws__stat", "data-dimmed": dim.then_some("true"),
                                MetricTile {
                                    metric_label: card.label.clone(),
                                    metric_value: MetricValue::Text(card.value.clone()),
                                    change_percent: card.delta_percent,
                                    is_chosen: Some(chosen),
                                    target_attributes: vec![Attribute::new("aria-pressed", if chosen { "true" } else { "false" }, None, false)],
                                    on_activate: move |_| focused_stat.set(if chosen { None } else { Some(id.clone()) }),
                                    inline_chart: rsx! {
                                        SeriesTrendChart {
                                            x_labels: x_labels,
                                            series: vec![spark],
                                            variant: ChartVariant::Sparkline,
                                            axis: AxisMode::Off,
                                            show_tooltip: false,
                                            height: 36.0,
                                            label: card.label.clone(),
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
            }
            nav { class: "kit-ws__sections", role: "tablist", "aria-label": "Workspace sections",
                for section in sections.iter() {
                    {
                        let sel = section.id == section_id;
                        let sid = section.id.clone();
                        let count = section.count.map(|c| format!(" {c}")).unwrap_or_default();
                        rsx! {
                            button {
                                key: "{section.id}",
                                r#type: "button",
                                role: "tab",
                                class: "kit-ws__tab",
                                "aria-selected": if sel { "true" } else { "false" },
                                tabindex: if sel { "0" } else { "-1" },
                                onclick: move |_| {
                                    local_section.set(Some(sid.clone()));
                                    if let Some(handler) = on_section_change { handler.call(sid.clone()); }
                                },
                                "{section.label}{count}"
                            }
                        }
                    }
                }
            }
            div { class: "kit-ws__bar",
                span { class: "kit-ws__count", "{count_text}" }
                if !is_settings {
                    input {
                        class: "kit-ws__search",
                        r#type: "search",
                        "aria-label": "Search timeline",
                        placeholder: "Search",
                        value: "{search_now}",
                        oninput: move |e: FormEvent| {
                            search.set(e.value());
                            if let Some(handler) = on_search_text { handler.call(e.value()); }
                        },
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Escape && !search().is_empty() {
                                search.set(String::new());
                            }
                        },
                    }
                    select {
                        class: "kit-ws__kind",
                        "aria-label": "Filter by kind",
                        onchange: move |e: FormEvent| {
                            let value = e.value();
                            let kind = ItemKind::ALL.into_iter().find(|k| k.as_str() == value);
                            if let Some(handler) = on_kind_change { handler.call(kind); }
                        },
                        option { value: "all", selected: selected_kind.is_none(), "All kinds" }
                        for kind in kind_options.iter().copied() {
                            option { key: "{kind.as_str()}", value: "{kind.as_str()}", selected: selected_kind == Some(kind), "{kind.word()}" }
                        }
                    }
                }
            }
            if is_settings {
                section { class: "kit-ws__settings", "aria-label": "{current_section_label}",
                    Switch { label: rsx! { "Share reports with the whole account" }, description: rsx! { "Everyone on the account can open reports." } }
                    Switch { label: rsx! { "Notify on new notes" }, description: rsx! { "A note posted here sends an alert." } }
                }
            } else {
                div { class: "kit-ws__pane", role: "tabpanel", "aria-label": "{current_section_label}",
                    form { class: "kit-ws__composer", onsubmit: move |e: FormEvent| e.prevent_default(),
                        if !mentions.is_empty() {
                            ul { class: "kit-ws__mentions", role: "listbox", "aria-label": "Reports",
                                for (i, r) in mentions.iter().enumerate() {
                                    li { key: "{r.target}", role: "option", "aria-selected": if i == mention_index() { "true" } else { "false" },
                                        onclick: {
                                            let title = r.label.clone();
                                            move |_| {
                                                draft.set(insert_reference(&draft(), &title));
                                                mention_closed.set(true);
                                            }
                                        },
                                        "{r.label}"
                                    }
                                }
                            }
                        }
                        textarea {
                            class: "kit-ws__input",
                            rows: "2",
                            "aria-label": "Add a note",
                            placeholder: "Add a goal, note or reference. Type @ to cite a report.",
                            value: "{draft}",
                            oninput: move |e: FormEvent| {
                                draft.set(e.value());
                                mention_closed.set(false);
                                mention_index.set(0);
                            },
                            onkeydown: {
                                let refs = mentions.clone();
                                move |e: KeyboardEvent| {
                                    let open = !refs.is_empty();
                                    if open {
                                        match e.key() {
                                            Key::ArrowDown => { e.prevent_default(); mention_index.set((mention_index() + 1) % refs.len()); }
                                            Key::ArrowUp => { e.prevent_default(); mention_index.set((mention_index() + refs.len() - 1) % refs.len()); }
                                            Key::Enter if !e.modifiers().shift() => {
                                                e.prevent_default();
                                                let pick = refs[mention_index().min(refs.len() - 1)].label.clone();
                                                draft.set(insert_reference(&draft(), &pick));
                                                mention_closed.set(true);
                                            }
                                            Key::Escape => { e.prevent_default(); mention_closed.set(true); }
                                            _ => {}
                                        }
                                        return;
                                    }
                                    if e.key() == Key::Enter && !e.modifiers().shift() {
                                        e.prevent_default();
                                        let text = draft().trim().to_string();
                                        if text.is_empty() { return; }
                                        let references: Vec<Reference> = text
                                            .match_indices('@')
                                            .filter_map(|(at, _)| {
                                                let rest = &text[at + 1..];
                                                let end = rest.find(' ').unwrap_or(rest.len());
                                                let label = &rest[..end];
                                                timeline_items
                                                    .iter()
                                                    .filter(|i| i.kind == ItemKind::Report)
                                                    .find(|i| i.title.as_deref().unwrap_or(&i.text).starts_with(label))
                                                    .map(|i| Reference { label: i.title.clone().unwrap_or_else(|| i.text.clone()), target: i.id.clone() })
                                            })
                                            .collect();
                                        let files = attachments();
                                        draft.set(String::new());
                                        attachments.set(Vec::new());
                                        if let Some(handler) = on_submit_note {
                                            handler.call(NoteSubmit { text, references, attachments: files });
                                        }
                                    }
                                }
                            },
                        }
                        div { class: "kit-ws__actions",
                            label { class: "kit-ws__attach",
                                "Attach"
                                input {
                                    r#type: "file",
                                    multiple: true,
                                    class: "kit-ws__file",
                                    onchange: move |e: FormEvent| {
                                        let names: Vec<String> = e.files().iter().map(|f| f.name()).collect();
                                        attachments.with_mut(|list| list.extend(names));
                                    },
                                }
                            }
                            button {
                                r#type: "submit",
                                class: "kit-ws__submit",
                                disabled: !has_draft,
                                onclick: move |e: MouseEvent| {
                                    e.prevent_default();
                                    let text = draft().trim().to_string();
                                    if text.is_empty() { return; }
                                    let files = attachments();
                                    draft.set(String::new());
                                    attachments.set(Vec::new());
                                    if let Some(handler) = on_submit_note {
                                        handler.call(NoteSubmit { text, references: Vec::new(), attachments: files });
                                    }
                                },
                                "Post"
                            }
                        }
                        if !attachments().is_empty() {
                            ul { class: "kit-ws__chips",
                                for name in attachments().iter() {
                                    {
                                        let remove = name.clone();
                                        rsx! {
                                            li { key: "{name}", class: "kit-ws__chip",
                                                span { "{name}" }
                                                button {
                                                    r#type: "button",
                                                    "aria-label": "Remove {name}",
                                                    onclick: move |_| attachments.with_mut(|list| list.retain(|n| *n != remove)),
                                                    "×"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if is_loading {
                        Loading { label: "Loading timeline", class: "kit-ws__timeline",
                            for _ in 0..3 {
                                div { class: "kit-ws__item kit-ws__item--skeleton",
                                    Skeleton { width: "32px", height: "32px" }
                                    div { class: "kit-ws__body",
                                        Skeleton { width: "40%", height: "14px" }
                                        Skeleton { width: "90%", height: "12px" }
                                    }
                                }
                            }
                        }
                    } else {
                    if shown.is_empty() {
                        p { class: "kit-ws__empty", "Nothing here yet. Add the first goal or note above." }
                    }
                    ol { class: "kit-ws__timeline", "aria-label": "Timeline",
                        for item in shown.iter() {
                            {
                                let open = expanded_report() == Some(item.id.clone());
                                let is_report = item.kind == ItemKind::Report;
                                let initial = item.author.chars().next().unwrap_or('?').to_string();
                                let age = relative_age(now_ms.saturating_sub(item.at_ms));
                                let id = item.id.clone();
                                let refs = item.references.clone();
                                let body = item.text.clone();
                                let title = item.title.clone().unwrap_or_else(|| item.kind.word().to_string());
                                let sentence = format!("{} {}", item.author, item.kind.action());
                                rsx! {
                                    li { key: "{item.id}", class: "kit-ws__item", "data-kind": item.kind.as_str(),
                                        div { class: "kit-ws__avatar", "aria-hidden": "true", "{initial}" }
                                        div { class: "kit-ws__body",
                                            header { class: "kit-ws__item-head",
                                                span { class: "kit-ws__kind-badge", "{item.kind.word()}" }
                                                span { class: "kit-ws__sentence", "{sentence}" }
                                                time { class: "kit-ws__age", "{age}" }
                                            }
                                            if is_report {
                                                button {
                                                    r#type: "button",
                                                    class: "kit-ws__disclosure",
                                                    "aria-expanded": if open { "true" } else { "false" },
                                                    onclick: move |_| expanded_report.set(if open { None } else { Some(id.clone()) }),
                                                    "{title}"
                                                }
                                                if open {
                                                    p { class: "kit-ws__text", "{body}" }
                                                }
                                            } else {
                                                p { class: "kit-ws__text", "{body}" }
                                            }
                                            if !refs.is_empty() {
                                                div { class: "kit-ws__refs",
                                                    for r in refs.iter() {
                                                        {
                                                            let target = r.clone();
                                                            rsx! {
                                                                button {
                                                                    key: "{r.target}",
                                                                    r#type: "button",
                                                                    class: "kit-ws__ref",
                                                                    onclick: move |_| {
                                                                        if let Some(handler) = on_open_reference { handler.call(target.clone()); }
                                                                    },
                                                                    "{r.label}"
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
            }
        }
    }
}
