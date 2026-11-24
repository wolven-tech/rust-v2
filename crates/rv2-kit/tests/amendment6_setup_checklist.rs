//! Amendment 6 of the workspace setup checklist: at-rest structure and the step and tool rules.

use dioxus::prelude::*;
use rv2_kit::screens::workspace_setup_checklist::{
    ConnectableTool, SetupValues, WorkspaceSetupChecklist, progress_line, step_hint, step_status,
    step_word, tool_mark,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

fn tool(id: &str, name: &str, connected: bool) -> ConnectableTool {
    ConnectableTool {
        id: id.into(),
        name: name.into(),
        summary: format!("{name} summary."),
        connected,
    }
}

fn rest_app() -> Element {
    rsx! {
        WorkspaceSetupChecklist {
            workspace_name: "Harbor lane",
            account_first_name: "Ana",
            default_step: 2usize,
            connected_tools: vec![
                tool("drive", "Shared drive", true),
                tool("crm", "Customer records", false),
                tool("calendar", "Calendar", false),
                tool("mail", "Mail", false),
                tool("forms", "Forms", false),
                tool("invoices", "Invoices", false),
            ],
            crumbs: vec!["Workspace home".to_string(), "Set up Harbor lane".to_string()],
            on_complete: move |_: SetupValues| {},
        }
    }
}

#[test]
fn at_rest_the_tool_column_lists_six_tools_and_only_the_first_is_connected() {
    let html = render(rest_app);
    assert_eq!(count(&html, "kit-wsc__tool\""), 6);
    assert_eq!(count(&html, "kit-wsc__badge"), 1);
}

#[test]
fn at_rest_the_help_box_holds_a_book_a_setup_call_link() {
    let html = render(rest_app);
    assert!(html.contains("Book a setup call"));
}

#[test]
fn at_rest_the_breadcrumb_names_the_workspace_home_and_the_flow() {
    let html = render(rest_app);
    assert!(html.contains("aria-label=\"Breadcrumb\""));
    assert!(html.contains("Set up Harbor lane"));
}

#[test]
fn progress_line_reads_one_of_four_done_at_rest() {
    assert_eq!(progress_line(1), "1 of 4 done");
}

#[test]
fn finished_step_one_shows_its_entered_value_and_step_two_reads_in_progress() {
    assert_eq!(step_status(1, "done", "Harbor lane"), "Harbor lane");
    assert_eq!(step_word("done"), "Done");
    assert_eq!(step_word("current"), "In progress");
    assert_eq!(step_word("upcoming"), "Not started");
}

#[test]
fn unfinished_steps_show_a_hint_not_an_empty_line() {
    assert_eq!(
        step_status(2, "current", "Harbor lane"),
        "Pick the tools to connect"
    );
    assert_eq!(step_status(3, "upcoming", ""), "Add teammates by email");
    assert_eq!(step_status(4, "upcoming", ""), step_hint(4));
}

#[test]
fn tool_rows_are_marked_with_their_first_letter() {
    assert_eq!(tool_mark("Shared drive"), "S");
    assert_eq!(tool_mark("calendar"), "C");
    assert_eq!(tool_mark(""), "");
}
