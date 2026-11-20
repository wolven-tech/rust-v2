//! Surfaces and state feedback: cards, headers, empty/loading/error.

use dioxus::prelude::*;

use crate::primitives::{Button, Variant};

/// Inner spacing of a [`Card`].
///
/// Two values, because two is what the pages need: a list of cards that should
/// read as a list, and a single card that is the whole of its section. Hand-made
/// surfaces had been using four (`p-3`, `p-4`, `p-6`, `p-8`), which is the
/// drift and not a design. `None` is a card whose content brings its own edge —
/// a table running to the border.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pad {
    None,
    #[default]
    Default,
    Roomy,
}

impl Pad {
    fn classes(self) -> &'static str {
        match self {
            Pad::None => "",
            Pad::Default => "p-4",
            Pad::Roomy => "p-6",
        }
    }
}

/// The edge of a [`Card`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Surface {
    /// Hairline border, rounded, no shadow.
    #[default]
    Flat,
    /// Square, a 2px ink border and a hard offset shadow: the printed-slab look
    /// of Logo Handoff, Talk Run Card, Sponsor Ledger and Solar Quote. Pairs
    /// with [`crate::Press::Offset`].
    Slab,
}

impl Surface {
    fn classes(self) -> &'static str {
        match self {
            Surface::Flat => "rounded-lg border border-rule-soft",
            Surface::Slab => "border-2 border-ink shadow-[4px_4px_0_var(--color-ink)]",
        }
    }
}

/// A bordered surface.
///
/// Flat by default, with no shadow. Every hand-made surface it replaced was
/// flat, and a product that spends a shadow on meaning — one verdict that must
/// stand out — cannot do so if every card already casts one. [`Surface::Slab`]
/// is that one.
///
/// `class` is for layout inside the card: flex direction, gaps, alignment.
#[component]
pub fn Card(
    #[props(default)] pad: Pad,
    #[props(default)] surface: Surface,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let (pad, edge) = (pad.classes(), surface.classes());
    rsx! {
        div { class: "{edge} bg-surface {pad} {class}", {children} }
    }
}

#[component]
pub fn PageHeader(
    title: String,
    #[props(default)] subtitle: Option<String>,
    #[props(default)] actions: Option<Element>,
) -> Element {
    rsx! {
        header { class: "mb-6 flex items-start justify-between gap-4",
            div {
                h1 { class: "text-2xl font-semibold text-ink", "{title}" }
                if let Some(text) = subtitle {
                    p { class: "mt-1 text-sm text-ink-muted", "{text}" }
                }
            }
            div { class: "flex gap-2", {actions} }
        }
    }
}

/// Shown when a collection is legitimately empty — as distinct from loading
/// (use [`Skeleton`]) or failed (use [`ErrorBanner`]). Conflating the three is
/// the most common way a list UI lies to the user.
#[component]
pub fn EmptyState(
    title: String,
    #[props(default)] description: Option<String>,
    #[props(default)] action: Option<Element>,
) -> Element {
    rsx! {
        div { class: "rounded-lg border border-dashed border-rule p-10 text-center",
            p { class: "text-sm font-medium text-ink", "{title}" }
            if let Some(text) = description {
                p { class: "mt-1 text-sm text-ink-muted", "{text}" }
            }
            div { class: "mt-4 flex justify-center", {action} }
        }
    }
}

#[component]
pub fn Skeleton(#[props(default = 3)] lines: u32) -> Element {
    rsx! {
        div { class: "space-y-2", role: "status", "aria-label": "Loading",
            for index in 0..lines {
                div { key: "{index}", class: "h-4 w-full animate-pulse rounded bg-raised" }
            }
        }
    }
}

#[component]
pub fn ErrorBanner(
    message: String,
    #[props(default)] onretry: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between gap-4 rounded-md border border-fault bg-surface px-4 py-3",
            role: "alert",
            p { class: "text-sm text-fault-ink", "{message}" }
            if let Some(handler) = onretry {
                Button { variant: Variant::Secondary, onclick: handler, "Retry" }
            }
        }
    }
}

/// What a [`Callout`] or [`Receipt`] is telling the reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verdict {
    /// Context worth reading; no judgement.
    #[default]
    Note,
    /// The check passed: it applies, it is ready, it is above the floor.
    Clear,
    /// Not wrong, not finished: missing evidence, stale, open.
    Caution,
    /// It does not apply, it failed, it is below the floor.
    Stop,
}

impl Verdict {
    /// Left-edge colour only, so a card's own all-sides border colour cannot
    /// fight it by stylesheet order.
    pub(crate) fn edge(self) -> &'static str {
        match self {
            Verdict::Note => "border-l-ink-muted",
            Verdict::Clear => "border-l-takeable",
            Verdict::Caution => "border-l-caution",
            Verdict::Stop => "border-l-fault",
        }
    }

    pub(crate) fn ink(self) -> &'static str {
        match self {
            Verdict::Note => "text-ink-muted",
            Verdict::Clear => "text-takeable-ink",
            Verdict::Caution => "text-caution-ink",
            Verdict::Stop => "text-fault-ink",
        }
    }
}

/// A boxed aside with a coloured edge: a verdict strip, a "what this does not
/// cover" note, a missing-evidence warning.
///
/// `title` is the verdict in words, so the edge colour is never the only
/// signal. A `Stop` is announced as an alert.
#[component]
pub fn Callout(
    #[props(default)] verdict: Verdict,
    #[props(default)] title: Option<String>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let role = matches!(verdict, Verdict::Stop).then_some("alert");
    rsx! {
        aside { class: "rounded-r-md border-l-4 {verdict.edge()} bg-raised px-4 py-3 {class}", role,
            if let Some(title) = title {
                p { class: "text-sm font-semibold {verdict.ink()}", "{title}" }
            }
            div { class: "text-sm text-ink-muted", {children} }
        }
    }
}

/// A square outline tag in capitals: `OPEN`, `READY`, `VOID`.
///
/// Reads as a rubber stamp on a record rather than a pill on a list, which is
/// the difference from [`crate::Badge`]. The word carries the verdict; the
/// colour repeats it.
#[component]
pub fn Stamp(
    #[props(default)] verdict: Verdict,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    rsx! {
        span {
            class: "inline-flex w-fit items-center border border-current px-2 py-0.5 text-xs \
                    font-bold uppercase tracking-widest {verdict.ink()} {class}",
            {children}
        }
    }
}

/// Where an assistant's work runs, as the reader is told it.
///
/// A kit-local enum: apps map `rv2_ai::DataBoundary` onto it, so the kit does
/// not depend on `rv2-ai` and a product without it can still say "on device".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Boundary {
    #[default]
    OnDevice,
    /// Sent to a remote model, with the reader's consent.
    RemoteWithConsent,
}

impl Boundary {
    pub fn word(self) -> &'static str {
        match self {
            Self::OnDevice => "On device",
            Self::RemoteWithConsent => "Remote · consented",
        }
    }

    fn scope(self) -> &'static str {
        match self {
            Self::OnDevice => "local only",
            Self::RemoteWithConsent => "sent with consent",
        }
    }

    fn verdict(self) -> Verdict {
        match self {
            Self::OnDevice => Verdict::Clear,
            Self::RemoteWithConsent => Verdict::Caution,
        }
    }
}

/// A pill naming where an assistant runs. The boundary is in the words, not
/// only the colour.
#[component]
pub fn BoundaryPill(#[props(default)] boundary: Boundary) -> Element {
    rsx! {
        span {
            class: "inline-flex w-fit items-center rounded-full border border-current px-2 py-0.5 \
                    font-mono text-[0.6875rem] font-bold uppercase tracking-widest {boundary.verdict().ink()}",
            "{boundary.word()}"
        }
    }
}

/// The line under a drafted answer saying what it was grounded in and where it
/// ran: "Grounded in 4 checklist facts · local only".
#[component]
pub fn GroundingNote(
    cited: usize,
    #[props(default)] boundary: Boundary,
    /// What was cited, plural.
    #[props(default = "facts".to_string())]
    noun: String,
) -> Element {
    rsx! {
        p { class: "font-mono text-xs text-ink-muted",
            "Grounded in {cited} {noun} · {boundary.scope()}"
        }
    }
}

/// The record of something done: a status in mono capitals, a grid of facts,
/// a note and what to do next.
#[component]
pub fn Receipt(
    title: String,
    /// The state in a word or two — `SENT`, `DRAFT`, `PAID`.
    #[props(default)]
    status: Option<String>,
    #[props(default)] verdict: Verdict,
    /// `(term, value)` rows.
    #[props(default)]
    facts: Vec<(String, String)>,
    #[props(default)] actions: Option<Element>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    rsx! {
        article { class: "rounded-lg border border-rule-soft border-l-4 {verdict.edge()} bg-surface p-4 {class}",
            header { class: "flex items-baseline justify-between gap-4",
                crate::Heading { level: 3, size: crate::HeadingSize::Card, "{title}" }
                if let Some(status) = status {
                    span { class: "font-mono text-xs font-semibold uppercase tracking-widest {verdict.ink()}", "{status}" }
                }
            }
            if !facts.is_empty() {
                dl { class: "mt-3 grid grid-cols-1 gap-px overflow-hidden rounded-md border border-rule-soft bg-rule-soft sm:grid-cols-3",
                    for (term , value) in facts {
                        div { key: "{term}", class: "bg-surface px-3 py-2",
                            dt { class: "font-mono text-xs uppercase tracking-wide text-ink-muted", "{term}" }
                            dd { class: "text-sm font-medium tabular-nums text-ink", "{value}" }
                        }
                    }
                }
            }
            div { class: "mt-3 text-sm text-ink-muted", {children} }
            if let Some(actions) = actions {
                div { class: "mt-4 flex flex-wrap gap-2", {actions} }
            }
        }
    }
}
