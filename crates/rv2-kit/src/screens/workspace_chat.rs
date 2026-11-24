//! A chat with an orchestrating agent that ends in a concrete task. The conversation sits beside a
//! work area with three fixed tabs: the brief (product, fields, Generate and files), the assets, and
//! the activity log.
//!
//! Design choice: the chat list and the app navigation belong to the workspace frame. This screen
//! renders the conversation and the work area only.
//! Design choice: the Generate line states what it writes. The duration is host data (`generate_eta`),
//! since the component does not know how long a draft takes.
//! Design choice: the work area's tabs are closable. Closing the last tab folds the work area, and the
//! open control brings back the three fixed tabs.
//! Design choice: a file opens in its own tab with the shared artifact viewer from the workspace frame.
//! Design choice: a multi-choice field has three looks. `MultiChoice` lists checkboxes, `Switches`
//! shows each option as a switch, and `Chips` shows each option as a toggle chip.
//! Design choice: below 960px the work area is a sheet over the conversation.
//! Not implemented: the global search (Ctrl or Cmd+K), which belongs to the app frame.
//! Browser checks: Left and Right move between tabs; closing the last tab folds the area and focuses
//! its open control; each agent's state reads in words.

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonVariant};
use rv2_affect::skeleton::Skeleton;

use super::assistant_chat_sidebars::{ApprovalMode, policy_for};
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
        /// Work-time line shown under the reply, for example "Worked for 2 min".
        work_time: Option<String>,
        /// Steps the agent took, folded under a summary until opened.
        step_list: Vec<String>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldType {
    Text,
    Choice,
    /// A choice shown as named colour swatches; each option takes a series colour by position.
    Swatches,
    MultiChoice,
    /// A multi-choice shown as toggle chips.
    Chips,
    Switches,
    Number,
    Date,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BriefField {
    pub id: String,
    pub entry_label: String,
    /// The chosen option for a choice; the chosen options joined with ", " for a multi-choice.
    pub entry_value: String,
    pub field_type: FieldType,
    /// Options for a choice or multi-choice field, in display order. Empty for text, number and date.
    pub option_list: Vec<String>,
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

/// The product the brief sells: its name, price, one line of sales figures and a top-seller mark.
#[derive(Clone, Debug, PartialEq)]
pub struct Product {
    pub entry_name: String,
    pub entry_price: String,
    pub sales_line: String,
    pub is_top_seller: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Brief {
    pub entry_title: String,
    pub product: Option<Product>,
    pub field_list: Vec<BriefField>,
    pub file_list: Vec<BriefFile>,
    /// Newest first; rendered in the order given.
    pub activity_log: Vec<ActivityEntry>,
}

/// Tab keys: the Brief, the Assets, the Activity, or an open file by name.
pub fn brief_key() -> String {
    "brief".to_string()
}

pub fn assets_key() -> String {
    "assets".to_string()
}

pub fn activity_key() -> String {
    "activity".to_string()
}

/// The fixed tabs, in the order they open.
pub fn fixed_keys() -> Vec<String> {
    vec![brief_key(), assets_key(), activity_key()]
}

/// Tab titles for keys. The brief's tab is named Brief; its heading holds the goal's title.
pub fn tab_title(key: &str) -> String {
    match key {
        "brief" => "Brief".to_string(),
        "assets" => "Assets".to_string(),
        "activity" => "Activity".to_string(),
        _ => key.strip_prefix("file:").unwrap_or(key).to_string(),
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

/// The fold summary for an agent's steps.
pub fn steps_summary(count: usize) -> String {
    match count {
        1 => "1 step".to_string(),
        n => format!("{n} steps"),
    }
}

/// Whether `option` is one of the chosen options in a multi-choice `value`.
pub fn is_selected(value: &str, option: &str) -> bool {
    value.split(',').any(|chosen| chosen.trim() == option)
}

/// The value of a multi-choice field after `option` is switched on or off. The chosen options keep
/// the order the field lists them in, so the stored value does not depend on the order of clicks.
pub fn toggle_option(value: &str, option: &str, option_list: &[String]) -> String {
    let mut chosen: Vec<&str> = value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if chosen.contains(&option) {
        chosen.retain(|part| *part != option);
    } else {
        chosen.push(option);
    }
    option_list
        .iter()
        .filter(|listed| chosen.contains(&listed.as_str()))
        .cloned()
        .collect::<Vec<_>>()
        .join(", ")
}

/// The chat with its agents and brief.
#[component]
pub fn WorkspaceChat(
    #[props(default)] message_list: Vec<AgentMessage>,
    #[props(default)] agent_status_list: Vec<AgentStatus>,
    #[props(default, into)] orchestrator_status: String,
    #[props(default)] brief_record: Option<Brief>,
    #[props(default)] artifact_list: Vec<Artifact>,
    #[props(default, into)] generate_eta: Option<String>,
    #[props(default)] approval_mode: ApprovalMode,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    on_send: EventHandler<String>,
    #[props(default)] on_field_change: Option<EventHandler<(String, String)>>,
    #[props(default)] on_generate: Option<EventHandler<()>>,
    #[props(default)] on_open_file: Option<EventHandler<String>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
) -> Element {
    let id = use_hook(|| format!("kit-wc-{}", next_id()));
    let open_id = format!("{id}-open");
    let mut work_open = use_signal(|| true);
    let mut tabs = use_signal(fixed_keys);
    let mut active = use_signal(|| 0usize);
    let brief = brief_record.clone();
    let models = vec![Model {
        model_id: "standard".to_string(),
        model_name: "Standard".to_string(),
        model_mark: None,
        model_note: None,
    }];
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
    let work_state = if work_open() { "open" } else { "closed" };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        document::Link { rel: "stylesheet", href: FRAME_CSS }
        div { class: "kit-wc", "data-work": work_state,
            section { class: "kit-wc__conversation", "aria-label": "Conversation",
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
                                    AgentMessage::Agent { id: key, author, text, work_time, step_list } => {
                                        let worked = work_time.clone();
                                        let steps = step_list.clone();
                                        let step_count = steps.len();
                                        let summary = steps_summary(step_count);
                                        rsx! {
                                            div { key: "{key}", class: "kit-wc__msg",
                                                span { class: "kit-wc__author", "{author}" }
                                                p { "{text}" }
                                                if let Some(line) = worked {
                                                    p { class: "kit-wc__worked", "{line}" }
                                                }
                                                if step_count > 0 {
                                                    details { class: "kit-wc__steps",
                                                        summary { "{summary}" }
                                                        ol { class: "kit-wc__step-list",
                                                            for step in steps.iter() {
                                                                li { key: "{step}", "{step}" }
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
                        if !agent_status_list.is_empty() {
                            ul { class: "kit-wc__agents", "aria-label": "Agents",
                                for agent in agent_status_list.iter() {
                                    {
                                        let key = agent.id.clone();
                                        let name = agent.entry_name.clone();
                                        let word = agent.entry_state.word();
                                        let title = agent.current_step.clone().unwrap_or_default();
                                        rsx! {
                                            li { key: "{key}", class: "kit-wc__agent", "data-state": "{word}", title: "{title}",
                                                span { class: "kit-wc__agent-name", "{name}" }
                                                span { class: "kit-wc__agent-state", "{word}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if !orchestrator_status.is_empty() && !loading && !empty {
                    p { class: "kit-wc__status", "{orchestrator_status}" }
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
                                let title = tab_title(key);
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
                            "aria-label": "Add a fixed tab",
                            onclick: move |_| {
                                let mut list = tabs.peek().clone();
                                for (position, key) in fixed_keys().into_iter().enumerate() {
                                    if !list.contains(&key) {
                                        list.insert(position.min(list.len()), key);
                                    }
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
                            if active_key == brief_key() {
                                section { class: "kit-wc__brief", "aria-labelledby": "{id}-brief-title",
                                    h2 { id: "{id}-brief-title", class: "kit-wc__brief-title", "{brief_title}" }
                                    if let Some(product) = record.product.clone() {
                                        article { class: "kit-wc__product",
                                            div { class: "kit-wc__product-head",
                                                h3 { class: "kit-wc__product-name", "{product.entry_name}" }
                                                span { class: "kit-wc__product-price", "{product.entry_price}" }
                                                if product.is_top_seller {
                                                    span { class: "kit-wc__badge", "Top seller" }
                                                }
                                            }
                                            p { class: "kit-wc__product-sales", "{product.sales_line}" }
                                        }
                                    }
                                    if record.field_list.is_empty() {
                                        p { class: "kit-wc__note", "{line_fields}" }
                                    }
                                    div { class: "kit-wc__fields",
                                        for field in record.field_list.iter() {
                                            {brief_field(&id, field, on_field_change)}
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
                                    h3 { class: "kit-wc__heading", "Files" }
                                    if record.file_list.is_empty() {
                                        p { class: "kit-wc__note", "{line_files}" }
                                    }
                                    {file_rows(record.file_list.clone(), tabs, active, work_open, on_open_file)}
                                }
                            } else if active_key == assets_key() {
                                section { class: "kit-wc__assets", "aria-label": "Assets",
                                    h3 { class: "kit-wc__heading", "Files" }
                                    if record.file_list.is_empty() {
                                        p { class: "kit-wc__note", "{line_files}" }
                                    }
                                    {file_rows(record.file_list.clone(), tabs, active, work_open, on_open_file)}
                                }
                            } else {
                                section { class: "kit-wc__activity", "aria-label": "Activity",
                                    h3 { class: "kit-wc__heading", "Activity" }
                                    if record.activity_log.is_empty() {
                                        p { class: "kit-wc__note", "{line_activity}" }
                                    }
                                    ol { class: "kit-wc__log",
                                        for entry in record.activity_log.iter() {
                                            {
                                                let at = entry.at.clone();
                                                let text = entry.text.clone();
                                                rsx! {
                                                    li { span { class: "kit-wc__at", "{at}" } span { "{text}" } }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            p { class: "kit-wc__note", "The brief appears once the agents start this goal." }
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
                        tabs.set(fixed_keys());
                        active.set(0);
                        work_open.set(true);
                    },
                    "Brief"
                }
            }
        }
    }
}

/// The files list: each row opens its file in a tab of the work area.
fn file_rows(
    files: Vec<BriefFile>,
    tabs: Signal<Vec<String>>,
    active: Signal<usize>,
    work_open: Signal<bool>,
    on_open_file: Option<EventHandler<String>>,
) -> Element {
    rsx! {
        ul { class: "kit-wc__files",
            for file in files.iter() {
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
    }
}

/// One brief field: a text, number or date input, a choice as radio buttons or swatches, or a
/// multi-choice as checkboxes, switches or chips. Every change reports the field's id and its new value.
fn brief_field(
    prefix: &str,
    field: &BriefField,
    on_change: Option<EventHandler<(String, String)>>,
) -> Element {
    let key = field.id.clone();
    let label = field.entry_label.clone();
    let value = field.entry_value.clone();
    let field_id = format!("{prefix}-field-{}", field.id);
    let options = field.option_list.clone();
    match field.field_type {
        FieldType::Choice | FieldType::Swatches => {
            let swatches = field.field_type == FieldType::Swatches;
            let class = if swatches {
                "kit-wc__field kit-wc__choice kit-wc__swatches"
            } else {
                "kit-wc__field kit-wc__choice"
            };
            rsx! {
                fieldset { key: "{key}", class: "{class}",
                    legend { "{label}" }
                    for (index, option) in options.iter().enumerate() {
                        {
                            let option_id = format!("{field_id}-{index}");
                            let group = field_id.clone();
                            let picked = option.clone();
                            let change_id = key.clone();
                            let checked = value == *option;
                            let series = (index % 6 + 1).to_string();
                            rsx! {
                                label { key: "{option}", class: "kit-wc__option", r#for: "{option_id}",
                                    input {
                                        id: "{option_id}",
                                        r#type: "radio",
                                        name: "{group}",
                                        value: "{picked}",
                                        checked,
                                        onchange: move |_| {
                                            if let Some(change) = on_change {
                                                change.call((change_id.clone(), picked.clone()));
                                            }
                                        },
                                    }
                                    if swatches {
                                        span { class: "kit-wc__swatch", "data-series": "{series}", "aria-hidden": "true" }
                                    }
                                    span { "{option}" }
                                }
                            }
                        }
                    }
                }
            }
        }
        FieldType::MultiChoice | FieldType::Switches | FieldType::Chips => {
            let switches = field.field_type == FieldType::Switches;
            let chips = field.field_type == FieldType::Chips;
            let class = if switches {
                "kit-wc__field kit-wc__multi kit-wc__switches"
            } else if chips {
                "kit-wc__field kit-wc__multi kit-wc__chips"
            } else {
                "kit-wc__field kit-wc__multi"
            };
            let option_class = if switches {
                "kit-wc__option kit-wc__switch"
            } else if chips {
                "kit-wc__option kit-wc__chip"
            } else {
                "kit-wc__option"
            };
            rsx! {
                fieldset { key: "{key}", class: "{class}",
                    legend { "{label}" }
                    for (index, option) in options.iter().enumerate() {
                        {
                            let option_id = format!("{field_id}-{index}");
                            let picked = option.clone();
                            let next_value = toggle_option(&value, option, &options);
                            let change_id = key.clone();
                            let checked = is_selected(&value, option);
                            rsx! {
                                label { key: "{option}", class: option_class, r#for: "{option_id}",
                                    input {
                                        id: "{option_id}",
                                        r#type: "checkbox",
                                        role: if switches { "switch" } else { "checkbox" },
                                        value: "{picked}",
                                        checked,
                                        onchange: move |_| {
                                            if let Some(change) = on_change {
                                                change.call((change_id.clone(), next_value.clone()));
                                            }
                                        },
                                    }
                                    span { "{option}" }
                                }
                            }
                        }
                    }
                }
            }
        }
        FieldType::Text | FieldType::Number | FieldType::Date => {
            let kind = match field.field_type {
                FieldType::Number => "number",
                FieldType::Date => "date",
                _ => "text",
            };
            let change_id = key.clone();
            rsx! {
                div { key: "{key}", class: "kit-wc__field",
                    label { r#for: "{field_id}", "{label}" }
                    input {
                        id: "{field_id}",
                        r#type: "{kind}",
                        value: "{value}",
                        oninput: move |event: FormEvent| {
                            if let Some(change) = on_change {
                                change.call((change_id.clone(), event.value()));
                            }
                        },
                    }
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
