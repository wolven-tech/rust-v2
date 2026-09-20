//! Standing questions, rendered.
//!
//! Every panel re-answers itself from today's register, so nothing here is a
//! number somebody wrote down. The two counterfactual panels are the reason the
//! view exists: "what you gave up by needing remote" beside "what you gave up
//! by needing a mandate", grouped from the same pool, turns the register's
//! finding from a sentence a reader has to trust into a shape they can argue
//! with.
//!
//! No panel shows a bare number. Every count carries the denominator
//! `rv2_hiring::breakdown` computes for it, and a panel with nothing in it says
//! which of the three empty states it is in — answered and genuinely nought,
//! partially read, or unable to see at all.

use dioxus::prelude::*;
use rv2_hiring::breakdown::{self, Coverage, GroupBy};
use rv2_hiring::dashboard::{Dashboard, Panel, Show};
use rv2_hiring::model::Register;
use rv2_hiring::view::{self, Filters, Sort};
use rv2_ui::Bar;

use crate::{Page, SharedRegister};

/// Render the standing dashboard, or one carried in from a link.
#[component]
pub fn DashboardView(page: Page) -> Element {
    let register = use_context::<SharedRegister>();
    let dashboard = use_hook(Dashboard::standing);

    rsx! {
        section { class: "space-y-4",
            header { class: "flex flex-wrap items-baseline justify-between gap-2",
                h2 { class: "font-display text-3xl font-bold leading-none", "{dashboard.name}" }
                p { class: "text-xs text-ink-muted",
                    "Every panel answers itself from the register as it stands. Run "
                    code { class: "rounded bg-raised px-1", "hiring-register check" }
                    " to move these numbers."
                }
            }
            div { class: "grid gap-4 xl:grid-cols-2",
                for (i , panel) in dashboard.panels.iter().enumerate() {
                    PanelCard {
                        key: "{i}",
                        panel: panel.clone(),
                        register: register.read().companies.len(),
                        page,
                    }
                }
            }
        }
    }
}

/// One standing question and its answer.
///
/// `register` is passed only as a length so the card re-renders when the
/// register changes without cloning 103 companies into every panel's props.
#[component]
fn PanelCard(panel: Panel, register: usize, page: Page) -> Element {
    let _ = register;
    rsx! {
        article { class: "rounded-lg border border-rule-soft bg-surface p-4",
            h3 { class: "text-sm font-semibold text-ink", "{panel.title}" }
            div { class: "pt-2",
                match panel.show {
                    Show::Count => rsx! {
                        CountPanel { filters: panel.filters.clone() }
                    },
                    Show::Breakdown { by } => rsx! {
                        BreakdownPanel { filters: panel.filters.clone(), by }
                    },
                    Show::List { top, by } => rsx! {
                        ListPanel { filters: panel.filters.clone(), top, by }
                    },
                    Show::Table => rsx! {
                        CountPanel { filters: panel.filters.clone() }
                    },
                }
            }
            div { class: "pt-2",
                OpenInRegister { filters: panel.filters.clone(), page }
            }
        }
    }
}

/// A link that loads this panel's filters into the main register view.
///
/// A panel that can only be looked at is a dead end. This is what turns a
/// standing question into the start of an afternoon's work.
#[component]
fn OpenInRegister(filters: Filters, page: Page) -> Element {
    let mut f = page.filters;
    let target = filters.clone();
    rsx! {
        button {
            r#type: "button",
            class: "text-xs text-ink-muted underline hover:text-ink",
            onclick: move |_| {
                let mut next = target.clone();
                next.layout = view::Layout::Register;
                *f.write() = next;
            },
            "Open these in the register"
        }
    }
}

#[component]
fn CountPanel(filters: Filters) -> Element {
    let register = use_context::<SharedRegister>();
    let register = register.read();
    let roles: usize = view::apply(&register, &filters)
        .iter()
        .map(|c| filters.surviving_roles(c).len())
        .sum();
    let b = breakdown::breakdown(&register, &filters, GroupBy::WorkPattern);

    rsx! {
        div {
            p { class: "font-display text-5xl font-bold leading-none text-ink", "{roles}" }
            p { class: "pt-1 text-xs text-ink-muted", "{b.denominator()}" }
            if roles == 0 {
                p { class: "pt-1 text-xs text-caution-ink", "{b.empty_copy()}" }
            }
        }
    }
}

#[component]
fn BreakdownPanel(filters: Filters, by: GroupBy) -> Element {
    let register = use_context::<SharedRegister>();
    let b = breakdown::breakdown(&register.read(), &filters, by);
    let largest = b.largest();

    rsx! {
        div {
            if b.is_empty() {
                p { class: "text-xs text-caution-ink", "{b.empty_copy()}" }
            } else {
                div { class: "space-y-0.5",
                    for bucket in b.buckets.iter().filter(|x| x.count > 0) {
                        Bar {
                            key: "{bucket.label}",
                            label: bucket.label.clone(),
                            count: bucket.count,
                            of: largest,
                            colour: bucket.colour.clone(),
                            emphasis: bucket.wanted,
                        }
                    }
                }
            }
            p { class: "pt-2 text-xs text-ink-muted", "{b.denominator()}" }
            if b.coverage() == Coverage::Partial && !b.is_empty() {
                p { class: "pt-0.5 text-xs text-caution-ink",
                    "Treat this as a floor: part of the register could not be read."
                }
            }
        }
    }
}

#[component]
fn ListPanel(filters: Filters, top: usize, by: Sort) -> Element {
    let register = use_context::<SharedRegister>();
    let register = register.read();
    let mut listed = filters.clone();
    listed.sort = by;

    let rows: Vec<(String, String, String, bool)> = view::apply(&register, &listed)
        .iter()
        .flat_map(|c| {
            listed
                .surviving_roles(c)
                .into_iter()
                .map(|r| {
                    (
                        c.name.clone(),
                        r.title.clone(),
                        r.location.clone(),
                        r.clears_essential_filters(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .take(top)
        .collect();

    let b = breakdown::breakdown(&register, &filters, GroupBy::WorkPattern);

    rsx! {
        div {
            if rows.is_empty() {
                p { class: "text-xs text-caution-ink", "{b.empty_copy()}" }
            } else {
                ul { class: "space-y-1.5",
                    for (company , title , where_it_is , clears) in rows {
                        li {
                            key: "{company}-{title}",
                            class: if clears { "rounded-md border border-takeable bg-takeable-wash p-2 shadow-takeable" } else { "rounded-md border border-rule-soft p-2" },
                            p { class: "text-sm font-medium text-ink", "{title}" }
                            p { class: "text-xs text-ink-muted", "{company} — {where_it_is}" }
                        }
                    }
                }
            }
            p { class: "pt-2 text-xs text-ink-muted", "{b.denominator()}" }
        }
    }
}

/// A count of what a register-wide answer would be, for the masthead.
#[must_use]
pub fn takeable_now(register: &Register) -> usize {
    register
        .companies
        .iter()
        .flat_map(|c| c.openings.iter().flat_map(|o| o.roles.iter()))
        .filter(|r| r.clears_essential_filters())
        .count()
}
