use dioxus::prelude::*;
use rv2_affect::date_picker::Day;
use rv2_kit::components::{
    charge_phase_builder::{
        Line, LineKind, ProductLine, Recurrence, Timing, parse_amount, period_end, totals,
    },
    connector_picker::{Connector, connected_count, grouped_matches, sample_connectors},
    connector_settings_card::{ConnectorValues, mask_key, validate_site_url},
    file_drop_area::{ItemState, extension_of, format_bytes, rejection},
    fill_in_prompt_templates::{Template, blank_names, estimate_tokens, parse_body, render_prompt},
    model_selector::{ThinkingEffort, bars_filled, first_token_text, price_text},
    multi_select_lookup::{
        LookupOption, highlight_parts, matching_indices, overflow, toggle_pick, visible_rows,
    },
    multi_step_wizard_form::{add_chip, compact_line, split_chips},
    onboarding_checklist_panel::{
        CheckGroup, CheckItem, apply_tick, locked_flags, progress_percent,
    },
    period_picker::{ComparisonMode, DateRange, comparison_range, format_range, resolve_builtin},
    quick_action_palette::{Command, CommandGroup, filtered_rows, quick_slot, section_jump},
    sheet_grid::{
        Cell, CheckState, Column, ColumnKind, GridRow, History, copy_text, format_number,
        header_check, move_cell, next_sort, parse_paste, paste_into,
    },
    transient_notice_stack::{HoldTime, tick},
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn day(year: i32, month: u8, day: u8) -> Day {
    Day::new(year, month, day).expect("valid date")
}

fn option(value: &str, label: &str) -> LookupOption {
    LookupOption::new(value, label)
}

#[test]
fn lookup_matches_label_and_terms_case_insensitively() {
    let mut second = option("b", "Lisbon");
    second.match_terms = vec!["Portugal".to_string()];
    let options = vec![option("a", "Madrid"), second];
    assert_eq!(matching_indices(&options, "MA"), vec![0]);
    assert_eq!(matching_indices(&options, "port"), vec![1]);
    assert_eq!(
        highlight_parts("Madrid", "ad"),
        ("M".into(), "ad".into(), "rid".into())
    );
}

#[test]
fn lookup_toggles_picks_and_replaces_in_single_mode() {
    let picked = vec!["a".to_string()];
    assert_eq!(toggle_pick(&picked, "a", true), Vec::<String>::new());
    assert_eq!(
        toggle_pick(&picked, "b", true),
        vec!["a".to_string(), "b".to_string()]
    );
    assert_eq!(toggle_pick(&picked, "b", false), vec!["b".to_string()]);
}

#[test]
fn lookup_overflow_counts_hidden_chips() {
    assert_eq!(overflow(5, Some(2)), (2, 3));
    assert_eq!(overflow(2, None), (2, 0));
}

#[test]
fn lookup_offers_create_row_last_only_for_new_text() {
    let options = vec![option("ma", "Madrid")];
    assert_eq!(visible_rows(&options, "Oslo", true), vec![None]);
    assert_eq!(visible_rows(&options, "madrid", true), vec![Some(0)]);
}

#[test]
fn lookup_renders_combobox_and_chips() {
    let html = render(|| {
        rsx! {
            rv2_kit::components::multi_select_lookup::MultiSelectLookup {
                option_list: vec![option("a", "Alpha"), option("b", "Beta")],
                initial_selection: vec!["a".to_string()],
                start_open: true,
            }
        }
    });
    assert!(html.contains("role=\"combobox\""));
    assert!(html.contains("role=\"listbox\""));
    assert_eq!(html.matches("class=\"kit-msl__chip\"").count(), 1);
}

#[test]
fn palette_filters_and_jumps_sections() {
    let groups = vec![
        CommandGroup {
            group_id: "suggested".into(),
            group_name: "Suggested".into(),
            commands: vec![Command::new("x", "Export CSV"), Command::new("y", "Invite")],
        },
        CommandGroup {
            group_id: "nav".into(),
            group_name: "Navigate".into(),
            commands: vec![Command::new("z", "Settings")],
        },
    ];
    assert_eq!(filtered_rows(&groups, "set"), vec![(1, 0)]);
    let rows = filtered_rows(&groups, "");
    assert_eq!(section_jump(&rows, 0, 1), 2);
    assert_eq!(section_jump(&rows, 2, -1), 0);
    assert_eq!(quick_slot(&groups, 2).as_deref(), Some("y"));
}

#[test]
fn palette_renders_listbox() {
    let html = render(|| {
        rsx! { rv2_kit::components::quick_action_palette::QuickActionPalette { command_groups: vec![CommandGroup { group_id: "g".into(), group_name: "Tools".into(), commands: vec![Command::new("c", "Run")] }] } }
    });
    assert!(html.contains("role=\"listbox\""));
    assert!(html.contains("role=\"option\""));
}

#[test]
fn period_comparison_spans_are_equal_length_and_adjacent() {
    let range = DateRange::ordered(day(2026, 3, 1), day(2026, 3, 10));
    let previous = comparison_range(range, ComparisonMode::PreviousPeriod).expect("span");
    assert_eq!(previous.days(), range.days());
    assert_eq!(previous.to, day(2026, 2, 28));
    let year = comparison_range(range, ComparisonMode::PreviousYear).expect("span");
    assert_eq!(year.from, day(2025, 3, 1));
    assert_eq!(comparison_range(range, ComparisonMode::None), None);
}

#[test]
fn period_presets_resolve_against_today() {
    let today = day(2026, 5, 15);
    assert_eq!(resolve_builtin("last_7", today).from, day(2026, 5, 9));
    assert_eq!(resolve_builtin("this_quarter", today).from, day(2026, 4, 1));
    assert_eq!(
        format_range(resolve_builtin("year_to_date", today)),
        "Jan 1, 2026 – May 15, 2026"
    );
}

#[test]
fn model_bars_clamp_and_router_text() {
    assert_eq!(bars_filled(Some(9)), 5);
    assert_eq!(bars_filled(None), 0);
    assert_eq!(price_text(None), "Varies");
    assert_eq!(
        first_token_text(Some(1.0), ThinkingEffort::High, true),
        "1.6s"
    );
    assert_eq!(
        first_token_text(Some(1.0), ThinkingEffort::High, false),
        "1.0s"
    );
}

#[test]
fn model_selector_renders_trigger_and_no_menu_when_closed() {
    let html = render(|| rsx! { rv2_kit::components::model_selector::ModelSelector {} });
    assert!(html.contains("aria-haspopup=\"listbox\""));
    assert!(!html.contains("role=\"listbox\""));
}

#[test]
fn file_rejection_checks_type_then_size() {
    let accepted = vec![".pdf".to_string()];
    assert_eq!(extension_of("Report.PDF"), ".pdf");
    assert!(rejection("a.pdf", 10, &accepted, 100).is_none());
    assert!(rejection("a.exe", 10, &accepted, 100).is_some());
    assert!(
        rejection("a.pdf", 200, &accepted, 100)
            .unwrap()
            .starts_with("Too large")
    );
    assert_eq!(format_bytes(1536), "1.5 KB");
    let _ = ItemState::Queued;
}

#[test]
fn file_rejection_accepts_extensions_with_or_without_dot() {
    let bare = vec!["pdf".to_string(), "CSV".to_string()];
    assert!(rejection("report.pdf", 10, &bare, 100).is_none());
    assert!(rejection("data.csv", 10, &bare, 100).is_none());
    assert!(rejection("a.exe", 10, &bare, 100).is_some());
}

#[test]
fn wizard_chip_and_step_helpers() {
    assert_eq!(split_chips("a@x.io, b@y.io,,"), vec!["a@x.io", "b@y.io"]);
    assert_eq!(add_chip(&["a".to_string()], "a"), vec!["a".to_string()]);
    assert_eq!(compact_line(1, 4), "Step 2 of 4");
}

#[test]
fn prompt_blanks_parse_repeat_and_substitute() {
    let body = "Write for {audience}, then {audience} again. {bad name} {}";
    assert_eq!(blank_names(body), vec!["audience".to_string()]);
    let mut values = std::collections::BTreeMap::new();
    values.insert("audience".to_string(), "kids".to_string());
    assert_eq!(
        render_prompt(body, &values),
        "Write for kids, then kids again. {bad name} {}"
    );
    assert!(parse_body("{x}").len() == 1);
    assert_eq!(estimate_tokens("abcdefgh"), 2);
    let _ = Template {
        template_id: String::new(),
        template_title: String::new(),
        group_label: None,
        group_key: None,
        template_body: String::new(),
        blank_specs: Default::default(),
    };
}

#[test]
fn charge_totals_apply_discount_minimum_and_floor() {
    let product = |amount: &str, timing: Timing| Line {
        line_id: amount.to_string(),
        kind: LineKind::Product(ProductLine {
            name: "p".into(),
            recurrence: Recurrence::Monthly,
            timing,
            amount: amount.into(),
            quantity: "2".into(),
        }),
    };
    let lines = vec![
        product("100", Timing::UpFront),
        product("50", Timing::PeriodEnd),
        Line {
            line_id: "d".into(),
            kind: LineKind::Discount {
                percent: true,
                amount: "10".into(),
            },
        },
        Line {
            line_id: "m".into(),
            kind: LineKind::Minimum {
                amount: "400".into(),
            },
        },
    ];
    let sums = totals(&lines);
    assert_eq!(sums.subtotal, 300.0);
    assert_eq!(sums.discount, 30.0);
    assert_eq!(sums.phase_sum, 400.0);
    assert_eq!(sums.initial_charge, 200.0);
    let floored = totals(&[Line {
        line_id: "d".into(),
        kind: LineKind::Discount {
            percent: false,
            amount: "999".into(),
        },
    }]);
    assert_eq!(floored.phase_sum, 0.0);
    assert_eq!(parse_amount("1,200.5"), Some(1200.5));
    assert_eq!(parse_amount("-3"), None);
    assert_eq!(period_end(day(2026, 1, 1), 3), Some(day(2026, 3, 31)));
    let _ = Recurrence::Once;
}

#[test]
fn connectors_count_and_group_the_sample() {
    let list = sample_connectors();
    assert_eq!(list.len(), 10);
    assert_eq!(connected_count(&list), 3);
    let (connected, available) = grouped_matches(&list, "");
    assert_eq!(connected.len(), 3);
    assert_eq!(available.len(), 7);
    let (_, none) = grouped_matches(&list, "zzz");
    assert!(none.is_empty());
    let _: Option<&Connector> = list.first();
}

#[test]
fn connector_renders_title_with_product_name() {
    let html = render(
        || rsx! { rv2_kit::components::connector_picker::ConnectorPicker { product_name: "Acme" } },
    );
    assert!(html.contains("Acme + Storefront"));
    assert!(html.contains("3 of 10 connected"));
}

#[test]
fn settings_validates_scheme_and_host() {
    assert!(validate_site_url("https://shop.example/path").is_ok());
    assert!(validate_site_url("shop.example").is_err());
    assert!(validate_site_url("ftp://x.y").is_err());
    assert!(validate_site_url("https://").is_err());
    assert_eq!(mask_key("abcd1234"), "••••••••••••1234");
    let _ = ConnectorValues::default();
}

#[test]
fn checklist_unlocks_chain_and_relocks_on_untick() {
    let task = |id: &str, done: bool| CheckItem {
        item_id: id.into(),
        item_text: id.into(),
        is_done: done,
        is_locked: false,
    };
    let groups = vec![CheckGroup {
        group_id: "g".into(),
        group_title: "G".into(),
        task_list: vec![task("a", false), task("b", false), task("c", false)],
    }];
    assert_eq!(locked_flags(&groups[0], &[]), vec![false, true, true]);
    let after_a = apply_tick(&groups, &[], "a");
    assert_eq!(after_a, vec!["a".to_string()]);
    let after_b = apply_tick(&groups, &after_a, "b");
    assert_eq!(after_b.len(), 2);
    assert!(apply_tick(&groups, &[], "c").is_empty());
    assert!(apply_tick(&groups, &after_b, "a").is_empty());
    assert_eq!(progress_percent(1, 3), 33);
}

#[test]
fn checklist_renders_progress_and_groups() {
    let html = render(|| {
        rsx! {
            rv2_kit::components::onboarding_checklist_panel::OnboardingChecklistPanel {
                group_list: vec![CheckGroup { group_id: "g".into(), group_title: "Basics".into(), task_list: vec![] }],
            }
        }
    });
    assert!(html.contains("role=\"progressbar\""));
    assert!(html.contains("aria-expanded=\"true\""));
}

#[test]
fn notice_timer_expires_and_infinite_holds() {
    let mut remaining = 200;
    assert!(!tick(&mut remaining, HoldTime::Millis(200), 100));
    assert!(tick(&mut remaining, HoldTime::Millis(200), 100));
    let mut forever = 0;
    assert!(!tick(&mut forever, HoldTime::Infinite, 100));
}

#[test]
fn sheet_moves_clamp_and_jump_to_edges() {
    let start = Cell { row: 0, col: 0 };
    assert_eq!(move_cell(start, 3, 3, (-1, 0), false), start);
    assert_eq!(
        move_cell(start, 3, 3, (1, 1), true),
        Cell { row: 2, col: 2 }
    );
}

#[test]
fn sheet_copy_and_paste_round_trip_tsv() {
    let columns = vec![
        Column::new("a", "A", ColumnKind::Text),
        Column::new("b", "B", ColumnKind::Number),
    ];
    let mut rows = vec![
        GridRow {
            row_id: "1".into(),
            row_logo: None,
            values: [
                ("a".to_string(), "x".to_string()),
                ("b".to_string(), "1".to_string()),
            ]
            .into_iter()
            .collect(),
        },
        GridRow {
            row_id: "2".into(),
            row_logo: None,
            values: Default::default(),
        },
    ];
    assert_eq!(copy_text(&rows, &columns, (0, 1, 0, 1)), "x\t1\n\t");
    let grid = parse_paste("p\tq\nr\ts");
    paste_into(&mut rows, &columns, Cell { row: 0, col: 0 }, &grid);
    assert_eq!(rows[1].values.get("b").map(String::as_str), Some("s"));
}

#[test]
fn sheet_sort_header_check_and_numbers() {
    assert_eq!(next_sort(None, 1), Some((1, true)));
    assert_eq!(next_sort(Some((1, true)), 1), Some((1, false)));
    assert_eq!(header_check(1, 3), CheckState::Mixed);
    assert_eq!(format_number("1234567.5"), "1,234,567.5");
    assert_eq!(format_number("-4200"), "−4,200");
}

#[test]
fn sheet_history_caps_undo_and_clears_redo() {
    let mut history = History::default();
    history.record(1);
    assert_eq!(history.undo(2), Some(1));
    assert_eq!(history.redo(1), Some(2));
    for i in 0..150 {
        history.record(i);
    }
    let mut undone = 0;
    while history.undo(0).is_some() {
        undone += 1;
    }
    assert_eq!(undone, 100);
}

#[test]
fn sheet_renders_grid_roles() {
    let html = render(|| {
        rsx! {
            rv2_kit::components::sheet_grid::SheetGrid {
                grid_columns: vec![Column::new("a", "Name", ColumnKind::Title)],
                grid_rows: vec![GridRow { row_id: "r".into(), row_logo: None, values: Default::default() }],
                read_only: true,
            }
        }
    });
    assert!(html.contains("role=\"grid\""));
    assert!(!html.contains("Add row"));
}

#[test]
fn file_drop_renders_zone_and_summary() {
    let html = render(|| rsx! { rv2_kit::components::file_drop_area::FileDropArea {} });
    assert!(html.contains("Drop files here or browse"));
    assert!(html.contains("role=\"status\""));
}

#[test]
fn wizard_renders_first_step_and_no_back_button() {
    let html = render(|| {
        rsx! {
            rv2_kit::components::multi_step_wizard_form::MultiStepWizardForm {
                step_list: vec![rv2_kit::components::multi_step_wizard_form::WizardStep {
                    step_id: "one".into(),
                    step_title: "Team".into(),
                    step_heading: None,
                    step_description: None,
                    render_step: Callback::new(|_| rsx! { p { "body" } }),
                    check_step: None,
                }],
                on_complete: Callback::new(|_| Box::pin(async { Ok(()) }) as rv2_kit::components::multi_step_wizard_form::Task),
            }
        }
    });
    assert!(html.contains("Step 1 of 1"));
    assert!(!html.contains(">Back<"));
}

#[test]
fn prompt_templates_render_listbox_and_disabled_use() {
    let html = render(|| {
        rsx! {
            rv2_kit::components::fill_in_prompt_templates::FillInPromptTemplates {
                template_list: vec![Template {
                    template_id: "t".into(),
                    template_title: "Summary".into(),
                    group_label: None,
                    group_key: None,
                    template_body: "Hi {name}".into(),
                    blank_specs: Default::default(),
                }],
            }
        }
    });
    assert!(html.contains("role=\"listbox\""));
    assert!(html.contains("0 of 1 filled"));
    assert!(html.contains("aria-disabled=\"true\""));
}

#[test]
fn settings_card_renders_enabled_switch_and_fields() {
    let html = render(
        || rsx! { rv2_kit::components::connector_settings_card::ConnectorSettingsCard { service_name: "Shop" } },
    );
    assert!(html.contains("Shop"));
    assert!(html.contains("role=\"switch\""));
}

#[test]
fn notice_host_renders_two_live_regions() {
    let html = render(|| rsx! { rv2_kit::components::transient_notice_stack::NoticeHost {} });
    assert!(html.contains("aria-label=\"Notifications\""));
}

#[test]
fn notice_stack_at_bottom_centre_names_its_focus_key_and_sits_after_the_host_content() {
    use rv2_kit::components::transient_notice_stack::StackEdge;
    let html = render(|| {
        rsx! {
            rv2_kit::components::transient_notice_stack::NoticeHost {
                stack_edge: StackEdge::BottomCenter,
                focus_key: Some("n".to_string()),
                region_label: "Notices, press N to focus",
                p { "Host content" }
            }
        }
    });
    assert!(html.contains("data-edge=\"bottom-center\""));
    assert!(html.contains("aria-label=\"Notices, press N to focus\""));
    assert!(html.find("Host content") < html.find("data-edge=\"bottom-center\""));
}

#[test]
fn charge_builder_renders_table_with_headers() {
    let html = render(|| rsx! { rv2_kit::components::charge_phase_builder::ChargePhaseBuilder {} });
    assert!(html.contains("<th"));
    assert!(html.contains("Initial charge"));
}

#[test]
fn period_picker_closed_shows_trigger_only() {
    let html = render(|| rsx! { rv2_kit::components::period_picker::PeriodPicker {} });
    assert!(html.contains("aria-haspopup=\"dialog\""));
    assert!(!html.contains("role=\"gridcell\""));
}

#[test]
fn period_picker_open_renders_day_cells_and_range_text_once() {
    use rv2_affect::date_picker::Day;
    use rv2_kit::components::period_picker::{DateRange, PeriodPicker};
    let html = render(|| {
        let day = |d: u8| Day {
            year: 2026,
            month: 11,
            day: d,
        };
        rsx! {
            PeriodPicker {
                starting_range: Some(DateRange { from: day(3), to: day(14) }),
                anchor_today: Some(day(21)),
                start_expanded: true,
            }
        }
    });
    assert!(html.matches("role=\"gridcell\"").count() >= 56, "{html}");
    assert!(html.contains("data-start=\"true\""), "{html}");
    assert!(html.contains("12 days"), "{html}");
    assert_eq!(
        html.matches("Nov 3, 2026 – Nov 14, 2026").count(),
        1,
        "{html}"
    );
}

#[test]
fn file_drop_zone_is_a_label_for_the_picker() {
    let html = render(|| rsx! { rv2_kit::components::file_drop_area::FileDropArea {} });
    assert!(html.contains("<label class=\"kit-fda__zone\""));
    assert!(html.contains("role=\"button\""));
    assert!(html.contains("for=\"kit-fda-"));
}

fn file_drop_rest_rows() -> Vec<rv2_kit::components::file_drop_area::FileItem> {
    use rv2_kit::components::file_drop_area::FileItem;
    vec![
        FileItem {
            item_id: "close".into(),
            file_name: "close-pack.csv".into(),
            byte_size: 3_984_589,
            item_state: ItemState::Uploading,
            progress: 0.4,
            error_text: None,
            seconds_left: Some(6),
        },
        FileItem {
            item_id: "brief".into(),
            file_name: "brief.pdf".into(),
            byte_size: 239_616,
            item_state: ItemState::Done,
            progress: 1.0,
            error_text: None,
            seconds_left: None,
        },
        FileItem {
            item_id: "setup".into(),
            file_name: "setup.exe".into(),
            byte_size: 12_582_912,
            item_state: ItemState::Rejected,
            progress: 0.0,
            error_text: Some("Type not permitted".into()),
            seconds_left: None,
        },
    ]
}

#[test]
fn file_drop_at_rest_shows_one_symbol_and_the_title_and_hint() {
    let html = render(|| {
        rsx! {
            rv2_kit::components::file_drop_area::FileDropArea {
                title_text: Some("Add files to this list".to_string()),
                hint_text: Some("Accepts PDF and CSV".to_string()),
            }
        }
    });
    assert_eq!(html.matches("class=\"kit-fda__symbol\"").count(), 1);
    assert!(html.contains("Add files to this list"));
    assert!(html.contains("Accepts PDF and CSV"));
}

#[test]
fn file_drop_rest_list_has_in_progress_stored_and_refused_rows_in_order() {
    let html = render(|| {
        rsx! {
            rv2_kit::components::file_drop_area::FileDropArea { initial_items: file_drop_rest_rows() }
        }
    });
    let progress = html.find("close-pack.csv").expect("in-progress row");
    let stored = html.find("brief.pdf").expect("stored row");
    let refused = html.find("setup.exe").expect("refused row");
    assert!(progress < stored && stored < refused, "{html}");
    assert!(
        html.contains("1.5 MB of 3.8 MB · about 6 seconds left"),
        "{html}"
    );
    assert!(html.contains("Stored"), "{html}");
    assert!(html.contains("Type not permitted"), "{html}");
    assert_eq!(html.matches("role=\"progressbar\"").count(), 2, "{html}");
}

#[test]
fn file_drop_rest_summary_counts_each_state_once() {
    let html = render(|| {
        rsx! {
            rv2_kit::components::file_drop_area::FileDropArea { initial_items: file_drop_rest_rows() }
        }
    });
    assert!(
        html.contains("1 in progress · 1 stored · 1 refused"),
        "{html}"
    );
}

#[test]
fn file_drop_refused_row_has_discard_but_no_try_again() {
    let html = render(|| {
        rsx! {
            rv2_kit::components::file_drop_area::FileDropArea { initial_items: file_drop_rest_rows() }
        }
    });
    assert_eq!(html.matches(">Discard<").count(), 3, "{html}");
    assert_eq!(html.matches(">Try again<").count(), 0, "{html}");
    assert!(!html.contains(">Remove<") && !html.contains(">Retry<"));
}

#[test]
fn file_drop_summary_appends_failed_only_when_an_upload_failed() {
    use rv2_kit::components::file_drop_area::summary_text;
    assert_eq!(
        summary_text((1, 1, 1, 0)),
        "1 in progress · 1 stored · 1 refused"
    );
    assert_eq!(
        summary_text((0, 2, 0, 1)),
        "0 in progress · 2 stored · 0 refused · 1 failed"
    );
}

#[test]
fn file_drop_time_left_reads_seconds_then_minutes() {
    use rv2_kit::components::file_drop_area::{seconds_left_from, time_left_text};
    assert_eq!(time_left_text(1), "about 1 second left");
    assert_eq!(time_left_text(6), "about 6 seconds left");
    assert_eq!(time_left_text(61), "about 2 minutes left");
    assert_eq!(seconds_left_from(0.0, 0.5), None);
    assert_eq!(seconds_left_from(1000.0, 0.0), None);
    assert_eq!(seconds_left_from(1000.0, 0.5), Some(1));
    assert_eq!(seconds_left_from(3000.0, 0.4), Some(5));
}

#[test]
fn file_drop_sizes_drop_a_trailing_point_zero() {
    assert_eq!(format_bytes(239_616), "234 KB");
    assert_eq!(format_bytes(12_582_912), "12 MB");
    assert_eq!(format_bytes(3_984_589), "3.8 MB");
}

#[test]
fn fill_in_footer_counts_blanks_still_empty() {
    use rv2_kit::components::fill_in_prompt_templates::blanks_remaining_text;
    assert_eq!(blanks_remaining_text(1), "1 blank still empty");
    assert_eq!(blanks_remaining_text(2), "2 blanks still empty");
}
