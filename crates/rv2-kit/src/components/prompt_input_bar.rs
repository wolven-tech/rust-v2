//! Composer at the foot of an agent chat: a message field, model and approval policy, attachments, folder and services.
//!
//! Design choice: the dictation control only toggles a recording state and appends `voice_sample_text`
//! when stopped. Real speech recognition is a host concern.
//! Design choice: `approval_policy`, `selected_folder` and `linked_service_ids` are starting values; the
//! component keeps its own selection and reports each change through its `on_*` callback.
//! Design choice: one menu is open at a time. Opening another closes the first.
//! Design choice: the plan-mode item is reported as an attach action; its checked state is not kept here.
//! Browser checks: menus open toward the side with more room; the policy label collapses to its icon on
//! narrow widths; the folder name truncates with its full name in the tooltip.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/prompt-input-bar.css");
/// Services shown as marks in the context row.
pub const VISIBLE_SERVICE_MARKS: usize = 3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ApprovalPolicy {
    #[default]
    ConfirmEach,
    AutoSafe,
    Unrestricted,
}

impl ApprovalPolicy {
    pub fn label(self) -> &'static str {
        match self {
            Self::ConfirmEach => "Confirm each",
            Self::AutoSafe => "Auto-approve safe",
            Self::Unrestricted => "Unrestricted",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ConfirmEach => "confirm_each",
            Self::AutoSafe => "auto_safe",
            Self::Unrestricted => "unrestricted",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachAction {
    AttachFile,
    AttachConnector,
    PickFolder,
    SetGoal,
    TogglePlanMode,
    AttachSkill,
    AttachDrawing,
    AttachDocument,
    AttachPdf,
}

impl AttachAction {
    pub const ALL: [AttachAction; 9] = [
        Self::AttachFile,
        Self::AttachConnector,
        Self::PickFolder,
        Self::SetGoal,
        Self::TogglePlanMode,
        Self::AttachSkill,
        Self::AttachDrawing,
        Self::AttachDocument,
        Self::AttachPdf,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::AttachFile => "attach_file",
            Self::AttachConnector => "attach_connector",
            Self::PickFolder => "pick_folder",
            Self::SetGoal => "set_goal",
            Self::TogglePlanMode => "toggle_plan_mode",
            Self::AttachSkill => "attach_skill",
            Self::AttachDrawing => "attach_drawing",
            Self::AttachDocument => "attach_document",
            Self::AttachPdf => "attach_pdf",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::AttachFile => "Attach file",
            Self::AttachConnector => "Attach connector",
            Self::PickFolder => "Pick folder",
            Self::SetGoal => "Set goal",
            Self::TogglePlanMode => "Plan mode",
            Self::AttachSkill => "Attach skill",
            Self::AttachDrawing => "Attach drawing",
            Self::AttachDocument => "Attach document",
            Self::AttachPdf => "Attach PDF",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Model {
    pub model_id: String,
    pub model_name: String,
    pub model_mark: Option<String>,
    pub model_note: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Service {
    pub service_id: String,
    pub service_name: String,
    pub service_mark: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Submission {
    pub text: String,
    pub model_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Menu {
    Plus,
    Policy,
    Model,
    Folder,
    Services,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Row {
    Attach(AttachAction),
    Policy(ApprovalPolicy),
    Model(usize),
    Folder(String),
    Service(String),
}

/// Text a send would carry: trimmed, and absent when only whitespace was typed.
pub fn sendable(draft: &str) -> Option<String> {
    let trimmed = draft.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// Linked ids after toggling `service_id`: appended when new, removed when present.
pub fn toggle_link(linked: &[String], service_id: &str) -> Vec<String> {
    let mut next = linked.to_vec();
    if let Some(at) = next.iter().position(|id| id == service_id) {
        next.remove(at);
    } else {
        next.push(service_id.to_string());
    }
    next
}

/// A folder pick: picking the selected folder again clears the selection.
pub fn folder_after_pick(current: Option<&str>, picked: &str) -> Option<String> {
    if current == Some(picked) {
        None
    } else {
        Some(picked.to_string())
    }
}

fn focus_element(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

fn focus_query(query: &str) {
    document::eval(&format!("document.querySelector({query:?})?.focus();"));
}

/// The chat composer.
#[component]
pub fn PromptInputBar(
    #[props(default = "Message the agent".to_string(), into)] empty_hint: String,
    model_options: Vec<Model>,
    #[props(default, into)] voice_sample_text: String,
    #[props(default)] on_submit: Option<EventHandler<Submission>>,
    #[props(default)] on_model_pick: Option<EventHandler<Model>>,
    #[props(default)] on_attach_pick: Option<EventHandler<AttachAction>>,
    #[props(default)] approval_policy: ApprovalPolicy,
    #[props(default)] on_policy_pick: Option<EventHandler<ApprovalPolicy>>,
    #[props(default)] on_help_open: Option<EventHandler<()>>,
    #[props(default)] folder_options: Vec<String>,
    #[props(default)] selected_folder: Option<String>,
    #[props(default)] on_folder_pick: Option<EventHandler<Option<String>>>,
    #[props(default)] service_options: Vec<Service>,
    #[props(default)] linked_service_ids: Vec<String>,
    #[props(default)] on_links_change: Option<EventHandler<Vec<String>>>,
    #[props(default)] custom_footer: Option<Element>,
    #[props(default = true)] show_context_row: bool,
    #[props(default, into)] root_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-pib-{}", next_id()));
    let input_id = format!("{id}-input");
    let mut draft = use_signal(String::new);
    let mut recording = use_signal(|| false);
    let mut model_index = use_signal(|| 0_usize);
    let mut policy = use_signal(|| approval_policy);
    let mut folder = use_signal(|| selected_folder.clone());
    let mut linked = use_signal(|| linked_service_ids.clone());
    let mut open_menu = use_signal(|| Option::<Menu>::None);

    let model = model_options.get(model_index()).cloned();
    let model_label = model
        .as_ref()
        .map(|m| m.model_name.clone())
        .unwrap_or_default();
    let model_id = model
        .as_ref()
        .map(|m| m.model_id.clone())
        .unwrap_or_default();
    let current_policy = policy();
    let policy_label = current_policy.label();
    let policy_key = current_policy.as_str();
    let unrestricted = current_policy == ApprovalPolicy::Unrestricted;
    let current_folder = folder();
    let folder_label = current_folder
        .clone()
        .unwrap_or_else(|| "No folder".to_string());
    let linked_now = linked();
    let linked_marks: Vec<String> = service_options
        .iter()
        .filter(|service| linked_now.contains(&service.service_id))
        .take(VISIBLE_SERVICE_MARKS)
        .map(|service| {
            service
                .service_mark
                .clone()
                .unwrap_or_else(|| service.service_name.chars().take(1).collect())
        })
        .collect();
    let linked_count = linked_now.len();
    let can_send = sendable(&draft()).is_some();
    let root_class = if root_class.is_empty() {
        "kit-pib".to_string()
    } else {
        format!("kit-pib {root_class}")
    };
    let context_row = show_context_row;
    let menu = open_menu();
    let recording_now = recording();
    let placeholder = if recording_now {
        "Listening…".to_string()
    } else {
        empty_hint.clone()
    };
    let heard = voice_sample_text.clone();
    let mic_label = if recording_now {
        "Stop dictation"
    } else {
        "Start dictation"
    };
    let context_visible = context_row && custom_footer.is_none();
    let has_custom_footer = custom_footer.is_some();
    let has_help = on_help_open.is_some();

    let send = {
        let handler = on_submit.clone();
        let model_id = model_id.clone();
        move |_: ()| {
            if let Some(text) = sendable(&draft()) {
                draft.set(String::new());
                if let Some(h) = &handler {
                    h.call(Submission {
                        text,
                        model_id: model_id.clone(),
                    });
                }
            }
        }
    };
    let mut send_key = send.clone();
    let mut send_click = send.clone();

    let run_row = {
        let attach = on_attach_pick.clone();
        let policy_handler = on_policy_pick.clone();
        let model_handler = on_model_pick.clone();
        let folder_handler = on_folder_pick.clone();
        let links_handler = on_links_change.clone();
        let models = model_options.clone();
        move |row: Row| {
            open_menu.set(None);
            match row {
                Row::Attach(action) => {
                    if let Some(h) = &attach {
                        h.call(action);
                    }
                }
                Row::Policy(next) => {
                    policy.set(next);
                    if let Some(h) = &policy_handler {
                        h.call(next);
                    }
                }
                Row::Model(index) => {
                    model_index.set(index);
                    if let (Some(h), Some(chosen)) = (&model_handler, models.get(index)) {
                        h.call(chosen.clone());
                    }
                }
                Row::Folder(name) => {
                    let next = folder_after_pick(folder().as_deref(), &name);
                    folder.set(next.clone());
                    if let Some(h) = &folder_handler {
                        h.call(next);
                    }
                }
                Row::Service(service_id) => {
                    let next = toggle_link(&linked(), &service_id);
                    linked.set(next.clone());
                    if let Some(h) = &links_handler {
                        h.call(next);
                    }
                }
            }
        }
    };

    let mut rows: Vec<(String, String, &'static str, bool, Row)> = Vec::new();
    match menu {
        Some(Menu::Plus) => {
            for action in AttachAction::ALL {
                let role = if action == AttachAction::TogglePlanMode {
                    "menuitemcheckbox"
                } else {
                    "menuitem"
                };
                rows.push((
                    action.as_str().to_string(),
                    action.label().to_string(),
                    role,
                    false,
                    Row::Attach(action),
                ));
            }
            if !context_row {
                for service in &service_options {
                    let checked = linked_now.contains(&service.service_id);
                    rows.push((
                        format!("svc-{}", service.service_id),
                        service.service_name.clone(),
                        "menuitemcheckbox",
                        checked,
                        Row::Service(service.service_id.clone()),
                    ));
                }
            }
        }
        Some(Menu::Policy) => {
            for choice in [
                ApprovalPolicy::ConfirmEach,
                ApprovalPolicy::AutoSafe,
                ApprovalPolicy::Unrestricted,
            ] {
                rows.push((
                    choice.as_str().to_string(),
                    choice.label().to_string(),
                    "option",
                    choice == current_policy,
                    Row::Policy(choice),
                ));
            }
        }
        Some(Menu::Model) => {
            for (index, option) in model_options.iter().enumerate() {
                rows.push((
                    option.model_id.clone(),
                    option.model_name.clone(),
                    "option",
                    index == model_index(),
                    Row::Model(index),
                ));
            }
        }
        Some(Menu::Folder) => {
            for name in &folder_options {
                let selected = current_folder.as_deref() == Some(name.as_str());
                rows.push((
                    name.clone(),
                    name.clone(),
                    "option",
                    selected,
                    Row::Folder(name.clone()),
                ));
            }
        }
        Some(Menu::Services) => {
            for service in &service_options {
                let checked = linked_now.contains(&service.service_id);
                rows.push((
                    service.service_id.clone(),
                    service.service_name.clone(),
                    "menuitemcheckbox",
                    checked,
                    Row::Service(service.service_id.clone()),
                ));
            }
        }
        None => {}
    }
    let menu_name = match menu {
        Some(Menu::Plus) => "plus",
        Some(Menu::Policy) => "policy",
        Some(Menu::Model) => "model",
        Some(Menu::Folder) => "folder",
        Some(Menu::Services) => "services",
        None => "",
    };
    let menu_open = menu.is_some();
    let trigger_selector = match menu {
        Some(Menu::Plus) => ".kit-pib__plus",
        Some(Menu::Policy) => ".kit-pib__policy",
        Some(Menu::Model) => ".kit-pib__model",
        Some(Menu::Folder) => ".kit-pib__folder",
        Some(Menu::Services) => ".kit-pib__services",
        None => "",
    };
    let trigger_query = format!("#{id} {trigger_selector}");
    let menu_id = format!("{id}-menu");
    let menu_role = if menu == Some(Menu::Plus) || menu == Some(Menu::Services) {
        "menu"
    } else {
        "listbox"
    };
    let help_handler = on_help_open.clone();
    let help_visible = menu == Some(Menu::Policy) && has_help;
    let custom_node = custom_footer;
    let folder_toggle_target = Menu::Folder;

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { id: "{id}", class: "{root_class}", "data-context": if context_row { "shown" } else { "hidden" },
            div { class: "kit-pib__field", onclick: move |_| focus_element(&input_id),
                textarea {
                    id: "{id}-input",
                    class: "kit-pib__input",
                    rows: "1",
                    placeholder: "{placeholder}",
                    value: "{draft}",
                    aria_label: "Message",
                    oninput: move |event: FormEvent| draft.set(event.value()),
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Enter && !event.modifiers().shift() {
                            event.prevent_default();
                            send_key(());
                        } else if event.key() == Key::Escape && recording() {
                            recording.set(false);
                        }
                    },
                }
                div { class: "kit-pib__dictation", role: "status", "aria-live": "polite",
                    if recording_now {
                        span { class: "kit-pib__heard", "{heard}" }
                    }
                }
                div { class: "kit-pib__controls",
                    button {
                        r#type: "button",
                        class: "kit-pib__plus",
                        aria_label: "Add",
                        "aria-haspopup": "menu",
                        "aria-expanded": if menu == Some(Menu::Plus) { "true" } else { "false" },
                        onclick: move |_| open_menu.set(if menu == Some(Menu::Plus) { None } else { Some(Menu::Plus) }),
                        "+"
                    }
                    button {
                        r#type: "button",
                        class: "kit-pib__policy",
                        "data-policy": policy_key,
                        "data-tone": if unrestricted { "danger" } else { "normal" },
                        "aria-haspopup": "listbox",
                        "aria-expanded": if menu == Some(Menu::Policy) { "true" } else { "false" },
                        onclick: move |_| open_menu.set(if menu == Some(Menu::Policy) { None } else { Some(Menu::Policy) }),
                        span { class: "kit-pib__policy-label", "{policy_label}" }
                    }
                    button {
                        r#type: "button",
                        class: "kit-pib__model",
                        "aria-haspopup": "listbox",
                        "aria-expanded": if menu == Some(Menu::Model) { "true" } else { "false" },
                        onclick: move |_| open_menu.set(if menu == Some(Menu::Model) { None } else { Some(Menu::Model) }),
                        "{model_label}"
                    }
                    button {
                        r#type: "button",
                        class: "kit-pib__mic",
                        "aria-pressed": if recording_now { "true" } else { "false" },
                        aria_label: "{mic_label}",
                        onclick: move |_| {
                            if recording() {
                                recording.set(false);
                                draft.with_mut(|text| text.push_str(&heard));
                            } else {
                                recording.set(true);
                            }
                        },
                        "Mic"
                    }
                    button {
                        r#type: "button",
                        class: "kit-pib__send",
                        disabled: !can_send,
                        aria_label: "Send",
                        onclick: move |_| send_click(()),
                        "Send"
                    }
                }
            }
            if menu_open {
                div { id: "{menu_id}", class: "kit-pib__menu", role: "{menu_role}", "data-menu": "{menu_name}",
                    onmounted: move |_: MountedEvent| focus_query(&format!("#{menu_id} .kit-pib__item")),
                    onkeydown: {
                        let trigger_query = trigger_query.clone();
                        move |event: KeyboardEvent| {
                            if event.key() == Key::Escape {
                                event.prevent_default();
                                open_menu.set(None);
                                focus_query(&trigger_query);
                            }
                        }
                    },
                    for (key, label, role, selected, row) in rows {
                        button {
                            key: "{key}",
                            r#type: "button",
                            role: "{role}",
                            class: "kit-pib__item",
                            "aria-selected": if role == "option" && selected { "true" } else { "false" },
                            "aria-checked": if role == "menuitemcheckbox" && selected { "true" } else { "false" },
                            onclick: {
                                let mut run = run_row.clone();
                                let trigger_query = trigger_query.clone();
                                move |_| {
                                    run(row.clone());
                                    focus_query(&trigger_query);
                                }
                            },
                            "{label}"
                        }
                    }
                    if help_visible {
                        if let Some(help) = help_handler {
                            button {
                                r#type: "button",
                                class: "kit-pib__item kit-pib__help",
                                onclick: move |_| help.call(()),
                                "What do these mean?"
                            }
                        }
                    }
                }
            }
            if context_visible {
                footer { class: "kit-pib__context",
                    button {
                        r#type: "button",
                        class: "kit-pib__folder",
                        title: "{folder_label}",
                        "aria-haspopup": "listbox",
                        onclick: move |_| open_menu.set(if menu == Some(folder_toggle_target) { None } else { Some(folder_toggle_target) }),
                        span { class: "kit-pib__folder-name", "{folder_label}" }
                    }
                    button {
                        r#type: "button",
                        class: "kit-pib__services",
                        "aria-haspopup": "menu",
                        onclick: move |_| open_menu.set(if menu == Some(Menu::Services) { None } else { Some(Menu::Services) }),
                        for mark in linked_marks {
                            span { class: "kit-pib__service-mark", aria_hidden: "true", "{mark}" }
                        }
                        span { class: "kit-pib__service-count", "{linked_count} linked" }
                    }
                }
            } else if has_custom_footer {
                footer { class: "kit-pib__context",
                    if let Some(node) = custom_node {
                        {node}
                    }
                }
            }
        }
    }
}
