use dioxus::prelude::*;
use rv2_affect::number_input::{Reading, format_value, next_on_grid, read_value, round_to_step};
use rv2_affect::otp_input::{clean_code, clock, key_move};
use rv2_affect::rating::{half_step, star_count, star_fill, value_label};
use rv2_affect::tag_input::{avatar_letter, plan_commit, split_tags};
use rv2_affect::text_area::{format_count, text_length};
use rv2_affect::time_picker::{bounds, format_time, from_value, parse_time};
use rv2_affect::toggle_group::{ToggleGroupItem, dedupe_items, next_selection};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn item(value: &str) -> ToggleGroupItem {
    ToggleGroupItem {
        value: value.to_string(),
        ..Default::default()
    }
}

#[test]
fn star_rating_read_only_renders_one_image_with_a_half_star_label() {
    let html = render(|| rsx! { rv2_affect::rating::Rating { read_only: true, value: 4.5 } });
    assert!(html.contains("role=\"img\""));
    assert!(html.contains("aria-label=\"4.5 out of 5\""));
    assert!(html.contains("data-readonly=\"true\""));
    assert_eq!(html.matches("class=\"ui-rtg__star\"").count(), 5);
}

#[test]
fn star_rating_editable_is_a_radio_group_with_one_checked_star() {
    let html = render(|| rsx! { rv2_affect::rating::Rating { value: 3.0 } });
    assert!(html.contains("role=\"radiogroup\""));
    assert!(html.contains("aria-label=\"Rating\""));
    assert_eq!(html.matches("role=\"radio\"").count(), 5);
    assert_eq!(html.matches("aria-checked=\"true\"").count(), 1);
}

#[test]
fn thumbs_rating_read_only_names_the_rated_answer() {
    let html = render(|| {
        rsx! { rv2_affect::rating::ThumbsRating { read_only: true, value: rv2_affect::rating::ThumbRating::Down } }
    });
    assert!(html.contains("aria-label=\"Rated: Bad response\""));
    assert!(html.contains("data-down=\"true\""));
}

#[test]
fn rating_value_label_drops_a_trailing_point_zero() {
    assert_eq!(value_label(3.0, 5), "3 out of 5");
    assert_eq!(value_label(4.5, 5), "4.5 out of 5");
    assert_eq!(value_label(4.25, 5), "4.3 out of 5");
}

#[test]
fn rating_star_fill_is_whole_half_or_empty_by_position() {
    let shown = half_step(4.4);
    assert_eq!(shown, 4.5);
    assert_eq!(star_fill(shown, 4), 1.0);
    assert_eq!(star_fill(shown, 5), 0.5);
    assert_eq!(star_fill(shown, 6), 0.0);
    assert_eq!(star_fill(shown, 1), 1.0);
    assert_eq!(star_count(0), 5);
    assert_eq!(star_count(80), 50);
}

#[test]
fn toggle_group_marks_the_default_item_as_pressed() {
    let html = render(|| {
        rsx! { rv2_affect::toggle_group::ToggleGroup { items: vec![item("a"), item("b")], default_value: vec!["b".to_string()] } }
    });
    assert!(html.contains("role=\"group\""));
    assert!(html.contains("data-variant=\"tray\""));
    assert_eq!(html.matches("aria-pressed=\"true\"").count(), 1);
    assert_eq!(html.matches("class=\"ui-tgl__item\"").count(), 2);
}

#[test]
fn toggle_group_single_selection_turns_a_pressed_item_off() {
    let items = vec![item("a"), item("b")];
    assert_eq!(
        next_selection(&items, &[], "a", true),
        Some(vec!["a".to_string()])
    );
    assert_eq!(
        next_selection(&items, &["a".to_string()], "a", true),
        Some(vec![])
    );
    assert_eq!(
        next_selection(&items, &["a".to_string()], "b", true),
        Some(vec!["b".to_string()])
    );
}

#[test]
fn toggle_group_multiple_selection_keeps_the_items_order() {
    let items = vec![item("a"), item("b"), item("c")];
    let on = vec!["c".to_string(), "a".to_string()];
    assert_eq!(
        next_selection(&items, &on, "b", false),
        Some(vec!["a".to_string(), "b".to_string(), "c".to_string()])
    );
}

#[test]
fn toggle_group_ignores_a_disabled_or_unknown_item() {
    let mut disabled = item("a");
    disabled.disabled = true;
    let items = vec![disabled, item("b")];
    assert_eq!(next_selection(&items, &[], "a", false), None);
    assert_eq!(next_selection(&items, &[], "zzz", false), None);
}

#[test]
fn toggle_group_drops_repeated_values_keeping_the_first() {
    let mut second = item("a");
    second.disabled = true;
    let deduped = dedupe_items(vec![item("a"), second, item("b")]);
    assert_eq!(deduped.len(), 2);
    assert!(!deduped[0].disabled);
}

#[test]
fn text_area_counter_shows_the_length_against_max_length() {
    let html = render(|| {
        rsx! { rv2_affect::text_area::TextArea { counter: true, max_length: 2000usize, default_value: "hello".to_string() } }
    });
    assert!(html.contains("class=\"ui-ta__count\""));
    assert!(html.contains("5/2,000"));
    assert!(html.contains("class=\"ui-ta__input\""));
}

#[test]
fn text_area_hint_is_replaced_by_the_error_when_one_is_set() {
    let html = render(|| {
        rsx! {
            rv2_affect::text_area::TextArea {
                hint: rsx! { "Keep it short" },
                error: rsx! { "Too long" },
            }
        }
    });
    assert!(html.contains("Too long"));
    assert!(!html.contains("Keep it short"));
    assert!(html.contains("data-invalid=\"true\""));
    assert!(html.contains("aria-live=\"polite\""));
}

#[test]
fn text_area_count_uses_utf16_units_and_grouped_digits() {
    assert_eq!(text_length("a😀"), 3);
    assert_eq!(format_count(1_234_567), "1,234,567");
    assert_eq!(format_count(999), "999");
}

#[test]
fn text_field_clear_button_appears_for_a_clearable_field_with_a_value() {
    let html = render(|| {
        rsx! { rv2_affect::text_field::TextField { clearable: true, default_value: "abc".to_string() } }
    });
    assert!(html.contains("aria-label=\"Clear\""));
    assert!(html.contains("class=\"ui-tf__action\""));
}

#[test]
fn text_field_without_a_value_hides_the_clear_button() {
    let html = render(|| rsx! { rv2_affect::text_field::TextField { clearable: true } });
    assert!(!html.contains("aria-label=\"Clear\""));
}

#[test]
fn password_field_offers_a_reveal_button_and_starts_hidden() {
    let html = render(|| {
        rsx! { rv2_affect::text_field::TextField { r#type: "password".to_string(), default_value: "secret".to_string() } }
    });
    assert!(html.contains("aria-label=\"Show password\""));
    assert!(html.contains("data-revealed=\"false\""));
    assert!(html.contains("type=\"password\""));
}

#[test]
fn text_field_error_marks_the_field_invalid_and_announces_it() {
    let html = render(|| {
        rsx! { rv2_affect::text_field::TextField { error: rsx! { "Required" } } }
    });
    assert!(html.contains("aria-invalid=\"true\""));
    assert!(html.contains("Required"));
}

#[test]
fn tag_input_renders_one_chip_per_value_and_a_hidden_input_each() {
    let html = render(|| {
        rsx! {
            rv2_affect::tag_input::TagInput {
                value: vec!["rust".to_string(), "dioxus".to_string()],
                name: "tags",
            }
        }
    });
    assert_eq!(html.matches("class=\"ui-tag__chip\"").count(), 2);
    assert_eq!(html.matches("type=\"hidden\"").count(), 2);
    assert!(html.contains("placeholder=\"Add more…\""));
}

#[test]
fn tag_input_splits_on_commas_and_semicolons_but_not_spaces_by_default() {
    assert_eq!(split_tags("a, b;c\nd", false), vec!["a", " b", "c", "d"]);
    assert_eq!(split_tags("a b", false), vec!["a b"]);
    assert_eq!(split_tags("a b", true), vec!["a", "b"]);
}

#[test]
fn tag_input_plan_refuses_duplicates_and_values_over_max() {
    let parts = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let existing = vec!["a".to_string()];
    let plan = plan_commit(&parts, &existing, Some(2), |_| None);
    assert_eq!(plan.added, vec!["b".to_string()]);
    assert_eq!(plan.left, vec!["c".to_string()]);
    assert_eq!(plan.problem, "Up to 2 at a time.");

    let dup = plan_commit(&["a".to_string()], &existing, None, |_| None);
    assert!(dup.added.is_empty());
    assert_eq!(dup.duplicate.as_deref(), Some("a"));
}

#[test]
fn tag_input_plan_keeps_a_refused_value_in_the_field_with_its_message() {
    let parts = vec!["bad".to_string(), "ok".to_string()];
    let plan = plan_commit(&parts, &[], None, |p| {
        (p == "bad").then(|| "Not an email".to_string())
    });
    assert_eq!(plan.added, vec!["ok".to_string()]);
    assert_eq!(plan.left, vec!["bad".to_string()]);
    assert_eq!(plan.problem, "Not an email");
}

#[test]
fn tag_input_avatar_letter_is_the_first_character_upper_cased() {
    assert_eq!(avatar_letter("ada"), "A");
    assert_eq!(avatar_letter(""), "");
}

#[test]
fn number_input_reads_grouped_negative_and_empty_text() {
    assert_eq!(read_value("1,234.5"), Reading::Value(1234.5));
    assert_eq!(read_value("−4"), Reading::Value(-4.0));
    assert_eq!(read_value("(7)"), Reading::Value(-7.0));
    assert_eq!(read_value("   "), Reading::Empty);
    assert_eq!(read_value("1.2.3"), Reading::Invalid);
    assert_eq!(read_value("."), Reading::Invalid);
}

#[test]
fn number_input_formats_with_grouping_and_trimmed_decimals() {
    assert_eq!(format_value(4000.0, 3), "4,000");
    assert_eq!(format_value(1234.5, 3), "1,234.5");
    assert_eq!(format_value(-12.3456, 2), "-12.35");
    assert_eq!(format_value(0.0, 3), "0");
}

#[test]
fn number_input_rounds_float_noise_to_the_step() {
    assert_eq!(round_to_step(0.1 + 0.2, 0.1), 0.3);
    assert_eq!(round_to_step(2.0 + 1e-13, 1.0), 2.0);
}

#[test]
fn number_input_steps_land_on_the_grid_from_the_origin() {
    // Grid of 5 starting at 0: from 12 a step up lands on 15, a step down on 10.
    assert_eq!(next_on_grid(12.0, 0.0, 5.0, 5.0, 1.0), 15.0);
    assert_eq!(next_on_grid(12.0, 0.0, 5.0, 5.0, -1.0), 10.0);
    // Off the grid by a fraction: the first step lands on it, not past it.
    assert_eq!(next_on_grid(10.4, 0.0, 1.0, 1.0, 1.0), 11.0);
}

fn day(year: i32, month: u8, day: u8) -> rv2_affect::date_picker::Day {
    rv2_affect::date_picker::Day::new(year, month, day).expect("valid date")
}

#[test]
fn date_new_rejects_impossible_calendar_dates() {
    use rv2_affect::date_picker::Day;
    assert!(Day::new(2024, 2, 29).is_some());
    assert!(Day::new(2023, 2, 29).is_none());
    assert!(Day::new(2024, 13, 1).is_none());
}

#[test]
fn date_add_months_clamps_to_the_end_of_a_shorter_month() {
    assert_eq!(day(2024, 1, 31).add_months(1), day(2024, 2, 29));
    assert_eq!(day(2024, 3, 31).add_months(-2), day(2024, 1, 31));
    assert_eq!(day(2024, 2, 29).add_months(12), day(2025, 2, 28));
}

#[test]
fn date_add_days_crosses_leap_day_and_year_end() {
    assert_eq!(day(2024, 3, 1).add_days(-1), day(2024, 2, 29));
    assert_eq!(day(2024, 1, 1).add_days(366), day(2025, 1, 1));
}

#[test]
fn date_weekday_is_sunday_zero() {
    assert_eq!(day(1970, 1, 1).weekday(), 4);
    assert_eq!(day(2024, 1, 7).weekday(), 0);
    assert_eq!(day(2026, 3, 3).weekday(), 2);
}

#[test]
fn date_clamp_keeps_min_when_bounds_cross() {
    let min = Some(day(2024, 1, 10));
    let max = Some(day(2024, 1, 5));
    assert_eq!(day(2024, 1, 5).clamp_to(min, None), day(2024, 1, 10));
    assert_eq!(day(2024, 1, 7).clamp_to(None, max), day(2024, 1, 5));
    assert_eq!(day(2024, 1, 7).clamp_to(min, max), day(2024, 1, 10));
}

#[test]
fn date_month_grid_starts_on_the_chosen_weekday() {
    let sunday_first = rv2_affect::date_picker::month_grid(day(2024, 2, 1), 0);
    assert_eq!(sunday_first[0][0], day(2024, 1, 28));
    assert_eq!(sunday_first[5][6], day(2024, 3, 9));
    let monday_first = rv2_affect::date_picker::month_grid(day(2024, 2, 1), 1);
    assert_eq!(monday_first[0][0], day(2024, 1, 29));
}

#[test]
fn date_days_in_month_and_leap_years() {
    use rv2_affect::date_picker::{days_in_month, is_leap_year};
    assert_eq!(days_in_month(2023, 2), 28);
    assert_eq!(days_in_month(2024, 2), 29);
    assert_eq!(days_in_month(2024, 4), 30);
    assert!(!is_leap_year(1900));
    assert!(is_leap_year(2000));
}

#[test]
fn date_week_start_follows_region_and_locale() {
    use rv2_affect::date_picker::{fallback_week_start, week_start_for_region};
    assert_eq!(week_start_for_region("US"), 0);
    assert_eq!(week_start_for_region("DE"), 1);
    assert_eq!(fallback_week_start(Some("en-US")), 0);
    assert_eq!(fallback_week_start(Some("de-DE")), 1);
    assert_eq!(fallback_week_start(None), 1);
}

#[test]
fn date_english_labels_match_the_long_and_short_forms() {
    use rv2_affect::date_picker::{english_long, english_short, english_title};
    assert_eq!(english_long(day(2026, 3, 3)), "Tuesday, March 3, 2026");
    assert_eq!(english_short(day(2026, 3, 3)), "Tue, Mar 3, 2026");
    assert_eq!(english_title(day(2026, 3, 1)), "March 2026");
}

#[test]
fn date_keyboard_moves_by_day_week_month_and_year() {
    use rv2_affect::date_picker::{GridKey, key_target};
    let focus = day(2024, 1, 31);
    assert_eq!(
        key_target(focus, GridKey::Right, false, 0, false, None, None),
        day(2024, 2, 1)
    );
    assert_eq!(
        key_target(focus, GridKey::Right, false, 0, true, None, None),
        day(2024, 1, 30)
    );
    assert_eq!(
        key_target(focus, GridKey::PageDown, false, 0, false, None, None),
        day(2024, 2, 29)
    );
    assert_eq!(
        key_target(focus, GridKey::PageDown, true, 0, false, None, None),
        day(2025, 1, 31)
    );
    assert_eq!(
        key_target(
            focus,
            GridKey::Right,
            false,
            0,
            false,
            None,
            Some(day(2024, 1, 31))
        ),
        day(2024, 1, 31)
    );
}

#[test]
fn date_place_flips_the_card_above_when_there_is_no_room_below() {
    use rv2_affect::date_picker::{Edges, Extent, place};
    let view = Extent {
        width: 1000.0,
        height: 800.0,
    };
    let card = Extent {
        width: 300.0,
        height: 200.0,
    };
    let below = place(
        Edges {
            left: 100.0,
            top: 100.0,
            right: 200.0,
            bottom: 130.0,
        },
        card,
        view,
        false,
    );
    assert_eq!((below.left, below.top, below.up), (100.0, 136.0, false));
    let above = place(
        Edges {
            left: 100.0,
            top: 700.0,
            right: 200.0,
            bottom: 730.0,
        },
        card,
        view,
        false,
    );
    assert!(above.up);
    assert_eq!(above.top, 494.0);
}

#[test]
fn color_hex_round_trips_with_and_without_alpha() {
    use rv2_affect::color_picker::{Rgba, format_hex, parse_color};
    let c = Rgba {
        r: 1.0,
        g: 144.0,
        b: 254.0,
        a: 1.0,
    };
    assert_eq!(format_hex(c, false), "#0190fe");
    assert_eq!(format_hex(Rgba { a: 0.8, ..c }, true), "#0190fecc");
    assert_eq!(parse_color("0190FE"), Some(c));
    assert_eq!(
        parse_color("#09f"),
        Some(Rgba {
            r: 0.0,
            g: 153.0,
            b: 255.0,
            a: 1.0
        })
    );
    assert_eq!(parse_color("#12"), None);
    assert_eq!(parse_color("red"), None);
}

#[test]
fn color_parse_reads_rgb_and_rgba_functions() {
    use rv2_affect::color_picker::parse_color;
    let c = parse_color("rgb(1 144 254 / 80%)").expect("parses");
    assert!((c.a - 0.8).abs() < 1e-9);
    assert_eq!(parse_color("rgb(1,2)"), None);
    assert!(parse_color("rgba(1,144,254,.8)").is_some());
}

#[test]
fn color_is_complete_only_for_full_hex_or_closed_rgb() {
    use rv2_affect::color_picker::is_complete;
    assert!(is_complete("#0190fe"));
    assert!(is_complete("rgb(1,2,3)"));
    assert!(!is_complete("#09f"));
    assert!(!is_complete("rgb(1,2,3"));
}

#[test]
fn color_hsv_conversion_keeps_hue_for_greys() {
    use rv2_affect::color_picker::{Hsva, Rgba, to_hsva, to_rgba};
    let red = to_rgba(Hsva {
        h: 0.0,
        s: 1.0,
        v: 1.0,
        a: 1.0,
    });
    assert_eq!((red.r, red.g, red.b), (255.0, 0.0, 0.0));
    let grey = to_hsva(
        Rgba {
            r: 128.0,
            g: 128.0,
            b: 128.0,
            a: 1.0,
        },
        200.0,
        true,
        0.5,
    );
    assert_eq!(grey.h, 200.0);
    assert_eq!(grey.s, 0.0);
    let blue = to_hsva(
        Rgba {
            r: 0.0,
            g: 0.0,
            b: 255.0,
            a: 1.0,
        },
        0.0,
        false,
        0.0,
    );
    assert_eq!(blue.h, 240.0);
}

#[test]
fn color_hue_names_cover_the_wheel() {
    use rv2_affect::color_picker::hue_name;
    assert_eq!(hue_name(0.0), "red");
    assert_eq!(hue_name(120.0), "green");
    assert_eq!(hue_name(300.0), "pink");
    assert_eq!(hue_name(359.0), "red");
}

#[test]
fn color_keyboard_moves_the_area_and_sliders() {
    use rv2_affect::color_picker::{area_key, slider_key};
    assert_eq!(area_key("ArrowRight", false, 50.0, 50.0), Some((0.51, 0.5)));
    assert_eq!(area_key("Home", false, 50.0, 50.0), Some((0.0, 0.5)));
    assert_eq!(area_key("x", false, 50.0, 50.0), None);
    assert_eq!(slider_key("Home", false, 350.0, 360.0), Some(0.0));
    assert_eq!(slider_key("End", false, 350.0, 360.0), Some(360.0));
    assert_eq!(slider_key("Tab", false, 350.0, 360.0), None);
}

#[test]
fn color_card_flips_to_the_top_when_the_trigger_is_low() {
    use rv2_affect::color_picker::{CardAlign, CardSide, Edges, Extent, place_card};
    let view = Extent {
        width: 1000.0,
        height: 800.0,
    };
    let card = Extent {
        width: 300.0,
        height: 200.0,
    };
    let low = place_card(
        Edges {
            left: 100.0,
            top: 700.0,
            right: 200.0,
            bottom: 730.0,
        },
        card,
        view,
        CardSide::Bottom,
        CardAlign::Start,
    );
    assert_eq!(low.side, CardSide::Top);
    assert_eq!(low.top, 494.0);
}

#[test]
fn time_value_parses_hh_mm_and_rejects_bare_digits() {
    assert_eq!(from_value("09:30"), Some(570));
    assert_eq!(from_value("9:30:15"), Some(570));
    assert_eq!(from_value("9:30:15.25"), Some(570));
    assert_eq!(from_value("24:00"), None);
    assert_eq!(from_value("9:60"), None);
    assert_eq!(from_value("930"), None);
    assert_eq!(from_value("09:30:"), None);
}

#[test]
fn time_format_shows_twelve_or_twenty_four_hour_clock() {
    assert_eq!(format_time(570, true), "9:30 AM");
    assert_eq!(format_time(0, true), "12:00 AM");
    assert_eq!(format_time(780, true), "1:00 PM");
    assert_eq!(format_time(570, false), "09:30");
}

#[test]
fn time_bounds_fall_back_to_the_whole_day_when_unreadable() {
    assert_eq!(bounds(None, None), (0, 1439));
    assert_eq!(bounds(Some("09:00"), Some("17:00")), (540, 1020));
    assert_eq!(bounds(Some("bad"), None), (0, 1439));
}

#[test]
fn time_parse_reads_typed_twelve_and_twenty_four_hour_forms() {
    let am = ["a", "am"];
    let pm = ["p", "pm"];
    assert_eq!(parse_time("9", &am, &pm), Some(540));
    assert_eq!(parse_time("930", &am, &pm), Some(570));
    assert_eq!(parse_time("9:30 pm", &am, &pm), Some(1290));
    assert_eq!(parse_time("9p", &am, &pm), Some(1260));
    assert_eq!(parse_time("12am", &am, &pm), Some(0));
    assert_eq!(parse_time("21.30", &am, &pm), Some(1290));
    assert_eq!(parse_time("2130", &am, &pm), Some(1290));
    assert_eq!(parse_time("24:00", &am, &pm), Some(0));
    assert_eq!(parse_time("24", &am, &pm), Some(0));
    assert_eq!(parse_time("13 pm", &am, &pm), None);
    assert_eq!(parse_time("am9pm", &am, &pm), None);
    assert_eq!(parse_time("9:61", &am, &pm), None);
    assert_eq!(parse_time("x9", &am, &pm), None);
}

#[test]
fn otp_code_keeps_digits_only_when_numeric_and_folds_fullwidth_digits() {
    assert_eq!(clean_code("482 913", true, 6), "482913");
    assert_eq!(clean_code("482913 9", true, 6), "482913");
    assert_eq!(clean_code("ab1c", false, 6), "AB1C");
    assert_eq!(clean_code("12x", true, 2), "12");
    assert_eq!(clean_code("１２", true, 6), "12");
}

#[test]
fn otp_arrow_and_home_end_keys_stay_inside_the_code() {
    assert_eq!(key_move("ArrowLeft", 0, 3, 6), Some(0));
    assert_eq!(key_move("ArrowRight", 2, 3, 6), Some(3));
    assert_eq!(key_move("End", 0, 3, 6), Some(3));
    assert_eq!(key_move("End", 0, 0, 6), Some(0));
    assert_eq!(key_move("Tab", 0, 0, 6), None);
}

#[test]
fn otp_resend_clock_formats_seconds_minutes_and_days() {
    assert_eq!(clock(24), "0:24");
    assert_eq!(clock(65), "1:05");
    assert_eq!(clock(3900), "1:05:00");
    assert_eq!(clock(90000), "1d 1h");
}

#[test]
fn segment_count_text_groups_digits_and_hides_non_finite_numbers() {
    use rv2_affect::segmented_control::{SegmentCount, count_text};
    assert_eq!(
        count_text(&SegmentCount::Number(1284.0)),
        Some("1,284".to_string())
    );
    assert_eq!(
        count_text(&SegmentCount::Number(-1500.0)),
        Some("-1,500".to_string())
    );
    assert_eq!(count_text(&SegmentCount::Number(f64::NAN)), None);
    assert_eq!(
        count_text(&SegmentCount::Text("3+".to_string())),
        Some("3+".to_string())
    );
}

#[test]
fn text_field_accepts_change_focus_blur_and_key_handlers_and_still_renders() {
    let html = render(|| {
        rsx! {
            rv2_affect::text_field::TextField {
                default_value: "abc".to_string(),
                on_input: move |_| {},
                onchange: move |_| {},
                onfocus: move |_| {},
                onblur: move |_| {},
                onkeydown: move |_| {},
            }
        }
    });
    assert!(html.contains("class=\"ui-tf__input\""));
    assert!(html.contains("value=\"abc\""));
}

#[test]
fn text_area_accepts_change_focus_blur_and_key_handlers_and_still_renders() {
    let html = render(|| {
        rsx! {
            rv2_affect::text_area::TextArea {
                default_value: "hello".to_string(),
                on_input: move |_| {},
                onchange: move |_| {},
                onfocus: move |_| {},
                onblur: move |_| {},
                onkeydown: move |_| {},
            }
        }
    });
    assert!(html.contains("class=\"ui-ta__input\""));
    assert!(html.contains("hello"));
}

#[test]
fn number_input_accepts_change_input_focus_blur_and_key_handlers_and_still_renders() {
    let html = render(|| {
        rsx! {
            rv2_affect::number_input::NumberInput {
                default_value: 4.0,
                onchange: move |_| {},
                oninput: move |_| {},
                onfocus: move |_| {},
                onblur: move |_| {},
                onkeydown: move |_| {},
            }
        }
    });
    assert!(html.contains("role=\"spinbutton\""));
    assert!(html.contains("4"));
}

#[test]
fn toggle_group_accepts_a_key_handler_and_still_renders() {
    let html = render(|| {
        rsx! {
            rv2_affect::toggle_group::ToggleGroup {
                items: vec![item("a"), item("b")],
                onkeydown: move |_| {},
            }
        }
    });
    assert!(html.contains("role=\"group\""));
    assert_eq!(html.matches("class=\"ui-tgl__item\"").count(), 2);
}

#[test]
fn color_picker_trigger_accepts_click_focus_blur_and_key_handlers_and_still_renders() {
    let html = render(|| {
        rsx! {
            rv2_affect::color_picker::ColorPicker {
                default_value: "#0190fe".to_string(),
                onclick: move |_| {},
                onfocus: move |_| {},
                onblur: move |_| {},
                onkeydown: move |_| {},
            }
        }
    });
    assert!(html.contains("class=\"ui-clp__trigger\""));
}
