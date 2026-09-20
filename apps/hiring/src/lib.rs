//! The UK tech hiring register as a page.
//!
//! **CSR, file-backed.** The register is compiled into the binary by
//! `rv2_hiring::REGISTER_JSON`, so there is no API, no loading state and no
//! failure mode where the data is missing. Refreshing it means running
//! `hiring-register check` and rebuilding.
//!
//! Every judgement the page renders was made by `rv2-hiring` and is shared with
//! the command line. The page classifies nothing itself, and that is deliberate:
//! the tool that read the job board had the description in hand, and the page
//! only has what the register kept.

#![allow(non_snake_case)]

pub mod rail;
pub mod results;

use dioxus::prelude::*;
use rv2_hiring::model::Register;
use rv2_hiring::view::{self, Filters};

/// The register, shared through context rather than passed as a prop.
///
/// Dioxus props are cloned on every render, and this one is 103 companies with
/// their roles and verdicts attached. Passing it down meant a deep copy per
/// component per keystroke in the search box. A context read borrows instead.
pub type SharedRegister = Signal<Register>;

/// The filter state, plus everything derived from it that more than one part of
/// the page needs.
#[derive(Clone, Copy, PartialEq)]
pub struct Page {
    pub filters: Signal<Filters>,
    /// Which facets the reader has expanded past their first few options.
    pub expanded: Signal<Vec<String>>,
}

impl Page {
    pub fn toggle_facet(&mut self, facet: &str, value: &str, on: bool) {
        let mut filters = self.filters.write();
        let list = match facet {
            "groups" => &mut filters.groups,
            "sectors" => &mut filters.sectors,
            _ => &mut filters.feeds,
        };
        if on {
            if !list.iter().any(|v| v == value) {
                list.push(value.to_string());
            }
        } else {
            list.retain(|v| v != value);
        }
    }

    pub fn expand(&mut self, facet: &str) {
        let mut expanded = self.expanded.write();
        if !expanded.iter().any(|f| f == facet) {
            expanded.push(facet.to_string());
        }
    }

    #[must_use]
    pub fn is_expanded(&self, facet: &str) -> bool {
        self.expanded.read().iter().any(|f| f == facet)
    }

    pub fn clear(&mut self) {
        *self.filters.write() = Filters::default();
    }
}

/// Read the fragment the page was opened with, so a bookmark reopens its view.
fn fragment() -> String {
    web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default()
}

/// Write the fragment back whenever the filters change.
///
/// `set_hash` rather than `replace_state`: this page is served from a static
/// host and opened from `file://` during development, where some browsers
/// refuse `history.replaceState` because the origin is opaque. Setting the hash
/// works in both.
fn set_fragment(value: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_hash(value);
    }
}

#[component]
pub fn App() -> Element {
    let register: SharedRegister =
        use_context_provider(|| Signal::new(rv2_hiring::embedded_register()));
    let page = Page {
        filters: use_signal(|| view::hash::read(&fragment())),
        expanded: use_signal(Vec::new),
    };

    use_effect(move || {
        set_fragment(&view::hash::write(&page.filters.read()));
    });

    let shown = {
        let register = register.read();
        view::apply(&register, &page.filters.read()).len()
    };

    rsx! {
        div { class: "min-h-screen bg-slate-50 text-slate-900",
            Masthead { shown, page }
            div { class: "mx-auto flex max-w-[110rem] flex-col gap-6 px-4 pb-16 lg:flex-row lg:items-start",
                rail::Rail { page }
                main { id: "results", class: "min-w-0 flex-1",
                    results::Results { page }
                }
            }
            Colophon {}
        }
    }
}

#[component]
fn Masthead(shown: usize, page: Page) -> Element {
    let register = use_context::<SharedRegister>();
    let register = register.read();
    let filters = page.filters.read().clone();
    let mut filters_signal = page.filters;

    let tracked: Vec<_> = register
        .companies
        .iter()
        .filter(|c| c.openings.is_some())
        .collect();
    let uk_engineering: usize = tracked
        .iter()
        .filter_map(|c| c.openings.as_ref())
        .map(|o| o.uk_engineering)
        .sum();
    let clears_both = register
        .companies
        .iter()
        .flat_map(|c| c.openings.iter().flat_map(|o| o.roles.iter()))
        .filter(|r| r.clears_essential_filters())
        .count();

    rsx! {
        header { class: "border-b border-slate-200 bg-white",
            div { class: "mx-auto max-w-[110rem] px-4 py-6",
                h1 { class: "text-2xl font-semibold tracking-tight", "UK tech hiring register" }
                p { class: "mt-1 max-w-3xl text-sm text-slate-600",
                    b { "{register.companies.len()}" }
                    " employers. Across the "
                    b { "{tracked.len()}" }
                    " job boards read, "
                    b { "{uk_engineering}" }
                    " UK engineering roles are open, and "
                    b { "{clears_both}" }
                    " of the roles held here are both remote at 25% or less and B2B."
                }
                div { class: "mt-4 flex flex-wrap items-center gap-3",
                    label { class: "flex-1 min-w-64",
                        span { class: "sr-only", "Search companies" }
                        input {
                            id: "q",
                            r#type: "search",
                            autocomplete: "off",
                            spellcheck: false,
                            placeholder: "Company, ticker, city or stack",
                            value: "{filters.q}",
                            class: "w-full rounded-md border border-slate-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-slate-900",
                            oninput: move |event| {
                                filters_signal.write().q = event.value().trim().to_string();
                            },
                        }
                    }
                    p { class: "text-sm tabular-nums text-slate-600",
                        if shown == register.companies.len() {
                            "All {register.companies.len()} companies"
                        } else {
                            "{shown} of {register.companies.len()} companies"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Colophon() -> Element {
    let register = use_context::<SharedRegister>();
    let checked = register
        .read()
        .openings_checked_on
        .clone()
        .unwrap_or_default();
    rsx! {
        footer { class: "border-t border-slate-200 bg-white",
            div { class: "mx-auto max-w-[110rem] space-y-1 px-4 py-6 text-xs text-slate-500",
                p {
                    "Index membership comes from FTSE Russell constituent files as at 30 June 2026. \
                     AIM listings and the target list come from research. Job boards last read {checked}."
                }
                p {
                    "Refresh with "
                    code { class: "rounded bg-slate-100 px-1", "cargo run -p hiring-register -- check" }
                    ", then rebuild the page. Every work pattern, engagement and mandate shown here \
                     was decided when the posting was read, not when the page was opened."
                }
            }
        }
    }
}
