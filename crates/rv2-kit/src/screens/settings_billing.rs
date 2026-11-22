//! Settings: Billing. The plan in force, the next bill, allowance meters, plan and
//! add-on choices, billing details, the payment card, invoice history and cancel.
//!
//! Design choice: the spec's `workspace_name` shell label is not rendered; the
//! workspace frame supplies it. `account` shows as the nav footer.
//! Design choice: the invoice field is `reference` (`ref` is a Rust keyword).
//! Design choice: `pending_plan` keeps the chosen plan's button busy until the host
//! clears it; the kit cannot know whether the upgrade has been confirmed.
//! Design choice: cancel uses the kit's `AlertDialog`, so Escape and the outside
//! click dismiss it without calling `on_cancel`.
//! Browser checks: loading shapes, empty history preview, yearly switch, details
//! toggle, current-plan disabled, cancel dialog keyboard path, phone width.

use dioxus::prelude::*;
use rv2_affect::dialog::AlertDialog;

use super::settings_profile::{
    SettingsCard, SettingsRow, SettingsShell, SettingsSkel, SettingsStatus,
};
use crate::internal::next_id;

const BILLING_CSS: Asset = asset!("/assets/settings-billing.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BillInterval {
    #[default]
    Month,
    Year,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BillingCycle {
    #[default]
    Monthly,
    Yearly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InvoiceStatus {
    #[default]
    Paid,
    Upcoming,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BreakdownLine {
    pub label: String,
    pub amount: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CurrentPlan {
    pub name: String,
    pub price_per_seat: String,
    pub interval: BillInterval,
    pub seats_used: u32,
    pub seats_total: u32,
    pub renews_on: String,
    pub upcoming_amount: String,
    pub breakdown: Vec<BreakdownLine>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Allowance {
    pub label: String,
    pub used: u64,
    pub total: u64,
    pub unit: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlanCard {
    pub id: String,
    pub name: String,
    pub monthly_per_seat: String,
    pub yearly_per_seat: String,
    pub features: Vec<String>,
    pub current: bool,
    pub recommended: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AddOnTier {
    pub id: String,
    pub label: String,
    pub price: String,
    pub current: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AddOn {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tiers: Vec<AddOnTier>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BillingAddress {
    pub company: String,
    pub line: String,
    pub email: String,
    pub vat_number: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CardInfo {
    pub brand: String,
    pub last4: String,
    pub expires: String,
    pub is_default: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Invoice {
    pub reference: String,
    pub date: String,
    pub amount: String,
    pub status: InvoiceStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AccountBlock {
    pub name: String,
    pub avatar_url: Option<String>,
}

/// Share of an allowance used, 0 to 100, clamped. A zero total reads as empty.
pub fn allowance_share(used: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    (used as f64 / total as f64 * 100.0).clamp(0.0, 100.0)
}

/// The meter's text: `used of total unit`, so the share is never colour alone.
pub fn allowance_text(allowance: &Allowance) -> String {
    format!(
        "{} of {} {}",
        allowance.used, allowance.total, allowance.unit
    )
}

/// The price a plan card shows for the chosen cycle.
pub fn plan_price(plan: &PlanCard, cycle: BillingCycle) -> &str {
    match cycle {
        BillingCycle::Monthly => &plan.monthly_per_seat,
        BillingCycle::Yearly => &plan.yearly_per_seat,
    }
}

/// The label a plan's action carries: the current plan is disabled.
pub fn plan_action_label(plan: &PlanCard) -> String {
    if plan.current {
        "Current plan".to_string()
    } else {
        format!("Choose {}", plan.name)
    }
}

/// Billing screen: plan summary, allowances, plans and add-ons, details, payment
/// card, invoices and cancel.
#[component]
pub fn SettingsBilling(
    current_plan: CurrentPlan,
    allowances: Vec<Allowance>,
    #[props(default)] billing_cycle: BillingCycle,
    plans: Vec<PlanCard>,
    add_ons: Vec<AddOn>,
    billing_address: BillingAddress,
    card: CardInfo,
    invoices: Vec<Invoice>,
    #[props(default)] account: Option<AccountBlock>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] pending_plan: Option<String>,
    #[props(default)] on_change_plan: Option<EventHandler<String>>,
    #[props(default)] on_add_add_on: Option<EventHandler<(String, String)>>,
    #[props(default)] on_edit_address: Option<EventHandler<()>>,
    #[props(default)] on_edit_card: Option<EventHandler<()>>,
    #[props(default)] on_cancel: Option<EventHandler<()>>,
    #[props(default)] on_preview_bill: Option<EventHandler<()>>,
    #[props(default)] on_open_invoice: Option<EventHandler<String>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] docs_href: Option<String>,
) -> Element {
    let uid = use_hook(|| next_id());
    let mut cycle = use_signal(|| billing_cycle);
    let mut details_open = use_signal(|| false);
    let mut compare_open = use_signal(|| false);
    let mut address_form = use_signal(|| false);
    let mut address = use_signal(|| billing_address.clone());
    let mut address_draft = use_signal(|| billing_address.clone());
    let mut cancel_open = use_signal(|| false);
    let mut row_focus = use_signal(|| 0usize);
    let invoice_count = invoices.len();
    let mut status = use_signal(String::new);
    let mut tier_choice = use_signal(|| {
        add_ons
            .iter()
            .map(|a| {
                let chosen = a.tiers.iter().find(|t| t.current).or(a.tiers.first());
                (
                    a.id.clone(),
                    chosen.map(|t| t.id.clone()).unwrap_or_default(),
                )
            })
            .collect::<Vec<(String, String)>>()
    });
    let account_name = account.as_ref().map(|a| a.name.clone());

    let interval_word = match current_plan.interval {
        BillInterval::Month => "month",
        BillInterval::Year => "year",
    };
    let seat_share = allowance_share(
        current_plan.seats_used as u64,
        current_plan.seats_total as u64,
    );
    let shown_address = address();
    let is_yearly = cycle() == BillingCycle::Yearly;

    if is_loading {
        return rsx! {
            SettingsShell { current: "Billing", title: "Billing", description: "Your plan, what it costs, and past invoices.", on_navigate, account_name, docs_href: docs_href.clone(),
                div { class: "kit-set__loading", "aria-busy": "true",
                    SettingsCard { heading: "Your plan".to_string(), SettingsSkel { kind: "block" } }
                    div { class: "kit-set__meters",
                        for slot in 0..4 {
                            span { key: "{slot}", class: "kit-set__skel", "data-kind": "block", aria_hidden: "true" }
                        }
                    }
                    SettingsCard { heading: "Plans".to_string(), SettingsSkel { kind: "block" } }
                    SettingsCard { heading: "Billing details".to_string(), SettingsSkel { kind: "block" } }
                    SettingsCard { heading: "Invoices".to_string(), SettingsSkel { kind: "block" } }
                }
            }
        };
    }

    rsx! {
        SettingsShell { current: "Billing", title: "Billing", description: "Your plan, what it costs, and past invoices.", on_navigate, account_name, docs_href: docs_href.clone(),
            document::Link { rel: "stylesheet", href: BILLING_CSS }
            SettingsCard { heading: current_plan.name.clone(),
                note: format!("{} per seat, per {interval_word}. Renews on {}.", current_plan.price_per_seat, current_plan.renews_on),
                div { class: "kit-set__bill",
                    div { class: "kit-set__bill-line",
                        span { class: "kit-set__value", "Next bill: {current_plan.upcoming_amount}" }
                        button {
                            r#type: "button",
                            class: "kit-set__btn",
                            aria_expanded: details_open().to_string(),
                            aria_controls: "{uid}-breakdown",
                            onclick: move |_| details_open.set(!details_open()),
                            if details_open() { "Hide details" } else { "Show details" }
                        }
                    }
                    if details_open() {
                        dl { id: "{uid}-breakdown", class: "kit-set__breakdown",
                            for line in current_plan.breakdown.clone() {
                                div { key: "{line.label}", class: "kit-set__breakdown-row",
                                    dt { "{line.label}" }
                                    dd { "{line.amount}" }
                                }
                            }
                        }
                    }
                    p { class: "kit-set__info", "{current_plan.seats_used} of {current_plan.seats_total} seats in use." }
                    if seat_share >= 100.0 {
                        p { class: "kit-set__info", "data-tone": "warn", "Every seat is in use." }
                    }
                    button {
                        r#type: "button",
                        class: "kit-set__btn kit-set__btn--quiet kit-set__link-btn",
                        onclick: move |_| {
                            if let Some(h) = &on_navigate {
                                h.call("Members".to_string());
                            }
                        },
                        "Manage members"
                    }
                }
            }
            div { class: "kit-set__meters",
                for allowance in allowances.clone() {
                    {
                        let share = allowance_share(allowance.used, allowance.total);
                        rsx! {
                            div { key: "{allowance.label}", class: "kit-set__meter",
                                div { class: "kit-set__meter-head",
                                    span { class: "kit-set__row-label", "{allowance.label}" }
                                    span { class: "kit-set__meter-text", "{allowance_text(&allowance)}" }
                                }
                                div {
                                    class: "kit-set__meter-track",
                                    role: "progressbar",
                                    aria_label: "{allowance.label}",
                                    aria_valuemin: "0",
                                    aria_valuemax: "100",
                                    aria_valuenow: "{share.round()}",
                                    div { class: "kit-set__meter-fill", style: "--kit-set-fill: {share:.1}%" }
                                }
                            }
                        }
                    }
                }
            }
            SettingsCard { heading: "Plans".to_string(),
                note: "Prices are per seat.".to_string(),
                div { class: "kit-set__cycle", role: "radiogroup", aria_label: "Billing cycle",
                    label { class: "kit-set__radio",
                        input {
                            r#type: "radio",
                            name: "billing-cycle-{uid}",
                            value: "monthly",
                            checked: !is_yearly,
                            onchange: move |_| cycle.set(BillingCycle::Monthly),
                        }
                        "Monthly"
                    }
                    label { class: "kit-set__radio",
                        input {
                            r#type: "radio",
                            name: "billing-cycle-{uid}",
                            value: "yearly",
                            checked: is_yearly,
                            onchange: move |_| cycle.set(BillingCycle::Yearly),
                        }
                        "Yearly"
                    }
                    span { class: "kit-set__saving", "Two months free" }
                }
                div { class: "kit-set__plans",
                    for plan in plans.clone() {
                        {
                            let price = plan_price(&plan, cycle()).to_string();
                            let busy = pending_plan.as_deref() == Some(plan.id.as_str());
                            let id = plan.id.clone();
                            rsx! {
                                article {
                                    key: "{plan.id}",
                                    class: "kit-set__plan",
                                    "data-current": plan.current.then_some("true"),
                                    "data-recommended": plan.recommended.then_some("true"),
                                    aria_label: "{plan.name}, {price} per seat per month",
                                    div { class: "kit-set__plan-head",
                                        h3 { class: "kit-set__plan-name", "{plan.name}" }
                                        if plan.recommended {
                                            span { class: "kit-set__tag", "Recommended" }
                                        }
                                    }
                                    p { class: "kit-set__plan-price",
                                        span { class: "kit-set__plan-amount", "{price}" }
                                        span { class: "kit-set__info", " per seat" }
                                    }
                                    ul { class: "kit-set__features",
                                        for feature in plan.features.clone() {
                                            li { key: "{feature}", "{feature}" }
                                        }
                                    }
                                    button {
                                        r#type: "button",
                                        class: "kit-set__btn kit-set__btn--primary",
                                        disabled: plan.current || busy,
                                        aria_busy: busy.then_some("true"),
                                        onclick: move |_| {
                                            if let Some(h) = &on_change_plan {
                                                h.call(id.clone());
                                            }
                                        },
                                        if busy { "Updating…" } else { "{plan_action_label(&plan)}" }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "kit-set__inline",
                    button {
                        r#type: "button",
                        class: "kit-set__btn kit-set__btn--quiet",
                        aria_expanded: compare_open().to_string(),
                        onclick: move |_| compare_open.set(!compare_open()),
                        "Compare plans"
                    }
                }
                if compare_open() {
                    ul { class: "kit-set__compare",
                        for plan in plans.clone() {
                            li { key: "{plan.id}", "{plan.name}: {plan.features.join(\", \")}" }
                        }
                    }
                }
            }
            SettingsCard { heading: "Add-ons".to_string(),
                for add_on in add_ons.clone() {
                    {
                        let chosen = tier_choice()
                            .iter()
                            .find(|(id, _)| *id == add_on.id)
                            .map(|(_, tier)| tier.clone())
                            .unwrap_or_default();
                        let add_on_id = add_on.id.clone();
                        rsx! {
                            fieldset { key: "{add_on.id}", class: "kit-set__addon",
                                legend { class: "kit-set__row-label", "{add_on.name}" }
                                p { class: "kit-set__info", "{add_on.description}" }
                                div { class: "kit-set__radios",
                                    for tier in add_on.tiers.clone() {
                                        label { key: "{tier.id}", class: "kit-set__radio",
                                            input {
                                                r#type: "radio",
                                                name: "addon-{add_on.id}-{uid}",
                                                value: "{tier.id}",
                                                checked: chosen == tier.id,
                                                onchange: {
                                                    let add_on_id = add_on_id.clone();
                                                    let tier_id = tier.id.clone();
                                                    move |_| {
                                                        let (a, t) = (add_on_id.clone(), tier_id.clone());
                                                        tier_choice.with_mut(|list| {
                                                            if let Some(entry) = list.iter_mut().find(|(id, _)| *id == a) {
                                                                entry.1 = t;
                                                            }
                                                        });
                                                    }
                                                },
                                            }
                                            "{tier.label}"
                                            span { class: "kit-set__info", " {tier.price}" }
                                        }
                                    }
                                }
                                button {
                                    r#type: "button",
                                    class: "kit-set__btn",
                                    onclick: {
                                        let add_on_id = add_on_id.clone();
                                        move |_| {
                                            let tier = tier_choice()
                                                .iter()
                                                .find(|(id, _)| *id == add_on_id)
                                                .map(|(_, t)| t.clone())
                                                .unwrap_or_default();
                                            if let Some(h) = &on_add_add_on {
                                                h.call((add_on_id.clone(), tier));
                                            }
                                            status.set("Add-on updated.".to_string());
                                        }
                                    },
                                    "Apply"
                                }
                            }
                        }
                    }
                }
            }
            SettingsCard { heading: "Billing details".to_string(),
                SettingsRow { label: "Company".to_string(), span { class: "kit-set__value", "{shown_address.company}" } }
                SettingsRow { label: "Billing address".to_string(), span { class: "kit-set__value", "{shown_address.line}" } }
                SettingsRow { label: "Email".to_string(), span { class: "kit-set__value", "{shown_address.email}" } }
                if let Some(vat) = shown_address.vat_number.clone() {
                    SettingsRow { label: "VAT number".to_string(), span { class: "kit-set__value", "{vat}" } }
                }
                div { class: "kit-set__row-actions",
                    button {
                        r#type: "button",
                        class: "kit-set__btn",
                        aria_expanded: address_form().to_string(),
                        onclick: move |_| {
                            address_draft.set(address());
                            address_form.set(!address_form());
                        },
                        "Edit billing details"
                    }
                }
                if address_form() {
                    form { class: "kit-set__form", aria_label: "Edit billing details",
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Escape {
                                address_form.set(false);
                            }
                        },
                        onsubmit: move |e: FormEvent| {
                            e.prevent_default();
                            address.set(address_draft());
                            address_form.set(false);
                            if let Some(h) = &on_edit_address {
                                h.call(());
                            }
                            status.set("Billing details saved.".to_string());
                        },
                        label { r#for: "{uid}-company", "Company" }
                        input {
                            id: "{uid}-company", class: "kit-set__input", r#type: "text", value: "{address_draft().company}",
                            oninput: move |e: FormEvent| address_draft.with_mut(|a| a.company = e.value()),
                        }
                        label { r#for: "{uid}-line", "Billing address" }
                        input {
                            id: "{uid}-line", class: "kit-set__input", r#type: "text", value: "{address_draft().line}",
                            oninput: move |e: FormEvent| address_draft.with_mut(|a| a.line = e.value()),
                        }
                        label { r#for: "{uid}-email-b", "Billing email" }
                        input {
                            id: "{uid}-email-b", class: "kit-set__input", r#type: "email", value: "{address_draft().email}",
                            oninput: move |e: FormEvent| address_draft.with_mut(|a| a.email = e.value()),
                        }
                        div { class: "kit-set__inline",
                            button { r#type: "submit", class: "kit-set__btn kit-set__btn--primary", "Save" }
                            button { r#type: "button", class: "kit-set__btn", onclick: move |_| address_form.set(false), "Cancel" }
                        }
                    }
                }
            }
            SettingsCard { heading: "Payment".to_string(),
                SettingsRow { label: format!("{} ending {}", card.brand, card.last4),
                    note: format!("Expires {}", card.expires),
                    div { class: "kit-set__inline",
                        if card.is_default {
                            span { class: "kit-set__tag", "Default" }
                        }
                        button {
                            r#type: "button",
                            class: "kit-set__btn",
                            onclick: move |_| {
                                if let Some(h) = &on_edit_card {
                                    h.call(());
                                }
                            },
                            "Edit card"
                        }
                    }
                }
            }
            SettingsCard { heading: "Invoices".to_string(),
                if is_empty || invoices.is_empty() {
                    div { class: "kit-set__empty",
                        p { class: "kit-set__info", "No invoices yet. The first one arrives when the subscription renews." }
                        button {
                            r#type: "button",
                            class: "kit-set__btn",
                            onclick: move |_| {
                                if let Some(h) = &on_preview_bill {
                                    h.call(());
                                }
                            },
                            "Preview first bill"
                        }
                    }
                } else {
                    div { class: "kit-set__table-wrap",
                        table { class: "kit-set__table",
                            onkeydown: move |e: KeyboardEvent| {
                                let step: isize = match e.key() {
                                    Key::ArrowDown => 1,
                                    Key::ArrowUp => -1,
                                    _ => return,
                                };
                                let next = (row_focus() as isize + step).clamp(0, invoice_count as isize - 1) as usize;
                                row_focus.set(next);
                                document::eval(&format!("document.getElementById('{uid}-inv-{next}')?.focus();"));
                            },
                            thead {
                                tr {
                                    th { scope: "col", "Reference" }
                                    th { scope: "col", "Total incl. tax" }
                                    th { scope: "col", "Date" }
                                    th { scope: "col", "Status" }
                                }
                            }
                            tbody {
                                for (index, invoice) in invoices.clone().into_iter().enumerate() {
                                    tr { key: "{invoice.reference}",
                                        td {
                                            button {
                                                id: "{uid}-inv-{index}",
                                                r#type: "button",
                                                class: "kit-set__link-btn",
                                                tabindex: if row_focus() == index { "0" } else { "-1" },
                                                onfocus: move |_| row_focus.set(index),
                                                onclick: {
                                                    let reference = invoice.reference.clone();
                                                    move |_| {
                                                        if let Some(h) = &on_open_invoice {
                                                            h.call(reference.clone());
                                                        }
                                                    }
                                                },
                                                "{invoice.reference}"
                                            }
                                        }
                                        td { class: "kit-set__num", "{invoice.amount}" }
                                        td { "{invoice.date}" }
                                        td {
                                            span { class: "kit-set__tag", "data-status": match invoice.status { InvoiceStatus::Paid => "paid", InvoiceStatus::Upcoming => "upcoming" },
                                                match invoice.status {
                                                    InvoiceStatus::Paid => "Paid",
                                                    InvoiceStatus::Upcoming => "Upcoming",
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
            div { class: "kit-set__danger-row",
                button {
                    r#type: "button",
                    class: "kit-set__btn",
                    onclick: move |_| cancel_open.set(true),
                    "Cancel subscription"
                }
            }
            AlertDialog {
                open: cancel_open(),
                onopenchange: move |open: bool| cancel_open.set(open),
                title: rsx! { "Cancel subscription?" },
                description: rsx! { "Your plan stays active until the end of this billing period." },
                confirm_label: "Cancel subscription",
                cancel_label: "Keep plan",
                danger: true,
                onconfirm: move |_| {
                    cancel_open.set(false);
                    status.set("Subscription cancelled. Access continues until the period ends.".to_string());
                    if let Some(h) = &on_cancel {
                        h.call(());
                    }
                },
                oncancel: move |_| cancel_open.set(false),
                p { class: "kit-set__info", "You can resume at any time from this page." }
            }
            SettingsStatus { message: status() }
        }
    }
}
