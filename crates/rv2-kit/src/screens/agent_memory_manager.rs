//! Agent memory manager: what the agent has kept about a person and their work,
//! where each item came from and where it was used, and the controls to correct,
//! scope, confirm or delete it. Memories group under About you, then one group
//! per project.
//!
//! Design choice: Forget hides the row at once and shows an undo notice that
//! restores it at its old index. `on_forget` fires with the id; `on_restore`
//! fires with the memory and its index when the notice is used.
//! Design choice: the tree is a list of group buttons (`aria-expanded`) and row
//! buttons rather than `role=tree`. Up and down arrows move through both.
//! Design choice: a recall is detected when `last_used_at` moves forward between
//! renders; that row flashes for 1.6 s.
//! Design choice: "ago" times read from `clock_ms`; without it they read "now".
//! Design choice: at 640 px and below an opened memory replaces the list as a
//! full-width sheet with a Back control; focus returns to its row.
//! Design choice: text fields stop key events from reaching the shortcut
//! handler, so typing "n", "j" or "/" stays text.
//! Browser checks: Delete on a row and the undo notice restores it at its index;
//! Keep on To review calls `on_keep`; Escape restores edited text; Enter in the
//! Remember line creates a memory with source "user" at the top of About you;
//! J/K and arrows move; "/" focuses search; no horizontal scroll at 320 px.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, focus_after_frame, next_id, sleep, use_now_ms};

const CSS: Asset = asset!("/assets/agent-memory-manager.css");

const ABOUT_YOU: &str = "About you";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MemoryTab {
    #[default]
    Saved,
    Review,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceKind {
    Conversation,
    Decision,
    User,
    File,
}

impl SourceKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Conversation => "From a conversation",
            Self::Decision => "From a decision",
            Self::User => "You added",
            Self::File => "From a file",
        }
    }

    fn short(self) -> &'static str {
        match self {
            Self::Conversation => "Chat",
            Self::Decision => "Decision",
            Self::User => "You",
            Self::File => "File",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceInfo {
    pub source_kind: SourceKind,
    pub occurred_at: u64,
    pub quote_text: Option<String>,
    pub question_text: Option<String>,
    pub file_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UsageEntry {
    pub task_name: String,
    pub how: String,
    pub occurred_at: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryRecord {
    pub id: String,
    pub text: String,
    pub project_id: Option<String>,
    pub source_info: SourceInfo,
    pub use_count: u32,
    pub last_used_at: Option<u64>,
    pub usage_log: Vec<UsageEntry>,
    pub is_pending_review: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct Forgotten {
    memory: MemoryRecord,
    index: usize,
}

/// `now`, `5 min ago`, `3 h ago`, `2 d ago`; `never` when unused.
pub fn ago_text(now_ms: u64, then_ms: Option<u64>) -> String {
    let Some(then) = then_ms else {
        return "never".to_string();
    };
    let seconds = now_ms.saturating_sub(then) / 1000;
    match seconds {
        0..=59 => "now".to_string(),
        60..=3599 => format!("{} min ago", seconds / 60),
        3600..=86_399 => format!("{} h ago", seconds / 3600),
        _ => format!("{} d ago", seconds / 86_400),
    }
}

/// Whether a memory belongs on the tab.
pub fn on_tab(memory: &MemoryRecord, tab: MemoryTab) -> bool {
    match tab {
        MemoryTab::Saved => !memory.is_pending_review,
        MemoryTab::Review => memory.is_pending_review,
    }
}

/// Group headings in display order: About you first, then the projects in the
/// order given. A project with nothing on the tab is left out.
pub fn group_order(
    list: &[MemoryRecord],
    project_names: &[String],
) -> Vec<(String, Option<String>)> {
    let mut groups = Vec::new();
    if list.iter().any(|m| m.project_id.is_none()) {
        groups.push((ABOUT_YOU.to_string(), None));
    }
    for name in project_names {
        if list
            .iter()
            .any(|m| m.project_id.as_deref() == Some(name.as_str()))
        {
            groups.push((name.clone(), Some(name.clone())));
        }
    }
    groups
}

pub fn matches_query(memory: &MemoryRecord, query: &str) -> bool {
    let needle = query.trim().to_lowercase();
    needle.is_empty() || memory.text.to_lowercase().contains(&needle)
}

/// Ids of the rows that can be reached by the keyboard, in display order.
pub fn tree_order(
    list: &[MemoryRecord],
    project_names: &[String],
    collapsed: &[String],
    query: &str,
) -> Vec<String> {
    let mut ids = Vec::new();
    for (name, project) in group_order(list, project_names) {
        if collapsed.contains(&name) {
            continue;
        }
        for memory in list
            .iter()
            .filter(|m| m.project_id == project && matches_query(m, query))
        {
            ids.push(memory.id.clone());
        }
    }
    ids
}

/// The id `delta` rows from `current` in `order`, clamped at both ends.
pub fn move_index(order: &[String], current: Option<&str>, delta: isize) -> Option<String> {
    if order.is_empty() {
        return None;
    }
    let last = order.len() as isize - 1;
    let next = match current.and_then(|id| order.iter().position(|o| o == id)) {
        Some(index) => (index as isize + delta).clamp(0, last),
        None if delta >= 0 => 0,
        None => last,
    };
    Some(order[next as usize].clone())
}

pub fn memory_noun(count: usize) -> &'static str {
    if count == 1 { "memory" } else { "memories" }
}

/// Live-region and notice text after a Forget.
pub fn forget_notice(text: &str) -> String {
    format!("Forgotten: {text}")
}

/// A list of memories as a tree, with a panel for the open one, a Remember line
/// and a tab for memories waiting on review.
#[component]
pub fn AgentMemoryManager(
    #[props(default)] memory_list: Vec<MemoryRecord>,
    #[props(default)] project_names: Vec<String>,
    #[props(default)] initial_tab: MemoryTab,
    #[props(default)] initial_memory_id: Option<String>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] clock_ms: Option<u64>,
    #[props(default)] on_select: Option<EventHandler<String>>,
    #[props(default)] on_create: Option<EventHandler<MemoryRecord>>,
    #[props(default)] on_change: Option<EventHandler<MemoryRecord>>,
    #[props(default)] on_forget: Option<EventHandler<String>>,
    #[props(default)] on_restore: Option<EventHandler<(MemoryRecord, usize)>>,
    #[props(default)] on_keep: Option<EventHandler<String>>,
    #[props(default)] on_open_source: Option<EventHandler<MemoryRecord>>,
) -> Element {
    let now = use_now_ms(clock_ms, 1000);
    let uid = use_hook(next_id);
    let mut tab = use_signal(move || initial_tab);
    let mut query = use_signal(String::new);
    let mut open_id = use_signal(move || initial_memory_id.clone());
    let mut collapsed = use_signal(Vec::<String>::new);
    let mut hidden = use_signal(Vec::<String>::new);
    let mut kept = use_signal(Vec::<String>::new);
    let mut forgotten = use_signal(|| None::<Forgotten>);
    let mut remember_open = use_signal(|| false);
    let mut remember_text = use_signal(String::new);
    let mut edit = use_signal(|| None::<String>);
    let mut recalled = use_signal(Vec::<String>::new);
    let mut seen_use = use_signal(Vec::<(String, Option<u64>)>::new);
    let mut announce = use_signal(String::new);
    let mut sheet_open = use_signal(|| false);

    // A recall moves `last_used_at` forward between renders: flash that row once.
    let current_use: Vec<(String, Option<u64>)> = memory_list
        .iter()
        .map(|m| (m.id.clone(), m.last_used_at))
        .collect();
    if seen_use.peek().as_slice() != current_use.as_slice() {
        let previous = seen_use.peek().clone();
        for (id, used) in &current_use {
            let before = previous.iter().find(|(p, _)| p == id).and_then(|(_, u)| *u);
            if used.is_some() && *used != before && before.is_some() {
                recalled.with_mut(|list| list.push(id.clone()));
                let flash_id = id.clone();
                spawn(async move {
                    sleep(1600).await;
                    recalled.with_mut(|list| list.retain(|v| v != &flash_id));
                });
            }
        }
        seen_use.set(current_use);
    }

    let hidden_now = hidden();
    let kept_now = kept();
    let visible: Vec<(usize, MemoryRecord)> = memory_list
        .iter()
        .enumerate()
        .filter(|(_, m)| !hidden_now.contains(&m.id))
        .map(|(index, m)| {
            let mut m = m.clone();
            if kept_now.contains(&m.id) {
                m.is_pending_review = false;
            }
            (index, m)
        })
        .collect();
    let plain: Vec<MemoryRecord> = visible.iter().map(|(_, m)| m.clone()).collect();
    let saved_count = plain.iter().filter(|m| on_tab(m, MemoryTab::Saved)).count();
    let review_count = plain
        .iter()
        .filter(|m| on_tab(m, MemoryTab::Review))
        .count();
    let current_tab = tab();
    let on_current: Vec<MemoryRecord> = plain
        .iter()
        .filter(|m| on_tab(m, current_tab))
        .cloned()
        .collect();
    let collapsed_now = collapsed();
    let query_now = query();
    let order = tree_order(&on_current, &project_names, &collapsed_now, &query_now);
    let groups = group_order(&on_current, &project_names);
    let open_now = open_id();
    let open_entry = open_now
        .as_deref()
        .and_then(|id| visible.iter().find(|(_, m)| m.id == id).cloned());
    let edit_now = edit();
    let remember_now = remember_open();
    let remember_value = remember_text();
    let notice = forgotten();
    let recalled_now = recalled();
    let search_id = format!("kit-amm-{uid}-search");
    let remember_id = format!("kit-amm-{uid}-remember");
    let tree_id = format!("kit-amm-{uid}-tree");
    let tab_saved = format!("kit-amm-{uid}-saved");
    let tab_review = format!("kit-amm-{uid}-review");
    let sheet_now = sheet_open();
    let key_order = order.clone();
    let search_focus = search_id.clone();
    let remember_focus = remember_id.clone();
    let remember_input_id = format!("kit-amm-{uid}-remember-input");
    let remember_input_focus = remember_input_id.clone();
    let row_prefix = format!("kit-amm-{uid}-row-");

    let mut forget = move |memory: MemoryRecord, index: usize| {
        hidden.with_mut(|list| list.push(memory.id.clone()));
        announce.set(forget_notice(&memory.text));
        if open_id().as_deref() == Some(memory.id.as_str()) {
            open_id.set(None);
        }
        if let Some(handler) = on_forget {
            handler.call(memory.id.clone());
        }
        forgotten.set(Some(Forgotten { memory, index }));
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: "kit-amm",
            "data-loading": is_loading.then_some("true"),
            "aria-busy": is_loading.then_some("true"),
            "data-sheet": sheet_now.then_some("open"),
            onkeydown: move |event: KeyboardEvent| {
                let Key::Character(c) = event.key() else {
                    return;
                };
                match c.as_str() {
                    "/" => {
                        event.prevent_default();
                        focus_after_frame(&search_focus);
                    }
                    "n" => {
                        remember_open.set(true);
                        focus_after_frame(&remember_input_focus);
                    }
                    "j" | "k" => {
                        let delta = if c == "j" { 1 } else { -1 };
                        if let Some(next) = move_index(&key_order, open_id().as_deref(), delta) {
                            open_id.set(Some(next.clone()));
                            if let Some(handler) = on_select {
                                handler.call(next);
                            }
                        }
                    }
                    _ => {}
                }
            },
            header { class: "kit-amm__head",
                h2 { class: "kit-amm__title", "Memory" }
                div { class: "kit-amm__tabs", role: "tablist", "aria-label": "Memories",
                    button {
                        id: "{tab_saved}",
                        r#type: "button",
                        role: "tab",
                        class: "kit-amm__tab",
                        "aria-selected": (current_tab == MemoryTab::Saved).to_string(),
                        tabindex: if current_tab == MemoryTab::Saved { "0" } else { "-1" },
                        onclick: move |_| tab.set(MemoryTab::Saved),
                        "Saved "
                        span { class: "kit-amm__count", "{saved_count}" }
                    }
                    button {
                        id: "{tab_review}",
                        r#type: "button",
                        role: "tab",
                        class: "kit-amm__tab",
                        "aria-selected": (current_tab == MemoryTab::Review).to_string(),
                        tabindex: if current_tab == MemoryTab::Review { "0" } else { "-1" },
                        onclick: move |_| tab.set(MemoryTab::Review),
                        "To review "
                        span { class: "kit-amm__count", "{review_count}" }
                    }
                }
                div { class: "kit-amm__tools",
                    input {
                        id: "{search_id}",
                        class: "kit-amm__search",
                        r#type: "search",
                        placeholder: "Search memory",
                        "aria-label": "Search memory",
                        value: "{query_now}",
                        oninput: move |event| query.set(event.value()),
                        onkeydown: move |event: KeyboardEvent| {
                            event.stop_propagation();
                            if event.key() == Key::Escape {
                                query.set(String::new());
                            }
                        },
                    }
                    button {
                        id: "{remember_id}",
                        r#type: "button",
                        class: "kit-amm__primary",
                        "aria-expanded": remember_now.to_string(),
                        onclick: move |_| remember_open.toggle(),
                        "Remember"
                    }
                }
            }

            if remember_now {
                form {
                    class: "kit-amm__remember",
                    "aria-label": "Remember",
                    onsubmit: move |event: FormEvent| {
                        event.prevent_default();
                        let text = remember_text().trim().to_string();
                        if text.is_empty() {
                            return;
                        }
                        let created = MemoryRecord {
                            id: format!("new-{}", plain.len() + 1),
                            text,
                            project_id: None,
                            source_info: SourceInfo {
                                source_kind: SourceKind::User,
                                occurred_at: now,
                                quote_text: None,
                                question_text: None,
                                file_name: None,
                            },
                            use_count: 0,
                            last_used_at: None,
                            usage_log: Vec::new(),
                            is_pending_review: false,
                        };
                        if let Some(handler) = on_create {
                            handler.call(created);
                        }
                        announce.set("Remembered".to_string());
                        remember_text.set(String::new());
                    },
                    input {
                        id: "{remember_input_id}",
                        class: "kit-amm__input",
                        r#type: "text",
                        "aria-label": "What to remember",
                        placeholder: "Something to remember",
                        value: "{remember_value}",
                        oninput: move |event| remember_text.set(event.value()),
                        onkeydown: move |event: KeyboardEvent| {
                            event.stop_propagation();
                            if event.key() == Key::Escape {
                                event.prevent_default();
                                remember_open.set(false);
                                focus_after_frame(&remember_focus);
                            }
                        },
                    }
                }
            }

            if is_empty && !is_loading {
                div { class: "kit-amm__empty",
                    if current_tab == MemoryTab::Saved {
                        p { "Memories come from your conversations, decisions and files. Remember adds one yourself." }
                        button {
                            r#type: "button",
                            class: "kit-amm__primary",
                            onclick: move |_| remember_open.set(true),
                            "Remember"
                        }
                    } else {
                        p { "Nothing needs review." }
                    }
                }
            } else if is_loading {
                ul { class: "kit-amm__tree", "aria-hidden": "true",
                    for n in 0..4 {
                        li { key: "{n}", class: "kit-amm__ph" }
                    }
                }
            } else {
                div { class: "kit-amm__body",
                    ul { id: "{tree_id}", class: "kit-amm__tree", role: "list",
                        if order.is_empty() {
                            li { class: "kit-amm__none", "No memories match." }
                        }
                        for (name, project) in groups.iter().cloned() {
                            {
                                let group_key = name.clone();
                                let group_open = !collapsed_now.contains(&name);
                                let group_count = on_current.iter().filter(|m| m.project_id == project).count();
                                let group_label = format!("{name}, {group_count} {}", memory_noun(group_count));
                                let toggle_name = name.clone();
                                let chevron = if group_open { "▾" } else { "▸" };
                                let rows: Vec<(usize, MemoryRecord)> = visible
                                    .iter()
                                    .filter(|(_, m)| on_tab(m, current_tab) && m.project_id == project && matches_query(m, &query_now))
                                    .cloned()
                                    .collect();
                                rsx! {
                                    li { key: "g-{group_key}", class: "kit-amm__group",
                                        button {
                                            r#type: "button",
                                            class: "kit-amm__heading",
                                            "aria-expanded": "{group_open}",
                                            "aria-label": "{group_label}",
                                            onclick: move |_| {
                                                let mut next = collapsed();
                                                if next.contains(&toggle_name) {
                                                    next.retain(|v| v != &toggle_name);
                                                } else {
                                                    next.push(toggle_name.clone());
                                                }
                                                collapsed.set(next);
                                            },
                                            span { class: "kit-amm__chevron", "aria-hidden": "true", "{chevron}" }
                                            span { class: "kit-amm__group-name", "{name}" }
                                            span { class: "kit-amm__count", "{group_count}" }
                                        }
                                        if group_open {
                                            ul { class: "kit-amm__rows",
                                                for (index, memory) in rows {
                                                    {
                                                        let row_key = memory.id.clone();
                                                        let row_current = open_now.as_deref() == Some(memory.id.as_str());
                                                        let row_flash = recalled_now.contains(&memory.id);
                                                        let source = memory.source_info.source_kind.short();
                                                        let uses = memory.use_count.to_string();
                                                        let last = ago_text(now, memory.last_used_at);
                                                        let row_label = format!("{}, {source}, used {uses} times, last used {last}", memory.text);
                                                        let row_text = memory.text.clone();
                                                        let select_id = memory.id.clone();
                                                        let row_dom = format!("{row_prefix}{row_key}");
                                                        let delete_target = memory.clone();
                                                        rsx! {
                                                            li { key: "{row_key}", class: "kit-amm__item",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "kit-amm__row",
                                                                    "aria-current": row_current.then_some("true"),
                                                                    "data-recalled": row_flash.then_some("true"),
                                                                    id: "{row_dom}",
                                                                            "aria-label": "{row_label}",
                                                                    onclick: move |_| {
                                                                        open_id.set(Some(select_id.clone()));
                                                                                sheet_open.set(true);
                                                                        edit.set(None);
                                                                        if let Some(handler) = on_select {
                                                                            handler.call(select_id.clone());
                                                                        }
                                                                    },
                                                                    onkeydown: move |event: KeyboardEvent| {
                                                                        if event.key() == Key::Delete || event.key() == Key::Backspace {
                                                                            event.prevent_default();
                                                                            forget(delete_target.clone(), index);
                                                                        }
                                                                    },
                                                                    span { class: "kit-amm__text", "{row_text}" }
                                                                    span { class: "kit-amm__source", "{source}" }
                                                                    span { class: "kit-amm__uses", "{uses}" }
                                                                    span { class: "kit-amm__last", "{last}" }
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

                    if let Some((open_index, memory)) = open_entry.clone() {
                        {
                            let panel_text = edit_now.clone().unwrap_or_else(|| memory.text.clone());
                            let panel_scope = memory.project_id.clone().unwrap_or_default();
                            let panel_scope_label = if panel_scope.is_empty() { "Everywhere".to_string() } else { panel_scope.clone() };
                            let source_label = memory.source_info.source_kind.label();
                            let source_line = match memory.source_info.source_kind {
                                SourceKind::Conversation => memory.source_info.quote_text.clone(),
                                SourceKind::Decision => memory.source_info.question_text.clone(),
                                SourceKind::File => memory.source_info.file_name.clone(),
                                SourceKind::User => None,
                            }
                            .unwrap_or_default();
                            let saved_text = memory.text.clone();
                            let panel_label = format!("Memory: {}", memory.text);
                            let is_review = memory.is_pending_review;
                            let usage = memory.usage_log.clone();
                            let usage_empty = usage.is_empty();
                            let save_base = memory.clone();
                            let scope_base = memory.clone();
                            let source_target = memory.clone();
                            let keep_id = memory.id.clone();
                            let forget_target = memory.clone();
                            let forget_index = open_index;
                            let keep_target = memory.id.clone();
                            let back_target = format!("{row_prefix}{}", memory.id);
                            rsx! {
                                aside { class: "kit-amm__panel", "aria-label": "{panel_label}",
                                    button {
                                        r#type: "button",
                                        class: "kit-amm__back",
                                        onclick: move |_| {
                                            sheet_open.set(false);
                                            focus_after_frame(&back_target);
                                        },
                                        "Back to memories"
                                    }
                                    label { class: "kit-amm__field",
                                        span { "Memory" }
                                        textarea {
                                            class: "kit-amm__input",
                                            rows: "3",
                                            value: "{panel_text}",
                                            oninput: move |event| edit.set(Some(event.value())),
                                            onkeydown: move |event: KeyboardEvent| {
                                                event.stop_propagation();
                                                if event.key() == Key::Escape {
                                                    event.prevent_default();
                                                    edit.set(None);
                                                } else if event.key() == Key::Enter && !event.modifiers().shift() {
                                                    event.prevent_default();
                                                    let next_text = edit().unwrap_or_else(|| saved_text.clone());
                                                    edit.set(None);
                                                    let mut updated = save_base.clone();
                                                    updated.text = next_text;
                                                    if let Some(handler) = on_change {
                                                        handler.call(updated);
                                                    }
                                                }
                                            },
                                        }
                                    }
                                    label { class: "kit-amm__field",
                                        span { "Applies to" }
                                        select {
                                            class: "kit-amm__input",
                                            value: "{panel_scope}",
                                            onchange: move |event| {
                                                let mut updated = scope_base.clone();
                                                let value = event.value();
                                                updated.project_id = if value.is_empty() { None } else { Some(value) };
                                                if let Some(handler) = on_change {
                                                    handler.call(updated);
                                                }
                                            },
                                            option { value: "", "Everywhere" }
                                            for name in project_names.iter().cloned() {
                                                option { key: "{name}", value: "{name}", "{name}" }
                                            }
                                        }
                                    }
                                    p { class: "kit-amm__meta", "{panel_scope_label}" }
                                    section { class: "kit-amm__block", "aria-label": "Source",
                                        h3 { class: "kit-amm__section", "{source_label}" }
                                        if !source_line.is_empty() {
                                            blockquote { class: "kit-amm__quote", "{source_line}" }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "kit-amm__link",
                                            onclick: move |_| {
                                                if let Some(handler) = on_open_source {
                                                    handler.call(source_target.clone());
                                                }
                                            },
                                            "Open source"
                                        }
                                    }
                                    section { class: "kit-amm__block", "aria-label": "Usage",
                                        h3 { class: "kit-amm__section", "Used in" }
                                        if usage_empty {
                                            p { class: "kit-amm__meta", "Not used yet." }
                                        }
                                        ul { class: "kit-amm__usage",
                                            for (index, entry) in usage.iter().enumerate() {
                                                {
                                                    let usage_key = format!("{index}");
                                                    let usage_task = entry.task_name.clone();
                                                    let usage_how = entry.how.clone();
                                                    rsx! {
                                                        li { key: "{usage_key}",
                                                            span { "{usage_task}" }
                                                            span { class: "kit-amm__meta", "{usage_how}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    div { class: "kit-amm__actions",
                                        if is_review {
                                            button {
                                                r#type: "button",
                                                class: "kit-amm__primary",
                                                onclick: move |_| {
                                                    kept.with_mut(|list| list.push(keep_target.clone()));
                                                    announce.set("Kept: it will be used from now on".to_string());
                                                    if let Some(handler) = on_keep {
                                                        handler.call(keep_id.clone());
                                                    }
                                                },
                                                "Keep"
                                            }
                                            button {
                                                r#type: "button",
                                                class: "kit-amm__ghost",
                                                onclick: move |_| forget(forget_target.clone(), forget_index),
                                                "Don't keep"
                                            }
                                        } else {
                                            button {
                                                r#type: "button",
                                                class: "kit-amm__ghost",
                                                onclick: move |_| forget(forget_target.clone(), forget_index),
                                                "Forget"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(gone) = notice {
                {
                    let undo_text = forget_notice(&gone.memory.text);
                    let restore_target = gone.clone();
                    rsx! {
                        div { class: "kit-amm__notice", role: "status",
                            span { "{undo_text}" }
                            button {
                                r#type: "button",
                                class: "kit-amm__link",
                                onclick: move |_| {
                                    let Forgotten { memory, index } = restore_target.clone();
                                    hidden.with_mut(|list| list.retain(|v| v != &memory.id));
                                    forgotten.set(None);
                                    announce.set(format!("Restored: {}", memory.text));
                                    if let Some(handler) = on_restore {
                                        handler.call((memory, index));
                                    }
                                },
                                "Undo"
                            }
                        }
                    }
                }
            }

            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
