//! Issue board: issues sorted into priority columns, moved by drag or by keyboard, shown as a board or a flat list
//! with filter and sort, with an issue panel and a New issue dialog.
//!
//! Design choice: a move applies to the board at once and is reported through `on_move`; the host persists it.
//! Design choice: keyboard move is a mode. Space picks a card up, arrows set its target column and position, Space
//! drops it, Escape cancels with no call. The drop line marks the target position among the cards.
//! Design choice: Filter and Sort are menus of radio and check items, so the checked state is in the markup.
//! Design choice: Escape closes the panel through a document listener, so it works with focus on `<body>`.
//! Design choice: the sidebar is drawn inline so the screen stands alone, without the workspace frame.

use dioxus::prelude::*;

use crate::internal::*;

const CSS: Asset = asset!("/assets/issue-board-screen.css");

/// Listens for Escape on the whole document, so it closes the open panel from any focus, including `<body>`.
/// The listener is registered under `key` and sends `escape`, or `end` when it is removed.
fn escape_listen_script(key: &str) -> String {
    format!(
        r#"const owners = (window.__kitIbOwners ||= {{}});
owners[{key:?}]?.remove();
const handlers = [];
const send = (message) => {{ try {{ dioxus.send(message); }} catch (_) {{}} }};
owners[{key:?}] = {{ remove: () => {{ handlers.forEach((undo) => undo()); send("end"); }} }};
const onKey = (e) => {{ if (e.key === "Escape" && !e.defaultPrevented) {{ e.preventDefault(); send("escape"); }} }};
document.addEventListener("keydown", onKey);
handlers.push(() => document.removeEventListener("keydown", onKey));"#
    )
}

fn escape_release_script(key: &str) -> String {
    format!("window.__kitIbOwners?.[{key:?}]?.remove();")
}

/// Closes the Filter and Sort menus on a press outside their buttons and lists. Registered under `key`; sends
/// `outside`, or `end` when removed.
fn menus_listen_script(key: &str) -> String {
    format!(
        r#"const owners = (window.__kitIbMenuOwners ||= {{}});
owners[{key:?}]?.remove();
const handlers = [];
const send = (message) => {{ try {{ dioxus.send(message); }} catch (_) {{}} }};
owners[{key:?}] = {{ remove: () => {{ handlers.forEach((undo) => undo()); send("end"); }} }};
const base = {key:?};
const inside = (node) => [base + "-filter-btn", base + "-filter-menu", base + "-sort-btn", base + "-sort-menu"]
  .some((id) => !!document.getElementById(id)?.contains(node));
const onDown = (e) => {{ if (!inside(e.target)) send("outside"); }};
document.addEventListener("pointerdown", onDown, true);
handlers.push(() => document.removeEventListener("pointerdown", onDown, true));"#
    )
}

fn menus_release_script(key: &str) -> String {
    format!("window.__kitIbMenuOwners?.[{key:?}]?.remove();")
}

/// Moves focus to the next or previous focusable control inside the issue panel, wrapping at both ends.
const TRAP_TAB: &str = "(() => { const p = document.getElementById('PANEL'); if (!p) return; const f = [...p.querySelectorAll('button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex=\"0\"]')]; if (!f.length) return; const i = f.indexOf(document.activeElement); const n = BACK ? (i <= 0 ? f.length - 1 : i - 1) : (i < 0 || i >= f.length - 1 ? 0 : i + 1); f[n].focus(); })();";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PriorityTone {
    Low,
    Medium,
    High,
    Done,
}

impl PriorityTone {
    pub fn word(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Done => "Done",
        }
    }

    fn mark(self) -> &'static str {
        match self {
            Self::Low => "↓",
            Self::Medium => "–",
            Self::High => "↑",
            Self::Done => "■",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueStatus {
    Review,
    Attention,
    Progress,
    Fixed,
}

impl IssueStatus {
    pub const ALL: [Self; 4] = [Self::Review, Self::Attention, Self::Progress, Self::Fixed];

    pub fn word(self) -> &'static str {
        match self {
            Self::Review => "In review",
            Self::Attention => "Needs attention",
            Self::Progress => "In progress",
            Self::Fixed => "Fixed",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Review => "review",
            Self::Attention => "attention",
            Self::Progress => "progress",
            Self::Fixed => "fixed",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.key() == key)
    }

    fn mark(self) -> &'static str {
        match self {
            Self::Review => "◎",
            Self::Attention => "!",
            Self::Progress => "◐",
            Self::Fixed => "✓",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoardColumn {
    pub id: String,
    pub column_label: String,
    pub priority_tone: PriorityTone,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    pub id: String,
    pub column_id: String,
    pub title: String,
    pub description: Option<String>,
    pub issue_status: Option<IssueStatus>,
    pub comment_count: u32,
    pub attachment_count: u32,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Draft {
    pub title: String,
    pub description: String,
    pub column_id: String,
}

/// Fields the issue panel can change. `None` leaves a field as it is.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IssuePatch {
    pub title: Option<String>,
    pub description: Option<String>,
    pub issue_status: Option<IssueStatus>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    Board,
    List,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortKey {
    #[default]
    Priority,
    Updated,
    Title,
}

/// Where a card would land: a column and an index among that column's other cards.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub column_id: String,
    pub index: usize,
}

/// Cards of one column, in board order.
pub fn column_cards<'a>(issues: &'a [Issue], column_id: &str) -> Vec<&'a Issue> {
    issues.iter().filter(|i| i.column_id == column_id).collect()
}

/// Moves an issue to a column before `before_id`, or to the end when None. The issue keeps its id and fields.
pub fn move_issue(
    issues: &[Issue],
    issue_id: &str,
    to_column: &str,
    before_id: Option<&str>,
) -> Vec<Issue> {
    let Some(pos) = issues.iter().position(|i| i.id == issue_id) else {
        return issues.to_vec();
    };
    let mut next = issues.to_vec();
    let mut moved = next.remove(pos);
    moved.column_id = to_column.to_string();
    let at = before_id
        .and_then(|b| next.iter().position(|i| i.id == b))
        .unwrap_or(next.len());
    next.insert(at, moved);
    next
}

/// The card a target index names among a column's cards, leaving out the card being moved.
pub fn before_for(issues: &[Issue], issue_id: &str, target: &Target) -> Option<String> {
    column_cards(issues, &target.column_id)
        .into_iter()
        .filter(|i| i.id != issue_id)
        .nth(target.index)
        .map(|i| i.id.clone())
}

/// One step of a keyboard move: `dx` changes column, `dy` changes position within the column.
pub fn step_target(
    columns: &[BoardColumn],
    issues: &[Issue],
    current: &Target,
    dx: isize,
    dy: isize,
) -> Target {
    let col_index = columns
        .iter()
        .position(|c| c.id == current.column_id)
        .unwrap_or(0) as isize;
    let last_col = columns.len().saturating_sub(1) as isize;
    if dx != 0 {
        let column_id = columns
            .get((col_index + dx).clamp(0, last_col) as usize)
            .map(|c| c.id.clone())
            .unwrap_or_else(|| current.column_id.clone());
        let len = column_cards(issues, &column_id).len();
        return Target {
            column_id,
            index: current.index.min(len),
        };
    }
    let len = column_cards(issues, &current.column_id).len() as isize;
    Target {
        column_id: current.column_id.clone(),
        index: (current.index as isize + dy).clamp(0, len) as usize,
    }
}

/// Issues a filter admits.
pub fn filtered(
    issues: &[Issue],
    status: Option<IssueStatus>,
    assignee: Option<&str>,
) -> Vec<Issue> {
    issues
        .iter()
        .filter(|i| status.is_none_or(|s| i.issue_status == Some(s)))
        .filter(|i| assignee.is_none_or(|a| i.assignee.as_deref() == Some(a)))
        .cloned()
        .collect()
}

/// Sorts a list by the key; priority follows the columns' order. Descending reverses the result.
pub fn sorted(
    issues: &[Issue],
    columns: &[BoardColumn],
    key: SortKey,
    descending: bool,
) -> Vec<Issue> {
    let rank = |col: &str| {
        columns
            .iter()
            .position(|c| c.id == col)
            .unwrap_or(usize::MAX)
    };
    let mut out = issues.to_vec();
    match key {
        SortKey::Priority => out.sort_by_key(|i| rank(&i.column_id)),
        SortKey::Title => out.sort_by_key(|i| i.title.to_lowercase()),
        SortKey::Updated => {}
    }
    if descending {
        out.reverse();
    }
    out
}

#[derive(Clone, PartialEq)]
struct CardView {
    id: String,
    title: String,
    description: Option<String>,
    status_word: &'static str,
    status_key: &'static str,
    status_mark: &'static str,
    priority_word: &'static str,
    priority_mark: &'static str,
    counts: Option<String>,
    aria: String,
    dragging: bool,
    picked: bool,
    index: usize,
}

#[derive(Clone, PartialEq)]
struct ColumnView {
    id: String,
    label: String,
    tone: &'static str,
    count: usize,
    is_over: bool,
    drop_index: Option<usize>,
    cards: Vec<CardView>,
}

fn status_word(issue: &Issue) -> &'static str {
    issue.issue_status.map(|s| s.word()).unwrap_or("No status")
}

/// Comment and file counts that are not zero, joined; None when a card has neither.
fn counts_text(comments: u32, files: u32) -> Option<String> {
    let plural = |n: u32, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
    let mut parts = Vec::new();
    if comments > 0 {
        parts.push(plural(comments, "comment", "comments"));
    }
    if files > 0 {
        parts.push(plural(files, "file", "files"));
    }
    (!parts.is_empty()).then(|| parts.join(" · "))
}

fn card_view(
    issue: &Issue,
    col_label: &str,
    tone: PriorityTone,
    index: usize,
    dragging: Option<&str>,
    picked: Option<&str>,
) -> CardView {
    let status = status_word(issue);
    CardView {
        id: issue.id.clone(),
        title: issue.title.clone(),
        description: issue.description.clone(),
        status_word: status,
        status_key: issue.issue_status.map(|s| s.key()).unwrap_or("none"),
        status_mark: issue.issue_status.map(|s| s.mark()).unwrap_or("·"),
        priority_word: tone.word(),
        priority_mark: tone.mark(),
        counts: counts_text(issue.comment_count, issue.attachment_count),
        aria: format!("{}, {}, {}", issue.title, status, col_label),
        dragging: dragging == Some(issue.id.as_str()),
        picked: picked == Some(issue.id.as_str()),
        index,
    }
}

/// One radio choice in the Filter menu.
#[derive(Clone, Debug, PartialEq)]
enum FilterChoice {
    Status(Option<IssueStatus>),
    Assignee(Option<String>),
}

/// Arrow, Home and End move focus among a menu's items; Escape closes the menu and returns focus to its button.
/// Tab closes the menu and lets focus leave it. `at` holds the focused item, set in Rust before the DOM focus
/// call, so keys repeated faster than a frame still step from the right item.
fn menu_item_keys(
    event: KeyboardEvent,
    mut open: Signal<bool>,
    mut at: Signal<usize>,
    button_id: &str,
    item_prefix: &str,
    count: usize,
) {
    let index = at();
    let target = match event.key() {
        Key::ArrowDown => (index + 1) % count,
        Key::ArrowUp => (index + count - 1) % count,
        Key::Home => 0,
        Key::End => count - 1,
        Key::Escape => {
            event.prevent_default();
            open.set(false);
            focus_after_frame(button_id);
            return;
        }
        Key::Tab => {
            open.set(false);
            return;
        }
        _ => return,
    };
    event.prevent_default();
    at.set(target);
    let id = format!("{item_prefix}{target}");
    document::eval(&format!(
        "document.getElementById({id:?})?.focus({{ preventScroll: true }});"
    ));
}

fn card_id(base: &str, issue_id: &str) -> String {
    format!("{base}-card-{issue_id}")
}

/// Issue board page: top bar, board or list, issue panel and the New issue dialog.
#[component]
pub fn IssueBoardScreen(
    #[props(default = "Juniper Works".to_string(), into)] workspace_name: String,
    #[props(default = "Ada Lovelace".to_string(), into)] account_name: String,
    #[props(default)] board_columns: Vec<BoardColumn>,
    #[props(default)] issues: Vec<Issue>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_create: EventHandler<Draft>,
    #[props(default)] on_move: EventHandler<(String, String, Option<String>)>,
    #[props(default)] on_update: EventHandler<(String, IssuePatch)>,
    #[props(default)] on_open: EventHandler<String>,
    #[props(default)] on_navigate: EventHandler<String>,
) -> Element {
    let base = use_hook(|| format!("kit-ib-{}", next_id()));
    let mut board = use_signal({
        let seed = issues.clone();
        move || seed.clone()
    });
    let mut view = use_signal(ViewMode::default);
    let mut status_filter = use_signal(|| None::<IssueStatus>);
    let mut assignee_filter = use_signal(|| None::<String>);
    let mut sort_key = use_signal(SortKey::default);
    let mut descending = use_signal(|| false);
    let mut moving = use_signal(|| None::<(String, Target)>);
    let mut dragging = use_signal(|| None::<String>);
    let mut drop_at = use_signal(|| None::<Target>);
    let mut open_id = use_signal(|| None::<String>);
    let mut filter_open = use_signal(|| false);
    let mut sort_open = use_signal(|| false);
    let mut filter_at = use_signal(|| 0_usize);
    let mut sort_at = use_signal(|| 0_usize);
    let menu_base = base.clone();
    use_effect(move || {
        if filter_open() || sort_open() {
            let key = menu_base.clone();
            spawn(async move {
                let mut events = document::eval(&menus_listen_script(&key));
                while let Ok(message) = events.recv::<String>().await {
                    match message.as_str() {
                        "outside" => {
                            filter_open.set(false);
                            sort_open.set(false);
                        }
                        "end" => break,
                        _ => {}
                    }
                }
            });
        } else {
            document::eval(&menus_release_script(&menu_base));
        }
    });
    let panel_dom = format!("{base}-panel");
    let panel_for_focus = panel_dom.clone();
    let esc_base = base.clone();
    use_effect(move || match open_id() {
        Some(id) => {
            focus_after_frame(&panel_for_focus);
            let key = esc_base.clone();
            let card = card_id(&key, &id);
            spawn(async move {
                let mut events = document::eval(&escape_listen_script(&key));
                while let Ok(message) = events.recv::<String>().await {
                    match message.as_str() {
                        "escape" => {
                            if open_id.peek().is_some() {
                                open_id.set(None);
                                focus_after_frame(&card);
                            }
                        }
                        "end" => break,
                        _ => {}
                    }
                }
            });
        }
        None => {
            document::eval(&escape_release_script(&esc_base));
        }
    });
    let mut new_draft = use_signal(|| None::<Draft>);
    let mut activity = use_signal(Vec::<String>::new);
    let mut announce = use_signal(String::new);

    let columns = board_columns.clone();
    let current = board();
    let is_list = view() == ViewMode::List;
    let assignees: Vec<String> = {
        let mut names: Vec<String> = current.iter().filter_map(|i| i.assignee.clone()).collect();
        names.sort();
        names.dedup();
        names
    };
    let assignee_now = assignee_filter();
    let status_now = status_filter();
    let key_now = sort_key();
    let desc_now = descending();
    let list_rows: Vec<(String, String, String, String, String)> = sorted(
        &filtered(&current, status_now, assignee_now.as_deref()),
        &columns,
        key_now,
        desc_now,
    )
    .iter()
    .map(|i| {
        (
            i.id.clone(),
            i.title.clone(),
            status_word(i).to_string(),
            columns
                .iter()
                .find(|c| c.id == i.column_id)
                .map(|c| c.priority_tone.word().to_string())
                .unwrap_or_default(),
            i.assignee.clone().unwrap_or_else(|| "Unassigned".into()),
        )
    })
    .collect();
    let picked_id = moving().map(|(id, _)| id);
    let target_now = moving().map(|(_, t)| t);
    let drag_now = dragging();
    let drop_now = drop_at();
    let column_views: Vec<ColumnView> = columns
        .iter()
        .map(|col| {
            let cards = column_cards(&current, &col.id);
            let cards_views: Vec<CardView> = cards
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    card_view(
                        c,
                        &col.column_label,
                        col.priority_tone,
                        i,
                        drag_now.as_deref(),
                        picked_id.as_deref(),
                    )
                })
                .collect();
            let drop_index = match (&drop_now, &target_now) {
                (Some(t), _) if t.column_id == col.id => Some(t.index),
                (_, Some(t)) if t.column_id == col.id && picked_id.is_some() => Some(t.index),
                _ => None,
            };
            ColumnView {
                id: col.id.clone(),
                label: col.column_label.clone(),
                tone: col.priority_tone.word(),
                count: cards_views.len(),
                is_over: drop_now.as_ref().is_some_and(|t| t.column_id == col.id),
                drop_index,
                cards: cards_views,
            }
        })
        .collect();
    let open_issue = open_id().and_then(|id| current.iter().find(|i| i.id == id).cloned());
    let draft_now = new_draft();
    let first_column = columns.first().map(|c| c.id.clone()).unwrap_or_default();
    let total = current.len();
    let filter_btn = format!("{base}-filter-btn");
    let filter_prefix = format!("{base}-f-");
    let sort_btn = format!("{base}-sort-btn");
    let sort_prefix = format!("{base}-s-");
    let filter_open_now = filter_open();
    let sort_open_now = sort_open();
    let status_count = 1 + IssueStatus::ALL.len();
    let filter_choices: Vec<(String, bool, FilterChoice)> = std::iter::once((
        "All statuses".to_string(),
        status_now.is_none(),
        FilterChoice::Status(None),
    ))
    .chain(IssueStatus::ALL.iter().map(|s| {
        (
            s.word().to_string(),
            status_now == Some(*s),
            FilterChoice::Status(Some(*s)),
        )
    }))
    .chain(std::iter::once((
        "Anyone".to_string(),
        assignee_now.is_none(),
        FilterChoice::Assignee(None),
    )))
    .chain(assignees.iter().map(|name| {
        (
            name.clone(),
            assignee_now.as_deref() == Some(name.as_str()),
            FilterChoice::Assignee(Some(name.clone())),
        )
    }))
    .collect();
    let filter_count = usize::from(status_now.is_some()) + usize::from(assignee_now.is_some());
    let sort_choices = [
        ("Priority", SortKey::Priority),
        ("Updated", SortKey::Updated),
        ("Title", SortKey::Title),
    ];
    let sort_label = sort_choices
        .iter()
        .find(|(_, k)| *k == key_now)
        .map(|(label, _)| *label)
        .unwrap_or("Priority");

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "kit-ib", "aria-labelledby": "{base}-title",
            nav { class: "kit-ib__nav", "aria-label": "Sidebar",
                p { class: "kit-ib__ws", "{workspace_name}" }
                ul { class: "kit-ib__nav-list",
                    for item in ["Home", "Issues", "Projects", "Settings"] {
                        li { key: "{item}",
                            button { class: "kit-ib__nav-item", r#type: "button",
                                "aria-current": if item == "Issues" { "page" } else { "false" },
                                onclick: move |_| on_navigate.call(item.to_string()),
                                "{item}"
                            }
                        }
                    }
                }
                p { class: "kit-ib__account", "{account_name}" }
            }

            div { class: "kit-ib__page",
                header { class: "kit-ib__bar",
                    h1 { id: "{base}-title", class: "kit-ib__title", "Issues" }
                    div { class: "kit-ib__radio", role: "radiogroup", "aria-label": "Layout",
                        button { class: "kit-ib__seg", r#type: "button", role: "radio", "aria-checked": if is_list { "false" } else { "true" },
                            onclick: move |_| view.set(ViewMode::Board),
                            "Board"
                        }
                        button { class: "kit-ib__seg", r#type: "button", role: "radio", "aria-checked": if is_list { "true" } else { "false" },
                            onclick: move |_| view.set(ViewMode::List),
                            "List"
                        }
                    }
                    div { class: "kit-ib__menu-wrap",
                        button { id: "{filter_btn}", class: "kit-ib__btn", r#type: "button", "aria-haspopup": "menu",
                            "aria-expanded": "{filter_open_now}", "aria-controls": "{base}-filter-menu",
                            onclick: {
                                let prefix = filter_prefix.clone();
                                move |_| {
                                    let next = !filter_open();
                                    filter_open.set(next);
                                    sort_open.set(false);
                                    filter_at.set(0);
                                    if next {
                                        focus_after_frame(&format!("{prefix}0"));
                                    }
                                }
                            },
                            "Filter"
                            if filter_count > 0 {
                                span { class: "kit-ib__count", "{filter_count}" }
                            }
                        }
                        if filter_open_now {
                            ul { id: "{base}-filter-menu", class: "kit-ib__menu", role: "menu", "aria-label": "Filter issues",
                                for (k, (label, checked, choice)) in filter_choices.iter().enumerate() {
                                    if k == 0 {
                                        li { role: "presentation", class: "kit-ib__menu-label", "Status" }
                                    }
                                    if k == status_count {
                                        li { role: "presentation", class: "kit-ib__menu-label", "Assignee" }
                                    }
                                    li { role: "none",
                                        button { id: "{filter_prefix}{k}", class: "kit-ib__menu-item", r#type: "button", role: "menuitemradio",
                                            "aria-checked": "{checked}",
                                            onclick: {
                                                let choice = choice.clone();
                                                let button = filter_btn.clone();
                                                move |_| {
                                                    match choice.clone() {
                                                        FilterChoice::Status(s) => status_filter.set(s),
                                                        FilterChoice::Assignee(a) => assignee_filter.set(a),
                                                    }
                                                    filter_open.set(false);
                                                    focus_after_frame(&button);
                                                }
                                            },
                                            onfocus: move |_| filter_at.set(k),
                                            onkeydown: {
                                                let button = filter_btn.clone();
                                                let prefix = filter_prefix.clone();
                                                let total = filter_choices.len();
                                                move |e: KeyboardEvent| menu_item_keys(e, filter_open, filter_at, &button, &prefix, total)
                                            },
                                            span { class: "kit-ib__check", "aria-hidden": "true", if *checked { "✓" } }
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "kit-ib__menu-wrap",
                        button { id: "{sort_btn}", class: "kit-ib__btn", r#type: "button", "aria-haspopup": "menu",
                            "aria-expanded": "{sort_open_now}", "aria-controls": "{base}-sort-menu",
                            onclick: {
                                let prefix = sort_prefix.clone();
                                move |_| {
                                    let next = !sort_open();
                                    sort_open.set(next);
                                    filter_open.set(false);
                                    sort_at.set(0);
                                    if next {
                                        focus_after_frame(&format!("{prefix}0"));
                                    }
                                }
                            },
                            "Sort by: {sort_label}"
                        }
                        if sort_open_now {
                            ul { id: "{base}-sort-menu", class: "kit-ib__menu", role: "menu", "aria-label": "Sort issues",
                                for (k, (label, key)) in sort_choices.iter().enumerate() {
                                    li { key: "{k}", role: "none",
                                        button { id: "{sort_prefix}{k}", class: "kit-ib__menu-item", r#type: "button", role: "menuitemradio",
                                            "aria-checked": if *key == key_now { "true" } else { "false" },
                                            onclick: {
                                                let key = *key;
                                                let button = sort_btn.clone();
                                                move |_| {
                                                    sort_key.set(key);
                                                    sort_open.set(false);
                                                    focus_after_frame(&button);
                                                }
                                            },
                                            onfocus: move |_| sort_at.set(k),
                                            onkeydown: {
                                                let button = sort_btn.clone();
                                                let prefix = sort_prefix.clone();
                                                move |e: KeyboardEvent| menu_item_keys(e, sort_open, sort_at, &button, &prefix, 4)
                                            },
                                            span { class: "kit-ib__check", "aria-hidden": "true", if *key == key_now { "✓" } }
                                            "{label}"
                                        }
                                    }
                                }
                                li { role: "separator" }
                                li { role: "none",
                                    button { id: "{sort_prefix}3", class: "kit-ib__menu-item", r#type: "button", role: "menuitemcheckbox",
                                        "aria-checked": "{desc_now}",
                                        onclick: move |_| descending.toggle(),
                                        onfocus: move |_| sort_at.set(3),
                                        onkeydown: {
                                            let button = sort_btn.clone();
                                            let prefix = sort_prefix.clone();
                                            move |e: KeyboardEvent| menu_item_keys(e, sort_open, sort_at, &button, &prefix, 4)
                                        },
                                        span { class: "kit-ib__check", "aria-hidden": "true", if desc_now { "✓" } }
                                        "Descending"
                                    }
                                }
                            }
                        }
                    }
                    button { class: "kit-ib__btn kit-ib__btn--primary", r#type: "button", "aria-keyshortcuts": "Control+Enter",
                        onclick: {
                            let first = first_column.clone();
                            move |_| new_draft.set(Some(Draft { title: String::new(), description: String::new(), column_id: first.clone() }))
                        },
                        "New issue"
                    }
                }

                if is_empty && !is_loading {
                    div { class: "kit-ib__empty",
                        p { class: "kit-ib__empty-title", "No tasks yet" }
                        p { class: "kit-ib__empty-text", "Issues you create or are assigned appear here, sorted by priority." }
                        button { class: "kit-ib__btn kit-ib__btn--primary", r#type: "button",
                            onclick: {
                                let first = first_column.clone();
                                move |_| new_draft.set(Some(Draft { title: String::new(), description: String::new(), column_id: first.clone() }))
                            },
                            "New issue"
                        }
                    }
                } else if is_list {
                    div { class: "kit-ib__listwrap", "aria-busy": "{is_loading}",
                        if is_loading {
                            for i in 0..4 {
                                span { key: "{i}", class: "kit-ib__ph kit-ib__ph--row" }
                            }
                        } else {
                            table { class: "kit-ib__table", "aria-label": "Issues",
                                thead {
                                    tr {
                                        th { scope: "col", "Title" }
                                        th { scope: "col", "Status" }
                                        th { scope: "col", "Priority" }
                                        th { scope: "col", "Assignee" }
                                    }
                                }
                                tbody {
                                    for (id, title, status, priority, assignee) in list_rows.iter() {
                                        tr { key: "{id}", class: "kit-ib__row",
                                            td {
                                                button { class: "kit-ib__rowlink", r#type: "button",
                                                    onclick: {
                                                        let id = id.clone();
                                                        move |_| {
                                                            open_id.set(Some(id.clone()));
                                                            on_open.call(id.clone());
                                                        }
                                                    },
                                                    "{title}"
                                                }
                                            }
                                            td { "{status}" }
                                            td { "{priority}" }
                                            td { "{assignee}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "kit-ib__boardwrap", "aria-busy": "{is_loading}",
                        div { class: "kit-ib__board",
                            for (ci, col) in column_views.iter().enumerate() {
                                section { key: "{col.id}", class: "kit-ib__col", "aria-labelledby": "{base}-col-{ci}",
                                    "data-over": if col.is_over { "true" } else { "false" },
                                    ondragover: {
                                        let col_id = col.id.clone();
                                        move |event: DragEvent| {
                                            event.prevent_default();
                                            let len = column_cards(&board(), &col_id).len();
                                            drop_at.set(Some(Target { column_id: col_id.clone(), index: len }));
                                        }
                                    },
                                    ondrop: {
                                        let col_id = col.id.clone();
                                        let col_label = col.label.clone();
                                        move |event: DragEvent| {
                                            event.prevent_default();
                                            if let Some(id) = dragging() {
                                                let before = drop_at()
                                                    .filter(|t| t.column_id == col_id)
                                                    .and_then(|t| before_for(&board(), &id, &t));
                                                let next = move_issue(&board(), &id, &col_id, before.as_deref());
                                                board.set(next);
                                                on_move.call((id.clone(), col_id.clone(), before.clone()));
                                                announce.set(format!("Moved to {col_label}"));
                                            }
                                            dragging.set(None);
                                            drop_at.set(None);
                                        }
                                    },
                                    header { class: "kit-ib__colhead",
                                        h2 { id: "{base}-col-{ci}", class: "kit-ib__colname", "{col.label}" }
                                        span { class: "kit-ib__count", "{col.count}" }
                                        span { class: "kit-ib__tone", "data-tone": "{col.tone}", "{col.tone}" }
                                    }
                                    if is_loading {
                                        for i in 0..2 {
                                            span { key: "{i}", class: "kit-ib__ph kit-ib__ph--card" }
                                        }
                                    } else {
                                        ul { class: "kit-ib__cards",
                                            for card in col.cards.iter() {
                                                li { key: "{card.id}", class: if col.drop_index == Some(card.index) && (drag_now.is_some() || picked_id.is_some()) { "kit-ib__slot is-drop" } else { "kit-ib__slot" },
                                                    article { id: "{card_id(&base, &card.id)}", class: "kit-ib__card",
                                                        draggable: "true", tabindex: "0",
                                                        "aria-label": "{card.aria}",
                                                        "aria-pressed": if card.picked { "true" } else { "false" },
                                                        "data-dragging": if card.dragging { "true" } else { "false" },
                                                        ondragstart: {
                                                            let id = card.id.clone();
                                                            move |_| dragging.set(Some(id.clone()))
                                                        },
                                                        ondragend: move |_| {
                                                            dragging.set(None);
                                                            drop_at.set(None);
                                                        },
                                                        ondragover: {
                                                            let col_id = col.id.clone();
                                                            let index = card.index;
                                                            move |event: DragEvent| {
                                                                event.prevent_default();
                                                                event.stop_propagation();
                                                                drop_at.set(Some(Target { column_id: col_id.clone(), index }));
                                                            }
                                                        },
                                                        onclick: {
                                                            let id = card.id.clone();
                                                            move |_| {
                                                                open_id.set(Some(id.clone()));
                                                                on_open.call(id.clone());
                                                            }
                                                        },
                                                        onkeydown: {
                                                            let id = card.id.clone();
                                                            let col_id = col.id.clone();
                                                            let cols = columns.clone();
                                                            move |event: KeyboardEvent| {
                                                                let snapshot = board();
                                                                handle_card_key(event, &id, &col_id, &cols, &snapshot, &mut board, &mut moving, &mut announce, &mut open_id, on_move, on_open);
                                                            }
                                                        },
                                                        span { class: "kit-ib__title-text", "{card.title}" }
                                                        if let Some(desc) = card.description.clone() {
                                                            span { class: "kit-ib__desc", "{desc}" }
                                                        }
                                                        span { class: "kit-ib__meta",
                                                            span { class: "kit-ib__prio",
                                                                span { class: "kit-ib__prio-mark", "aria-hidden": "true", "{card.priority_mark}" }
                                                                "{card.priority_word}"
                                                            }
                                                            span { class: "kit-ib__status", "data-status": "{card.status_key}", "aria-hidden": "true", "{card.status_mark}" }
                                                            span { "{card.status_word}" }
                                                            if let Some(counts) = card.counts.clone() {
                                                                span { class: "kit-ib__counts", "{counts}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            if col.drop_index == Some(col.cards.len()) && (drag_now.is_some() || picked_id.is_some()) {
                                                li { key: "drop-{col.id}", class: "kit-ib__dropline", "aria-hidden": "true" }
                                            }
                                        }
                                        button { class: "kit-ib__add", r#type: "button", "aria-label": "New issue in {col.label}",
                                            onclick: {
                                                let col_id = col.id.clone();
                                                move |_| new_draft.set(Some(Draft { title: String::new(), description: String::new(), column_id: col_id.clone() }))
                                            },
                                            "+ New issue"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(issue) = open_issue.clone() {
                    aside { id: "{panel_dom}", class: "kit-ib__panel", role: "dialog", "aria-modal": "false", "aria-label": "Issue: {issue.title}", tabindex: "-1",
                        onkeydown: {
                            let panel_dom = panel_dom.clone();
                            move |event: KeyboardEvent| {
                                if matches!(event.key(), Key::Tab) {
                                    event.prevent_default();
                                    let back = event.modifiers().shift();
                                    document::eval(&TRAP_TAB.replace("PANEL", &panel_dom).replace("BACK", if back { "true" } else { "false" }));
                                }
                            }
                        },
                        div { class: "kit-ib__panel-head",
                            button { class: "kit-ib__btn", r#type: "button",
                                onclick: {
                                    let id = issue.id.clone();
                                    let base_close = base.clone();
                                    move |_| {
                                        open_id.set(None);
                                        focus_after_frame(&card_id(&base_close, &id));
                                    }
                                },
                                "Close"
                            }
                        }
                        label { class: "kit-ib__field kit-ib__field--block",
                            span { "Title" }
                            input { class: "kit-ib__input", r#type: "text", value: "{issue.title}",
                                onchange: {
                                    let id = issue.id.clone();
                                    move |e: FormEvent| {
                                        let value = e.value();
                                        board.with_mut(|list| {
                                            if let Some(i) = list.iter_mut().find(|i| i.id == id) {
                                                i.title = value.clone();
                                            }
                                        });
                                        activity.with_mut(|log| log.push("Title changed".to_string()));
                                        on_update.call((id.clone(), IssuePatch { title: Some(value), ..Default::default() }));
                                    }
                                },
                            }
                        }
                        label { class: "kit-ib__field kit-ib__field--block",
                            span { "Description" }
                            textarea { class: "kit-ib__input", rows: "3", value: "{issue.description.clone().unwrap_or_default()}",
                                onchange: {
                                    let id = issue.id.clone();
                                    move |e: FormEvent| {
                                        let value = e.value();
                                        board.with_mut(|list| {
                                            if let Some(i) = list.iter_mut().find(|i| i.id == id) {
                                                i.description = Some(value.clone());
                                            }
                                        });
                                        activity.with_mut(|log| log.push("Description changed".to_string()));
                                        on_update.call((id.clone(), IssuePatch { description: Some(value), ..Default::default() }));
                                    }
                                },
                            }
                        }
                        label { class: "kit-ib__field kit-ib__field--block",
                            span { "Status" }
                            select { class: "kit-ib__select",
                                onchange: {
                                    let id = issue.id.clone();
                                    move |e: FormEvent| {
                                        let next = IssueStatus::from_key(&e.value());
                                        board.with_mut(|list| {
                                            if let Some(i) = list.iter_mut().find(|i| i.id == id) {
                                                i.issue_status = next;
                                            }
                                        });
                                        activity.with_mut(|log| log.push("Status changed".to_string()));
                                        on_update.call((id.clone(), IssuePatch { issue_status: next, ..Default::default() }));
                                    }
                                },
                                option { value: "", selected: issue.issue_status.is_none(), "No status" }
                                for s in IssueStatus::ALL {
                                    option { key: "{s.key()}", value: "{s.key()}", selected: issue.issue_status == Some(s), "{s.word()}" }
                                }
                            }
                        }
                        h3 { class: "kit-ib__activity-title", "Activity" }
                        ul { class: "kit-ib__activity",
                            for (i, line) in activity().iter().enumerate() {
                                li { key: "{i}", "{line}" }
                            }
                        }
                    }
                }

                if let Some(draft) = draft_now.clone() {
                    div { class: "kit-ib__scrim",
                        div { class: "kit-ib__dialog", role: "dialog", "aria-modal": "true", "aria-labelledby": "{base}-new",
                            onkeydown: move |event: KeyboardEvent| {
                                let mods = event.modifiers();
                                if matches!(event.key(), Key::Escape) {
                                    event.prevent_default();
                                    new_draft.set(None);
                                } else if matches!(event.key(), Key::Enter) && (mods.ctrl() || mods.meta()) {
                                    event.prevent_default();
                                    if let Some(d) = new_draft() {
                                        if !d.title.trim().is_empty() {
                                            on_create.call(d);
                                            new_draft.set(None);
                                            announce.set("Issue created".to_string());
                                        }
                                    }
                                }
                            },
                            h2 { id: "{base}-new", class: "kit-ib__dialog-title", "New issue" }
                            label { class: "kit-ib__field kit-ib__field--block",
                                span { "Title" }
                                input { class: "kit-ib__input", r#type: "text", value: "{draft.title}",
                                    oninput: move |e| new_draft.with_mut(|d| {
                                        if let Some(d) = d {
                                            d.title = e.value();
                                        }
                                    }),
                                }
                            }
                            label { class: "kit-ib__field kit-ib__field--block",
                                span { "Description" }
                                textarea { class: "kit-ib__input", rows: "3", value: "{draft.description}",
                                    oninput: move |e| new_draft.with_mut(|d| {
                                        if let Some(d) = d {
                                            d.description = e.value();
                                        }
                                    }),
                                }
                            }
                            label { class: "kit-ib__field kit-ib__field--block",
                                span { "Column" }
                                select { class: "kit-ib__select",
                                    onchange: move |e| new_draft.with_mut(|d| {
                                        if let Some(d) = d {
                                            d.column_id = e.value();
                                        }
                                    }),
                                    for c in columns.iter() {
                                        option { key: "{c.id}", value: "{c.id}", selected: c.id == draft.column_id, "{c.column_label}" }
                                    }
                                }
                            }
                            div { class: "kit-ib__dialog-actions",
                                button { class: "kit-ib__btn", r#type: "button", onclick: move |_| new_draft.set(None), "Cancel" }
                                button { class: "kit-ib__btn kit-ib__btn--primary", r#type: "button", disabled: draft.title.trim().is_empty(),
                                    onclick: move |_| {
                                        if let Some(d) = new_draft() {
                                            on_create.call(d);
                                            new_draft.set(None);
                                            announce.set("Issue created".to_string());
                                        }
                                    },
                                    "Create"
                                }
                            }
                        }
                    }
                }

                p { class: "kit-ib__note", "{total} issues" }
            }
            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}

/// Keyboard move on a card: Space picks it up, arrows move the target, Space or Enter drops, Escape cancels.
/// Enter on a card that is not being moved opens its issue.
#[allow(clippy::too_many_arguments)]
fn handle_card_key(
    event: KeyboardEvent,
    issue_id: &str,
    column_id: &str,
    columns: &[BoardColumn],
    snapshot: &[Issue],
    board: &mut Signal<Vec<Issue>>,
    moving: &mut Signal<Option<(String, Target)>>,
    announce: &mut Signal<String>,
    open_id: &mut Signal<Option<String>>,
    on_move: EventHandler<(String, String, Option<String>)>,
    on_open: EventHandler<String>,
) {
    let picked = moving();
    match event.key() {
        Key::Escape => {
            if picked.is_some() {
                event.prevent_default();
                moving.set(None);
                announce.set("Move cancelled".to_string());
            }
        }
        Key::Enter if picked.is_none() => {
            event.prevent_default();
            open_id.set(Some(issue_id.to_string()));
            on_open.call(issue_id.to_string());
        }
        Key::Character(ref c) if c == " " => {
            event.prevent_default();
            match picked {
                None => {
                    let index = column_cards(snapshot, column_id)
                        .iter()
                        .position(|i| i.id == issue_id)
                        .unwrap_or(0);
                    moving.set(Some((
                        issue_id.to_string(),
                        Target {
                            column_id: column_id.to_string(),
                            index,
                        },
                    )));
                    announce.set("Picked up. Arrow keys move it, Space drops it.".to_string());
                }
                Some((id, target)) => {
                    let before = before_for(snapshot, &id, &target);
                    let next = move_issue(snapshot, &id, &target.column_id, before.as_deref());
                    let label = columns
                        .iter()
                        .find(|c| c.id == target.column_id)
                        .map(|c| c.column_label.clone())
                        .unwrap_or_default();
                    board.set(next);
                    moving.set(None);
                    announce.set(format!("Moved to {label}, position {}", target.index + 1));
                    on_move.call((id, target.column_id.clone(), before));
                }
            }
        }
        Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown => {
            if let Some((id, target)) = picked {
                event.prevent_default();
                let (dx, dy) = match event.key() {
                    Key::ArrowLeft => (-1, 0),
                    Key::ArrowRight => (1, 0),
                    Key::ArrowUp => (0, -1),
                    _ => (0, 1),
                };
                let next = step_target(columns, snapshot, &target, dx, dy);
                moving.set(Some((id, next)));
            }
        }
        _ => {}
    }
}
