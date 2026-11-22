//! Plan comparison section: plans side by side, a monthly or yearly switch, and a details area
//! per plan with its integrations and the features it includes.
//!
//! Design choice: a details area opens for its own card only; the others keep their state.
//! Design choice: the price words "per month" and "per year" are in the text, not only in the
//! layout, so the period reaches assistive technology.
//! Design choice: a not-included feature keeps its row and says "not included" beside its
//! marker.
//! Design choice: the billing indicator is one pill that glides behind the two options, and each
//! price digit rolls from its old value to the new one; the screen-reader text is the whole price.
//! Design choice: the price is a NumberTicker, so on first view it counts up from zero.
//! Browser checks: yearly switch changes every price, one card's details does not open the
//! others, Left/Right on the billing switch, Escape closes details and returns focus, the
//! highlighted badge and emphasised call to action, phone stacking.

use dioxus::prelude::*;

use rv2_affect::motion::number_ticker::NumberTicker;

use crate::internal::{KIT_CSS, class_list, focus_after_frame};

const CSS: Asset = asset!("/assets/plan-comparison-section.css");

pub const MONTHLY: &str = "monthly";
pub const YEARLY: &str = "yearly";

#[derive(Clone, Debug, PartialEq)]
pub struct PlanLimit {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlanFeature {
    pub label: String,
    pub included: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlanIntegrations {
    pub names: Vec<String>,
    pub sync_label: String,
    pub max_selectable: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub id: String,
    pub name: String,
    pub price_monthly: f64,
    pub price_yearly: f64,
    pub billing_note_monthly: String,
    pub billing_note_yearly: String,
    pub cta_label: String,
    pub cta_href: String,
    pub highlighted: bool,
    pub badge: Option<String>,
    pub limits: Vec<PlanLimit>,
    pub integrations: PlanIntegrations,
    pub features: Vec<PlanFeature>,
}

/// Price for the chosen billing period.
pub fn price_for(plan: &Plan, billing: &str) -> f64 {
    if billing == YEARLY {
        plan.price_yearly
    } else {
        plan.price_monthly
    }
}

/// `per month` or `per year`.
pub fn period_words(billing: &str) -> &'static str {
    if billing == YEARLY {
        "per year"
    } else {
        "per month"
    }
}

/// Saving of yearly against twelve monthly payments, in percent, rounded down.
pub fn yearly_saving_percent(monthly: f64, yearly: f64) -> u32 {
    if monthly <= 0.0 {
        return 0;
    }
    let full = monthly * 12.0;
    if yearly >= full {
        return 0;
    }
    ((full - yearly) / full * 100.0).floor() as u32
}

/// Two decimals only when the price has cents.
pub fn format_price(amount: f64) -> String {
    if amount.fract() == 0.0 {
        format!("${amount:.0}")
    } else {
        format!("${amount:.2}")
    }
}

/// Billing period an arrow key selects: right and down go yearly, left and up go monthly.
pub fn billing_for_key(key: &Key) -> Option<&'static str> {
    match key {
        Key::ArrowRight | Key::ArrowDown => Some(YEARLY),
        Key::ArrowLeft | Key::ArrowUp => Some(MONTHLY),
        _ => None,
    }
}

#[component]
fn PlanCard(
    plan: Plan,
    billing: String,
    details_open: bool,
    on_toggle: EventHandler<bool>,
) -> Element {
    let price = price_for(&plan, &billing);
    let period = period_words(&billing);
    let note = if billing == YEARLY {
        plan.billing_note_yearly.clone()
    } else {
        plan.billing_note_monthly.clone()
    };
    let toggle_id = format!("kit-pcs-{}-toggle", plan.id);
    let details_id = format!("kit-pcs-{}-details", plan.id);
    let toggle_for_escape = toggle_id.clone();
    let names = plan.integrations.names.join(", ");
    let selectable = format!(
        "{} of {} selectable",
        plan.integrations
            .names
            .len()
            .min(plan.integrations.max_selectable),
        plan.integrations.max_selectable
    );
    let toggle_label = if details_open {
        "Hide details"
    } else {
        "Show details"
    };
    rsx! {
        article { class: "kit-pcs__card", "aria-label": "{plan.name}", "data-highlighted": plan.highlighted.then_some("true"),
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape && details_open {
                    on_toggle.call(false);
                    focus_after_frame(&toggle_for_escape);
                }
            },
            header { class: "kit-pcs__head",
                h3 { class: "kit-pcs__name", "{plan.name}" }
                if let Some(badge) = plan.badge.clone() {
                    span { class: "kit-pcs__badge", "{badge}" }
                }
            }
            p { class: "kit-pcs__price",
                NumberTicker {
                    value: price,
                    decimals: if price.fract() == 0.0 { 0 } else { 2 },
                    grouping: false,
                    prefix: "$",
                    class: "kit-pcs__figure".to_string(),
                }
                span { class: "kit-pcs__period", " {period}" }
            }
            p { class: "kit-pcs__note", "{note}" }
            a {
                class: "kit-pcs__cta",
                href: "{plan.cta_href}",
                "data-emphasis": plan.highlighted.then_some("true"),
                "{plan.cta_label}"
            }
            ul { class: "kit-pcs__limits",
                for limit in plan.limits.iter() {
                    li { key: "{limit.label}", span { "{limit.label}" } strong { "{limit.value}" } }
                }
            }
            button {
                id: "{toggle_id}",
                r#type: "button",
                class: "kit-pcs__toggle",
                "aria-expanded": if details_open { "true" } else { "false" },
                "aria-controls": "{details_id}",
                onclick: move |_| on_toggle.call(!details_open),
                "{toggle_label}"
            }
            if details_open {
                div { id: "{details_id}", class: "kit-pcs__details", role: "region", "aria-labelledby": "{toggle_id}",
                    section { class: "kit-pcs__integrations",
                        h4 { "Integrations" }
                        p { class: "kit-pcs__sync", "{plan.integrations.sync_label} · {selectable}" }
                        p { class: "kit-pcs__names", "{names}" }
                    }
                    ul { class: "kit-pcs__features",
                        for feature in plan.features.iter() {
                            li { key: "{feature.label}", "data-included": feature.included.then_some("true"),
                                span { class: "kit-pcs__mark", "aria-hidden": "true", if feature.included { "✓" } else { "–" } }
                                span { class: "kit-pcs__feature-label", "{feature.label}" }
                                if !feature.included {
                                    span { class: "kit-pcs__not", "not included" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn PlanComparisonSection(
    plans: Vec<Plan>,
    #[props(default = "Plans".to_string(), into)] heading: String,
    #[props(default, into)] intro: String,
    #[props(default = MONTHLY.to_string(), into)] selected_billing: String,
    #[props(default)] on_billing_change: Option<EventHandler<String>>,
    #[props(default)] yearly_saving_label: Option<String>,
    #[props(default)] initial_details_open: bool,
    #[props(default, into)] extra_classes: String,
) -> Element {
    let mut billing = use_signal(|| selected_billing.clone());
    let mut open = use_signal(|| {
        if initial_details_open {
            plans.iter().map(|p| p.id.clone()).collect::<Vec<_>>()
        } else {
            Vec::<String>::new()
        }
    });
    let current = billing();
    let root_class = class_list("kit-pcs", &extra_classes);
    let saving = yearly_saving_label.clone().or_else(|| {
        plans.iter().find(|p| p.price_monthly > 0.0).map(|p| {
            format!(
                "Save {}%",
                yearly_saving_percent(p.price_monthly, p.price_yearly)
            )
        })
    });

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{root_class}", "aria-labelledby": "kit-pcs-title",
            header { class: "kit-pcs__intro",
                h2 { id: "kit-pcs-title", class: "kit-pcs__title", "{heading}" }
                if !intro.is_empty() {
                    p { class: "kit-pcs__lede", "{intro}" }
                }
            }
            div { class: "kit-pcs__billing", role: "radiogroup", "aria-label": "Billing period",
                div { class: "kit-pcs__switch", "data-yearly": (current == YEARLY).then_some("true"),
                span { class: "kit-pcs__pill", "aria-hidden": "true" }
                for option in [MONTHLY, YEARLY] {
                    {
                        let on = current == option;
                        let label = if option == YEARLY { "Yearly" } else { "Monthly" };
                        rsx! {
                            button {
                                key: "{option}",
                                r#type: "button",
                                role: "radio",
                                class: "kit-pcs__option",
                                "aria-checked": if on { "true" } else { "false" },
                                tabindex: if on { "0" } else { "-1" },
                                onclick: move |_| {
                                    billing.set(option.to_string());
                                    if let Some(handler) = on_billing_change { handler.call(option.to_string()); }
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    if let Some(next) = billing_for_key(&e.key()) {
                                        e.prevent_default();
                                        billing.set(next.to_string());
                                        if let Some(handler) = on_billing_change { handler.call(next.to_string()); }
                                    }
                                },
                                "{label}"
                            }
                        }
                    }
                }
                }
                if let Some(text) = saving.clone() {
                    if current == YEARLY {
                        span { class: "kit-pcs__saving", "{text}" }
                    }
                }
            }
            div { class: "kit-pcs__grid",
                for plan in plans.iter().cloned() {
                    {
                        let id = plan.id.clone();
                        let is_open = open().contains(&id);
                        rsx! {
                            PlanCard {
                                key: "{plan.id}",
                                plan: plan,
                                billing: current.clone(),
                                details_open: is_open,
                                on_toggle: move |next: bool| {
                                    let mut list = open();
                                    list.retain(|p| *p != id);
                                    if next {
                                        list.push(id.clone());
                                    }
                                    open.set(list);
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}
