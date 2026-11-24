use dioxus::prelude::*;
use rv2_kit::screens::issue_board_screen::{
    BoardColumn, Issue, IssueBoardScreen, IssueStatus, PriorityTone, column_cards,
};

const CSS: &str = include_str!("../assets/issue-board-screen.css");

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// The CSS block for one selector, from its opening brace to the closing brace.
fn rule<'a>(css: &'a str, selector: &str) -> &'a str {
    let start = css
        .find(&format!("{selector} {{"))
        .unwrap_or_else(|| panic!("no rule for {selector}"));
    let end = css[start..]
        .find('}')
        .map(|i| start + i)
        .unwrap_or(css.len());
    &css[start..end]
}

fn columns() -> Vec<BoardColumn> {
    [
        ("low", "Low", PriorityTone::Low),
        ("medium", "Medium", PriorityTone::Medium),
        ("high", "High", PriorityTone::High),
        ("done", "Done", PriorityTone::Done),
    ]
    .into_iter()
    .map(|(id, label, tone)| BoardColumn {
        id: id.into(),
        column_label: label.into(),
        priority_tone: tone,
    })
    .collect()
}

fn issue(id: &str, column: &str, status: IssueStatus, comments: u32, files: u32) -> Issue {
    Issue {
        id: id.into(),
        column_id: column.into(),
        title: format!("Issue {id}"),
        description: Some(format!("Description for issue {id}.")),
        issue_status: Some(status),
        comment_count: comments,
        attachment_count: files,
        assignee: None,
        labels: Vec::new(),
    }
}

/// The rest state: Low 3, Medium 3, High 3, Done 2.
fn rest_issues() -> Vec<Issue> {
    vec![
        issue("l1", "low", IssueStatus::Review, 0, 0),
        issue("l2", "low", IssueStatus::Progress, 1, 0),
        issue("l3", "low", IssueStatus::Fixed, 0, 1),
        issue("m1", "medium", IssueStatus::Review, 2, 0),
        issue("m2", "medium", IssueStatus::Progress, 0, 2),
        issue("m3", "medium", IssueStatus::Attention, 3, 1),
        issue("h1", "high", IssueStatus::Attention, 4, 1),
        issue("h2", "high", IssueStatus::Progress, 2, 0),
        issue("h3", "high", IssueStatus::Attention, 5, 2),
        issue("d1", "done", IssueStatus::Fixed, 1, 0),
        issue("d2", "done", IssueStatus::Fixed, 0, 1),
    ]
}

fn rest_app() -> Element {
    rsx! { IssueBoardScreen { board_columns: columns(), issues: rest_issues() } }
}

fn counts_app() -> Element {
    rsx! {
        IssueBoardScreen {
            board_columns: columns(),
            issues: vec![
                issue("quiet", "low", IssueStatus::Review, 0, 0),
                issue("talk", "low", IssueStatus::Progress, 2, 0),
                issue("file", "low", IssueStatus::Fixed, 0, 1),
                issue("both", "low", IssueStatus::Fixed, 1, 3),
            ],
        }
    }
}

#[test]
fn rest_board_split_is_three_three_three_and_two_cards() {
    let issues = rest_issues();
    let counts: Vec<usize> = columns()
        .iter()
        .map(|c| column_cards(&issues, &c.id).len())
        .collect();
    assert_eq!(counts, vec![3, 3, 3, 2]);
    assert_eq!(issues.len(), 11);
}

#[test]
fn rest_board_shows_four_columns_left_to_right_as_low_medium_high_done() {
    let html = render(rest_app);
    let at = |label: &str| {
        html.find(&format!("New issue in {label}"))
            .unwrap_or_else(|| panic!("no column {label}"))
    };
    assert!(at("Low") < at("Medium"));
    assert!(at("Medium") < at("High"));
    assert!(at("High") < at("Done"));
    assert_eq!(html.matches("kit-ib__col\"").count(), 4);
}

#[test]
fn rest_board_renders_eleven_cards_each_with_a_priority_mark() {
    let html = render(rest_app);
    assert_eq!(html.matches("kit-ib__card\"").count(), 11);
    assert_eq!(html.matches("kit-ib__prio-mark").count(), 11);
    assert_eq!(html.matches("kit-ib__desc").count(), 11);
}

#[test]
fn card_names_only_the_counts_that_are_not_zero() {
    let html = render(counts_app);
    assert!(html.contains("2 comments"));
    assert!(html.contains("1 file"));
    assert!(html.contains("1 comment · 3 files"));
    assert_eq!(html.matches("kit-ib__counts").count(), 3);
}

#[test]
fn sort_button_reads_sort_by_with_the_active_key() {
    let html = render(rest_app);
    assert!(html.contains("Sort by: Priority"));
}

#[test]
fn board_region_has_no_bottom_gap_so_columns_reach_its_edge() {
    let wrap = rule(CSS, ".kit-ib__boardwrap");
    assert!(wrap.contains("flex: 1 1 auto"));
    assert!(!wrap.contains("padding-bottom"));
    let board = rule(CSS, ".kit-ib__board");
    assert!(board.contains("flex: 1 1 auto"));
    assert!(board.contains("width: 100%"));
}
