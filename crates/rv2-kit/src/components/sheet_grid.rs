//! Sheet grid: an editable, spreadsheet-style table for small datasets. Select cells and ranges,
//! edit in place, copy, cut and paste tab-separated text, sort by column, undo and redo.
//!
//! Design choice: sorting reorders the stored rows and does not fire `on_rows_change`. Undo keeps
//! the last 100 states. Ctrl/Cmd+V reads the clipboard through `navigator.clipboard.readText`.
//!
//! Not implemented: the row logo; link cells show their text without an anchor target.
//!
//! Browser checks: drag selection across cells, Shift and Ctrl arrows, the mixed header checkbox,
//! and horizontal scroll inside the panel at phone width.

use std::collections::BTreeMap;

use dioxus::prelude::*;
use rv2_affect::checkbox::Checkbox;
use rv2_affect::select::{Select, SelectOption};

use crate::components::agent_context_map::focus_on_mount;
use crate::internal::{KIT_CSS, class_list, copy_to_clipboard, next_id};

const CSS: Asset = asset!("/assets/sheet-grid.css");
pub const HISTORY_LIMIT: usize = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnKind {
    Title,
    Tag,
    Link,
    Number,
    Text,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    Info,
    Brand,
    Highlight,
    Positive,
    Caution,
    #[default]
    Muted,
}

impl Tone {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Brand => "brand",
            Self::Highlight => "highlight",
            Self::Positive => "positive",
            Self::Caution => "caution",
            Self::Muted => "muted",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TagTone {
    pub tag: String,
    pub tone: Tone,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub column_key: String,
    pub column_label: String,
    pub column_width: Option<u32>,
    pub column_kind: ColumnKind,
    pub cells_editable: bool,
    pub tag_choices: Vec<String>,
}

impl Column {
    pub fn new(column_key: &str, column_label: &str, column_kind: ColumnKind) -> Self {
        Self {
            column_key: column_key.to_string(),
            column_label: column_label.to_string(),
            column_width: None,
            column_kind,
            cells_editable: true,
            tag_choices: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GridRow {
    pub row_id: String,
    pub row_logo: Option<String>,
    pub values: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cell {
    pub row: usize,
    pub col: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckState {
    Unchecked,
    Checked,
    Mixed,
}

/// Rectangle `(top, bottom, left, right)` spanned by two cells, inclusive.
pub fn span(anchor: Cell, end: Cell) -> (usize, usize, usize, usize) {
    (
        anchor.row.min(end.row),
        anchor.row.max(end.row),
        anchor.col.min(end.col),
        anchor.col.max(end.col),
    )
}

/// Moves `cell` by `(dr, dc)` clamped to the grid; `to_edge` jumps to the grid's edge instead.
pub fn move_cell(
    cell: Cell,
    rows: usize,
    cols: usize,
    delta: (isize, isize),
    to_edge: bool,
) -> Cell {
    if rows == 0 || cols == 0 {
        return cell;
    }
    let clamp = |v: isize, max: usize| v.clamp(0, max as isize - 1) as usize;
    if to_edge {
        let row = match delta.0.signum() {
            1 => rows - 1,
            -1 => 0,
            _ => cell.row,
        };
        let col = match delta.1.signum() {
            1 => cols - 1,
            -1 => 0,
            _ => cell.col,
        };
        return Cell { row, col };
    }
    Cell {
        row: clamp(cell.row as isize + delta.0, rows),
        col: clamp(cell.col as isize + delta.1, cols),
    }
}

/// Tab-separated text for the rectangle, one line per row.
pub fn copy_text(
    rows: &[GridRow],
    columns: &[Column],
    rect: (usize, usize, usize, usize),
) -> String {
    let (top, bottom, left, right) = rect;
    (top..=bottom.min(rows.len().saturating_sub(1)))
        .map(|r| {
            (left..=right.min(columns.len().saturating_sub(1)))
                .map(|c| {
                    rows[r]
                        .values
                        .get(&columns[c].column_key)
                        .cloned()
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
                .join("\t")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Splits clipboard text into rows of cells.
pub fn parse_paste(text: &str) -> Vec<Vec<String>> {
    text.trim_end_matches(['\n', '\r'])
        .split('\n')
        .map(|line| {
            line.trim_end_matches('\r')
                .split('\t')
                .map(str::to_string)
                .collect()
        })
        .collect()
}

/// Writes pasted cells starting at `anchor`, clipped to the existing rows and columns.
pub fn paste_into(rows: &mut [GridRow], columns: &[Column], anchor: Cell, grid: &[Vec<String>]) {
    for (dr, line) in grid.iter().enumerate() {
        let Some(row) = rows.get_mut(anchor.row + dr) else {
            break;
        };
        for (dc, value) in line.iter().enumerate() {
            if let Some(column) = columns.get(anchor.col + dc) {
                row.values.insert(column.column_key.clone(), value.clone());
            }
        }
    }
}

/// Sort state after a click on `column`: ascending first, then the direction flips.
pub fn next_sort(current: Option<(usize, bool)>, column: usize) -> Option<(usize, bool)> {
    match current {
        Some((c, ascending)) if c == column => Some((column, !ascending)),
        _ => Some((column, true)),
    }
}

pub fn sort_rows(rows: &mut [GridRow], key: &str, ascending: bool) {
    rows.sort_by(|a, b| {
        let (x, y) = (
            a.values.get(key).map_or("", String::as_str),
            b.values.get(key).map_or("", String::as_str),
        );
        let ordering = match (x.parse::<f64>(), y.parse::<f64>()) {
            (Ok(p), Ok(q)) => p.partial_cmp(&q).unwrap_or(std::cmp::Ordering::Equal),
            _ => x.to_lowercase().cmp(&y.to_lowercase()),
        };
        if ascending {
            ordering
        } else {
            ordering.reverse()
        }
    });
}

pub fn header_check(checked: usize, total: usize) -> CheckState {
    match (checked, total) {
        (0, _) => CheckState::Unchecked,
        (c, t) if c >= t => CheckState::Checked,
        _ => CheckState::Mixed,
    }
}

/// Groups the digits of a number and uses a true minus sign. Text that is not a number is returned unchanged.
pub fn format_number(text: &str) -> String {
    let cleaned: String = text.trim().chars().filter(|c| *c != ',').collect();
    let Ok(value) = cleaned.parse::<f64>() else {
        return text.to_string();
    };
    let sign = if value < 0.0 { "−" } else { "" };
    let whole = format!("{:.0}", value.abs().trunc());
    let grouped: String = whole
        .chars()
        .rev()
        .collect::<Vec<char>>()
        .chunks(3)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<String>>()
        .join(",")
        .chars()
        .rev()
        .collect();
    let fraction = cleaned
        .split_once('.')
        .map(|(_, f)| format!(".{f}"))
        .unwrap_or_default();
    format!("{sign}{grouped}{fraction}")
}

/// Undo and redo stacks, capped at `HISTORY_LIMIT` undo states.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct History<T> {
    past: Vec<T>,
    future: Vec<T>,
}

impl<T: Clone> History<T> {
    /// Records the state before an edit.
    pub fn record(&mut self, before: T) {
        self.past.push(before);
        if self.past.len() > HISTORY_LIMIT {
            self.past.remove(0);
        }
        self.future.clear();
    }

    pub fn undo(&mut self, current: T) -> Option<T> {
        let previous = self.past.pop()?;
        self.future.push(current);
        Some(previous)
    }

    pub fn redo(&mut self, current: T) -> Option<T> {
        let next = self.future.pop()?;
        self.past.push(current);
        Some(next)
    }
}

fn key_direction(key: &Key) -> Option<(isize, isize)> {
    match key {
        Key::ArrowUp => Some((-1, 0)),
        Key::ArrowDown => Some((1, 0)),
        Key::ArrowLeft => Some((0, -1)),
        Key::ArrowRight => Some((0, 1)),
        _ => None,
    }
}

#[component]
pub fn SheetGrid(
    #[props(default = "Sheet".to_string(), into)] grid_heading: String,
    #[props(default)] grid_columns: Vec<Column>,
    #[props(default)] grid_rows: Vec<GridRow>,
    #[props(default)] tag_tones: Vec<TagTone>,
    #[props(default)] read_only: bool,
    #[props(default)] on_rows_change: EventHandler<Vec<GridRow>>,
    #[props(default)] on_checked_change: EventHandler<Vec<String>>,
    #[props(default, into)] extra_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-sg-{}", next_id()));
    let mut rows = use_signal(|| grid_rows.clone());
    let mut sort = use_signal(|| None::<(usize, bool)>);
    let mut anchor = use_signal(Cell::default);
    let mut active = use_signal(Cell::default);
    let mut editing = use_signal(|| None::<(Cell, String)>);
    let mut checked = use_signal(Vec::<String>::new);
    let mut history = use_signal(History::<Vec<GridRow>>::default);
    let mut dragging = use_signal(|| false);

    let columns = grid_columns.clone();
    let view = rows();
    let row_count = view.len();
    let col_count = columns.len();
    let (top, bottom, left, right) = span(anchor(), active());
    let in_range = move |r: usize, c: usize| r >= top && r <= bottom && c >= left && c <= right;
    let checked_now = checked();
    let header_state = header_check(checked_now.len(), row_count);
    let is_editing = editing();

    let mut commit = move |next: Vec<GridRow>| {
        let before = rows();
        history.with_mut(|h| h.record(before));
        rows.set(next.clone());
        on_rows_change.call(next);
    };

    let columns_for_keys = columns.clone();
    let on_key = move |event: KeyboardEvent| {
        if editing().is_some() {
            return;
        }
        let modified = event.modifiers().ctrl() || event.modifiers().meta();
        let shift = event.modifiers().shift();
        let key = event.key();
        if let Some(delta) = key_direction(&key) {
            event.prevent_default();
            let next = move_cell(active(), row_count, col_count, delta, modified);
            active.set(next);
            if !shift {
                anchor.set(next);
            }
            return;
        }
        if modified {
            if let Key::Character(c) = &key {
                match c.to_lowercase().as_str() {
                    "c" | "x" => {
                        let text = copy_text(&rows(), &columns_for_keys, span(anchor(), active()));
                        copy_to_clipboard(&text);
                        if c.eq_ignore_ascii_case("x") && !read_only {
                            let (t, b, l, r) = span(anchor(), active());
                            let mut next = rows();
                            for row in next.iter_mut().take(b + 1).skip(t) {
                                for col in columns_for_keys.iter().take(r + 1).skip(l) {
                                    row.values.insert(col.column_key.clone(), String::new());
                                }
                            }
                            commit(next);
                        }
                    }
                    "z" if !read_only => {
                        let current = rows();
                        let previous = history.with_mut(|h| {
                            if shift {
                                h.redo(current.clone())
                            } else {
                                h.undo(current.clone())
                            }
                        });
                        if let Some(state) = previous {
                            rows.set(state.clone());
                            on_rows_change.call(state);
                        }
                    }
                    "v" if !read_only => {
                        let paste_columns = columns_for_keys.clone();
                        let paste_anchor = active();
                        let mut commit_paste = commit;
                        spawn(async move {
                            let Ok(value) =
                                document::eval("return await navigator.clipboard.readText();")
                                    .await
                            else {
                                return;
                            };
                            let Some(text) = value.as_str() else {
                                return;
                            };
                            let grid = parse_paste(text);
                            let mut next = rows();
                            paste_into(&mut next, &paste_columns, paste_anchor, &grid);
                            commit_paste(next);
                        });
                    }
                    "a" => {
                        anchor.set(Cell { row: 0, col: 0 });
                        active.set(Cell {
                            row: row_count.saturating_sub(1),
                            col: col_count.saturating_sub(1),
                        });
                    }
                    _ => {}
                }
                event.prevent_default();
            }
            return;
        }
        match key {
            Key::Escape => anchor.set(active()),
            Key::Enter | Key::F2 if !read_only => {
                let cur = active();
                let value = rows()
                    .get(cur.row)
                    .and_then(|r| r.values.get(&columns_for_keys[cur.col].column_key).cloned())
                    .unwrap_or_default();
                editing.set(Some((cur, value)));
            }
            Key::Delete | Key::Backspace if !read_only => {
                let (t, b, l, r) = span(anchor(), active());
                let mut next = rows();
                for row in next.iter_mut().take(b + 1).skip(t) {
                    for col in columns_for_keys.iter().take(r + 1).skip(l) {
                        row.values.insert(col.column_key.clone(), String::new());
                    }
                }
                commit(next);
            }
            Key::Character(c) if !read_only && c.chars().count() == 1 => {
                // Seeds the editor; the keystroke must not also reach the editor it mounts.
                event.prevent_default();
                let cur = active();
                editing.set(Some((cur, c.clone())));
            }
            _ => {}
        }
    };

    let scroll_dom_id = id.clone();
    let add_prefix = id.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class_list("kit-sg", &extra_class), "aria-label": "{grid_heading}",
            header { class: "kit-sg__head", h3 { class: "kit-sg__title", "{grid_heading}" } }
            div {
                id: "{scroll_dom_id}",
                class: "kit-sg__scroll",
                tabindex: "0",
                onkeydown: on_key,
                table { class: "kit-sg__table", role: "grid", "aria-label": "{grid_heading}", "aria-rowcount": "{row_count}",
                    thead {
                        tr {
                            th { class: "kit-sg__check",
                                Checkbox {
                                    checked: header_state == CheckState::Checked,
                                    indeterminate: header_state == CheckState::Mixed,
                                    onchange: move |_| {
                                        let next: Vec<String> = if header_state == CheckState::Checked {
                                            Vec::new()
                                        } else {
                                            rows().iter().map(|r| r.row_id.clone()).collect()
                                        };
                                        checked.set(next.clone());
                                        on_checked_change.call(next);
                                    },
                                    label: rsx! { "Select all rows" },
                                }
                            }
                            for (ci, column) in columns.iter().enumerate() {
                                {
                                    let label = column.column_label.clone();
                                    let width = column.column_width.map(|w| format!("{w}px")).unwrap_or_default();
                                    let aria = match sort() {
                                        Some((c, true)) if c == ci => "ascending",
                                        Some((c, false)) if c == ci => "descending",
                                        _ => "none",
                                    };
                                    let key = column.column_key.clone();
                                    rsx! {
                                        th { key: "{key}", class: "kit-sg__head-cell", "aria-sort": aria, style: "min-width: {width}",
                                            button {
                                                r#type: "button",
                                                class: "kit-sg__sort",
                                                onclick: move |_| {
                                                    let next = next_sort(sort(), ci);
                                                    sort.set(next);
                                                    if let Some((_, asc)) = next {
                                                        let mut sorted = rows();
                                                        sort_rows(&mut sorted, &key, asc);
                                                        rows.set(sorted);
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
                    tbody {
                        for (ri, row) in view.iter().enumerate() {
                            {
                                let row_id = row.row_id.clone();
                                let is_checked = checked_now.contains(&row_id);
                                let row_label = format!("Row {}", ri + 1);
                                rsx! {
                                    tr { key: "{row_id}", "aria-rowindex": "{ri + 1}",
                                        td { class: "kit-sg__check",
                                            Checkbox {
                                                checked: is_checked,
                                                onchange: {
                                                    let rid = row_id.clone();
                                                    move |_| {
                                                        let mut next = checked();
                                                        if let Some(pos) = next.iter().position(|v| *v == rid) {
                                                            next.remove(pos);
                                                        } else {
                                                            next.push(rid.clone());
                                                        }
                                                        checked.set(next.clone());
                                                        on_checked_change.call(next);
                                                    }
                                                },
                                                label: rsx! { "{row_label}" },
                                            }
                                        }
                                        for (ci, column) in columns.iter().enumerate() {
                                            {
                                                let value = row.values.get(&column.column_key).cloned().unwrap_or_default();
                                                let shown = if column.column_kind == ColumnKind::Number { format_number(&value) } else { value.clone() };
                                                let selected = in_range(ri, ci);
                                                let is_active = active() == Cell { row: ri, col: ci };
                                                let cell_editing = is_editing.as_ref().filter(|(c, _)| *c == Cell { row: ri, col: ci }).map(|(_, draft)| draft.clone());
                                                let cell_key = format!("{row_id}-{}", column.column_key);
                                                let editable = column.cells_editable && !read_only;
                                                let tones = tag_tones.clone();
                                                let is_tag = column.column_kind == ColumnKind::Tag;
                                                let tags: Vec<String> = if is_tag { value.split(',').map(str::trim).filter(|t| !t.is_empty()).map(str::to_string).collect() } else { Vec::new() };
                                                let choices: Vec<SelectOption> = column.tag_choices.iter().map(|t| SelectOption { value: t.clone(), label: t.clone(), description: None, icon: None, disabled: false }).collect();
                                                let column_key = column.column_key.clone();
                                                let mut commit_value = commit;
                                                let rows_snapshot = view.clone();
                                                rsx! {
                                                    td {
                                                        key: "{cell_key}",
                                                        class: "kit-sg__cell",
                                                        role: "gridcell",
                                                        "data-active": is_active.then_some("true"),
                                                        "data-selected": selected.then_some("true"),
                                                        "data-kind": if is_tag { "tag" } else { "value" },
                                                        onmousedown: move |_| {
                                                            anchor.set(Cell { row: ri, col: ci });
                                                            active.set(Cell { row: ri, col: ci });
                                                            editing.set(None);
                                                            dragging.set(true);
                                                        },
                                                        onmouseenter: move |_| {
                                                            if dragging() {
                                                                active.set(Cell { row: ri, col: ci });
                                                            }
                                                        },
                                                        onmouseup: move |_| dragging.set(false),
                                                        ondoubleclick: move |_| {
                                                            if editable {
                                                                let current = rows_snapshot.get(ri).and_then(|r| r.values.get(&column_key).cloned()).unwrap_or_default();
                                                                editing.set(Some((Cell { row: ri, col: ci }, current)));
                                                            }
                                                        },
                                                        if let Some(draft) = cell_editing {
                                                            if is_tag {
                                                                Select {
                                                                    options: choices,
                                                                    value: value.clone(),
                                                                    onchange: {
                                                                        let key = column.column_key.clone();
                                                                        move |option: SelectOption| {
                                                                            let mut next = rows();
                                                                            if let Some(r) = next.get_mut(ri) {
                                                                                r.values.insert(key.clone(), option.value);
                                                                            }
                                                                            editing.set(None);
                                                                            commit_value(next);
                                                                        }
                                                                    },
                                                                }
                                                            } else {
                                                                input {
                                                                    class: "kit-sg__editor",
                                                                    r#type: "text",
                                                                    "aria-label": "Edit cell",
                                                                    value: "{draft}",
                                                                    onmounted: focus_on_mount,
                                                                    oninput: move |event| {
                                                                        let text = event.value();
                                                                        editing.set(Some((Cell { row: ri, col: ci }, text)));
                                                                    },
                                                                    onkeydown: {
                                                                        let key = column.column_key.clone();
                                                                        let scroll_id = id.clone();
                                                                        move |event: KeyboardEvent| {
                                                                            event.stop_propagation();
                                                                            let select_all = (event.modifiers().ctrl() || event.modifiers().meta())
                                                                                && matches!(event.key(), Key::Character(ref c) if c.eq_ignore_ascii_case("a"));
                                                                            if select_all {
                                                                                // macOS maps Ctrl+A to line start in inputs, so select explicitly.
                                                                                event.prevent_default();
                                                                                document::eval("document.activeElement?.select?.();");
                                                                                return;
                                                                            }
                                                                                            match event.key() {
                                                                                Key::Enter => {
                                                                                    event.prevent_default();
                                                                                    let text = editing().map(|(_, d)| d).unwrap_or_default();
                                                                                    let mut next = rows();
                                                                                    if let Some(r) = next.get_mut(ri) {
                                                                                        r.values.insert(key.clone(), text);
                                                                                    }
                                                                                    editing.set(None);
                                                                                    commit_value(next);
                                                                                    active.set(Cell { row: (ri + 1).min(row_count.saturating_sub(1)), col: ci });
                                                                                    document::eval(&format!("document.getElementById({scroll_id:?})?.focus();"));
                                                                                }
                                                                                Key::Escape => {
                                                                                    editing.set(None);
                                                                                    document::eval(&format!("document.getElementById({scroll_id:?})?.focus();"));
                                                                                }
                                                                                _ => {}
                                                                            }
                                                                        }
                                                                    },
                                                                }
                                                            }
                                                        } else if is_tag {
                                                            for tag in tags.iter() {
                                                                {
                                                                    let tone = tones.iter().find(|t| t.tag == *tag).map(|t| t.tone).unwrap_or_default();
                                                                    rsx! { span { key: "{tag}", class: "kit-sg__chip", "data-tone": tone.as_str(), "{tag}" } }
                                                                }
                                                            }
                                                        } else {
                                                            span { class: if column.column_kind == ColumnKind::Link { "kit-sg__link" } else { "kit-sg__value" }, "{shown}" }
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
            if !read_only {
                footer { class: "kit-sg__foot",
                    button {
                        r#type: "button",
                        class: "kit-sg__add",
                        onclick: move |_| {
                            let mut next = rows();
                            let row_id = format!("{add_prefix}-new-{}", next.len() + 1);
                            next.push(GridRow { row_id, ..GridRow::default() });
                            active.set(Cell { row: next.len() - 1, col: 0 });
                            anchor.set(Cell { row: next.len() - 1, col: 0 });
                            commit(next);
                        },
                        "Add row"
                    }
                }
            }
            p { class: "kit-sg__live", role: "status", "aria-live": "polite", "{row_count} rows" }
        }
    }
}
