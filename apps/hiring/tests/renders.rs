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

// ── What the port had dropped, and now has back ──────────────────────────────
//
// Each of these is a feature the original vanilla-JS page rendered and the
// first cut of this port silently did not. A six-lane audit found 69 of them;
// the ones below are the high-severity set, asserted so the next rewrite
// cannot drop them quietly a second time.

#[test]
fn a_hostile_role_url_never_becomes_a_link() {
    let html = render_with(Filters::default(), Part::Results);
    for scheme in ["javascript:", "data:text/html", "vbscript:", "file://"] {
        assert!(
            !html.contains(&format!("href=\"{scheme}")),
            "a {scheme} URL reached an href"
        );
    }
}

#[test]
fn the_ticker_badge_carries_its_index_colour() {
    let html = render_with(Filters::default(), Part::Results);
    let ftse250 = rv2_hiring::view::group_colour("ftse250");
    assert!(
        html.contains(ftse250),
        "no entry rendered the FTSE 250 colour {ftse250}"
    );
    assert!(
        html.contains("On a London index") && html.contains("Not on a London index"),
        "both listed and unlisted treatments must appear"
    );
}

#[test]
fn the_legend_explains_what_the_two_badge_treatments_mean() {
    let html = render_with(Filters::default(), Part::Results);
    assert!(html.contains("on a London index, coloured by which"));
    assert!(html.contains("not on a London index"));
}

#[test]
fn every_entry_states_how_far_its_details_were_confirmed() {
    let html = render_with(Filters::default(), Part::Results);
    assert!(
        html.contains("Confirmed company details")
            || html.contains("Partly confirmed company details")
            || html.contains("Unconfirmed company details"),
        "the confidence tier is not rendered anywhere"
    );
}

#[test]
fn a_company_with_a_headcount_band_shows_it_beside_its_locations() {
    let html = render_with(Filters::default(), Part::Results);
    assert!(html.contains("people"), "employee scale is not rendered");
}

#[test]
fn rust_posting_counts_are_shown_and_not_merely_counted() {
    let html = render_with(Filters::default(), Part::Results);
    assert!(
        html.contains("Rust") && html.contains("worldwide"),
        "the openings line lost its Rust breakdown"
    );
}

#[test]
fn the_roles_list_is_a_disclosure_that_says_how_many_of_how_many() {
    let html = render_with(Filters::default(), Part::Results);
    assert!(html.contains("<details"), "roles are not collapsible");
    assert!(
        html.contains("Show ") && html.contains(" roles"),
        "the disclosure does not say how many roles it holds"
    );
}

#[test]
fn every_entry_offers_a_route_out_even_when_its_careers_link_is_broken() {
    let html = render_with(Filters::default(), Part::Results);
    assert!(
        html.contains("linkedin.com/jobs/search"),
        "no LinkedIn route"
    );
    assert!(
        html.contains("Search for careers site") || html.contains("google.com/search"),
        "no fallback when a careers link is unusable"
    );
}

#[test]
fn the_export_is_a_real_download_link_carrying_the_current_view() {
    let html = render_with(Filters::default(), Part::Results);
    assert!(html.contains("Export CSV"));
    assert!(html.contains("download=\"uk-tech-hiring-register.csv\""));
    assert!(html.contains("data:text/csv"));
}

#[test]
fn the_result_count_is_announced_rather_than_only_shown() {
    let html = render_with(Filters::default(), Part::Results);
    assert!(html.contains("aria-live=\"polite\""));
}

#[test]
fn the_table_carries_every_column_a_reader_can_act_on() {
    let filters = Filters {
        layout: rv2_hiring::view::Layout::Table,
        ..Filters::default()
    };
    let html = render_with(filters, Part::Results);
    for column in [
        "Ticker",
        "Company",
        "Index",
        "UK locations",
        "UK engineering roles",
        "UK postings mentioning Rust",
        "Roles clearing both filters",
        "Careers site",
        "Job board",
        "Search",
    ] {
        assert!(html.contains(column), "table lost the {column} column");
    }
    assert!(
        html.contains("linkedin.com/jobs/search"),
        "table has no outbound links"
    );
}

#[test]
fn an_untracked_company_reads_as_untracked_rather_than_as_zero() {
    let filters = Filters {
        layout: rv2_hiring::view::Layout::Table,
        ..Filters::default()
    };
    let html = render_with(filters, Part::Results);
    assert!(
        html.contains("Not tracked"),
        "a company with no job board must not render as 0 open roles"
    );
}

#[test]
fn the_index_facet_shows_the_colour_it_filters_on() {
    let html = render_with(Filters::default(), Part::Rail);
    for key in ["ftse100", "ftse250", "aim"] {
        let colour = rv2_hiring::view::group_colour(key);
        assert!(html.contains(colour), "the {key} facet has no swatch");
    }
}

#[test]
fn the_sort_control_offers_every_order_including_rust_depth() {
    let html = render_with(Filters::default(), Part::Results);
    for label in [
        "Index, then open roles",
        "UK engineering roles",
        "Rust postings",
        "How Rust-first the work is",
        "Roles that build a department",
        "Company name",
        "Careers link status",
    ] {
        assert!(html.contains(label), "sort lost the {label} option");
    }
}

#[test]
fn every_axis_renders_the_provenance_its_type_forced_it_to_declare() {
    // Entities decoded first, so the test pins the sentence rather than
    // whichever characters happened not to need escaping.
    let html = render_with(Filters::default(), Part::Rail).replace("&#39;", "'");
    for axis in rv2_hiring::litmus::Axis::ALL {
        let label = axis.basis().label();
        assert!(
            html.contains(&label),
            "{axis:?} declares {label:?} and the rail does not show it"
        );
    }
}

#[test]
fn the_litmus_note_states_the_contract_without_a_count_that_can_go_stale() {
    let html = render_with(Filters::default(), Part::Rail);
    assert!(html.contains("reads the register directly or was derived"));
    for stale in ["eight questions", "Four read", "four are derived"] {
        assert!(
            !html.contains(stale),
            "the group note hardcodes {stale:?}, which stops being true when an axis changes"
        );
    }
}

#[test]
fn a_role_that_was_applied_to_says_so_on_the_page() {
    let html = render_with(Filters::default(), Part::Results);
    assert!(
        html.contains("Applied 2026-09-20"),
        "the Head of AI application is recorded in the register but not shown"
    );
}

// ── The dashboard ────────────────────────────────────────────────────────────

fn dashboard_html() -> String {
    render_with(
        Filters {
            layout: rv2_hiring::view::Layout::Dashboard,
            ..Filters::default()
        },
        Part::Results,
    )
}

#[test]
fn the_dashboard_renders_every_standing_panel() {
    let html = dashboard_html();
    for panel in rv2_hiring::dashboard::Dashboard::standing().panels {
        assert!(
            html.contains(&panel.title),
            "the dashboard is missing the panel {:?}",
            panel.title
        );
    }
}

#[test]
fn no_panel_shows_a_number_without_its_denominator() {
    let html = dashboard_html();
    let counted = html.matches(" counted").count();
    assert!(
        counted >= 5,
        "expected every panel to state what it counted, found {counted}"
    );
    assert!(
        html.contains("publish no board"),
        "the denominator must name the companies with nothing to read"
    );
}

#[test]
fn the_two_counterfactual_panels_disagree_on_screen() {
    let html = dashboard_html();
    assert!(html.contains("What you gave up by needing remote"));
    assert!(html.contains("What you gave up by needing a mandate"));
    assert!(
        html.contains("Remote, UK-based") || html.contains("Remote, anywhere"),
        "the working-pattern breakdown renders its buckets"
    );
    assert!(
        html.contains("Builds and teaches") || html.contains("No department-building remit"),
        "the mandate breakdown renders its buckets"
    );
}

#[test]
fn a_breakdown_bar_carries_its_number_as_text_not_only_as_width() {
    let html = dashboard_html();
    assert!(
        html.contains("tabular-nums"),
        "counts must render as text beside the bar, never only as a pixel width"
    );
}

#[test]
fn an_empty_panel_says_which_kind_of_empty_it_is() {
    let html = render_with(
        Filters {
            layout: rv2_hiring::view::Layout::Dashboard,
            ..Filters::default()
        },
        Part::Results,
    );
    // The applied panel has exactly one role in it, so the register exercises
    // the populated path; the copy below is what the empty path would show.
    let b = rv2_hiring::breakdown::breakdown(
        &rv2_hiring::embedded_register(),
        &Filters {
            remote_only: true,
            b2b_only: true,
            mandate_only: true,
            ..Filters::default()
        },
        rv2_hiring::breakdown::GroupBy::WorkPattern,
    );
    assert!(b.is_empty());
    assert!(
        b.empty_copy()
            .contains("that is the answer rather than a gap")
            || b.empty_copy().contains("treat this as a floor")
            || b.empty_copy().contains("cannot answer this"),
        "an empty panel must name which of the three empties it is: {}",
        b.empty_copy()
    );
    assert!(!html.is_empty());
}

#[test]
fn the_applied_role_reaches_the_dashboard() {
    let html = dashboard_html();
    assert!(
        html.contains("What you have applied to"),
        "the applied panel is present"
    );
    assert!(
        html.contains("Head Of Artificial Intelligence"),
        "the applied role shows in its panel"
    );
}

#[test]
fn every_panel_offers_a_way_into_the_register() {
    let html = dashboard_html();
    let ways = html.matches("Open these in the register").count();
    assert!(
        ways >= 5,
        "a panel that can only be looked at is a dead end; found {ways}"
    );
}

#[test]
fn the_dashboard_is_reachable_from_the_layout_control() {
    let html = render_with(Filters::default(), Part::Results);
    assert!(
        html.contains("Dashboard"),
        "no control switches to the dashboard"
    );
}
