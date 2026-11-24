use dioxus::prelude::*;
use rv2_kit::agents::run_timeline_rows::{
    RowKind, RowSpec, RunTimelineRows, Span, SpanMode, Transfer, describe_span, describe_transfer,
    tick_under_pill,
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

fn finished_run() -> Element {
    let rows = vec![row(
        "drafter",
        "Drafter",
        RowKind::Agent,
        "writing the summary",
    )];
    let spans = vec![span("drafter", SpanMode::Reasoning, 26.0, Some(44.0), None)];
    rsx! {
        RunTimelineRows {
            run_heading: "Refund batch",
            rows: rows,
            spans: spans,
            axis_length_s: Some(60.0),
        }
    }
}

#[test]
fn one_playhead_runs_from_the_lanes_to_the_axis_at_the_52_second_point() {
    let html = render(refund_batch);
    assert_eq!(count(&html, "class=\"kit-rtr__playhead\""), 1);
    assert!(html.contains("left: 86.6667%"));
    let rows = html.find("class=\"kit-rtr__rows\"").expect("lanes");
    let playhead = html.find("kit-rtr__playhead").expect("playhead");
    let axis = html.find("class=\"kit-rtr__axis\"").expect("axis");
    assert!(rows < playhead && playhead < axis);
}

#[test]
fn time_pill_is_centred_on_the_playhead_and_reads_the_clock() {
    let html = render(refund_batch);
    assert_eq!(count(&html, "class=\"kit-rtr__pill\""), 1);
    assert!(html.contains("kit-rtr__pill\" style=\"left: 86.6667%\""));
    assert!(html.contains(">0:52<"));
    assert!(html.contains("clock 0:52"));
}

#[test]
fn tick_under_the_pill_is_not_drawn_and_the_others_are() {
    let html = render(refund_batch);
    assert!(!html.contains(">50s<"));
    assert!(html.contains(">40s<"));
    assert!(tick_under_pill(50.0, Some(52.0), 60.0));
    assert!(!tick_under_pill(40.0, Some(52.0), 60.0));
    assert!(!tick_under_pill(50.0, None, 60.0));
}

#[test]
fn finished_run_draws_neither_playhead_nor_time_pill() {
    let html = render(finished_run);
    assert_eq!(count(&html, "kit-rtr__playhead"), 0);
    assert_eq!(count(&html, "kit-rtr__pill"), 0);
}

#[test]
fn axis_sits_below_the_last_lane_and_each_note_sits_below_its_name() {
    let html = render(refund_batch);
    let courier = html.find(">Courier<").expect("last row name");
    let axis = html.find("class=\"kit-rtr__axis\"").expect("axis");
    assert!(courier < axis);
    let who = html.find("class=\"kit-rtr__who\"").expect("row name");
    let note = html.find("class=\"kit-rtr__note\"").expect("row note");
    assert!(who < note);
}
