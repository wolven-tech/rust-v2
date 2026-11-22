use dioxus::prelude::*;
use rv2_affect::date_picker::Day;
use rv2_kit::components::period_picker::{PeriodPicker, resolve_builtin};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn anchor() -> Day {
    Day::new(2026, 11, 21).expect("valid date")
}

fn open_panel(first_weekday: u8) -> String {
    let app: fn() -> Element = if first_weekday == 1 {
        || {
            rsx! {
                PeriodPicker {
                    selected_range: Some(resolve_builtin("last_30", anchor())),
                    anchor_today: Some(anchor()),
                    start_expanded: true,
                }
            }
        }
    } else {
        || {
            rsx! {
                PeriodPicker {
                    selected_range: Some(resolve_builtin("last_30", anchor())),
                    anchor_today: Some(anchor()),
                    first_weekday: 0,
                    start_expanded: true,
                }
            }
        }
    };
    render(app)
}

#[test]
fn an_open_period_picker_renders_one_trigger_and_one_range_summary() {
    let html = open_panel(1);
    assert_eq!(
        html.matches("class=\"kit-per__trigger\"").count(),
        1,
        "{html}"
    );
    assert_eq!(
        html.matches("class=\"kit-per__summary\"").count(),
        1,
        "{html}"
    );
    assert_eq!(
        html.matches("class=\"kit-per__value\"").count(),
        1,
        "{html}"
    );
}

#[test]
fn preset_column_lists_ten_entries_with_custom_last() {
    let html = open_panel(1);
    assert_eq!(html.matches("class=\"kit-per__preset\"").count(), 10);
    let custom = html.find("Custom").expect("custom");
    let year = html.find("Year to date").expect("year to date");
    assert!(year < custom);
}

#[test]
fn preset_column_marks_the_preset_matching_the_selection() {
    let html = open_panel(1);
    let marked = html
        .split("class=\"kit-per__preset\"")
        .skip(1)
        .filter(|chunk| chunk.contains("aria-pressed=\"true\""))
        .count();
    assert_eq!(marked, 1);
}

#[test]
fn two_month_grids_sit_side_by_side_with_one_nav_control_each() {
    let html = open_panel(1);
    assert_eq!(html.matches("role=\"grid\"").count(), 2);
    assert_eq!(html.matches("aria-label=\"Previous month\"").count(), 1);
    assert_eq!(html.matches("aria-label=\"Next month\"").count(), 1);
}

#[test]
fn weekday_headers_start_on_monday_when_first_weekday_is_one() {
    let html = open_panel(1);
    assert_eq!(html.matches("class=\"kit-per__weekdays\"").count(), 2);
    assert!(html.find("Mo").expect("Mo") < html.find("Su").expect("Su"));
}

#[test]
fn weekday_headers_start_on_sunday_when_first_weekday_is_zero() {
    let html = open_panel(0);
    assert!(html.find("Su").expect("Su") < html.find("Mo").expect("Mo"));
}

#[test]
fn the_anchor_day_carries_the_today_marker_once() {
    let html = open_panel(1);
    assert_eq!(html.matches("data-today=\"true\"").count(), 1);
}

#[test]
fn the_range_band_covers_thirty_days_across_both_grids() {
    let html = open_panel(1);
    assert!(html.matches("data-inside=\"true\"").count() >= 30);
    assert!(html.matches("data-comparison=\"true\"").count() > 0);
}

#[test]
fn comparison_radio_group_offers_three_options_with_previous_period_selected() {
    let html = open_panel(1);
    assert_eq!(html.matches("role=\"radio\"").count(), 3);
    let checked = html
        .split("role=\"radio\"")
        .skip(1)
        .filter(|chunk| chunk.contains("aria-checked=\"true\""))
        .count();
    assert_eq!(checked, 1);
    assert!(html.find("Previous period") > html.find("kit-per__comparison"));
}

#[test]
fn footer_reads_comparison_then_summary_then_cancel_then_apply() {
    let html = open_panel(1);
    let comparison = html.find("kit-per__comparison").expect("comparison");
    let summary = html.find("kit-per__summary").expect("summary");
    let cancel = html.find("Cancel").expect("cancel");
    let apply = html.find("Apply").expect("apply");
    assert!(comparison < summary && summary < cancel && cancel < apply);
}
