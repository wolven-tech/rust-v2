//! The assistant's first screen: a greeting and a prompt box that leads straight into work, with tool
//! shortcuts, connected apps, suggestions, and a work area that opens artifacts in tabs.
//!
//! Design choice: the composer is a plain textarea, not `PromptInputBar`. A tool chip fills the draft
//! with its template, and `PromptInputBar` keeps its draft internal, so it cannot be filled.
//! Design choice: `SendRequest.attachments` is always empty here; no attach contract exists in the
//! composer's data, so the attach control is disabled until `on_attach` is given.
//! Design choice: suggestions live in the work area's New tab, not the centre column, so the start
//! column stays a greeting, a composer and its chips.
//! Not implemented: the sidebar. It belongs to the shared app frame, which this screen does not render.
//! Browser checks: Enter sends and Shift+Enter adds a line; a tool chip fills and does not send;
//! closing the last tab folds the work area and focuses its open control.

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonVariant};
use rv2_affect::skeleton::Skeleton;

use crate::internal::{KIT_CSS, focus_after_frame, next_id};

const CSS: Asset = asset!("/assets/assistant-start-page.css");

#[derive(Clone, Debug, PartialEq)]
pub struct ToolChip {
    pub id: String,
    pub entry_label: String,
    pub prompt_template: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConnectedApp {
    pub id: String,
    pub entry_name: String,
    pub app_mark: Option<String>,
}

/// A suggested start. With `artifact` set, choosing it opens that artifact instead of sending.
#[derive(Clone, Debug, PartialEq)]
pub struct Suggestion {
    pub id: String,
    pub entry_label: String,
    pub prompt_text: String,
    pub artifact: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputKind {
    Document,
    Sheet,
    Folder,
}

impl OutputKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Document => "Document",
            Self::Sheet => "Sheet",
            Self::Folder => "Folder",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Artifact {
    pub entry_name: String,
    pub output_kind: OutputKind,
    pub preview_text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SendRequest {
    pub text: String,
    pub attachments: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TabKind {
    New,
    Output(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkTab {
    pub id: usize,
    pub title: String,
    pub kind: TabKind,
}

/// The trimmed draft, or nothing when only whitespace was typed.
pub fn send_text(draft: &str) -> Option<String> {
    let trimmed = draft.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// Sending waits while the page loads: the text stays in the field.
pub fn can_send(draft: &str, loading: bool) -> bool {
    !loading && send_text(draft).is_some()
}

/// Arrow keys move between tabs and wrap at either end.
pub fn tab_after_move(current: usize, count: usize, forward: bool) -> usize {
    if count == 0 {
        0
    } else if forward {
        (current + 1) % count
    } else {
        (current + count - 1) % count
    }
}

/// Index of the tab that takes focus after the tab at `closed` is removed from `count_before` tabs.
pub fn active_after_close(closed: usize, count_before: usize) -> usize {
    if closed + 1 >= count_before {
        count_before.saturating_sub(2)
    } else {
        closed
    }
}

/// The start page: greeting, composer, tool shortcuts, apps, suggestions, and the work area.
#[component]
pub fn AssistantStartPage(
    #[props(into)] greeting_text: String,
    #[props(default, into)] invitation_text: Option<String>,
    #[props(default, into)] working_folder: Option<String>,
    #[props(default, into)] approval_mode: Option<String>,
    #[props(default)] open_work_area: bool,
    #[props(default)] tool_chip_list: Vec<ToolChip>,
    #[props(default)] connected_app_list: Vec<ConnectedApp>,
    #[props(default)] suggestion_list: Vec<Suggestion>,
    #[props(default)] suggested_artifacts: Vec<Artifact>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    on_send: EventHandler<SendRequest>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] on_attach: Option<EventHandler<()>>,
) -> Element {
    let id = use_hook(|| format!("kit-start-{}", next_id()));
    let composer_id = format!("{id}-composer");
    let open_id = format!("{id}-open");
    let area_id = format!("{id}-area");
    let mut draft = use_signal(String::new);
    let mut area_open = use_signal(|| open_work_area);
    let mut tab_seq = use_signal(|| 1usize);
    let mut tabs = use_signal(move || {
        if open_work_area {
            vec![WorkTab {
                id: 0,
                title: "New tab".to_string(),
                kind: TabKind::New,
            }]
        } else {
            Vec::new()
        }
    });
    let mut active = use_signal(|| 0usize);

    let loading = is_loading;
    let sendable = can_send(&draft(), loading);
    let apps_empty = is_empty || connected_app_list.is_empty();
    let artifacts_for_tabs = suggested_artifacts.clone();
    let open_output = Callback::new(move |name: String| {
        let next_id = *tab_seq.peek();
        tab_seq.set(next_id + 1);
        tabs.write().push(WorkTab {
            id: next_id,
            title: name.clone(),
            kind: TabKind::Output(name),
        });
        active.set(next_id);
        area_open.set(true);
    });
    let key_id = id.clone();
    let key_open = open_id.clone();

    let tabs_now = tabs();
    let active_now = active();
    let current_kind = tabs_now
        .iter()
        .find(|t| t.id == active_now)
        .map(|t| t.kind.clone());
    let output_view = match &current_kind {
        Some(TabKind::Output(name)) => artifacts_for_tabs
            .iter()
            .find(|a| &a.entry_name == name)
            .cloned(),
        _ => None,
    };
    let tab_count = tabs_now.len();
    let output_kind_label = output_view
        .as_ref()
        .map(|a| a.output_kind.label())
        .unwrap_or_default();
    let output_body = output_view
        .as_ref()
        .map(|a| a.preview_text.clone())
        .unwrap_or_default();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "kit-start", "data-area": if area_open() { "open" } else { "closed" },
            div { class: "kit-start__main",
                header { class: "kit-start__top",
                    button {
                        id: "{open_id}",
                        r#type: "button",
                        class: "kit-start__open",
                        "aria-expanded": if area_open() { "true" } else { "false" },
                        "aria-controls": "{area_id}",
                        onclick: move |_| {
                            let next = !area_open();
                            area_open.set(next);
                            if next && tabs.peek().is_empty() {
                                tabs.write().push(WorkTab { id: 0, title: "New tab".to_string(), kind: TabKind::New });
                                active.set(0);
                            }
                        },
                        "Work area"
                    }
                }
                div { class: "kit-start__centre",
                    if loading {
                        Skeleton { width: "60%", height: "32px" }
                    } else {
                        h1 { class: "kit-start__greeting", "{greeting_text}" }
                        if let Some(line) = invitation_text.clone() {
                            p { class: "kit-start__invitation", "{line}" }
                        }
                    }
                    form { class: "kit-start__composer", "aria-busy": loading.then_some("true"),
                        onsubmit: move |event: FormEvent| event.prevent_default(),
                        label { class: "kit-sr", r#for: "{composer_id}", "Message the assistant" }
                        textarea {
                            id: "{composer_id}",
                            class: "kit-start__field",
                            rows: "2",
                            autofocus: true,
                            placeholder: "Ask for a document, a table, research or a chart",
                            value: "{draft}",
                            oninput: move |event: FormEvent| draft.set(event.value()),
                            onkeydown: move |event: KeyboardEvent| {
                                if event.key() == Key::Enter && !event.modifiers().shift() {
                                    event.prevent_default();
                                    if can_send(&draft.peek(), loading) {
                                        let text = send_text(&draft.peek()).unwrap_or_default();
                                        draft.set(String::new());
                                        on_send.call(SendRequest { text, attachments: Vec::new() });
                                    }
                                }
                            },
                        }
                        div { class: "kit-start__footer",
                            button {
                                r#type: "button",
                                class: "kit-start__attach",
                                "aria-label": "Attach a file",
                                disabled: on_attach.is_none(),
                                onclick: move |_| {
                                    if let Some(attach) = on_attach {
                                        attach.call(());
                                    }
                                },
                                "+"
                            }
                            if let Some(folder) = working_folder.clone() {
                                span { class: "kit-start__control", "{folder}" }
                            }
                            if let Some(mode) = approval_mode.clone() {
                                span { class: "kit-start__control", "{mode}" }
                            }
                            div { class: "kit-start__apps", role: "group", "aria-label": "Connected apps",
                                if loading {
                                    Skeleton { width: "120px", height: "28px" }
                                } else {
                                    Button {
                                        variant: ButtonVariant::Secondary,
                                        label: "Connect apps",
                                        onclick: move |_| {
                                            if let Some(nav) = on_navigate {
                                                nav.call("Connect apps".to_string());
                                            }
                                        },
                                    }
                                    if !apps_empty {
                                        ul { class: "kit-start__app-list",
                                            for app in connected_app_list.iter() {
                                                {
                                                    let mark = app.app_mark.clone().unwrap_or_else(|| app.entry_name.chars().take(1).collect());
                                                    let name = app.entry_name.clone();
                                                    rsx! {
                                                        li { key: "{app.id}", class: "kit-start__app",
                                                            span { class: "kit-start__app-mark", role: "img", title: "{name}", "aria-label": "{name}", "{mark}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            button {
                                r#type: "button",
                                class: "kit-start__send",
                                disabled: !sendable,
                                onclick: move |_| {
                                    if can_send(&draft.peek(), loading) {
                                        let text = send_text(&draft.peek()).unwrap_or_default();
                                        draft.set(String::new());
                                        on_send.call(SendRequest { text, attachments: Vec::new() });
                                    }
                                },
                                "Send"
                            }
                        }
                    }
                    div { class: "kit-start__tools", role: "group", "aria-label": "Tools",
                        if loading {
                            Skeleton { width: "96px", height: "28px" }
                            Skeleton { width: "88px", height: "28px" }
                        } else {
                            for chip in tool_chip_list.iter() {
                                {
                                    let template = chip.prompt_template.clone();
                                    let label = chip.entry_label.clone();
                                    let key = chip.id.clone();
                                    let target = composer_id.clone();
                                    rsx! {
                                        button {
                                            key: "{key}",
                                            r#type: "button",
                                            class: "kit-start__tool",
                                            onclick: move |_| {
                                                draft.set(template.clone());
                                                document::eval(&format!(
                                                    "const el = document.getElementById({target:?}); if (el) {{ el.focus(); el.setSelectionRange({len}, {len}); }}",
                                                    len = template.chars().count()
                                                ));
                                            },
                                            span { class: "kit-start__tool-icon", "aria-hidden": "true" }
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if area_open() {
                aside { id: "{area_id}", class: "kit-start__area", "aria-label": "Work area",
                    div { class: "kit-start__tabs", role: "tablist", "aria-label": "Open items",
                        onkeydown: move |event: KeyboardEvent| {
                            let count = tabs.peek().len();
                            let index = tabs.peek().iter().position(|t| t.id == *active.peek()).unwrap_or(0);
                            match event.key() {
                                Key::ArrowRight | Key::ArrowLeft => {
                                    event.prevent_default();
                                    let next = tab_after_move(index, count, event.key() == Key::ArrowRight);
                                    let target = tabs.peek()[next].id;
                                    active.set(target);
                                    focus_after_frame(&format!("{key_id}-tab-{target}"));
                                }
                                Key::Delete => {
                                    event.prevent_default();
                                    let closing = *active.peek();
                                    close_tab(&mut tabs, &mut active, &mut area_open, closing, &key_open);
                                }
                                _ => {}
                            }
                        },
                        for tab in tabs_now.iter() {
                            {
                                let tab_id = tab.id;
                                let title = tab.title.clone();
                                let selected = tab_id == active_now;
                                let dom_id = format!("{id}-tab-{tab_id}");
                                let close_label = format!("Close {title}");
                                let panel = format!("{id}-panel");
                                let close_open = open_id.clone();
                                rsx! {
                                    div { key: "{tab_id}", class: "kit-start__tab", "data-selected": selected.then_some("true"),
                                        button {
                                            id: "{dom_id}",
                                            r#type: "button",
                                            role: "tab",
                                            "aria-selected": if selected { "true" } else { "false" },
                                            "aria-controls": "{panel}",
                                            tabindex: if selected { "0" } else { "-1" },
                                            onclick: move |_| active.set(tab_id),
                                            "{title}"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "kit-start__tab-close",
                                            "aria-label": "{close_label}",
                                            onclick: move |_| close_tab(&mut tabs, &mut active, &mut area_open, tab_id, &close_open),
                                            "×"
                                        }
                                    }
                                }
                            }
                        }
                        button {
                            r#type: "button",
                            class: "kit-start__tab-new",
                            "aria-label": "New tab",
                            onclick: move |_| {
                                let next_id = *tab_seq.peek();
                                tab_seq.set(next_id + 1);
                                tabs.write().push(WorkTab { id: next_id, title: "New tab".to_string(), kind: TabKind::New });
                                active.set(next_id);
                            },
                            "+"
                        }
                    }
                    div { id: "{id}-panel", role: "tabpanel", class: "kit-start__panel", "data-tabs": "{tab_count}",
                        {
                            match &current_kind {
                                Some(TabKind::Output(name)) => rsx! {
                                    article { class: "kit-start__output",
                                        h2 { class: "kit-start__output-name", title: "{name}", "{name}" }
                                        if output_view.is_some() {
                                            p { class: "kit-start__output-kind", "{output_kind_label}" }
                                            p { class: "kit-start__output-body", "{output_body}" }
                                        }
                                    }
                                },
                                _ => rsx! {
                                    div { class: "kit-start__new",
                                        input {
                                            class: "kit-start__address",
                                            r#type: "text",
                                            "aria-label": "Web address",
                                            placeholder: "Enter a web address",
                                        }
                                        h2 { class: "kit-start__heading", "Tools" }
                                        ul { class: "kit-start__list",
                                            for chip in tool_chip_list.iter() {
                                                li { key: "{chip.id}", "{chip.entry_label}" }
                                            }
                                        }
                                        h2 { class: "kit-start__heading", "Suggested" }
                                        if loading {
                                            Skeleton { width: "100%", height: "36px" }
                                            Skeleton { width: "100%", height: "36px" }
                                        } else {
                                            ul { class: "kit-start__list",
                                                for suggestion in suggestion_list.iter() {
                                                    {
                                                        let key = suggestion.id.clone();
                                                        let label = suggestion.entry_label.clone();
                                                        let prompt = suggestion.prompt_text.clone();
                                                        let artifact = suggestion.artifact.clone();
                                                        rsx! {
                                                            li { key: "{key}",
                                                                button {
                                                                    r#type: "button",
                                                                    class: "kit-start__link",
                                                                    onclick: move |_| match artifact.clone() {
                                                                        Some(name) => open_output.call(name),
                                                                        None => on_send.call(SendRequest { text: prompt.clone(), attachments: Vec::new() }),
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
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Removes a tab. Closing the last one folds the work area and returns focus to its control.
fn close_tab(
    tabs: &mut Signal<Vec<WorkTab>>,
    active: &mut Signal<usize>,
    area_open: &mut Signal<bool>,
    tab_id: usize,
    open_id: &str,
) {
    let before = tabs.peek().len();
    let Some(index) = tabs.peek().iter().position(|t| t.id == tab_id) else {
        return;
    };
    let next_index = active_after_close(index, before);
    tabs.write().remove(index);
    if tabs.peek().is_empty() {
        area_open.set(false);
        focus_after_frame(open_id);
    } else {
        let next = tabs.peek()[next_index].id;
        active.set(next);
    }
}
