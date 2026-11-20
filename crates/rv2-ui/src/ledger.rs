//! Records of money and where a figure came from: a ledger table and a dated
//! citation.
//!
//! `LedgerTable` is drawn from Worthwake's account and activity tables, Edit
//! Scope's quote table, A Place Ahead's calculator schedule and Talk Run
//! Card's timings. It is not [`crate::CompareTable`]: there the columns are
//! options and the page stacks them on a phone; here the rows are records,
//! figures align at the end, a row can be voided or mark a milestone, and a
//! narrow screen scrolls the table inside a focusable region instead.
//!
//! `Citation` is the dated source line that Sponsor Ledger, A Place Ahead and
//! Jobbreakdown each print under a figure taken from somewhere else.

use dioxus::prelude::*;

use crate::primitives::TEXT_LINK;
use crate::readout::Lean;

/// What a [`LedgerTable`] column holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColumnKind {
    #[default]
    Text,
    /// A number: end-aligned, tabular digits, never wrapped.
    Figure,
}

impl ColumnKind {
    fn class(self) -> &'static str {
        match self {
            Self::Text => "text-start",
            Self::Figure => "text-end tabular-nums whitespace-nowrap",
        }
    }
}

/// One column of a [`LedgerTable`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerColumn {
    pub label: String,
    pub kind: ColumnKind,
}

impl LedgerColumn {
    pub fn text(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: ColumnKind::Text,
        }
    }

    pub fn figure(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: ColumnKind::Figure,
        }
    }
}

/// One cell: its value, an optional second line, and the ink of the value.
#[derive(Debug, Clone, PartialEq)]
pub struct LedgerCell {
    pub value: String,
    pub note: Option<String>,
    /// Colours the value. The value's own text must still carry its sign.
    pub lean: Lean,
    /// A link on the row's first cell, to the record itself.
    pub href: Option<String>,
}

impl LedgerCell {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            note: None,
            lean: Lean::Neutral,
            href: None,
        }
    }

    #[must_use]
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    #[must_use]
    pub fn lean(mut self, lean: Lean) -> Self {
        self.lean = lean;
        self
    }

    #[must_use]
    pub fn href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }
}

impl<T: Into<String>> From<T> for LedgerCell {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// What a [`LedgerRow`] is, beyond its figures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RowState {
    #[default]
    Normal,
    /// Kept for the record and no longer counted. Every cell says so in words
    /// to assistive technology, since a reader moving cell by cell may never
    /// hear the row header; on screen the cells after the header are struck
    /// through and the row drops to muted ink.
    Voided,
    /// A boundary in the run — a year end, a subtotal.
    Milestone,
}

impl RowState {
    fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Voided => "voided",
            Self::Milestone => "milestone",
        }
    }

    fn class(self) -> &'static str {
        match self {
            Self::Normal | Self::Voided => "",
            Self::Milestone => "border-t-2 border-rule font-semibold",
        }
    }
}

/// One record of a [`LedgerTable`].
#[derive(Clone, PartialEq)]
pub struct LedgerRow {
    pub key: String,
    pub cells: Vec<LedgerCell>,
    pub state: RowState,
    /// A control for this row — void, edit — in a trailing column.
    pub action: Option<Element>,
}

impl LedgerRow {
    pub fn new(key: impl Into<String>, cells: Vec<LedgerCell>) -> Self {
        Self {
            key: key.into(),
            cells,
            state: RowState::Normal,
            action: None,
        }
    }

    #[must_use]
    pub fn voided(mut self) -> Self {
        self.state = RowState::Voided;
        self
    }

    #[must_use]
    pub fn milestone(mut self) -> Self {
        self.state = RowState::Milestone;
        self
    }

    #[must_use]
    pub fn action(mut self, action: Element) -> Self {
        self.action = Some(action);
        self
    }
}

/// Records as rows: transactions, a quote's lines, a schedule.
///
/// The first cell of each row is its row header. A `total` goes in `tfoot`, so
/// it is never read as one more record. Callers render an [`crate::EmptyState`]
/// themselves when there are no rows; the table owns no copy.
#[component]
pub fn LedgerTable(
    caption: String,
    /// Keeps the caption for assistive technology and hides it on screen,
    /// where a heading above already names the table.
    #[props(default)]
    caption_hidden: bool,
    columns: Vec<LedgerColumn>,
    rows: Vec<LedgerRow>,
    #[props(default)] total: Option<LedgerRow>,
    /// The header of the trailing action column, when any row has an action.
    #[props(default = "Actions".to_string())]
    action_label: String,
) -> Element {
    let has_actions = rows.iter().any(|row| row.action.is_some());
    let kinds: Vec<ColumnKind> = columns.iter().map(|c| c.kind).collect();
    let caption_class = if caption_hidden {
        "sr-only"
    } else {
        "px-3 pt-3 pb-2 text-start text-sm font-semibold text-ink"
    };
    rsx! {
        div {
            class: "overflow-x-auto rounded-lg border border-rule-soft focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent",
            role: "region",
            aria_label: "{caption}",
            tabindex: "0",
            table { class: "w-full border-collapse text-sm",
                caption { class: caption_class, "{caption}" }
                thead {
                    tr { class: "border-b border-rule bg-raised",
                        for column in columns.iter() {
                            th {
                                key: "{column.label}",
                                scope: "col",
                                class: "px-3 py-2 font-semibold text-ink-muted {column.kind.class()}",
                                "{column.label}"
                            }
                        }
                        if has_actions {
                            th { scope: "col", class: "px-3 py-2 text-end font-semibold text-ink-muted", "{action_label}" }
                        }
                    }
                }
                tbody {
                    for row in rows {
                        LedgerRowView { key: "{row.key}", row, kinds: kinds.clone(), has_actions }
                    }
                }
                if let Some(total) = total {
                    tfoot { class: "border-t-2 border-ink font-semibold",
                        LedgerRowView { row: total, kinds: kinds.clone(), has_actions }
                    }
                }
            }
        }
    }
}

#[component]
fn LedgerRowView(row: LedgerRow, kinds: Vec<ColumnKind>, has_actions: bool) -> Element {
    let voided = row.state == RowState::Voided;
    rsx! {
        tr { class: "border-b border-rule-soft last:border-b-0 {row.state.class()}", "data-state": row.state.name(),
            for (i , cell) in row.cells.into_iter().enumerate() {
                {
                    let kind = kinds.get(i).copied().unwrap_or_default();
                    let body = rsx! {
                        CellBody { cell, voided, header: i == 0 }
                    };
                    if i == 0 {
                        rsx! { th { scope: "row", class: "px-3 py-2 font-normal {kind.class()}", {body} } }
                    } else {
                        rsx! { td { class: "px-3 py-2 {kind.class()}", {body} } }
                    }
                }
            }
            if has_actions {
                td { class: "px-3 py-2 text-end",
                    if let Some(action) = row.action {
                        {action}
                    }
                }
            }
        }
    }
}

#[component]
fn CellBody(cell: LedgerCell, voided: bool, header: bool) -> Element {
    // Muted ink rather than opacity: opacity drops lean-coloured figures
    // below text contrast.
    let ink = if voided {
        "text-ink-muted"
    } else {
        cell.lean.text()
    };
    let content = match cell.href {
        Some(href) => rsx! { a { class: "{TEXT_LINK} {ink}", href, "{cell.value}" } },
        None => rsx! { span { class: ink, "{cell.value}" } },
    };
    let value = if voided && !header {
        rsx! { del { {content} } }
    } else {
        content
    };
    rsx! {
        if voided {
            span { class: "sr-only", "Voided: " }
        }
        {value}
        if let Some(note) = cell.note {
            small { class: "block text-xs text-ink-muted", "{note}" }
        }
    }
}

/// What a [`Citation`]'s date records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dated {
    /// When the figure was copied from the source.
    #[default]
    Captured,
    /// When the source was last read and still said this.
    Checked,
    /// When the source itself was published.
    Published,
}

impl Dated {
    pub fn word(self) -> &'static str {
        match self {
            Self::Captured => "Captured",
            Self::Checked => "Checked",
            Self::Published => "Published",
        }
    }
}

/// Where a figure came from and when: "Checked 24 Sep 2026 · Open Ofsted ↗".
///
/// The kit has no date library, so the caller passes the date twice: ISO for
/// `datetime` and as the reader should see it.
#[component]
pub fn Citation(
    source: String,
    date_iso: String,
    date_text: String,
    #[props(default)] dated: Dated,
    #[props(default)] href: Option<String>,
) -> Element {
    rsx! {
        p { class: "text-xs text-ink-muted",
            "{dated.word()} "
            time { datetime: "{date_iso}", "{date_text}" }
            if let Some(href) = href {
                " · "
                a { class: TEXT_LINK, href, target: "_blank", rel: "noopener noreferrer",
                    "Open {source}"
                    span { aria_hidden: "true", " ↗" }
                    span { class: "sr-only", " (opens in new tab)" }
                }
            } else {
                " · {source}"
            }
        }
    }
}
