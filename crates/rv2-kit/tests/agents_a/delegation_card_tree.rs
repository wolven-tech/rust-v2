use super::render;
use dioxus::prelude::*;
use rv2_kit::agents::delegation_card_tree::{
    Agent, AgentState, CommandAction, DelegationCardTree, LastTransfer, LoadTone, ToolName,
    Transfer, Workspace, activity_line, agent_name, card_label, clock_text, command_line,
    commands_for, compact_tokens, elapsed_seconds, flatten_ids, format_spend, handoff_line,
    load_percent, load_tone, new_transfers, tally, token_text, tool_tiles,
};

fn agent(id: &str, name: &str, state: AgentState) -> Agent {
    Agent {
        id: id.to_string(),
        name: name.to_string(),
        state,
        doing_text: None,
        load_ratio: None,
        load_capacity_tokens: None,
        load_used_tokens: None,
        model_label: None,
        model_mark: None,
        tool_names: Vec::new(),
        last_transfer: None,
        assignment: None,
        started_at: None,
        ended_at: None,
        spend: None,
        tokens: None,
        identity_mark: None,
        sub_agents: Vec::new(),
    }
}

fn tools(count: usize) -> Vec<ToolName> {
    (0..count)
        .map(|n| ToolName {
            mark: None,
            name: format!("tool{n}"),
        })
        .collect()
}

fn transfer(id: &str, source: &str, target: &str, note: Option<&str>) -> Transfer {
    Transfer {
        id: id.to_string(),
        source_id: source.to_string(),
        target_id: target.to_string(),
        note: note.map(str::to_string),
    }
}

#[test]
fn load_is_a_clamped_whole_percent_from_ratio_or_tokens() {
    let mut a = agent("a", "A", AgentState::Active);
    a.load_ratio = Some(0.44);
    assert_eq!(load_percent(&a), 44);
    a.load_ratio = Some(1.7);
    assert_eq!(load_percent(&a), 100);
    a.load_ratio = Some(f64::NAN);
    assert_eq!(load_percent(&a), 0);
    a.load_ratio = None;
    a.load_used_tokens = Some(88_000);
    a.load_capacity_tokens = Some(200_000);
    assert_eq!(load_percent(&a), 44);
}

#[test]
fn load_warns_above_eighty_and_is_critical_above_ninety_five() {
    assert_eq!(load_tone(80), LoadTone::Normal);
    assert_eq!(load_tone(81), LoadTone::Warn);
    assert_eq!(load_tone(95), LoadTone::Warn);
    assert_eq!(load_tone(96), LoadTone::Critical);
}

#[test]
fn token_figures_read_as_compact_used_over_capacity() {
    assert_eq!(
        token_text(Some(44_000), Some(200_000)),
        Some("44k / 200k".to_string())
    );
    assert_eq!(compact_tokens(1_200_000), "1.2M");
    assert_eq!(token_text(None, Some(200_000)), None);
}

#[test]
fn spend_formats_in_dollars_and_small_amounts_in_words() {
    assert_eq!(format_spend(1.5), "$1.50");
    assert_eq!(format_spend(0.004), "less than one cent");
    assert_eq!(format_spend(f64::INFINITY), "$0.00");
}

#[test]
fn elapsed_time_runs_from_start_and_a_future_start_reads_zero() {
    assert_eq!(elapsed_seconds(Some(10_000), None, 75_000), 65);
    assert_eq!(elapsed_seconds(Some(10_000), Some(40_000), 900_000), 30);
    assert_eq!(elapsed_seconds(Some(10_000), None, 5_000), 0);
    assert_eq!(elapsed_seconds(None, None, 5_000), 0);
    assert_eq!(clock_text(3_725), "1:02:05");
}

#[test]
fn activity_says_what_the_agent_does_or_why_it_rests() {
    let mut a = agent("a", "Coder", AgentState::Idle);
    assert_eq!(activity_line(&a), "Waiting for work");
    a.doing_text = Some("Writing tests".to_string());
    assert_eq!(activity_line(&a), "Writing tests");
}

#[test]
fn commands_follow_the_state() {
    assert!(commands_for(AgentState::Waiting).contains(&CommandAction::Approve));
    assert!(commands_for(AgentState::Waiting).contains(&CommandAction::Decline));
    assert!(!commands_for(AgentState::Active).contains(&CommandAction::Approve));
    assert!(!commands_for(AgentState::Paused).contains(&CommandAction::Pause));
    assert_eq!(
        commands_for(AgentState::Failed),
        vec![CommandAction::Retry, CommandAction::Message]
    );
    assert!(!commands_for(AgentState::Done).contains(&CommandAction::Retry));
}

#[test]
fn tool_tiles_collapse_past_four() {
    assert_eq!(tool_tiles(3), (3, 0));
    assert_eq!(tool_tiles(4), (4, 0));
    assert_eq!(tool_tiles(6), (3, 3));
}

fn tree() -> Agent {
    let mut lead = agent("lead", "Lead", AgentState::Active);
    lead.doing_text = Some("Planning".to_string());
    let mut coder = agent("coder", "Coder", AgentState::Waiting);
    coder.load_used_tokens = Some(150_000);
    coder.load_capacity_tokens = Some(200_000);
    let mut tester = agent("tester", "Tester", AgentState::Failed);
    tester.sub_agents = vec![agent("linter", "Linter", AgentState::Done)];
    lead.tool_names = tools(5);
    lead.load_ratio = Some(1.7);
    lead.sub_agents = vec![coder, tester];
    lead
}

#[test]
fn ids_walk_the_tree_depth_first_from_the_lead() {
    assert_eq!(
        flatten_ids(&tree()),
        vec!["lead", "coder", "tester", "linter"]
    );
}

#[test]
fn tally_counts_working_and_needs_you_across_the_tree() {
    assert_eq!(tally(&tree()), (1, 1));
    assert_eq!(agent_name(&tree(), "linter"), Some("Linter".to_string()));
    assert_eq!(agent_name(&tree(), "nobody"), None);
}

#[test]
fn only_transfers_unknown_at_mount_are_new() {
    let known = vec!["t1".to_string()];
    let all = vec![
        transfer("t1", "lead", "coder", None),
        transfer("t2", "lead", "tester", None),
    ];
    let news = new_transfers(&known, &all);
    assert_eq!(news.len(), 1);
    assert_eq!(news[0].id, "t2");
}

#[test]
fn hand_off_and_command_lines_read_as_sentences() {
    assert_eq!(
        handoff_line("Lead", "Coder", Some("Guard empty carts")),
        "Lead handed to Coder: Guard empty carts"
    );
    assert_eq!(handoff_line("Lead", "Coder", None), "Lead handed to Coder.");
    assert_eq!(command_line("Coder", CommandAction::Pause), "Coder paused.");
}

#[test]
fn card_label_names_state_activity_and_load() {
    let coder = tree().sub_agents.remove(0);
    assert_eq!(
        card_label(&coder),
        "Coder, Needs you, Waiting for you, load 75%, opens a dialog"
    );
}

#[test]
fn load_above_ninety_five_clamps_to_full_and_critical() {
    let mut a = agent("a", "A", AgentState::Active);
    a.load_ratio = Some(3.0);
    assert_eq!(load_percent(&a), 100);
    assert_eq!(load_tone(load_percent(&a)), LoadTone::Critical);
}

fn board() -> Element {
    let mut lead = tree();
    lead.last_transfer = Some(LastTransfer {
        from_name: "Lead".to_string(),
        text: "Guard empty carts".to_string(),
    });
    rsx! {
        DelegationCardTree {
            top_agent: lead,
            transfers: vec![transfer("t1", "lead", "coder", Some("Guard empty carts"))],
            heading: "Checkout run",
            on_command: move |_: (Agent, CommandAction, Option<String>)| {},
        }
    }
}

#[test]
fn board_renders_one_card_per_agent_with_a_labelled_toggle() {
    let html = render(board);
    assert!(
        html.contains("aria-label=\"Coder, Needs you, Waiting for you, load 75%, opens a dialog\"")
    );
    assert!(html.contains("data-state=\"failed\""));
    assert!(html.contains("aria-haspopup=\"dialog\""));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(html.contains("1 working · 1 need you"));
}

#[test]
fn over_limit_load_clamps_its_meter_and_tools_collapse() {
    let html = render(board);
    assert!(html.contains("aria-valuenow=\"100\""));
    assert!(html.contains("+2"));
}

#[test]
fn roving_tab_stop_is_single_and_details_stay_closed_until_opened() {
    let html = render(board);
    assert!(html.contains("tabindex=\"0\""));
    assert!(html.contains("tabindex=\"-1\""));
    assert!(!html.contains("kit-delg__details"));
}

#[test]
fn hand_off_present_at_mount_is_not_announced_or_pulsed() {
    let html = render(board);
    assert!(!html.contains("handed to"));
    assert!(!html.contains("kit-delg__pulse"));
}

#[test]
fn sub_agents_nest_inside_a_group_under_their_parent() {
    let html = render(board);
    assert!(html.contains("kit-delg__children"));
    assert!(html.contains("role=\"group\""));
}

fn switchable() -> Element {
    rsx! {
        DelegationCardTree {
            top_agent: tree(),
            heading: "Checkout run",
            workspaces: vec![
                Workspace { id: "ws1".to_string(), name: "Checkout".to_string(), active_count: Some(2), attention_count: Some(1) },
                Workspace { id: "ws2".to_string(), name: "Search".to_string(), active_count: None, attention_count: None },
            ],
            active_workspace: "ws2".to_string(),
        }
    }
}

#[test]
fn workspace_switcher_is_a_closed_listbox_button_showing_the_active_workspace() {
    let html = render(switchable);
    assert!(html.contains("aria-haspopup=\"listbox\""));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(html.contains("Search"));
    assert!(!html.contains("role=\"listbox\""));
}
