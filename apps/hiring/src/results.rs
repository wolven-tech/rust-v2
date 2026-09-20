//! The results: a register of entries, or a table, and what to say when
//! nothing survives.
//!
//! The empty state is the part worth reading twice. With both essential filters
//! on, the register legitimately returns almost nothing — that is the finding,
//! not a fault — so the page has to say which filter emptied it and what would
//! change the answer. It offers no retry, because the answer is authoritative
//! rather than unreachable: re-running the same query cannot produce a
//! different result, and a button that pretends otherwise is worse than no
//! button.

use dioxus::prelude::*;
use rv2_hiring::mandate::MandateStrength;
use rv2_hiring::model::{Company, Register, Role};
use rv2_hiring::view::{self, Layout, Sort, group_label};
use rv2_ui::{Badge, Select};

use crate::{Page, SharedRegister};

/// The tone a badge takes when its filter would keep the role. Green is the
/// only colour on the page that means "this one is takeable", so it is spent
/// on nothing else.
const PASSES: &str = "bg-emerald-100 text-emerald-900";

#[component]
pub fn Results(page: Page) -> Element {
    let register = use_context::<SharedRegister>();
    let register = register.read();
    let filters = page.filters.read().clone();
    let shown = view::apply(&register, &filters);

    rsx! {
        div { class: "space-y-4",
            Toolbar { page, rows: shown.len() }
            if shown.is_empty() {
                Empty { page }
            } else if filters.layout == Layout::Table {
                Table { companies: shown.into_iter().cloned().collect::<Vec<_>>(), page }
            } else {
                div { class: "space-y-3",
                    for company in shown.into_iter().cloned() {
                        Entry { key: "{company.name}", company, page }
                    }
                }
            }
        }
    }
}

#[component]
fn Toolbar(page: Page, rows: usize) -> Element {
    let filters = page.filters.read().clone();
    let mut f = page.filters;
    let layout = filters.layout;

    let options: Vec<(String, String)> = Sort::ALL
        .iter()
        .map(|s| (s.key().to_string(), s.label().to_string()))
        .collect();

    rsx! {
        div { class: "flex flex-wrap items-end justify-between gap-3 rounded-lg border border-slate-200 bg-white p-3",
            div { class: "w-64",
                Select {
                    label: "Sort",
                    value: filters.sort.key().to_string(),
                    options,
                    onchange: move |key: String| f.write().sort = Sort::from_key(&key),
                }
            }
            div { class: "flex items-center gap-2",
                div { class: "inline-flex overflow-hidden rounded-md border border-slate-300",
                    button {
                        r#type: "button",
                        class: if layout == Layout::Register { "bg-slate-900 px-3 py-1.5 text-sm text-white" } else { "px-3 py-1.5 text-sm" },
                        aria_pressed: "{layout == Layout::Register}",
                        onclick: move |_| f.write().layout = Layout::Register,
                        "Register"
                    }
                    button {
                        r#type: "button",
                        class: if layout == Layout::Table { "bg-slate-900 px-3 py-1.5 text-sm text-white" } else { "px-3 py-1.5 text-sm" },
                        aria_pressed: "{layout == Layout::Table}",
                        onclick: move |_| f.write().layout = Layout::Table,
                        "Table"
                    }
                }
                span { class: "text-sm tabular-nums text-slate-600", "{rows} shown" }
            }
        }
    }
}

/// Which filter emptied the list, most restrictive first.
///
/// Naming one is the whole job: "no companies match these filters" tells a
/// reader nothing they can act on, and with eleven controls on the page they
/// cannot be expected to bisect it themselves.
///
/// The counts are computed from the register rather than written into the copy.
/// A sentence saying "six roles are remote" is true on the day it is typed and
/// wrong after the next `hiring-register check`, and a stale number in an
/// explanation is worse than no number, because a reader believes it.
fn why_empty(page: &Page, register: &Register) -> (String, String) {
    let f = page.filters.read();
    let roles = || {
        register
            .companies
            .iter()
            .flat_map(|c| c.openings.iter().flat_map(|o| o.roles.iter()))
    };

    if f.remote_only && f.b2b_only && f.mandate_only {
        let takeable = roles().filter(|r| r.clears_essential_filters()).count();
        let mandated = roles()
            .filter(|r| {
                r.mandate
                    .as_ref()
                    .is_some_and(|m| matches!(m.strength, MandateStrength::Strong))
            })
            .count();
        return (
            "Nothing in the register is remote, B2B and department-building at once.".to_string(),
            format!(
                "That is the finding, not a fault. {takeable} roles here are remote and outside \
                 IR35, and {mandated} carry a real mandate, and they are not the same roles. Turn \
                 off \"Builds a department\" to see the {takeable}, or leave it on and turn off \
                 \"Remote\" to see the {mandated}."
            ),
        );
    }
    if f.b2b_only {
        return (
            "No role here is a B2B engagement under these filters.".to_string(),
            "Every board the register reads is an employer's own applicant-tracking system, which \
             advertises employment. Contract engagements come from the outside-IR35 board, so \
             loosen the other filters before this one."
                .to_string(),
        );
    }
    if f.remote_only {
        return (
            "No role here stays inside the 25% office ceiling under these filters.".to_string(),
            "A posting that names a city and says nothing about remote work counts as in the \
             office, and so does a bare \"hybrid\" with no day count. Both are recorded as such \
             rather than guessed at."
                .to_string(),
        );
    }
    if !f.q.is_empty() {
        return (
            format!("No companies match “{}” with these filters.", f.q),
            "Every word has to match somewhere in the company's name, ticker, sector, cities, \
             stack notes or open role titles."
                .to_string(),
        );
    }
    (
        "No companies match these filters.".to_string(),
        "Clearing the narrowest one usually brings the list back.".to_string(),
    )
}

#[component]
fn Empty(page: Page) -> Element {
    let register = use_context::<SharedRegister>();
    let (headline, explanation) = why_empty(&page, &register.read());
    let mut page_state = page;

    rsx! {
        section { class: "rounded-lg border border-slate-200 bg-white p-8 text-center",
            h2 { class: "text-lg font-semibold", "{headline}" }
            p { class: "mx-auto mt-2 max-w-2xl text-sm text-slate-600", "{explanation}" }
            button {
                r#type: "button",
                class: "mt-4 rounded-md border border-slate-300 px-3 py-1.5 text-sm hover:bg-slate-50",
                onclick: move |_| page_state.clear(),
                "Clear every filter"
            }
        }
    }
}

#[component]
fn Entry(company: Company, page: Page) -> Element {
    let filters = page.filters.read().clone();
    let roles = filters.surviving_roles(&company);
    let show_evidence = filters.litmus.shows_mandate_evidence();
    let openings = company.openings.clone();
    let mark = company.ticker();
    let mark = if mark.is_empty() {
        company
            .name
            .chars()
            .take(4)
            .collect::<String>()
            .to_uppercase()
    } else {
        mark
    };

    rsx! {
        article { class: "rounded-lg border border-slate-200 bg-white p-4",
            div { class: "flex flex-wrap items-start justify-between gap-3",
                div { class: "min-w-0",
                    div { class: "flex items-center gap-2",
                        span { class: "rounded bg-slate-900 px-1.5 py-0.5 font-mono text-xs text-white", "{mark}" }
                        h2 { class: "truncate text-base font-semibold", "{company.name}" }
                    }
                    p { class: "mt-0.5 text-sm text-slate-600",
                        "{company.sector()}"
                        if !company.uk_locations().is_empty() {
                            " in {company.uk_locations()}"
                        }
                    }
                    if !company.tech_stack_note().is_empty() {
                        p { class: "mt-1 text-sm text-slate-500", "{company.tech_stack_note()}" }
                    }
                }
                div { class: "flex shrink-0 flex-col items-end gap-1 text-sm",
                    if !company.careers_url.is_empty() {
                        a {
                            href: "{company.careers_url}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "rounded-md bg-slate-900 px-3 py-1.5 text-white",
                            "Open careers site"
                        }
                    }
                    if let Some(ats) = company.ats.as_ref() {
                        a {
                            href: "{ats.board_url()}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "text-slate-600 underline",
                            "All roles on {company.feed()}"
                        }
                    }
                    span { class: "text-xs text-slate-500", "{company.link_status().label()}" }
                }
            }

            ul { class: "mt-3 flex flex-wrap gap-1.5",
                for index in company.indices() {
                    li { key: "{index}",
                        Badge { "{group_label(&index)}" }
                    }
                }
                if company.public_equity() {
                    li {
                        Badge { "Listed equity, {company.exchange()}" }
                    }
                }
                if company.known_rust() {
                    li {
                        Badge { "Known Rust use" }
                    }
                }
            }

            if let Some(o) = openings {
                p { class: "mt-3 text-sm text-slate-700",
                    b { "{o.uk_engineering}" }
                    " UK engineering roles open, out of {o.uk} UK roles and {o.total} worldwide."
                    if let Some(failed) = o.last_read_failed.as_ref() {
                        span { class: "text-amber-700", " Counts are from the last successful read; {failed}'s read failed." }
                    }
                }
            }

            if roles.is_empty() {
                p { class: "mt-2 text-sm text-slate-500", "No roles recorded for this company." }
            } else {
                ul { class: "mt-2 space-y-2",
                    for role in roles {
                        li { key: "{role.url}",
                            RoleLine { role: role.clone(), show_evidence }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RoleLine(role: Role, show_evidence: bool) -> Element {
    let clears = role.clears_essential_filters();
    let mandate = role.mandate.clone();

    rsx! {
        div { class: if clears { "rounded-md border border-emerald-300 bg-emerald-50 p-2" } else { "rounded-md border border-slate-200 p-2" },
            div { class: "flex flex-wrap items-baseline justify-between gap-2",
                a {
                    href: "{role.url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "text-sm font-medium underline",
                    "{role.title}"
                }
                span { class: "text-xs text-slate-500", "{role.location}" }
            }
            div { class: "mt-1 flex flex-wrap gap-1.5",
                Badge {
                    class: if role.work_pattern.at_most_quarter_office() { PASSES } else { "" },
                    "{role.work_pattern.label()}"
                }
                Badge {
                    class: if role.engagement.is_b2b() { PASSES } else { "" },
                    "{role.engagement.label()}"
                }
                if let Some(m) = mandate.as_ref() {
                    Badge { "{m.strength.label()}" }
                    Badge { "{m.level.label()}" }
                }
            }
            if !role.work_pattern_note.is_empty() {
                p { class: "mt-1 text-[0.6875rem] leading-snug text-slate-500",
                    "Working pattern: {role.work_pattern_note}. Engagement: {role.engagement_note}."
                }
            }
            if show_evidence
                && let Some(m) = mandate.as_ref()
                && !m.quotes.is_empty()
            {
                ul { class: "mt-1 space-y-0.5",
                    for (signal , quote) in m.signals.iter().zip(m.quotes.iter()) {
                        li { key: "{quote}", class: "text-[0.6875rem] leading-snug text-slate-600",
                            b { "{signal.label()}: " }
                            "“{quote}”"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Table(companies: Vec<Company>, page: Page) -> Element {
    let filters = page.filters.read().clone();
    rsx! {
        div { class: "overflow-x-auto rounded-lg border border-slate-200 bg-white",
            table { class: "w-full text-sm",
                thead { class: "border-b border-slate-200 bg-slate-50 text-left",
                    tr {
                        th { class: "px-3 py-2", scope: "col", "Company" }
                        th { class: "px-3 py-2", scope: "col", "Index" }
                        th { class: "px-3 py-2", scope: "col", "UK locations" }
                        th { class: "px-3 py-2 text-right", scope: "col", "UK engineering roles" }
                        th { class: "px-3 py-2 text-right", scope: "col", "Roles clearing both filters" }
                        th { class: "px-3 py-2", scope: "col", "Job board" }
                    }
                }
                tbody {
                    for company in companies {
                        tr { key: "{company.name}", class: "border-b border-slate-100",
                            td { class: "px-3 py-2",
                                div { class: "font-medium", "{company.name}" }
                                div { class: "text-xs text-slate-500", "{company.sector()}" }
                            }
                            td { class: "px-3 py-2 text-xs",
                                "{company.indices().iter().map(|g| group_label(g).to_string()).collect::<Vec<_>>().join(\", \")}"
                            }
                            td { class: "px-3 py-2 text-xs", "{company.uk_locations()}" }
                            td { class: "px-3 py-2 text-right tabular-nums",
                                "{company.openings.as_ref().map_or(0, |o| o.uk_engineering)}"
                            }
                            td { class: "px-3 py-2 text-right tabular-nums",
                                "{filters.surviving_roles(&company).iter().filter(|r| r.clears_essential_filters()).count()}"
                            }
                            td { class: "px-3 py-2 text-xs", "{company.feed()}" }
                        }
                    }
                }
            }
        }
    }
}
