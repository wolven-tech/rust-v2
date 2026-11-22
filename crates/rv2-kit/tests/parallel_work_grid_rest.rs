//! SSR check for the parallel work grid at rest: no details popover is drawn before a cell is engaged.

use dioxus::prelude::*;
use rv2_kit::components::parallel_work_grid::{Item, ItemState, ParallelWorkGrid};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn grid_app() -> Element {
    let items: Vec<Item> = (0..16)
        .map(|n| Item {
            item_id: format!("chunk-{n}"),
            item_label: format!("Chunk {}", n + 1),
            item_state: ItemState::Done,
            worker_name: Some("Worker".into()),
            start_offset: Some(0.0),
            end_offset: Some(4.0),
            item_cost: Some(0.03),
            attempt_count: 1,
            output_value: Some("ok".into()),
            error_text: None,
        })
        .collect();
    rsx! {
        ParallelWorkGrid { job_items: items, job_name: "Translate 16 chunks", concurrency: Some(3) }
    }
}

#[test]
fn parallel_work_grid_at_rest_draws_cells_and_no_details_popover() {
    let html = render(grid_app);
    assert_eq!(html.matches("role=\"gridcell\"").count(), 16);
    assert!(!html.contains("kit-pwg__details"));
}
