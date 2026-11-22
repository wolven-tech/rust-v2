use std::collections::BTreeMap;

use dioxus::prelude::*;
use rv2_affect::breadcrumb::{Breadcrumb, BreadcrumbItem, folded_count, more_label};
use rv2_affect::filter_menu::{
    Chip, FilterGroup, FilterMenu, FilterOption, FilterValue, Row as FilterRow, chips,
    initial_open, picked_count, rows as filter_rows, toggle_option,
};
use rv2_affect::pagination::{
    Pagination, RangeText, Slot, clamp_page, group_digits, next_tight, page_after_size_change,
    page_count, page_slots, range_text, size_choices,
};
use rv2_affect::profile_menu::{
    NavKey, ProfileMenu, ProfileSetup, next_index, setup_percent, setup_ratio,
};
use rv2_affect::resizable::PaneConfig;
use rv2_affect::resizable::{
    Direction, ResizableHandle, ResizablePanel, ResizablePanelGroup, distribute, pane_config,
    shift, toggle,
};
use rv2_affect::scroll_area::{ScrollArea, ScrollOrientation, ScrollbarVisibility};
use rv2_affect::tree_view::{
    Row, TreeAction, TreeKey, TreeNode, TreeView, TypeAhead, key_actions, normalize,
    type_ahead_target, visible_rows,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

fn cfg(min: f64, max: f64) -> PaneConfig {
    pane_config(min, max, false, 0.0, None)
}

fn collapsible(min: f64, collapsed: f64) -> PaneConfig {
    pane_config(min, 100.0, true, collapsed, None)
}

fn leaf(id: &str, label: &str) -> TreeNode {
    TreeNode {
        id: id.to_string(),
        label: label.to_string(),
        icon: None,
        open_icon: None,
        meta: None,
        disabled: false,
        children: None,
    }
}

fn branch(id: &str, label: &str, children: Vec<TreeNode>) -> TreeNode {
    TreeNode {
        children: Some(children),
        ..leaf(id, label)
    }
}

fn row(id: &str, parent: Option<&str>, branch: bool, expanded: bool, disabled: bool) -> Row {
    Row {
        id: id.to_string(),
        label: id.to_string(),
        depth: usize::from(parent.is_some()),
        parent: parent.map(str::to_string),
        pos: 1,
        size: 1,
        branch,
        expanded,
        disabled,
    }
}

#[test]
fn page_slots_show_first_run_then_gap_and_last_page_when_near_the_start() {
    assert_eq!(
        page_slots(1, 10, 1),
        vec![
            Slot::Page(1),
            Slot::Page(2),
            Slot::Page(3),
            Slot::Page(4),
            Slot::Page(5),
            Slot::Gap,
            Slot::Page(10)
        ]
    );
}

#[test]
fn page_slots_keep_both_gaps_when_the_current_page_is_in_the_middle() {
    assert_eq!(
        page_slots(5, 10, 1),
        vec![
            Slot::Page(1),
            Slot::Gap,
            Slot::Page(4),
            Slot::Page(5),
            Slot::Page(6),
            Slot::Gap,
            Slot::Page(10)
        ]
    );
}

#[test]
fn page_slots_mirror_the_start_when_the_current_page_is_near_the_end() {
    assert_eq!(
        page_slots(9, 10, 1),
        vec![
            Slot::Page(1),
            Slot::Gap,
            Slot::Page(6),
            Slot::Page(7),
            Slot::Page(8),
            Slot::Page(9),
            Slot::Page(10)
        ]
    );
}

#[test]
fn page_slots_list_every_page_when_they_fit() {
    assert_eq!(
        page_slots(3, 5, 1),
        (1..=5).map(Slot::Page).collect::<Vec<_>>()
    );
}

#[test]
fn range_text_reads_rows_for_a_middle_page_and_groups_thousands() {
    assert_eq!(
        range_text(1, 20, Some(1284), 65),
        RangeText::Rows {
            first: "1".into(),
            last: "20".into(),
            total: "1,284".into(),
        }
    );
    assert_eq!(
        range_text(65, 20, Some(1284), 65),
        RangeText::Rows {
            first: "1,281".into(),
            last: "1,284".into(),
            total: "1,284".into(),
        }
    );
}

#[test]
fn range_text_reads_zero_of_zero_and_page_of_count_without_a_total() {
    assert_eq!(range_text(1, 20, Some(0), 1), RangeText::Empty);
    assert_eq!(
        range_text(3, 10, None, 10),
        RangeText::Pages {
            page: "3".into(),
            count: "10".into(),
        }
    );
}

#[test]
fn group_digits_puts_a_comma_every_three_digits() {
    assert_eq!(group_digits(0), "0");
    assert_eq!(group_digits(999), "999");
    assert_eq!(group_digits(1000), "1,000");
    assert_eq!(group_digits(1_234_567), "1,234,567");
}

#[test]
fn page_count_rounds_rows_up_and_never_goes_below_one() {
    assert_eq!(page_count(Some(0), 20, 1), 1);
    assert_eq!(page_count(Some(41), 20, 1), 3);
    assert_eq!(page_count(None, 20, 7), 7);
    assert_eq!(clamp_page(0, 3), 1);
    assert_eq!(clamp_page(9, 3), 3);
}

#[test]
fn a_size_change_keeps_the_first_row_that_was_on_screen() {
    assert_eq!(page_after_size_change(3, 10, 25), 1);
    assert_eq!(page_after_size_change(3, 10, 20), 2);
}

#[test]
fn size_choices_add_the_current_size_once_and_sort() {
    assert_eq!(size_choices(&[20, 10, 50, 10], 25), vec![10, 20, 25, 50]);
}

#[test]
fn the_nav_goes_narrow_on_overflow_and_wide_only_past_the_narrow_width() {
    assert_eq!(next_tight(false, 0.0, 300.0, true), (true, 300.0));
    assert_eq!(next_tight(false, 0.0, 300.0, false), (false, 0.0));
    assert_eq!(next_tight(true, 300.0, 300.0, false), (true, 300.0));
    assert_eq!(next_tight(true, 300.0, 302.0, false), (false, 300.0));
}

#[test]
fn pagination_renders_the_range_and_disables_previous_on_the_first_page() {
    let html = render(
        || rsx! { Pagination { page: 1, total: 1284, page_size: 20, on_page_change: |_| {} } },
    );
    assert!(html.contains("class=\"ui-pgnc\""));
    assert!(html.contains("1–20"));
    assert!(html.contains("1,284"));
    assert!(html.contains("aria-label=\"Previous page\""));
    assert!(html.contains("aria-disabled=\"true\""));
    assert!(html.contains("aria-current=\"page\""));
}

#[test]
fn pagination_marks_the_current_page_and_shows_an_ellipsis_between_runs() {
    let html = render(
        || rsx! { Pagination { page: 5, total: 200, page_size: 10, on_page_change: |_| {} } },
    );
    assert!(html.contains("data-page=\"5\""));
    assert!(html.contains("aria-current=\"page\""));
    assert!(html.contains("ui-pgnc__gap"));
    assert!(html.contains("Page 20"));
}

#[test]
fn compact_pagination_shows_the_range_and_arrows_without_page_numbers() {
    let html = render(
        || rsx! { Pagination { page: 2, total: 200, page_size: 10, compact: true, on_page_change: |_| {} } },
    );
    assert!(html.contains("Next page"));
    assert!(!html.contains("ui-pgnc__num"));
}

#[test]
fn folded_count_hides_the_middle_past_max_items_and_adds_width_folds() {
    assert_eq!(folded_count(10, 4, 0), 6);
    assert_eq!(folded_count(10, 4, 2), 8);
    assert_eq!(folded_count(10, 4, 99), 8);
    assert_eq!(folded_count(5, 2, 0), 3);
}

#[test]
fn folded_count_never_hides_the_first_or_the_last_crumb() {
    assert_eq!(folded_count(3, 4, 0), 0);
    assert_eq!(folded_count(2, 1, 5), 0);
    assert_eq!(folded_count(0, 4, 0), 0);
    assert_eq!(folded_count(1, 4, 0), 0);
}

#[test]
fn more_label_counts_the_hidden_crumbs() {
    assert_eq!(more_label(1), "Show 1 more crumb");
    assert_eq!(more_label(3), "Show 3 more crumbs");
}

#[test]
fn breadcrumb_links_every_crumb_but_the_last_and_marks_the_last_as_current() {
    let html = render(|| {
        let items = vec![
            BreadcrumbItem {
                label: "Home".into(),
                href: Some("/".into()),
                onclick: None,
                icon: None,
            },
            BreadcrumbItem {
                label: "Reports".into(),
                href: Some("/reports".into()),
                onclick: None,
                icon: None,
            },
            BreadcrumbItem {
                label: "Q3".into(),
                href: None,
                onclick: None,
                icon: None,
            },
        ];
        rsx! { Breadcrumb { items } }
    });
    assert!(html.contains("aria-label=\"Breadcrumb\""));
    assert!(html.contains("href=\"/reports\""));
    assert!(html.contains("aria-current=\"page\""));
    assert!(!html.contains("ui-bcrc__more"));
}

#[test]
fn breadcrumb_folds_its_middle_into_a_more_button_past_max_items() {
    let html = render(|| {
        let items = [
            "Root", "Folder A", "Folder B", "Folder C", "Folder D", "Here",
        ]
        .into_iter()
        .map(|label| BreadcrumbItem {
            label: label.into(),
            href: None,
            onclick: None,
            icon: None,
        })
        .collect::<Vec<_>>();
        rsx! { Breadcrumb { items } }
    });
    assert!(html.contains("ui-bcrc__more"));
    assert!(html.contains("aria-label=\"Show 2 more crumbs\""));
    assert!(!html.contains(">Folder A<"));
    assert!(html.contains(">Folder D<"));
}

fn filter_groups() -> Vec<FilterGroup> {
    vec![
        FilterGroup {
            id: "score".into(),
            label: "Score".into(),
            options: vec![
                FilterOption {
                    id: "80".into(),
                    label: "80–90".into(),
                },
                FilterOption {
                    id: "90".into(),
                    label: "90+".into(),
                },
            ],
            chip_prefix: Some("Score: ".into()),
        },
        FilterGroup {
            id: "stage".into(),
            label: "Stage".into(),
            options: vec![FilterOption {
                id: "open".into(),
                label: "Open".into(),
            }],
            chip_prefix: None,
        },
    ]
}

fn value(pairs: &[(&str, &[&str])]) -> FilterValue {
    pairs
        .iter()
        .map(|(group, ids)| {
            (
                group.to_string(),
                ids.iter().map(|id| id.to_string()).collect(),
            )
        })
        .collect::<BTreeMap<_, _>>()
}

#[test]
fn toggle_option_adds_then_removes_one_pick_and_keeps_other_groups() {
    let base = value(&[("stage", &["open"])]);
    let added = toggle_option(&base, "score", "80");
    assert_eq!(added.get("score"), Some(&vec!["80".to_string()]));
    assert_eq!(added.get("stage"), Some(&vec!["open".to_string()]));
    let removed = toggle_option(&added, "score", "80");
    assert_eq!(removed.get("score"), Some(&Vec::<String>::new()));
}

#[test]
fn chips_follow_group_then_option_order_and_use_the_chip_prefix() {
    let picked = value(&[("score", &["90", "80"]), ("stage", &["open"])]);
    let labels: Vec<String> = chips(&filter_groups(), &picked)
        .into_iter()
        .map(|c: Chip| c.label)
        .collect();
    assert_eq!(labels, vec!["Score: 80–90", "Score: 90+", "Open"]);
}

#[test]
fn picked_count_ignores_ids_that_match_no_option_and_repeats() {
    let groups = filter_groups();
    let picked = value(&[("score", &["80", "80", "missing"])]);
    assert_eq!(picked_count(&groups[0], &picked), 1);
}

#[test]
fn filter_rows_list_groups_and_only_the_options_of_open_groups() {
    let groups = filter_groups();
    let open = initial_open(&groups, None);
    assert_eq!(
        open.iter().cloned().collect::<Vec<_>>(),
        vec!["score".to_string()]
    );
    let keys: Vec<FilterRow> = filter_rows(&groups, &open);
    assert_eq!(keys.len(), 4);
    assert_eq!(keys[0].key(), "g:0");
    assert_eq!(keys[1].key(), "o:0:0");
    assert_eq!(keys[3].key(), "g:1");
}

#[test]
fn filter_menu_renders_checked_options_chips_and_a_disabled_clear_when_nothing_is_picked() {
    let html = render(|| {
        let picked = value(&[("score", &["90"])]);
        rsx! { FilterMenu { groups: filter_groups(), value: picked, title: "Deals" } }
    });
    assert!(html.contains("class=\"ui-fmc\""));
    assert!(html.contains("Deals"));
    assert!(html.contains("aria-checked=\"true\""));
    assert!(html.contains("Remove Score: 90+"));
    assert!(html.contains("1 filter"));

    let empty = render(|| rsx! { FilterMenu { groups: filter_groups() } });
    assert!(empty.contains("class=\"ui-fmc__clear\""));
    assert!(empty.contains("disabled"));
    assert!(empty.contains("to navigate"));
}

#[test]
fn setup_ratio_and_percent_clamp_and_round() {
    assert!(close(setup_ratio(3, 4), 0.75));
    assert!(close(setup_ratio(5, 4), 1.0));
    assert!(close(setup_ratio(1, 0), 0.0));
    assert_eq!(setup_percent(1, 3), 33);
    assert_eq!(setup_percent(2, 3), 67);
}

#[test]
fn next_index_moves_within_the_menu_and_wraps_at_both_ends() {
    assert_eq!(next_index(None, 3, NavKey::Down), Some(0));
    assert_eq!(next_index(Some(2), 3, NavKey::Down), Some(0));
    assert_eq!(next_index(Some(0), 3, NavKey::Up), Some(2));
    assert_eq!(next_index(None, 3, NavKey::Up), Some(2));
    assert_eq!(next_index(Some(1), 3, NavKey::Home), Some(0));
    assert_eq!(next_index(Some(1), 3, NavKey::End), Some(2));
    assert_eq!(next_index(None, 0, NavKey::Down), None);
}

#[test]
fn profile_menu_shows_the_user_default_actions_and_setup_progress() {
    let html = render(|| {
        let setup = ProfileSetup {
            done: 2,
            total: 5,
            next: Some("Add a photo".into()),
            on_continue: None,
        };
        rsx! { ProfileMenu { name: "Ada Lovelace", email: "ada@example.com", setup } }
    });
    assert!(html.contains("Ada Lovelace"));
    assert!(html.contains("ada@example.com"));
    assert!(html.contains("role=\"menuitem\""));
    assert!(html.contains("Settings"));
    assert!(html.contains("Logout"));
    assert!(html.contains(">New<"));
    assert!(html.contains("2/5"));
    assert!(html.contains("aria-valuenow=\"40\""));
    assert!(html.contains("Add a photo"));
}

#[test]
fn profile_menu_without_setup_renders_no_progress_bar() {
    let html = render(|| rsx! { ProfileMenu { name: "", email: "ada@example.com" } });
    assert!(!html.contains("ui-pmc__setup"));
    assert!(!html.contains("ui-pmc__name"));
}

#[test]
fn distribute_splits_what_is_left_evenly_and_keeps_known_sizes() {
    let even = distribute(&[None, None, None], &[None, None, None]);
    assert!(even.iter().all(|s| close(*s, 100.0 / 3.0)));

    assert_eq!(
        distribute(&[Some(30.0), None], &[None, None])
            .iter()
            .map(|s| s.round())
            .collect::<Vec<_>>(),
        vec![30.0, 70.0]
    );
    assert_eq!(
        distribute(&[None, None], &[Some(40.0), None])
            .iter()
            .map(|s| s.round())
            .collect::<Vec<_>>(),
        vec![40.0, 60.0]
    );
}

#[test]
fn drag_moves_the_handle_and_stops_at_the_limits() {
    let free = [cfg(0.0, 100.0), cfg(0.0, 100.0)];
    assert_eq!(
        shift(&[50.0, 50.0], &free, 0, 10.0, false),
        vec![60.0, 40.0]
    );
    assert_eq!(
        shift(&[50.0, 50.0], &free, 0, -10.0, false),
        vec![40.0, 60.0]
    );

    let capped = [cfg(0.0, 55.0), cfg(0.0, 100.0)];
    assert_eq!(
        shift(&[50.0, 50.0], &capped, 0, 10.0, false),
        vec![55.0, 45.0]
    );

    let floored = [cfg(0.0, 100.0), cfg(45.0, 100.0)];
    assert_eq!(
        shift(&[50.0, 50.0], &floored, 0, 10.0, false),
        vec![55.0, 45.0]
    );
}

#[test]
fn a_collapsible_pane_snaps_shut_only_when_pushed_past_half_its_minimum() {
    let cfgs = [collapsible(20.0, 0.0), cfg(0.0, 100.0)];
    assert_eq!(
        shift(&[30.0, 70.0], &cfgs, 0, -15.0, false),
        vec![20.0, 80.0]
    );
    assert_eq!(
        shift(&[30.0, 70.0], &cfgs, 0, -25.0, false),
        vec![0.0, 100.0]
    );
}

#[test]
fn a_folded_pane_opens_from_the_keyboard_at_once_but_from_a_drag_only_past_half() {
    let cfgs = [collapsible(20.0, 0.0), cfg(0.0, 100.0)];
    assert_eq!(shift(&[0.0, 100.0], &cfgs, 0, 5.0, true), vec![20.0, 80.0]);
    assert_eq!(shift(&[0.0, 100.0], &cfgs, 0, 5.0, false), vec![0.0, 100.0]);
}

#[test]
fn enter_folds_the_pane_before_the_handle_and_remembers_its_size() {
    let cfgs = [collapsible(20.0, 0.0), cfg(0.0, 100.0)];
    let folded = toggle(&[30.0, 70.0], &cfgs, 0, |_| None).expect("pane is collapsible");
    assert_eq!(folded.sizes, vec![0.0, 100.0]);
    assert_eq!(folded.side, 0);
    assert_eq!(folded.remember, Some(30.0));
}

#[test]
fn enter_on_a_folded_pane_brings_back_the_remembered_size() {
    let cfgs = [collapsible(20.0, 0.0), cfg(0.0, 100.0)];
    let opened = toggle(&[0.0, 100.0], &cfgs, 0, |side| {
        if side == 0 { Some(30.0) } else { None }
    })
    .expect("pane is collapsible");
    assert_eq!(opened.sizes, vec![30.0, 70.0]);
    assert_eq!(opened.remember, None);
}

#[test]
fn enter_does_nothing_when_neither_pane_beside_the_handle_collapses() {
    let cfgs = [cfg(0.0, 100.0), cfg(0.0, 100.0)];
    assert_eq!(toggle(&[50.0, 50.0], &cfgs, 0, |_| None), None);
}

#[test]
fn pane_config_holds_max_above_min_and_collapsed_within_min() {
    let c = pane_config(20.0, 150.0, true, 30.0, Some(40.0));
    assert!(close(c.min, 20.0));
    assert!(close(c.max, 100.0));
    assert!(close(c.collapsed, 20.0));
    assert_eq!(c.initial, Some(40.0));
    let tight = pane_config(50.0, 10.0, false, 0.0, None);
    assert!(close(tight.max, 50.0));
}

#[test]
fn resizable_group_renders_panes_and_a_separator_handle() {
    let html = render(|| {
        rsx! {
            ResizablePanelGroup { direction: Direction::Horizontal,
                ResizablePanel { id: "list", default_size: 30.0, "list" }
                ResizableHandle { with_handle: true }
                ResizablePanel { id: "detail", "detail" }
            }
        }
    });
    assert!(html.contains("data-direction=\"horizontal\""));
    assert!(html.contains("data-rszc-panel=\"list\""));
    assert!(html.contains("data-rszc-panel=\"detail\""));
    assert!(html.contains("role=\"separator\""));
    assert!(html.contains("aria-orientation=\"vertical\""));
    assert!(html.contains("ui-rszc__grip"));
}

#[test]
fn scroll_area_draws_a_vertical_bar_and_no_horizontal_one() {
    let html = render(|| rsx! { ScrollArea { max_height: "200px", "content" } });
    assert!(html.contains("data-orientation=\"vertical\""));
    assert!(html.contains("data-type=\"hover\""));
    assert!(html.contains("data-axis=\"y\""));
    assert!(!html.contains("data-axis=\"x\""));
    assert!(html.contains("max-height: 200px"));
}

#[test]
fn scroll_area_both_orientations_draw_both_bars_and_visibility_is_set() {
    let html = render(|| {
        rsx! { ScrollArea { orientation: ScrollOrientation::Both, visibility: ScrollbarVisibility::Always, fade: true, "wide" } }
    });
    assert!(html.contains("data-axis=\"y\""));
    assert!(html.contains("data-axis=\"x\""));
    assert!(html.contains("data-type=\"always\""));
    assert!(html.contains("data-fade"));
}

fn sample_tree() -> Vec<TreeNode> {
    vec![
        branch(
            "a",
            "Alpha",
            vec![leaf("a1", "Apple"), branch("a2", "Apricot", vec![])],
        ),
        leaf("b", "Banana"),
    ]
}

#[test]
fn normalize_keeps_the_first_of_repeated_ids() {
    let items = vec![leaf("x", "First"), leaf("x", "Second"), leaf("y", "Third")];
    let labels: Vec<String> = normalize(&items).into_iter().map(|n| n.label).collect();
    assert_eq!(labels, vec!["First", "Third"]);
}

#[test]
fn visible_rows_flatten_only_the_open_branches_with_their_place_among_siblings() {
    let rows = visible_rows(&normalize(&sample_tree()), &["a".to_string()]);
    let ids: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, vec!["a", "a1", "a2", "b"]);
    assert_eq!(rows[0].size, 2);
    assert_eq!(rows[0].pos, 1);
    assert!(rows[0].expanded);
    assert_eq!(rows[1].depth, 1);
    assert_eq!(rows[1].parent.as_deref(), Some("a"));
    assert_eq!(rows[3].pos, 2);
}

#[test]
fn down_and_up_move_between_visible_rows_and_skip_disabled_ones() {
    let rows = vec![
        row("a", None, false, false, false),
        row("c", None, false, false, true),
        row("d", None, false, false, false),
    ];
    assert_eq!(
        key_actions(TreeKey::Down, &rows, "a"),
        vec![TreeAction::Focus("d".into())]
    );
    assert_eq!(
        key_actions(TreeKey::Up, &rows, "d"),
        vec![TreeAction::Focus("a".into())]
    );
    assert!(key_actions(TreeKey::Up, &rows, "a").is_empty());
    assert!(key_actions(TreeKey::Down, &rows, "c").is_empty());
}

#[test]
fn right_opens_a_closed_branch_then_goes_to_its_first_child() {
    let closed = vec![
        row("a", None, true, false, false),
        row("b", None, false, false, false),
    ];
    assert_eq!(
        key_actions(TreeKey::Inward, &closed, "a"),
        vec![TreeAction::Expand {
            id: "a".into(),
            open: true
        }]
    );

    let mut child = row("a1", Some("a"), false, false, false);
    child.depth = 1;
    let open = vec![
        row("a", None, true, true, false),
        child,
        row("b", None, false, false, false),
    ];
    assert_eq!(
        key_actions(TreeKey::Inward, &open, "a"),
        vec![TreeAction::Focus("a1".into())]
    );
}

#[test]
fn left_closes_an_open_branch_then_goes_to_the_parent() {
    let open = vec![
        row("a", None, true, true, false),
        row("a1", Some("a"), false, false, false),
    ];
    assert_eq!(
        key_actions(TreeKey::Outward, &open, "a"),
        vec![TreeAction::Expand {
            id: "a".into(),
            open: false
        }]
    );
    assert_eq!(
        key_actions(TreeKey::Outward, &open, "a1"),
        vec![TreeAction::Focus("a".into())]
    );
}

#[test]
fn enter_selects_and_toggles_a_branch_while_space_only_selects() {
    let rows = vec![
        row("a", None, true, false, false),
        row("b", None, false, false, false),
    ];
    assert_eq!(
        key_actions(TreeKey::Enter, &rows, "a"),
        vec![
            TreeAction::Select("a".into()),
            TreeAction::Expand {
                id: "a".into(),
                open: true
            }
        ]
    );
    assert_eq!(
        key_actions(TreeKey::Space, &rows, "b"),
        vec![TreeAction::Select("b".into())]
    );
}

#[test]
fn star_opens_every_closed_sibling_branch() {
    let mut x = row("x", Some("root"), true, false, false);
    let mut y = row("y", Some("root"), true, true, false);
    let mut z = row("z", Some("root"), true, false, true);
    x.depth = 1;
    y.depth = 1;
    z.depth = 1;
    let rows = vec![
        row("root", None, true, true, false),
        x.clone(),
        y,
        z,
        row("w", Some("root"), false, false, false),
    ];
    assert_eq!(
        key_actions(TreeKey::Star, &rows, "x"),
        vec![TreeAction::ExpandAll(vec!["x".into()])]
    );
}

#[test]
fn type_ahead_finds_the_next_label_with_the_typed_prefix_and_wraps() {
    let rows = vec![
        row("Apple", None, false, false, false),
        row("Apricot", None, false, false, false),
        row("Banana", None, false, false, false),
    ];
    assert_eq!(type_ahead_target(&rows, "ap", 1), Some(1));
    assert_eq!(type_ahead_target(&rows, "b", 0), Some(2));
    assert_eq!(type_ahead_target(&rows, "a", 2), Some(0));
    assert_eq!(type_ahead_target(&rows, "zz", 0), None);
}

#[test]
fn a_repeated_letter_steps_through_rows_that_start_with_it() {
    let rows = vec![
        row("Apple", None, false, false, false),
        row("Apricot", None, false, false, false),
        row("Banana", None, false, false, false),
    ];
    assert_eq!(type_ahead_target(&rows, "aa", 0), Some(1));
    assert_eq!(type_ahead_target(&rows, "aaa", 1), Some(0));
}

#[test]
fn type_ahead_buffer_restarts_after_a_pause_longer_than_700ms() {
    let mut buffer = TypeAhead::default();
    assert_eq!(buffer.push('a', 0.0), "a");
    assert_eq!(buffer.push('p', 500.0), "ap");
    assert_eq!(buffer.push('b', 1300.0), "b");
}

#[test]
fn tree_view_renders_open_branches_and_hides_closed_ones() {
    let html = render(|| {
        let items = vec![
            branch("src", "src", vec![leaf("main", "main.rs")]),
            branch("docs", "docs", vec![leaf("readme", "README.md")]),
        ];
        rsx! { TreeView { items, default_expanded: vec!["src".to_string()], aria_label: "Files" } }
    });
    assert!(html.contains("role=\"tree\""));
    assert!(html.contains("aria-label=\"Files\""));
    assert!(html.contains("main.rs"));
    assert!(!html.contains("README.md"));
    assert!(html.contains("aria-expanded=\"true\""));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(html.contains("aria-level=\"2\""));
}

#[test]
fn tree_view_gives_one_tab_stop_to_the_first_row_and_shows_empty_text_in_an_empty_branch() {
    let html = render(|| {
        let items = vec![branch("empty", "empty", vec![]), leaf("other", "other")];
        rsx! { TreeView { items, default_expanded: vec!["empty".to_string()], empty_text: "Nothing here" } }
    });
    assert!(html.contains("tabindex=\"0\""));
    assert!(html.contains("tabindex=\"-1\""));
    assert!(html.contains("Nothing here"));
}

#[test]
fn tree_view_marks_the_selected_row() {
    let html = render(|| {
        let items = vec![leaf("one", "One"), leaf("two", "Two")];
        rsx! { TreeView { items, default_selected: "two".to_string() } }
    });
    assert!(html.contains("aria-selected=\"true\""));
    assert!(html.contains("aria-selected=\"false\""));
}

#[test]
fn resizable_handle_accepts_focus_blur_and_key_handlers_and_still_renders() {
    let html = render(|| {
        rsx! {
            rv2_affect::resizable::ResizablePanelGroup {
                rv2_affect::resizable::ResizableHandle {
                    with_handle: true,
                    onfocus: move |_| {},
                    onblur: move |_| {},
                    onkeydown: move |_| {},
                }
            }
        }
    });
    assert!(html.contains("role=\"separator\""));
    assert!(html.contains("class=\"ui-rszc__grip\""));
}
