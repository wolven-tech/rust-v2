use dioxus::prelude::*;
use rv2_kit::agents::run_timeline_rows::{
    RowKind, RowSpec, RunTimelineRows, Span, SpanMode, Transfer, describe_span, describe_transfer,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

fn row(key: &str, name: &str, kind: RowKind, note: &str) -> RowSpec {
    RowSpec {
        row_key: key.into(),
        display_name: name.into(),
        kind,
        tint_key: None,
        mark_source: None,
        note: Some(note.into()),
    }
}

fn span(row: &str, mode: SpanMode, begin: f64, end: Option<f64>, tool: Option<&str>) -> Span {
    Span {
        span_key: None,
        row_key: row.into(),
        mode,
        begin_s: begin,
        end_s: end,
        tool_name: tool.map(Into::into),
        caption: None,
    }
}

fn refund_batch() -> Element {
    let rows = vec![
        row(
            "maintainer",
            "Maintainer",
            RowKind::Person,
            "reviewing later",
        ),
        row("scout", "Scout", RowKind::Agent, "reading the index"),
        row("drafter", "Drafter", RowKind::Agent, "writing the summary"),
        row("auditor", "Auditor", RowKind::Agent, "comparing totals"),
        row("courier", "Courier", RowKind::Agent, "holding the batch"),
    ];
    let spans = vec![
        span(
            "scout",
            SpanMode::Tool,
            8.0,
            Some(26.0),
            Some("fetch_index"),
        ),
        span("drafter", SpanMode::Reasoning, 26.0, Some(44.0), None),
        span("auditor", SpanMode::Reasoning, 44.0, None, None),
    ];
    let transfers = vec![Transfer {
        transfer_key: None,
        from_row_key: "drafter".into(),
        to_row_key: "auditor".into(),
        at_s: 44.0,
        caption: None,
    }];
    rsx! {
        RunTimelineRows {
            run_heading: "Refund batch",
            rows: rows,
            spans: spans,
            transfers: transfers,
            clock_s: Some(52.0),
            axis_length_s: Some(60.0),
        }
    }
}

#[test]
fn at_rest_three_steps_and_one_transfer_are_drawn() {
    let html = render(refund_batch);
    assert_eq!(count(&html, "data-mode=\""), 3);
    assert_eq!(count(&html, "class=\"kit-rtr__transfer\""), 1);
}

#[test]
fn every_drawn_transfer_is_a_tab_stop() {
    let html = render(refund_batch);
    let start = html
        .find("class=\"kit-rtr__transfer\"")
        .expect("transfer marker");
    let tag_end = start + html[start..].find('>').expect("end of tag");
    assert!(html[start..tag_end].contains("tabindex=\"0\""));
}

#[test]
fn heading_reads_run_name_row_and_step_counts_and_the_clock() {
    let html = render(refund_batch);
    let heading = html.find("Refund batch").expect("run heading");
    let counts = html.find("5 rows · 3 steps").expect("row and step counts");
    let clock = html.find("clock 0:52").expect("clock figure");
    assert!(heading < counts && counts < clock);
    assert!(!html.contains("role=\"dialog\""));
}

#[test]
fn quiet_lanes_show_their_note_and_no_step() {
    let html = render(refund_batch);
    assert!(html.contains("reviewing later"));
    assert!(html.contains("holding the batch"));
    assert_eq!(count(&html, "data-mode=\"tool\""), 1);
}

#[test]
fn in_flight_step_ends_at_the_clock_with_the_live_marker() {
    let html = render(refund_batch);
    assert_eq!(count(&html, "data-running=\"true\""), 1);
    assert!(html.contains("Auditor, reasoning, from 0:44 for 8.0 s"));
}

#[test]
fn tool_name_is_shown_for_the_tool_step_only() {
    let html = render(refund_batch);
    assert_eq!(count(&html, ">fetch_index<"), 1);
    assert_eq!(count(&html, ">thinking<"), 2);
}

#[test]
fn step_card_wording_names_who_what_start_and_duration() {
    let drafter = span("drafter", SpanMode::Reasoning, 26.0, Some(44.0), None);
    assert_eq!(
        describe_span("Drafter", &drafter, 44.0),
        "Drafter, reasoning, from 0:26 for 18.0 s"
    );
}

#[test]
fn transfer_card_wording_names_both_parties_and_the_time() {
    assert_eq!(
        describe_transfer("Drafter", "Auditor", 44.0),
        "Drafter to Auditor, hand-off, at 0:44"
    );
}

#[test]
fn legend_keys_each_step_kind_with_its_swatch() {
    let html = render(refund_batch);
    for kind in ["reasoning", "tool", "waiting"] {
        assert_eq!(count(&html, &format!("data-swatch=\"{kind}\"")), 1);
    }
}

#[test]
fn transfer_is_placed_by_row_index_so_it_follows_the_row_height() {
    let html = render(refund_batch);
    assert!(html.contains("top: calc(var(--kit-rtr-row-h) * 2 + var(--kit-rtr-row-h) / 2)"));
    assert!(html.contains("height: calc(var(--kit-rtr-row-h) * 1)"));
}
