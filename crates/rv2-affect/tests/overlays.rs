use dioxus::prelude::*;
use rv2_affect::context_menu::{ContextMenu, is_menu_key, keyboard_origin};
use rv2_affect::dialog::{AlertDialog, Dialog, DialogSize, tab_wrap};
use rv2_affect::dropdown_menu::{
    DropdownMenu, DropdownTrigger, MenuItem, RovingStep, menu_typeahead, roving_index,
};
use rv2_affect::hover_card::{HoverCard, Point, corridor, hull, inside_hull};
use rv2_affect::popover::{
    Align, Placement, Popover, Rect, Side, Size, flip_side, place, placed_style,
};
use rv2_affect::select::{Select, SelectOption, list_placement, select_step, typeahead_index};
use rv2_affect::tooltip::{Tooltip, has_label};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn view() -> Size {
    Size {
        width: 1000.0,
        height: 800.0,
    }
}

fn anchor(left: f64, top: f64) -> Rect {
    Rect {
        left,
        top,
        width: 80.0,
        height: 32.0,
    }
}

fn menu_items() -> Vec<MenuItem> {
    let mut copy = MenuItem::new("Copy");
    copy.shortcut = Some("⌘C".to_string());
    let mut wrap = MenuItem::new("Wrap lines");
    wrap.checked = Some(true);
    wrap.checkbox = true;
    let mut delete = MenuItem::new("Delete");
    delete.danger = true;
    delete.divider = true;
    let mut share = MenuItem::new("Share");
    share.submenu = Some(vec![MenuItem::new("Email")]);
    vec![copy, wrap, delete, share]
}

fn select_options() -> Vec<SelectOption> {
    vec![
        SelectOption {
            value: "a".into(),
            label: "Alpha".into(),
            description: None,
            icon: None,
            disabled: false,
        },
        SelectOption {
            value: "b".into(),
            label: "Beta".into(),
            description: None,
            icon: None,
            disabled: true,
        },
    ]
}

#[test]
fn popover_open_renders_a_dialog_card_named_by_its_title() {
    let html = render(|| {
        rsx! {
            Popover { open: true, onclose: move |_| {}, title: rsx! { "Filters" }, description: rsx! { "Narrow the list" }, p { "body" } }
        }
    });
    assert!(html.contains("class=\"ui-popc\""));
    assert!(html.contains("role=\"dialog\""));
    assert!(html.contains("data-side=\"bottom\""));
    assert!(html.contains("ui-popc__title"));
    assert!(html.contains("-title\""));
    assert!(html.contains("visibility:hidden;"));
}

#[test]
fn popover_closed_renders_nothing() {
    let html = render(|| rsx! { Popover { open: false, onclose: move |_| {}, p { "body" } } });
    assert!(!html.contains("ui-popc"));
}

#[test]
fn tooltip_open_renders_a_tooltip_with_its_shortcut_on_the_chosen_side() {
    let html = render(|| {
        rsx! { Tooltip { content: "Save", shortcut: "⌘S", open: true, button { "Save" } } }
    });
    assert!(html.contains("role=\"tooltip\""), "{html}");
    assert!(html.contains("class=\"ui-tipc\""), "{html}");
    assert!(html.contains("data-side=\"top\""), "{html}");
    assert!(html.contains("ui-tipc__keys"));
    assert!(html.contains("ui-tip-anchor-"));
}

#[test]
fn tooltip_with_empty_label_renders_only_its_child() {
    let html = render(|| rsx! { Tooltip { content: "", open: true, button { "x" } } });
    assert!(!html.contains("role=\"tooltip\""));
    assert!(html.contains("<button>x</button>"));
}

#[test]
fn hover_card_open_renders_its_content_on_the_bottom_side_by_default() {
    let html =
        render(|| rsx! { HoverCard { content: rsx! { "Ada Lovelace" }, open: true, a { "Ada" } } });
    assert!(html.contains("class=\"ui-hvcc\""));
    assert!(html.contains("data-side=\"bottom\""));
    assert!(html.contains("Ada Lovelace"));
}

#[test]
fn hover_card_without_content_never_opens() {
    let html = render(|| rsx! { HoverCard { open: true, a { "Ada" } } });
    assert!(!html.contains("ui-hvcc\""));
}

#[test]
fn select_closed_names_its_placeholder_and_its_chosen_value_with_a_hidden_field() {
    let html = render(|| {
        rsx! { Select { options: select_options(), default_value: "a", name: "fruit" } }
    });
    assert!(html.contains("role=\"combobox\""));
    assert!(html.contains("aria-haspopup=\"listbox\""));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(html.contains("Alpha"));
    assert!(html.contains("name=\"fruit\""));
    assert!(html.contains("value=\"a\""));
}

#[test]
fn select_without_a_choice_shows_the_placeholder() {
    let html = render(|| rsx! { Select { options: select_options(), placeholder: "Pick one" } });
    assert!(html.contains("Pick one"));
    assert!(html.contains("data-empty"));
}

#[test]
fn dropdown_open_renders_roles_checks_danger_dividers_and_submenu_hints() {
    let html = render(|| {
        rsx! {
            DropdownMenu {
                trigger: Callback::new(|t: DropdownTrigger| rsx! { button { id: "{t.id}", "Open" } }),
                items: menu_items(),
                open: true,
            }
        }
    });
    assert!(html.contains("role=\"menu\""));
    assert!(html.contains("role=\"menuitem\""));
    assert!(html.contains("role=\"menuitemcheckbox\""));
    assert!(html.contains("aria-checked=\"true\""));
    assert!(html.contains("data-danger=\"true\""));
    assert!(html.contains("ui-dmc__sep"));
    assert!(html.contains("ui-dmc__kbd"));
    assert!(html.contains("aria-haspopup=\"menu\""));
    assert!(html.contains("ui-dmc-"));
}

#[test]
fn dropdown_trigger_carries_the_expanded_state_it_is_given() {
    let html = render(|| {
        rsx! {
            DropdownMenu {
                trigger: Callback::new(|t: DropdownTrigger| rsx! { button { id: "{t.id}", "aria-expanded": "{t.expanded}", "Open" } }),
                items: menu_items(),
            }
        }
    });
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(!html.contains("role=\"menu\""));
}

#[test]
fn context_menu_wraps_its_area_and_draws_no_menu_until_opened() {
    let html = render(|| rsx! { ContextMenu { items: menu_items(), p { "row" } } });
    assert!(html.contains("ui-ctxc-"));
    assert!(html.contains("display: contents"));
    assert!(html.contains("row"));
    assert!(!html.contains("ui-ctxc__panel"));
}

#[test]
fn dialog_closed_by_default_draws_nothing() {
    let html = render(|| rsx! { Dialog { title: rsx! { "Settings" }, p { "body" } } });
    assert!(!html.contains("<dialog"));
}

#[test]
fn dialog_open_is_a_named_modal_dialog_at_its_default_size() {
    let html = render(|| {
        rsx! { Dialog { default_open: true, title: rsx! { "Settings" }, description: rsx! { "Local" }, p { "body" } } }
    });
    assert!(html.contains("<dialog"));
    assert!(html.contains("class=\"ui-dlgc\""));
    assert!(html.contains("role=\"dialog\""));
    assert!(html.contains("data-size=\"md\""));
    assert!(html.contains("aria-labelledby"));
    assert!(html.contains("ui-dlgc__desc"));
    assert!(html.contains("ui-dlgc__close"));
    assert!(!html.contains("data-contained"));
}

#[test]
fn contained_large_dialog_is_marked_contained_and_sized_lg() {
    let html = render(|| {
        rsx! { Dialog { default_open: true, contained: true, size: DialogSize::Lg, title: rsx! { "Preview" }, p { "body" } } }
    });
    assert!(html.contains("data-contained=\"true\""));
    assert!(html.contains("aria-modal=\"true\""));
    assert!(html.contains("data-size=\"lg\""));
}

#[test]
fn alert_dialog_is_an_alertdialog_whose_confirm_button_is_red_when_dangerous() {
    let html = render(|| {
        rsx! { AlertDialog { default_open: true, title: rsx! { "Delete agent?" }, danger: true, confirm_label: "Delete agent", p { "gone" } } }
    });
    assert!(html.contains("role=\"alertdialog\""));
    assert!(html.contains("data-tone=\"danger\""));
    assert!(html.contains("Delete agent"));
    assert!(html.contains("data-alert=\"true\""));
    assert!(html.contains("data-size=\"sm\""));
}

#[test]
fn alert_dialog_defaults_to_a_primary_confirm_and_a_cancel_button() {
    let html =
        render(|| rsx! { AlertDialog { default_open: true, title: rsx! { "Sure?" }, p { "x" } } });
    assert!(html.contains("data-tone=\"primary\""));
    assert!(html.contains("Cancel"));
    assert!(html.contains("Confirm"));
}

#[test]
fn flip_keeps_the_preferred_side_when_it_has_room() {
    let trigger = anchor(400.0, 300.0);
    let card = Size {
        width: 200.0,
        height: 120.0,
    };
    let room = rv2_affect::popover::room(trigger, view(), 6.0);
    assert_eq!(flip_side(Side::Bottom, room, card), Side::Bottom);
}

#[test]
fn flip_turns_to_the_opposite_side_when_the_preferred_one_is_too_small() {
    let trigger = anchor(400.0, 700.0);
    let card = Size {
        width: 200.0,
        height: 200.0,
    };
    let room = rv2_affect::popover::room(trigger, view(), 6.0);
    assert_eq!(flip_side(Side::Bottom, room, card), Side::Top);
}

#[test]
fn flip_keeps_the_side_with_more_room_when_both_sides_are_too_small() {
    let trigger = Rect {
        left: 100.0,
        top: 100.0,
        width: 40.0,
        height: 40.0,
    };
    let card = Size {
        width: 200.0,
        height: 2000.0,
    };
    let tiny = Size {
        width: 50.0,
        height: 50.0,
    };
    let room = rv2_affect::popover::room(trigger, tiny, 0.0);
    assert_eq!(flip_side(Side::Bottom, room, card), Side::Top);
}

#[test]
fn flip_stays_put_when_the_opposite_side_has_no_more_room() {
    let card = Size {
        width: 200.0,
        height: 2000.0,
    };
    let tall = rv2_affect::popover::Room {
        top: 10.0,
        bottom: 10.0,
        left: 0.0,
        right: 0.0,
    };
    assert_eq!(flip_side(Side::Bottom, tall, card), Side::Bottom);
}

#[test]
fn place_centres_a_card_under_its_trigger() {
    let p = place(
        anchor(400.0, 100.0),
        Size {
            width: 200.0,
            height: 100.0,
        },
        view(),
        Side::Bottom,
        Align::Center,
        6.0,
        false,
    );
    assert_eq!(p.side, Side::Bottom);
    assert_eq!(p.top, 138.0);
    assert_eq!(p.left, 340.0);
    assert_eq!(p.max_height, None);
}

#[test]
fn place_keeps_a_card_inside_the_window_horizontally() {
    let p = place(
        anchor(990.0, 100.0),
        Size {
            width: 300.0,
            height: 100.0,
        },
        view(),
        Side::Bottom,
        Align::Center,
        6.0,
        false,
    );
    assert!(p.left + 300.0 <= 1000.0 - 8.0 + f64::EPSILON);
    assert!(p.left >= 8.0);
}

#[test]
fn place_sets_a_max_height_when_the_card_is_too_tall_for_its_room() {
    let p = place(
        anchor(400.0, 700.0),
        Size {
            width: 200.0,
            height: 900.0,
        },
        view(),
        Side::Bottom,
        Align::Start,
        6.0,
        false,
    );
    assert!(p.max_height.is_some());
}

#[test]
fn place_moves_a_side_card_under_the_trigger_on_a_narrow_screen() {
    let phone = Size {
        width: 360.0,
        height: 700.0,
    };
    let trigger = Rect {
        left: 140.0,
        top: 300.0,
        width: 80.0,
        height: 32.0,
    };
    let p = place(
        trigger,
        Size {
            width: 300.0,
            height: 100.0,
        },
        phone,
        Side::Right,
        Align::Start,
        6.0,
        false,
    );
    assert!(p.side.is_vertical());
}

#[test]
fn place_swaps_start_and_end_above_and_below_in_rtl() {
    let p = place(
        anchor(400.0, 100.0),
        Size {
            width: 200.0,
            height: 100.0,
        },
        view(),
        Side::Bottom,
        Align::Start,
        6.0,
        true,
    );
    assert_eq!(p.align, Align::End);
}

#[test]
fn placed_style_always_states_its_visibility_so_a_stale_hidden_cannot_survive_a_merge() {
    let placed = Placement {
        side: Side::Top,
        align: Align::Center,
        top: 1.0,
        left: 1.0,
        max_height: None,
    };
    assert!(placed_style(Some(placed), None).contains("visibility:visible;"));
    assert!(!placed_style(Some(placed), None).contains("hidden"));
    assert!(placed_style(None, None).contains("visibility:hidden;"));
    assert!(placed_style(Some(placed), None).contains("max-height:none;"));
}

#[test]
fn placed_style_hides_the_card_until_it_is_placed() {
    assert_eq!(placed_style(None, None), "visibility:hidden;");
    let p = Placement {
        side: Side::Bottom,
        align: Align::Start,
        top: 12.4,
        left: 3.0,
        max_height: Some(200.0),
    };
    let style = placed_style(Some(p), Some("280px"));
    assert!(style.contains("top:12px;left:3px;"));
    assert!(style.contains("max-height:200px;"));
    assert!(style.contains("width:280px;"));
}

#[test]
fn roving_index_wraps_and_starts_from_either_end_when_nothing_is_focused() {
    assert_eq!(roving_index(3, None, RovingStep::Next), Some(0));
    assert_eq!(roving_index(3, None, RovingStep::Previous), Some(2));
    assert_eq!(roving_index(3, Some(2), RovingStep::Next), Some(0));
    assert_eq!(roving_index(3, Some(0), RovingStep::Previous), Some(2));
    assert_eq!(roving_index(3, Some(1), RovingStep::Last), Some(2));
    assert_eq!(roving_index(0, None, RovingStep::First), None);
}

#[test]
fn menu_typeahead_jumps_to_the_next_row_that_starts_with_a_single_letter() {
    let labels = ["Apple", "Avocado", "Banana"];
    assert_eq!(menu_typeahead(&labels, "a", Some(0)), Some(1));
    assert_eq!(menu_typeahead(&labels, "a", Some(1)), Some(0));
    assert_eq!(menu_typeahead(&labels, "b", None), Some(2));
}

#[test]
fn menu_typeahead_matches_a_longer_query_from_the_current_row_case_insensitively() {
    let labels = ["Apple", "Avocado", "Banana"];
    assert_eq!(menu_typeahead(&labels, "AV", Some(0)), Some(1));
    assert_eq!(menu_typeahead(&labels, "cherry", Some(0)), None);
}

#[test]
fn select_step_skips_disabled_options_and_stays_put_at_the_end() {
    let enabled = [true, false, true];
    assert_eq!(select_step(0, &enabled, 1), 2);
    assert_eq!(select_step(2, &enabled, 1), 2);
    assert_eq!(select_step(2, &enabled, -1), 0);
}

#[test]
fn select_typeahead_skips_disabled_options() {
    let labels = [("Alpha", true), ("Almond", false), ("Alto", true)];
    assert_eq!(typeahead_index(&labels, "al", -1), Some(2));
    assert_eq!(typeahead_index(&labels, "a", 0), Some(2));
    assert_eq!(typeahead_index(&labels, "z", 0), None);
}

#[test]
fn select_list_opens_below_when_there_is_room_and_above_when_the_room_below_is_short() {
    let low = list_placement(anchor(400.0, 100.0), 288.0, view());
    assert!(!low.up);
    assert_eq!(low.top, 136.0);
    let high = list_placement(anchor(400.0, 700.0), 288.0, view());
    assert!(high.up);
    assert_eq!(high.view_height, 800.0);
}

#[test]
fn select_list_is_at_least_180_px_wide() {
    let narrow = list_placement(
        Rect {
            left: 10.0,
            top: 100.0,
            width: 60.0,
            height: 32.0,
        },
        288.0,
        view(),
    );
    assert_eq!(narrow.width, 180.0);
}

#[test]
fn hull_of_a_square_with_an_inner_point_is_the_square() {
    let points = [
        Point { x: 0.0, y: 0.0 },
        Point { x: 10.0, y: 0.0 },
        Point { x: 10.0, y: 10.0 },
        Point { x: 0.0, y: 10.0 },
        Point { x: 5.0, y: 5.0 },
    ];
    assert_eq!(hull(&points).len(), 4);
}

#[test]
fn corridor_keeps_the_path_from_the_trigger_to_the_card_and_drops_the_rest() {
    let exit = Point { x: 50.0, y: 50.0 };
    let card = Rect {
        left: 40.0,
        top: 100.0,
        width: 100.0,
        height: 60.0,
    };
    let poly = corridor(exit, card);
    assert!(inside_hull(&poly, 50.0, 80.0));
    assert!(inside_hull(&poly, 90.0, 130.0));
    assert!(!inside_hull(&poly, 300.0, 50.0));
    assert!(!inside_hull(&poly, 50.0, 10.0));
}

#[test]
fn tab_wrap_loops_inside_a_contained_layer_and_leaves_the_middle_to_the_browser() {
    assert_eq!(tab_wrap(3, Some(2), false), Some(0));
    assert_eq!(tab_wrap(3, Some(0), true), Some(2));
    assert_eq!(tab_wrap(3, Some(1), false), None);
    assert_eq!(tab_wrap(3, None, false), Some(0));
    assert_eq!(tab_wrap(3, None, true), Some(2));
    assert_eq!(tab_wrap(0, None, false), None);
}

#[test]
fn keyboard_origin_sits_in_from_the_start_edge_of_the_focused_element() {
    let rect = Rect {
        left: 100.0,
        top: 200.0,
        width: 120.0,
        height: 40.0,
    };
    assert_eq!(keyboard_origin(rect, false), (112.0, 226.0));
    assert_eq!(keyboard_origin(rect, true), (208.0, 226.0));
}

#[test]
fn shift_f10_and_the_menu_key_open_a_context_menu_and_plain_f10_does_not() {
    assert!(is_menu_key(&Key::ContextMenu, false));
    assert!(is_menu_key(&Key::F10, true));
    assert!(!is_menu_key(&Key::F10, false));
}

#[test]
fn an_empty_tooltip_label_has_nothing_to_show() {
    assert!(!has_label(""));
    assert!(has_label("Save"));
}
