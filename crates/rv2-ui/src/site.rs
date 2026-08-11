//! Marketing-site composites.
//!
//! Scope was set by auditing a real target page (chargewindow-web.fly.dev):
//! nav, hero, feature-card grid, step list, pricing block, cost breakdown, FAQ,
//! footer. That page uses no modal, tab, tooltip or carousel, so this kit has
//! none — an unused component is a maintenance cost and bundle weight with no
//! offsetting benefit. Add them when a page actually needs them.

use dioxus::prelude::*;

use crate::primitives::StepMarker;

/// One entry in a [`NavBar`] or [`Footer`] column.
#[derive(Debug, Clone, PartialEq)]
pub struct NavItem {
    pub label: String,
    pub href: String,
    /// Opens in a new tab with `rel="noopener noreferrer"`.
    pub external: bool,
}

impl NavItem {
    pub fn new(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: href.into(),
            external: false,
        }
    }

    pub fn external(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: href.into(),
            external: true,
        }
    }
}

/// Site header: brand, links, and a call-to-action slot.
///
/// The nav is wrapped in `<nav aria-label>` so a screen-reader user can jump
/// to it, and the link list is a real `<ul>` — assistive technology announces
/// "list, 3 items", which a row of bare `<a>`s does not.
#[component]
pub fn NavBar(
    brand: String,
    #[props(default = "/".to_string())] brand_href: String,
    #[props(default)] items: Vec<NavItem>,
    /// Trailing call-to-action, e.g. a `LinkButton`.
    #[props(default)]
    action: Option<Element>,
) -> Element {
    rsx! {
        header { class: "border-b border-slate-200 bg-white",
            div { class: "mx-auto flex max-w-5xl items-center justify-between gap-6 px-6 py-4",
                a { class: "text-sm font-semibold tracking-tight text-slate-900", href: "{brand_href}", "{brand}" }
                nav { "aria-label": "Main",
                    ul { class: "flex flex-wrap items-center gap-6",
                        for item in items.iter() {
                            li {
                                a {
                                    class: "text-sm text-slate-600 hover:text-slate-900",
                                    href: "{item.href}",
                                    target: item.external.then_some("_blank"),
                                    rel: item.external.then_some("noopener noreferrer"),
                                    "{item.label}"
                                }
                            }
                        }
                    }
                }
                if let Some(action) = action {
                    div { {action} }
                }
            }
        }
    }
}

/// Above-the-fold headline block.
///
/// The heading is a literal `<h1>` — a marketing page has exactly one, and it
/// belongs here.
#[component]
pub fn Hero(
    headline: String,
    #[props(default)] subheadline: Option<String>,
    #[props(default)] eyebrow: Option<String>,
    /// Call-to-action row.
    #[props(default)]
    actions: Option<Element>,
    /// Illustration or screenshot, rendered beside the copy on wide screens.
    #[props(default)]
    media: Option<Element>,
) -> Element {
    rsx! {
        div { class: "grid items-center gap-10 lg:grid-cols-2",
            div { class: "flex flex-col gap-5",
                if let Some(eyebrow) = eyebrow {
                    p { class: "text-xs font-medium uppercase tracking-widest text-slate-500", "{eyebrow}" }
                }
                h1 { class: "text-4xl font-semibold tracking-tight text-slate-900 sm:text-5xl", "{headline}" }
                if let Some(subheadline) = subheadline {
                    p { class: "text-lg text-slate-600", "{subheadline}" }
                }
                if let Some(actions) = actions {
                    div { class: "flex flex-wrap items-center gap-3 pt-2", {actions} }
                }
            }
            if let Some(media) = media {
                div { class: "flex justify-center lg:justify-end", {media} }
            }
        }
    }
}

/// Numbered card in a feature grid.
#[component]
pub fn FeatureCard(
    title: String,
    body: String,
    #[props(default)] step: Option<u8>,
    /// Trailing link, e.g. an `ArrowLink`.
    #[props(default)]
    action: Option<Element>,
) -> Element {
    rsx! {
        div { class: "flex h-full flex-col gap-3 rounded-lg border border-slate-200 bg-white p-6",
            if let Some(step) = step {
                StepMarker { n: step }
            }
            h3 { class: "text-base font-semibold text-slate-900", "{title}" }
            p { class: "flex-1 text-sm text-slate-600", "{body}" }
            if let Some(action) = action {
                div { class: "pt-1", {action} }
            }
        }
    }
}

/// One step of a numbered process.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub title: String,
    pub body: String,
}

impl Step {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
        }
    }
}

/// Ordered "how it works" list.
///
/// A real `<ol>`: the order carries meaning, and the numbers are generated by
/// [`StepMarker`] from the index so they can never drift out of sequence when
/// a step is inserted.
#[component]
pub fn StepList(steps: Vec<Step>) -> Element {
    rsx! {
        ol { class: "flex flex-col gap-6",
            for (index, step) in steps.iter().enumerate() {
                li { class: "flex gap-4",
                    StepMarker { n: (index + 1) as u8 }
                    div { class: "flex flex-col gap-1",
                        h3 { class: "text-base font-semibold text-slate-900", "{step.title}" }
                        p { class: "text-sm text-slate-600", "{step.body}" }
                    }
                }
            }
        }
    }
}

/// A `label → value` row, for cost breakdowns and spec tables.
#[derive(Debug, Clone, PartialEq)]
pub struct Fact {
    pub label: String,
    pub value: String,
    /// Renders bold with a top rule — for the total row.
    pub emphasis: bool,
}

impl Fact {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            emphasis: false,
        }
    }

    pub fn total(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            emphasis: true,
        }
    }
}

/// Itemised figures.
///
/// A `<dl>` rather than a `<table>`: these are name/value pairs, not tabular
/// data with meaningful rows *and* columns. Values are `tabular-nums` so
/// digits align down the column.
#[component]
pub fn FactList(facts: Vec<Fact>, #[props(default)] class: String) -> Element {
    rsx! {
        dl { class: "flex flex-col gap-2 {class}",
            for fact in facts.iter() {
                div {
                    class: if fact.emphasis {
                        "flex items-baseline justify-between gap-4 border-t border-slate-200 pt-2 font-semibold text-slate-900"
                    } else {
                        "flex items-baseline justify-between gap-4"
                    },
                    dt { class: "text-sm text-slate-600", "{fact.label}" }
                    dd { class: "text-sm tabular-nums", "{fact.value}" }
                }
            }
        }
    }
}

/// One-off or subscription price block.
#[component]
pub fn PricingCard(
    price: String,
    #[props(default)] cadence: Option<String>,
    #[props(default)] title: Option<String>,
    #[props(default)] features: Vec<String>,
    #[props(default)] action: Option<Element>,
    #[props(default)] note: Option<String>,
) -> Element {
    rsx! {
        div { class: "flex flex-col gap-5 rounded-lg border border-slate-200 bg-white p-8",
            if let Some(title) = title {
                h3 { class: "text-base font-semibold text-slate-900", "{title}" }
            }
            div { class: "flex items-baseline gap-2",
                span { class: "text-4xl font-semibold tracking-tight tabular-nums text-slate-900", "{price}" }
                if let Some(cadence) = cadence {
                    span { class: "text-sm text-slate-600", "{cadence}" }
                }
            }
            if !features.is_empty() {
                ul { class: "flex flex-col gap-2",
                    for feature in features.iter() {
                        li { class: "flex items-start gap-2 text-sm text-slate-600",
                            span { class: "text-slate-900", "aria-hidden": "true", "✓" }
                            "{feature}"
                        }
                    }
                }
            }
            if let Some(action) = action {
                div { {action} }
            }
            if let Some(note) = note {
                p { class: "text-xs text-slate-500", "{note}" }
            }
        }
    }
}

/// One question/answer pair.
#[derive(Debug, Clone, PartialEq)]
pub struct QandA {
    pub question: String,
    pub answer: String,
}

impl QandA {
    pub fn new(question: impl Into<String>, answer: impl Into<String>) -> Self {
        Self {
            question: question.into(),
            answer: answer.into(),
        }
    }
}

/// Expandable FAQ list.
///
/// Built on native `<details>`/`<summary>`, which gives correct expand/collapse
/// semantics, keyboard operation and in-page find with **zero JavaScript** —
/// so it works in the SSG build before (or without) hydration. A hand-rolled
/// accordion would need state, an effect, and `aria-expanded` wiring to reach
/// the same place.
#[component]
pub fn Faq(items: Vec<QandA>) -> Element {
    rsx! {
        div { class: "divide-y divide-slate-200 border-y border-slate-200",
            for item in items.iter() {
                details { class: "group py-4",
                    summary {
                        class: "flex cursor-pointer list-none items-center justify-between gap-4 \
                                text-base font-medium text-slate-900 focus-visible:outline-none \
                                focus-visible:underline",
                        "{item.question}"
                        span {
                            class: "shrink-0 text-slate-400 transition-transform group-open:rotate-45",
                            "aria-hidden": "true",
                            "+"
                        }
                    }
                    p { class: "pt-3 text-sm text-slate-600", "{item.answer}" }
                }
            }
        }
    }
}

/// A titled column of links in the [`Footer`].
#[derive(Debug, Clone, PartialEq)]
pub struct FooterColumn {
    pub title: String,
    pub items: Vec<NavItem>,
}

impl FooterColumn {
    pub fn new(title: impl Into<String>, items: Vec<NavItem>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }
}

/// Site footer.
#[component]
pub fn Footer(
    #[props(default)] columns: Vec<FooterColumn>,
    /// Attribution or copyright line.
    #[props(default)]
    note: Option<Element>,
) -> Element {
    rsx! {
        footer { class: "border-t border-slate-200 bg-white",
            div { class: "mx-auto max-w-5xl px-6 py-12",
                if !columns.is_empty() {
                    div { class: "grid grid-cols-2 gap-8 sm:grid-cols-3 lg:grid-cols-4",
                        for column in columns.iter() {
                            div {
                                h2 { class: "text-xs font-semibold uppercase tracking-widest text-slate-500",
                                    "{column.title}"
                                }
                                ul { class: "mt-3 flex flex-col gap-2",
                                    for item in column.items.iter() {
                                        li {
                                            a {
                                                class: "text-sm text-slate-600 hover:text-slate-900",
                                                href: "{item.href}",
                                                target: item.external.then_some("_blank"),
                                                rel: item.external.then_some("noopener noreferrer"),
                                                "{item.label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(note) = note {
                    div { class: "mt-10 border-t border-slate-200 pt-6 text-xs text-slate-500", {note} }
                }
            }
        }
    }
}
