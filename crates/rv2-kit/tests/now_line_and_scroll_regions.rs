use dioxus::prelude::*;
use rv2_kit::components::session_packing_planner::{PackMode, SessionPackingPlanner, Slot};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn css_block<'a>(css: &'a str, selector: &str) -> &'a str {
    css.split(selector)
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .unwrap_or_else(|| panic!("no rule for {selector}"))
}

#[test]
fn now_line_sits_inside_the_session_in_progress_and_nowhere_else() {
    fn app() -> Element {
        rsx! {
            SessionPackingPlanner {
                task_queue: Vec::new(),
                day_headers: vec!["Mon".into(), "Tue".into()],
                row_headers: vec!["07:00".into(), "12:00".into()],
                current_slot: Some(Slot { day_index: 0, session_index: 1 }),
                now_fraction: 0.35,
                session_budget: 1.0,
                smallest_fragment: 0.2,
                initial_mode: PackMode::Auto,
                auto_delay: 0,
            }
        }
    }
    let html = render(app);
    assert_eq!(html.matches("kit-pack__nowline").count(), 1);
    let current = html.find("data-current=\"true\"").expect("current cell");
    let line = html.find("kit-pack__nowline").expect("now line");
    let next_cell = html[current..]
        .find("role=\"gridcell\"")
        .map(|offset| current + offset);
    assert!(line > current);
    assert!(next_cell.is_none_or(|next| line < next));
}

#[test]
fn decision_queue_body_fills_the_card_so_each_region_scrolls_inside_it() {
    let css = include_str!("../assets/decision-review-queue.css");
    assert!(css_block(css, ".kit-drq__body {").contains("grid-template-rows: minmax(0, 1fr)"));
    let queue = css_block(css, ".kit-drq__queue {");
    assert!(queue.contains("min-height: 0"));
    assert!(queue.contains("overflow-y: auto"));
    let content = css_block(css, "\n.kit-drq__content {");
    assert!(content.contains("overflow-y: auto"));
    assert!(content.contains("calc(var(--kit-space-6) + var(--kit-space-1))"));
}

#[test]
fn multi_select_list_is_a_grid_of_whole_rows_that_snaps_between_them() {
    let css = include_str!("../assets/multi-select-lookup.css");
    let list = css_block(css, ".kit-msl__list {");
    assert!(list.contains("grid-auto-rows: var(--kit-msl-unit)"));
    assert!(list.contains("max-height: calc(var(--kit-msl-unit) * 7 + 2px)"));
    assert!(list.contains("scroll-snap-type: y mandatory"));
    assert!(css.contains(".kit-msl__list > * { scroll-snap-align: start; }"));
}
