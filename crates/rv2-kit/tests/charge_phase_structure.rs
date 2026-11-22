use dioxus::prelude::*;
use rv2_kit::components::charge_phase_builder::{
    ChargePhaseBuilder, Line, LineKind, ProductLine, Recurrence, Timing,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

fn kickoff() -> Line {
    Line {
        line_id: "k1".into(),
        kind: LineKind::Product(ProductLine {
            name: "Kickoff workshop".into(),
            recurrence: Recurrence::Once,
            timing: Timing::UpFront,
            amount: "300".into(),
            quantity: "1".into(),
        }),
    }
}

fn rest_lines() -> Vec<Line> {
    vec![kickoff()]
}

fn edit_lines() -> Vec<Line> {
    vec![
        kickoff(),
        Line {
            line_id: "p1".into(),
            kind: LineKind::Product(ProductLine {
                name: "Platform seat".into(),
                recurrence: Recurrence::Monthly,
                timing: Timing::PeriodEnd,
                amount: "120".into(),
                quantity: "1".into(),
            }),
        },
        Line {
            line_id: "d1".into(),
            kind: LineKind::Discount {
                percent: true,
                amount: "10".into(),
            },
        },
    ]
}

fn at_rest() -> Element {
    rsx! {
        ChargePhaseBuilder {
            heading_text: "Contract phase",
            phase_label: "Rollout phase",
            item_options: vec![],
            starting_lines: rest_lines(),
            starting_date: "2026-03-01",
        }
    }
}

fn editing() -> Element {
    rsx! {
        ChargePhaseBuilder {
            heading_text: "Contract phase",
            phase_label: "Rollout phase",
            item_options: vec![],
            starting_lines: edit_lines(),
            starting_date: "2026-03-01",
            start_editing: true,
        }
    }
}

#[test]
fn at_rest_the_only_control_is_edit_phase_and_every_value_is_text() {
    let html = render(at_rest);
    assert!(html.contains("Edit phase"));
    assert_eq!(count(&html, "<button"), 1);
    assert_eq!(count(&html, "<input"), 0);
    assert_eq!(count(&html, "<select"), 0);
    assert_eq!(count(&html, "kit-cpb__adds"), 0);
    assert!(!html.contains("Include item"));
}

#[test]
fn at_rest_the_lines_table_has_four_heads_and_one_line_row() {
    let html = render(at_rest);
    let name = html.find("scope=\"col\">Name").expect("name head");
    let recurrence = html
        .find("scope=\"col\">Recurrence")
        .expect("recurrence head");
    let timing = html.find("scope=\"col\">Timing").expect("timing head");
    let amount = html.find("scope=\"col\">Amount").expect("amount head");
    assert!(name < recurrence && recurrence < timing && timing < amount);
    assert_eq!(count(&html, "scope=\"col\""), 4);
    assert_eq!(count(&html, "<tr"), 2);
    assert!(html.contains("Kickoff workshop"));
}

#[test]
fn at_rest_the_schedule_line_shows_start_and_length_as_text_with_the_end_beside_the_length() {
    let html = render(at_rest);
    let start = html.find("1 March 2026").expect("start date as text");
    let length = html.find("12 months").expect("length as text");
    let end = html
        .find("Ends 28 February 2027")
        .expect("end label beside the length");
    assert!(start < length && length < end);
}

#[test]
fn at_rest_the_summary_figures_are_the_kickoff_charge() {
    let html = render(at_rest);
    assert!(html.contains("Phase sum</dt><dd>$300.00</dd>"));
    assert!(html.contains("Initial charge</dt><dd>$300.00</dd>"));
}

#[test]
fn at_rest_no_menu_calendar_or_invalid_message_is_open() {
    let html = render(at_rest);
    assert_eq!(count(&html, "kit-cpb__error"), 0);
    assert_eq!(count(&html, "aria-expanded=\"true\""), 0);
    assert!(!html.contains("Enter a number"));
}

#[test]
fn edit_phase_reveals_the_builder_and_stop_editing_is_offered_there() {
    let html = render(editing);
    assert!(html.contains("Stop editing"));
    assert!(!html.contains("Edit phase"));
    assert_eq!(count(&html, "kit-cpb__adds"), 1);
}

#[test]
fn editing_name_sits_in_the_heading_and_start_and_length_share_the_schedule_row() {
    let html = render(editing);
    let heading = html.find("kit-cpb__heading").expect("heading rendered");
    let name = html.find("Phase name").expect("name field rendered");
    let phase = html.find("kit-cpb__phase").expect("schedule row rendered");
    let start = html.find(">Start<").expect("start rendered");
    let length = html.find("Length").expect("length rendered");
    let end = html.find("kit-cpb__end").expect("end label rendered");
    assert!(heading < name && name < phase);
    assert!(phase < start && start < length && length < end);
}

#[test]
fn line_table_headers_read_name_recurrence_timing_amount_then_an_unlabelled_column() {
    let html = render(editing);
    let name = html.find("scope=\"col\">Name").expect("name head");
    let recurrence = html
        .find("scope=\"col\">Recurrence")
        .expect("recurrence head");
    let timing = html.find("scope=\"col\">Timing").expect("timing head");
    let amount = html.find("scope=\"col\">Amount").expect("amount head");
    assert!(name < recurrence && recurrence < timing && timing < amount);
    assert_eq!(count(&html, "<th scope=\"col\"></th>"), 1);
}

#[test]
fn every_line_has_a_remove_action_labelled_with_its_line_name() {
    let html = render(editing);
    assert!(html.contains("aria-label=\"Remove Kickoff workshop\""));
    assert!(html.contains("aria-label=\"Remove Platform seat\""));
    assert!(html.contains("aria-label=\"Remove Discount\""));
    assert_eq!(count(&html, "class=\"kit-cpb__remove\""), 3);
}

#[test]
fn discount_row_has_dashes_and_a_unit_select_beneath_its_amount() {
    let html = render(editing);
    let row = html.find("Remove Discount").expect("discount row");
    let before = &html[..row];
    let start = before.rfind("<tr").expect("row start");
    let discount = &html[start..row];
    assert_eq!(count(discount, "kit-cpb__dash"), 2);
    assert!(discount.contains("kit-cpb__amount"));
    assert!(discount.contains('%'));
}

#[test]
fn edit_table_holds_exactly_three_line_rows() {
    let html = render(editing);
    assert_eq!(count(&html, "aria-label=\"Product name\""), 2);
    assert_eq!(count(&html, "aria-label=\"Discount amount\""), 1);
}

#[test]
fn add_row_holds_the_picker_and_both_buttons_in_order() {
    let html = render(editing);
    let adds = html.find("kit-cpb__adds").expect("add row rendered");
    let picker = html[adds..]
        .find("Include item")
        .expect("picker placeholder");
    let adjustment = html[adds..]
        .find("Include adjustment")
        .expect("adjustment button");
    let floor = html[adds..].find("Include floor").expect("floor button");
    assert!(picker < adjustment && adjustment < floor);
}

#[test]
fn summary_shows_initial_charge_end_and_phase_sum_in_order() {
    let html = render(editing);
    let initial = html.find("Initial charge").expect("initial charge");
    let end = html.find("Phase end").expect("phase end");
    let sum = html.find("Phase sum").expect("phase sum");
    assert!(initial < end && end < sum);
    assert!(html.contains("Phase sum</dt><dd>$378.00</dd>"));
}

#[test]
fn phase_end_and_start_use_the_same_day_month_year_form() {
    let html = render(at_rest);
    assert!(html.contains("1 March 2026"));
    assert!(html.contains("Ends 28 February 2027"));
    assert!(!html.contains("2026-03-01"));
    assert!(!html.contains("2027-02-28"));
}
