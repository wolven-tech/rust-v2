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

/// The id the search box carries, so the `/` shortcut can find it.
const SEARCH_ID: &str = "q";

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
        focus_search();
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
/// `replaceState` rather than setting the hash: assigning to `location.hash`
/// pushes a history entry, so typing five characters into the search box left
/// five entries the reader has to press Back through to escape the page.
///
/// It falls back to assigning the hash because a page opened from `file://` has
/// an opaque origin, and some browsers refuse `replaceState` there. Losing the
/// clean history is better than losing the bookmarkable URL.
fn write_fragment(value: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let url = if value.is_empty() {
        window
            .location()
            .pathname()
            .unwrap_or_else(|_| "/".to_string())
    } else {
        format!("#{value}")
    };
    if window
        .history()
        .and_then(|h| h.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url)))
        .is_err()
    {
        let _ = window.location().set_hash(value);
    }
}

/// Move focus into the search box.
fn focus_search() {
    if let Some(element) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(SEARCH_ID))
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = element.focus();
    }
}

/// Whether the reader is currently typing into something.
///
/// The `/` shortcut must not steal a slash from a text field — a reader
/// searching for "and/or" would lose the character and their place at once.
fn typing_somewhere() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.active_element())
        .is_some_and(|e| matches!(e.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT"))
}

use wasm_bindgen::JsCast;

#[component]
pub fn App() -> Element {
    let register: SharedRegister =
        use_context_provider(|| Signal::new(rv2_hiring::embedded_register()));
    let page = Page {
        filters: use_signal(|| view::hash::read(&fragment())),
        expanded: use_signal(Vec::new),
    };

    use_effect(move || {
        write_fragment(&view::hash::write(&page.filters.read()));
    });

    let shown = {
        let register = register.read();
        view::apply(&register, &page.filters.read()).len()
    };

    rsx! {
        div {
            class: "min-h-screen bg-ground text-ink",
            tabindex: "-1",
            autofocus: true,
            onkeydown: move |event| {
                if event.key() == Key::Character("/".to_string()) && !typing_somewhere() {
                    event.prevent_default();
                    focus_search();
                }
            },
            a {
                class: "sr-only focus:not-sr-only focus:absolute focus:left-2 focus:top-2 focus:z-50 focus:rounded-md focus:bg-ink focus:px-3 focus:py-2 focus:text-on-ink",
                href: "#results",
                "Skip to results"
            }
            Masthead { shown, page }
            div { class: "mx-auto flex max-w-[110rem] flex-col gap-6 px-4 pb-16 lg:flex-row lg:items-start",
                rail::Rail { page }
                main { id: "results", tabindex: "-1", class: "min-w-0 flex-1",
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

    let tracked = register
        .companies
        .iter()
        .filter(|c| c.openings.is_some())
        .count();
    let uk_engineering: usize = register
        .companies
        .iter()
        .filter_map(|c| c.openings.as_ref())
        .map(|o| o.uk_engineering)
        .sum();
    let uk_rust: usize = register
        .companies
        .iter()
        .filter_map(|c| c.openings.as_ref())
        .map(|o| o.uk_rust)
        .sum();
    let clears_both = register
        .companies
        .iter()
        .flat_map(|c| c.openings.iter().flat_map(|o| o.roles.iter()))
        .filter(|r| r.clears_essential_filters())
        .count();
    let total = register.companies.len();

    rsx! {
        header { class: "border-b border-rule-soft bg-surface",
            div { class: "mx-auto max-w-[110rem] px-4 py-6",
                h1 { class: "font-display text-5xl font-bold leading-none", "UK tech hiring register" }
                p { class: "mt-1 max-w-4xl text-sm text-ink-muted",
                    b { "{total}" }
                    " employers. Across the "
                    b { "{tracked}" }
                    " job boards read, "
                    b { "{uk_engineering}" }
                    " UK engineering roles are open and "
                    b { "{uk_rust}" }
                    " UK postings mention Rust. "
                    b { "{clears_both}" }
                    " of the roles held here are both remote at 25% or less and B2B."
                }
                div { class: "mt-4 flex flex-wrap items-center gap-3",
                    label { class: "relative min-w-64 flex-1",
                        span { class: "sr-only", "Search companies" }
                        input {
                            id: SEARCH_ID,
                            r#type: "search",
                            autocomplete: "off",
                            spellcheck: false,
                            placeholder: "Company, ticker, city or stack",
                            value: "{filters.q}",
                            class: "w-full rounded-md border border-rule py-2 pl-3 pr-10 text-sm focus:outline-none focus:ring-2 focus:ring-accent",
                            oninput: move |event| {
                                filters_signal.write().q = event.value().trim().to_string();
                            },
                            onkeydown: move |event| {
                                if event.key() == Key::Escape {
                                    filters_signal.write().q = String::new();
                                }
                            },
                        }
                        kbd {
                            class: "pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 rounded border border-rule px-1.5 text-xs text-ink-faint",
                            aria_hidden: "true",
                            "/"
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
    let register = register.read();
    let openings = register.openings_checked_on.clone().unwrap_or_default();
    let links = register.links_checked_on.clone().unwrap_or_default();
    let live = register
        .companies
        .iter()
        .filter(|c| c.link_status() == rv2_hiring::model::LinkStatus::Live)
        .count();

    rsx! {
        footer { class: "border-t border-rule-soft bg-surface",
            div { class: "mx-auto max-w-[110rem] space-y-1 px-4 py-6 text-xs text-ink-muted",
                p {
                    "Index membership comes from FTSE Russell constituent files as at 30 June 2026. \
                     AIM listings and the target list come from research. Job boards last read {openings}. \
                     {live} careers links answered when checked {links}."
                }
                p {
                    "Refresh with "
                    code { class: "rounded bg-raised px-1", "cargo run -p hiring-register -- check" }
                    ", then rebuild the page. Every work pattern, engagement and mandate shown here \
                     was decided when the posting was read, not when the page was opened."
                }
            }
        }
    }
}
