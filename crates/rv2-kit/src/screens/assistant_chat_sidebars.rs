//! A plain assistant chat with web search, for products with several sections. Chats are grouped by
//! project and can be narrowed to one. Outputs open in a resizable work area on the right.
//!
//! Design choice: consecutive web searches fold into one `ToolCallGroup` line, which expands to list
//! the queries.
//! Design choice: the work area is resized with pointer drag or, when its edge has focus, with the
//! arrow keys. Its minimum width keeps the tabs usable.
//! Design choice: the approval control maps onto `PromptInputBar`'s policy (Approve is `ConfirmEach`,
//! Automatic is `AutoSafe`).
//! Not implemented: the global search (Ctrl or Cmd+K), which belongs to the app frame. Answer text
//! appears whole; the typing effect is not reproduced.
//! Browser checks: choosing a project narrows the list; the drawer on a narrow screen closes on
//! Escape; dragging the edge resizes the work area.

use std::rc::Rc;

use dioxus::prelude::*;
use rv2_affect::skeleton::Skeleton;
use rv2_affect::tool_call_card::{GroupCall, ToolCallGroup, ToolCallStatus};

use crate::components::prompt_input_bar::{ApprovalPolicy, Model, PromptInputBar, Submission};
use crate::internal::{KIT_CSS, focus_after_frame, next_id, sleep};

const CSS: Asset = asset!("/assets/assistant-chat-sidebars.css");

/// Work area widths in px.
const MIN_WORK_WIDTH: f64 = 280.0;
const DEFAULT_WORK_WIDTH: f64 = 360.0;
const KEY_STEP: f64 = 16.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    pub id: String,
    pub entry_label: String,
    pub icon_key: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    pub id: String,
    pub entry_name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChatItem {
    pub id: String,
    pub entry_title: String,
    pub project_id: Option<String>,
    pub is_pinned: bool,
    pub updated_at: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchState {
    Running,
    Done,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchStep {
    pub query: String,
    pub state: SearchState,
    pub pages: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChatMessage {
    User {
        id: String,
        text: String,
    },
    Assistant {
        id: String,
        text: String,
        searches: Vec<SearchStep>,
        outputs: Vec<String>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputKind {
    Document,
    Sheet,
    Folder,
    Code,
}

impl OutputKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Document => "Document",
            Self::Sheet => "Sheet",
            Self::Folder => "Folder",
            Self::Code => "Code",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OutputFile {
    pub entry_name: String,
    pub output_kind: OutputKind,
    pub content: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ApprovalMode {
    #[default]
    Auto,
    Approve,
}

/// The approval control's mapping onto the composer's policy.
pub fn policy_for(mode: ApprovalMode) -> ApprovalPolicy {
    match mode {
        ApprovalMode::Auto => ApprovalPolicy::AutoSafe,
        ApprovalMode::Approve => ApprovalPolicy::ConfirmEach,
    }
}

/// Chats shown for the selected project; all of them when none is selected.
pub fn visible_chats<'a>(chats: &'a [ChatItem], selected: Option<&str>) -> Vec<&'a ChatItem> {
    chats
        .iter()
        .filter(|chat| match selected {
            None => true,
            Some(project) => chat.project_id.as_deref() == Some(project),
        })
        .collect()
}

/// A project heading and its chats, in the project list's order. Chats with no project come last.
pub fn group_chats<'a>(
    chats: &[&'a ChatItem],
    projects: &[Project],
) -> Vec<(String, Vec<&'a ChatItem>)> {
    let mut groups: Vec<(String, Vec<&'a ChatItem>)> = Vec::new();
    for project in projects {
        let members: Vec<&'a ChatItem> = chats
            .iter()
            .copied()
            .filter(|chat| {
                !chat.is_pinned && chat.project_id.as_deref() == Some(project.id.as_str())
            })
            .collect();
        if !members.is_empty() {
            groups.push((project.entry_name.clone(), members));
        }
    }
    let loose: Vec<&'a ChatItem> = chats
        .iter()
        .copied()
        .filter(|chat| !chat.is_pinned && chat.project_id.is_none())
        .collect();
    if !loose.is_empty() {
        groups.push(("No project".to_string(), loose));
    }
    groups
}

/// The fold line for a run of searches.
pub fn search_summary(count: usize) -> String {
    match count {
        1 => "Searched the web, 1 search".to_string(),
        n => format!("Searched the web, {n} searches"),
    }
}

/// Outline path for a section's icon key. A key with no glyph shows the label's first letter instead.
pub fn section_glyph(key: &str) -> Option<&'static str> {
    match key {
        "chat" => Some(
            "M2.5 4.5a2 2 0 0 1 2-2h7a2 2 0 0 1 2 2v4.5a2 2 0 0 1-2 2H7l-3 2.5v-2.5h-.5a2 2 0 0 1-2-2z",
        ),
        "folder" => Some(
            "M2 4.5a1.5 1.5 0 0 1 1.5-1.5h2.7l1.3 1.6h5a1.5 1.5 0 0 1 1.5 1.5v5.4a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 11.5z",
        ),
        "reports" => Some("M4 2.5h5.5L12.5 5.5v8H4zM9.5 2.5v3h3M6 9h4M6 11.5h3"),
        _ => None,
    }
}

/// Work area width kept between its minimum and a little over half the container.
pub fn clamp_work_width(width: f64, container: f64) -> f64 {
    let max = (container * 0.6).max(MIN_WORK_WIDTH);
    width.clamp(MIN_WORK_WIDTH, max)
}

/// Keys that move the work area edge: left widens it, right narrows it.
pub fn width_after_key(width: f64, container: f64, key: &Key) -> Option<f64> {
    match key {
        Key::ArrowLeft => Some(clamp_work_width(width + KEY_STEP, container)),
        Key::ArrowRight => Some(clamp_work_width(width - KEY_STEP, container)),
        _ => None,
    }
}

/// Sending waits while the chat is loading or empty, and when the text is blank.
pub fn can_send(text: &str, loading: bool, empty: bool) -> bool {
    !loading && !empty && !text.trim().is_empty()
}

/// Shows an output in the work area, reusing its tab when one is already open.
fn open_work_tab(
    mut tabs: Signal<Vec<String>>,
    mut active: Signal<usize>,
    mut open: Signal<bool>,
    name: &str,
) {
    let mut list = tabs.peek().clone();
    let index = match list.iter().position(|n| n == name) {
        Some(index) => index,
        None => {
            list.push(name.to_string());
            list.len() - 1
        }
    };
    tabs.set(list);
    active.set(index);
    open.set(true);
}

/// Rows of a sheet's content: one line per row, cells separated by commas. The first row is the header.
pub fn sheet_rows(content: &str) -> Vec<Vec<String>> {
    content
        .lines()
        .map(|line| {
            line.split(',')
                .map(|cell| cell.trim().to_string())
                .collect()
        })
        .collect()
}

/// Spreadsheet reference of a zero-based cell: column letter, then the row number counted from 1.
pub fn cell_ref(row: usize, col: usize) -> String {
    format!("{}{}", char::from(b'A' + (col % 26) as u8), row + 1)
}

/// An output in the work area: a sheet with a formula bar over its grid, a document as paragraphs,
/// a folder as a list, or code.
fn output_view(file: &OutputFile, mut cell: Signal<(usize, usize)>) -> Element {
    let kind = file.output_kind.label();
    match file.output_kind {
        OutputKind::Sheet => {
            let rows = sheet_rows(&file.content);
            let header = rows.first().cloned().unwrap_or_default();
            let body: Vec<Vec<String>> = rows.iter().skip(1).cloned().collect();
            let (active_row, active_col) = cell();
            let reference = cell_ref(active_row, active_col);
            let value = body
                .get(active_row.saturating_sub(1))
                .and_then(|line| line.get(active_col))
                .cloned()
                .unwrap_or_default();
            rsx! {
                p { class: "kit-sb__kind", "{kind}" }
                div { class: "kit-sb__formula",
                    span { class: "kit-sb__ref", "{reference}" }
                    span { class: "kit-sb__value", "{value}" }
                }
                table { class: "kit-sb__sheet",
                    thead {
                        tr {
                            for (col, head) in header.iter().enumerate() {
                                th { key: "{col}", "{head}" }
                            }
                        }
                    }
                    tbody {
                        for (line_index, line) in body.iter().enumerate() {
                            tr { key: "{line_index}",
                                for (col, text) in line.iter().enumerate() {
                                    {
                                        let row = line_index + 1;
                                        let selected = (row, col) == (active_row, active_col);
                                        rsx! {
                                            td { key: "{col}", "data-active": selected.then_some("true"),
                                                onclick: move |_| cell.set((row, col)),
                                                "{text}"
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
        OutputKind::Document => {
            let paragraphs: Vec<String> = file.content.split("\n\n").map(str::to_string).collect();
            rsx! {
                p { class: "kit-sb__kind", "{kind}" }
                for para in paragraphs.iter() {
                    p { key: "{para}", class: "kit-sb__doc", "{para}" }
                }
            }
        }
        OutputKind::Folder => {
            let lines: Vec<String> = file.content.lines().map(str::to_string).collect();
            rsx! {
                p { class: "kit-sb__kind", "{kind}" }
                ul { class: "kit-sb__items",
                    for line in lines.iter() {
                        li { key: "{line}", "{line}" }
                    }
                }
            }
        }
        OutputKind::Code => {
            let content = file.content.clone();
            rsx! {
                p { class: "kit-sb__kind", "{kind}" }
                pre { class: "kit-sb__content", "{content}" }
            }
        }
    }
}

/// Whether the viewer asked for reduced motion. Unknown counts as yes, so the answer appears whole.
async fn prefers_reduced_motion() -> bool {
    document::eval("return window.matchMedia('(prefers-reduced-motion: reduce)').matches;")
        .await
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(true)
}

/// Reads the root's client rectangle into `bounds` as (left edge, width). Layout is read in an event handler,
/// so a drag or key move always uses the width on screen at that moment.
fn measure_bounds(mount: Signal<Option<Rc<MountedData>>>, mut bounds: Signal<Option<(f64, f64)>>) {
    let Some(data) = mount() else {
        return;
    };
    spawn(async move {
        if let Ok(rect) = data.get_client_rect().await {
            bounds.set(Some((rect.origin.x, rect.size.width)));
        }
    });
}

/// The assistant chat with a section rail, a project-grouped chat list and a work area.
#[component]
pub fn AssistantChatSidebars(
    #[props(into)] workspace_name: String,
    #[props(default, into)] account_name: String,
    #[props(default)] section_list: Vec<Section>,
    #[props(default)] active_section_id: Option<String>,
    #[props(default)] project_list: Vec<Project>,
    #[props(default)] selected_project_id: Option<String>,
    #[props(default)] chat_list: Vec<ChatItem>,
    #[props(default)] active_chat_id: Option<String>,
    #[props(default)] active_chat: Vec<ChatMessage>,
    #[props(default)] output_list: Vec<OutputFile>,
    /// Outputs open in the work area at mount, in tab order; the first one is current.
    #[props(default)]
    open_tab_list: Vec<String>,
    #[props(default)] approval_mode: ApprovalMode,
    #[props(default)] is_generating: bool,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    on_send: EventHandler<String>,
    #[props(default)] on_stop: Option<EventHandler<()>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] on_select_chat: Option<EventHandler<String>>,
    #[props(default)] on_select_project: Option<EventHandler<Option<String>>>,
    #[props(default)] on_open_output: Option<EventHandler<String>>,
) -> Element {
    let id = use_hook(|| format!("kit-sb-{}", next_id()));
    let drawer_id = format!("{id}-drawer");
    let menu_id = format!("{id}-menu");
    let trigger_id = format!("{id}-project");
    let mut menu_open = use_signal(|| false);
    let mut menu_focus = use_signal(|| 0usize);
    let mut drawer_open = use_signal(|| false);
    let mut work_width = use_signal(|| DEFAULT_WORK_WIDTH);
    let mut root_mount = use_signal(|| None::<Rc<MountedData>>);
    let bounds = use_signal(|| None::<(f64, f64)>);
    let mut dragging = use_signal(|| false);
    let initial_open = !open_tab_list.is_empty();
    let mut work_open = use_signal(move || initial_open);
    let key_trigger = trigger_id.clone();
    let mut work_tabs = use_signal(move || open_tab_list);
    let mut work_active = use_signal(|| 0usize);
    let sheet_cell = use_signal(|| (1usize, 0usize));
    let mut known_answers = use_signal(Vec::<String>::new);
    let mut known_chat = use_signal(|| None::<String>);
    let mut typing = use_signal(|| None::<(String, usize)>);
    let mut typing_ticket = use_signal(|| 0u32);
    let mut announce = use_signal(String::new);

    // Only answers that arrive while their chat is open type out; the live region gets the whole answer once.
    use_effect(use_reactive(
        (&active_chat, &active_chat_id),
        move |(messages, chat)| {
            let answer_ids: Vec<String> = messages
                .iter()
                .filter_map(|m| match m {
                    ChatMessage::Assistant { id, .. } => Some(id.clone()),
                    ChatMessage::User { .. } => None,
                })
                .collect();
            if known_chat.peek().as_ref() != chat.as_ref() {
                known_chat.set(chat);
                known_answers.set(answer_ids);
                return;
            }
            let prev = known_answers.peek().clone();
            let fresh = messages.iter().rev().find_map(|m| match m {
                ChatMessage::Assistant { id, text, .. } if !prev.contains(id) => {
                    Some((id.clone(), text.clone()))
                }
                _ => None,
            });
            known_answers.set(answer_ids);
            let Some((answer_id, text)) = fresh else {
                return;
            };
            let mine = *typing_ticket.peek() + 1;
            typing_ticket.set(mine);
            spawn(async move {
                if prefers_reduced_motion().await {
                    announce.set(text);
                    return;
                }
                announce.set(text.clone());
                let total = text.chars().count();
                for shown in 0..=total {
                    if *typing_ticket.peek() != mine {
                        return;
                    }
                    typing.set(Some((answer_id.clone(), shown)));
                    sleep(16).await;
                }
                typing.set(None);
            });
        },
    ));

    let selected = selected_project_id.clone();
    let project_label = match &selected {
        None => "All projects".to_string(),
        Some(sel) => project_list
            .iter()
            .find(|p| &p.id == sel)
            .map(|p| p.entry_name.clone())
            .unwrap_or_else(|| "All projects".to_string()),
    };
    let chats_visible = visible_chats(&chat_list, selected.as_deref());
    let pinned: Vec<&ChatItem> = chats_visible
        .iter()
        .copied()
        .filter(|c| c.is_pinned)
        .collect();
    let groups = if selected.is_some() {
        vec![(
            project_label.clone(),
            chats_visible
                .iter()
                .copied()
                .filter(|c| !c.is_pinned)
                .collect::<Vec<_>>(),
        )]
    } else {
        group_chats(&chats_visible, &project_list)
    };
    let initial = workspace_name.chars().take(1).collect::<String>();
    let account_initial = account_name.chars().take(1).collect::<String>();
    let loading = is_loading;
    let empty = is_empty;
    let width = work_width();
    let width_text = format!("{width:.0}");
    let width_css = format!("width: {width:.0}px");
    let min_text = format!("{MIN_WORK_WIDTH:.0}");
    let tabs_now = work_tabs();
    let active_tab = work_active();
    let shown_tab = tabs_now.get(active_tab).cloned();
    let shown_output = shown_tab
        .as_ref()
        .and_then(|name| output_list.iter().find(|o| &o.entry_name == name).cloned());
    let options: Vec<Option<String>> = std::iter::once(None)
        .chain(project_list.iter().map(|p| Some(p.id.clone())))
        .collect();
    let models = vec![Model {
        model_id: "standard".to_string(),
        model_name: "Standard".to_string(),
        model_mark: None,
        model_note: None,
    }];

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "kit-sb", "data-drawer": drawer_open().then_some("open"), "data-dragging": dragging().then_some("true"),
            onmounted: move |event: Event<MountedData>| root_mount.set(Some(event.data())),
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape {
                    if menu_open() {
                        menu_open.set(false);
                        focus_after_frame(&key_trigger);
                    } else if drawer_open() {
                        drawer_open.set(false);
                    }
                }
            },
            onpointermove: move |event: PointerEvent| {
                if dragging() {
                    if let Some((left, container)) = bounds() {
                        let client = event.client_coordinates().x;
                        work_width.set(clamp_work_width(left + container - client, container));
                    }
                }
            },
            onpointerup: move |_| dragging.set(false),
            button {
                r#type: "button",
                class: "kit-sb__menu",
                "aria-expanded": if drawer_open() { "true" } else { "false" },
                "aria-controls": "{drawer_id}",
                onclick: move |_| drawer_open.toggle(),
                "Menu"
            }
            div { id: "{drawer_id}", class: "kit-sb__drawer",
                nav { class: "kit-sb__rail", "aria-label": "Sections",
                    div { class: "kit-sb__mark", "aria-hidden": "true", "{initial}" }
                    for section in section_list.iter() {
                        {
                            let key = section.id.clone();
                            let label = section.entry_label.clone();
                            let glyph = section_glyph(&section.icon_key);
                            let initial = section.entry_label.chars().take(1).collect::<String>();
                            let current = active_section_id.as_deref() == Some(section.id.as_str());
                            let target = section.entry_label.clone();
                            rsx! {
                                button {
                                    key: "{key}",
                                    r#type: "button",
                                    class: "kit-sb__icon",
                                    "aria-label": "{label}",
                                    title: "{label}",
                                    "aria-current": current.then_some("page"),
                                    onclick: move |_| {
                                        if let Some(nav) = on_navigate {
                                            nav.call(target.clone());
                                        }
                                    },
                                    span { "aria-hidden": "true",
                                        if let Some(d) = glyph {
                                            svg { class: "kit-sb__glyph", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round",
                                                path { d: "{d}" }
                                            }
                                        } else {
                                            "{initial}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "kit-sb__avatar", "aria-hidden": "true", "{account_initial}" }
                }
                aside { class: "kit-sb__list", "aria-label": "Chats",
                    div { class: "kit-sb__switcher",
                        button {
                            id: "{trigger_id}",
                            r#type: "button",
                            class: "kit-sb__project",
                            "aria-haspopup": "listbox",
                            "aria-expanded": if menu_open() { "true" } else { "false" },
                            "aria-controls": "{menu_id}",
                            onclick: move |_| menu_open.toggle(),
                            "{project_label}"
                        }
                        if menu_open() {
                            ul { id: "{menu_id}", role: "listbox", class: "kit-sb__menu-list", "aria-label": "Projects",
                                onkeydown: {
                                    let count = options.len();
                                    let menu_key = id.clone();
                                    move |event: KeyboardEvent| {
                                        match event.key() {
                                            Key::ArrowDown | Key::ArrowUp => {
                                                event.prevent_default();
                                                let forward = event.key() == Key::ArrowDown;
                                                let next = crate::screens::assistant_start_page::tab_after_move(menu_focus(), count, forward);
                                                menu_focus.set(next);
                                                focus_after_frame(&format!("{menu_key}-option-{next}"));
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                                for (index, option) in options.iter().enumerate() {
                                    {
                                        let value = option.clone();
                                        let name = match option {
                                            None => "All projects".to_string(),
                                            Some(pid) => project_list.iter().find(|p| &p.id == pid).map(|p| p.entry_name.clone()).unwrap_or_default(),
                                        };
                                        let checked = value == selected;
                                        let dom = format!("{id}-option-{index}");
                                        let trigger_target = trigger_id.clone();
                                        rsx! {
                                            li { key: "{index}", role: "option", "aria-selected": if checked { "true" } else { "false" },
                                                button {
                                                    id: "{dom}",
                                                    r#type: "button",
                                                    class: "kit-sb__option",
                                                    onclick: move |_| {
                                                        menu_open.set(false);
                                                        if let Some(pick) = on_select_project {
                                                            pick.call(value.clone());
                                                        }
                                                        focus_after_frame(&trigger_target);
                                                    },
                                                    span { "{name}" }
                                                    if checked {
                                                        span { class: "kit-sb__check", "aria-hidden": "true", "✓" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if loading {
                        div { class: "kit-sb__skeleton", "aria-busy": "true",
                            span { class: "kit-sb__bar" }
                            span { class: "kit-sb__bar" }
                            span { class: "kit-sb__bar" }
                        }
                    } else if empty || chat_list.is_empty() {
                        p { class: "kit-sb__note", "There are no chats yet." }
                    } else {
                        if !pinned.is_empty() {
                            h2 { class: "kit-sb__heading", "Pinned" }
                            ul { class: "kit-sb__chats",
                                for chat in pinned.iter() {
                                    {
                                        let key = chat.id.clone();
                                        let chat_id = chat.id.clone();
                                        let title = chat.entry_title.clone();
                                        let current = active_chat_id.as_deref() == Some(chat.id.as_str());
                                        rsx! {
                                            li { key: "{key}",
                                                button {
                                                    r#type: "button",
                                                    class: "kit-sb__chat",
                                                    "aria-current": current.then_some("true"),
                                                    onclick: move |_| {
                                                        if let Some(pick) = on_select_chat {
                                                            pick.call(chat_id.clone());
                                                        }
                                                    },
                                                    "{title}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        for (heading, members) in groups.iter() {
                            {
                                let heading_text = heading.clone();
                                rsx! {
                                    h2 { class: "kit-sb__heading", "{heading_text}" }
                                    ul { class: "kit-sb__chats",
                                        for chat in members.iter() {
                                            {
                                                let key = chat.id.clone();
                                                let chat_id = chat.id.clone();
                                                let title = chat.entry_title.clone();
                                                let current = active_chat_id.as_deref() == Some(chat.id.as_str());
                                                rsx! {
                                                    li { key: "{key}",
                                                        button {
                                                            r#type: "button",
                                                            class: "kit-sb__chat",
                                                            "aria-current": current.then_some("true"),
                                                            onclick: move |_| {
                                                                if let Some(pick) = on_select_chat {
                                                                    pick.call(chat_id.clone());
                                                                }
                                                            },
                                                            "{title}"
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
            main { class: "kit-sb__main",
                section { class: "kit-sb__conversation", "aria-label": "Conversation",
                    p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce()}" }
                    div { class: "kit-sb__thread", role: "log", "aria-live": "off", "aria-busy": loading.then_some("true"),
                        if loading {
                            Skeleton { width: "60%", height: "40px" }
                            Skeleton { width: "100%", height: "96px" }
                        } else if empty {
                            p { class: "kit-sb__note", "Start a conversation. Your chats appear on the left once you send one." }
                        } else {
                            for message in active_chat.iter() {
                                {
                                    match message {
                                        ChatMessage::User { id: key, text } => rsx! {
                                            div { key: "{key}", class: "kit-sb__msg kit-sb__msg--user",
                                                p { class: "kit-sb__bubble", "{text}" }
                                            }
                                        },
                                        ChatMessage::Assistant { id: key, text, searches, outputs } => {
                                            let count = searches.len();
                                            let summary = search_summary(count);
                                            let typing_now = matches!(typing(), Some((ref tid, _)) if tid == key);
                                            let shown_text: String = match typing() {
                                                Some((ref tid, shown)) if tid == key => text.chars().take(shown).collect(),
                                                _ => text.clone(),
                                            };
                                            let calls: Vec<GroupCall> = searches.iter().map(|s| GroupCall {
                                                name: s.query.clone(),
                                                icon: None,
                                                status: if s.state == SearchState::Running { ToolCallStatus::Running } else { ToolCallStatus::Success },
                                                duration: None,
                                                started_at: None,
                                            }).collect();
                                            rsx! {
                                                div { key: "{key}", class: "kit-sb__msg kit-sb__msg--assistant",
                                                    if count > 0 {
                                                        ToolCallGroup {
                                                            calls,
                                                            label: summary,
                                                            running_label: "Searching the web…",
                                                            children: rsx! {
                                                                ul { class: "kit-sb__queries",
                                                                    for search in searches.iter() {
                                                                        li { "{search.query}" }
                                                                    }
                                                                }
                                                            },
                                                        }
                                                    }
                                                    p { class: "kit-sb__text", "aria-hidden": typing_now.then_some("true"), "{shown_text}" }
                                                    if !outputs.is_empty() {
                                                        div { class: "kit-sb__chips",
                                                            for name in outputs.iter() {
                                                                {
                                                                    let open_name = name.clone();
                                                                    let label = name.clone();
                                                                    rsx! {
                                                                        button {
                                                                            r#type: "button",
                                                                            class: "kit-sb__chip",
                                                                            onclick: move |_| {
                                                                                open_work_tab(work_tabs, work_active, work_open, &open_name);
                                                                                if let Some(open) = on_open_output {
                                                                                    open.call(open_name.clone());
                                                                                }
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
                                }
                            }
                        }
                    }
                    div { class: "kit-sb__composer",
                        if is_generating {
                            button {
                                r#type: "button",
                                class: "kit-sb__stop",
                                onclick: move |_| {
                                    if let Some(stop) = on_stop {
                                        stop.call(());
                                    }
                                },
                                "Stop"
                            }
                        }
                        PromptInputBar {
                            root_class: "kit-sb__input",
                            empty_hint: "Ask anything, or search the web",
                            model_options: models,
                            approval_policy: policy_for(approval_mode),
                            on_submit: move |submission: Submission| {
                                if can_send(&submission.text, loading, empty) {
                                    on_send.call(submission.text);
                                }
                            },
                        }
                    }
                }
                if work_open() {
                    div {
                        class: "kit-sb__edge",
                        role: "separator",
                        tabindex: "0",
                        "aria-orientation": "vertical",
                        "aria-label": "Resize work area",
                        "aria-valuenow": "{width_text}",
                        "aria-valuemin": "{min_text}",
                        onpointerdown: move |_| {
                            measure_bounds(root_mount, bounds);
                            dragging.set(true);
                        },
                        onfocus: move |_| measure_bounds(root_mount, bounds),
                        onkeydown: move |event: KeyboardEvent| {
                            if let Some((_, container)) = bounds() {
                                if let Some(next) = width_after_key(work_width(), container, &event.key()) {
                                    event.prevent_default();
                                    work_width.set(next);
                                }
                            }
                        },
                    }
                    aside { class: "kit-sb__work", style: "{width_css}", "aria-label": "Work area",
                        div { class: "kit-sb__tabs", role: "tablist",
                            for (index, name) in tabs_now.iter().enumerate() {
                                {
                                    let title = name.clone();
                                    let close_label = format!("Close {name}");
                                    let selected = index == active_tab;
                                    rsx! {
                                        div { key: "{title}", class: "kit-sb__tab", "data-selected": selected.then_some("true"),
                                            button {
                                                r#type: "button",
                                                role: "tab",
                                                "aria-selected": if selected { "true" } else { "false" },
                                                onclick: move |_| work_active.set(index),
                                                "{title}"
                                            }
                                            button {
                                                r#type: "button",
                                                class: "kit-sb__tab-close",
                                                "aria-label": "{close_label}",
                                                onclick: move |_| {
                                                    let mut list = work_tabs.peek().clone();
                                                    list.remove(index);
                                                    let last = list.is_empty();
                                                    work_tabs.set(list);
                                                    work_active.set(index.saturating_sub(1));
                                                    if last {
                                                        work_open.set(false);
                                                    }
                                                },
                                                "×"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "kit-sb__panel", role: "tabpanel",
                            if let Some(file) = shown_output.clone() {
                                {output_view(&file, sheet_cell)}
                            }
                        }
                    }
                }
            }
        }
    }
}
