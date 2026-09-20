//! The page, actually rendered.
//!
//! `cargo xtask ci` compiles this app and never opens it. AGENTS.md records
//! what that costs: the login screen shipped as a form that could not submit
//! anything — no `name` attributes, form-encoded against a JSON-only route —
//! while every test stayed green and the whole dashboard sat behind a working
//! redirect to it.
//!
//! So these tests render the component tree to a string on the host and assert
//! the controls are in the output. That is not a substitute for opening the
//! page in a browser, and it does not check that anything is clickable. It does
//! catch the class of defect where a control silently renders as nothing.

use dioxus::prelude::*;
use hiring::{Page, SharedRegister};
use rv2_hiring::view::Filters;

/// Which half of the page to render.
///
/// A plain enum rather than a function pointer in the props: props derive
/// `PartialEq`, and comparing function pointers is not meaningful — the same
/// function can have different addresses across codegen units, and different
/// functions can be merged to the same one.
#[derive(Clone, Copy, PartialEq)]
enum Part {
    Rail,
    Results,
}

/// Render part of the page with the register and filter state it would have.
///
/// The component under test is wrapped rather than called, because a Dioxus
/// component only has its hooks and context available inside a running
/// `VirtualDom`.
fn render_with(filters: Filters, part: Part) -> String {
    #[derive(Props, Clone, PartialEq)]
    struct HarnessProps {
        filters: Filters,
        part: Part,
    }

    #[allow(non_snake_case)]
    fn Harness(props: HarnessProps) -> Element {
        let _register: SharedRegister =
            use_context_provider(|| Signal::new(rv2_hiring::embedded_register()));
        let page = Page {
            filters: use_signal(|| props.filters.clone()),
            expanded: use_signal(Vec::new),
        };
        match props.part {
            Part::Rail => rsx! {
                hiring::rail::Rail { page }
            },
            Part::Results => rsx! {
                hiring::results::Results { page }
            },
        }
    }

    let mut dom = VirtualDom::new_with_props(Harness, HarnessProps { filters, part });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn the_rail_renders_both_essential_filters_and_every_litmus_axis() {
    let html = render_with(Filters::default(), Part::Rail);

    assert!(
        html.contains("Litmus Submission"),
        "the Litmus fieldset is missing from the rail"
    );
    for question in [
        "What level of role are you considering?",
        "Which markets are you positioning yourself in?",
        "What is the typical scale of the organisations you are targeting?",
        "Which of the following best describes your current situation?",
        "How soon are you looking to make a move?",
        "What is the main challenge you&#39;re facing?",
        "Which countries are you focused on?",
        "Which industries are relevant to your background?",
    ] {
        assert!(
            html.contains(question),
            "axis missing from the rail: {question}"
        );
    }

    assert!(html.contains("Remote, at most 25% in the office"));
    assert!(html.contains("B2B, outside IR35"));
    assert!(html.contains("Builds a department and teaches"));
}

#[test]
fn every_checkbox_in_the_rail_is_a_real_input_rather_than_a_styled_span() {
    let html = render_with(Filters::default(), Part::Rail);
    let inputs = html.matches("type=\"checkbox\"").count();
    assert!(
        inputs > 25,
        "expected the rail's controls to render as checkbox inputs, found {inputs}"
    );
}

#[test]
fn a_derived_axis_states_what_it_was_derived_from() {
    let html = render_with(Filters::default(), Part::Rail);
    assert!(
        html.contains("not from revenue"),
        "the organisation-scale axis must say it came from headcount"
    );
}

#[test]
fn the_results_list_renders_entries_with_no_filters_on() {
    let html = render_with(Filters::default(), Part::Results);
    assert!(
        html.contains("Thunes"),
        "the register's entries are missing"
    );
    assert!(html.contains("shown"), "the toolbar count is missing");
}

#[test]
fn both_essential_filters_leave_the_outside_ir35_board_and_its_six_roles() {
    let filters = Filters {
        remote_only: true,
        b2b_only: true,
        ..Filters::default()
    };
    let html = render_with(filters, Part::Results);
    assert!(html.contains("Outside IR35"));
    assert!(
        !html.contains("Thunes"),
        "a permanent London role survived both filters"
    );
}

#[test]
fn the_empty_state_names_the_filter_that_emptied_the_list() {
    let filters = Filters {
        remote_only: true,
        b2b_only: true,
        mandate_only: true,
        ..Filters::default()
    };
    let html = render_with(filters, Part::Results);

    assert!(
        html.contains("remote, B2B and department-building at once"),
        "the empty state must name what emptied the list"
    );
    assert!(
        !html.contains("Try again") && !html.contains("Retry"),
        "an authoritative empty answer must not offer a retry"
    );
}
