//! A chat with an orchestrating agent that ends in a concrete task. Beside the conversation, the
//! agents at work are shown with their state, and a brief for the current goal is filled in alongside.
//!
//! Design choice: the Generate line states what it writes. The duration is host data (`generate_eta`),
//! since the component does not know how long a draft takes.
//! Design choice: the Brief is a closable tab like any other. Closing the last tab folds the work area,
//! and opening it again shows the Brief.
//! Design choice: a file opens in its own tab with the shared artifact viewer from the workspace frame.
//! Design choice: chat list grouping reuses `group_chats` from the assistant chat screen.
//! Design choice: below 960px the chat list is a drawer behind a Chats control, closed by default.
//! Not implemented: the sidebar's navigation icons and the global search (Ctrl or Cmd+K), which belong
//! to the app frame.
//! Browser checks: Left and Right move between tabs; closing the last tab folds the area and focuses
//! its open control; each agent's state reads in words.

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonVariant};
use rv2_affect::skeleton::Skeleton;

use super::assistant_chat_sidebars::{ApprovalMode, ChatItem, Project, group_chats, policy_for};
use super::assistant_start_page::{active_after_close, tab_after_move};
use super::workspace_frame_inset::{Artifact, ArtifactView};
use crate::components::prompt_input_bar::{Model, PromptInputBar, Submission};
use crate::internal::{KIT_CSS, focus_after_frame, next_id};

const CSS: Asset = asset!("/assets/workspace-chat.css");
const FRAME_CSS: Asset = asset!("/assets/workspace-frame-inset.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentState {
    Idle,
    Working,
    Waiting,
    Done,
}

impl AgentState {
    pub fn word(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Working => "Working",
            Self::Waiting => "Waiting for you",
            Self::Done => "Done",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentStatus {
    pub id: String,
    pub entry_name: String,
    pub node_role: String,
    pub entry_state: AgentState,
    pub current_step: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AgentMessage {
    User {
        id: String,
        text: String,
    },
    Agent {
        id: String,
        author: String,
        text: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldType {
    Text,
    Choice,
    MultiChoice,
    Number,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BriefField {
    pub id: String,
    pub entry_label: String,
    pub entry_value: String,
    pub field_type: FieldType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    Document,
    Sheet,
    Folder,
}

impl FileKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Document => "Document",
            Self::Sheet => "Sheet",
            Self::Folder => "Folder",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BriefFile {
    pub entry_name: String,
    pub file_kind: FileKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActivityEntry {
    pub at: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Brief {
    pub entry_title: String,
    pub field_list: Vec<BriefField>,
    pub file_list: Vec<BriefFile>,
    pub activity_log: Vec<ActivityEntry>,
}

/// Tab keys: the Brief, or an open file by name.
pub fn brief_key() -> String {
    "brief".to_string()
}

/// Tab titles for keys, so the list can name each tab.
pub fn tab_title(key: &str, brief: &Brief) -> String {
    if key == "brief" {
        brief.entry_title.clone()
    } else {
        key.strip_prefix("file:").unwrap_or(key).to_string()
    }
}

/// Lines the Brief shows where a section is empty. Each names what will appear there.
pub fn empty_section_line(section: &str) -> &'static str {
    match section {
        "fields" => "Fields you fill in for this goal appear here.",
        "files" => "Documents and sheets the agent makes appear here.",
        _ => "The agent's steps and decisions are logged here.",
    }
}

/// The chat with its agents and brief.
#[component]
pub fn WorkspaceChat(
    #[props(into)] workspace_name: String,
    #[props(default, into)] account_name: String,
    #[props(default)] chat_list: Vec<ChatItem>,
    #[props(default)] project_list: Vec<Project>,
    #[props(default)] active_chat_id: Option<String>,
    #[props(default)] message_list: Vec<AgentMessage>,
    #[props(default)] agent_status_list: Vec<AgentStatus>,
    #[props(default)] orchestrator_status: String,
    #[props(default)] brief_record: Option<Brief>,
    #[props(default)] artifact_list: Vec<Artifact>,
    #[props(default, into)] generate_eta: Option<String>,
    #[props(default)] approval_mode: ApprovalMode,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    on_send: EventHandler<String>,
    on_select_chat: EventHandler<String>,
    #[props(default)] on_field_change: Option<EventHandler<(String, String)>>,
    #[props(default)] on_generate: Option<EventHandler<()>>,
    #[props(default)] on_open_file: Option<EventHandler<String>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
) -> Element {
    let id = use_hook(|| format!("kit-wc-{}", next_id()));
    let open_id = format!("{id}-open");
    let side_id = format!("{id}-side");
    let mut side_open = use_signal(|| false);
    let mut work_open = use_signal(|| true);
    let mut tabs = use_signal(|| vec![brief_key()]);
    let mut active = use_signal(|| 0usize);
    let brief = brief_record.clone();
    let models = vec![Model {
        model_id: "standard".to_string(),
        model_name: "Standard".to_string(),
        model_mark: None,
        model_note: None,
    }];
    let groups = group_chats(&chat_list.iter().collect::<Vec<_>>(), &project_list);
    let initial = workspace_name.chars().take(1).collect::<String>();
    let account_initial = account_name.chars().take(1).collect::<String>();
    let tabs_now = tabs();
    let active_now = active().min(tabs_now.len().saturating_sub(1));
    let key_id = id.clone();
    let key_open = open_id.clone();
    let loading = is_loading;
    let empty = is_empty;
    let brief_title = brief
        .as_ref()
        .map(|b| b.entry_title.clone())
        .unwrap_or_default();
    let line_fields = empty_section_line("fields");
    let line_files = empty_section_line("files");
    let line_activity = empty_section_line("activity");
    let active_key = tabs_now.get(active_now).cloned().unwrap_or_default();
    let file_name = active_key.strip_prefix("file:").map(str::to_string);
    let file_artifact = file_name
        .as_ref()
        .and_then(|name| artifact_list.iter().find(|a| &a.name == name).cloned());
    let side_state = if side_open() { "open" } else { "closed" };
    let menu_id = format!("{id}-menu");
    let menu_target = menu_id.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        document::Link { rel: "stylesheet", href: FRAME_CSS }
        div { class: "kit-wc", "data-side": side_state,
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape && side_open() {
                    event.prevent_default();
                    side_open.set(false);
                    focus_after_frame(&menu_target);
                }
            },
            aside { id: "{side_id}", class: "kit-wc__side", "aria-label": "Workspace navigation",
                div { class: "kit-wc__head",
                    span { class: "kit-wc__mark", "aria-hidden": "true", "{initial}" }
                    span { class: "kit-wc__name", "{workspace_name}" }
                }
                nav { class: "kit-wc__nav", "aria-label": "Sections",
                    for entry in ["New chat", "Folders", "Integrations"] {
                        {
                            let target = entry.to_string();
                            rsx! {
                                button {
                                    key: "{entry}",
                                    r#type: "button",
                                    class: "kit-wc__nav-item",
                                    onclick: move |_| {
                                        if let Some(nav) = on_navigate {
                                            nav.call(target.clone());
                                        }
                                    },
                                    "{entry}"
                                }
                            }
                        }
                    }
                }
                if loading {
                    Skeleton { width: "100%", height: "28px" }
                    Skeleton { width: "80%", height: "28px" }
                } else if chat_list.is_empty() {
                    p { class: "kit-wc__note", "No chats yet." }
                } else {
                    for (heading, members) in groups.iter() {
                        {
                            let title = heading.clone();
                            rsx! {
                                h2 { class: "kit-wc__heading", "{title}" }
                                ul { class: "kit-wc__chats",
                                    for chat in members.iter() {
                                        {
                                            let key = chat.id.clone();
                                            let pick = chat.id.clone();
                                            let label = chat.entry_title.clone();
                                            let current = active_chat_id.as_deref() == Some(chat.id.as_str());
                                            rsx! {
                                                li { key: "{key}",
                                                    button {
                                                        r#type: "button",
                                                        class: "kit-wc__chat",
                                                        "aria-current": current.then_some("true"),
                                                        onclick: move |_| {
                                                            side_open.set(false);
                                                            on_select_chat.call(pick.clone());
                                                        },
                                                        "{label}"
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
                div { class: "kit-wc__account", "aria-hidden": "true", "{account_initial}" }
            }
            section { class: "kit-wc__conversation", "aria-label": "Conversation",
                button {
                    id: "{menu_id}",
                    r#type: "button",
                    class: "kit-wc__menu",
                    "aria-expanded": if side_open() { "true" } else { "false" },
                    "aria-controls": "{side_id}",
                    onclick: move |_| side_open.toggle(),
                    "Chats"
                }
                if !agent_status_list.is_empty() {
                    ul { class: "kit-wc__agents", "aria-label": "Agents",
                        for agent in agent_status_list.iter() {
                            {
                                let key = agent.id.clone();
                                let name = agent.entry_name.clone();
                                let role = agent.node_role.clone();
                                let word = agent.entry_state.word();
                                let step = agent.current_step.clone().unwrap_or_default();
                                rsx! {
                                    li { key: "{key}", class: "kit-wc__agent", "data-state": "{word}",
                                        span { class: "kit-wc__agent-name", "{name}" }
                                        span { class: "kit-wc__agent-role", "{role}" }
                                        span { class: "kit-wc__agent-state", "{word}" }
                                        if !step.is_empty() {
                                            span { class: "kit-wc__agent-step", "{step}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "kit-wc__thread", role: "log", "aria-live": "polite", "aria-busy": loading.then_some("true"),
                    if loading {
                        Skeleton { width: "60%", height: "40px" }
                        Skeleton { width: "100%", height: "120px" }
                    } else if empty {
                        div { class: "kit-wc__empty",
                            p { class: "kit-wc__note", "No chats yet. Start one to give the agents a goal." }
                            Button {
                                variant: ButtonVariant::Secondary,
                                label: "Start a chat",
                                onclick: move |_| {
                                    if let Some(nav) = on_navigate {
                                        nav.call("New chat".to_string());
                                    }
                                },
                            }
                        }
                    } else {
                        for message in message_list.iter() {
                            {
                                match message {
                                    AgentMessage::User { id: key, text } => rsx! {
                                        div { key: "{key}", class: "kit-wc__msg kit-wc__msg--user", p { "{text}" } }
                                    },
                                    AgentMessage::Agent { id: key, author, text } => rsx! {
                                        div { key: "{key}", class: "kit-wc__msg",
                                            span { class: "kit-wc__author", "{author}" }
                                            p { "{text}" }
                                        }
                                    },
                                }
                            }
                        }
                        if !orchestrator_status.is_empty() {
                            p { class: "kit-wc__status", "{orchestrator_status}" }
                        }
                    }
                }
                div { class: "kit-wc__composer",
                    PromptInputBar {
                        root_class: "kit-wc__input",
                        empty_hint: "Give the agents a goal",
                        model_options: models,
                        approval_policy: policy_for(approval_mode),
                        on_submit: move |submission: Submission| {
                            if !empty && !loading && !submission.text.trim().is_empty() {
                                on_send.call(submission.text);
                            }
                        },
                    }
                }
            }
            if work_open() {
                aside { class: "kit-wc__work", "aria-label": "Work area",
                    div { class: "kit-wc__tabs", role: "tablist", "aria-label": "Open items",
                        onkeydown: move |event: KeyboardEvent| {
                            let count = tabs.peek().len();
                            let index = (*active.peek()).min(count.saturating_sub(1));
                            match event.key() {
                                Key::ArrowRight | Key::ArrowLeft => {
                                    event.prevent_default();
                                    let next = tab_after_move(index, count, event.key() == Key::ArrowRight);
                                    active.set(next);
                                    focus_after_frame(&format!("{key_id}-tab-{next}"));
                                }
                                Key::Delete => {
                                    event.prevent_default();
                                    close_at(&mut tabs, &mut active, &mut work_open, index, &key_open);
                                }
                                _ => {}
                            }
                        },
                        for (index, key) in tabs_now.iter().enumerate() {
                            {
                                let title = match &brief {
                                    Some(record) => tab_title(key, record),
                                    None => key.strip_prefix("file:").unwrap_or(key).to_string(),
                                };
                                let selected = index == active_now;
                                let dom = format!("{id}-tab-{index}");
                                let close_label = format!("Close {title}");
                                let close_open = open_id.clone();
                                rsx! {
                                    div { key: "{key}", class: "kit-wc__tab", "data-selected": selected.then_some("true"),
                                        button {
                                            id: "{dom}",
                                            r#type: "button",
                                            role: "tab",
                                            "aria-selected": if selected { "true" } else { "false" },
                                            tabindex: if selected { "0" } else { "-1" },
                                            onclick: move |_| active.set(index),
                                            "{title}"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "kit-wc__tab-close",
                                            "aria-label": "{close_label}",
                                            onclick: move |_| close_at(&mut tabs, &mut active, &mut work_open, index, &close_open),
                                            "×"
                                        }
                                    }
                                }
                            }
                        }
                        button {
                            r#type: "button",
                            class: "kit-wc__tab-new",
                            "aria-label": "Open the brief",
                            onclick: move |_| {
                                let mut list = tabs.peek().clone();
                                if !list.contains(&brief_key()) {
                                    list.insert(0, brief_key());
                                }
                                tabs.set(list);
                                active.set(0);
                            },
                            "+"
                        }
                    }
                    div { class: "kit-wc__panel", role: "tabpanel",
                        if let Some(name) = file_name.clone() {
                            if let Some(artifact) = file_artifact.clone() {
                                ArtifactView { artifact }
                            } else {
                                p { class: "kit-wc__note", "No preview for {name} yet." }
                            }
                        } else if let Some(record) = brief.clone() {
                            if tabs_now.get(active_now).map(|k| k == "brief").unwrap_or(false) {
                                section { class: "kit-wc__brief", "aria-labelledby": "{id}-brief-title",
                                    h2 { id: "{id}-brief-title", class: "kit-wc__brief-title", "{brief_title}" }
                                    if record.field_list.is_empty() {
                                        p { class: "kit-wc__note", "{line_fields}" }
                                    }
                                    for field in record.field_list.iter() {
                                        {
                                            let key = field.id.clone();
                                            let label = field.entry_label.clone();
                                            let value = field.entry_value.clone();
                                            let field_id = format!("{id}-field-{}", field.id);
                                            let change_id = field.id.clone();
                                            let kind = match field.field_type {
                                                FieldType::Number => "number",
                                                _ => "text",
                                            };
                                            rsx! {
                                                div { key: "{key}", class: "kit-wc__field",
                                                    label { r#for: "{field_id}", "{label}" }
                                                    input {
                                                        id: "{field_id}",
                                                        r#type: "{kind}",
                                                        value: "{value}",
                                                        oninput: move |event: FormEvent| {
                                                            if let Some(change) = on_field_change {
                                                                change.call((change_id.clone(), event.value()));
                                                            }
                                                        },
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    h3 { class: "kit-wc__heading", "Files" }
                                    if record.file_list.is_empty() {
                                        p { class: "kit-wc__note", "{line_files}" }
                                    }
                                    ul { class: "kit-wc__files",
                                        for file in record.file_list.iter() {
                                            {
                                                let name = file.entry_name.clone();
                                                let kind_label = file.file_kind.label();
                                                let open_name = name.clone();
                                                rsx! {
                                                    li { key: "{name}",
                                                        button {
                                                            r#type: "button",
                                                            class: "kit-wc__file",
                                                            onclick: move |_| {
                                                                open_file_tab(tabs, active, work_open, &open_name);
                                                                if let Some(open) = on_open_file {
                                                                    open.call(open_name.clone());
                                                                }
                                                            },
                                                            span { "{name}" }
                                                            span { class: "kit-wc__file-kind", "{kind_label}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    h3 { class: "kit-wc__heading", "Activity" }
                                    if record.activity_log.is_empty() {
                                        p { class: "kit-wc__note", "{line_activity}" }
                                    }
                                    ol { class: "kit-wc__log",
                                        for entry in record.activity_log.iter() {
                                            {
                                                let line = format!("{} {}", entry.at, entry.text);
                                                rsx! { li { "{line}" } }
                                            }
                                        }
                                    }
                                    div { class: "kit-wc__generate",
                                        p { class: "kit-wc__note",
                                            "Generate writes a draft from these fields."
                                            if let Some(eta) = generate_eta.clone() {
                                                " {eta}"
                                            }
                                        }
                                        Button {
                                            variant: ButtonVariant::Primary,
                                            label: "Generate",
                                            disabled: empty || loading,
                                            onclick: move |_| {
                                                if let Some(generate) = on_generate {
                                                    generate.call(());
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                button {
                    id: "{open_id}",
                    r#type: "button",
                    class: "kit-wc__open",
                    "aria-expanded": "false",
                    onclick: move |_| {
                        tabs.set(vec![brief_key()]);
                        active.set(0);
                        work_open.set(true);
                    },
                    "Brief"
                }
            }
        }
    }
}

/// Opens a file in the work area, reusing its tab when one is already open.
fn open_file_tab(
    mut tabs: Signal<Vec<String>>,
    mut active: Signal<usize>,
    mut work_open: Signal<bool>,
    name: &str,
) {
    let key = format!("file:{name}");
    let mut list = tabs.peek().clone();
    let index = match list.iter().position(|k| *k == key) {
        Some(index) => index,
        None => {
            list.push(key);
            list.len() - 1
        }
    };
    tabs.set(list);
    active.set(index);
    work_open.set(true);
}

/// Closes the tab at `index`. Closing the last tab folds the work area and returns focus to its control.
fn close_at(
    tabs: &mut Signal<Vec<String>>,
    active: &mut Signal<usize>,
    work_open: &mut Signal<bool>,
    index: usize,
    open_id: &str,
) {
    let before = tabs.peek().len();
    if index >= before {
        return;
    }
    let next = active_after_close(index, before);
    tabs.write().remove(index);
    if tabs.peek().is_empty() {
        work_open.set(false);
        focus_after_frame(open_id);
    } else {
        active.set(next.min(before - 2));
    }
}
