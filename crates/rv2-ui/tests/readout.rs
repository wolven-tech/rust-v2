//! The instructions drawn from the bets' calculators and ledgers, rendered.
//! Each test pins the part of the contract a reader without sight of the
//! colour depends on.

use dioxus::prelude::*;
use rv2_ui::{
    Callout, Lean, Meter, Part, Receipt, ResultPanel, SegmentedControl, SplitBar, Stat, StatStrip,
    UnitField, Verdict,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn a_stat_strip_pairs_every_figure_with_its_label() {
    let html = render(|| {
        rsx! {
            StatStrip {
                stats: vec![
                    Stat::new("Monthly", "£412").note("before fees"),
                    Stat::new("Balance", "−£30").lean(Lean::Bad),
                ],
            }
        }
    });
    assert_eq!(html.matches("<dt").count(), 2, "{html}");
    assert!(html.contains("text-fault-ink"), "{html}");
    assert!(html.contains("before fees"), "{html}");
}

#[test]
fn a_result_panel_announces_a_changed_answer_politely() {
    let html = render(|| {
        rsx! {
            ResultPanel { label: "True cost", value: "£18.40", context: "per session", "Working" }
        }
    });
    assert_eq!(html.matches("aria-live").count(), 1, "{html}");
    let live = html
        .split(r#"aria-live="polite""#)
        .nth(1)
        .and_then(|rest| rest.split("</p>").next())
        .expect("live figure");
    assert!(live.contains("£18.40"), "{html}");
    assert!(!live.contains("Working"), "{html}");
}

#[test]
fn a_meter_exposes_its_value_and_turns_at_its_thresholds() {
    let calm = render(|| {
        rsx! {
            Meter { label: "Runs", value: 100.0, max: 500.0, readout: "100 of 500", warn_at: 0.8, critical_at: 0.95 }
        }
    });
    let close = render(|| {
        rsx! {
            Meter { label: "Runs", value: 420.0, max: 500.0, readout: "420 of 500", warn_at: 0.8, critical_at: 0.95 }
        }
    });
    let over = render(|| {
        rsx! {
            Meter { label: "Runs", value: 499.0, max: 500.0, readout: "499 of 500", warn_at: 0.8, critical_at: 0.95 }
        }
    });
    assert!(calm.contains(r#"role="meter""#), "{calm}");
    assert!(calm.contains(r#"aria-valuetext="100 of 500""#), "{calm}");
    assert!(calm.contains("bg-takeable"), "{calm}");
    assert!(close.contains("bg-caution"), "{close}");
    assert!(over.contains("bg-fault"), "{over}");
}

#[test]
fn a_meter_with_no_range_renders_empty_rather_than_full() {
    let html = render(|| rsx! { Meter { label: "Empty", value: 3.0, max: 0.0, readout: "n/a" } });
    assert!(html.contains("width:0.0%"), "{html}");
}

#[test]
fn a_meter_with_a_value_it_cannot_measure_renders_empty_and_neutral() {
    let html = render(|| {
        rsx! { Meter { label: "Runs", value: f64::NAN, max: 500.0, readout: "unknown", warn_at: 0.8, critical_at: 0.95 } }
    });
    assert!(html.contains("width:0.0%"), "{html}");
    assert!(!html.contains("NaN"), "{html}");
    assert!(!html.contains("bg-takeable"), "{html}");
    assert!(html.contains(r#"aria-valuetext="unknown""#), "{html}");
}

#[test]
fn a_split_bar_draws_nothing_for_an_infinite_part() {
    let html = render(|| {
        rsx! {
            SplitBar {
                label: "Cost",
                parts: vec![
                    Part::new("Energy", 3.0, "£3.00", "red"),
                    Part::new("Broken", f64::INFINITY, "?", "blue"),
                    Part::new("Fees", 1.0, "£1.00", "green"),
                ],
            }
        }
    });
    assert!(!html.contains("NaN"), "{html}");
    assert!(html.contains("width:75.0%"), "{html}");
    assert!(html.contains("width:0.0%"), "{html}");
}

#[test]
fn a_split_bar_names_every_part_and_figure_for_a_screen_reader() {
    let html = render(|| {
        rsx! {
            SplitBar {
                label: "Session cost",
                parts: vec![
                    Part::new("Energy", 3.0, "£3.00", "var(--color-ink)"),
                    Part::new("Fees", 1.0, "£1.00", "var(--color-fault)"),
                ],
            }
        }
    });
    assert!(
        html.contains(r#"aria-label="Session cost: Energy £3.00, Fees £1.00""#),
        "{html}"
    );
    assert!(html.contains("width:75.0%"), "{html}");
}

#[test]
fn only_a_stop_callout_interrupts_as_an_alert() {
    let stop =
        render(|| rsx! { Callout { verdict: Verdict::Stop, title: "Does not apply", "Why" } });
    let note = render(|| rsx! { Callout { title: "Context", "Why" } });
    assert!(stop.contains(r#"role="alert""#), "{stop}");
    assert!(!note.contains("role="), "{note}");
}

#[test]
fn a_receipt_renders_its_facts_as_a_definition_list() {
    let html = render(|| {
        rsx! {
            Receipt {
                title: "Scope sent",
                status: "SENT",
                verdict: Verdict::Clear,
                facts: vec![("To".to_string(), "Client".to_string()), ("Hours".to_string(), "12".to_string())],
                "Logged."
            }
        }
    });
    assert_eq!(html.matches("<dt").count(), 2, "{html}");
    assert!(html.contains("SENT"), "{html}");
}

#[test]
fn a_unit_field_hides_its_affixes_and_asks_for_a_decimal_keyboard() {
    let html = render(|| {
        rsx! {
            UnitField { label: "Price, pounds", value: "12", prefix: "£", oninput: |_| {} }
        }
    });
    assert!(html.contains(r#"inputmode="decimal""#), "{html}");
    assert!(html.contains(r#"aria-hidden="true""#), "{html}");
    assert!(html.contains("focus-within:ring-2"), "{html}");
}

#[test]
fn a_segmented_control_presses_exactly_the_current_option() {
    let html = render(|| {
        rsx! {
            SegmentedControl {
                label: "Billing",
                options: vec![("m".to_string(), "Monthly".to_string()), ("y".to_string(), "Yearly".to_string())],
                value: "y",
                onchange: |_| {},
            }
        }
    });
    assert!(html.contains(r#"role="group""#), "{html}");
    assert_eq!(html.matches(r#"aria-pressed="true""#).count(), 1, "{html}");
}
