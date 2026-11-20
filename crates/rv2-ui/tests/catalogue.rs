//! The catalogue's sand and water pieces, rendered. Each test pins the part of
//! the contract that holds without sight of the page.

use dioxus::prelude::*;
use rv2_ui::{
    Arrive, AttentionQueue, Bar, Boundary, BoundaryPill, Citation, ClockDirection, Dated, Fact,
    FactList, GroundingNote, HelpTip, Lean, LedgerCell, LedgerColumn, LedgerRow, LedgerTable, Pace,
    Part, QueueItem, RowMenu, SectionTimer, SegmentFlow, SegmentedControl, Sparkline, SplitBar,
    Tally, TimerReadout, Verdict, clock, pace, spark_points,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn a_tally_keeps_the_exact_number_as_text_and_hides_the_counter() {
    let html = render(|| rsx! { Tally { value: 174 } });
    assert!(
        html.contains(r#"<span class="rv2-tally-text">174</span>"#),
        "{html}"
    );
    assert!(html.contains(r#"aria-hidden="true""#), "{html}");
    assert!(html.contains("--to:174"), "{html}");
}

#[test]
fn a_tally_shows_its_text_until_the_stylesheet_swaps_in_the_counter() {
    let zero = render(|| rsx! { Tally { value: 0 } });
    assert!(
        zero.contains(r#"<span class="rv2-tally-text">0</span>"#),
        "{zero}"
    );
    assert!(zero.contains("--to:0"), "{zero}");
    // Only arrive.css may hide the text copy, or no page could ever show it.
    assert!(!zero.contains("sr-only"), "{zero}");
    let css = include_str!("../assets/arrive.css");
    let rule = css
        .split(".rv2-tally-text {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .expect("arrive.css hides the text copy");
    assert!(rule.contains("clip: rect(0, 0, 0, 0)"), "{rule}");
}

#[test]
fn a_tally_past_the_counter_range_still_says_the_whole_number() {
    let html = render(|| rsx! { Tally { value: u32::MAX } });
    assert!(html.contains(&format!(">{}</span>", u32::MAX)), "{html}");
    assert!(html.contains(&format!("--to:{}", i32::MAX)), "{html}");
}

#[test]
fn growing_marks_opt_in_and_stagger_their_parts() {
    let html = render(|| {
        rsx! {
            Bar { label: "Roles", count: 3, of: 6, arrive: Arrive::Grow }
            SplitBar {
                label: "Cost",
                arrive: Arrive::Grow,
                parts: vec![Part::new("Energy", 3.0, "£3", "red"), Part::new("Fees", 1.0, "£1", "blue")],
            }
        }
    });
    assert_eq!(html.matches("rv2-grow").count(), 3, "{html}");
    assert!(html.contains("--i:1"), "{html}");
    let still = render(|| rsx! { Bar { label: "Roles", count: 3, of: 6 } });
    assert!(!still.contains("rv2-grow"), "{still}");
}

#[test]
fn scroll_figures_finish_inside_entry_and_move_only_without_reduced_motion() {
    let css = include_str!("../assets/arrive.css");
    let ranges: Vec<String> = css
        .split("animation-range:")
        .skip(1)
        .map(|rest| {
            rest.split(';')
                .next()
                .unwrap_or_default()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    assert_eq!(
        ranges,
        [
            "entry 0% entry 100%",
            "entry min(30%, calc(var(--i, 0) * 8%)) entry min(100%, calc(70% + (var(--i, 0) * 8%)))",
        ],
        "every range ends no later than entry 100%, whatever the stagger"
    );

    let (before, rest) = css
        .split_once("@supports (animation-timeline: view())")
        .expect("motion sits behind @supports");
    let (block, after) = rest
        .split_once("@keyframes")
        .expect("keyframes follow the gated block");
    let (_, gated) = block
        .split_once("@media (prefers-reduced-motion: no-preference)")
        .expect("motion sits under no-preference");
    assert_eq!(gated.matches("animation-timeline: view();").count(), 2);
    for outside in [before, after] {
        assert!(!outside.contains("animation-timeline"), "{outside}");
        assert!(!outside.contains("animation:"), "{outside}");
    }
}

#[test]
fn a_row_menu_is_a_named_popover_button_and_not_an_aria_menu() {
    let html = render(|| rsx! { RowMenu { id: "row-7", "Edit" } });
    assert!(html.contains(r#"popovertarget="row-7""#), "{html}");
    assert!(html.contains(r#"aria-label="More actions""#), "{html}");
    assert!(html.contains(r#"popover="auto""#), "{html}");
    assert!(html.contains(r#"data-align="end""#), "{html}");
    assert!(!html.contains(r#"role="menu""#), "{html}");
}

#[test]
fn a_help_tip_is_a_note_named_by_its_title() {
    let html = render(|| {
        rsx! { HelpTip { id: "scope-help", label: "Explain scope change", title: "What counts?", "New format or version." } }
    });
    assert!(html.contains(r#"role="note""#), "{html}");
    assert!(
        html.contains(r#"aria-labelledby="scope-help-tip-title""#),
        "{html}"
    );
    assert!(html.contains(r#"id="scope-help-tip-title""#), "{html}");
    assert!(
        html.contains(r#"aria-label="Explain scope change""#),
        "{html}"
    );
}

#[test]
fn pace_is_over_only_strictly_past_the_plan() {
    assert_eq!(pace(10, 100, 80), Pace::OnPace);
    assert_eq!(pace(80, 100, 80), Pace::Closing);
    assert_eq!(pace(100, 100, 80), Pace::Closing);
    assert_eq!(pace(101, 100, 80), Pace::Over);
}

#[test]
fn clocks_read_minutes_and_seconds_then_hours() {
    assert_eq!(clock(5), "0:05");
    assert_eq!(clock(125), "2:05");
    assert_eq!(clock(3_725), "1:02:05");
}

#[test]
fn a_timer_keeps_its_digits_quiet_and_announces_only_the_pace() {
    let html = render(|| {
        rsx! { TimerReadout { label: "Section", elapsed_secs: 130, planned_secs: 120, direction: ClockDirection::Down } }
    });
    assert!(html.contains(r#"role="timer""#), "{html}");
    assert!(html.contains("+0:10"), "{html}");
    assert!(html.contains(r#"role="status""#), "{html}");
    assert!(html.contains("Over time"), "{html}");
    assert!(!html.contains("aria-live"), "{html}");
}

#[test]
fn a_section_timer_says_where_it_is_in_the_run() {
    let html = render(|| {
        rsx! { SectionTimer { section: 1, sections: 5, title: "Demo", elapsed_secs: 30, planned_secs: 120, next: "Questions" } }
    });
    assert!(html.contains("Section 2 of 5"), "{html}");
    assert!(html.contains(r#"role="meter""#), "{html}");
    assert!(html.contains("Next · Questions"), "{html}");
}

#[test]
fn the_assistant_boundary_is_said_in_words() {
    let html = render(|| {
        rsx! {
            BoundaryPill {}
            GroundingNote { cited: 4, noun: "checklist facts" }
            GroundingNote { cited: 2, boundary: Boundary::RemoteWithConsent }
        }
    });
    assert!(html.contains("On device"), "{html}");
    assert!(
        html.contains("Grounded in 4 checklist facts · local only"),
        "{html}"
    );
    assert!(
        html.contains("Grounded in 2 facts · sent with consent"),
        "{html}"
    );
}

#[test]
fn a_stacking_segmented_control_is_a_column_only_below_sm() {
    let html = render(|| {
        rsx! {
            SegmentedControl {
                label: "Tone",
                options: vec![("warm".into(), "Warm".into()), ("plain".into(), "Plain".into())],
                value: "warm",
                onchange: |_| {},
                flow: SegmentFlow::StackNarrow,
            }
        }
    });
    assert!(
        html.contains("flex-col sm:inline-flex sm:flex-row"),
        "{html}"
    );
    assert!(html.contains(r#"aria-pressed="true""#), "{html}");
}

#[test]
fn a_chosen_fact_is_set_apart_from_the_computed_total() {
    let html = render(|| {
        rsx! { FactList { facts: vec![Fact::new("Slots", "3"), Fact::total("Computed", "£90"), Fact::chosen("Quote", "£85")] } }
    });
    let row = |name: &str| {
        html.split("<div")
            .find(|part| part.contains(&format!(">{name}<")))
            .unwrap_or_else(|| panic!("no row for {name}: {html}"))
            .to_string()
    };
    assert!(row("Quote").contains("bg-ink"), "{html}");
    assert!(!row("Computed").contains("bg-ink"), "{html}");
    assert!(!row("Slots").contains("bg-ink"), "{html}");
    assert_eq!(html.matches("<dt").count(), 3, "{html}");
}

#[test]
fn a_ledger_names_its_rows_says_voided_in_words_and_foots_its_total() {
    let html = render(|| {
        rsx! {
            LedgerTable {
                caption: "Activity",
                caption_hidden: true,
                columns: vec![LedgerColumn::text("Entry"), LedgerColumn::figure("Amount")],
                rows: vec![
                    LedgerRow::new("a", vec!["Rent".into(), LedgerCell::new("-£900").lean(Lean::Bad)]),
                    LedgerRow::new("b", vec!["Refund".into(), "£40".into()]).voided(),
                ],
                total: LedgerRow::new("t", vec!["Net".into(), "-£900".into()]),
            }
        }
    });
    assert!(html.contains(r#"role="region""#), "{html}");
    assert!(html.contains(r#"tabindex="0""#), "{html}");
    assert!(
        html.contains(r#"<caption class="sr-only">Activity</caption>"#),
        "{html}"
    );
    assert_eq!(html.matches(r#"scope="col""#).count(), 2, "{html}");
    assert_eq!(html.matches(r#"scope="row""#).count(), 3, "{html}");
    assert!(html.contains(r#"data-state="voided""#), "{html}");
    let voided = html
        .split(r#"data-state="voided""#)
        .nth(1)
        .and_then(|rest| rest.split("</tr>").next())
        .expect("voided row");
    assert_eq!(voided.matches("Voided: ").count(), 2, "{voided}");
    assert_eq!(voided.matches("<del").count(), 1, "{voided}");
    assert!(voided.contains("text-ink-muted"), "{voided}");
    assert!(!html.contains("opacity-"), "{html}");
    let foot = html.split("<tfoot").nth(1).expect("total in tfoot");
    assert!(foot.contains("Net"), "{html}");
    assert!(
        html.contains(r#"<th scope="col" class="px-3 py-2 font-semibold text-ink-muted text-end tabular-nums whitespace-nowrap">Amount</th>"#),
        "{html}"
    );
}

#[test]
fn a_voided_cell_keeps_its_link_inside_the_strike() {
    let html = render(|| {
        rsx! {
            LedgerTable {
                caption: "Activity",
                columns: vec![LedgerColumn::text("Entry"), LedgerColumn::figure("Amount")],
                rows: vec![
                    LedgerRow::new("b", vec![LedgerCell::new("Refund").href("/r/1"), LedgerCell::new("£40").href("/r/1/amount")]).voided(),
                ],
            }
        }
    });
    assert!(html.contains(r#"href="/r/1""#), "{html}");
    let struck = html.split("<del").nth(1).expect("struck figure");
    assert!(struck.contains(r#"href="/r/1/amount""#), "{html}");
}

#[test]
fn a_citation_dates_its_source_and_warns_of_a_new_tab() {
    let html = render(|| {
        rsx! { Citation { source: "Ofsted", date_iso: "2026-09-24", date_text: "24 Sep 2026", dated: Dated::Checked, href: "https://example.org" } }
    });
    assert!(html.contains("Checked "), "{html}");
    assert!(html.contains(r#"datetime="2026-09-24""#), "{html}");
    assert!(html.contains(r#"rel="noopener noreferrer""#), "{html}");
    assert!(html.contains("(opens in new tab)"), "{html}");
}

#[test]
fn an_attention_queue_links_go_rows_and_words_state_rows() {
    let html = render(|| {
        rsx! {
            AttentionQueue {
                label: "Needs review",
                empty: "Nothing waiting.",
                numbered: true,
                items: vec![
                    QueueItem::go("a", "Sam", "3 sessions missed", "Review", "/clients/sam"),
                    QueueItem::state("b", "Billing", "Card expiring", "Blocked", Verdict::Stop),
                ],
            }
        }
    });
    assert!(
        html.contains(r#"<ol class="space-y-2" aria-label="Needs review""#),
        "{html}"
    );
    assert_eq!(html.matches("<a ").count(), 1, "{html}");
    assert!(html.contains(r#"href="/clients/sam""#), "{html}");
    assert!(html.contains("Blocked"), "{html}");
    assert!(html.contains(">01</span>"), "{html}");

    let empty = render(
        || rsx! { AttentionQueue { label: "Needs review", empty: "Nothing waiting.", items: vec![] } },
    );
    assert!(empty.contains("Nothing waiting."), "{empty}");
    assert!(!empty.contains("<ol"), "{empty}");
}

#[test]
fn spark_points_refuse_broken_series_and_flatten_single_values() {
    assert_eq!(spark_points(&[], 240.0, 48.0), "");
    assert_eq!(spark_points(&[1.0, f64::NAN], 240.0, 48.0), "");
    assert_eq!(spark_points(&[7.0], 240.0, 48.0), "0,24 240,24");
    assert_eq!(
        spark_points(&[0.0, 10.0], 240.0, 48.0),
        "0.0,48.0 240.0,0.0"
    );
    assert_eq!(spark_points(&[0.0, 0.0], 240.0, 48.0), "0,24 240,24");
    assert_eq!(
        spark_points(&[1e-17, 2e-17], 240.0, 48.0),
        "0.0,48.0 240.0,0.0",
        "a series of tiny values is a trend, not a flat line"
    );
}

#[test]
fn a_named_sparkline_that_cannot_draw_still_says_its_name() {
    let html = render(|| rsx! { Sparkline { values: vec![1.0, f64::NAN], label: "Rising" } });
    assert!(!html.contains("<svg"), "{html}");
    assert!(html.contains(">Rising</span>"), "{html}");
}

#[cfg(debug_assertions)]
#[test]
fn an_attention_queue_refuses_repeated_ids() {
    use std::sync::{Arc, Mutex};
    // Dioxus catches a panic raised while rendering, so `should_panic` never
    // sees it; the hook still does.
    let seen = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&seen);
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        sink.lock().unwrap().push(info.to_string());
    }));
    let _ = std::panic::catch_unwind(|| {
        render(|| {
            rsx! {
                AttentionQueue {
                    label: "Needs review",
                    empty: "Nothing waiting.",
                    items: vec![
                        QueueItem::go("a", "Sam", "Missed", "Review", "/a"),
                        QueueItem::go("a", "Ria", "Record", "Open", "/b"),
                    ],
                }
            }
        })
    });
    std::panic::set_hook(previous);
    let seen = seen.lock().unwrap();
    assert!(
        seen.iter()
            .any(|message| message.contains("AttentionQueue item ids must be unique")),
        "{seen:?}"
    );
}

#[test]
fn a_sparkline_is_decorative_unless_named() {
    let quiet = render(|| rsx! { Sparkline { values: vec![1.0, 3.0, 2.0] } });
    assert!(quiet.contains(r#"aria-hidden="true""#), "{quiet}");
    assert!(!quiet.contains("role="), "{quiet}");
    assert!(quiet.contains("non-scaling-stroke"), "{quiet}");

    let named = render(|| rsx! { Sparkline { values: vec![1.0, 3.0], label: "Rising" } });
    assert!(named.contains(r#"role="img""#), "{named}");
    assert!(named.contains(r#"aria-label="Rising""#), "{named}");

    let none = render(|| rsx! { Sparkline { values: vec![] } });
    assert!(!none.contains("<svg"), "{none}");
}
