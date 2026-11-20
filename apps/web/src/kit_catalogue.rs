//! The third part of `/kit`: the catalogue's sand and water pieces — figures
//! that arrive on scroll, popovers, run timers, the assistant boundary, ledgers,
//! citations, an attention queue and a sparkline.
//!
//! A coverage fixture like the rest of the page. The timer has no clock here:
//! the kit never ticks, so the buttons advance it the way an app's interval
//! would.

use dioxus::prelude::*;
use rv2_ui::{
    Arrive, AttentionQueue, Bar, Boundary, BoundaryPill, Button, Citation, Container, Dated,
    Eyebrow, Fact, FactList, Grid, GroundingNote, Heading, HelpTip, Lean, LedgerCell, LedgerColumn,
    LedgerRow, LedgerTable, Part, QueueItem, RowMenu, Section, SectionTimer, SegmentFlow,
    SegmentedControl, Size, Sparkline, SplitBar, Tally, Text, Tone, Variant, Verdict,
};

#[component]
pub fn KitCatalogue() -> Element {
    rsx! {
        Section { id: "figures",
            Container {
                Eyebrow { "Default theme" }
                Heading { "Figures that arrive" }
                Text { class: "mt-2", tone: Tone::Muted,
                    "Counts and bars settle as they scroll in. The number is text from the first paint; with reduced motion nothing moves."
                }
                Figures {}
            }
        }
        Section { id: "running",
            Container {
                Heading { "Running and asking" }
                Running {}
            }
        }
        Section { id: "ledgers",
            Container {
                Heading { "Ledgers and queues" }
                Ledgers {}
            }
        }
    }
}

#[component]
fn Figures() -> Element {
    rsx! {
        Grid { columns: 2, class: "mt-6 items-start gap-8",
            div { class: "space-y-4",
                p { class: "text-5xl font-semibold text-ink", Tally { value: 174 } }
                Text { tone: Tone::Muted, "roles counted this week" }
                Bar { label: "Remote", count: 92, of: 120, arrive: Arrive::Grow }
                Bar { label: "Hybrid", count: 120, of: 120, arrive: Arrive::Grow, emphasis: true }
                Bar { label: "On site", count: 31, of: 120, arrive: Arrive::Grow }
            }
            div { class: "space-y-6",
                SplitBar {
                    label: "Session cost",
                    arrive: Arrive::Grow,
                    parts: vec![
                        Part::new("Energy", 4.2, "£4.20", "var(--color-takeable)"),
                        Part::new("Fees", 1.1, "£1.10", "var(--color-caution)"),
                        Part::new("VAT", 0.27, "£0.27", "var(--color-ink-muted)"),
                    ],
                }
                div { class: "flex items-end gap-4",
                    div { class: "w-40", Sparkline { values: vec![61.0, 60.0, 62.0, 59.0, 58.0, 57.0], label: "Resting heart rate, falling over 6 weeks" } }
                    p { class: "text-sm text-ink", "57 bpm" }
                }
            }
        }
    }
}

#[component]
fn Running() -> Element {
    let mut elapsed = use_signal(|| 50_u32);
    let mut tone = use_signal(|| "plain".to_string());
    rsx! {
        Grid { columns: 2, class: "mt-6 items-start gap-8",
            SectionTimer {
                section: 1,
                sections: 5,
                title: "The demo",
                cue: "Open the ledger before you say the number.",
                next: "Questions",
                elapsed_secs: elapsed(),
                planned_secs: 120,
                Button { size: Size::Small, onclick: move |_| elapsed += 15, "+15s" }
                Button { size: Size::Small, variant: Variant::Outline, onclick: move |_| elapsed.set(0), "Restart" }
            }
            div { class: "space-y-4",
                div { class: "flex items-center gap-3",
                    Heading { level: 3, "Local composer" }
                    BoundaryPill {}
                    HelpTip { id: "kit-scope-help", label: "Explain what counts as a scope change", title: "What counts as a scope change?",
                        "A new format, version, cutdown or output beyond the written scope."
                    }
                }
                SegmentedControl {
                    label: "Tone",
                    options: vec![("plain".into(), "Plain and direct".into()), ("warm".into(), "Warm and brief".into()), ("formal".into(), "Formal".into())],
                    value: tone(),
                    onchange: move |next| tone.set(next),
                    flow: SegmentFlow::StackNarrow,
                }
                GroundingNote { cited: 4, noun: "checklist facts" }
                GroundingNote { cited: 2, boundary: Boundary::RemoteWithConsent }
                FactList {
                    facts: vec![
                        Fact::new("3 slots × £30", "£90"),
                        Fact::total("Computed", "£90"),
                        Fact::chosen("Your quote", "£85"),
                    ],
                }
            }
        }
    }
}

#[component]
fn Ledgers() -> Element {
    let mut voided = use_signal(|| false);
    let void_button = rsx! {
        RowMenu { id: "kit-row-refund",
            Button { variant: Variant::Ghost, size: Size::Small, onclick: move |_| voided.toggle(), if voided() { "Restore entry" } else { "Void entry" } }
            Button { variant: Variant::Ghost, size: Size::Small, "Edit note" }
        }
    };
    let refund = LedgerRow::new(
        "refund",
        vec![
            LedgerCell::new("Refund").note("12 Nov"),
            LedgerCell::new("+£40.00").lean(Lean::Good),
            "£1,140.00".into(),
        ],
    )
    .action(void_button);
    let refund = if voided() { refund.voided() } else { refund };
    rsx! {
        div { class: "mt-6 space-y-8",
            LedgerTable {
                caption: "Current account activity",
                columns: vec![LedgerColumn::text("Entry"), LedgerColumn::figure("Amount"), LedgerColumn::figure("Balance")],
                rows: vec![
                    LedgerRow::new("rent", vec![LedgerCell::new("Rent").note("1 Nov").href("#ledgers"), LedgerCell::new("-£900.00").lean(Lean::Bad), "£1,100.00".into()]),
                    refund,
                    LedgerRow::new("q4", vec!["Quarter end".into(), "".into(), "£1,140.00".into()]).milestone(),
                ],
                total: LedgerRow::new("net", vec!["Net this month".into(), "-£860.00".into(), "".into()]),
            }
            Citation { source: "the school's inspection report", date_iso: "2026-09-24", date_text: "24 Sep 2026", dated: Dated::Checked, href: "https://reports.ofsted.gov.uk/" }
            AttentionQueue {
                label: "Needs review today",
                empty: "Nothing waiting.",
                numbered: true,
                items: vec![
                    QueueItem::go("sam", "Sam Okafor", "3 sessions missed since Monday", "Review", "#ledgers"),
                    QueueItem::go("ria", "Ria Patel", "New personal best on deadlift", "Open", "#ledgers"),
                    QueueItem::state("billing", "Billing export", "Card on file expires Friday", "Blocked", Verdict::Stop),
                ],
            }
        }
    }
}
