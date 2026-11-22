//! Decision queue: the questions an orchestrating agent is waiting on, answered one at a time from the keyboard so
//! the agent can keep going. Open and Done lists, a reading column for the answer, and the task beside it.
//!
//! Design choice: the answer area follows `request_kind`: option cards for decision and pick, approve or decline for
//! approval, a text field with suggestions for question. Enter sends whichever answer is ready.
//! Design choice: shortcuts are handled on the section; fields stop key propagation, so typing never triggers them.
//! Design choice: a sent item leaves the open list at once; the spec's confirmation mark is shown for a short beat.
//! Design choice: artifacts render through the shared `SheetGrid` for sheets, editable; documents are paragraphs with
//! marks (bold, italic, underline, link, checklist), edited in place; folder entries rename, add and remove. Each
//! edit is reported through `on_artifact_change`.
//! Design choice: a document is paragraphs of runs. Ctrl/Cmd+B, I and U toggle marks on the selection, Ctrl/Cmd+K
//! links it (http or https only), Ctrl/Cmd+Shift+L toggles a checklist item, Enter splits. `- `, `* `, `1. `, `[ ] `
//! and `# ` at the start of a line set the paragraph kind. The model is the source of truth: the DOM is read back
//! into runs after each edit and re-rendered from runs after each structural change.
//! Design choice: the "+" tab suggests stored artifacts that are not open; Delete or Backspace closes a tab.
//! Gap: folder entries have arrow movement but no Enter-to-open or type-ahead; they are not files to open.
//! Design choice: the sidebar is drawn inline so the screen stands alone, without the workspace frame.

use std::collections::BTreeMap;

use dioxus::prelude::*;

use crate::components::sheet_grid::{Column, ColumnKind, GridRow, SheetGrid};
use crate::internal::*;

const CSS: Asset = asset!("/assets/decision-queue.css");

/// How long the confirmation mark shows after a send, in milliseconds.
pub const CONFIRM_MS: u32 = 450;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestKind {
    Decision,
    Pick,
    Approval,
    Question,
}

impl RequestKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Decision => "Decision",
            Self::Pick => "Pick",
            Self::Approval => "Approval",
            Self::Question => "Question",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RequestOption {
    pub id: String,
    pub entry_title: String,
    pub summary: Option<String>,
    pub is_recommended: bool,
    pub advantages: Vec<String>,
    pub drawbacks: Vec<String>,
    pub preview_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fact {
    pub entry_label: String,
    pub entry_value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnswerRecord {
    pub answer_text: String,
    pub answered_at: String,
    pub entry_label: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub id: String,
    pub request_kind: RequestKind,
    pub question_text: String,
    pub context_text: Option<String>,
    pub task_name: String,
    pub asked_at: String,
    pub is_unread: bool,
    pub option_list: Vec<RequestOption>,
    pub fact_rows: Vec<Fact>,
    pub suggested_answers: Vec<String>,
    pub answer_record: Option<AnswerRecord>,
}

/// What the person sent: an option, typed text, or a decision on an approval.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnswerValue {
    Option { option_id: String },
    Text { free_text: String },
    Approve,
    Decline,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TaskStep {
    pub label: String,
    pub done: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TaskDetail {
    pub goal: Option<String>,
    pub project_name: Option<String>,
    pub task_state: Option<String>,
    pub step_list: Vec<TaskStep>,
    pub artifact_names: Vec<String>,
}

/// What a paragraph is. Line starts set it: `- ` or `* ` a bullet, `1. ` numbered, `[ ] ` a checklist item, `# ` a heading.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ParagraphKind {
    #[default]
    Plain,
    Bullet,
    Numbered,
    Checklist {
        done: bool,
    },
    Heading,
}

/// A stretch of text that shares one set of inline marks.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Run {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    /// An `http://` or `https://` address the text links to.
    pub link: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Paragraph {
    pub kind: ParagraphKind,
    pub runs: Vec<Run>,
}

/// An inline mark a selection toggles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Bold,
    Italic,
    Underline,
}

impl Paragraph {
    /// A plain paragraph holding one unmarked run.
    pub fn plain(text: &str) -> Self {
        Self {
            kind: ParagraphKind::Plain,
            runs: vec![Run {
                text: text.to_string(),
                ..Run::default()
            }],
        }
    }

    pub fn text(&self) -> String {
        paragraph_text(&self.runs)
    }
}

/// The paragraph's characters, without marks.
pub fn paragraph_text(runs: &[Run]) -> String {
    runs.iter().map(|run| run.text.as_str()).collect()
}

fn run_chars(runs: &[Run]) -> usize {
    runs.iter().map(|run| run.text.chars().count()).sum()
}

fn mark_on(run: &Run, mark: Mark) -> bool {
    match mark {
        Mark::Bold => run.bold,
        Mark::Italic => run.italic,
        Mark::Underline => run.underline,
    }
}

fn set_mark(run: &mut Run, mark: Mark, on: bool) {
    match mark {
        Mark::Bold => run.bold = on,
        Mark::Italic => run.italic = on,
        Mark::Underline => run.underline = on,
    }
}

fn same_marks(a: &Run, b: &Run) -> bool {
    a.bold == b.bold && a.italic == b.italic && a.underline == b.underline && a.link == b.link
}

/// Drops empty runs and joins neighbours that share their marks, so each text has one canonical run split.
pub fn normalise_runs(runs: &[Run]) -> Vec<Run> {
    let mut out: Vec<Run> = Vec::new();
    for run in runs.iter().filter(|run| !run.text.is_empty()) {
        match out.last_mut() {
            Some(last) if same_marks(last, run) => last.text.push_str(&run.text),
            _ => out.push(run.clone()),
        }
    }
    out
}

/// Splits runs at character offset `at`: the first part holds the first `at` characters.
pub fn split_runs(runs: &[Run], at: usize) -> (Vec<Run>, Vec<Run>) {
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut seen = 0;
    for run in runs {
        let start = seen;
        seen += run.text.chars().count();
        if seen <= at {
            left.push(run.clone());
        } else if start >= at {
            right.push(run.clone());
        } else {
            let cut = at - start;
            left.push(Run {
                text: run.text.chars().take(cut).collect(),
                ..run.clone()
            });
            right.push(Run {
                text: run.text.chars().skip(cut).collect(),
                ..run.clone()
            });
        }
    }
    (normalise_runs(&left), normalise_runs(&right))
}

/// Toggles a mark over `start..end`: off when every character already has it, on otherwise.
pub fn toggle_mark(runs: &[Run], start: usize, end: usize, mark: Mark) -> Vec<Run> {
    let end_clamped = end.min(run_chars(runs));
    if start >= end_clamped {
        return normalise_runs(runs);
    }
    let (left, rest) = split_runs(runs, start);
    let (mut mid, right) = split_runs(&rest, end_clamped - start);
    let on = !mid.iter().all(|run| mark_on(run, mark));
    for run in mid.iter_mut() {
        set_mark(run, mark, on);
    }
    normalise_runs(&[left, mid, right].concat())
}

/// Sets (or with `None`, clears) the link over `start..end`.
pub fn set_link(runs: &[Run], start: usize, end: usize, link: Option<String>) -> Vec<Run> {
    let end = end.min(run_chars(runs));
    if start >= end {
        return normalise_runs(runs);
    }
    let (left, rest) = split_runs(runs, start);
    let (mut mid, right) = split_runs(&rest, end - start);
    for run in mid.iter_mut() {
        run.link = link.clone();
    }
    normalise_runs(&[left, mid, right].concat())
}

/// A line start typed at the front of a plain paragraph: the kind it becomes and how many characters it removes.
pub fn line_start(paragraph: &Paragraph) -> Option<(ParagraphKind, usize)> {
    if paragraph.kind != ParagraphKind::Plain {
        return None;
    }
    let text = paragraph.text();
    if text.starts_with("- ") || text.starts_with("* ") {
        return Some((ParagraphKind::Bullet, 2));
    }
    if text.starts_with("# ") {
        return Some((ParagraphKind::Heading, 2));
    }
    if text.starts_with("[ ] ") {
        return Some((ParagraphKind::Checklist { done: false }, 4));
    }
    let digits = text.chars().take_while(|c| c.is_ascii_digit()).count();
    let marker: String = text.chars().skip(digits).take(2).collect();
    (digits > 0 && marker == ". ").then_some((ParagraphKind::Numbered, digits + 2))
}

/// Splits a paragraph at character offset `at` for Enter. The new paragraph keeps a list or checklist kind.
pub fn split_paragraph(paragraph: &Paragraph, at: usize) -> (Paragraph, Paragraph) {
    let (left, right) = split_runs(&paragraph.runs, at);
    let next_kind = match paragraph.kind {
        ParagraphKind::Bullet => ParagraphKind::Bullet,
        ParagraphKind::Numbered => ParagraphKind::Numbered,
        ParagraphKind::Checklist { .. } => ParagraphKind::Checklist { done: false },
        ParagraphKind::Plain | ParagraphKind::Heading => ParagraphKind::Plain,
    };
    (
        Paragraph {
            kind: paragraph.kind,
            runs: left,
        },
        Paragraph {
            kind: next_kind,
            runs: right,
        },
    )
}

/// The number each paragraph shows: consecutive numbered paragraphs count from 1, others are 0.
pub fn numbers(paragraphs: &[Paragraph]) -> Vec<usize> {
    let mut count = 0;
    paragraphs
        .iter()
        .map(|p| {
            if p.kind == ParagraphKind::Numbered {
                count += 1;
                count
            } else {
                count = 0;
                0
            }
        })
        .collect()
}

/// Escapes the five characters that matter in markup text and attribute values.
pub fn html_escape(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            other => other.to_string(),
        })
        .collect()
}

fn flag(on: bool) -> &'static str {
    if on { "1" } else { "0" }
}

/// Markup for a paragraph's runs, one `span` per run with its marks as data attributes. Built from the model with
/// every value escaped; the DOM is never read back into markup, only into runs.
pub fn runs_html(runs: &[Run]) -> String {
    runs.iter()
        .map(|run| {
            let href = run
                .link
                .as_deref()
                .map(|link| format!(" data-href=\"{}\"", html_escape(link)))
                .unwrap_or_default();
            format!(
                "<span data-run data-b=\"{}\" data-i=\"{}\" data-u=\"{}\"{href}>{}</span>",
                flag(run.bold),
                flag(run.italic),
                flag(run.underline),
                html_escape(&run.text)
            )
        })
        .collect()
}

/// Reads the message the DOM sends back: the selection's start and end in characters, then one run per field group,
/// each as `flags US text US link` with the groups separated by RS.
pub fn parse_readback(message: &str) -> Option<(usize, usize, Vec<Run>)> {
    let mut parts = message.split('\u{1e}');
    let start = parts.next()?.parse().ok()?;
    let end = parts.next()?.parse().ok()?;
    let mut runs = Vec::new();
    for part in parts {
        let mut fields = part.splitn(3, '\u{1f}');
        let flags = fields.next()?;
        let text = fields.next()?;
        let link = fields.next().unwrap_or("");
        runs.push(Run {
            text: text.to_string(),
            bold: flags.contains('b'),
            italic: flags.contains('i'),
            underline: flags.contains('u'),
            link: (!link.is_empty()).then(|| link.to_string()),
        });
    }
    Some((start, end, normalise_runs(&runs)))
}

/// A link address the selection may carry: `http://` or `https://` only, so no script URL reaches an anchor.
pub fn normalise_link(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let web = trimmed.starts_with("https://") || trimmed.starts_with("http://");
    (web && trimmed.len() > "https://".len()).then(|| trimmed.to_string())
}

#[derive(Clone, Debug, PartialEq)]
pub enum Artifact {
    Document {
        paragraphs: Vec<Paragraph>,
    },
    Sheet {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Folder {
        items: Vec<String>,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Open,
    Done,
}

/// The item after `current` in `len` items, moving by `step` (+1 or -1), clamped at both ends.
pub fn step_index(len: usize, current: usize, step: isize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let next = current as isize + step;
    Some(next.clamp(0, len as isize - 1) as usize)
}

/// Moves an id to the end of the open order (Later).
pub fn move_to_end(order: &[String], id: &str) -> Vec<String> {
    let mut next: Vec<String> = order.iter().filter(|x| x.as_str() != id).cloned().collect();
    if order.iter().any(|x| x == id) {
        next.push(id.to_string());
    }
    next
}

/// Moves an id to the top of the open order (Reopen).
pub fn move_to_top(order: &[String], id: &str) -> Vec<String> {
    let mut next: Vec<String> = vec![id.to_string()];
    next.extend(order.iter().filter(|x| x.as_str() != id).cloned());
    next
}

/// The option a number key picks: `1` is the first option. None when the key is out of range.
pub fn option_for_digit(digit: char, count: usize) -> Option<usize> {
    let n = digit.to_digit(10)? as usize;
    (n >= 1 && n <= count).then(|| n - 1)
}

/// The answer a request sends for the chosen option, typed text, or approval choice.
pub fn answer_for(
    kind: RequestKind,
    option_id: Option<&str>,
    text: &str,
    approve: Option<bool>,
) -> Option<AnswerValue> {
    match kind {
        RequestKind::Decision | RequestKind::Pick => option_id.map(|id| AnswerValue::Option {
            option_id: id.to_string(),
        }),
        RequestKind::Approval => approve.map(|ok| {
            if ok {
                AnswerValue::Approve
            } else {
                AnswerValue::Decline
            }
        }),
        RequestKind::Question => (!text.trim().is_empty()).then(|| AnswerValue::Text {
            free_text: text.trim().to_string(),
        }),
    }
}

/// A row's full accessible name: kind, question, task, age and unread state.
pub fn row_label(r: &Request) -> String {
    let unread = if r.is_unread { ", unread" } else { "" };
    format!(
        "{}, {}, {}, {}{unread}",
        r.request_kind.label(),
        r.question_text,
        r.task_name,
        r.asked_at
    )
}

fn sheet_from(columns: &[String], rows: &[Vec<String>]) -> (Vec<Column>, Vec<GridRow>) {
    let cols: Vec<Column> = columns
        .iter()
        .enumerate()
        .map(|(i, label)| Column::new(&format!("c{i}"), label, ColumnKind::Text))
        .collect();
    let grid: Vec<GridRow> = rows
        .iter()
        .enumerate()
        .map(|(r, cells)| GridRow {
            row_id: format!("r{r}"),
            row_logo: None,
            values: cells
                .iter()
                .enumerate()
                .map(|(c, v)| (format!("c{c}"), v.clone()))
                .collect::<BTreeMap<_, _>>(),
        })
        .collect();
    (cols, grid)
}

/// The decision queue screen: list, reading column and task panel, with keyboard answers.
#[component]
pub fn DecisionQueue(
    #[props(default = "Juniper Works".to_string(), into)] workspace_name: String,
    #[props(default = "Ada Lovelace".to_string(), into)] account_name: String,
    #[props(default)] request_list: Vec<Request>,
    #[props(default)] task_details: BTreeMap<String, TaskDetail>,
    #[props(default)] artifact_store: BTreeMap<String, Artifact>,
    #[props(default)] initial_tab: Tab,
    #[props(default)] initial_request_id: Option<String>,
    #[props(default)] running_count: usize,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_answer: EventHandler<(String, AnswerValue)>,
    #[props(default)] on_later: EventHandler<String>,
    #[props(default)] on_reopen: EventHandler<String>,
    #[props(default)] on_select: EventHandler<Option<String>>,
    #[props(default)] on_navigate: EventHandler<String>,
    #[props(default)] on_artifact_change: EventHandler<(String, Artifact)>,
) -> Element {
    let base = use_hook(|| format!("kit-dq-{}", next_id()));
    let mut tab = use_signal(move || initial_tab);
    let mut order = use_signal({
        let requests = request_list.clone();
        move || {
            requests
                .iter()
                .filter(|r| r.answer_record.is_none())
                .map(|r| r.id.clone())
                .collect::<Vec<String>>()
        }
    });
    let mut answered = use_signal({
        let requests = request_list.clone();
        move || {
            requests
                .iter()
                .filter_map(|r| r.answer_record.clone().map(|a| (r.id.clone(), a)))
                .collect::<BTreeMap<String, AnswerRecord>>()
        }
    });
    let mut selected = use_signal({
        let first = order_first(&request_list, initial_request_id.as_deref());
        move || first
    });
    let mut picked = use_signal(|| None::<usize>);
    let mut reply = use_signal(String::new);
    let mut sending = use_signal(|| None::<String>);
    let mut artifact = use_signal(|| None::<String>);
    let mut opened = use_signal(BTreeMap::<String, Vec<String>>::new);
    let mut edited = use_signal(BTreeMap::<String, Artifact>::new);
    let mut art_menu = use_signal(|| false);
    let mut announce = use_signal(String::new);
    let mut mobile_detail = use_signal(|| false);

    let all: Vec<Request> = request_list.clone();
    let open_order = order();
    let answered_now = answered();
    let current_tab = tab();
    let done_ids: Vec<String> = all
        .iter()
        .filter(|r| answered_now.contains_key(&r.id))
        .map(|r| r.id.clone())
        .collect();
    let list_ids: Vec<String> = match current_tab {
        Tab::Open => open_order.clone(),
        Tab::Done => done_ids.clone(),
    };
    let selected_id = selected()
        .filter(|id| list_ids.contains(id))
        .or_else(|| list_ids.first().cloned());
    let selected_request = selected_id
        .as_ref()
        .and_then(|id| all.iter().find(|r| &r.id == id).cloned());
    let open_count = open_order.len();
    let caught_up = current_tab == Tab::Open && open_count == 0 && !is_empty && !is_loading;
    let detail_class = if mobile_detail() {
        "kit-dq__page is-detail"
    } else {
        "kit-dq__page"
    };
    let task_detail = selected_request
        .as_ref()
        .and_then(|r| task_details.get(&r.task_name).cloned())
        .unwrap_or_default();
    let task_key = selected_request
        .as_ref()
        .map(|r| r.task_name.clone())
        .unwrap_or_default();
    let open_names: Vec<String> = opened()
        .get(&task_key)
        .cloned()
        .unwrap_or_else(|| task_detail.artifact_names.clone());
    let active_artifact = artifact()
        .filter(|a| open_names.contains(a))
        .or_else(|| open_names.first().cloned());
    let suggested: Vec<String> = artifact_store
        .keys()
        .filter(|name| !open_names.contains(name))
        .cloned()
        .collect();
    let edited_now = edited();
    let sent_mark = sending();

    let tab_base = base.clone();
    let names_for_tabs = open_names.clone();
    let task_for_tabs = task_key.clone();
    let tab_keys = move |event: KeyboardEvent, index: usize| {
        event.stop_propagation();
        let plus = index == names_for_tabs.len();
        match event.key() {
            Key::ArrowRight => {
                event.prevent_default();
                let next = (index + 1).min(names_for_tabs.len());
                focus_after_frame(&artifact_tab_id(&tab_base, next, names_for_tabs.len()));
            }
            Key::ArrowLeft => {
                event.prevent_default();
                focus_after_frame(&artifact_tab_id(
                    &tab_base,
                    index.saturating_sub(1),
                    names_for_tabs.len(),
                ));
            }
            Key::Escape if plus => {
                art_menu.set(false);
            }
            Key::Delete | Key::Backspace if !plus => {
                event.prevent_default();
                let mut next = names_for_tabs.clone();
                let closed = next.remove(index);
                let focus_index = index.min(next.len().saturating_sub(1));
                let neighbour = next.get(focus_index).cloned();
                opened.with_mut(|m| {
                    m.insert(task_for_tabs.clone(), next.clone());
                });
                artifact.set(neighbour);
                announce.set(format!("Closed {closed}"));
                focus_after_frame(&artifact_tab_id(&tab_base, focus_index, next.len()));
            }
            _ => {}
        }
    };

    let all_for_keys = all.clone();
    let base_for_keys = base.clone();
    let on_section_keys = move |event: KeyboardEvent| {
        let all = &all_for_keys;
        let base = &base_for_keys;
        let order_now = order();
        let answered_map = answered();
        let current_tab = tab();
        let list = match current_tab {
            Tab::Open => order_now.clone(),
            Tab::Done => all
                .iter()
                .filter(|r| answered_map.contains_key(&r.id))
                .map(|r| r.id.clone())
                .collect(),
        };
        let current = selected()
            .and_then(|id| list.iter().position(|x| *x == id))
            .unwrap_or(0);
        let current_request = list
            .get(current)
            .and_then(|id| all.iter().find(|r| r.id == *id))
            .cloned();
        let key = event.key();
        match key {
            Key::Character(ref c) if c == "j" || c == "J" => {
                event.prevent_default();
                if let Some(i) = step_index(list.len(), current, 1) {
                    selected.set(list.get(i).cloned());
                    picked.set(None);
                }
            }
            Key::Character(ref c) if c == "k" || c == "K" => {
                event.prevent_default();
                if let Some(i) = step_index(list.len(), current, -1) {
                    selected.set(list.get(i).cloned());
                    picked.set(None);
                }
            }
            Key::ArrowDown => {
                event.prevent_default();
                if let Some(i) = step_index(list.len(), current, 1) {
                    selected.set(list.get(i).cloned());
                    picked.set(None);
                }
            }
            Key::ArrowUp => {
                event.prevent_default();
                if let Some(i) = step_index(list.len(), current, -1) {
                    selected.set(list.get(i).cloned());
                    picked.set(None);
                }
            }
            Key::ArrowRight if current_tab == Tab::Open => {
                event.prevent_default();
                tab.set(Tab::Done);
                selected.set(None);
            }
            Key::ArrowLeft if current_tab == Tab::Done => {
                event.prevent_default();
                tab.set(Tab::Open);
                selected.set(None);
            }
            Key::Character(ref c) if c == "l" || c == "L" => {
                event.prevent_default();
                if let Some(req) = current_request.clone() {
                    if current_tab == Tab::Open {
                        let next_order = move_to_end(&order_now, &req.id);
                        let next_selected = next_order.first().cloned();
                        order.set(next_order);
                        selected.set(next_selected.clone());
                        picked.set(None);
                        announce.set(format!(
                            "Later. Next: {}",
                            next_selected.as_deref().unwrap_or("none")
                        ));
                        on_later.call(req.id.clone());
                    }
                }
            }
            Key::Character(ref c) if c == "r" || c == "R" || c == "/" => {
                event.prevent_default();
                focus_after_frame(&format!("{base}-reply"));
            }
            Key::Character(ref c) if c.len() == 1 && c.chars().all(|ch| ch.is_ascii_digit()) => {
                if let (Some(req), Some(ch)) = (current_request.clone(), c.chars().next()) {
                    if let Some(i) = option_for_digit(ch, req.option_list.len()) {
                        event.prevent_default();
                        picked.set(Some(i));
                    }
                }
            }
            Key::Enter => {
                if let Some(req) = current_request.clone() {
                    if current_tab == Tab::Open {
                        event.prevent_default();
                        let option_id = picked()
                            .and_then(|i| req.option_list.get(i))
                            .map(|o| o.id.clone());
                        let answer = match req.request_kind {
                            RequestKind::Approval => Some(AnswerValue::Approve),
                            kind => answer_for(kind, option_id.as_deref(), &reply(), None),
                        };
                        if let Some(value) = answer {
                            send(
                                &req.id,
                                value,
                                &mut sending,
                                &mut announce,
                                &mut reply,
                                &mut picked,
                                &mut mobile_detail,
                                on_answer,
                                order,
                                answered,
                                selected,
                                base.clone(),
                            );
                        }
                    }
                }
            }
            Key::Escape => {
                if mobile_detail() {
                    mobile_detail.set(false);
                }
            }
            _ => {}
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "kit-dq", "aria-labelledby": "{base}-title", onkeydown: on_section_keys,
            nav { class: "kit-dq__nav", "aria-label": "Sidebar",
                p { class: "kit-dq__ws", "{workspace_name}" }
                ul { class: "kit-dq__nav-list",
                    for item in ["Home", "Inbox", "Thread", "Settings"] {
                        li { key: "{item}",
                            button { class: "kit-dq__nav-item", r#type: "button",
                                "aria-current": if item == "Inbox" { "page" } else { "false" },
                                onclick: move |_| on_navigate.call(item.to_string()),
                                "{item}"
                            }
                        }
                    }
                }
                p { class: "kit-dq__account", "{account_name}" }
            }

            div { class: "{detail_class}",
                header { class: "kit-dq__head",
                    h1 { id: "{base}-title", class: "kit-dq__title", "Decisions" }
                    button { class: "kit-dq__back", r#type: "button", onclick: move |_| mobile_detail.set(false), "Back to list" }
                }

                div { class: "kit-dq__columns",
                    section { class: "kit-dq__list-col", "aria-labelledby": "{base}-list-title",
                        h2 { id: "{base}-list-title", class: "kit-dq__sr", "Requests" }
                        div { class: "kit-dq__tabs", role: "tablist", "aria-label": "Requests",
                            button { class: "kit-dq__tab", r#type: "button", role: "tab",
                                "aria-selected": if current_tab == Tab::Open { "true" } else { "false" },
                                onclick: move |_| {
                                    tab.set(Tab::Open);
                                    selected.set(None);
                                },
                                "Open "
                                span { class: "kit-dq__count", "{open_count}" }
                            }
                            button { class: "kit-dq__tab", r#type: "button", role: "tab",
                                "aria-selected": if current_tab == Tab::Done { "true" } else { "false" },
                                onclick: move |_| {
                                    tab.set(Tab::Done);
                                    selected.set(None);
                                },
                                "Done "
                                span { class: "kit-dq__count", "{done_ids.len()}" }
                            }
                        }
                        if is_loading {
                            for i in 0..4 {
                                span { key: "{i}", class: "kit-dq__ph kit-dq__ph--row" }
                            }
                        } else {
                            ul { class: "kit-dq__rows", role: "listbox", "aria-label": "{current_tab:?} requests",
                                for id in list_ids.iter() {
                                    {
                                        let req = all.iter().find(|r| &r.id == id).cloned();
                                        let is_sel = selected_id.as_deref() == Some(id.as_str());
                                        match req {
                                            Some(r) => rsx! {
                                                li { key: "{r.id}", class: "kit-dq__row", role: "option", "aria-selected": if is_sel { "true" } else { "false" },
                                                    "aria-label": "{row_label(&r)}",
                                                    tabindex: if is_sel { "0" } else { "-1" },
                                                    onclick: {
                                                        let id = r.id.clone();
                                                        move |_| {
                                                            selected.set(Some(id.clone()));
                                                            picked.set(None);
                                                            mobile_detail.set(true);
                                                            on_select.call(Some(id.clone()));
                                                        }
                                                    },
                                                    span { class: "kit-dq__kind", "{r.request_kind.label()}" }
                                                    span { class: "kit-dq__q", "{r.question_text}" }
                                                    span { class: "kit-dq__meta", "{r.task_name} · {r.asked_at}" }
                                                    if r.is_unread {
                                                        span { class: "kit-dq__unread", "aria-hidden": "true" }
                                                    }
                                                }
                                            },
                                            None => rsx! {},
                                        }
                                    }
                                }
                            }
                        }
                    }

                    section { class: "kit-dq__reading", "aria-labelledby": "{base}-question",
                        if is_empty {
                            div { class: "kit-dq__caught",
                                p { "Nothing is waiting for you." }
                            }
                        } else if is_loading {
                            span { class: "kit-dq__ph kit-dq__ph--line" }
                            span { class: "kit-dq__ph kit-dq__ph--block" }
                        } else if caught_up {
                            div { class: "kit-dq__caught",
                                p { "You're all caught up." }
                                button { class: "kit-dq__btn", r#type: "button", onclick: move |_| on_navigate.call("Thread".to_string()), "Go to Thread" }
                            }
                        } else if let Some(req) = selected_request.clone() {
                            {
                                let is_answered = answered_now.get(&req.id).cloned();
                                let answered_flag = is_answered.is_some();
                                let send_id = req.id.clone();
                                let send_base = base.clone();
                                rsx! {
                                    if let Some(ctx) = req.context_text.clone() {
                                        p { class: "kit-dq__context", "{ctx}" }
                                    }
                                    h2 { id: "{base}-question", class: "kit-dq__question", tabindex: "-1", "{req.question_text}" }
                                    if let Some(record) = is_answered.clone() {
                                        div { class: "kit-dq__answered", role: "status", onkeydown: stop_enter,
                                            p { "Answered {record.answered_at}: {record.answer_text}" }
                                            button { class: "kit-dq__btn", r#type: "button",
                                                onclick: {
                                                    let id = req.id.clone();
                                                    move |_| {
                                                        answered.with_mut(|m| { m.remove(&id); });
                                                        order.set(move_to_top(&order(), &id));
                                                        tab.set(Tab::Open);
                                                        selected.set(Some(id.clone()));
                                                        on_reopen.call(id.clone());
                                                        announce.set("Reopened".to_string());
                                                    }
                                                },
                                                "Reopen"
                                            }
                                        }
                                    }
                                    if !answered_flag {
                                        AnswerArea {
                                            request: req.clone(),
                                            picked,
                                            reply,
                                            base: base.clone(),
                                            on_send: move |value: AnswerValue| {
                                                send(&send_id, value, &mut sending, &mut announce, &mut reply, &mut picked, &mut mobile_detail, on_answer, order, answered, selected, send_base.clone());
                                            },
                                        }
                                        div { class: "kit-dq__actions", onkeydown: stop_enter,
                                            button { class: "kit-dq__btn", r#type: "button", "aria-keyshortcuts": "L",
                                                onclick: {
                                                    let id = req.id.clone();
                                                    move |_| {
                                                        let next_order = move_to_end(&order(), &id);
                                                        selected.set(next_order.first().cloned());
                                                        order.set(next_order);
                                                        picked.set(None);
                                                        on_later.call(id.clone());
                                                        announce.set("Later".to_string());
                                                    }
                                                },
                                                "Later"
                                            }
                                            if sent_mark.as_deref() == Some(req.id.as_str()) {
                                                span { class: "kit-dq__sent", role: "status", "✓ Sent" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    aside { class: "kit-dq__task", "aria-label": "Task",
                        if let Some(req) = selected_request.clone() {
                            h2 { class: "kit-dq__task-title", "{req.task_name}" }
                            if let Some(goal) = task_detail.goal.clone() {
                                p { class: "kit-dq__goal", "{goal}" }
                            }
                            if let Some(state) = task_detail.task_state.clone() {
                                p { class: "kit-dq__state", "State: {state}" }
                            }
                            if !task_detail.step_list.is_empty() {
                                ul { class: "kit-dq__steps",
                                    for (i, step) in task_detail.step_list.iter().enumerate() {
                                        li { key: "{i}", "data-done": if step.done { "true" } else { "false" },
                                            span { class: "kit-dq__tick", "aria-hidden": "true", if step.done { "✓" } else { "·" } }
                                            span { class: "kit-dq__sr", if step.done { "Done: " } else { "To do: " } }
                                            "{step.label}"
                                        }
                                    }
                                }
                            }
                            div { class: "kit-dq__tabs", role: "tablist", "aria-label": "Artifacts",
                                for (i, name) in open_names.iter().enumerate() {
                                    button { key: "{name}", id: "{base}-art-{i}", class: "kit-dq__tab", r#type: "button", role: "tab",
                                        "aria-selected": if active_artifact.as_deref() == Some(name.as_str()) { "true" } else { "false" },
                                        tabindex: if active_artifact.as_deref() == Some(name.as_str()) { "0" } else { "-1" },
                                        onclick: {
                                            let name = name.clone();
                                            move |_| {
                                                artifact.set(Some(name.clone()));
                                                art_menu.set(false);
                                            }
                                        },
                                        onkeydown: {
                                            let mut keys = tab_keys.clone();
                                            move |event: KeyboardEvent| keys(event, i)
                                        },
                                        "{name}"
                                    }
                                }
                                button { class: "kit-dq__tab kit-dq__tab--add", r#type: "button", role: "tab", id: "{base}-art-add",
                                    "aria-label": "Open an artifact", "aria-expanded": "{art_menu()}",
                                    tabindex: if active_artifact.is_none() { "0" } else { "-1" },
                                    onclick: move |_| art_menu.toggle(),
                                    onkeydown: {
                                        let mut keys = tab_keys.clone();
                                        let plus_index = open_names.len();
                                        move |event: KeyboardEvent| keys(event, plus_index)
                                    },
                                    "+"
                                }
                            }
                            if art_menu() {
                                if suggested.is_empty() {
                                    p { class: "kit-dq__note", "Every stored artifact is open." }
                                } else {
                                    div { class: "kit-dq__suggest", role: "menu", "aria-label": "Artifacts to open",
                                        onkeydown: {
                                            let menu_base = base.clone();
                                            move |event: KeyboardEvent| {
                                                if matches!(event.key(), Key::Escape) {
                                                    event.prevent_default();
                                                    art_menu.set(false);
                                                    focus_after_frame(&format!("{menu_base}-art-add"));
                                                }
                                            }
                                        },
                                        for name in suggested.iter() {
                                            button { key: "{name}", class: "kit-dq__suggest-item", r#type: "button", role: "menuitem",
                                                onclick: {
                                                    let name = name.clone();
                                                    let task = task_key.clone();
                                                    let names = open_names.clone();
                                                    let menu_base = base.clone();
                                                    move |_| {
                                                        let mut next = names.clone();
                                                        next.push(name.clone());
                                                        let at = next.len() - 1;
                                                        opened.with_mut(|m| {
                                                            m.insert(task.clone(), next.clone());
                                                        });
                                                        artifact.set(Some(name.clone()));
                                                        art_menu.set(false);
                                                        announce.set(format!("Opened {name}"));
                                                        focus_after_frame(&artifact_tab_id(&menu_base, at, next.len()));
                                                    }
                                                },
                                                "{name}"
                                            }
                                        }
                                    }
                                }
                            }
                            if let Some(name) = active_artifact.clone() {
                                if let Some(item) = edited_now.get(&name).or_else(|| artifact_store.get(&name)).cloned() {
                                    ArtifactView {
                                        key: "{name}",
                                        item,
                                        id_base: format!("{base}-art"),
                                        on_change: {
                                            let name = name.clone();
                                            move |next: Artifact| {
                                                edited.with_mut(|m| {
                                                    m.insert(name.clone(), next.clone());
                                                });
                                                on_artifact_change.call((name.clone(), next));
                                            }
                                        },
                                    }
                                }
                            }
                        } else {
                            p { class: "kit-dq__note", "Nothing is open." }
                        }
                    }
                }

                footer { class: "kit-dq__footer", role: "status", "aria-live": "polite",
                    if running_count > 0 {
                        span { "{running_count} tasks still running" }
                    } else {
                        span { "No tasks are running" }
                    }
                }
            }
            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}

fn order_first(requests: &[Request], initial: Option<&str>) -> Option<String> {
    if let Some(id) = initial {
        if requests
            .iter()
            .any(|r| r.id == id && r.answer_record.is_none())
        {
            return Some(id.to_string());
        }
    }
    requests
        .iter()
        .find(|r| r.answer_record.is_none())
        .map(|r| r.id.clone())
}

/// Records a send: the item leaves the open list, the next item is selected, and focus moves to its heading.
#[allow(clippy::too_many_arguments)]
fn send(
    id: &str,
    value: AnswerValue,
    sending: &mut Signal<Option<String>>,
    announce: &mut Signal<String>,
    reply: &mut Signal<String>,
    picked: &mut Signal<Option<usize>>,
    mobile_detail: &mut Signal<bool>,
    on_answer: EventHandler<(String, AnswerValue)>,
    mut order: Signal<Vec<String>>,
    mut answered: Signal<BTreeMap<String, AnswerRecord>>,
    mut selected: Signal<Option<String>>,
    base: String,
) {
    let remaining: Vec<String> = order().into_iter().filter(|x| x != id).collect();
    let next = remaining.first().cloned();
    let text = match &value {
        AnswerValue::Option { option_id } => format!("option {option_id}"),
        AnswerValue::Text { free_text } => free_text.clone(),
        AnswerValue::Approve => "Approved".to_string(),
        AnswerValue::Decline => "Declined".to_string(),
    };
    answered.with_mut(|m| {
        m.insert(
            id.to_string(),
            AnswerRecord {
                answer_text: text,
                answered_at: "now".to_string(),
                entry_label: None,
            },
        );
    });
    sending.set(Some(id.to_string()));
    on_answer.call((id.to_string(), value));
    order.set(remaining.clone());
    selected.set(next.clone());
    reply.set(String::new());
    picked.set(None);
    mobile_detail.set(false);
    announce.set(format!("Sent. {} left.", remaining.len()));
    let heading = format!("{base}-question");
    let mut sending = *sending;
    spawn(async move {
        sleep(CONFIRM_MS).await;
        sending.set(None);
    });
    if next.is_some() {
        focus_after_frame(&heading);
    }
}

/// The answer area for a request kind: option cards, approval buttons, or a text field with suggestions.
#[component]
fn AnswerArea(
    request: Request,
    mut picked: Signal<Option<usize>>,
    mut reply: Signal<String>,
    base: String,
    on_send: EventHandler<AnswerValue>,
) -> Element {
    let focus_index =
        picked().or_else(|| request.option_list.iter().position(|o| o.is_recommended));
    match request.request_kind {
        RequestKind::Decision | RequestKind::Pick => rsx! {
            div { class: "kit-dq__options", role: "radiogroup", "aria-label": "Options",
                onkeydown: {
                    let options = request.option_list.clone();
                    move |event: KeyboardEvent| {
                        let last = options.len().saturating_sub(1);
                        match event.key() {
                            Key::ArrowDown => {
                                event.prevent_default();
                                picked.set(Some(focus_index.map_or(0, |i| (i + 1).min(last))));
                            }
                            Key::ArrowUp => {
                                event.prevent_default();
                                picked.set(Some(focus_index.map_or(0, |i| i.saturating_sub(1))));
                            }
                            Key::Enter => {
                                event.stop_propagation();
                                event.prevent_default();
                                if let Some(o) = focus_index.and_then(|i| options.get(i)) {
                                    on_send.call(AnswerValue::Option { option_id: o.id.clone() });
                                }
                            }
                            _ => {}
                        }
                    }
                },
                for (i, option) in request.option_list.iter().enumerate() {
                    {
                        let checked = focus_index == Some(i);
                        let option_id = option.id.clone();
                        rsx! {
                            div { key: "{option.id}", class: "kit-dq__option", role: "radio", "aria-checked": if checked { "true" } else { "false" },
                                tabindex: if checked { "0" } else { "-1" },
                                "data-expanded": if checked { "true" } else { "false" },
                                onclick: move |_| picked.set(Some(i)),
                                ondoubleclick: {
                                    let option_id = option_id.clone();
                                    move |_| on_send.call(AnswerValue::Option { option_id: option_id.clone() })
                                },
                                span { class: "kit-dq__opt-key", "aria-hidden": "true", "{i + 1}" }
                                span { class: "kit-dq__opt-title", "{option.entry_title}" }
                                if option.is_recommended {
                                    span { class: "kit-dq__rec", "Recommended" }
                                }
                                if let Some(summary) = option.summary.clone() {
                                    span { class: "kit-dq__opt-sum", "{summary}" }
                                }
                                if checked {
                                    div { class: "kit-dq__pros",
                                        ul { for (j, a) in option.advantages.iter().enumerate() { li { key: "{j}", class: "kit-dq__pro", "+ {a}" } } }
                                        ul { for (j, d) in option.drawbacks.iter().enumerate() { li { key: "{j}", class: "kit-dq__con", "− {d}" } } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            button { class: "kit-dq__btn kit-dq__btn--primary", r#type: "button", "aria-keyshortcuts": "Enter", onkeydown: stop_enter,
                disabled: focus_index.is_none(),
                onclick: move |_| {
                    if let Some(i) = focus_index {
                        if let Some(o) = request.option_list.get(i) {
                            on_send.call(AnswerValue::Option { option_id: o.id.clone() });
                        }
                    }
                },
                "Send"
            }
        },
        RequestKind::Approval => rsx! {
            div { class: "kit-dq__facts", onkeydown: stop_enter,
                dl {
                    for (i, fact) in request.fact_rows.iter().enumerate() {
                        div { key: "{i}", class: "kit-dq__fact",
                            dt { "{fact.entry_label}" }
                            dd { "{fact.entry_value}" }
                        }
                    }
                }
            }
            div { class: "kit-dq__decide",
                button { class: "kit-dq__btn kit-dq__btn--primary", r#type: "button", onclick: move |_| on_send.call(AnswerValue::Approve), "Approve" }
                button { class: "kit-dq__btn", r#type: "button", onclick: move |_| on_send.call(AnswerValue::Decline), "Decline" }
            }
        },
        RequestKind::Question => rsx! {
            if !request.suggested_answers.is_empty() {
                div { class: "kit-dq__chips",
                    for (i, s) in request.suggested_answers.iter().enumerate() {
                        button { key: "{i}", class: "kit-dq__chip", r#type: "button", onclick: {
                            let s = s.clone();
                            move |_| reply.set(s.clone())
                        }, "{s}" }
                    }
                }
            }
            textarea { id: "{base}-reply", class: "kit-dq__reply", "aria-label": "Your answer", rows: "3", value: "{reply()}",
                oninput: move |e| reply.set(e.value()),
                onkeydown: move |event: KeyboardEvent| {
                    event.stop_propagation();
                    if matches!(event.key(), Key::Enter) && !event.modifiers().shift() {
                        event.prevent_default();
                        if let Some(value) = answer_for(RequestKind::Question, None, &reply(), None) {
                            on_send.call(value);
                        }
                    }
                    if matches!(event.key(), Key::Escape) {
                        focus_after_frame(&format!("{base}-question"));
                    }
                },
            }
        },
    }
}

/// Keeps Enter on a control from also reaching the section's shortcut handler, which would send a second time.
fn stop_enter(event: KeyboardEvent) {
    if matches!(event.key(), Key::Enter) {
        event.stop_propagation();
    }
}

/// Id of the artifact tab at `index`; an index past the artifact tabs is the "+" tab.
pub fn artifact_tab_id(base: &str, index: usize, count: usize) -> String {
    if index >= count {
        format!("{base}-art-add")
    } else {
        format!("{base}-art-{index}")
    }
}

/// Sheet rows read back from the grid, `width` cells per row, in the column order `sheet_from` wrote.
pub fn sheet_rows_back(width: usize, grid: &[GridRow]) -> Vec<Vec<String>> {
    grid.iter()
        .map(|row| {
            (0..width)
                .map(|c| {
                    row.values
                        .get(&format!("c{c}"))
                        .cloned()
                        .unwrap_or_default()
                })
                .collect()
        })
        .collect()
}

/// Reads a paragraph element back: the selection in characters, then its child nodes as runs. Text nodes and
/// unmarked elements read as plain runs; `data-run` spans read their marks from their data attributes.
fn readback_script(id: &str) -> String {
    format!(
        r#"(() => {{
const el = document.getElementById({id:?});
const clean = (s) => String(s).replace(/[\u001e\u001f]/g, " ");
const chars = (s) => [...s].length;
if (!el) {{ dioxus.send("0\u001e0"); return; }}
const offset = (node, at) => {{ const r = document.createRange(); r.selectNodeContents(el); r.setEnd(node, at); return chars(r.toString()); }};
let start = 0, end = 0;
const sel = window.getSelection();
if (sel && sel.rangeCount) {{
  const r = sel.getRangeAt(0);
  if (el.contains(r.startContainer) && el.contains(r.endContainer)) {{
    start = offset(r.startContainer, r.startOffset);
    end = offset(r.endContainer, r.endOffset);
  }}
}}
const parts = [start, end];
for (const node of el.childNodes) {{
  if (node.nodeType === 3) {{
    parts.push(["", clean(node.data), ""].join("\u001f"));
  }} else if (node.nodeType === 1) {{
    const d = node.dataset;
    if (d.run !== undefined) {{
      const flags = (d.b === "1" ? "b" : "") + (d.i === "1" ? "i" : "") + (d.u === "1" ? "u" : "");
      parts.push([flags, clean(node.textContent), clean(d.href || "")].join("\u001f"));
    }} else {{
      parts.push(["", clean(node.textContent), ""].join("\u001f"));
    }}
  }}
}}
dioxus.send(parts.join("\u001e"));
}})();"#
    )
}

/// Puts the caret, or the selection, back on the characters `start..end` of a paragraph after it re-rendered.
fn restore_script(id: &str, start: usize, end: usize) -> String {
    format!(
        r#"requestAnimationFrame(() => requestAnimationFrame(() => {{
const el = document.getElementById({id:?});
if (!el) return;
const at = (target) => {{
  const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
  let left = target;
  let node;
  while ((node = walker.nextNode())) {{
    const chars = [...node.data];
    if (left <= chars.length) return [node, chars.slice(0, left).join("").length];
    left -= chars.length;
  }}
  return [el, el.childNodes.length];
}};
const [sn, so] = at({start});
const [en, eo] = at({end});
const range = document.createRange();
range.setStart(sn, so);
range.setEnd(en, eo);
el.focus();
const sel = window.getSelection();
sel.removeAllRanges();
sel.addRange(range);
}}));"#
    )
}

/// The document's state, passed as one value: all of it is `Copy`.
#[derive(Clone, Copy)]
struct DocState {
    doc: Signal<Vec<Paragraph>>,
    snaps: Signal<Vec<String>>,
    on_change: EventHandler<Vec<Paragraph>>,
    link_at: Signal<Option<(usize, usize, usize)>>,
    link_draft: Signal<String>,
    /// Bumped by every structural commit; each row is keyed by it, so the DOM is rebuilt from the model.
    revision: Signal<usize>,
}

/// Commands a paragraph answers with the DOM selection: formatting, a link, a checklist toggle, or a split on Enter.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ParaAction {
    Mark(Mark),
    Link,
    Checklist,
    Split,
}

fn snapshots(paragraphs: &[Paragraph]) -> Vec<String> {
    paragraphs.iter().map(|p| runs_html(&p.runs)).collect()
}

/// Stores new paragraphs and rebuilds every row's DOM from them. Typing never goes through here, so the caret
/// survives it; a line start does, because its marker text has to leave the DOM.
fn commit_paragraphs(state: DocState, next: Vec<Paragraph>) {
    let mut doc = state.doc;
    let mut snaps = state.snaps;
    let mut revision = state.revision;
    snaps.set(snapshots(&next));
    revision.set(revision() + 1);
    doc.set(next.clone());
    state.on_change.call(next);
}

/// Reads the paragraph's runs from the DOM, then runs `action` on the selection in them.
fn run_paragraph_action(state: DocState, index: usize, id_base: String, action: ParaAction) {
    let id = format!("{id_base}-para-{index}");
    let script = readback_script(&id);
    spawn(async move {
        let mut events = document::eval(&script);
        let Ok(message) = events.recv::<String>().await else {
            return;
        };
        let Some((start, end, dom_runs)) = parse_readback(&message) else {
            return;
        };
        let mut doc = state.doc;
        let mut next = doc();
        let Some(para) = next.get_mut(index) else {
            return;
        };
        para.runs = dom_runs;
        match action {
            ParaAction::Mark(mark) => {
                para.runs = toggle_mark(&para.runs, start, end, mark);
                commit_paragraphs(state, next);
                document::eval(&restore_script(&id, start, end));
            }
            ParaAction::Link => {
                let mut link_at = state.link_at;
                let mut link_draft = state.link_draft;
                if start < end {
                    link_at.set(Some((index, start, end)));
                    link_draft.set("https://".to_string());
                    focus_after_frame(&format!("{id_base}-link"));
                }
                doc.set(next.clone());
                state.on_change.call(next);
            }
            ParaAction::Checklist => {
                para.kind = match para.kind {
                    ParagraphKind::Checklist { .. } => ParagraphKind::Plain,
                    _ => ParagraphKind::Checklist { done: false },
                };
                doc.set(next.clone());
                state.on_change.call(next);
            }
            ParaAction::Split => {
                let (left, right) = split_paragraph(para, start);
                *para = left;
                next.insert(index + 1, right);
                commit_paragraphs(state, next);
                document::eval(&restore_script(
                    &format!("{id_base}-para-{}", index + 1),
                    0,
                    0,
                ));
            }
        }
    });
}

/// Reads a paragraph after typing. A line start typed at its front becomes the paragraph's kind and its marker
/// characters go; the caret moves back by the same count.
fn sync_typed(state: DocState, index: usize, id_base: String) {
    let id = format!("{id_base}-para-{index}");
    let script = readback_script(&id);
    spawn(async move {
        let mut events = document::eval(&script);
        let Ok(message) = events.recv::<String>().await else {
            return;
        };
        let Some((caret, _, dom_runs)) = parse_readback(&message) else {
            return;
        };
        let mut doc = state.doc;
        let mut next = doc();
        let Some(para) = next.get_mut(index) else {
            return;
        };
        para.runs = dom_runs;
        if let Some((kind, removed)) = line_start(para) {
            para.kind = kind;
            para.runs = split_runs(&para.runs, removed).1;
            commit_paragraphs(state, next);
            let caret = caret.saturating_sub(removed);
            document::eval(&restore_script(&id, caret, caret));
        } else {
            doc.set(next.clone());
            state.on_change.call(next);
        }
    });
}

/// Keys in a paragraph: Ctrl/Cmd+B, I, U toggle a mark, Ctrl/Cmd+K links the selection, Ctrl/Cmd+Shift+L toggles a
/// checklist item, Enter splits. Every key stops here so typing never reaches the section's shortcuts.
fn paragraph_key(event: KeyboardEvent, state: DocState, index: usize, id_base: String) {
    event.stop_propagation();
    let mods = event.modifiers();
    let cmd = mods.ctrl() || mods.meta();
    match event.key() {
        Key::Enter if !cmd => {
            event.prevent_default();
            run_paragraph_action(state, index, id_base, ParaAction::Split);
        }
        Key::Character(c) if cmd => {
            let action = match c.to_lowercase().as_str() {
                "b" => Some(ParaAction::Mark(Mark::Bold)),
                "i" => Some(ParaAction::Mark(Mark::Italic)),
                "u" => Some(ParaAction::Mark(Mark::Underline)),
                "k" => Some(ParaAction::Link),
                "l" if mods.shift() => Some(ParaAction::Checklist),
                _ => None,
            };
            if let Some(action) = action {
                event.prevent_default();
                run_paragraph_action(state, index, id_base, action);
            }
        }
        _ => {}
    }
}

/// One paragraph row, with every string its markup needs.
#[derive(Clone, PartialEq)]
struct DocRow {
    index: usize,
    key: String,
    id: String,
    class: String,
    marker: String,
    label: String,
    done_label: String,
    checked: Option<bool>,
    snap: String,
}

fn row_class(kind: ParagraphKind) -> &'static str {
    match kind {
        ParagraphKind::Plain => "kit-dq__pararow",
        ParagraphKind::Bullet => "kit-dq__pararow kit-dq__pararow--bullet",
        ParagraphKind::Numbered => "kit-dq__pararow kit-dq__pararow--numbered",
        ParagraphKind::Checklist { done: false } => "kit-dq__pararow kit-dq__pararow--check",
        ParagraphKind::Checklist { done: true } => {
            "kit-dq__pararow kit-dq__pararow--check kit-dq__pararow--done"
        }
        ParagraphKind::Heading => "kit-dq__pararow kit-dq__pararow--heading",
    }
}

/// The document: paragraphs with inline marks, line-start kinds, a link field for the selection, and an add control.
#[component]
fn DocumentView(
    paragraphs: Vec<Paragraph>,
    id_base: String,
    on_change: EventHandler<Vec<Paragraph>>,
) -> Element {
    let doc = use_signal(|| paragraphs.clone());
    let snaps = use_signal(|| snapshots(&paragraphs));
    let link_at = use_signal(|| None::<(usize, usize, usize)>);
    let link_draft = use_signal(String::new);
    let revision = use_signal(|| 0_usize);
    let state = DocState {
        doc,
        snaps,
        on_change,
        link_at,
        link_draft,
        revision,
    };

    let current = doc();
    let markup = snaps();
    let nums = numbers(&current);
    let rev = revision();
    let rows: Vec<DocRow> = current
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let marker = match p.kind {
                ParagraphKind::Bullet => "•".to_string(),
                ParagraphKind::Numbered => format!("{}.", nums[i]),
                _ => String::new(),
            };
            DocRow {
                index: i,
                key: format!("{i}-{rev}"),
                id: format!("{id_base}-para-{i}"),
                class: row_class(p.kind).to_string(),
                marker,
                label: format!("Paragraph {}", i + 1),
                done_label: format!("Done: {}", p.text()),
                checked: match p.kind {
                    ParagraphKind::Checklist { done } => Some(done),
                    _ => None,
                },
                snap: markup.get(i).cloned().unwrap_or_default(),
            }
        })
        .collect();
    let open_link = link_at();
    let draft = link_draft();
    let add_base = id_base.clone();

    rsx! {
        div { class: "kit-dq__doc", onkeydown: move |e: KeyboardEvent| e.stop_propagation(),
            for DocRow { index, key, id, class, marker, label, done_label, checked, snap } in rows {
                div { key: "{key}", class: "{class}",
                    if let Some(done) = checked {
                        button { class: "kit-dq__check", r#type: "button", role: "checkbox", "aria-checked": "{done}", "aria-label": "{done_label}",
                            onclick: move |_| {
                                let mut doc = state.doc;
                                let mut next = doc();
                                if let Some(para) = next.get_mut(index) {
                                    para.kind = ParagraphKind::Checklist { done: !done };
                                }
                                doc.set(next.clone());
                                state.on_change.call(next);
                            },
                            if done { "✓" }
                        }
                    } else if !marker.is_empty() {
                        span { class: "kit-dq__marker", "aria-hidden": "true", "{marker}" }
                    }
                    div { id: "{id}", class: "kit-dq__ptext", contenteditable: "true", role: "textbox", "aria-multiline": "false",
                        "aria-label": "{label}", dangerous_inner_html: "{snap}",
                        oninput: {
                            let base = id_base.clone();
                            move |_| sync_typed(state, index, base.clone())
                        },
                        onkeydown: {
                            let base = id_base.clone();
                            move |e: KeyboardEvent| paragraph_key(e, state, index, base.clone())
                        },
                    }
                    if open_link.is_some_and(|(i, _, _)| i == index) {
                        form { class: "kit-dq__linkform",
                            onsubmit: {
                                let base = id_base.clone();
                                move |e: FormEvent| {
                                    e.prevent_default();
                                    let Some((at, start, end)) = state.link_at.peek().clone() else {
                                        return;
                                    };
                                    let mut next = state.doc.peek().clone();
                                    if let Some(para) = next.get_mut(at) {
                                        para.runs = set_link(&para.runs, start, end, normalise_link(&state.link_draft.peek()));
                                    }
                                    let mut link_at = state.link_at;
                                    link_at.set(None);
                                    commit_paragraphs(state, next);
                                    document::eval(&restore_script(&format!("{base}-para-{at}"), start, end));
                                }
                            },
                            label { class: "kit-sr", r#for: "{id_base}-link", "Link address" }
                            input { id: "{id_base}-link", class: "kit-dq__linkinput", r#type: "url", value: "{draft}",
                                oninput: move |e: FormEvent| {
                                    let mut link_draft = state.link_draft;
                                    link_draft.set(e.value());
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    if matches!(e.key(), Key::Escape) {
                                        e.prevent_default();
                                        let mut link_at = state.link_at;
                                        link_at.set(None);
                                    }
                                },
                            }
                        }
                    }
                }
            }
            button { class: "kit-dq__btn kit-dq__addpara", r#type: "button", id: "{add_base}-add-para",
                onclick: move |_| {
                    let mut next = state.doc.peek().clone();
                    let at = next.len();
                    next.push(Paragraph::default());
                    commit_paragraphs(state, next);
                    focus_after_frame(&format!("{add_base}-para-{at}"));
                },
                "Add paragraph"
            }
        }
    }
}

/// A task artifact: a document with marks, a sheet through an editable `SheetGrid`, or an editable folder list.
#[component]
fn ArtifactView(
    item: Artifact,
    #[props(default)] id_base: String,
    #[props(default)] on_change: EventHandler<Artifact>,
) -> Element {
    match item {
        Artifact::Document { paragraphs } => rsx! {
            DocumentView {
                paragraphs,
                id_base,
                on_change: move |next: Vec<Paragraph>| on_change.call(Artifact::Document { paragraphs: next }),
            }
        },
        Artifact::Sheet { columns, rows } => {
            let width = columns.len();
            let (cols, grid) = sheet_from(&columns, &rows);
            rsx! {
                div { class: "kit-dq__sheet", onkeydown: move |e: KeyboardEvent| e.stop_propagation(),
                    SheetGrid {
                        grid_heading: "Sheet",
                        grid_columns: cols,
                        grid_rows: grid,
                        on_rows_change: move |next: Vec<GridRow>| {
                            on_change.call(Artifact::Sheet {
                                columns: columns.clone(),
                                rows: sheet_rows_back(width, &next),
                            });
                        },
                    }
                }
            }
        }
        Artifact::Folder { items } => {
            let count = items.len();
            let rows: Vec<(usize, String)> = items.iter().cloned().enumerate().collect();
            rsx! {
                div { class: "kit-dq__folderwrap", onkeydown: move |e: KeyboardEvent| e.stop_propagation(),
                    ul { class: "kit-dq__folder", "aria-label": "Folder entries",
                        for (i, name) in rows.iter() {
                            li { key: "{i}", class: "kit-dq__entry",
                                input { id: "{id_base}-entry-{i}", class: "kit-dq__entryname", r#type: "text", value: "{name}",
                                    "aria-label": "Entry {i}",
                                    oninput: {
                                        let items = items.clone();
                                        let i = *i;
                                        move |e: FormEvent| {
                                            let mut next = items.clone();
                                            next[i] = e.value();
                                            on_change.call(Artifact::Folder { items: next });
                                        }
                                    },
                                    onkeydown: {
                                        let base = id_base.clone();
                                        let i = *i;
                                        move |e: KeyboardEvent| {
                                            match e.key() {
                                                Key::ArrowDown if i + 1 < count => {
                                                    e.prevent_default();
                                                    focus_after_frame(&format!("{base}-entry-{}", i + 1));
                                                }
                                                Key::ArrowUp if i > 0 => {
                                                    e.prevent_default();
                                                    focus_after_frame(&format!("{base}-entry-{}", i - 1));
                                                }
                                                _ => {}
                                            }
                                        }
                                    },
                                }
                                button { class: "kit-dq__entryremove", r#type: "button", "aria-label": "Remove {name}",
                                    onclick: {
                                        let items = items.clone();
                                        let base = id_base.clone();
                                        let i = *i;
                                        move |_| {
                                            let mut next = items.clone();
                                            next.remove(i);
                                            let focus = if next.is_empty() {
                                                format!("{base}-entry-add")
                                            } else {
                                                format!("{base}-entry-{}", i.min(next.len() - 1))
                                            };
                                            on_change.call(Artifact::Folder { items: next });
                                            focus_after_frame(&focus);
                                        }
                                    },
                                    "Remove"
                                }
                            }
                        }
                    }
                    button { class: "kit-dq__btn", id: "{id_base}-entry-add", r#type: "button",
                        onclick: {
                            let items = items.clone();
                            let base = id_base.clone();
                            move |_| {
                                let mut next = items.clone();
                                next.push("New entry".to_string());
                                let at = next.len() - 1;
                                on_change.call(Artifact::Folder { items: next });
                                focus_after_frame(&format!("{base}-entry-{at}"));
                            }
                        },
                        "Add entry"
                    }
                }
            }
        }
    }
}
