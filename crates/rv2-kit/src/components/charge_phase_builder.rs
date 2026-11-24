//! Charge phase builder: one billing phase of a contract, with product lines, an optional discount,
//! an optional minimum, and running totals.
//!
//! Design choice: the discount applies to the recurring subtotal and is capped at it; the minimum
//! raises the phase sum after the discount. `initial_charge` is the sum of up-front product lines
//! before discount, which is what the summary reports as due on day one.
//!
//! Browser checks: menus and the calendar come from `rv2_affect` (Select, DatePicker); the remove
//! action appears on hover or focus; the line table stacks into one column below 640 px.

use dioxus::prelude::*;
use rv2_affect::date_picker::{DatePicker, Day};
use rv2_affect::select::{Select, SelectOption};

use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/charge-phase-builder.css");
const LENGTHS: [(u32, &str); 6] = [
    (1, "One month"),
    (3, "Three months"),
    (6, "Six months"),
    (12, "Twelve months"),
    (24, "Twenty-four months"),
    (36, "Thirty-six months"),
];
const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Recurrence {
    Once,
    #[default]
    Monthly,
    Quarterly,
    Yearly,
}

impl Recurrence {
    pub const ALL: [Recurrence; 4] = [Self::Once, Self::Monthly, Self::Quarterly, Self::Yearly];

    pub fn value(self) -> &'static str {
        match self {
            Self::Once => "once",
            Self::Monthly => "monthly",
            Self::Quarterly => "quarterly",
            Self::Yearly => "yearly",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Once => "Once",
            Self::Monthly => "Monthly",
            Self::Quarterly => "Quarterly",
            Self::Yearly => "Yearly",
        }
    }

    pub fn from_value(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.value() == value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Timing {
    #[default]
    UpFront,
    PeriodEnd,
}

impl Timing {
    pub fn value(self) -> &'static str {
        match self {
            Self::UpFront => "up_front",
            Self::PeriodEnd => "period_end",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::UpFront => "Charged up front",
            Self::PeriodEnd => "Charged at period end",
        }
    }

    pub fn from_value(value: &str) -> Option<Self> {
        [Self::UpFront, Self::PeriodEnd]
            .into_iter()
            .find(|t| t.value() == value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProductLine {
    pub name: String,
    pub recurrence: Recurrence,
    pub timing: Timing,
    pub amount: String,
    pub quantity: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LineKind {
    Product(ProductLine),
    /// `percent` reads the amount as a percentage of the subtotal; otherwise it is a fixed amount.
    Discount {
        percent: bool,
        amount: String,
    },
    Minimum {
        amount: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub line_id: String,
    pub kind: LineKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CatalogItem {
    pub option_name: String,
    pub recurrence: Recurrence,
    pub unit_amount: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Schedule {
    pub phase_label: String,
    pub start: String,
    pub duration: u32,
    pub lines: Vec<Line>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Totals {
    pub subtotal: f64,
    pub discount: f64,
    pub minimum: Option<f64>,
    pub phase_sum: f64,
    pub initial_charge: f64,
}

/// A non-negative amount, digits and one decimal point; thousands separators are allowed.
pub fn parse_amount(text: &str) -> Option<f64> {
    let cleaned: String = text.trim().chars().filter(|c| *c != ',').collect();
    let value: f64 = cleaned.parse().ok()?;
    (value.is_finite() && value >= 0.0).then_some(value)
}

/// Quantity counts as 1 when blank.
fn quantity(text: &str) -> Option<f64> {
    if text.trim().is_empty() {
        Some(1.0)
    } else {
        parse_amount(text)
    }
}

/// Sums and the phase total. Invalid values are left out of every sum.
pub fn totals(lines: &[Line]) -> Totals {
    let mut subtotal = 0.0;
    let mut up_front = 0.0;
    let mut discount_input = None;
    let mut minimum = None;
    for line in lines {
        match &line.kind {
            LineKind::Product(product) => {
                if let (Some(unit), Some(qty)) =
                    (parse_amount(&product.amount), quantity(&product.quantity))
                {
                    subtotal += unit * qty;
                    if product.timing == Timing::UpFront {
                        up_front += unit * qty;
                    }
                }
            }
            LineKind::Discount { percent, amount } => {
                discount_input = parse_amount(amount).map(|value| (*percent, value));
            }
            LineKind::Minimum { amount } => minimum = parse_amount(amount),
        }
    }
    let discount = match discount_input {
        Some((true, pct)) => subtotal * pct / 100.0,
        Some((false, fixed)) => fixed,
        None => 0.0,
    }
    .min(subtotal);
    let after_discount = subtotal - discount;
    let phase_sum = minimum.map_or(after_discount, |floor| after_discount.max(floor));
    Totals {
        subtotal,
        discount,
        minimum,
        phase_sum,
        initial_charge: up_front,
    }
}

/// The last day of a phase that starts on `start` and runs `months` months.
pub fn period_end(start: Day, months: u32) -> Option<Day> {
    Some(start.add_months(months as i32).add_days(-1))
}

/// Parses `yyyy-mm-dd`.
pub fn parse_iso(text: &str) -> Option<Day> {
    let mut parts = text.trim().split('-');
    let year = parts.next()?.parse().ok()?;
    let month = parts.next()?.parse().ok()?;
    let day = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Day::new(year, month, day)
}

/// "28 February 2027". The rest state shows the start in this form.
pub fn long_date(day: Day) -> String {
    format!(
        "{} {} {}",
        day.day,
        MONTH_NAMES[day.month as usize - 1],
        day.year
    )
}

/// "Twelve months" for the listed lengths; any other length reads "<n> months".
pub fn length_text(months: u32) -> String {
    LENGTHS
        .iter()
        .find(|(m, _)| *m == months)
        .map_or_else(|| format!("{months} months"), |(_, text)| text.to_string())
}

/// The start date and the length as the schedule strip shows them, from the same values as the
/// schedule row.
pub fn schedule_text(start: Option<Day>, months: u32) -> (String, String) {
    (
        start.map_or_else(|| "—".to_string(), long_date),
        length_text(months),
    )
}

/// What a line's amount is charged for, shown beside the amount.
pub fn recurrence_suffix(recurrence: Recurrence) -> &'static str {
    match recurrence {
        Recurrence::Once => "once",
        Recurrence::Monthly => "/ month",
        Recurrence::Quarterly => "/ quarter",
        Recurrence::Yearly => "/ year",
    }
}

pub fn format_money(symbol: &str, value: f64) -> String {
    format!("{symbol}{value:.2}")
}

/// Blank is allowed while typing; anything else must parse.
fn amount_invalid(text: &str) -> bool {
    parse_amount(text).is_none() && !text.trim().is_empty()
}

/// `lines` with the line `line_id` changed by `edit`; other lines keep their order and values.
fn edited(lines: Vec<Line>, line_id: &str, edit: impl Fn(&mut LineKind)) -> Vec<Line> {
    lines
        .into_iter()
        .map(|mut line| {
            if line.line_id == line_id {
                edit(&mut line.kind);
            }
            line
        })
        .collect()
}

fn notify(
    on_change: EventHandler<Schedule>,
    label: String,
    start: Option<Day>,
    duration: u32,
    lines: Vec<Line>,
) {
    on_change.call(Schedule {
        phase_label: label,
        start: start.map(|d| d.iso()).unwrap_or_default(),
        duration,
        lines,
    });
}

fn option(value: &str, label: &str) -> SelectOption {
    SelectOption {
        value: value.to_string(),
        label: label.to_string(),
        description: None,
        icon: None,
        disabled: false,
    }
}

fn length_options() -> Vec<SelectOption> {
    LENGTHS
        .iter()
        .map(|(months, text)| option(&months.to_string(), text))
        .collect()
}

fn recurrence_options() -> Vec<SelectOption> {
    Recurrence::ALL
        .iter()
        .map(|r| option(r.value(), r.label()))
        .collect()
}

fn timing_options() -> Vec<SelectOption> {
    [Timing::UpFront, Timing::PeriodEnd]
        .iter()
        .map(|t| option(t.value(), t.label()))
        .collect()
}

fn unit_options() -> Vec<SelectOption> {
    vec![option("percent", "%"), option("fixed", "Fixed")]
}

/// One line of the table. Each edit writes the whole list back, then reports the schedule.
#[component]
fn LineRow(
    line: Line,
    currency_symbol: String,
    lines: Signal<Vec<Line>>,
    label: Signal<String>,
    start: Signal<Option<Day>>,
    duration: Signal<u32>,
    on_schedule_change: EventHandler<Schedule>,
) -> Element {
    let line_id = line.line_id.clone();
    let mut commit = move |next: Vec<Line>| {
        lines.set(next.clone());
        notify(on_schedule_change, label(), start(), duration(), next);
    };
    let remove_id = line_id.clone();
    let remove = move |_: MouseEvent| {
        commit(
            lines()
                .into_iter()
                .filter(|l| l.line_id != remove_id)
                .collect(),
        );
    };

    match &line.kind {
        LineKind::Product(product) => {
            let name = product.name.clone();
            let recurrence = product.recurrence;
            let timing = product.timing;
            let amount = product.amount.clone();
            let invalid = amount_invalid(&amount);
            let suffix = recurrence_suffix(recurrence);
            let remove_label = format!("Remove {name}");
            let rec_id = line_id.clone();
            let timing_id = line_id.clone();
            let amount_id = line_id.clone();
            rsx! {
                tr {
                    td { "{name}" }
                    td { "data-label": "Recurrence",
                        Select {
                            class: "kit-cpb__control",
                            options: recurrence_options(),
                            value: recurrence.value().to_string(),
                            aria_label: format!("Recurrence for {name}"),
                            onchange: move |option: SelectOption| {
                                if let Some(next) = Recurrence::from_value(&option.value) {
                                    commit(edited(lines(), &rec_id, |kind| {
                                        if let LineKind::Product(p) = kind {
                                            p.recurrence = next;
                                        }
                                    }));
                                }
                            },
                        }
                    }
                    td { class: "kit-cpb__money", "data-label": "Amount",
                        div { class: "kit-cpb__amount",
                            span { class: "kit-cpb__prefix", "aria-hidden": "true", "{currency_symbol}" }
                            input {
                                r#type: "text",
                                inputmode: "decimal",
                                "aria-label": "Amount for {name}",
                                "aria-invalid": if invalid { "true" } else { "false" },
                                value: "{amount}",
                                oninput: move |event: FormEvent| {
                                    let text = event.value();
                                    commit(edited(lines(), &amount_id, |kind| {
                                        if let LineKind::Product(p) = kind {
                                            p.amount = text.clone();
                                        }
                                    }));
                                },
                            }
                            span { class: "kit-cpb__suffix", "{suffix}" }
                        }
                        div { class: "kit-cpb__under",
                            Select {
                                class: "kit-cpb__control",
                                options: timing_options(),
                                value: timing.value().to_string(),
                                aria_label: format!("Timing for {name}"),
                                onchange: move |option: SelectOption| {
                                    if let Some(next) = Timing::from_value(&option.value) {
                                        commit(edited(lines(), &timing_id, |kind| {
                                            if let LineKind::Product(p) = kind {
                                                p.timing = next;
                                            }
                                        }));
                                    }
                                },
                            }
                        }
                        if invalid {
                            span { class: "kit-cpb__error", role: "alert", "Enter a number" }
                        }
                    }
                    td {
                        button { r#type: "button", class: "kit-cpb__remove", "aria-label": "{remove_label}", onclick: remove, "Remove" }
                    }
                }
            }
        }
        LineKind::Discount { percent, amount } => {
            let unit = if *percent { "percent" } else { "fixed" };
            let amount = amount.clone();
            let invalid = amount_invalid(&amount);
            let amount_id = line_id.clone();
            let unit_id = line_id.clone();
            rsx! {
                tr {
                    td { "Discount" }
                    td { class: "kit-cpb__dash", "data-label": "Recurrence", "—" }
                    td { class: "kit-cpb__money", "data-label": "Amount",
                        div { class: "kit-cpb__amount",
                            input {
                                r#type: "text",
                                inputmode: "decimal",
                                "aria-label": "Discount amount",
                                "aria-invalid": if invalid { "true" } else { "false" },
                                value: "{amount}",
                                oninput: move |event: FormEvent| {
                                    let text = event.value();
                                    commit(edited(lines(), &amount_id, |kind| {
                                        if let LineKind::Discount { amount: value, .. } = kind {
                                            *value = text.clone();
                                        }
                                    }));
                                },
                            }
                        }
                        div { class: "kit-cpb__under",
                            Select {
                                class: "kit-cpb__control",
                                options: unit_options(),
                                value: unit.to_string(),
                                aria_label: "Discount unit",
                                onchange: move |option: SelectOption| {
                                    let is_percent = option.value == "percent";
                                    commit(edited(lines(), &unit_id, |kind| {
                                        if let LineKind::Discount { percent: flag, .. } = kind {
                                            *flag = is_percent;
                                        }
                                    }));
                                },
                            }
                        }
                        if invalid {
                            span { class: "kit-cpb__error", role: "alert", "Enter a number" }
                        }
                    }
                    td {
                        button { r#type: "button", class: "kit-cpb__remove", "aria-label": "Remove Discount", onclick: remove, "Remove" }
                    }
                }
            }
        }
        LineKind::Minimum { amount } => {
            let amount = amount.clone();
            let invalid = amount_invalid(&amount);
            let amount_id = line_id.clone();
            rsx! {
                tr {
                    td { "Minimum" }
                    td { class: "kit-cpb__dash", "data-label": "Recurrence", "—" }
                    td { class: "kit-cpb__money", "data-label": "Amount",
                        div { class: "kit-cpb__amount",
                            span { class: "kit-cpb__prefix", "aria-hidden": "true", "{currency_symbol}" }
                            input {
                                r#type: "text",
                                inputmode: "decimal",
                                "aria-label": "Minimum amount",
                                "aria-invalid": if invalid { "true" } else { "false" },
                                value: "{amount}",
                                oninput: move |event: FormEvent| {
                                    let text = event.value();
                                    commit(edited(lines(), &amount_id, |kind| {
                                        if let LineKind::Minimum { amount: value } = kind {
                                            *value = text.clone();
                                        }
                                    }));
                                },
                            }
                        }
                        if invalid {
                            span { class: "kit-cpb__error", role: "alert", "Enter a number" }
                        }
                    }
                    td {
                        button { r#type: "button", class: "kit-cpb__remove", "aria-label": "Remove Minimum", onclick: remove, "Remove" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ChargePhaseBuilder(
    #[props(default = "Phase schedule".to_string(), into)] heading_text: String,
    #[props(default = "Phase 1".to_string(), into)] phase_label: String,
    #[props(default = "$".to_string(), into)] currency_symbol: String,
    #[props(default)] item_options: Vec<CatalogItem>,
    #[props(default)] starting_lines: Vec<Line>,
    #[props(default, into)] starting_date: String,
    #[props(default)] on_schedule_change: EventHandler<Schedule>,
    #[props(default, into)] root_class: String,
) -> Element {
    let mut label = use_signal(|| phase_label.clone());
    let mut start = use_signal(|| parse_iso(&starting_date));
    let mut duration = use_signal(|| 12u32);
    let mut lines = use_signal(|| starting_lines.clone());
    let mut next_line = use_signal(|| starting_lines.len());

    let current_lines = lines();
    let sums = totals(&current_lines);
    let (start_text, length_text) = schedule_text(start(), duration());
    let initial_text = format_money(&currency_symbol, sums.initial_charge);
    let sum_text = format_money(&currency_symbol, sums.phase_sum);
    let catalog: Vec<SelectOption> = item_options
        .iter()
        .enumerate()
        .map(|(i, item)| option(&i.to_string(), &item.option_name))
        .collect();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class_list("kit-cpb", &root_class), "aria-label": "{heading_text}",
            div { class: "kit-cpb__head",
                h3 { class: "kit-cpb__heading", "{heading_text}" }
            }
            div { class: "kit-cpb__strip",
                label { class: "kit-cpb__name-field",
                    span { class: "kit-cpb__caption", "Phase name" }
                    input {
                        r#type: "text",
                        value: "{label}",
                        oninput: move |event: FormEvent| {
                            let text = event.value();
                            label.set(text.clone());
                            notify(on_schedule_change, text, start(), duration(), lines());
                        },
                    }
                }
                div { class: "kit-cpb__readout",
                    span { class: "kit-cpb__read", "{start_text}" }
                    svg { class: "kit-cpb__arrow", view_box: "0 0 16 16", width: "14", height: "14", fill: "none", stroke: "currentColor", "aria-hidden": "true",
                        path { d: "M3 8h10M9 4l4 4-4 4" }
                    }
                    span { class: "kit-cpb__read", "{length_text}" }
                }
            }
            div { class: "kit-cpb__phase",
                div { class: "kit-cpb__field",
                    span { class: "kit-cpb__label",
                        svg { class: "kit-cpb__glyph", view_box: "0 0 16 16", width: "14", height: "14", fill: "none", stroke: "currentColor", "aria-hidden": "true",
                            path { d: "M3 4h10v9H3zM3 7h10M6 2v3M10 2v3" }
                        }
                        "Start date"
                    }
                    DatePicker {
                        class: "kit-cpb__control",
                        value: start(),
                        aria_label: "Start date",
                        onchange: move |day: Day| {
                            start.set(Some(day));
                            notify(on_schedule_change, label(), Some(day), duration(), lines());
                        },
                    }
                }
                div { class: "kit-cpb__field",
                    span { class: "kit-cpb__label", "Length" }
                    Select {
                        class: "kit-cpb__control",
                        options: length_options(),
                        value: duration().to_string(),
                        name: "phase-length",
                        aria_label: "Length",
                        onchange: move |option: SelectOption| {
                            let months = option.value.parse::<u32>().unwrap_or(12);
                            duration.set(months);
                            notify(on_schedule_change, label(), start(), months, lines());
                        },
                    }
                }
            }
            table { class: "kit-cpb__lines", role: "table",
                thead {
                    tr {
                        th { scope: "col", "Name" }
                        th { scope: "col", "Recurrence" }
                        th { scope: "col", "Amount" }
                        th { scope: "col" }
                    }
                }
                tbody {
                    for line in current_lines.iter() {
                        LineRow {
                            key: "{line.line_id}",
                            line: line.clone(),
                            currency_symbol: currency_symbol.clone(),
                            lines,
                            label,
                            start,
                            duration,
                            on_schedule_change,
                        }
                    }
                }
            }
            if current_lines.is_empty() {
                p { class: "kit-cpb__empty", "No lines yet." }
            }
            div { class: "kit-cpb__adds",
                div { class: "kit-cpb__picker",
                    Select {
                        class: "kit-cpb__control",
                        options: catalog,
                        placeholder: "Include item",
                        aria_label: "Include item",
                        onchange: move |option: SelectOption| {
                            let index: usize = option.value.parse().unwrap_or(0);
                            if let Some(item) = item_options.get(index) {
                                let id = next_line();
                                next_line.set(id + 1);
                                let product = ProductLine {
                                    name: item.option_name.clone(),
                                    recurrence: item.recurrence,
                                    timing: Timing::UpFront,
                                    amount: format!("{:.2}", item.unit_amount),
                                    quantity: "1".to_string(),
                                };
                                let mut next = lines();
                                next.push(Line { line_id: id.to_string(), kind: LineKind::Product(product) });
                                lines.set(next.clone());
                                notify(on_schedule_change, label(), start(), duration(), next);
                            }
                        },
                    }
                }
                div { class: "kit-cpb__add-buttons",
                    button {
                        r#type: "button",
                        onclick: move |_| {
                            let id = next_line();
                            next_line.set(id + 1);
                            let mut next = lines();
                            next.push(Line { line_id: id.to_string(), kind: LineKind::Discount { percent: true, amount: "10".to_string() } });
                            lines.set(next.clone());
                            notify(on_schedule_change, label(), start(), duration(), next);
                        },
                        "Include adjustment"
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| {
                            let id = next_line();
                            next_line.set(id + 1);
                            let mut next = lines();
                            next.push(Line { line_id: id.to_string(), kind: LineKind::Minimum { amount: "0".to_string() } });
                            lines.set(next.clone());
                            notify(on_schedule_change, label(), start(), duration(), next);
                        },
                        "Include floor"
                    }
                }
            }
            dl { class: "kit-cpb__summary", role: "status", "aria-live": "polite",
                div { dt { "First invoice" } dd { "{initial_text}" } }
                div { dt { "Phase total" } dd { "{sum_text}" } }
            }
        }
    }
}
