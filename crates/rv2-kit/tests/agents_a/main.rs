//! Lane agents-a: SSR and pure-logic tests for the agents group.
//! One child module per component, under `tests/agents_a/`.

mod background_task_inbox;
mod baton_pass_log;
mod co_editing_presence;
mod delegation_card_tree;
mod editable_step_plan;
mod exchange_diagram;
mod mid_run_note_panel;
mod overnight_run_digest;
mod parallel_probe_feed;
mod parallel_span_timeline;
mod request_routing_map;

/// Renders a component to HTML without a browser.
pub fn render(app: fn() -> dioxus::prelude::Element) -> String {
    let mut dom = dioxus::prelude::VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}
