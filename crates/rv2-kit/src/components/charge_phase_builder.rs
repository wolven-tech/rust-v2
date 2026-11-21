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

pub fn format_money(symbol: &str, value: f64) -> String {
    format!("{symbol}{value:.2}")
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
    let end = start().and_then(|s| period_end(s, duration()));
    let end_text = end.map(|d| d.iso()).unwrap_or_else(|| "—".to_string());
    let initial_text = format_money(&currency_symbol, sums.initial_charge);
    let sum_text = format_money(&currency_symbol, sums.phase_sum);
    let catalog: Vec<SelectOption> = item_options
        .iter()
        .enumerate()
        .map(|(i, item)| SelectOption {
            value: i.to_string(),
            label: item.option_name.clone(),
            description: None,
            icon: None,
            disabled: false,
        })
        .collect();
    let length_options: Vec<SelectOption> = LENGTHS
        .iter()
        .map(|(months, text)| SelectOption {
            value: months.to_string(),
            label: text.to_string(),
            description: None,
            icon: None,
            disabled: false,
        })
        .collect();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class_list("kit-cpb", &root_class), "aria-label": "{heading_text}",
            h3 { class: "kit-cpb__heading", "{heading_text}" }
            div { class: "kit-cpb__phase",
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
                div { class: "kit-cpb__field",
                    span { "Start" }
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
                span { class: "kit-cpb__end", "Ends {end_text}" }
            }
            table { class: "kit-cpb__lines", role: "table",
                thead {
                    tr {
                        th { scope: "col", "Name" }
                        th { scope: "col", "Recurrence" }
                        th { scope: "col", "Amount" }
                    }
                }
                tbody {
                    for line in current_lines.iter() {
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
                                            let edit_id = line_id.clone();
                                            let edit_id2 = line_id.clone();
                                            rsx! {
                                                td {
                                                    input {
                                                        r#type: "text",
                                                        "aria-label": "Product name",
                                                        value: "{name}",
                                                        oninput: move |event| {
                                                            let text = event.value();
                                                            lines.with_mut(|ls| for l in ls.iter_mut().filter(|l| l.line_id == edit_id) {
                                                                if let LineKind::Product(p) = &mut l.kind { p.name = text.clone(); }
                                                            });
                                                            notify(on_schedule_change, label(), start(), duration(), lines());
                                                        },
                                                    }
                                                }
                                                td {
                                                    Select {
                                                        options: Recurrence::ALL.iter().map(|r| SelectOption { value: r.value().to_string(), label: r.label().to_string(), description: None, icon: None, disabled: false }).collect::<Vec<_>>(),
                                                        value: rec_value.to_string(),
                                                        onchange: move |option: SelectOption| {
                                                            if let Some(rec) = Recurrence::from_value(&option.value) {
                                                                lines.with_mut(|ls| for l in ls.iter_mut().filter(|l| l.line_id == edit_id2) {
                                                                    if let LineKind::Product(p) = &mut l.kind { p.recurrence = rec; }
                                                                });
                                                                notify(on_schedule_change, label(), start(), duration(), lines());
                                                            }
                                                        },
                                                    }
                                                }
                                                td {
                                                    input {
                                                        r#type: "text",
                                                        inputmode: "decimal",
                                                        "aria-label": "Amount",
                                                        "aria-invalid": if amount_error.is_some() { "true" } else { "false" },
                                                        value: "{amount}",
                                                        oninput: {
                                                            let amount_id = line_id.clone();
                                                            move |event| {
                                                            let text = event.value();
                                                            lines.with_mut(|ls| for l in ls.iter_mut().filter(|l| l.line_id == amount_id) {
                                                                if let LineKind::Product(p) = &mut l.kind { p.amount = text.clone(); }
                                                            });
                                                            notify(on_schedule_change, label(), start(), duration(), lines());
                                                        }
                                                        },
                                                    }
                                                    if let Some(message) = amount_error {
                                                        span { class: "kit-cpb__error", role: "alert", "{message}" }
                                                    }
                                                }
                                                td { button { r#type: "button", class: "kit-cpb__remove", onclick: remove_line, "Remove" } }
                                            }
                                        }
                                        LineKind::Discount { percent, amount } => {
                                            let value = amount.clone();
                                            let suffix = if *percent { "%" } else { "" };
                                            let edit_id = line_id.clone();
                                            rsx! {
                                                td { "Discount" }
                                                td { "—" }
                                                td {
                                                    input {
                                                        r#type: "text",
                                                        "aria-label": "Discount amount",
                                                        value: "{value}",
                                                        oninput: move |event| {
                                                            let text = event.value();
                                                            lines.with_mut(|ls| for l in ls.iter_mut().filter(|l| l.line_id == edit_id) {
                                                                if let LineKind::Discount { amount, .. } = &mut l.kind { *amount = text.clone(); }
                                                            });
                                                            notify(on_schedule_change, label(), start(), duration(), lines());
                                                        },
                                                    }
                                                    span { "{suffix}" }
                                                }
                                                td { button { r#type: "button", class: "kit-cpb__remove", onclick: remove_line, "Remove" } }
                                            }
                                        }
                                        LineKind::Minimum { amount } => {
                                            let value = amount.clone();
                                            let edit_id = line_id.clone();
                                            rsx! {
                                                td { "Minimum" }
                                                td { "—" }
                                                td {
                                                    input {
                                                        r#type: "text",
                                                        "aria-label": "Minimum amount",
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
                                                td { button { r#type: "button", class: "kit-cpb__remove", onclick: remove_line, "Remove" } }
                                            }
                                        }
                                    };
                                    rsx! { tr { key: "{row_key}", {cells} } }
                        }
                    }
                }
            }
            if current_lines.is_empty() {
                p { class: "kit-cpb__empty", "No lines yet." }
            }
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
            dl { class: "kit-cpb__summary", role: "status", "aria-live": "polite",
                div { dt { "Initial charge" } dd { "{initial_text}" } }
                div { dt { "Phase end" } dd { "{end_text}" } }
                div { dt { "Phase sum" } dd { "{sum_text}" } }
            }
        }
    }
}
