//! Invoice and credit pair: an invoice and the credit note that reverses part of it, as two
//! sheets of one document. Lines are picked on the credit side, each with an amount that
//! cannot exceed the line total.
//!
//! Design choice: the credit draft is local until the host stores it; `on_credit_change` reports
//! every change. An issued credit note is read-only and comes from `credit_document`.
//! Design choice: an amount over the line total is kept as typed, marked invalid and left out of
//! the total; issue stays disabled until every picked amount is valid.
//! Design choice: the amount field commits on Enter or blur and discards on Escape.
//! Design choice: the sheets swap by a CSS transform; the back sheet stays in the tree as
//! `aria-hidden` and `inert`, and its top edge shows above the front sheet.
//! Browser checks: side switch slides, picking a line adds its amount, invalid amount shows an
//! error and drops from the total, Escape restores, credit action pre-picks, issue gating,
//! issued note read-only, phone stacking.

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonSize, ButtonVariant};
use rv2_affect::segmented_control::{SegmentOption, SegmentedControl};

use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/invoice-credit-pair.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Fixed,
    Metered,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvoiceStatus {
    Open,
    Paid,
    Credited,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreditStatus {
    Draft,
    Issued,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Invoice,
    Credit,
}

impl Side {
    fn as_str(self) -> &'static str {
        match self {
            Self::Invoice => "invoice",
            Self::Credit => "credit",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Party {
    pub name: String,
    pub address: Vec<String>,
    pub registration_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub id: String,
    pub name: String,
    pub quantity: f64,
    pub unit_amount: f64,
    pub tax_percent: f64,
    pub total: f64,
    pub kind: LineKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Invoice {
    pub number: String,
    pub issued_on: String,
    pub due_on: String,
    pub period_from: String,
    pub period_to: String,
    pub currency: String,
    pub issuer: Party,
    pub recipient: Party,
    pub lines: Vec<Line>,
    pub tax_percent: Option<f64>,
    pub status: InvoiceStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreditedLine {
    pub line_id: String,
    pub amount: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreditDocument {
    pub number: String,
    pub reason: Option<String>,
    pub credited: Vec<CreditedLine>,
    pub status: CreditStatus,
}

/// Default money format: two decimals and the currency code.
pub fn default_money(amount: f64, currency: &str) -> String {
    format!("{currency} {amount:.2}")
}

/// An amount is valid when it is positive and does not exceed the line total.
pub fn is_valid_amount(amount: f64, line_total: f64) -> bool {
    amount > 0.0 && amount <= line_total + f64::EPSILON
}

/// Sum of the credited amounts that are valid for their line.
pub fn credit_total(credited: &[CreditedLine], lines: &[Line]) -> f64 {
    credited
        .iter()
        .filter(|c| {
            lines
                .iter()
                .find(|l| l.id == c.line_id)
                .is_some_and(|l| is_valid_amount(c.amount, l.total))
        })
        .fold(0.0, |total, c| total + c.amount)
}

/// Every line picked at its full amount.
pub fn full_credit(lines: &[Line]) -> Vec<CreditedLine> {
    lines
        .iter()
        .map(|l| CreditedLine {
            line_id: l.id.clone(),
            amount: l.total,
        })
        .collect()
}

/// Issue is allowed with at least one line picked and no invalid amount.
pub fn can_issue(credited: &[CreditedLine], lines: &[Line]) -> bool {
    !credited.is_empty()
        && credited.iter().all(|c| {
            lines
                .iter()
                .find(|l| l.id == c.line_id)
                .is_some_and(|l| is_valid_amount(c.amount, l.total))
        })
}

/// Where a credit change goes: the local draft, and the host when it listens.
#[derive(Clone, Copy)]
struct CreditSink {
    draft: Signal<Vec<CreditedLine>>,
    on_change: Option<EventHandler<Vec<CreditedLine>>>,
}

impl CreditSink {
    fn set(self, next: Vec<CreditedLine>) {
        let mut draft = self.draft;
        draft.set(next.clone());
        if let Some(handler) = self.on_change {
            handler.call(next);
        }
    }
}

fn party_block(title: &str, party: &Party) -> Element {
    rsx! {
        section { class: "kit-inv__party", "aria-label": "{title}",
            h4 { class: "kit-inv__party-title", "{title}" }
            p { class: "kit-inv__party-name", "{party.name}" }
            for line in party.address.iter() {
                p { key: "{line}", class: "kit-inv__party-line", "{line}" }
            }
            if let Some(reg) = party.registration_id.clone() {
                p { class: "kit-inv__party-line", "Reg. {reg}" }
            }
        }
    }
}

#[component]
pub fn InvoiceCreditPair(
    invoice: Invoice,
    #[props(default)] credit_document: Option<CreditDocument>,
    #[props(default = Side::Invoice)] active_side: Side,
    #[props(default)] on_side_change: Option<EventHandler<Side>>,
    #[props(default)] on_credit_change: Option<EventHandler<Vec<CreditedLine>>>,
    #[props(default)] on_issue: Option<EventHandler<CreditDocument>>,
    #[props(default)] on_download: Option<EventHandler<()>>,
    #[props(default = default_money as fn(f64, &str) -> String)] format_amount: fn(
        f64,
        &str,
    ) -> String,
    #[props(default, into)] extra_classes: String,
) -> Element {
    let mut draft = use_signal(Vec::<CreditedLine>::new);
    let mut editing = use_signal(|| None::<(String, String)>);
    let mut seeded = use_signal(|| false);
    if !seeded() {
        seeded.set(true);
        if let Some(doc) = &credit_document {
            draft.set(doc.credited.clone());
        }
    }
    let issued = credit_document
        .as_ref()
        .is_some_and(|d| d.status == CreditStatus::Issued);
    let credited: Vec<CreditedLine> = match &credit_document {
        Some(doc) if issued => doc.credited.clone(),
        _ => draft(),
    };
    let total = credit_total(&credited, &invoice.lines);
    let issuable = !issued && can_issue(&credited, &invoice.lines);
    let currency = invoice.currency.clone();
    let root_class = class_list("kit-inv", &extra_classes);
    let invoice_total: f64 = invoice.lines.iter().map(|l| l.total).sum();
    let status_word = match invoice.status {
        InvoiceStatus::Open => "Open",
        InvoiceStatus::Paid => "Paid",
        InvoiceStatus::Credited => "Credited",
    };
    let credit_number = credit_document
        .as_ref()
        .map(|d| d.number.clone())
        .unwrap_or_else(|| format!("CN-{}", invoice.number));
    let picked_ids: Vec<String> = credited.iter().map(|c| c.line_id.clone()).collect();
    let credit_word = if issued { "Issued" } else { "Draft" };
    let on_credit_front = active_side == Side::Credit;

    let sink = CreditSink {
        draft,
        on_change: on_credit_change,
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{root_class}", "aria-label": "Invoice and credit note",
            header { class: "kit-inv__bar",
                SegmentedControl {
                    aria_label: "Document",
                    value: active_side.as_str().to_string(),
                    on_change: move |value: String| {
                        let side = if value == "credit" { Side::Credit } else { Side::Invoice };
                        if let Some(handler) = on_side_change { handler.call(side); }
                    },
                    options: vec![
                        SegmentOption { value: "invoice".to_string(), label: Some(rsx! { "Invoice" }), ..Default::default() },
                        SegmentOption { value: "credit".to_string(), label: Some(rsx! { "Credit note" }), ..Default::default() },
                    ],
                }
                div { class: "kit-inv__actions",
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
                        label: "Download",
                        onclick: move |_| {
                            if let Some(handler) = on_download { handler.call(()); }
                        },
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Sm,
                        label: "Credit this invoice",
                        disabled: invoice.status == InvoiceStatus::Credited || issued,
                        onclick: {
                            let lines = invoice.lines.clone();
                            move |_| {
                                sink.set(full_credit(&lines));
                                if let Some(handler) = on_side_change { handler.call(Side::Credit); }
                            }
                        },
                    }
                }
            }
            div { class: "kit-inv__stack",
                article { class: "kit-inv__sheet", "data-front": (!on_credit_front).then_some("true"), "aria-label": "Invoice {invoice.number}",
                    "aria-hidden": on_credit_front.then_some("true"),
                    "inert": on_credit_front.then_some("true"),
                    header { class: "kit-inv__sheet-head",
                        h3 { class: "kit-inv__heading", "Invoice {invoice.number}" }
                        span { class: "kit-inv__status", "data-status": status_word.to_lowercase(), "{status_word}" }
                        p { class: "kit-inv__total",
                            span { class: "kit-inv__total-figure", "{format_amount(invoice_total, &currency)}" }
                            span { class: "kit-inv__due", "Due {invoice.due_on}" }
                        }
                    }
                    div { class: "kit-inv__parties",
                        {party_block("From", &invoice.issuer)}
                        {party_block("To", &invoice.recipient)}
                    }
                    dl { class: "kit-inv__terms",
                        dt { "Issued" } dd { "{invoice.issued_on}" }
                        dt { "Due" } dd { "{invoice.due_on}" }
                        dt { "Period" } dd { "{invoice.period_from} to {invoice.period_to}" }
                    }
                    table { class: "kit-inv__table",
                        caption { class: "kit-sr", "Lines" }
                        thead { tr {
                            th { scope: "col", "Item" }
                            th { scope: "col", class: "kit-inv__col-rate", "Qty" }
                            th { scope: "col", class: "kit-inv__col-rate", "Rate" }
                            th { scope: "col", class: "kit-inv__col-rate", "Tax" }
                            th { scope: "col", "Price" }
                        } }
                        tbody {
                            for line in invoice.lines.iter() {
                                tr { key: "{line.id}",
                                    td { "{line.name}" }
                                    td { class: "kit-inv__col-rate", "{line.quantity}" }
                                    td { class: "kit-inv__col-rate", "{format_amount(line.unit_amount, &currency)}" }
                                    td { class: "kit-inv__col-rate", "{line.tax_percent:.0}%" }
                                    td { "{format_amount(line.total, &currency)}" }
                                }
                            }
                        }
                    }
                    p { class: "kit-inv__grand", "Total {format_amount(invoice_total, &currency)}" }
                }
                article { class: "kit-inv__sheet", "data-front": on_credit_front.then_some("true"), "aria-label": "Credit note {credit_number}",
                    "aria-hidden": (!on_credit_front).then_some("true"),
                    "inert": (!on_credit_front).then_some("true"),
                    header { class: "kit-inv__sheet-head",
                        h3 { class: "kit-inv__heading", "Credit note {credit_number}" }
                        span { class: "kit-inv__status", "data-status": if issued { "issued" } else { "draft" }, "{credit_word}" }
                    }
                    if !issued {
                        p { class: "kit-inv__hint", "Pick the lines to credit and set each amount." }
                    }
                    table { class: "kit-inv__table kit-inv__table--credit",
                        caption { class: "kit-sr", "Lines to credit" }
                        thead { tr {
                            th { scope: "col", "Credit" }
                            th { scope: "col", "Item" }
                            th { scope: "col", class: "kit-inv__col-rate", "Line total" }
                            th { scope: "col", "Amount" }
                        } }
                        tbody {
                            for line in invoice.lines.iter() {
                                {
                                    let picked = picked_ids.contains(&line.id);
                                    let entry = credited.iter().find(|c| c.line_id == line.id).cloned();
                                    let amount = entry.as_ref().map(|c| c.amount).unwrap_or(0.0);
                                    let valid = !picked || is_valid_amount(amount, line.total);
                                    let is_editing = editing().filter(|(id, _)| *id == line.id).map(|(_, text)| text);
                                    let toggle_id = line.id.clone();
                                    let field_id = line.id.clone();
                                    let line_name = line.name.clone();
                                    let line_total = line.total;
                                    let current = credited.clone();
                                    let toggle_current = credited.clone();
                                    let max_label = format_amount(line.total, &currency);
                                    let shown_amount = format!("{amount:.2}");
                                    rsx! {
                                        tr { key: "{line.id}", class: "kit-inv__credit-row", "data-picked": picked.then_some("true"),
                                            td {
                                                input {
                                                    r#type: "checkbox",
                                                    "aria-label": "Credit {line_name}",
                                                    checked: picked,
                                                    disabled: issued,
                                                    onchange: move |_| {
                                                        let next: Vec<CreditedLine> = if picked {
                                                            toggle_current.iter().filter(|c| c.line_id != toggle_id).cloned().collect()
                                                        } else {
                                                            let mut list = toggle_current.clone();
                                                            list.push(CreditedLine { line_id: toggle_id.clone(), amount: line_total });
                                                            list
                                                        };
                                                        sink.set(next);
                                                    },
                                                }
                                            }
                                            td { "{line_name}" }
                                            td { class: "kit-inv__col-rate", "{max_label}" }
                                            td {
                                                if picked {
                                                    input {
                                                        class: "kit-inv__amount",
                                                        r#type: "text",
                                                        inputmode: "decimal",
                                                        "aria-label": "Amount for {line_name}, up to {max_label}",
                                                        "aria-invalid": if valid { "false" } else { "true" },
                                                        disabled: issued,
                                                        value: is_editing.clone().unwrap_or(shown_amount),
                                                        oninput: {
                                                            let field = field_id.clone();
                                                            move |e: FormEvent| editing.set(Some((field.clone(), e.value())))
                                                        },
                                                        onblur: {
                                                            let field = field_id.clone();
                                                            let base = current.clone();
                                                            move |_| {
                                                                if let Some((id, text)) = editing() {
                                                                    if id == field {
                                                                        let typed: f64 = text.trim().parse().unwrap_or(0.0);
                                                                        let next: Vec<CreditedLine> = base.iter().map(|c| if c.line_id == id { CreditedLine { line_id: c.line_id.clone(), amount: typed } } else { c.clone() }).collect();
                                                                        sink.set(next);
                                                                    }
                                                                }
                                                                editing.set(None);
                                                            }
                                                        },
                                                        onkeydown: {
                                                            let field = field_id.clone();
                                                            let base = current.clone();
                                                            move |e: KeyboardEvent| {
                                                                if e.key() == Key::Enter {
                                                                    e.prevent_default();
                                                                    if let Some((id, text)) = editing() {
                                                                        if id == field {
                                                                            let typed: f64 = text.trim().parse().unwrap_or(0.0);
                                                                            let next: Vec<CreditedLine> = base.iter().map(|c| if c.line_id == id { CreditedLine { line_id: c.line_id.clone(), amount: typed } } else { c.clone() }).collect();
                                                                            sink.set(next);
                                                                        }
                                                                    }
                                                                    editing.set(None);
                                                                }
                                                                if e.key() == Key::Escape {
                                                                    editing.set(None);
                                                                }
                                                            }
                                                        },
                                                    }
                                                    if !valid {
                                                        span { class: "kit-inv__error", role: "alert", "Above the line total of {max_label}" }
                                                    }
                                                } else {
                                                    span { class: "kit-inv__muted", "Not credited" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "kit-inv__credit-foot",
                        p { class: "kit-inv__credit-total", "aria-live": "polite",
                            span { "Credit total " }
                            span { class: "kit-inv__total-figure", "{format_amount(total, &currency)}" }
                        }
                        if !issued {
                            Button {
                                variant: ButtonVariant::Primary,
                                label: "Issue credit note",
                                disabled: !issuable,
                                onclick: {
                                    let number = credit_number.clone();
                                    let reason = credit_document.as_ref().and_then(|d| d.reason.clone());
                                    let list = credited.clone();
                                    move |_| {
                                        if let Some(handler) = on_issue {
                                            handler.call(CreditDocument { number: number.clone(), reason: reason.clone(), credited: list.clone(), status: CreditStatus::Issued });
                                        }
                                    }
                                },
                            }
                        } else {
                            span { class: "kit-inv__issued", "Issued" }
                        }
                    }
                }
            }
        }
    }
}
