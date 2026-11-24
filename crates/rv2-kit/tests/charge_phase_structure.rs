use dioxus::prelude::*;
use rv2_kit::components::charge_phase_builder::{
    CatalogItem, ChargePhaseBuilder, Line, LineKind, ProductLine, Recurrence, Timing, parse_iso,
    recurrence_suffix, schedule_text,
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

fn three_lines() -> Vec<Line> {
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

fn catalog() -> Vec<CatalogItem> {
    vec![CatalogItem {
        option_name: "Platform seat".into(),
        recurrence: Recurrence::Monthly,
        unit_amount: 120.0,
    }]
}

fn gallery() -> Element {
    rsx! {
        ChargePhaseBuilder {
            heading_text: "Contract phase",
            phase_label: "Rollout phase",
            item_options: catalog(),
            starting_lines: vec![kickoff()],
            starting_date: "2026-03-01",
        }
    }
}

fn three_line_card() -> Element {
    rsx! {
        ChargePhaseBuilder {
            heading_text: "Contract phase",
            phase_label: "Rollout phase",
            item_options: catalog(),
            starting_lines: three_lines(),
            starting_date: "2026-03-01",
        }
    }
}

#[test]
fn card_stacks_heading_strip_schedule_table_adds_and_summary_in_order() {
    let html = render(gallery);
    let heading = html.find("kit-cpb__heading").expect("heading");
    let strip = html.find("class=\"kit-cpb__strip\"").expect("strip");
    let phase = html.find("class=\"kit-cpb__phase\"").expect("schedule row");
    let lines = html.find("class=\"kit-cpb__lines\"").expect("line table");
    let adds = html.find("class=\"kit-cpb__adds\"").expect("add row");
    let summary = html.find("class=\"kit-cpb__summary\"").expect("summary");
    assert!(heading < strip && strip < phase);
    assert!(phase < lines && lines < adds && adds < summary);
}

#[test]
fn strip_shows_an_editable_phase_name_and_the_start_and_length_as_text() {
    let html = render(gallery);
    let strip = html.find("class=\"kit-cpb__strip\"").expect("strip");
    let phase = html.find("class=\"kit-cpb__phase\"").expect("schedule row");
    let band = &html[strip..phase];
    assert!(band.contains("Phase name"));
    assert!(band.contains("value=\"Rollout phase\""));
    assert!(band.contains("1 March 2026"));
    assert!(band.contains("Twelve months"));
}

#[test]
fn start_and_length_sit_side_by_side_and_the_start_field_has_a_calendar_glyph() {
    let html = render(gallery);
    let phase = html.find("class=\"kit-cpb__phase\"").expect("schedule row");
    let lines = html.find("class=\"kit-cpb__lines\"").expect("line table");
    let row = &html[phase..lines];
    let glyph = row.find("<svg").expect("calendar glyph");
    let start = row.find("Start date").expect("start field");
    let length = row.find(">Length<").expect("length field");
    assert!(glyph < start && start < length);
}

#[test]
fn line_table_header_reads_name_recurrence_amount_with_no_timing_head() {
    let html = render(gallery);
    let name = html.find("scope=\"col\">Name").expect("name head");
    let recurrence = html
        .find("scope=\"col\">Recurrence")
        .expect("recurrence head");
    let amount = html.find("scope=\"col\">Amount").expect("amount head");
    assert!(name < recurrence && recurrence < amount);
    assert_eq!(count(&html, "scope=\"col\""), 4);
    assert_eq!(count(&html, "<th scope=\"col\"></th>"), 1);
    assert!(!html.contains(">Timing<"));
}

#[test]
fn timing_select_sits_beneath_the_amount_inside_the_amount_cell() {
    let html = render(gallery);
    let cell = html.find("data-label=\"Amount\"").expect("amount cell");
    let end = cell + html[cell..].find("</td>").expect("cell end");
    let body = &html[cell..end];
    let amount = body.find("kit-cpb__amount").expect("amount field");
    let timing = body
        .find("Timing for Kickoff workshop")
        .expect("timing select");
    assert!(amount < timing);
}

#[test]
fn gallery_table_holds_one_line_row_whose_remove_action_is_named_for_it() {
    let html = render(gallery);
    assert_eq!(count(&html, "<tr"), 2);
    assert!(html.contains("aria-label=\"Remove Kickoff workshop\""));
    assert!(html.contains("Kickoff workshop"));
}

#[test]
fn remove_action_is_hidden_until_its_row_is_hovered_or_focused() {
    let css = include_str!("../assets/charge-phase-builder.css");
    assert!(css.contains(".kit-cpb__remove { min-height: 30px"));
    assert!(css.contains("opacity: 0;"));
    assert!(css.contains("tr:hover .kit-cpb__remove"));
    assert!(css.contains("tr:focus-within .kit-cpb__remove"));
}

#[test]
fn add_row_holds_the_picker_at_its_start_and_both_buttons_at_its_end() {
    let html = render(gallery);
    let adds = html.find("class=\"kit-cpb__adds\"").expect("add row");
    let picker = html[adds..].find("Include item").expect("picker trigger");
    let adjustment = html[adds..]
        .find("Include adjustment")
        .expect("adjustment button");
    let floor = html[adds..].find("Include floor").expect("floor button");
    assert!(picker < adjustment && adjustment < floor);
    assert_eq!(count(&html, "aria-expanded=\"true\""), 0);
}

#[test]
fn summary_band_shows_first_invoice_then_phase_total_and_no_end_date() {
    let html = render(gallery);
    let invoice = html
        .find("First invoice</dt><dd>$300.00</dd>")
        .expect("first invoice");
    let total = html
        .find("Phase total</dt><dd>$300.00</dd>")
        .expect("phase total");
    assert!(invoice < total);
    assert!(!html.contains("Phase end"));
    assert!(!html.contains("Ends "));
}

#[test]
fn changing_the_length_changes_only_the_length_text() {
    let start = parse_iso("2026-03-01");
    let (start_at_twelve, length_at_twelve) = schedule_text(start, 12);
    let (start_at_six, length_at_six) = schedule_text(start, 6);
    assert_eq!(start_at_twelve, "1 March 2026");
    assert_eq!(start_at_six, start_at_twelve);
    assert_eq!(length_at_twelve, "Twelve months");
    assert_eq!(length_at_six, "Six months");
}

#[test]
fn each_recurrence_names_the_period_its_amount_is_charged_for() {
    assert_eq!(recurrence_suffix(Recurrence::Once), "once");
    assert_eq!(recurrence_suffix(Recurrence::Monthly), "/ month");
    assert_eq!(recurrence_suffix(Recurrence::Quarterly), "/ quarter");
    assert_eq!(recurrence_suffix(Recurrence::Yearly), "/ year");
}

#[test]
fn every_line_has_a_remove_action_labelled_with_its_line_name() {
    let html = render(three_line_card);
    assert!(html.contains("aria-label=\"Remove Kickoff workshop\""));
    assert!(html.contains("aria-label=\"Remove Platform seat\""));
    assert!(html.contains("aria-label=\"Remove Discount\""));
    assert_eq!(count(&html, "class=\"kit-cpb__remove\""), 3);
}

#[test]
fn discount_row_has_a_dash_for_recurrence_and_a_unit_select_beneath_its_amount() {
    let html = render(three_line_card);
    let row = html.find("Remove Discount").expect("discount row");
    let start = html[..row].rfind("<tr").expect("row start");
    let discount = &html[start..row];
    assert_eq!(count(discount, "kit-cpb__dash"), 1);
    assert!(discount.contains("kit-cpb__amount"));
    assert!(discount.contains("Discount unit"));
}

#[test]
fn three_line_table_holds_two_products_and_one_discount() {
    let html = render(three_line_card);
    assert_eq!(count(&html, "<tr"), 4);
    assert!(html.contains(">Platform seat<"));
    assert!(html.contains(">Discount<"));
}

#[test]
fn summary_sums_the_three_lines_with_the_discount_applied() {
    let html = render(three_line_card);
    assert!(html.contains("First invoice</dt><dd>$300.00</dd>"));
    assert!(html.contains("Phase total</dt><dd>$378.00</dd>"));
}

#[test]
fn at_rest_no_menu_calendar_or_invalid_message_is_open() {
    let html = render(gallery);
    assert_eq!(count(&html, "kit-cpb__error"), 0);
    assert_eq!(count(&html, "aria-expanded=\"true\""), 0);
    assert!(!html.contains("Enter a number"));
}
