use dioxus::prelude::*;
use rv2_kit::agents::exchange_diagram::{
    Actor, ActorRole, Exchange, ExchangeDiagram, ExchangeKind, ExchangeState,
};
use rv2_kit::agents::reversible_change_log::{ChangeRow, ReversibleChangeLog, ValueChange};
use rv2_kit::agents::run_checkpoint_rail::{CheckpointEntry, RunCheckpointRail, RunState};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

const RAIL_CSS: &str = include_str!("../assets/run-checkpoint-rail.css");

/// Instance one: two done entries on the main line and the running current entry.
fn rail_at_rest() -> Vec<CheckpointEntry> {
    let plan = CheckpointEntry::new("a", "Plan approved");
    let mut draft = CheckpointEntry::new("b", "Drafted section 1");
    draft.follows_id = Some("a".into());
    let mut review = CheckpointEntry::new("c", "Review pass");
    review.follows_id = Some("b".into());
    review.run_state = RunState::Running;
    vec![plan, draft, review]
}

/// Instance two: a failed entry left behind on the main line, and the restart from
/// the second entry carrying the current attempt on a branch line.
fn rail_branched() -> Vec<CheckpointEntry> {
    let plan = CheckpointEntry::new("a", "Plan approved");
    let mut draft = CheckpointEntry::new("b", "Drafted section 1");
    draft.follows_id = Some("a".into());
    let mut figures = CheckpointEntry::new("c", "Figures and tables");
    figures.follows_id = Some("b".into());
    figures.run_state = RunState::Failed;
    let mut rewrite = CheckpointEntry::new("d", "Rewrite section 2");
    rewrite.follows_id = Some("b".into());
    rewrite.run_state = RunState::Running;
    vec![plan, draft, figures, rewrite]
}

#[test]
fn rail_at_rest_shows_three_rows_with_the_last_current_and_no_branch() {
    let html = render(|| {
        rsx! { RunCheckpointRail { entries: rail_at_rest(), current_id: "c".to_string() } }
    });
    assert_eq!(html.matches("class=\"kit-rcr__row\"").count(), 3);
    assert_eq!(html.matches("data-current=\"true\"").count(), 1);
    assert!(html.contains("Review pass"));
    assert!(!html.contains("kit-rcr__lane"));
    assert!(!html.contains("attempts"));
}

#[test]
fn rail_bar_counts_rows_and_names_attempts_only_after_a_restart() {
    let at_rest = render(|| {
        rsx! { RunCheckpointRail { entries: rail_at_rest(), current_id: "c".to_string() } }
    });
    assert!(at_rest.contains("3 entries"));
    assert!(!at_rest.contains("attempts"));
    let branched = render(|| {
        rsx! { RunCheckpointRail { entries: rail_branched(), current_id: "d".to_string() } }
    });
    assert!(branched.contains("4 entries"));
    assert!(branched.contains("2 attempts"));
    assert_eq!(branched.matches("class=\"kit-rcr__row\"").count(), 4);
}

#[test]
fn abandoned_rows_stay_muted_on_the_main_line_and_the_new_attempt_sits_left_of_it() {
    let html = render(|| {
        rsx! { RunCheckpointRail { entries: rail_branched(), current_id: "d".to_string() } }
    });
    assert_eq!(html.matches("data-left-behind=\"true\"").count(), 1);
    assert!(html.contains("data-state=\"failed\""));
    assert!(html.contains("Figures and tables"));
    assert_eq!(html.matches("data-branch=\"true\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-rcr__lane\"").count(), 1);
    assert!(html.contains("--rcr-x: 8px"));
    assert!(html.contains("--rcr-gutter: 16px"));
}

#[test]
fn no_row_prints_an_attempt_number() {
    let html = render(|| {
        rsx! { RunCheckpointRail { entries: rail_branched(), current_id: "d".to_string() } }
    });
    assert!(!html.contains("attempt 2"));
    assert!(!html.contains("kit-rcr__attempt"));
}

#[test]
fn current_node_is_filled_and_done_nodes_are_not() {
    assert!(RAIL_CSS.contains(
        ".kit-rcr__row[data-current=\"true\"] .kit-rcr__node {\n  background: var(--kit-accent);"
    ));
    let node_rule = RAIL_CSS
        .split(".kit-rcr__node {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .unwrap_or_default();
    assert!(node_rule.contains("background: var(--kit-surface);"));
}

const LEDGER_CSS: &str = include_str!("../assets/reversible-change-log.css");

fn ledger_value(key: &str, verb: &str, target: &str, at: u64, from: &str, to: &str) -> ChangeRow {
    let mut row = ChangeRow::new(key, verb, "Sheets", at);
    row.target_label = Some(target.to_string());
    row.value_change = Some(ValueChange {
        from_text: from.to_string(),
        to_text: to.to_string(),
    });
    row
}

fn ledger_summary(
    key: &str,
    verb: &str,
    target: &str,
    system: &str,
    at: u64,
    text: &str,
) -> ChangeRow {
    let mut row = ChangeRow::new(key, verb, system, at);
    row.target_label = Some(target.to_string());
    row.summary_line = Some(text.to_string());
    row
}

/// Instance one: two applied value changes, newest first.
fn ledger_at_rest() -> Vec<ChangeRow> {
    vec![
        ledger_value("c1", "Renamed", "Q3 plan", 1_000, "Q3 draft", "Q3 plan"),
        ledger_value("c5", "Updated", "price table", 9_000, "$19", "$24"),
    ]
}

/// Instance two: an applied value change, an applied summary change, a not
/// reversible row, and the reverted rename.
fn ledger_branched() -> Vec<ChangeRow> {
    let mut sent = ledger_summary("c4", "Sent", "Invoice 204", "Mail", 8_200, "To accounts");
    sent.reversible = false;
    vec![
        ledger_value("c5", "Updated", "price table", 9_700, "$19", "$24"),
        sent,
        ledger_summary(
            "c3",
            "Deleted",
            "old-export.csv",
            "Drive",
            4_600,
            "Moved to trash",
        ),
        ledger_value("c1", "Renamed", "Q3 plan", 1_000, "Q3 draft", "Q3 plan"),
    ]
}

#[test]
fn ledger_lists_newest_first_and_strikes_the_old_value_in_each_detail_line() {
    let html = render(|| {
        rsx! { ReversibleChangeLog { change_rows: ledger_at_rest(), clock_ms: 10_000u64 } }
    });
    assert_eq!(html.matches("class=\"kit-rcl__row\"").count(), 2);
    assert_eq!(html.matches("class=\"kit-rcl__from\"").count(), 2);
    let newer = html.find("from $19 to $24").expect("newer row");
    let older = html.find("from Q3 draft to Q3 plan").expect("older row");
    assert!(newer < older);
    assert!(LEDGER_CSS.contains(".kit-rcl__from {\n  text-decoration: line-through;"));
}

#[test]
fn ledger_controls_appear_on_hover_or_focus_and_on_touch() {
    let html = render(|| {
        rsx! { ReversibleChangeLog { change_rows: ledger_at_rest(), clock_ms: 10_000u64 } }
    });
    assert_eq!(html.matches("class=\"kit-rcl__control\"").count(), 2);
    assert!(LEDGER_CSS.contains(".kit-rcl__control {\n"));
    assert!(LEDGER_CSS.contains("  opacity: 0;"));
    assert!(LEDGER_CSS.contains(".kit-rcl__row:hover .kit-rcl__control"));
    assert!(LEDGER_CSS.contains("@media (hover: none)"));
}

#[test]
fn ledger_reverted_row_offers_reapply_and_the_cannot_revert_flag_sits_in_the_slot() {
    let html = render(|| {
        rsx! {
            ReversibleChangeLog {
                change_rows: ledger_branched(),
                clock_ms: 10_000u64,
                reverted_keys: Some(vec!["c1".to_string()]),
            }
        }
    });
    assert!(html.contains("Reapply: Renamed Q3 plan"));
    assert!(html.contains("1 reverted"));
    assert!(html.contains("data-state=\"not_reversible\""));
    assert!(html.contains("Cannot revert"));
    assert_eq!(html.matches("class=\"kit-rcl__control\"").count(), 3);
}

#[test]
fn ledger_time_and_system_share_a_line_and_every_row_keeps_its_slot() {
    let html = render(|| {
        rsx! {
            ReversibleChangeLog {
                change_rows: ledger_branched(),
                clock_ms: 10_000u64,
                reverted_keys: Some(vec!["c1".to_string()]),
            }
        }
    });
    assert_eq!(html.matches("class=\"kit-rcl__meta\"").count(), 4);
    assert_eq!(html.matches("class=\"kit-rcl__slot\"").count(), 4);
    assert!(html.contains("kit-rcl__system\">Drive</span>"));
}

const DIAGRAM_CSS: &str = include_str!("../assets/exchange-diagram.css");

fn cast() -> Vec<Actor> {
    let agent = |key: &str, caption: &str| Actor {
        actor_key: key.to_string(),
        role: ActorRole::Agent,
        caption: caption.to_string(),
        mark_source: None,
        tint_key: None,
        model_tag: None,
        summary: None,
    };
    vec![
        Actor {
            actor_key: "p".to_string(),
            role: ActorRole::Person,
            caption: "You".to_string(),
            mark_source: None,
            tint_key: None,
            model_tag: None,
            summary: None,
        },
        agent("a", "Planner"),
        agent("t", "Researcher"),
        agent("w", "Writer"),
        agent("r", "Reviewer"),
    ]
}

fn exchange(
    key: &str,
    from: &str,
    to: &str,
    kind: ExchangeKind,
    state: ExchangeState,
    headline: &str,
) -> Exchange {
    Exchange {
        exchange_key: key.to_string(),
        from_key: from.to_string(),
        to_key: to.to_string(),
        kind,
        answers_key: None,
        state,
        sent_at_ms: None,
        group_tag: None,
        headline: headline.to_string(),
        detail_text: None,
    }
}

/// Instance one: an open request from You to Planner, and Planner's self-addressed exchange.
fn diagram_at_rest() -> Vec<Exchange> {
    vec![
        exchange(
            "e1",
            "p",
            "a",
            ExchangeKind::Request,
            ExchangeState::Ok,
            "Plan the release notes",
        ),
        exchange(
            "e2",
            "a",
            "a",
            ExchangeKind::Answer,
            ExchangeState::Ok,
            "Scope noted",
        ),
    ]
}

/// Instance two: the first two rows as at rest, then a failed request, a request
/// awaiting the person, and the answer that closes e1.
fn diagram_branched() -> Vec<Exchange> {
    let mut answer = exchange(
        "e5",
        "a",
        "p",
        ExchangeKind::Answer,
        ExchangeState::Ok,
        "Plan ready, four steps",
    );
    answer.answers_key = Some("e1".to_string());
    vec![
        exchange(
            "e1",
            "p",
            "a",
            ExchangeKind::Request,
            ExchangeState::Ok,
            "Plan the release notes",
        ),
        exchange(
            "e2",
            "a",
            "a",
            ExchangeKind::Answer,
            ExchangeState::Ok,
            "Scope noted",
        ),
        exchange(
            "e3",
            "a",
            "r",
            ExchangeKind::Request,
            ExchangeState::Failed,
            "Review the draft",
        ),
        exchange(
            "e4",
            "a",
            "p",
            ExchangeKind::Request,
            ExchangeState::AwaitingPerson,
            "Approve the outline?",
        ),
        answer,
    ]
}

fn row_names(html: &str) -> Vec<String> {
    html.split("aria-label=\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .filter(|name| name.contains("Opens a card."))
        .map(str::to_string)
        .collect()
}

#[test]
fn diagram_at_rest_shows_five_chips_and_counts_working_and_messages() {
    let html = render(|| {
        rsx! { ExchangeDiagram { actors: cast(), exchanges: diagram_at_rest(), run_heading: "Release run" } }
    });
    assert_eq!(html.matches("class=\"kit-agsq__chip\"").count(), 5);
    assert_eq!(html.matches("class=\"kit-agsq__life\"").count(), 10);
    assert!(html.contains("1 working · 2 messages"));
}

#[test]
fn request_carries_its_headline_tag_and_open_bar_and_self_loop_is_marked() {
    let html = render(|| {
        rsx! { ExchangeDiagram { actors: cast(), exchanges: diagram_at_rest(), run_heading: "Release run" } }
    });
    assert!(html.contains(">Plan the release notes<"));
    assert_eq!(html.matches("data-state=\"working\"").count(), 1);
    assert_eq!(html.matches("data-self=\"true\"").count(), 1);
}

#[test]
fn arrows_are_not_boxed_and_only_group_frames_draw_a_frame() {
    let html = render(|| {
        rsx! { ExchangeDiagram { actors: cast(), exchanges: diagram_at_rest(), run_heading: "Release run" } }
    });
    assert_eq!(html.matches("kit-agsq__group").count(), 0);
    let row_rule = DIAGRAM_CSS
        .split(".kit-agsq__row {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .unwrap_or_default();
    assert!(row_rule.contains("background: none;"));
    assert!(row_rule.contains("border: 0;"));
}

#[test]
fn answer_runs_back_and_failed_and_awaiting_exchanges_say_so_in_words() {
    let html = render(|| {
        rsx! { ExchangeDiagram { actors: cast(), exchanges: diagram_branched(), run_heading: "Release run" } }
    });
    assert!(html.contains("data-direction=\"backward\""));
    assert!(html.contains("Planner to Reviewer, Review the draft, Failed. Opens a card."));
    assert_eq!(html.matches("kit-agsq__cross").count(), 1);
    assert!(html.contains("Needs your reply"));
    assert!(html.contains("data-state=\"awaiting-person\""));
}

#[test]
fn rows_of_the_first_diagram_keep_their_places_in_the_second() {
    let first = render(|| {
        rsx! { ExchangeDiagram { actors: cast(), exchanges: diagram_at_rest(), run_heading: "Release run" } }
    });
    let second = render(|| {
        rsx! { ExchangeDiagram { actors: cast(), exchanges: diagram_branched(), run_heading: "Release run" } }
    });
    let first_rows = row_names(&first);
    let second_rows = row_names(&second);
    assert_eq!(first_rows.len(), 2);
    assert_eq!(second_rows.len(), 5);
    assert_eq!(second_rows[..2], first_rows[..]);
}
