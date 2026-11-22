use dioxus::prelude::*;
use rv2_kit::components::agent_context_map::{
    AgentContextMap, AgentNode, ItemKind, Memory, SourceArtifact, Tool, arc_positions,
    bracket_centres, effective_disabled, memory_state_words, outdated_ids, overflow_label,
    recalled_visible, relative_age, short_date, side_slots, tree_positions, treemap, visible_slots,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn memory(id: &str) -> Memory {
    Memory {
        memory_id: id.into(),
        memory_text: format!("Fact {id}"),
        source_artifact: SourceArtifact {
            artifact_name: "Brief".into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn tool(id: &str) -> Tool {
    Tool {
        tool_id: id.into(),
        tool_name: format!("Tool {id}"),
        access_level: "Read".into(),
        ..Default::default()
    }
}

fn agent() -> AgentNode {
    AgentNode {
        agent_id: "a1".into(),
        agent_name: "Ada".into(),
        agent_subtitle: "Idle".into(),
        agent_mark: None,
    }
}

#[test]
fn ring_slots_fill_the_left_arc_top_to_bottom() {
    let slots = arc_positions(6, true, 500.0, 300.0, 380.0, 230.0);
    assert_eq!(slots.len(), 6);
    assert!(slots.iter().all(|(x, _)| *x < 500.0));
    assert!(slots.windows(2).all(|w| w[0].1 < w[1].1));
}

#[test]
fn right_arc_mirrors_the_left_arc() {
    let right = arc_positions(4, false, 500.0, 300.0, 380.0, 230.0);
    assert!(right.iter().all(|(x, _)| *x > 500.0));
    let left = arc_positions(4, true, 500.0, 300.0, 380.0, 230.0);
    for (r, l) in right.iter().zip(left.iter()) {
        assert!((r.1 - l.1).abs() < 1e-9);
    }
}

#[test]
fn side_slots_stack_evenly_on_their_half_with_room_for_a_card() {
    let left = side_slots(6, true, 500.0, 360.0, 340.0, 360.0);
    let right = side_slots(6, false, 500.0, 360.0, 340.0, 360.0);
    assert!(left.iter().all(|(x, _)| *x < 500.0));
    assert!(right.iter().all(|(x, _)| *x > 500.0));
    let gaps: Vec<f64> = left.windows(2).map(|w| w[1].1 - w[0].1).collect();
    assert!(gaps.iter().all(|g| (g - gaps[0]).abs() < 1e-9));
    assert!(
        gaps[0] >= 119.9,
        "vertical step {} would overlap a 110px card",
        gaps[0]
    );
    for (l, r) in left.iter().zip(right.iter()) {
        assert!((l.0 + r.0 - 1000.0).abs() < 1e-9);
        assert!((l.1 - r.1).abs() < 1e-9);
    }
}

#[test]
fn tree_parent_sits_over_the_mean_of_its_children() {
    let children = vec![vec![1, 2], vec![], vec![]];
    let pos = tree_positions(&children, 0, 100.0, 80.0);
    assert_eq!(pos[1].0, 0.0);
    assert_eq!(pos[2].0, 100.0);
    assert_eq!(pos[0].0, 50.0);
    assert_eq!(pos[0].1, 0.0);
    assert_eq!(pos[2].1, 80.0);
}

#[test]
fn tree_unreachable_nodes_stay_at_origin_and_cycles_terminate() {
    let children = vec![vec![1], vec![0], vec![]];
    let pos = tree_positions(&children, 0, 10.0, 10.0);
    assert_eq!(pos.len(), 3);
    assert_eq!(pos[2], (0.0, 0.0));
}

#[test]
fn bracket_of_eight_has_four_two_one_pairings_and_adjacent_seeds_meet_first() {
    let rounds = bracket_centres(8, 40.0, 200.0);
    assert_eq!(rounds.len(), 3);
    assert_eq!(
        rounds.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![4, 2, 1]
    );
    let first_y: Vec<f64> = rounds[0].iter().map(|p| p.1).collect();
    assert_eq!(first_y, vec![40.0, 120.0, 200.0, 280.0]);
    assert_eq!(rounds[1][0].1, 80.0);
    assert_eq!(rounds[2][0].1, 160.0);
}

#[test]
fn bracket_rejects_counts_that_are_not_powers_of_two() {
    assert!(bracket_centres(6, 40.0, 200.0).is_empty());
    assert!(bracket_centres(1, 40.0, 200.0).is_empty());
}

#[test]
fn treemap_areas_follow_the_weight_ratio() {
    let rects = treemap(&[100.0, 300.0], 0.0, 0.0, 400.0, 100.0);
    let total = 400.0 * 100.0;
    let a = rects[0].w * rects[0].h / total;
    let b = rects[1].w * rects[1].h / total;
    assert!((a - 0.25).abs() < 1e-9, "{a}");
    assert!((b - 0.75).abs() < 1e-9, "{b}");
}

#[test]
fn treemap_zero_weights_split_equally() {
    let rects = treemap(&[0.0, 0.0], 0.0, 0.0, 200.0, 100.0);
    assert!((rects[0].w - rects[1].w).abs() < 1e-9);
}

#[test]
fn seven_memories_show_six_slots_and_one_overflow() {
    assert_eq!(visible_slots(7, 6), (6, 1));
    assert_eq!(visible_slots(8, 6), (6, 2));
    assert_eq!(visible_slots(2, 6), (2, 0));
    assert_eq!(overflow_label(1), "+1 more");
}

#[test]
fn replacing_memory_marks_the_replaced_one_outdated() {
    let old = memory("old");
    let mut new = memory("new");
    new.replaces_memory_id = Some("old".into());
    assert_eq!(outdated_ids(&[old, new]), vec!["old".to_string()]);
}

#[test]
fn switched_off_memory_leaves_the_recalled_list_and_gains_a_word() {
    let disabled = vec!["m2".to_string()];
    let recalled = vec!["m1".to_string(), "m2".to_string()];
    assert_eq!(
        recalled_visible(&recalled, &disabled),
        vec!["m1".to_string()]
    );
    assert_eq!(memory_state_words(false, true, true), vec!["switched off"]);
}

#[test]
fn local_switch_overrides_the_item_flag() {
    assert!(effective_disabled(false, Some(false)));
    assert!(!effective_disabled(true, Some(true)));
    assert!(effective_disabled(true, None));
}

#[test]
fn relative_age_reads_in_the_largest_whole_unit() {
    assert_eq!(relative_age(Some(10_000.0), Some(0.0)), "just now");
    assert_eq!(relative_age(Some(3_600_000.0 * 3.0), Some(0.0)), "3h ago");
    assert_eq!(relative_age(Some(86_400_000.0 * 2.0), Some(0.0)), "2d ago");
    assert_eq!(relative_age(None, Some(0.0)), "");
}

#[test]
fn short_date_formats_month_and_day_in_utc() {
    assert_eq!(short_date(0.0), "Jan 1");
    assert_eq!(short_date(1_700_000_000_000.0), "Nov 14");
}

#[test]
fn active_tool_shows_in_use_and_a_pulse_wire() {
    let html = render(|| {
        rsx! {
            AgentContextMap {
                agent_node: agent(),
                tool_items: vec![tool("t1"), tool("t2")],
                active_tool_ids: vec!["t1".to_string()],
            }
        }
    });
    assert!(html.contains("in use"), "{html}");
    assert!(html.contains("data-state=\"active\""), "{html}");
    assert!(html.contains("role=\"status\""));
}

#[test]
fn seven_memories_render_six_cards_and_a_plus_one_entry() {
    let html = render(|| {
        rsx! {
            AgentContextMap {
                agent_node: agent(),
                memory_items: (1..=7).map(|i| memory(&format!("m{i}"))).collect::<Vec<_>>(),
            }
        }
    });
    assert_eq!(html.matches("kit-actx__card").count(), 6, "{html}");
    assert!(html.contains("+1 more"));
    assert!(html.contains("7 memories"));
}

#[test]
fn agent_is_a_button_that_declares_a_dialog_popup() {
    let html = render(|| rsx! { AgentContextMap { agent_node: agent() } });
    assert!(html.contains("aria-haspopup=\"dialog\""), "{html}");
    assert!(html.contains("data-empty=\"true\""));
}

#[test]
fn memory_card_names_its_origin_and_outdated_state_in_text() {
    let html = render(|| {
        rsx! {
            AgentContextMap {
                agent_node: agent(),
                memory_items: {
                    let mut old = memory("old");
                    old.is_outdated = true;
                    vec![old]
                },
            }
        }
    });
    assert!(html.contains("outdated"), "{html}");
    assert!(html.contains("from Brief"), "{html}");
}

#[test]
fn item_kind_is_comparable_for_callbacks() {
    assert_ne!(ItemKind::Memory, ItemKind::Tool);
}

mod reach {
    use dioxus::prelude::*;
    use rv2_kit::components::agent_tool_reach_map::{
        AccessTool, AgentState, AgentToolReachMap, CallState, Grant, MapAgent, Reach, ReplayInput,
        ToolCall, is_json_object, map_counts, replay_input, sorted_calls, tool_reach,
    };

    fn call(id: &str, state: CallState, at: f64) -> ToolCall {
        ToolCall {
            call_id: id.into(),
            call_state: state,
            call_input: "{\"q\":1}".into(),
            call_output: None,
            call_error: None,
            called_at: at,
            duration: None,
            is_replay: false,
        }
    }

    pub fn agent() -> MapAgent {
        MapAgent {
            agent_id: "a".into(),
            agent_name: "Ada".into(),
            model_name: None,
            agent_mark: None,
            agent_state: AgentState::Running,
            agent_task: None,
            granted_tools: vec![
                Grant {
                    tool_id: "t1".into(),
                    call_list: vec![
                        call("c2", CallState::Failed, 20.0),
                        call("c1", CallState::Done, 10.0),
                    ],
                },
                Grant {
                    tool_id: "t2".into(),
                    call_list: vec![],
                },
            ],
        }
    }

    fn catalog() -> Vec<AccessTool> {
        ["t1", "t2", "t3"]
            .iter()
            .map(|id| AccessTool {
                tool_id: (*id).into(),
                tool_name: format!("Tool {id}"),
                ..Default::default()
            })
            .collect()
    }

    #[test]
    fn reach_is_no_access_not_used_or_failed_by_the_grant_and_its_last_call() {
        let a = agent();
        assert_eq!(tool_reach(None), Reach::NoAccess);
        assert_eq!(tool_reach(a.granted_tools.get(1)), Reach::NotUsed);
        assert_eq!(tool_reach(a.granted_tools.first()), Reach::Failed);
    }

    #[test]
    fn calls_are_listed_oldest_first() {
        let sorted = sorted_calls(&agent().granted_tools[0].call_list);
        assert_eq!(sorted[0].call_id, "c1");
        assert_eq!(sorted[1].call_id, "c2");
    }

    #[test]
    fn heading_counts_track_used_running_and_calls() {
        assert_eq!(map_counts(&agent(), &catalog()), (0, 1, 2, 2));
    }

    #[test]
    fn edited_object_replays_as_json_and_a_broken_one_is_refused() {
        assert_eq!(
            replay_input("{\"a\":1}", "{\"a\": 2}"),
            Ok(ReplayInput::Json("{\"a\": 2}".into()))
        );
        assert!(replay_input("{\"a\":1}", "{\"a\": ").is_err());
        assert_eq!(
            replay_input("plain", "anything"),
            Ok(ReplayInput::Text("anything".into()))
        );
        assert!(is_json_object(
            "{\"a\":[1,2.5e3,true,null,{\"b\":\"x\\\"y\"}]}"
        ));
        assert!(!is_json_object("{\"a\":1} trailing"));
    }

    #[test]
    fn granted_tool_reads_not_used_yet_and_ungranted_reads_no_access() {
        let html = render(|| {
            rsx! { AgentToolReachMap { agent_list: vec![agent()], tool_catalog: catalog() } }
        });
        assert!(html.contains("Not used yet"), "{html}");
        assert!(html.contains("No access"), "{html}");
        assert!(html.contains("data-granted=\"false\""));
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
}

mod branches {
    use dioxus::prelude::*;
    use rv2_kit::components::branch_choice_map::{
        Branch, BranchChoiceMap, Consequence, Fork, KeyPick, Risk, branch_label_for_screen_reader,
        digit_pick, flagged_consequences, next_flagged, pager_step, pending_after, preference_ok,
    };

    pub fn branch(label: &str, preferred: bool, flagged: bool) -> Branch {
        Branch {
            branch_label: label.into(),
            branch_mark: None,
            consequences: vec![Consequence {
                text: "Sends mail".into(),
                flag_explanation: flagged.then(|| "Irreversible".to_string()),
                reference_url: None,
            }],
            touched_apps: vec!["Mail".into()],
            agent_preferred: preferred,
            follow_up: None,
        }
    }

    pub fn fork(id: &str) -> Fork {
        Fork {
            fork_id: id.into(),
            question_text: "Send the reminder?".into(),
            context_text: None,
            agent_name: "Ada".into(),
            model_name: None,
            agent_mark: None,
            risk_level: Some(Risk::High),
            branches: vec![
                branch("Send now", true, true),
                branch("Wait", false, false),
                branch("Skip", false, false),
            ],
        }
    }

    #[test]
    fn digit_keys_pick_branches_and_the_one_after_opens_custom() {
        assert_eq!(digit_pick(2, 3, true), Some(KeyPick::Branch(1)));
        assert_eq!(digit_pick(4, 3, true), Some(KeyPick::Custom));
        assert_eq!(digit_pick(4, 3, false), None);
        assert_eq!(digit_pick(0, 3, true), None);
    }

    #[test]
    fn flagged_consequences_walk_forward_and_wrap() {
        let f = fork("f1");
        assert_eq!(flagged_consequences(&f), vec![(0, 0)]);
        assert_eq!(next_flagged(1, None), Some(0));
        assert_eq!(next_flagged(1, Some(0)), Some(0));
        assert_eq!(next_flagged(0, None), None);
    }

    #[test]
    fn pager_clamps_and_decided_forks_leave_the_list() {
        assert_eq!(pager_step(3, 0, -1), 0);
        assert_eq!(pager_step(3, 2, 1), 2);
        let forks = vec![fork("a"), fork("b"), fork("c")];
        let left = pending_after(&forks, &["a".to_string()]);
        assert_eq!(
            left.iter().map(|f| f.fork_id.as_str()).collect::<Vec<_>>(),
            vec!["b", "c"]
        );
    }

    #[test]
    fn preference_is_announced_and_at_most_one_per_level() {
        assert_eq!(
            branch_label_for_screen_reader(&branch("Send now", true, false)),
            "Send now, agent's preference"
        );
        assert!(preference_ok(&fork("x").branches));
    }

    #[test]
    fn high_risk_tag_and_three_branch_buttons_render() {
        let html = render(|| rsx! { BranchChoiceMap { pending_forks: vec![fork("f1")] } });
        assert!(html.contains("High risk"), "{html}");
        assert_eq!(html.matches("kit-bcm__branch\"").count(), 3, "{html}");
        assert!(
            html.contains("agent&#39;s preference") || html.contains("agent's preference"),
            "{html}"
        );
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
}

mod causes {
    use dioxus::prelude::Key as DKey;
    use dioxus::prelude::*;
    use rv2_kit::components::cause_analysis_tree::{
        Cause, CauseAnalysisTree, CheckState, Effective, JoinRule, effective, flatten,
        highlighted_path, hypothesis_counts, keyboard_target,
    };

    fn cause(id: &str, state: CheckState, subs: Vec<Cause>) -> Cause {
        Cause {
            cause_id: id.into(),
            cause_title: format!("Cause {id}"),
            check_state: state,
            sub_causes: subs,
            ..Default::default()
        }
    }

    #[test]
    fn all_join_with_one_confirmed_child_stays_pending() {
        let c = Cause {
            join_rule: JoinRule::All,
            ..cause(
                "p",
                CheckState::Confirmed,
                vec![
                    cause("a", CheckState::Confirmed, vec![]),
                    cause("b", CheckState::Open, vec![]),
                ],
            )
        };
        assert_eq!(effective(&c), Effective::Pending);
        let any = Cause {
            join_rule: JoinRule::Any,
            ..c.clone()
        };
        assert_eq!(effective(&any), Effective::Confirmed);
    }

    #[test]
    fn a_confirmed_leaf_is_the_root_and_its_path_is_highlighted() {
        let tree = cause(
            "f",
            CheckState::Open,
            vec![cause(
                "mid",
                CheckState::Confirmed,
                vec![cause("leaf", CheckState::Confirmed, vec![])],
            )],
        );
        let nodes = flatten(&tree);
        assert_eq!(nodes[2].state, Effective::Root);
        assert_eq!(highlighted_path(&nodes), vec![true, true, true]);
    }

    #[test]
    fn ruled_out_causes_still_count_as_hypotheses() {
        let tree = cause(
            "f",
            CheckState::Open,
            vec![
                cause("x", CheckState::RuledOut, vec![]),
                cause("y", CheckState::Open, vec![]),
            ],
        );
        assert_eq!(hypothesis_counts(&flatten(&tree)), (2, 1));
        assert_eq!(Effective::RuledOut.word(), "Skipped");
    }

    #[test]
    fn arrows_move_between_parent_and_children() {
        let tree = cause(
            "f",
            CheckState::Open,
            vec![
                cause("x", CheckState::Open, vec![]),
                cause("y", CheckState::Open, vec![]),
            ],
        );
        let nodes = flatten(&tree);
        assert_eq!(keyboard_target(&nodes, 0, &DKey::ArrowDown), 1);
        assert_eq!(keyboard_target(&nodes, 1, &DKey::ArrowUp), 0);
        assert_eq!(keyboard_target(&nodes, 1, &DKey::ArrowRight), 2);
        assert_eq!(keyboard_target(&nodes, 2, &DKey::Home), 0);
    }

    #[test]
    fn ruled_out_cause_renders_with_its_word() {
        let html = render(|| {
            rsx! { CauseAnalysisTree { failure_node: Cause { cause_title: "Build fails".into(), sub_causes: vec![Cause { cause_title: "Cache".into(), check_state: CheckState::RuledOut, ..Default::default() }], ..Default::default() } } }
        });
        assert!(html.contains("Skipped"), "{html}");
        assert!(html.contains("Build fails"));
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
}

mod failover {
    use dioxus::prelude::*;
    use rv2_kit::components::model_failover_flow::{
        Attempt, AttemptState, FlowStatus, ModelFailoverFlow, attempt_position, cost_text,
        flow_status, shown_state, state_word,
    };

    fn attempt(id: &str, state: AttemptState) -> Attempt {
        Attempt {
            attempt_id: id.into(),
            model_name: format!("model-{id}"),
            provider_name: "Provider".into(),
            attempt_state: state,
            duration: None,
            attempt_cost: None,
            error_text: None,
        }
    }

    #[test]
    fn attempts_after_an_answer_read_as_skipped_not_failed() {
        let list = vec![
            attempt("1", AttemptState::Failed),
            attempt("2", AttemptState::Answered),
            attempt("3", AttemptState::Queued),
        ];
        assert_eq!(shown_state(&list, 2), AttemptState::Skipped);
        assert_eq!(flow_status(&list), FlowStatus::Answered);
    }

    #[test]
    fn each_attempt_names_a_tooltip_card_it_is_described_by() {
        fn app() -> Element {
            rsx! {
                ModelFailoverFlow {
                    attempt_list: vec![attempt("1", AttemptState::Failed), attempt("2", AttemptState::Answered)],
                }
            }
        }
        let html = render(app);
        assert_eq!(html.matches("role=\"tooltip\"").count(), 2, "{html}");
        assert!(html.contains("aria-describedby=\"kit-mfl-"), "{html}");
    }

    #[test]
    fn position_follows_the_running_attempt() {
        let list = vec![
            attempt("1", AttemptState::Failed),
            attempt("2", AttemptState::InProgress),
            attempt("3", AttemptState::Queued),
            attempt("4", AttemptState::Queued),
        ];
        assert_eq!(attempt_position(&list), (2, 4));
    }

    #[test]
    fn all_failed_is_exhausted_and_missing_cost_has_no_text() {
        let list = vec![
            attempt("1", AttemptState::Failed),
            attempt("2", AttemptState::Failed),
        ];
        assert_eq!(flow_status(&list), FlowStatus::Exhausted);
        assert_eq!(cost_text(None), None);
        assert_eq!(
            state_word(AttemptState::Failed, Some("429 rate limit")),
            "429 rate limit"
        );
    }

    #[test]
    fn running_summary_reads_attempt_n_of_n() {
        let html = render(|| {
            rsx! {
                ModelFailoverFlow {
                    attempt_list: vec![
                        Attempt { error_text: Some("429".into()), ..attempt("1", AttemptState::Failed) },
                        attempt("2", AttemptState::InProgress),
                        attempt("3", AttemptState::Queued),
                        attempt("4", AttemptState::Queued),
                    ],
                }
            }
        });
        assert!(html.contains("Attempt 2 of 4"), "{html}");
        assert!(html.contains("429"));
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
}

mod routing {
    use dioxus::prelude::*;
    use rv2_kit::components::traffic_split_router::{
        Fallback, Policy, PolicyState, Route, Strategy, TrafficSplitRouter, blended_cost,
        normalise_shares, policy_state, resolve_fallback,
    };

    fn route(id: &str, name: &str, p95: f64, cost: f64) -> Route {
        Route {
            route_id: id.into(),
            route_name: name.into(),
            route_provider: None,
            route_mark: None,
            route_tone: None,
            p95_seconds: p95,
            cost_per_1k: cost,
        }
    }

    fn routes() -> Vec<Route> {
        vec![
            route("a", "Fast", 1.0, 0.002),
            route("b", "Smart", 4.0, 0.010),
        ]
    }

    fn live() -> Policy {
        Policy {
            active_strategy: "even".into(),
            fallback: Fallback::default(),
        }
    }

    #[test]
    fn shares_normalise_to_one_hundred_and_missing_routes_get_none() {
        let ids = vec!["a".to_string(), "b".to_string()];
        let s = Strategy {
            strategy_id: "x".into(),
            strategy_label: "X".into(),
            share_map: vec![("a".into(), 3.0)],
        };
        let shares = normalise_shares(&s, &ids);
        let total: f64 = shares.iter().map(|(_, v)| v).sum();
        assert!((total - 100.0).abs() < 1e-9);
        assert_eq!(shares[1].1, 0.0);
    }

    #[test]
    fn blended_cost_is_the_share_weighted_sum() {
        let shares = vec![("a".to_string(), 50.0), ("b".to_string(), 50.0)];
        assert!((blended_cost(&shares, &routes()) - 0.006).abs() < 1e-12);
    }

    #[test]
    fn fallback_never_repeats_a_route_and_unknown_names_resolve_to_the_first() {
        let fb = Fallback {
            enabled: true,
            from_route_id: Some("a".into()),
            to_route_id: Some("a".into()),
        };
        assert_eq!(
            resolve_fallback(&fb, &routes()),
            Some(("a".into(), "b".into()))
        );
        let unknown = Fallback {
            enabled: true,
            from_route_id: Some("zz".into()),
            to_route_id: None,
        };
        assert_eq!(
            resolve_fallback(&unknown, &routes()),
            Some(("a".into(), "b".into()))
        );
        assert_eq!(resolve_fallback(&Fallback::default(), &routes()), None);
        assert_eq!(resolve_fallback(&unknown, &routes()[..1]), None);
    }

    #[test]
    fn an_edited_draft_is_unsaved_until_it_matches_live() {
        let mut draft = live();
        assert_eq!(policy_state(&draft, &live()), PolicyState::Live);
        draft.active_strategy = "cost".into();
        assert_eq!(policy_state(&draft, &live()), PolicyState::Unsaved);
    }

    #[test]
    fn live_policy_reads_live_and_disables_deploy() {
        let html = render(|| {
            rsx! {
                TrafficSplitRouter {
                    route_list: routes(),
                    strategy_list: vec![Strategy { strategy_id: "even".into(), strategy_label: "Even".into(), share_map: vec![] }],
                    live_policy: Policy { active_strategy: "even".into(), fallback: Fallback::default() },
                    environment_label: "production",
                }
            }
        });
        assert!(html.contains("Live"), "{html}");
        assert!(html.contains("Policy v0"), "{html}");
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
}

mod flow {
    use dioxus::prelude::*;
    use rv2_kit::components::automation_flow_editor::{
        AutomationFlowEditor, Path, RunResult, Step, StepKind, Template, UNDO_LIMIT, can_run,
        insert_after, move_step, path_skipped, push_snapshot, remove_step,
    };

    pub fn step(id: &str) -> Step {
        Step {
            step_id: id.into(),
            step_kind: StepKind::Action,
            step_title: format!("Step {id}"),
            step_detail: None,
            step_note: None,
            step_icon: None,
            branch_paths: vec![],
        }
    }

    fn branch(id: &str, paths: Vec<Vec<Step>>) -> Step {
        Step {
            step_kind: StepKind::Branch,
            branch_paths: paths
                .into_iter()
                .enumerate()
                .map(|(i, s)| Path {
                    path_name: format!("Path {i}"),
                    path_steps: s,
                })
                .collect(),
            ..step(id)
        }
    }

    #[test]
    fn inserting_after_a_step_lands_between_its_neighbours() {
        let mut flow = vec![step("1"), step("2"), step("3")];
        assert!(insert_after(&mut flow, "2", step("new")));
        let ids: Vec<&str> = flow.iter().map(|s| s.step_id.as_str()).collect();
        assert_eq!(ids, vec!["1", "2", "new", "3"]);
    }

    #[test]
    fn insertion_reaches_into_a_branch_path() {
        let mut flow = vec![branch("b", vec![vec![step("p1")], vec![]])];
        assert!(insert_after(&mut flow, "p1", step("p2")));
        assert_eq!(flow[0].branch_paths[0].path_steps.len(), 2);
    }

    #[test]
    fn removing_a_branch_hands_its_first_path_back_in_place() {
        let mut flow = vec![
            step("0"),
            branch("b", vec![vec![step("p1"), step("p2")], vec![step("q")]]),
            step("9"),
        ];
        assert!(remove_step(&mut flow, "b"));
        let ids: Vec<&str> = flow.iter().map(|s| s.step_id.as_str()).collect();
        assert_eq!(ids, vec!["0", "p1", "p2", "9"]);
    }

    #[test]
    fn move_stops_at_the_ends_and_flow_needs_two_steps_to_run() {
        let mut flow = vec![step("1"), step("2")];
        assert!(!move_step(&mut flow, "1", -1));
        assert!(move_step(&mut flow, "1", 1));
        assert_eq!(flow[0].step_id, "2");
        assert!(!can_run(&[step("1")]));
        assert!(can_run(&flow));
    }

    #[test]
    fn a_path_is_skipped_only_when_all_its_steps_are_skipped() {
        let p = Path {
            path_name: "No".into(),
            path_steps: vec![step("x")],
        };
        let results = vec![("x".to_string(), RunResult::Skipped)];
        assert!(path_skipped(&p, &results));
        assert!(!path_skipped(&Path::default(), &results));
    }

    #[test]
    fn undo_history_is_capped() {
        let mut stack = Vec::new();
        for _ in 0..(UNDO_LIMIT + 5) {
            push_snapshot(&mut stack, vec![]);
        }
        assert_eq!(stack.len(), UNDO_LIMIT);
    }

    #[test]
    fn empty_flow_offers_only_the_trigger_menu() {
        let html = render(|| {
            rsx! {
                AutomationFlowEditor {
                    trigger_templates: vec![Template { step_kind: StepKind::Trigger, step_title: "Webhook".into(), step_detail: None, step_icon: None, branch_paths: vec![] }],
                }
            }
        });
        assert!(html.contains("Start the flow with a trigger."), "{html}");
        assert!(html.contains("Webhook"));
    }

    fn five_steps() -> Vec<Step> {
        (1..=5).map(|i| step(&i.to_string())).collect()
    }

    #[test]
    fn the_canvas_has_no_inline_height_unless_one_is_given() {
        let html = render(|| rsx! { AutomationFlowEditor { default_flow_steps: five_steps() } });
        assert!(!html.contains("style=\"height"), "{html}");
    }

    #[test]
    fn an_explicit_canvas_height_is_kept_on_the_canvas() {
        let html = render(|| {
            rsx! { AutomationFlowEditor { default_flow_steps: five_steps(), canvas_height: 480 } }
        });
        assert!(html.contains("height: 480px;"), "{html}");
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
}

mod bracket {
    use dioxus::prelude::*;
    use rv2_kit::components::ranking_bracket::{
        Entrant, RankingBracket, Verdict, champion, resolve_bracket, round_name, status_line,
    };

    fn entrants(n: usize) -> Vec<Entrant> {
        (0..n)
            .map(|i| Entrant {
                entrant_id: format!("e{i}"),
                entrant_name: format!("Answer {i}"),
                ..Default::default()
            })
            .collect()
    }

    fn verdict(winner: &str) -> Verdict {
        Verdict {
            winner_id: winner.into(),
            rationale: Some("Clearer".into()),
            scores: Some((80.0, 60.0)),
        }
    }

    #[test]
    fn adjacent_seeds_meet_first_and_the_winner_advances() {
        let rounds = resolve_bracket(&entrants(8), &[("0:2".to_string(), verdict("e4"))]);
        assert_eq!(
            rounds.iter().map(Vec::len).collect::<Vec<_>>(),
            vec![4, 2, 1]
        );
        let pairing = &rounds[0][2];
        assert_eq!(
            pairing.upper.as_ref().map(|e| e.entrant_id.as_str()),
            Some("e4")
        );
        assert_eq!(pairing.winner.as_deref(), Some("e4"));
        assert_eq!(
            rounds[1][1].upper.as_ref().map(|e| e.entrant_id.as_str()),
            Some("e4")
        );
    }

    #[test]
    fn a_winner_who_is_not_in_the_pairing_is_ignored() {
        let rounds = resolve_bracket(&entrants(4), &[("0:0".to_string(), verdict("e3"))]);
        assert_eq!(rounds[0][0].winner, None);
        assert_eq!(rounds[0][0].verdict, None);
    }

    #[test]
    fn rounds_are_named_from_the_final_back() {
        assert_eq!(round_name(2, 3), "Final");
        assert_eq!(round_name(1, 3), "Semifinal");
        assert_eq!(round_name(0, 3), "Quarterfinal");
        assert_eq!(round_name(0, 4), "Round 1");
    }

    #[test]
    fn champion_needs_a_valid_final_verdict() {
        let verdicts = vec![
            ("0:0".to_string(), verdict("e0")),
            ("0:1".to_string(), verdict("e2")),
            ("1:0".to_string(), verdict("e2")),
        ];
        assert_eq!(
            champion(&resolve_bracket(&entrants(4), &verdicts)).as_deref(),
            Some("e2")
        );
        assert_eq!(
            status_line(Some("0:1"), None, "judge"),
            "Judging 0:1 · judge"
        );
    }

    #[test]
    fn non_power_of_two_renders_no_rounds() {
        assert!(resolve_bracket(&entrants(3), &[]).is_empty());
    }

    #[test]
    fn bracket_renders_round_names_and_the_status_line() {
        let html = render(
            || rsx! { RankingBracket { entrants: vec![Entrant { entrant_id: "a".into(), entrant_name: "Alpha".into(), ..Default::default() }, Entrant { entrant_id: "b".into(), entrant_name: "Beta".into(), ..Default::default() }] } },
        );
        assert!(html.contains("Final"), "{html}");
        assert!(html.contains("Waiting"), "{html}");
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
}

mod race {
    use dioxus::prelude::*;
    use rv2_kit::components::candidate_race_board::{
        Candidate, CandidateRaceBoard, RunState, best_key, choosable, in_use_key, progress_percent,
    };

    fn cand(key: &str, state: RunState, score: Option<f64>) -> Candidate {
        Candidate {
            candidate_key: key.into(),
            display_name: format!("Model {key}"),
            method_summary: "Direct".into(),
            model_id: None,
            mark: None,
            run_state: state,
            fraction_done: Some(0.4),
            current_action: None,
            quality_score: score,
            outcome_note: None,
            elapsed_secs: None,
            run_cost: None,
        }
    }

    #[test]
    fn best_is_the_highest_finished_score_and_is_in_use_by_default() {
        let list = vec![
            cand("a", RunState::Finished, Some(70.0)),
            cand("b", RunState::Finished, Some(90.0)),
            cand("c", RunState::Running, Some(99.0)),
        ];
        assert_eq!(best_key(&list).as_deref(), Some("b"));
        assert_eq!(in_use_key(&list, None, None).as_deref(), Some("b"));
    }

    #[test]
    fn a_running_chosen_key_is_in_use_but_not_choosable() {
        let list = vec![
            cand("a", RunState::Finished, Some(70.0)),
            cand("r", RunState::Running, None),
        ];
        assert_eq!(in_use_key(&list, Some("r"), None).as_deref(), Some("r"));
        assert!(!choosable(&list[1]));
        assert!(choosable(&list[0]));
    }

    #[test]
    fn progress_is_full_when_finished_and_kept_when_halted() {
        let mut halted = cand("h", RunState::Halted, None);
        halted.fraction_done = Some(0.35);
        assert!((progress_percent(&halted) - 35.0).abs() < 1e-9);
        assert_eq!(
            progress_percent(&cand("f", RunState::Finished, None)),
            100.0
        );
    }

    #[test]
    fn best_marker_and_in_use_state_render_in_words() {
        let html = render(
            || rsx! { CandidateRaceBoard { candidate_list: vec![cand("a", RunState::Finished, Some(70.0)), cand("b", RunState::Finished, Some(90.0))] } },
        );
        assert!(html.contains("best"), "{html}");
        assert!(html.contains("in use"), "{html}");
        assert!(html.contains("Best of 2"), "{html}");
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
}

mod work {
    use dioxus::prelude::*;
    use rv2_kit::components::parallel_work_grid::{
        Item, ItemState, ParallelWorkGrid, counts, progress_tenths, remaining_secs, retry_ids,
    };

    fn item(id: &str, state: ItemState, start: f64, end: f64) -> Item {
        Item {
            item_id: id.into(),
            item_label: format!("Item {id}"),
            item_state: state,
            worker_name: None,
            start_offset: Some(start),
            end_offset: Some(end),
            item_cost: Some(0.5),
            attempt_count: 1,
            output_value: None,
            error_text: None,
        }
    }

    #[test]
    fn counts_split_by_state() {
        let items = vec![
            item("1", ItemState::Done, 0.0, 1.0),
            item("2", ItemState::InProgress, 0.0, 0.0),
            item("3", ItemState::Retrying, 0.0, 0.0),
            item("4", ItemState::Failed, 0.0, 1.0),
        ];
        let c = counts(&items);
        assert_eq!((c.done, c.in_progress, c.retrying, c.failed), (1, 1, 1, 1));
        assert_eq!(retry_ids(&items), vec!["4".to_string()]);
    }

    #[test]
    fn remaining_time_stays_hidden_until_three_items_finish() {
        let two = vec![
            item("1", ItemState::Done, 0.0, 10.0),
            item("2", ItemState::Done, 0.0, 10.0),
            item("3", ItemState::Queued, 0.0, 0.0),
        ];
        assert_eq!(remaining_secs(&two, Some(1)), None);
        let three = vec![
            item("1", ItemState::Done, 0.0, 10.0),
            item("2", ItemState::Done, 0.0, 10.0),
            item("3", ItemState::Done, 0.0, 10.0),
            item("4", ItemState::Queued, 0.0, 0.0),
            item("5", ItemState::Queued, 0.0, 0.0),
        ];
        assert_eq!(remaining_secs(&three, Some(2)), Some(10.0 * 2.0 / 2.0));
        assert_eq!(remaining_secs(&three, None), None);
    }

    #[test]
    fn progress_is_counted_in_tenths_of_settled_items() {
        let items: Vec<Item> = (0..10)
            .map(|i| {
                item(
                    &i.to_string(),
                    if i < 4 {
                        ItemState::Done
                    } else {
                        ItemState::Queued
                    },
                    0.0,
                    1.0,
                )
            })
            .collect();
        assert_eq!(progress_tenths(&items), 4);
    }

    #[test]
    fn grid_shows_the_done_count_line() {
        let html = render(
            || rsx! { ParallelWorkGrid { job_items: vec![item("1", ItemState::Done, 0.0, 1.0), item("2", ItemState::Queued, 0.0, 0.0)], job_name: "Translate" } },
        );
        assert!(html.contains("1 of 2 done"), "{html}");
        assert!(html.contains("Translate"));
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
}

mod overlap {
    use dioxus::prelude::*;
    use rv2_kit::components::parallel_edit_overlap_grid::{
        ConflictOption, Contributor, Edit, FileRow, ParallelEditOverlapGrid, can_merge,
        is_conflict, spans_overlap, status_word,
    };

    fn contributor(id: &str, active: bool) -> Contributor {
        Contributor {
            contributor_id: id.into(),
            contributor_name: format!("Agent {id}"),
            model_name: None,
            branch_name: None,
            contributor_mark: None,
            is_active: active,
        }
    }

    fn edit(id: &str, a: u32, b: u32) -> Edit {
        Edit {
            contributor_id: id.into(),
            line_range: (a, b),
            is_editing: false,
            edit_note: None,
        }
    }

    fn overlapping_row() -> FileRow {
        FileRow {
            file_path: "src/lib.rs".into(),
            edits: vec![edit("x", 1, 10), edit("y", 8, 20)],
            conflict: None,
        }
    }

    #[test]
    fn spans_are_inclusive() {
        assert!(spans_overlap((1, 5), (5, 9)));
        assert!(!spans_overlap((1, 4), (5, 9)));
    }

    #[test]
    fn merge_waits_for_every_pick_and_every_contributor() {
        let rows = vec![overlapping_row()];
        let cs = vec![contributor("x", false), contributor("y", false)];
        assert!(is_conflict(&rows[0]));
        assert!(!can_merge(&rows, &[], &cs));
        assert!(can_merge(
            &rows,
            &[("src/lib.rs".to_string(), "x".to_string())],
            &cs
        ));
        assert!(!can_merge(&rows, &[], &[contributor("x", true)]));
    }

    #[test]
    fn active_contributors_are_counted_in_the_status_word() {
        assert_eq!(
            status_word(&[], &[], &[contributor("x", true), contributor("y", true)]),
            "2 still active"
        );
        assert_eq!(
            status_word(&[], &[], &[contributor("x", false)]),
            "Ready to combine"
        );
        let _ = ConflictOption {
            contributor_id: String::new(),
            option_text: String::new(),
        };
    }

    #[test]
    fn conflicts_are_named_in_text_and_active_columns_say_active() {
        let html = render(|| {
            rsx! { ParallelEditOverlapGrid { contributors: vec![Contributor { is_active: true, ..Default::default() }], file_rows: vec![FileRow { file_path: "src/a.rs".into(), edits: vec![], conflict: None }] } }
        });
        assert!(html.contains("Active"), "{html}");
        let html2 =
            render(|| rsx! { ParallelEditOverlapGrid { contributors: vec![], file_rows: vec![] } });
        assert!(html2.contains("0 files"), "{html2}");
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
}

mod territory {
    use dioxus::prelude::*;
    use rv2_kit::components::workspace_territory_map::{
        AgentAction, Change, MAP_H, MAP_W, RepoAgent, RepoFile, WorkspaceTerritoryMap,
        after_escape, delta_text, file_card, folder_of, folder_runs, is_conflict, last_editor,
        layout, map_order, normalised_files, status_line, step_index, tile_rects,
    };

    fn file(path: &str, lines: u32) -> RepoFile {
        RepoFile {
            file_path: path.into(),
            line_count: lines,
            change_log: vec![],
        }
    }

    fn agent(id: &str, file: Option<&str>, action: AgentAction) -> RepoAgent {
        RepoAgent {
            agent_id: id.into(),
            agent_name: id.into(),
            agent_mark: None,
            current_file: file.map(str::to_string),
            read_file: None,
            agent_action: action,
            agent_task: None,
        }
    }

    #[test]
    fn arrows_walk_every_file_of_a_folder_before_the_next_folder() {
        let files = map_order(
            &[
                file("billing/a.ts", 1),
                file("api/b.ts", 1),
                file("api/a.ts", 1),
                file("billing/c.ts", 1),
            ],
            2,
        );
        let paths: Vec<&str> = files.iter().map(|f| f.file_path.as_str()).collect();
        assert_eq!(
            paths,
            ["api/a.ts", "api/b.ts", "billing/a.ts", "billing/c.ts"]
        );
        assert_eq!(step_index(4, 0, &Key::ArrowRight), Some(1));
        assert_eq!(step_index(4, 1, &Key::ArrowDown), Some(2));
        assert_eq!(step_index(4, 3, &Key::ArrowLeft), Some(2));
        assert_eq!(step_index(4, 0, &Key::ArrowLeft), None);
        assert_eq!(step_index(4, 3, &Key::ArrowRight), None);
        assert_eq!(step_index(4, 2, &Key::End), Some(3));
        assert_eq!(step_index(4, 2, &Key::Home), Some(0));
        assert_eq!(step_index(4, 0, &Key::Enter), None);
    }

    #[test]
    fn escape_hides_an_open_card_and_keeps_the_agent_focus() {
        assert_eq!(
            after_escape(Some(2), Some("a".into())),
            (None, Some("a".into()))
        );
        assert_eq!(after_escape(None, Some("a".into())), (None, None));
    }

    #[test]
    fn a_file_with_two_edits_shows_the_most_recent_editor() {
        let f = RepoFile {
            file_path: "src/x.rs".into(),
            line_count: 5,
            change_log: vec![
                Change {
                    agent_id: "a".into(),
                    lines_added: None,
                    lines_removed: None,
                },
                Change {
                    agent_id: "b".into(),
                    lines_added: None,
                    lines_removed: None,
                },
            ],
        };
        assert_eq!(last_editor(&f).as_deref(), Some("b"));
    }

    #[test]
    fn a_read_only_agent_tile_has_a_dashed_edge_and_no_fill() {
        let html = render(|| {
            rsx! {
                WorkspaceTerritoryMap {
                    file_list: vec![file("src/x.rs", 40)],
                    agent_list: vec![agent("r", Some("src/x.rs"), AgentAction::Read)],
                    repo_name: "rv2",
                }
            }
        });
        assert!(html.contains("data-read=\"true\""), "{html}");
        assert!(!html.contains("data-editor"), "{html}");
    }

    #[test]
    fn focusing_an_agent_dims_every_file_it_is_not_in() {
        let html = render(|| {
            rsx! {
                WorkspaceTerritoryMap {
                    file_list: vec![file("src/x.rs", 40), file("src/y.rs", 40)],
                    agent_list: vec![agent("a", Some("src/x.rs"), AgentAction::Edit)],
                    repo_name: "rv2",
                    focused_agent_id: Some("a".to_string()),
                }
            }
        });
        assert_eq!(
            wide(&html).matches("data-dim=\"true\"").count(),
            1,
            "{html}"
        );
    }

    #[test]
    fn the_file_card_lists_path_lines_agents_present_and_history() {
        let f = RepoFile {
            file_path: "billing/rounding.ts".into(),
            line_count: 150,
            change_log: vec![Change {
                agent_id: "a".into(),
                lines_added: Some(18),
                lines_removed: Some(3),
            }],
        };
        let card = file_card(
            &f,
            &[agent("a", Some("billing/rounding.ts"), AgentAction::Edit)],
        );
        assert_eq!(card.path, "billing/rounding.ts");
        assert_eq!(card.lines, 150);
        assert_eq!(card.agents, ["a"]);
        assert_eq!(card.history, ["a +18 -3"]);
    }

    #[test]
    fn the_status_line_names_the_editing_agent_and_its_file() {
        let mut agents = vec![
            agent("billing", Some("billing/rounding.ts"), AgentAction::Edit),
            agent("ledger", Some("billing/rounding.ts"), AgentAction::Edit),
            agent("docs", Some("tests/rounding.test.ts"), AgentAction::Read),
        ];
        agents[0].agent_name = "Billing agent".into();
        agents[1].agent_name = "Ledger agent".into();
        assert_eq!(
            status_line(&agents).as_deref(),
            Some("Ledger agent is editing billing/rounding.ts")
        );
        agents[1].current_file = Some("db/migrations.sql".into());
        assert_eq!(
            status_line(&agents).as_deref(),
            Some("Ledger agent is editing db/migrations.sql")
        );
    }

    #[test]
    fn tiles_fill_the_map_and_each_area_is_proportional_to_its_lines() {
        let files = vec![
            file("api/a.ts", 120),
            file("api/b.ts", 90),
            file("billing/c.ts", 240),
            file("billing/d.ts", 180),
            file("docs/e.md", 58),
        ];
        let ordered = map_order(&files, 2);
        let (folders, tiles) = layout(&ordered, &folder_runs(&ordered, 2));
        let box_area = MAP_W * MAP_H;
        let folder_area: f64 = folders.iter().map(|r| r.w * r.h).sum();
        let tile_area: f64 = tiles.iter().map(|r| r.w * r.h).sum();
        assert!((folder_area - box_area).abs() < 1e-6, "{folder_area}");
        assert!((tile_area - box_area).abs() < 1e-6, "{tile_area}");
        let per_line: Vec<f64> = tiles
            .iter()
            .zip(&ordered)
            .map(|(r, f)| r.w * r.h / f.line_count as f64)
            .collect();
        for k in &per_line {
            assert!((k - per_line[0]).abs() < 1e-6, "{per_line:?}");
        }
    }

    #[test]
    fn the_gallery_at_rest_reads_six_of_twenty_three_changed_in_five_folders() {
        let html = render(gallery_at_rest);
        assert!(html.contains("6 of 23 files changed"), "{html}");
        assert_eq!(
            html.matches("class=\"kit-wtm__group\"").count(),
            5,
            "{html}"
        );
        assert_eq!(
            html.matches("class=\"kit-wtm__tile\"").count(),
            23,
            "{html}"
        );
    }

    #[test]
    fn the_gallery_at_rest_shows_each_agents_touched_file_count() {
        let html = render(gallery_at_rest);
        assert!(html.contains("Payments agent, 4 files touched"), "{html}");
        assert!(html.contains("Store agent, 3 files touched"), "{html}");
        assert!(html.contains("Notes agent, 1 file touched"), "{html}");
    }

    #[test]
    fn the_gallery_at_rest_has_one_read_tile_one_conflict_tile_and_no_open_card() {
        let html = render(gallery_at_rest);
        assert_eq!(
            wide(&html).matches("data-read=\"true\"").count(),
            1,
            "{html}"
        );
        assert_eq!(
            wide(&html).matches("data-conflict=\"true\"").count(),
            1,
            "{html}"
        );
        assert!(html.contains("api/quotes.ts, 95 lines"), "{html}");
        assert!(
            html.contains("payments/rounding.ts, 180 lines, conflict, two agents editing"),
            "{html}"
        );
        assert!(!html.contains("kit-wtm__card"), "{html}");
    }

    #[test]
    fn changed_tiles_show_added_and_removed_lines_and_unchanged_tiles_show_none() {
        let html = render(gallery_at_rest);
        assert_eq!(
            html.matches("class=\"kit-wtm__delta\"").count(),
            6,
            "{html}"
        );
        assert!(html.contains("+18 \u{2212}6"), "{html}");
        assert!(html.contains("+14"), "{html}");
    }

    #[test]
    fn the_gallery_at_rest_status_line_names_the_store_agent_and_its_file() {
        let html = render(gallery_at_rest);
        assert!(
            html.contains("Store agent is editing store/retry.rs"),
            "{html}"
        );
    }

    #[test]
    fn a_file_delta_sums_its_logged_changes_and_leaves_out_an_unreported_side() {
        let logged = |added: Option<u32>, removed: Option<u32>| Change {
            agent_id: "a".into(),
            lines_added: added,
            lines_removed: removed,
        };
        let two_edits = RepoFile {
            file_path: "src/x.rs".into(),
            line_count: 180,
            change_log: vec![logged(Some(8), Some(2)), logged(Some(10), Some(4))],
        };
        assert_eq!(delta_text(&two_edits).as_deref(), Some("+18 \u{2212}6"));
        let added_only = RepoFile {
            change_log: vec![logged(Some(14), None)],
            ..two_edits.clone()
        };
        assert_eq!(delta_text(&added_only).as_deref(), Some("+14"));
        assert_eq!(delta_text(&file("src/y.rs", 5)), None);
    }

    fn wide(html: &str) -> &str {
        let start = html.find("class=\"kit-wtm__map\"").expect("wide map");
        let end = html.find("class=\"kit-wtm__rows\"").expect("narrow rows");
        &html[start..end]
    }

    fn narrow(html: &str) -> &str {
        &html[html.find("class=\"kit-wtm__rows\"").expect("narrow rows")..]
    }

    #[test]
    fn the_narrow_form_lists_every_file_as_a_row_under_its_folder() {
        let html = render(gallery_at_rest);
        assert_eq!(
            narrow(&html)
                .matches("class=\"kit-wtm__row-group\"")
                .count(),
            5,
            "{html}"
        );
        assert_eq!(
            narrow(&html).matches("class=\"kit-wtm__row\"").count(),
            23,
            "{html}"
        );
    }

    #[test]
    fn the_narrow_rows_keep_line_counts_change_counts_tags_and_conflict_state() {
        let html = render(gallery_at_rest);
        let rows = narrow(&html);
        assert!(rows.contains("+18 \u{2212}6"), "{html}");
        assert!(rows.contains("2 changes"), "{html}");
        assert_eq!(rows.matches("1 change").count(), 5, "{html}");
        assert_eq!(rows.matches("data-conflict=\"true\"").count(), 1, "{html}");
        assert_eq!(rows.matches("data-read=\"true\"").count(), 1, "{html}");
        assert_eq!(rows.matches("kit-wtm__tag").count(), 2, "{html}");
    }

    fn gallery_at_rest() -> Element {
        let change = |agent: &str, added: u32, removed: u32| Change {
            agent_id: agent.into(),
            lines_added: Some(added),
            lines_removed: Some(removed),
        };
        let f = |path: &str, lines: u32, log: Vec<Change>| RepoFile {
            file_path: path.into(),
            line_count: lines,
            change_log: log,
        };
        let files = vec![
            f("api/routes.ts", 120, vec![]),
            f("api/auth.ts", 90, vec![]),
            f("api/webhooks.ts", 60, vec![]),
            f("api/session.ts", 70, vec![]),
            f("api/rates.ts", 55, vec![]),
            f("api/quotes.ts", 95, vec![]),
            f(
                "payments/rounding.ts",
                180,
                vec![change("store", 8, 2), change("payments", 10, 4)],
            ),
            f("payments/invoice.ts", 240, vec![change("payments", 12, 4)]),
            f("payments/ledger.ts", 200, vec![]),
            f("payments/refunds.ts", 130, vec![]),
            f("payments/tax.ts", 110, vec![]),
            f("store/cursor.rs", 150, vec![change("store", 30, 2)]),
            f("store/retry.rs", 110, vec![change("store", 9, 1)]),
            f("store/pool.rs", 80, vec![]),
            f("store/schema.rs", 300, vec![]),
            f(
                "checks/rounding.test.ts",
                140,
                vec![change("payments", 22, 3)],
            ),
            f("checks/invoice.test.ts", 95, vec![]),
            f("checks/ledger.test.ts", 120, vec![]),
            f("checks/fixtures.json", 40, vec![]),
            f("checks/setup.ts", 30, vec![]),
            f(
                "notes/rounding-note.md",
                90,
                vec![Change {
                    agent_id: "notes".into(),
                    lines_added: Some(14),
                    lines_removed: None,
                }],
            ),
            f("notes/billing.md", 58, vec![]),
            f("notes/api.md", 40, vec![]),
        ];
        let at = |id: &str, name: &str, file: &str| RepoAgent {
            agent_id: id.into(),
            agent_name: name.into(),
            agent_mark: None,
            current_file: Some(file.into()),
            read_file: None,
            agent_action: AgentAction::Edit,
            agent_task: None,
        };
        let mut agents = vec![
            at("payments", "Payments agent", "payments/rounding.ts"),
            at("notes", "Notes agent", "payments/rounding.ts"),
            at("store", "Store agent", "store/retry.rs"),
        ];
        agents[0].read_file = Some("api/quotes.ts".into());
        rsx! {
            WorkspaceTerritoryMap { file_list: files, agent_list: agents, repo_name: "tally-service", branch_name: Some("fix/cent-rounding".to_string()) }
        }
    }

    #[test]
    fn tile_areas_follow_line_counts() {
        let files = vec![file("a.rs", 100), file("b.rs", 300)];
        let rects = tile_rects(&files, 2);
        let total = rects.iter().map(|r| r.w * r.h).sum::<f64>();
        let ratio = rects[0].w * rects[0].h / (rects[1].w * rects[1].h);
        assert!((ratio - 1.0 / 3.0).abs() < 1e-9, "{total}");
    }

    #[test]
    fn layout_is_the_same_whatever_the_input_order() {
        let a = map_order(&[file("z/b.rs", 10), file("a/c.rs", 20)], 2);
        let b = map_order(
            &[file("a/c.rs", 20), file("z/b.rs", 10), file("z/b.rs", 10)],
            2,
        );
        assert_eq!(a, b);
        assert_eq!(normalised_files(&b).len(), 2);
        assert_eq!(
            tile_rects(&[file("z/b.rs", 10), file("a/c.rs", 20)], 2),
            tile_rects(&[file("a/c.rs", 20), file("z/b.rs", 10)], 2)
        );
        assert_eq!(folder_runs(&a, 2).len(), 2);
    }

    #[test]
    fn folders_group_by_depth() {
        assert_eq!(folder_of("a/b/c/d.rs", 2), "a/b");
        assert_eq!(folder_of("top.rs", 2), "");
    }

    #[test]
    fn two_agents_editing_one_file_is_a_conflict_until_one_moves_on() {
        let f = file("src/x.rs", 5);
        let mut agents = vec![
            agent("a", Some("src/x.rs"), AgentAction::Edit),
            agent("b", Some("src/x.rs"), AgentAction::Edit),
        ];
        assert!(is_conflict(&f, &agents));
        agents[1].current_file = Some("other.rs".into());
        assert!(!is_conflict(&f, &agents));
        let _ = Change {
            agent_id: String::new(),
            lines_added: None,
            lines_removed: None,
        };
    }

    #[test]
    fn conflict_tile_is_named_in_its_label() {
        let html = render(|| {
            rsx! {
                WorkspaceTerritoryMap {
                    file_list: vec![RepoFile { file_path: "src/x.rs".into(), line_count: 240, change_log: vec![] }],
                    agent_list: vec![RepoAgent { agent_id: "a".into(), agent_name: "Ada".into(), agent_mark: None, current_file: Some("src/x.rs".into()), read_file: None, agent_action: AgentAction::Edit, agent_task: None }, RepoAgent { agent_id: "b".into(), agent_name: "Bo".into(), agent_mark: None, current_file: Some("src/x.rs".into()), read_file: None, agent_action: AgentAction::Edit, agent_task: None }],
                    repo_name: "rv2",
                }
            }
        });
        assert!(html.contains("conflict, two agents editing"), "{html}");
        assert!(html.contains("rv2"));
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
}
