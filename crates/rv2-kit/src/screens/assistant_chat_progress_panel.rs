//! A plain chat in which the assistant carries out a task with tools. A side panel shows progress,
//! the outputs produced, and what the session drew on.
//!
//! Design choice: each tool call is `ToolCallCard`, which already expands to arguments and result.
//! Design choice: progress folds all but the last two finished steps behind a button that shows the
//! folded count.
//! Design choice: the panel is one state (`panel_open`). Its layout (inline on wide screens, a sheet on
//! narrow ones) is CSS only.
//! Not implemented: the sidebar, which belongs to the shared app frame.
//! Browser checks: each tool call reads running, done or failed; Escape closes the panel and leaves focus
//! on its bar button; a long output name keeps its extension.

use dioxus::prelude::*;
use rv2_affect::tool_call_card::{ToolCallCard, ToolCallStatus};

use super::ai_workspace_home::middle_truncate;
use crate::components::prompt_input_bar::{ApprovalPolicy, Model, PromptInputBar, Submission};
use crate::internal::{KIT_CSS, focus_after_frame, next_id};
use rv2_affect::skeleton::Skeleton;

const CSS: Asset = asset!("/assets/assistant-chat-progress-panel.css");

/// Steps kept visible when earlier finished steps fold.
const VISIBLE_STEPS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallState {
    Running,
    Done,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub tool_name: String,
    pub app_label: Option<String>,
    pub entry_label: String,
    pub started_at: Option<u64>,
    pub duration: Option<u64>,
    pub arguments_data: String,
    pub result_data: Option<String>,
    pub call_state: CallState,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Part {
    Text(String),
    Tool(ToolCall),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    User { id: String, text: String },
    Assistant { id: String, parts: Vec<Part> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepState {
    Done,
    Running,
    Pending,
}

impl StepState {
    pub fn word(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Running => "running",
            Self::Pending => "pending",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub id: String,
    pub entry_label: String,
    pub step_state: StepState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputKind {
    Document,
    Sheet,
    Folder,
    Markdown,
    Image,
}

impl OutputKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Document => "Document",
            Self::Sheet => "Sheet",
            Self::Folder => "Folder",
            Self::Markdown => "Markdown",
            Self::Image => "Image",
        }
    }
}

/// One produced file. `output_list` is newest first: its first entry gets the preview card.
#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    pub entry_name: String,
    pub output_kind: OutputKind,
    pub app_name: Option<String>,
    /// The opening lines the preview card shows; the card shows at most two.
    pub preview_lines: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UsageRecord {
    pub upload_names: Vec<String>,
    pub app_names: Vec<String>,
    pub web_search_pages: Option<usize>,
    pub skill_names: Vec<String>,
}

/// Map a call's state to the card's status.
pub fn card_status(state: CallState) -> ToolCallStatus {
    match state {
        CallState::Running => ToolCallStatus::Running,
        CallState::Done => ToolCallStatus::Success,
        CallState::Failed => ToolCallStatus::Error,
    }
}

/// Finished steps before the last `visible` are folded; running and pending steps never fold.
pub fn folded_count(steps: &[Step], visible: usize) -> usize {
    let finished_prefix = steps
        .iter()
        .take_while(|s| s.step_state == StepState::Done)
        .count();
    finished_prefix.min(steps.len().saturating_sub(visible))
}

/// The output count in words, singular for one.
fn outputs_words(outputs: usize) -> String {
    if outputs == 1 {
        "1 output".to_string()
    } else {
        format!("{outputs} outputs")
    }
}

/// The panel's bar label: its name, and the output count once there are outputs.
pub fn bar_label(outputs: usize) -> String {
    if outputs == 0 {
        "Progress".to_string()
    } else {
        format!("Progress, {}", outputs_words(outputs))
    }
}

/// Words for the bar button's state, read with its count.
pub fn bar_state_text(open: bool, outputs: usize) -> String {
    let state = if open { "open" } else { "closed" };
    format!("Progress and outputs, {}, {state}", outputs_words(outputs))
}

/// Whether the "Used in this session" section has anything to show.
pub fn usage_present(usage: &UsageRecord) -> bool {
    !usage.upload_names.is_empty()
        || !usage.app_names.is_empty()
        || usage.web_search_pages.is_some()
        || !usage.skill_names.is_empty()
}

/// The fold line's words: "1 earlier step", "3 earlier steps".
pub fn earlier_steps_words(folded: usize) -> String {
    if folded == 1 {
        "1 earlier step".to_string()
    } else {
        format!("{folded} earlier steps")
    }
}

/// The one-letter mark that stands for an app wherever its name would not fit.
pub fn app_mark(app: &str) -> String {
    app.chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default()
}

/// The uploads row: the first file name, then a count of the others.
pub fn uploads_words(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [only] => only.clone(),
        [first, rest @ ..] => format!("{first} and {} more", rest.len()),
    }
}

/// The web search row's page count, singular for one.
pub fn pages_words(pages: usize) -> String {
    if pages == 1 {
        "1 page".to_string()
    } else {
        format!("{pages} pages")
    }
}

/// The assistant's chat with its progress panel.
#[component]
pub fn AssistantChatProgressPanel(
    #[props(default)] active_chat: Vec<Message>,
    #[props(default)] step_list: Vec<Step>,
    #[props(default)] output_list: Vec<Output>,
    #[props(default)] usage_record: UsageRecord,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] approval_policy: ApprovalPolicy,
    on_send: EventHandler<String>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] on_open_output: Option<EventHandler<String>>,
) -> Element {
    let id = use_hook(|| format!("kit-pp-{}", next_id()));
    let bar_id = format!("{id}-bar");
    let panel_id = format!("{id}-panel");
    let title_id = format!("{id}-title");
    let composer_class = "kit-pp__composer";
    let mut panel_open = use_signal(|| false);
    let mut panel_ready = use_signal(|| false);
    let mut steps_open = use_signal(|| false);
    let mut outputs_open = use_signal(|| true);
    let mut used_open = use_signal(|| true);
    let key_bar = bar_id.clone();
    let close_bar = bar_id.clone();

    // The panel is not rendered until the viewport is read, so narrow screens never flash it open.
    use_effect(move || {
        spawn(async move {
            let narrow = document::eval("return window.matchMedia('(max-width: 959px)').matches;")
                .await
                .ok()
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            panel_open.set(!narrow);
            panel_ready.set(true);
        });
    });

    let outputs_count = output_list.len();
    let folded = folded_count(&step_list, VISIBLE_STEPS);
    let shown_steps: Vec<Step> = if steps_open() {
        step_list.clone()
    } else {
        step_list[folded..].to_vec()
    };
    let running_label = step_list
        .iter()
        .find(|s| s.step_state == StepState::Running)
        .map(|s| s.entry_label.clone())
        .unwrap_or_default();
    let usage_any = usage_present(&usage_record);
    let models = vec![Model {
        model_id: "standard".to_string(),
        model_name: "Standard".to_string(),
        model_mark: None,
        model_note: None,
    }];
    let bar_text = bar_label(outputs_count);
    let bar_state = bar_state_text(panel_open(), outputs_count);
    let chat_empty = is_empty;
    let outputs_text = outputs_words(outputs_count);
    let fold_text = earlier_steps_words(folded);
    let uploads_text = uploads_words(&usage_record.upload_names);
    let pages_text = usage_record
        .web_search_pages
        .map(pages_words)
        .unwrap_or_default();
    let skills_text = usage_record.skill_names.join(", ");

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "kit-pp", "data-panel": if panel_open() { "open" } else { "closed" },
            section { class: "kit-pp__chat", "aria-label": "Conversation",
                div { class: "kit-pp__thread", "aria-busy": is_loading.then_some("true"),
                    if is_loading {
                        Skeleton { width: "70%", height: "44px" }
                        Skeleton { width: "100%", height: "96px" }
                        Skeleton { width: "55%", height: "44px" }
                    } else if chat_empty {
                        div { class: "kit-pp__empty",
                            p { class: "kit-pp__empty-line", "This chat is empty. Ask for a task to start." }
                            button {
                                r#type: "button",
                                class: "kit-pp__empty-action",
                                onclick: move |_| {
                                    if let Some(nav) = on_navigate {
                                        nav.call("New chat".to_string());
                                    }
                                },
                                "Start a chat"
                            }
                        }
                    } else {
                        for message in active_chat.iter() {
                            {
                                match message {
                                    Message::User { id: key, text } => rsx! {
                                        div { key: "{key}", class: "kit-pp__msg kit-pp__msg--user",
                                            p { class: "kit-pp__bubble", "{text}" }
                                        }
                                    },
                                    Message::Assistant { id: key, parts } => rsx! {
                                        div { key: "{key}", class: "kit-pp__msg kit-pp__msg--assistant",
                                            for (index, part) in parts.iter().enumerate() {
                                                {
                                                    match part {
                                                        Part::Text(text) => rsx! {
                                                            p { key: "{index}", class: "kit-pp__text", "{text}" }
                                                        },
                                                        Part::Tool(call) => {
                                                            let status = card_status(call.call_state);
                                                            let name = call.tool_name.clone();
                                                            let summary = call.entry_label.clone();
                                                            let args = call.arguments_data.clone();
                                                            let result = call.result_data.clone();
                                                            let error = call.error.clone();
                                                            let started = call.started_at;
                                                            let duration = call.duration;
                                                            rsx! {
                                                                div { key: "{index}", class: "kit-pp__tool",
                                                                    ToolCallCard {
                                                                        name,
                                                                        summary,
                                                                        status,
                                                                        args,
                                                                        result,
                                                                        error,
                                                                        started_at: started,
                                                                        duration,
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
                div { class: "kit-pp__composer-wrap",
                    PromptInputBar {
                        root_class: composer_class,
                        empty_hint: "Message the assistant",
                        model_options: models,
                        approval_policy,
                        on_submit: move |submission: Submission| on_send.call(submission.text),
                    }
                }
            }
            if !(panel_ready() && panel_open()) {
                button {
                    id: "{bar_id}",
                    r#type: "button",
                    class: "kit-pp__bar",
                    "aria-expanded": "false",
                    "aria-controls": "{panel_id}",
                    "aria-label": "{bar_state}",
                    onclick: move |_| panel_open.set(true),
                    "{bar_text}"
                }
            }
            if panel_ready() && panel_open() {
                aside { id: "{panel_id}", class: "kit-pp__panel", "aria-labelledby": "{title_id}",
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape {
                            event.prevent_default();
                            panel_open.set(false);
                            focus_after_frame(&key_bar);
                        }
                    },
                    header { class: "kit-pp__head",
                        h2 { id: "{title_id}", class: "kit-pp__title", "This session" }
                        if outputs_count > 0 {
                            span { class: "kit-pp__count", "{outputs_text}" }
                        }
                        button {
                            r#type: "button",
                            class: "kit-pp__close",
                            "aria-label": "Close progress panel",
                            onclick: move |_| {
                                panel_open.set(false);
                                focus_after_frame(&close_bar);
                            },
                            "×"
                        }
                    }
                    section { class: "kit-pp__section",
                        button {
                            r#type: "button",
                            class: "kit-pp__heading",
                            "aria-expanded": if steps_open() { "true" } else { "false" },
                            "Progress"
                        }
                        if step_list.is_empty() {
                            p { class: "kit-pp__note", "No steps yet." }
                        } else {
                            if folded > 0 && !steps_open() {
                                button {
                                    r#type: "button",
                                    class: "kit-pp__fold",
                                    "aria-expanded": "false",
                                    onclick: move |_| steps_open.set(true),
                                    "{fold_text}"
                                }
                            }
                            ol { class: "kit-pp__steps",
                                for step in shown_steps.iter() {
                                    {
                                        let key = step.id.clone();
                                        let word = step.step_state.word();
                                        let label = step.entry_label.clone();
                                        rsx! {
                                            li { key: "{key}", class: "kit-pp__step", "data-state": word,
                                                span { class: "kit-pp__step-label", "{label}" }
                                                span { class: "kit-pp__step-word", "{word}" }
                                            }
                                        }
                                    }
                                }
                            }
                            p { class: "kit-sr", role: "status", "aria-live": "polite",
                                if running_label.is_empty() { "" } else { "Running: {running_label}" }
                            }
                        }
                    }
                    if outputs_count > 0 {
                        section { class: "kit-pp__section",
                            button {
                                r#type: "button",
                                class: "kit-pp__heading",
                                "aria-expanded": if outputs_open() { "true" } else { "false" },
                                onclick: move |_| outputs_open.toggle(),
                                "Outputs"
                            }
                            if outputs_open() {
                                if let Some(newest) = output_list.first().filter(|o| !o.preview_lines.is_empty()) {
                                    {
                                        let title = middle_truncate(&newest.entry_name, 22);
                                        let lines: Vec<String> = newest.preview_lines.iter().take(2).cloned().collect();
                                        rsx! {
                                            article { class: "kit-pp__preview",
                                                p { class: "kit-pp__preview-title", "{title}" }
                                                for (index, line) in lines.iter().enumerate() {
                                                    p { key: "{index}", class: "kit-pp__preview-line", "{line}" }
                                                }
                                            }
                                        }
                                    }
                                }
                                ul { class: "kit-pp__outputs",
                                    for output in output_list.iter() {
                                        {
                                            let name = output.entry_name.clone();
                                            let short = middle_truncate(&output.entry_name, 22);
                                            let kind = output.output_kind.label();
                                            let app = output.app_name.clone().unwrap_or_default();
                                            let mark = app_mark(&app);
                                            rsx! {
                                                li { key: "{name}",
                                                    button {
                                                        r#type: "button",
                                                        class: "kit-pp__output",
                                                        title: "{name}",
                                                        "aria-label": "Open {name}",
                                                        onclick: {
                                                            let open_name = name.clone();
                                                            move |_| {
                                                                if let Some(open) = on_open_output {
                                                                    open.call(open_name.clone());
                                                                }
                                                            }
                                                        },
                                                        span { class: "kit-pp__output-name", "{short}" }
                                                        span { class: "kit-pp__output-kind", "{kind}" }
                                                        if !app.is_empty() {
                                                            span { class: "kit-pp__mark", role: "img", "aria-label": "{app}", "{mark}" }
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
                    if usage_any {
                        section { class: "kit-pp__section",
                            button {
                                r#type: "button",
                                class: "kit-pp__heading",
                                "aria-expanded": if used_open() { "true" } else { "false" },
                                onclick: move |_| used_open.toggle(),
                                "Used in this session"
                            }
                            if used_open() {
                                ul { class: "kit-pp__used",
                                    if !usage_record.upload_names.is_empty() {
                                        li { class: "kit-pp__used-row",
                                            span { class: "kit-pp__used-label", "Uploads" }
                                            span { class: "kit-pp__used-value", "{uploads_text}" }
                                        }
                                    }
                                    if !usage_record.app_names.is_empty() {
                                        li { class: "kit-pp__used-row",
                                            span { class: "kit-pp__used-label", "Connectors" }
                                            span { class: "kit-pp__used-value kit-pp__marks",
                                                for app in usage_record.app_names.iter() {
                                                    {
                                                        let mark = app_mark(app);
                                                        rsx! {
                                                            span { key: "{app}", class: "kit-pp__mark", role: "img", "aria-label": "{app}", "{mark}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if usage_record.web_search_pages.is_some() {
                                        li { class: "kit-pp__used-row",
                                            span { class: "kit-pp__used-label", "Web search" }
                                            span { class: "kit-pp__used-value", "{pages_text}" }
                                        }
                                    }
                                    if !usage_record.skill_names.is_empty() {
                                        li { class: "kit-pp__used-row",
                                            span { class: "kit-pp__used-label", "Skills" }
                                            span { class: "kit-pp__used-value", "{skills_text}" }
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
