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
//!
//! Nothing here becomes a link without passing `safe_url` first. Records come
//! from research, bookmark exports and other people's feeds.

use dioxus::prelude::*;
use rv2_hiring::links::{careers_search, linkedin_search, safe_url};
use rv2_hiring::mandate::MandateStrength;
use rv2_hiring::model::{Company, LinkStatus, Register, Role};
use rv2_hiring::view::{self, Layout, Sort, group_colour, group_label};
use rv2_ui::{Badge, Disclosure, Select, Status, StatusTone};

use crate::{Page, SharedRegister};

/// The tone a badge takes when its filter would keep the role. Green is the
/// only colour on the page that means "this one is takeable", so it is spent
/// on nothing else.
const PASSES: &str = "bg-takeable-soft text-takeable-ink";

const EXT: &str = "noopener noreferrer";

fn tone_of(status: LinkStatus) -> StatusTone {
    match status {
        LinkStatus::Live => StatusTone::Ok,
        LinkStatus::Blocked | LinkStatus::Unreachable => StatusTone::Warn,
        LinkStatus::Broken => StatusTone::Bad,
        LinkStatus::Missing | LinkStatus::Unchecked => StatusTone::Muted,
    }
}

#[component]
pub fn Results(page: Page) -> Element {
    let register = use_context::<SharedRegister>();
    let register = register.read();
    let filters = page.filters.read().clone();
    let shown = view::apply(&register, &filters);

    rsx! {
        div { class: "space-y-4",
            Toolbar { page, rows: shown.len(), total: register.companies.len() }
            Legend {}
            if filters.layout == Layout::Dashboard {
                crate::dashboard::DashboardView { page }
            } else if shown.is_empty() {
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

/// What the ticker badge's two treatments mean.
///
/// Without this the outlined badge is decoration. With it, a reader can tell at
/// a glance which companies have equity they could actually sell.
#[component]
fn Legend() -> Element {
    let listed = group_colour("ftse100");
    rsx! {
        p { class: "flex flex-wrap items-center gap-x-4 gap-y-1 px-1 text-xs text-ink-muted",
            span { class: "inline-flex items-center gap-1.5",
                span {
                    class: "rounded px-1.5 py-0.5 font-semibold",
                    style: "color:{listed};border:1px solid {listed}",
                    "SGE"
                }
                "on a London index, coloured by which"
            }
            span { class: "inline-flex items-center gap-1.5",
                span {
                    class: "rounded border border-dashed border-rule px-1.5 py-0.5 font-semibold text-ink-muted",
                    "OAI"
                }
                "not on a London index"
            }
        }
    }
}

#[component]
fn Toolbar(page: Page, rows: usize, total: usize) -> Element {
    let filters = page.filters.read().clone();
    let mut f = page.filters;
    let layout = filters.layout;

    let options: Vec<(String, String)> = Sort::ALL
        .iter()
        .map(|s| (s.key().to_string(), s.label().to_string()))
        .collect();

    rsx! {
        div { class: "flex flex-wrap items-end justify-between gap-3 rounded-lg border border-rule-soft bg-surface p-3",
            div { class: "w-72",
                Select {
                    label: "Sort",
                    value: filters.sort.key().to_string(),
                    options,
                    onchange: move |key: String| f.write().sort = Sort::from_key(&key),
                }
            }
            div { class: "flex flex-wrap items-center gap-2",
                p {
                    class: "text-sm tabular-nums text-ink-muted",
                    aria_live: "polite",
                    if rows == total {
                        "All {total} companies"
                    } else {
                        "{rows} of {total} companies"
                    }
                }
                div { class: "inline-flex overflow-hidden rounded-md border border-rule",
                    button {
                        r#type: "button",
                        class: if layout == Layout::Register { "bg-ink px-3 py-1.5 text-sm text-on-ink" } else { "px-3 py-1.5 text-sm" },
                        aria_pressed: "{layout == Layout::Register}",
                        onclick: move |_| f.write().layout = Layout::Register,
                        "Register"
                    }
                    button {
                        r#type: "button",
                        class: if layout == Layout::Table { "bg-ink px-3 py-1.5 text-sm text-on-ink" } else { "px-3 py-1.5 text-sm" },
                        aria_pressed: "{layout == Layout::Table}",
                        onclick: move |_| f.write().layout = Layout::Table,
                        "Table"
                    }
                    button {
                        r#type: "button",
                        class: if layout == Layout::Dashboard { "bg-ink px-3 py-1.5 text-sm text-on-ink" } else { "px-3 py-1.5 text-sm" },
                        aria_pressed: "{layout == Layout::Dashboard}",
                        onclick: move |_| f.write().layout = Layout::Dashboard,
                        "Dashboard"
                    }
                }
                ExportButton { page, rows }
            }
        }
    }
}

/// Download the current view as a spreadsheet.
///
/// A `data:` URL on an anchor rather than a click handler building a blob: the
/// browser already knows how to save a link, so this control is a real link
/// that middle-click and "save link as" both work on.
#[component]
fn ExportButton(page: Page, rows: usize) -> Element {
    let register = use_context::<SharedRegister>();
    let filters = page.filters.read().clone();
    let csv = rv2_hiring::csv::export(&register.read(), &filters);
    let href = format!("data:text/csv;charset=utf-8,{}", encode_data(&csv));

    rsx! {
        a {
            class: if rows == 0 { "pointer-events-none rounded-md border border-rule-soft px-3 py-1.5 text-sm text-ink-faint" } else { "rounded-md border border-rule px-3 py-1.5 text-sm hover:bg-ground" },
            href: "{href}",
            download: "uk-tech-hiring-register.csv",
            aria_disabled: "{rows == 0}",
            "Export CSV"
        }
    }
}

/// Percent-encode a `data:` URL payload.
///
/// `#` is the one that matters: a fragment marker inside the payload truncates
/// the file at that byte, and the register carries enough free text to hit one.
fn encode_data(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Which filter emptied the list, most restrictive first.
///
/// Naming one is the whole job: "no companies match these filters" tells a
/// reader nothing they can act on, and with a dozen controls on the page they
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
        section { class: "rounded-lg border border-rule-soft bg-surface p-8 text-center",
            h2 { class: "text-lg font-semibold", "{headline}" }
            p { class: "mx-auto mt-2 max-w-2xl text-sm text-ink-muted", "{explanation}" }
            button {
                r#type: "button",
                class: "mt-4 rounded-md border border-rule px-3 py-1.5 text-sm hover:bg-ground",
                onclick: move |_| page_state.clear(),
                "Clear every filter"
            }
        }
    }
}

/// The ticker badge, carrying its index colour.
///
/// Filled where the company sits on a London index, outlined where it does not.
/// That is the one distinction a reader making an equity decision needs before
/// reading anything else, so it is the loudest thing in the entry.
#[component]
fn Mark(company: Company) -> Element {
    let indices = company.indices();
    let key = indices.first().cloned().unwrap_or_else(|| "targets".into());
    let colour = group_colour(&key);
    let mark = company.mark();
    let listed = company.london_listed();
    let exchange = company.exchange().to_string();
    let title = if listed {
        "On a London index"
    } else {
        "Not on a London index"
    };
    let style = if listed {
        format!("color:{colour};border:1px solid {colour}")
    } else {
        format!("color:{colour};border:1px dashed {colour}")
    };

    rsx! {
        div { class: "flex shrink-0 flex-col items-center",
            span {
                class: "rounded px-2 py-1 font-display text-2xl font-extrabold leading-none tracking-tight",
                style: "{style}",
                title,
                "{mark}"
            }
            if !exchange.is_empty() {
                span { class: "pt-0.5 text-[0.625rem] font-semibold uppercase tracking-wide text-ink-muted",
                    "{exchange}"
                }
            }
        }
    }
}

#[component]
fn Entry(company: Company, page: Page) -> Element {
    let filters = page.filters.read().clone();
    let roles = filters.surviving_roles(&company);
    let show_evidence = filters.litmus.shows_mandate_evidence();
    let total_roles = company.openings.as_ref().map_or(0, |o| o.roles.len());
    let shown = roles.len();
    let summary = if shown == total_roles {
        format!("Show {shown} {}", if shown == 1 { "role" } else { "roles" })
    } else {
        format!("Show {shown} of {total_roles} roles")
    };

    rsx! {
        article { class: "rounded-lg border border-rule-soft bg-surface p-4",
            div { class: "flex flex-wrap items-start justify-between gap-3",
                div { class: "flex min-w-0 gap-3",
                    Mark { company: company.clone() }
                    div { class: "min-w-0",
                        h2 { class: "font-display text-2xl font-bold leading-tight", "{company.name}" }
                        p { class: "mt-0.5 text-sm text-ink-muted",
                            "{company.sector()}"
                            if !company.uk_locations().is_empty() {
                                " in {company.uk_locations()}"
                            }
                            if !company.employee_scale().is_empty() {
                                ", {company.employee_scale()} people"
                            }
                        }
                        if !company.tech_stack_note().is_empty() {
                            p { class: "mt-1 text-sm text-ink-muted", "{company.tech_stack_note()}" }
                        }
                    }
                }
                Actions { company: company.clone(), rust: filters.rust }
            }

            Chips { company: company.clone(), matched: shown }
            Openings { company: company.clone() }
            Depth { company: company.clone() }

            if roles.is_empty() {
                p { class: "mt-2 text-sm text-ink-muted",
                    if total_roles == 0 {
                        "No roles recorded for this company."
                    } else {
                        "None of this company’s {total_roles} recorded roles survive these filters."
                    }
                }
            } else {
                Disclosure { class: "mt-3", open: true, summary,
                    ul { class: "space-y-2",
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
}

/// Every route off an entry, including the ones that exist because the first
/// one failed.
///
/// An entry whose careers link is broken and which offers nothing else is a
/// dead end, so a web search replaces the primary link rather than sitting
/// beside it, and LinkedIn is there regardless.
#[component]
fn Actions(company: Company, rust: bool) -> Element {
    let status = company.link_status();
    let careers = safe_url(&company.careers_url)
        .map(str::to_string)
        .filter(|_| !matches!(status, LinkStatus::Broken | LinkStatus::Missing));
    let board = company.ats.as_ref().map(|a| a.board_url());
    let board = board.as_deref().and_then(safe_url).map(str::to_string);
    let code = company
        .link
        .as_ref()
        .and_then(|l| l.http_code)
        .filter(|_| status != LinkStatus::Live)
        .map(|c| format!("({c})"));
    let linkedin = linkedin_search(&company.name, rust);
    let search = careers_search(&company.name);

    rsx! {
        div { class: "flex shrink-0 flex-col items-end gap-1 text-sm",
            match careers {
                Some(url) => rsx! {
                    a {
                        href: "{url}",
                        target: "_blank",
                        rel: EXT,
                        class: "rounded-md bg-ink px-3 py-1.5 text-on-ink",
                        "Open careers site"
                    }
                },
                None => rsx! {
                    a {
                        href: "{search}",
                        target: "_blank",
                        rel: EXT,
                        class: "rounded-md border border-rule px-3 py-1.5",
                        "Search for careers site"
                    }
                },
            }
            if let Some(board) = board {
                a {
                    href: "{board}",
                    target: "_blank",
                    rel: EXT,
                    class: "text-ink-muted underline",
                    "All roles on {company.feed()}"
                }
            }
            a {
                href: "{linkedin}",
                target: "_blank",
                rel: EXT,
                class: "text-ink-muted underline",
                if rust {
                    "Rust roles on LinkedIn"
                } else {
                    "Engineering roles on LinkedIn"
                }
            }
            Status { label: status.label().to_string(), tone: tone_of(status), detail: code }
        }
    }
}

#[component]
fn Chips(company: Company, matched: usize) -> Element {
    let roles_word = if matched == 1 {
        "role shown"
    } else {
        "roles shown"
    };
    rsx! {
        ul { class: "mt-3 flex flex-wrap gap-1.5",
            for index in company.indices() {
                li { key: "{index}",
                    Badge { colour: group_colour(&index).to_string(), "{group_label(&index)}" }
                }
            }
            if let Some(depth) = company.depth() {
                li {
                    Badge { colour: depth.colour().to_string(), "{depth.label()}" }
                }
            }
            if company.openings.is_none() && company.hires_engineers_in_uk() {
                li {
                    Badge { "Hiring UK engineers, per research" }
                }
            }
            if company.public_equity() {
                li {
                    Badge { "Listed equity, {company.exchange()} {company.ticker()}" }
                }
            }
            if matched > 0 {
                li {
                    Badge { "{matched} {roles_word}" }
                }
            }
            if company.known_rust() {
                li {
                    Badge { "Known Rust use" }
                }
            }
            li {
                Badge { "{company.confidence().label()}" }
            }
        }
    }
}

#[component]
fn Openings(company: Company) -> Element {
    let Some(o) = company.openings.clone() else {
        return rsx! {};
    };
    let feed = company.feed();
    let role_word = if o.uk_engineering == 1 {
        "UK engineering role"
    } else {
        "UK engineering roles"
    };
    let mention_word = if o.uk_rust == 1 {
        "posting mentions"
    } else {
        "postings mention"
    };

    rsx! {
        div { class: "mt-3 space-y-1",
            match o.error.clone() {
                Some(error) => rsx! {
                    p { class: "text-sm text-caution-ink", "Couldn’t read the {feed} job board ({error})." }
                },
                None => rsx! {
                    p { class: "text-sm text-ink",
                        b { "{o.uk_engineering}" }
                        " {role_word} open, out of {o.uk} UK and {o.total} worldwide."
                        if o.uk_rust > 0 {
                            span { class: "text-rust-ink",
                                " {o.uk_rust} UK {mention_word} Rust"
                                if o.uk_rust_in_title > 0 {
                                    ", {o.uk_rust_in_title} in the title"
                                }
                                "."
                            }
                        }
                    }
                },
            }
            if let Some(failed) = o.last_read_failed.clone() {
                p { class: "text-xs text-caution-ink",
                    "Counts are from {o.checked_on}; the read on {failed} failed, so these may be stale."
                }
            }
        }
    }
}

/// How Rust actually features here, where somebody read the descriptions.
///
/// Distinct from the Rust *signal*, which only means the word appeared. A
/// company whose boilerplate lists its whole stack mentions Rust in every
/// posting and may write none, so the evidence sentence is the point and the
/// label alone would mislead.
#[component]
fn Depth(company: Company) -> Element {
    let Some(depth) = company.depth() else {
        return rsx! {};
    };
    let note = company.depth_note().to_string();
    let checked = company.depth_checked_on().to_string();
    let summary = if checked.is_empty() {
        format!("{} — how Rust actually features", depth.label())
    } else {
        format!(
            "{} — how Rust actually features, read {checked}",
            depth.label()
        )
    };

    rsx! {
        Disclosure { class: "mt-2", accent: depth.colour().to_string(), summary,
            p { class: "text-sm text-ink-muted", "{note}" }
        }
    }
}

#[component]
fn RoleLine(role: Role, show_evidence: bool) -> Element {
    let clears = role.clears_essential_filters();
    let mandate = role.mandate.clone();
    let url = safe_url(&role.url).map(str::to_string);

    rsx! {
        div { class: if clears { "rounded-md border border-takeable bg-takeable-wash p-2 shadow-takeable" } else { "rounded-md border border-rule-soft p-2" },
            div { class: "flex flex-wrap items-baseline justify-between gap-2",
                match url {
                    Some(url) => rsx! {
                        a {
                            href: "{url}",
                            target: "_blank",
                            rel: EXT,
                            class: "text-sm font-medium underline",
                            "{role.title}"
                        }
                    },
                    None => rsx! {
                        span { class: "text-sm font-medium", "{role.title}" }
                    },
                }
                span { class: "text-xs text-ink-muted", "{role.location}" }
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
                if role.rust_in_title {
                    Badge { class: "bg-rust-strong text-rust-ink", "Rust role" }
                } else if role.rust {
                    Badge { class: "bg-rust-soft text-rust-ink", "Mentions Rust" }
                }
                if let Some(m) = mandate.as_ref() {
                    Badge { "{m.strength.label()}" }
                    Badge { "{m.level.label()}" }
                }
                if !role.applied_on.is_empty() {
                    Badge { class: "bg-takeable-soft text-takeable-ink", "Applied {role.applied_on}" }
                }
            }
            if !role.work_pattern_note.is_empty() {
                p { class: "mt-1 text-[0.6875rem] leading-snug text-ink-muted",
                    "Working pattern: {role.work_pattern_note}. Engagement: {role.engagement_note}."
                }
            }
            if show_evidence
                && let Some(m) = mandate.as_ref()
                && !m.quotes.is_empty()
            {
                ul { class: "mt-1 space-y-0.5",
                    for (signal , quote) in m.signals.iter().zip(m.quotes.iter()) {
                        li { key: "{quote}", class: "text-[0.6875rem] leading-snug text-ink-muted",
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
    let rust = page.filters.read().rust;
    rsx! {
        div { class: "overflow-x-auto rounded-lg border border-rule-soft bg-surface",
            table { class: "w-full text-sm",
                thead { class: "border-b border-rule-soft bg-ground text-left",
                    tr {
                        th { class: "px-3 py-2", scope: "col", "Ticker" }
                        th { class: "px-3 py-2", scope: "col", "Company" }
                        th { class: "px-3 py-2", scope: "col", "Index" }
                        th { class: "px-3 py-2", scope: "col", "UK locations" }
                        th { class: "px-3 py-2 text-right", scope: "col", "UK engineering roles" }
                        th { class: "px-3 py-2 text-right", scope: "col", "UK postings mentioning Rust" }
                        th { class: "px-3 py-2 text-right", scope: "col", "Roles clearing both filters" }
                        th { class: "px-3 py-2", scope: "col", "Careers site" }
                        th { class: "px-3 py-2", scope: "col", "Job board" }
                        th { class: "px-3 py-2", scope: "col", "Search" }
                    }
                }
                tbody {
                    for company in companies {
                        TableRow { key: "{company.name}", company, page, rust }
                    }
                }
            }
        }
    }
}

#[component]
fn TableRow(company: Company, page: Page, rust: bool) -> Element {
    let filters = page.filters.read().clone();
    let status = company.link_status();
    let careers = safe_url(&company.careers_url)
        .map(str::to_string)
        .filter(|_| !matches!(status, LinkStatus::Broken | LinkStatus::Missing));
    let board = company.ats.as_ref().map(|a| a.board_url());
    let board = board.as_deref().and_then(safe_url).map(str::to_string);
    let clearing = filters
        .surviving_roles(&company)
        .iter()
        .filter(|r| r.clears_essential_filters())
        .count();
    let openings = company.openings.clone();
    let linkedin = linkedin_search(&company.name, rust);
    let search = careers_search(&company.name);

    rsx! {
        tr { class: "border-b border-rule-soft align-top",
            td { class: "px-3 py-2",
                Mark { company: company.clone() }
            }
            td { class: "px-3 py-2",
                div { class: "font-medium", "{company.name}" }
                div { class: "text-xs text-ink-muted", "{company.sector()}" }
            }
            td { class: "px-3 py-2 text-xs",
                div { class: "flex flex-col gap-0.5",
                    for index in company.indices() {
                        span { key: "{index}", style: "color:{group_colour(&index)}",
                            "{group_label(&index)}"
                        }
                    }
                }
            }
            td { class: "px-3 py-2 text-xs", "{company.uk_locations()}" }
            td { class: "px-3 py-2 text-right tabular-nums",
                match openings.as_ref() {
                    Some(o) => rsx! { "{o.uk_engineering}" },
                    None => rsx! {
                        span { class: "text-ink-faint", "Not tracked" }
                    },
                }
            }
            td { class: "px-3 py-2 text-right tabular-nums",
                match openings.as_ref() {
                    Some(o) => rsx! { "{o.uk_rust}" },
                    None => rsx! {
                        span { class: "text-ink-faint", "Not tracked" }
                    },
                }
                if company.known_rust() {
                    div { class: "text-xs text-ink-muted", "Known Rust use" }
                }
            }
            td { class: "px-3 py-2 text-right tabular-nums", "{clearing}" }
            td { class: "px-3 py-2",
                match careers {
                    Some(url) => rsx! {
                        a { href: "{url}", target: "_blank", rel: EXT, class: "underline", "Careers site" }
                    },
                    None => rsx! {
                        a { href: "{search}", target: "_blank", rel: EXT, class: "underline", "Search" }
                    },
                }
                div { class: "pt-0.5",
                    Status { label: status.label().to_string(), tone: tone_of(status) }
                }
            }
            td { class: "px-3 py-2 text-xs",
                match board {
                    Some(url) => rsx! {
                        a { href: "{url}", target: "_blank", rel: EXT, class: "underline", "{company.feed()}" }
                    },
                    None => rsx! {
                        span { class: "text-ink-faint", "{company.feed()}" }
                    },
                }
            }
            td { class: "px-3 py-2 text-xs",
                a { href: "{linkedin}", target: "_blank", rel: EXT, class: "underline", "LinkedIn" }
            }
        }
    }
}
