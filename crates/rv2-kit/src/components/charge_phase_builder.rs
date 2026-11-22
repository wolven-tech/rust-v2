//! Charge phase builder: one billing phase of a contract, with product lines, an optional discount,
//! an optional minimum, and running totals.
//!
//! Design choice: the discount applies to the recurring subtotal and is capped at it; the minimum
//! raises the phase sum after the discount. `initial_charge` is the sum of up-front product lines
//! before discount, which is what the summary reports as due on day one.
//!
//! Browser checks: menus and the calendar come from `rv2_affect` (Select, DatePicker); the end
//! date fades when the length changes; no horizontal scroll at phone width.

use dioxus::prelude::*;
use rv2_affect::date_picker::{DatePicker, Day};
use rv2_affect::select::{Select, SelectOption};

use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/charge-phase-builder.css");
const LENGTHS: [(u32, &str); 6] = [
    (1, "1 month"),
    (3, "3 months"),
    (6, "6 months"),
    (12, "12 months"),
    (24, "24 months"),
    (36, "36 months"),
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

/// "28 February 2027". The rest state shows the start and the end in this form.
pub fn long_date(day: Day) -> String {
    format!(
        "{} {} {}",
        day.day,
        MONTH_NAMES[day.month as usize - 1],
        day.year
    )
}

pub fn format_money(symbol: &str, value: f64) -> String {
    format!("{symbol}{value:.2}")
}

fn money_text(symbol: &str, amount: &str) -> String {
    parse_amount(amount).map_or_else(|| amount.trim().to_string(), |v| format_money(symbol, v))
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

/// One line at rest: its values as text.
#[component]
fn ReadOnlyLine(line: Line, currency_symbol: String) -> Element {
    let (name, recurrence, timing, amount) = match &line.kind {
        LineKind::Product(p) => (
            p.name.clone(),
            p.recurrence.label().to_string(),
            p.timing.label().to_string(),
            money_text(&currency_symbol, &p.amount),
        ),
        LineKind::Discount { percent, amount } => (
            "Discount".to_string(),
            "—".to_string(),
            "—".to_string(),
            if *percent {
                format!("{}%", amount.trim())
            } else {
                money_text(&currency_symbol, amount)
            },
        ),
        LineKind::Minimum { amount } => (
            "Minimum".to_string(),
            "—".to_string(),
            "—".to_string(),
            money_text(&currency_symbol, amount),
        ),
    };
    rsx! {
        tr {
            td {
                "{name}"
                span { class: "kit-cpb__inline-recur", "{recurrence}" }
            }
            td { "{recurrence}" }
            td { "{timing}" }
            td { "{amount}" }
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
    #[props(default)] start_editing: bool,
    #[props(default)] on_schedule_change: EventHandler<Schedule>,
    #[props(default, into)] root_class: String,
) -> Element {
    let mut label = use_signal(|| phase_label.clone());
    let mut start = use_signal(|| parse_iso(&starting_date));
    let mut duration = use_signal(|| 12u32);
    let mut lines = use_signal(|| starting_lines.clone());
    let mut next_line = use_signal(|| starting_lines.len());
    let mut editing = use_signal(|| start_editing);

    let current_lines = lines();
    let is_editing = editing();
    let sums = totals(&current_lines);
    let end = start().and_then(|s| period_end(s, duration()));
    let end_text = end.map(long_date).unwrap_or_else(|| "—".to_string());
    let start_text = start().map(long_date).unwrap_or_else(|| "—".to_string());
    let length_text = LENGTHS
        .iter()
        .find(|(months, _)| *months == duration())
        .map_or_else(
            || format!("{} months", duration()),
            |(_, text)| text.to_string(),
        );
    let initial_text = format_money(&currency_symbol, sums.initial_charge);
    let sum_text = format_money(&currency_symbol, sums.phase_sum);
    let catalog: Vec<SelectOption> = item_options
        .iter()
        .enumerate()
        .map(|(i, item)| option(&i.to_string(), &item.option_name))
        .collect();
    let length_options: Vec<SelectOption> = LENGTHS
        .iter()
        .map(|(months, text)| option(&months.to_string(), text))
        .collect();
    let recurrence_options: Vec<SelectOption> = Recurrence::ALL
        .iter()
        .map(|r| option(r.value(), r.label()))
        .collect();
    let timing_options: Vec<SelectOption> = [Timing::UpFront, Timing::PeriodEnd]
        .iter()
        .map(|t| option(t.value(), t.label()))
        .collect();
    let unit_options = vec![option("percent", "%"), option("fixed", "Fixed")];

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class_list("kit-cpb", &root_class), "aria-label": "{heading_text}",
            div { class: "kit-cpb__head",
                h3 { class: "kit-cpb__heading", "{heading_text}" }
                if is_editing {
                    label { class: "kit-cpb__field",
                        span { "Phase name" }
                        input {
                            r#type: "text",
                            value: "{label}",
                            oninput: move |event| {
                                label.set(event.value());
                                notify(on_schedule_change, event.value(), start(), duration(), lines());
                            },
                        }
                    }
                } else {
                    p { class: "kit-cpb__name", "{label}" }
                }
            }
            div { class: "kit-cpb__phase",
                if is_editing {
                    div { class: "kit-cpb__field",
                        span { class: "kit-cpb__label",
                            svg { class: "kit-cpb__glyph", view_box: "0 0 16 16", width: "14", height: "14", fill: "none", stroke: "currentColor", "aria-hidden": "true",
                                path { d: "M3 4h10v9H3zM3 7h10M6 2v3M10 2v3" }
                            }
                            "Start"
                        }
                        DatePicker {
                            value: start(),
                            onchange: move |day: Day| {
                                start.set(Some(day));
                                notify(on_schedule_change, label(), Some(day), duration(), lines());
                            },
                        }
                    }
                    div { class: "kit-cpb__field",
                        span { "Length" }
                        Select {
                            options: length_options,
                            value: duration().to_string(),
                            name: "phase-length",
                            onchange: move |option: SelectOption| {
                                let months = option.value.parse::<u32>().unwrap_or(12);
                                duration.set(months);
                                notify(on_schedule_change, label(), start(), months, lines());
                            },
                        }
                    }
                } else {
                    div { class: "kit-cpb__field",
                        span { class: "kit-cpb__label",
                            svg { class: "kit-cpb__glyph", view_box: "0 0 16 16", width: "14", height: "14", fill: "none", stroke: "currentColor", "aria-hidden": "true",
                                path { d: "M3 4h10v9H3zM3 7h10M6 2v3M10 2v3" }
                            }
                            "Start"
                        }
                        span { class: "kit-cpb__value", "{start_text}" }
                    }
                    div { class: "kit-cpb__field",
                        span { "Length" }
                        span { class: "kit-cpb__value", "{length_text}" }
                    }
                }
                span { class: "kit-cpb__end", "Ends {end_text}" }
            }
            table { class: "kit-cpb__lines", role: "table",
                thead {
                    tr {
                        th { scope: "col", "Name" }
                        th { scope: "col", "Recurrence" }
                        th { scope: "col", "Timing" }
                        th { scope: "col", "Amount" }
                        if is_editing {
                            th { scope: "col" }
                        }
                    }
                }
                tbody {
                    for line in current_lines.iter() {
                        if is_editing {
                            {
                                let line_id = line.line_id.clone();
                                let row_key = line_id.clone();
                                let amount_ok = match &line.kind {
                                    LineKind::Product(p) => parse_amount(&p.amount).is_some() || p.amount.trim().is_empty(),
                                    LineKind::Discount { amount, .. } | LineKind::Minimum { amount } => parse_amount(amount).is_some() || amount.trim().is_empty(),
                                };
                                let amount_error = (!amount_ok).then_some("Enter a number");
                                let remove_id = line_id.clone();
                                let remove_line = move |_| {
                                    let next: Vec<Line> = lines().into_iter().filter(|l| l.line_id != remove_id).collect();
                                    lines.set(next.clone());
                                    notify(on_schedule_change, label(), start(), duration(), next);
                                };
                                let cells: Element = match &line.kind {
                                    LineKind::Product(product) => {
                                        let amount = product.amount.clone();
                                        let name = product.name.clone();
                                        let rec_value = product.recurrence.value();
                                        let rec_label = product.recurrence.label();
                                        let timing_value = product.timing.value();
                                        let remove_label = format!("Remove {name}");
                                        let name_id = line_id.clone();
                                        let rec_id = line_id.clone();
                                        let timing_id = line_id.clone();
                                        let amount_id = line_id.clone();
                                        rsx! {
                                            td {
                                                input {
                                                    r#type: "text",
                                                    "aria-label": "Product name",
                                                    value: "{name}",
                                                    oninput: move |event| {
                                                        let text = event.value();
                                                        lines.with_mut(|ls| for l in ls.iter_mut().filter(|l| l.line_id == name_id) {
                                                            if let LineKind::Product(p) = &mut l.kind { p.name = text.clone(); }
                                                        });
                                                        notify(on_schedule_change, label(), start(), duration(), lines());
                                                    },
                                                }
                                                span { class: "kit-cpb__inline-recur", "{rec_label}" }
                                            }
                                            td { class: "kit-cpb__recur",
                                                Select {
                                                    options: recurrence_options.clone(),
                                                    value: rec_value.to_string(),
                                                    onchange: move |option: SelectOption| {
                                                        if let Some(rec) = Recurrence::from_value(&option.value) {
                                                            lines.with_mut(|ls| for l in ls.iter_mut().filter(|l| l.line_id == rec_id) {
                                                                if let LineKind::Product(p) = &mut l.kind { p.recurrence = rec; }
                                                            });
                                                            notify(on_schedule_change, label(), start(), duration(), lines());
                                                        }
                                                    },
                                                }
                                            }
                                            td {
                                                Select {
                                                    options: timing_options.clone(),
                                                    value: timing_value.to_string(),
                                                    onchange: move |option: SelectOption| {
                                                        if let Some(timing) = Timing::from_value(&option.value) {
                                                            lines.with_mut(|ls| for l in ls.iter_mut().filter(|l| l.line_id == timing_id) {
                                                                if let LineKind::Product(p) = &mut l.kind { p.timing = timing; }
                                                            });
                                                            notify(on_schedule_change, label(), start(), duration(), lines());
                                                        }
                                                    },
                                                }
                                            }
                                            td {
                                                div { class: "kit-cpb__amount",
                                                    span { class: "kit-cpb__prefix", "aria-hidden": "true", "{currency_symbol}" }
                                                    input {
                                                        r#type: "text",
                                                        inputmode: "decimal",
                                                        "aria-label": "Amount",
                                                        "aria-invalid": if amount_error.is_some() { "true" } else { "false" },
                                                        value: "{amount}",
                                                        oninput: move |event| {
                                                            let text = event.value();
                                                            lines.with_mut(|ls| for l in ls.iter_mut().filter(|l| l.line_id == amount_id) {
                                                                if let LineKind::Product(p) = &mut l.kind { p.amount = text.clone(); }
                                                            });
                                                            notify(on_schedule_change, label(), start(), duration(), lines());
                                                        },
                                                    }
                                                }
                                                if let Some(message) = amount_error {
                                                    span { class: "kit-cpb__error", role: "alert", "{message}" }
                                                }
                                            }
                                            td { button { r#type: "button", class: "kit-cpb__remove", "aria-label": "{remove_label}", onclick: remove_line, "Remove" } }
                                        }
                                    }
                                    LineKind::Discount { percent, amount } => {
                                        let value = amount.clone();
                                        let unit_value = if *percent { "percent" } else { "fixed" };
                                        let edit_id = line_id.clone();
                                        let unit_id = line_id.clone();
                                        rsx! {
                                            td {
                                                input { r#type: "text", "aria-label": "Discount name", value: "Discount", readonly: true }
                                            }
                                            td { class: "kit-cpb__dash", "—" }
                                            td { class: "kit-cpb__dash", "—" }
                                            td {
                                                div { class: "kit-cpb__amount",
                                                    input {
                                                        r#type: "text",
                                                        inputmode: "decimal",
                                                        "aria-label": "Discount amount",
                                                        "aria-invalid": if amount_error.is_some() { "true" } else { "false" },
                                                        value: "{value}",
                                                        oninput: move |event| {
                                                            let text = event.value();
                                                            lines.with_mut(|ls| for l in ls.iter_mut().filter(|l| l.line_id == edit_id) {
                                                                if let LineKind::Discount { amount, .. } = &mut l.kind { *amount = text.clone(); }
                                                            });
                                                            notify(on_schedule_change, label(), start(), duration(), lines());
                                                        },
                                                    }
                                                }
                                                Select {
                                                    options: unit_options.clone(),
                                                    value: unit_value.to_string(),
                                                    onchange: move |option: SelectOption| {
                                                        let is_percent = option.value == "percent";
                                                        lines.with_mut(|ls| for l in ls.iter_mut().filter(|l| l.line_id == unit_id) {
                                                            if let LineKind::Discount { percent, .. } = &mut l.kind { *percent = is_percent; }
                                                        });
                                                        notify(on_schedule_change, label(), start(), duration(), lines());
                                                    },
                                                }
                                                if let Some(message) = amount_error {
                                                    span { class: "kit-cpb__error", role: "alert", "{message}" }
                                                }
                                            }
                                            td { button { r#type: "button", class: "kit-cpb__remove", "aria-label": "Remove Discount", onclick: remove_line, "Remove" } }
                                        }
                                    }
                                    LineKind::Minimum { amount } => {
                                        let value = amount.clone();
                                        let edit_id = line_id.clone();
                                        rsx! {
                                            td {
                                                input { r#type: "text", "aria-label": "Minimum name", value: "Minimum", readonly: true }
                                            }
                                            td { class: "kit-cpb__dash", "—" }
                                            td { class: "kit-cpb__dash", "—" }
                                            td {
                                                div { class: "kit-cpb__amount",
                                                    span { class: "kit-cpb__prefix", "aria-hidden": "true", "{currency_symbol}" }
                                                    input {
                                                        r#type: "text",
                                                        inputmode: "decimal",
                                                        "aria-label": "Minimum amount",
                                                        "aria-invalid": if amount_error.is_some() { "true" } else { "false" },
                                                        value: "{value}",
                                                        oninput: move |event| {
                                                            let text = event.value();
                                                            lines.with_mut(|ls| for l in ls.iter_mut().filter(|l| l.line_id == edit_id) {
                                                                if let LineKind::Minimum { amount } = &mut l.kind { *amount = text.clone(); }
                                                            });
                                                            notify(on_schedule_change, label(), start(), duration(), lines());
                                                        },
                                                    }
                                                }
                                                if let Some(message) = amount_error {
                                                    span { class: "kit-cpb__error", role: "alert", "{message}" }
                                                }
                                            }
                                            td { button { r#type: "button", class: "kit-cpb__remove", "aria-label": "Remove Minimum", onclick: remove_line, "Remove" } }
                                        }
                                    }
                                };
                                rsx! { tr { key: "{row_key}", {cells} } }
                            }
                        } else {
                            ReadOnlyLine {
                                key: "{line.line_id}",
                                line: line.clone(),
                                currency_symbol: currency_symbol.clone(),
                            }
                        }
                    }
                }
            }
            if current_lines.is_empty() {
                p { class: "kit-cpb__empty", "No lines yet." }
            }
            if is_editing {
                div { class: "kit-cpb__adds",
                    Select {
                        options: catalog,
                        placeholder: "Include item",
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
            div { class: "kit-cpb__action",
                if is_editing {
                    button { r#type: "button", onclick: move |_| editing.set(false), "Stop editing" }
                } else {
                    button { r#type: "button", onclick: move |_| editing.set(true), "Edit phase" }
                }
            }
            dl { class: "kit-cpb__summary", role: "status", "aria-live": "polite",
                div { dt { "Initial charge" } dd { "{initial_text}" } }
                div { dt { "Phase end" } dd { "{end_text}" } }
                div { dt { "Phase sum" } dd { "{sum_text}" } }
            }
        }
    }
}
