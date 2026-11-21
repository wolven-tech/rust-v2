//! Numbers and judgements a reader acts on: a strip of headline figures, a
//! calculator's answer, a meter against thresholds, a whole split into its
//! parts, a readiness checklist, a redline of an edit, options side by side.
//!
//! Every bet with a calculator or a dashboard had written these by hand, with
//! the same shape each time: a mono uppercase label, a large tabular figure, a
//! note under it. Chargewindow, Move to Own, Solar Quote Check, Viewing Ledger,
//! Sponsor Ledger, Talk Run Card, Worthwake, A Place Ahead and the portfolio
//! control room are the consumers this module was drawn from.
//!
//! The figure is always text. A bar or a colour sits beside it and never
//! replaces it, for the reason [`crate::Bar`] gives.

use dioxus::prelude::*;

use crate::feedback::Verdict;

/// Which way a figure leans, for a reader deciding whether it is good news.
///
/// Said in colour **and** in a leading sign the caller writes into the value
/// (`+£120`, `−4 min`): colour alone is not a judgement anyone can act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lean {
    #[default]
    Neutral,
    /// Under budget, above the floor, ahead of time.
    Good,
    /// Close to a limit, not over it.
    Close,
    /// Over budget, below the floor, behind.
    Bad,
}

impl Lean {
    fn text(self) -> &'static str {
        match self {
            Lean::Neutral => "text-ink",
            Lean::Good => "text-takeable-ink",
            Lean::Close => "text-caution-ink",
            Lean::Bad => "text-fault-ink",
        }
    }

    fn fill(self) -> &'static str {
        match self {
            Lean::Neutral => "bg-ink-muted",
            Lean::Good => "bg-takeable",
            Lean::Close => "bg-caution",
            Lean::Bad => "bg-fault",
        }
    }
}

/// One cell of a [`StatStrip`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stat {
    pub label: String,
    pub value: String,
    pub note: Option<String>,
    pub lean: Lean,
}

impl Stat {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            note: None,
            lean: Lean::Neutral,
        }
    }

    #[must_use]
    pub fn note(self, note: impl Into<String>) -> Self {
        Self {
            note: Some(note.into()),
            ..self
        }
    }

    #[must_use]
    pub fn lean(self, lean: Lean) -> Self {
        Self { lean, ..self }
    }
}

/// How loud a [`StatStrip`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Density {
    /// A summary row above a table or a form.
    #[default]
    Compact,
    /// The band a page is about: figures at display size.
    Hero,
}

/// A row of headline figures between rules, collapsing to two columns on a
/// narrow screen.
///
/// A `dl`, so each figure is announced with its label rather than as a bare
/// number.
#[component]
pub fn StatStrip(
    stats: Vec<Stat>,
    #[props(default)] density: Density,
    #[props(default)] class: String,
) -> Element {
    let figure = match density {
        Density::Compact => "text-2xl",
        Density::Hero => "text-4xl sm:text-5xl tracking-tight",
    };
    let columns = match stats.len() {
        0..=2 => "sm:grid-cols-2",
        3 => "sm:grid-cols-3",
        _ => "sm:grid-cols-4",
    };
    rsx! {
        dl { class: "grid grid-cols-2 {columns} border-y-2 border-ink {class}",
            for stat in stats {
                div { key: "{stat.label}", class: "border-rule-soft px-4 py-3 not-first:border-l max-sm:even:border-l max-sm:odd:border-l-0",
                    dt { class: "font-mono text-xs font-medium uppercase tracking-widest text-ink-muted", "{stat.label}" }
                    dd { class: "mt-1 font-mono font-semibold tabular-nums {figure} {stat.lean.text()}", "{stat.value}" }
                    if let Some(note) = stat.note {
                        dd { class: "mt-1 text-xs text-ink-muted", "{note}" }
                    }
                }
            }
        }
    }
}

/// A calculator's answer: a kicker row, one large figure, and the working
/// beneath it (a [`crate::FactList`], usually).
///
/// `aria-live="polite"`, so a screen reader hears the new answer when an
/// input changes it, without being interrupted mid-sentence.
#[component]
pub fn ResultPanel(
    label: String,
    value: String,
    /// The right side of the kicker: the period, the basis, the date.
    #[props(default)]
    context: Option<String>,
    #[props(default)] lean: Lean,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    rsx! {
        section { class: "rounded-lg border-2 border-ink bg-surface p-6 {class}", aria_live: "polite",
            div { class: "flex items-baseline justify-between gap-4 font-mono text-xs font-medium uppercase tracking-widest text-ink-muted",
                span { "{label}" }
                if let Some(context) = context {
                    span { "{context}" }
                }
            }
            p { class: "mt-2 font-mono text-5xl font-semibold tabular-nums tracking-tight {lean.text()}", "{value}" }
            div { class: "mt-4", {children} }
        }
    }
}

/// How far along something is against its limit, coloured by threshold.
///
/// `role="meter"` with its value and range, so the number is available to a
/// screen reader that cannot see the fill.
#[component]
pub fn Meter(
    label: String,
    value: f64,
    max: f64,
    /// The text shown for the value — `"412 of 500 runs"`. The fill never
    /// carries the number alone.
    readout: String,
    /// From this share of `max` the fill turns to caution.
    #[props(default)]
    warn_at: Option<f64>,
    /// From this share of `max` the fill turns to fault.
    #[props(default)]
    critical_at: Option<f64>,
    /// Draws this many equal ticks across the track, for a quantity counted in
    /// whole portions — doses, slots, sections.
    #[props(default)]
    ticks: Option<u32>,
) -> Element {
    let share = if max > 0.0 {
        (value / max).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let lean = match (critical_at, warn_at) {
        (Some(c), _) if share >= c => Lean::Bad,
        (_, Some(w)) if share >= w => Lean::Close,
        (None, None) => Lean::Neutral,
        _ => Lean::Good,
    };
    let width = share * 100.0;
    let tick_style = ticks
        .filter(|n| *n > 1)
        .map(|n| {
            let step = 100.0 / f64::from(n);
            format!(
                "background-image:repeating-linear-gradient(to right,transparent 0 calc({step}% - 1px),var(--color-surface) calc({step}% - 1px) {step}%)"
            )
        })
        .unwrap_or_default();
    rsx! {
        div { class: "space-y-1",
            div { class: "flex items-baseline justify-between gap-4 text-sm",
                span { class: "font-medium text-ink", "{label}" }
                span { class: "tabular-nums {lean.text()}", "{readout}" }
            }
            div {
                class: "relative h-2.5 w-full overflow-hidden rounded-full bg-raised",
                role: "meter",
                aria_label: "{label}",
                aria_valuenow: "{value}",
                aria_valuemin: "0",
                aria_valuemax: "{max}",
                aria_valuetext: "{readout}",
                div { class: "h-full rounded-full {lean.fill()}", style: "width:{width:.1}%" }
                div { class: "absolute inset-0", style: "{tick_style}", aria_hidden: "true" }
            }
        }
    }
}

/// One part of a [`SplitBar`].
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub label: String,
    pub amount: f64,
    /// The amount as the reader should see it — `"£4.20"`.
    pub readout: String,
    /// A CSS colour; the category's own, where it has one.
    pub colour: String,
}

impl Part {
    pub fn new(
        label: impl Into<String>,
        amount: f64,
        readout: impl Into<String>,
        colour: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            amount,
            readout: readout.into(),
            colour: colour.into(),
        }
    }
}

/// A whole split into its parts — energy against fees, principal against
/// interest — as one bar with a legend that carries every figure.
///
/// [`crate::Bar`] compares many things to the biggest of them; this shows what
/// one thing is made of.
#[component]
pub fn SplitBar(label: String, parts: Vec<Part>) -> Element {
    let total: f64 = parts.iter().map(|p| p.amount.max(0.0)).sum();
    let share = |amount: f64| {
        if total > 0.0 {
            amount.max(0.0) / total * 100.0
        } else {
            0.0
        }
    };
    let summary = parts
        .iter()
        .map(|p| format!("{} {}", p.label, p.readout))
        .collect::<Vec<_>>()
        .join(", ");
    rsx! {
        figure { class: "space-y-2",
            div { class: "flex h-3 w-full overflow-hidden rounded-full bg-raised", role: "img", aria_label: "{label}: {summary}",
                for part in parts.iter() {
                    div { key: "{part.label}", style: "width:{share(part.amount):.1}%;background:{part.colour}" }
                }
            }
            figcaption { class: "flex flex-wrap gap-x-4 gap-y-1 text-xs text-ink-muted",
                for part in parts.iter() {
                    span { key: "{part.label}", class: "inline-flex items-center gap-1.5",
                        crate::Swatch { colour: part.colour.clone() }
                        "{part.label}"
                        span { class: "tabular-nums font-semibold text-ink", "{part.readout}" }
                    }
                }
            }
        }
    }
}

/// One check in a [`GateList`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    pub label: String,
    pub verdict: Verdict,
    pub detail: Option<String>,
}

impl Gate {
    pub fn new(label: impl Into<String>, verdict: Verdict) -> Self {
        Self {
            label: label.into(),
            verdict,
            detail: None,
        }
    }

    #[must_use]
    pub fn detail(self, detail: impl Into<String>) -> Self {
        Self {
            detail: Some(detail.into()),
            ..self
        }
    }
}

fn gate_mark(verdict: Verdict) -> (&'static str, &'static str) {
    match verdict {
        Verdict::Clear => ("✓", "Passed"),
        Verdict::Caution => ("○", "Open"),
        Verdict::Stop => ("✕", "Failed"),
        Verdict::Note => ("–", "Not checked"),
    }
}

/// A readiness checklist: each row a glyph, a label and why.
///
/// The glyph is hidden from assistive technology and its meaning is spoken as
/// text instead, so the state never rests on a shape or a colour. Drawn from
/// the control room's gate rows, Logo Handoff's asset rows and Reefdose's
/// verification grid.
#[component]
pub fn GateList(label: String, gates: Vec<Gate>, #[props(default)] class: String) -> Element {
    rsx! {
        ul { class: "divide-y divide-rule-soft rounded-md border border-rule-soft {class}", aria_label: "{label}",
            for gate in gates {
                GateRow { key: "{gate.label}", gate }
            }
        }
    }
}

#[component]
fn GateRow(gate: Gate) -> Element {
    let (glyph, spoken) = gate_mark(gate.verdict);
    rsx! {
        li { class: "grid grid-cols-[1.5rem_minmax(0,1fr)] gap-x-2 px-3 py-2",
            span { class: "text-center font-bold {gate.verdict.ink()}", aria_hidden: "true", "{glyph}" }
            span { class: "text-sm font-medium text-ink",
                span { class: "sr-only", "{spoken}: " }
                "{gate.label}"
            }
            if let Some(detail) = gate.detail {
                span { class: "col-start-2 text-xs text-ink-muted", "{detail}" }
            }
        }
    }
}

/// One changed field in a [`RedlineDiff`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redline {
    pub label: String,
    pub before: String,
    pub after: String,
}

impl Redline {
    pub fn new(
        label: impl Into<String>,
        before: impl Into<String>,
        after: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            before: before.into(),
            after: after.into(),
        }
    }
}

/// What an edit changed, as struck and inserted text per field.
///
/// `del` and `ins` carry the meaning for assistive technology and in a
/// printout; the colours repeat it. Drawn from Edit Scope's draft receipt.
#[component]
pub fn RedlineDiff(lines: Vec<Redline>, #[props(default)] class: String) -> Element {
    rsx! {
        dl { class: "divide-y divide-rule-soft rounded-md border border-rule-soft bg-surface {class}",
            for line in lines {
                div { key: "{line.label}", class: "grid gap-1 px-3 py-2 sm:grid-cols-[8rem_minmax(0,1fr)] sm:gap-3",
                    dt { class: "font-mono text-xs uppercase tracking-wide text-ink-muted", "{line.label}" }
                    dd { class: "grid gap-1 text-sm font-medium [overflow-wrap:anywhere]",
                        del { class: "text-fault-ink decoration-fault decoration-2", "{line.before}" }
                        ins { class: "text-takeable-ink no-underline", "{line.after}" }
                    }
                }
            }
        }
    }
}

/// One row of a [`CompareTable`]: what is compared, then one cell per option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareRow {
    pub label: String,
    pub cells: Vec<String>,
}

impl CompareRow {
    pub fn new(label: impl Into<String>, cells: Vec<String>) -> Self {
        Self {
            label: label.into(),
            cells,
        }
    }
}

/// Options side by side, a row per feature.
///
/// A real `table` on wide screens. Below `sm` each option's cells stack as a
/// card and every cell prints its column's name from `data-label`, so nothing
/// scrolls sideways and no cell loses its heading. Drawn from A Place Ahead's
/// comparison, All Source's pricing matrix and Move to Own's responsive table.
#[component]
pub fn CompareTable(
    caption: String,
    /// The header of the row-label column, then one per option.
    columns: Vec<String>,
    rows: Vec<CompareRow>,
    /// Index into the options (not counting the label column) to set apart:
    /// the recommended plan, the reader's current choice.
    #[props(default)]
    highlight: Option<usize>,
    #[props(default)] class: String,
) -> Element {
    let options: Vec<String> = columns.iter().skip(1).cloned().collect();
    let corner = columns.first().cloned().unwrap_or_default();
    let shade = move |index: usize| {
        if highlight == Some(index) {
            "bg-raised font-semibold"
        } else {
            ""
        }
    };
    rsx! {
        table { class: "w-full border-collapse text-sm text-ink {class}",
            caption { class: "pb-2 text-left text-sm font-semibold text-ink", "{caption}" }
            thead { class: "max-sm:sr-only",
                tr { class: "border-b border-rule",
                    th { scope: "col", class: "px-3 py-2 text-left font-mono text-xs uppercase tracking-wide text-ink-muted", "{corner}" }
                    for (index , option) in options.iter().enumerate() {
                        th { key: "{option}", scope: "col", class: "px-3 py-2 text-left font-semibold {shade(index)}", "{option}" }
                    }
                }
            }
            tbody {
                for row in rows {
                    tr { key: "{row.label}", class: "border-b border-rule-soft max-sm:mb-3 max-sm:block max-sm:rounded-md max-sm:border max-sm:border-rule-soft",
                        th { scope: "row", class: "px-3 py-2 text-left font-medium max-sm:block max-sm:bg-raised", "{row.label}" }
                        for (index , cell) in row.cells.iter().enumerate() {
                            td {
                                key: "{index}",
                                "data-label": options.get(index).cloned().unwrap_or_default(),
                                class: "px-3 py-2 tabular-nums max-sm:flex max-sm:justify-between max-sm:gap-4 \
                                        max-sm:before:font-mono max-sm:before:text-xs max-sm:before:uppercase \
                                        max-sm:before:text-ink-muted max-sm:before:content-[attr(data-label)] {shade(index)}",
                                "{cell}"
                            }
                        }
                    }
                }
            }
        }
    }
}
