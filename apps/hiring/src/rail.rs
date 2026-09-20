//! The filter rail.
//!
//! Three groups, and the order is the point. The two **essential** filters come
//! first because they are the ones that decide whether a role is takeable at
//! all. **Litmus Submission** follows, carrying all eight axes of the
//! questionnaire. The register's own facets come last.
//!
//! Every derived value shows its basis. "Global corporates" here came from a
//! headcount band, not from revenue, and a reader who cannot see that cannot
//! disagree with it.

use dioxus::prelude::*;
use rv2_hiring::litmus::{Axis, Challenge, Situation, Timing};
use rv2_hiring::mandate::LevelBand;
use rv2_hiring::model::{MarketReach, OrgScale};
use rv2_hiring::view::{self, FACET_PREVIEW, group_colour};
use rv2_ui::{Checkbox, FieldSet, Swatch};

use crate::{Page, SharedRegister};

#[component]
pub fn Rail(page: Page) -> Element {
    let filters = page.filters.read().clone();
    let active = filters.active_count();
    let mut page_state = page;

    rsx! {
        aside { class: "w-full shrink-0 lg:sticky lg:top-4 lg:w-80",
            details {
                class: "rounded-lg border border-rule-soft bg-surface p-4 [&[open]>summary>.marker]:rotate-90",
                open: true,
                summary { class: "flex cursor-pointer list-none items-baseline justify-between",
                    span { class: "flex items-baseline gap-2",
                        span { class: "marker inline-block transition-transform lg:hidden", aria_hidden: "true", "›" }
                        h2 { class: "text-sm font-semibold", "Filters" }
                        if active > 0 {
                            span { class: "text-xs font-normal text-ink-muted", "({active} on)" }
                        }
                    }
                    if active > 0 {
                        span {
                            role: "button",
                            tabindex: "0",
                            class: "text-xs text-ink-muted underline hover:text-ink",
                            onclick: move |event| {
                                event.stop_propagation();
                                page_state.clear();
                            },
                            "Clear all"
                        }
                    }
                }
                div { class: "space-y-4 pt-3",
                    Essentials { page }
                    Litmus { page }
                    Register_Facets { page }
                    Toggles { page }
                }
            }
        }
    }
}

/// The two filters the reader called essential.
///
/// Both default to off rather than on. On by default would open the page on an
/// almost-empty list, and an empty first impression reads as a broken page
/// rather than as a finding about the market.
#[component]
fn Essentials(page: Page) -> Element {
    let filters = page.filters.read().clone();
    let mut f = page.filters;
    let active = usize::from(filters.remote_only)
        + usize::from(filters.b2b_only)
        + usize::from(filters.mandate_only)
        + usize::from(filters.applied_only);

    rsx! {
        FieldSet {
            legend: "Essential",
            note: "Both apply to roles, not companies. A company stays when one of its roles survives.",
            active,
            Checkbox {
                label: "Remote, at most 25% in the office",
                checked: filters.remote_only,
                basis: "One office day a week passes. Two does not. A stated \"hybrid\" with no number does not.",
                onchange: move |on| f.write().remote_only = on,
            }
            Checkbox {
                label: "B2B, outside IR35",
                checked: filters.b2b_only,
                basis: "A contract that can be taken through your own company. Permanent employment does not pass.",
                onchange: move |on| f.write().b2b_only = on,
            }
            Checkbox {
                label: "Builds a department and teaches",
                checked: filters.mandate_only,
                basis: "Stands up a new unit, hires and grows, teaches, or bridges research and production.",
                onchange: move |on| f.write().mandate_only = on,
            }
            Checkbox {
                label: "Already applied to",
                checked: filters.applied_only,
                basis: "Roles the register records an application against.",
                onchange: move |on| f.write().applied_only = on,
            }
        }
    }
}

const LEVELS: &[(LevelBand, &str)] = &[
    (LevelBand::Director, "Director"),
    (LevelBand::VpGm, "VP / GM"),
    (LevelBand::CLevel, "C-level / Advisory"),
    (LevelBand::HeadLead, "Head, lead or principal"),
];

const MARKETS: &[(MarketReach, &str)] = &[
    (MarketReach::Local, "Local"),
    (MarketReach::Regional, "Regional"),
    (MarketReach::International, "International"),
    (MarketReach::MultiRegion, "Multi-region"),
];

const SCALES: &[(OrgScale, &str)] = &[
    (OrgScale::Under50m, "Under €50M"),
    (OrgScale::From50mTo500m, "€50M – €500M"),
    (OrgScale::Over500m, "€500M+"),
    (OrgScale::GlobalCorporate, "Global corporates"),
];

const SITUATIONS: &[(Situation, &str)] = &[
    (
        Situation::AdvancingSameTrack,
        "Advancing within the same track",
    ),
    (Situation::MovingHigher, "Moving to a higher level"),
    (
        Situation::TransitioningAcrossFunctions,
        "Transitioning across functions",
    ),
    (
        Situation::ShiftingToAdvisory,
        "Shifting toward advisory or board-level roles",
    ),
];

const TIMINGS: &[(Timing, &str)] = &[
    (Timing::Exploring, "Exploring"),
    (Timing::ThreeToSixMonths, "Within 3–6 months"),
    (Timing::OneToThreeMonths, "Within 1–3 months"),
    (Timing::Immediately, "Immediately"),
];

const CHALLENGES: &[(Challenge, &str)] = &[
    (
        Challenge::LimitedInterviewTraction,
        "Limited interview traction",
    ),
    (
        Challenge::ConsideredBelowMyLevel,
        "Being considered below my actual level",
    ),
    (
        Challenge::ProfileNotReflectingScope,
        "Profile not reflecting scope",
    ),
    (
        Challenge::UnclearPositioning,
        "Unclear positioning in the market",
    ),
];

#[component]
fn Litmus(page: Page) -> Element {
    let filters = page.filters.read().clone();
    let active = filters.litmus.active_axes();

    rsx! {
        FieldSet {
            legend: "Litmus Submission",
            note: "Your questionnaire answers, as filters. Each one says whether it reads the register directly or was derived, and from what.",
            active,
            for axis in Axis::ALL {
                AxisBlock { key: "{axis:?}", axis, page }
            }
        }
    }
}

/// One question, the provenance of its answer, and its controls.
///
/// Driven off `Axis::ALL` rather than written out eight times, so the questions
/// and the controls cannot drift apart: a ninth question fails to compile until
/// this match handles it, and the group's note never claims a count that has
/// stopped being true.
///
/// The basis line is required by the type — `Basis::Derived` has no form
/// without a `from` — so an axis cannot render a derivation it refuses to
/// explain. Derived answers are tinted, because a reader deciding whether to
/// trust "Global corporates" needs to see at a glance that nobody recorded it.
#[component]
fn AxisBlock(axis: Axis, page: Page) -> Element {
    let basis = axis.basis();
    rsx! {
        div { class: "pt-2",
            p { class: "text-xs font-medium text-ink", "{axis.title()}" }
            p {
                class: if basis.is_derived() { "pb-1 text-[0.6875rem] leading-snug text-caution-ink" } else { "pb-1 text-[0.6875rem] leading-snug text-ink-muted" },
                "{basis.label()}"
            }
            match axis {
                Axis::Level => rsx! {
                    for (band , label) in LEVELS.iter().copied() {
                        LevelBox { key: "{label}", band, label, page }
                    }
                },
                Axis::Markets => rsx! {
                    for (reach , label) in MARKETS.iter().copied() {
                        MarketBox { key: "{label}", reach, label, page }
                    }
                },
                Axis::Scale => rsx! {
                    for (scale , label) in SCALES.iter().copied() {
                        ScaleBox { key: "{label}", scale, label, page }
                    }
                },
                Axis::Situation => rsx! {
                    for (situation , label) in SITUATIONS.iter().copied() {
                        SituationBox { key: "{label}", situation, label, page }
                    }
                },
                Axis::Timing => rsx! {
                    for (timing , label) in TIMINGS.iter().copied() {
                        TimingBox { key: "{label}", timing, label, page }
                    }
                },
                Axis::Challenge => rsx! {
                    for (challenge , label) in CHALLENGES.iter().copied() {
                        ChallengeBox { key: "{label}", challenge, label, page }
                    }
                },
                Axis::Countries => rsx! {
                    FreeText { page, countries: true }
                },
                Axis::Industries => rsx! {
                    FreeText { page, countries: false }
                },
            }
        }
    }
}

macro_rules! axis_box {
    ($name:ident, $ty:ty, $prop:ident, $field:ident) => {
        #[component]
        fn $name($prop: $ty, label: String, page: Page) -> Element {
            let checked = page.filters.read().litmus.$field.contains(&$prop);
            let mut f = page.filters;
            rsx! {
                Checkbox {
                    label,
                    checked,
                    onchange: move |on| {
                        let list = &mut f.write().litmus.$field;
                        if on {
                            if !list.contains(&$prop) {
                                list.push($prop);
                            }
                        } else {
                            list.retain(|v| *v != $prop);
                        }
                    },
                }
            }
        }
    };
}

axis_box!(LevelBox, LevelBand, band, levels);
axis_box!(MarketBox, MarketReach, reach, markets);
axis_box!(ScaleBox, OrgScale, scale, scales);
axis_box!(SituationBox, Situation, situation, situations);
axis_box!(TimingBox, Timing, timing, timings);
axis_box!(ChallengeBox, Challenge, challenge, challenges);

/// The two free-text axes. Comma-separated, because the questionnaire asks for
/// a list and a reader writing "UK, Ireland" means two countries.
#[component]
fn FreeText(page: Page, countries: bool) -> Element {
    let filters = page.filters.read().clone();
    let value = if countries {
        filters.litmus.countries.join(", ")
    } else {
        filters.litmus.industries.join(", ")
    };
    let mut f = page.filters;

    rsx! {
        div {
            input {
                r#type: "text",
                value: "{value}",
                placeholder: "Comma separated",
                class: "w-full rounded-md border border-rule px-2 py-1 text-sm focus:outline-none focus:ring-2 focus:ring-accent",
                oninput: move |event| {
                    let parsed: Vec<String> = event
                        .value()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    let mut filters = f.write();
                    if countries {
                        filters.litmus.countries = parsed;
                    } else {
                        filters.litmus.industries = parsed;
                    }
                },
            }
        }
    }
}

#[component]
fn Toggles(page: Page) -> Element {
    let filters = page.filters.read().clone();
    let mut f = page.filters;
    let active = usize::from(filters.hiring)
        + usize::from(filters.rust)
        + usize::from(filters.rust_first)
        + usize::from(filters.public_equity)
        + usize::from(filters.live);

    rsx! {
        FieldSet { legend: "Show only", active,
            Checkbox {
                label: "Hiring UK engineers",
                checked: filters.hiring,
                onchange: move |on| f.write().hiring = on,
            }
            Checkbox {
                label: "Rust signal",
                checked: filters.rust,
                basis: "Known Rust use, or at least one UK posting that mentions it.",
                onchange: move |on| f.write().rust = on,
            }
            Checkbox {
                label: "Rust-first, description read",
                checked: filters.rust_first,
                onchange: move |on| f.write().rust_first = on,
            }
            Checkbox {
                label: "Listed equity",
                checked: filters.public_equity,
                basis: "On a public market, which is where share awards can be sold.",
                onchange: move |on| f.write().public_equity = on,
            }
            Checkbox {
                label: "Careers link checked and live",
                checked: filters.live,
                onchange: move |on| f.write().live = on,
            }
        }
    }
}

#[component]
fn Register_Facets(page: Page) -> Element {
    rsx! {
        Facet { page, facet: "groups", legend: "Index" }
        Facet { page, facet: "sectors", legend: "Sector" }
        Facet { page, facet: "feeds", legend: "Job board" }
    }
}

#[component]
fn Facet(page: Page, facet: String, legend: String) -> Element {
    let register = use_context::<SharedRegister>();
    let filters = page.filters.read().clone();
    let options = view::facet_options(&register.read(), &filters, &facet);
    let selected: Vec<String> = match facet.as_str() {
        "groups" => filters.groups.clone(),
        "sectors" => filters.sectors.clone(),
        _ => filters.feeds.clone(),
    };

    let expanded = page.is_expanded(&facet) || options.len() <= FACET_PREVIEW + 2;
    let shown: Vec<_> = options
        .iter()
        .enumerate()
        .filter(|(i, o)| expanded || *i < FACET_PREVIEW || selected.contains(&o.value))
        .map(|(_, o)| o)
        .collect();
    let total = options.len();

    let mut page_state = page;
    let facet_for_expand = facet.clone();

    rsx! {
        FieldSet { legend, active: selected.len(),
            for option in shown {
                FacetBox {
                    key: "{facet}-{option.value}",
                    facet: facet.clone(),
                    value: option.value.clone(),
                    label: option.label.clone(),
                    count: option.count,
                    checked: selected.contains(&option.value),
                    page,
                }
            }
            if !expanded {
                button {
                    r#type: "button",
                    class: "pt-1 text-xs text-ink-muted underline hover:text-ink",
                    onclick: move |_| page_state.expand(&facet_for_expand),
                    "Show all {total}"
                }
            }
        }
    }
}

#[component]
fn FacetBox(
    facet: String,
    value: String,
    label: String,
    count: usize,
    checked: bool,
    page: Page,
) -> Element {
    let mut page_state = page;
    let swatch = (facet == "groups").then(|| group_colour(&value).to_string());
    rsx! {
        div { class: "flex items-center gap-1.5",
            if let Some(colour) = swatch {
                Swatch { colour }
            }
            div { class: "min-w-0 flex-1",
                Checkbox {
                    label,
                    checked,
                    count,
                    onchange: move |on| page_state.toggle_facet(&facet, &value, on),
                }
            }
        }
    }
}
