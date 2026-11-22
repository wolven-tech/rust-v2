use std::collections::BTreeMap;

use dioxus::prelude::*;
use rv2_kit::components::{
    decision_review_queue::{BodyFormat, DecisionReviewQueue, ReviewItem, ReviewRisk, diff_lines},
    deliberation_digest::{
        DeliberationDigest, DigestModel, DigestStep, Effort, Emphasis, KindSpec, kind_totals,
        merged_kinds, segment_widths, total_time,
    },
    fill_in_prompt_templates::{FillInPromptTemplates, Template, render_prompt},
    interruptible_answer_stream::{AgentRef, InterruptibleAnswerStream, StreamStatus},
    run_control_composer::{AgentIdentity, QueuedMessage, RunControlComposer, RunState},
    source_linked_answer::{Reference, Segment, SourceLinkedAnswer, citing_text, mark_passage},
    working_trail_panel::{TrailSource, TrailStep, WorkingTrailPanel, rail_start},
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

fn reference(id: &str, headline: &str, passage: Option<&str>) -> Reference {
    Reference {
        ref_id: id.into(),
        headline: headline.into(),
        origin_site: "example.com".into(),
        link_target: None,
        published_on: Some("2 Oct 2026".into()),
        passage: passage.map(Into::into),
        exact_phrase: None,
        leading_mark: None,
    }
}

fn segment(text: &str, cited: &[&str], starts_paragraph: bool) -> Segment {
    Segment {
        text: text.into(),
        cited_ids: cited.iter().map(|id| (*id).into()).collect(),
        starts_paragraph,
    }
}

fn step(id: &str, kind: &str, elapsed: Option<f64>) -> DigestStep {
    DigestStep {
        step_id: id.into(),
        kind_key: kind.into(),
        step_label: format!("Step {id}"),
        elapsed_time: elapsed,
        is_running: elapsed.is_none(),
        model_id: Some("m1".into()),
    }
}

#[test]
fn marked_passage_keeps_only_words_shared_with_the_citing_sentence() {
    let runs = mark_passage(
        "We now target June, pending the partner review.",
        "The launch moved to June after the partner review.",
        None,
    );
    let marked: Vec<&str> = runs
        .iter()
        .filter(|(_, flagged)| *flagged)
        .map(|(text, _)| text.trim())
        .collect();
    assert_eq!(marked, vec!["June,", "partner review."]);
}

#[test]
fn explicit_exact_phrase_is_the_only_marked_text() {
    let runs = mark_passage(
        "We now target June, pending the partner review.",
        "The launch moved to June after the partner review.",
        Some("target June"),
    );
    let marked: Vec<&str> = runs
        .iter()
        .filter(|(_, flagged)| *flagged)
        .map(|(text, _)| text.as_str())
        .collect();
    assert_eq!(marked, vec!["target June"]);
}

#[test]
fn source_is_quoted_by_every_sentence_that_cites_it() {
    let segments = vec![
        segment("First claim.", &["a"], true),
        segment("Unrelated.", &["b"], false),
        segment("Second claim.", &["a"], false),
    ];
    assert_eq!(citing_text(&segments, "a"), "First claim. Second claim.");
}

#[test]
fn source_linked_answer_at_rest_opens_first_preview_with_matched_words_marked() {
    fn app() -> Element {
        let refs = vec![
            reference(
                "r1",
                "Launch schedule",
                Some("We now target June, pending the partner review."),
            ),
            reference("r2", "Pricing FAQ", None),
        ];
        rsx! {
            SourceLinkedAnswer {
                answer_segments: vec![segment("The launch moved to June after the partner review.", &["r1"], true)],
                reference_list: refs,
                pages_consulted: 6,
                start_open_ref: Some("r1".to_string()),
                on_open_request: move |_| {},
                trailing_action_text: Some("Open all sources".to_string()),
            }
        }
    }
    let html = render(app);
    assert_eq!(count(&html, "class=\"kit-sla__preview\""), 1);
    assert!(html.contains("June,</mark>"));
    assert!(html.contains("Copy passage"));
    assert!(html.contains("Open reference"));
    assert!(html.contains("2 Oct 2026"));
    assert!(html.contains("role=\"status\""));
}

#[test]
fn source_linked_answer_without_open_preview_removes_the_region_but_keeps_status() {
    fn app() -> Element {
        rsx! {
            SourceLinkedAnswer {
                answer_segments: vec![segment("A claim.", &["r1"], true)],
                reference_list: vec![reference("r1", "Only source", Some("A passage."))],
            }
        }
    }
    let html = render(app);
    assert_eq!(count(&html, "class=\"kit-sla__preview\""), 0);
    assert!(html.contains("role=\"status\""));
}

#[test]
fn answer_with_two_paragraph_flags_renders_two_paragraphs() {
    fn app() -> Element {
        rsx! {
            SourceLinkedAnswer {
                answer_segments: vec![
                    segment("One.", &[], true),
                    segment("Two.", &[], false),
                    segment("Three.", &[], true),
                ],
                reference_list: vec![],
            }
        }
    }
    let html = render(app);
    assert_eq!(count(&html, "class=\"kit-sla__para\""), 2);
}

#[test]
fn segment_citing_two_ids_renders_both_markers_after_its_last_word() {
    fn app() -> Element {
        rsx! {
            SourceLinkedAnswer {
                answer_segments: vec![segment("Pricing holds for Team.", &["r2", "r1"], true)],
                reference_list: vec![reference("r1", "First", None), reference("r2", "Second", None)],
            }
        }
    }
    let html = render(app);
    assert_eq!(count(&html, "class=\"kit-sla__marker\""), 2);
    let tail = html.find("class=\"kit-sla__tail\"").expect("tail span");
    let first_marker = html.find("class=\"kit-sla__marker\"").expect("marker");
    assert!(tail < first_marker);
}

#[test]
fn citation_to_unknown_reference_renders_no_marker() {
    fn app() -> Element {
        rsx! {
            SourceLinkedAnswer {
                answer_segments: vec![segment("Orphan citation.", &["missing"], true)],
                reference_list: vec![reference("r1", "Known", None)],
            }
        }
    }
    let html = render(app);
    assert_eq!(count(&html, "class=\"kit-sla__marker\""), 0);
}

#[test]
fn finished_digest_widths_are_proportional_to_step_time() {
    let steps = vec![step("a", "search", Some(1.0)), step("b", "read", Some(3.0))];
    let (widths, remainder) = segment_widths(&steps, None);
    assert!((widths[0] - 25.0).abs() < 1e-9);
    assert!((widths[1] - 75.0).abs() < 1e-9);
    assert_eq!(remainder, 0.0);
}

#[test]
fn expected_time_beyond_elapsed_adds_a_remainder_segment() {
    let steps = vec![step("a", "search", Some(2.0)), step("b", "write", None)];
    let (_, remainder) = segment_widths(&steps, Some(4.0));
    assert!((remainder - 50.0).abs() < 1e-9);
}

#[test]
fn kinds_are_listed_in_order_of_first_appearance() {
    let steps = vec![
        step("a", "search", Some(1.0)),
        step("b", "read", Some(1.0)),
        step("c", "search", Some(1.0)),
        step("d", "reason", Some(1.0)),
    ];
    let table = merged_kinds(&BTreeMap::new());
    let keys: Vec<String> = kind_totals(&steps, &table)
        .into_iter()
        .map(|k| k.key)
        .collect();
    assert_eq!(keys, vec!["search", "read", "reason"]);
}

#[test]
fn step_without_a_time_counts_as_zero_in_the_total() {
    let steps = vec![step("a", "search", Some(2.0)), step("b", "read", None)];
    assert!((total_time(&steps) - 2.0).abs() < 1e-9);
}

fn at_rest_model(id: &str, name: &str, role: &str, effort: Effort, secs: f64) -> DigestModel {
    DigestModel {
        model_id: id.into(),
        display_name: name.into(),
        logo: None,
        role_label: role.into(),
        effort_level: effort,
        time_taken: secs,
        provider_name: "Northwind".into(),
        token_counts: None,
        spend: None,
        selection_note: None,
    }
}

fn at_rest_kinds() -> BTreeMap<String, KindSpec> {
    let kind = |label: &str| KindSpec {
        kind_label: label.into(),
        kind_emphasis: Emphasis::Normal,
        kind_mark: None,
    };
    BTreeMap::from([
        ("lookup".to_string(), kind("Lookup")),
        ("compare".to_string(), kind("Compare")),
        ("verify".to_string(), kind("Verify")),
    ])
}

fn at_rest_steps() -> Vec<DigestStep> {
    vec![
        step("s1", "lookup", Some(4.0)),
        step("s2", "lookup", Some(3.0)),
        step("s3", "compare", Some(2.0)),
        step("s4", "compare", Some(2.0)),
        step("s5", "verify", Some(1.0)),
        step("s6", "verify", None),
    ]
}

fn at_rest_digest() -> Element {
    rsx! {
        DeliberationDigest {
            step_list: at_rest_steps(),
            model_list: vec![
                at_rest_model("m1", "Quill", "Retrieval", Effort::High, 4.0),
                at_rest_model("m2", "Harbor", "Critique", Effort::Low, 1.0),
                at_rest_model("m3", "Lumen", "Drafting", Effort::Max, 3.0),
            ],
            total_source_count: Some(9),
            is_reasoning: true,
            expected_time: Some(14.0),
            kind_table: at_rest_kinds(),
        }
    }
}

#[test]
fn running_digest_at_rest_reads_working_with_a_remainder_segment() {
    let html = render(at_rest_digest);
    assert!(html.contains("Working"));
    assert!(html.contains("kit-digest__pulse"));
    assert_eq!(count(&html, "is-remainder"), 1);
}

#[test]
fn kinds_legend_at_rest_is_one_row_of_three_in_first_appearance_order() {
    let html = render(at_rest_digest);
    assert_eq!(count(&html, "class=\"kit-digest__kind\""), 3);
    let lookup = html.find("Lookup").unwrap();
    let compare = html.find("Compare").unwrap();
    let verify = html.find("Verify").unwrap();
    assert!(lookup < compare && compare < verify);
    assert!(html.contains(">7 seconds<"));
    assert!(html.contains(">4 seconds<"));
    assert!(html.contains(">1 second<"));
}

#[test]
fn model_lines_at_rest_show_three_effort_levels_and_no_open_detail() {
    let html = render(at_rest_digest);
    assert_eq!(count(&html, "class=\"kit-digest__model-row\""), 3);
    assert_eq!(count(&html, "class=\"kit-digest__bar-lit\""), 3 + 1 + 4);
    assert_eq!(count(&html, "class=\"kit-digest__detail\""), 0);
    assert!(html.contains("kit-digest__slot"));
}

#[test]
fn disclosures_at_rest_are_closed_and_labelled_with_counts() {
    let html = render(at_rest_digest);
    assert!(html.contains("References (9)"));
    assert!(html.contains("Step log (6)"));
    assert_eq!(count(&html, "class=\"kit-digest__toggle\""), 2);
    assert_eq!(count(&html, "class=\"kit-digest__sources\""), 0);
    assert_eq!(count(&html, "class=\"kit-digest__step\""), 0);
}

#[test]
fn opened_step_log_lists_six_rows_with_the_running_row_untimed() {
    fn app() -> Element {
        rsx! {
            DeliberationDigest {
                step_list: at_rest_steps(),
                kind_table: at_rest_kinds(),
                start_open: true,
            }
        }
    }
    let html = render(app);
    assert_eq!(count(&html, "class=\"kit-digest__step\""), 6);
    assert!(html.contains(">running<"));
}

#[test]
fn finished_digest_header_takes_the_total_form_and_drops_the_live_indicator() {
    fn app() -> Element {
        rsx! {
            DeliberationDigest {
                step_list: at_rest_steps(),
                kind_table: at_rest_kinds(),
                is_reasoning: false,
            }
        }
    }
    let html = render(app);
    assert!(html.contains("Took 12 seconds across 6 steps"));
    assert_eq!(count(&html, "kit-digest__pulse"), 0);
}

#[test]
fn streaming_turn_at_rest_has_question_bubble_and_only_the_stop_control() {
    fn app() -> Element {
        rsx! {
            InterruptibleAnswerStream {
                text: "Rounding moves to one helper.",
                question: Some("How does rounding change?".to_string()),
                status: StreamStatus::Streaming,
                agent: AgentRef { id: "writer".into(), name: "Writer".into(), color: None },
                on_regenerate: move |_| {},
            }
        }
    }
    let html = render(app);
    assert_eq!(count(&html, "class=\"kit-isa__question\""), 1);
    assert!(html.contains("Stop answer"));
    assert!(html.contains("Esc also stops"));
    assert_eq!(count(&html, "Copy answer"), 0);
    assert_eq!(count(&html, "Regenerate answer"), 0);
    assert_eq!(count(&html, "aria-busy=\"true\""), 1);
}

#[test]
fn review_diff_gives_each_line_a_sign_and_keeps_context_lines() {
    assert_eq!(
        diff_lines("-old line\n+new line\n shared"),
        vec![
            ('-', "old line".to_string()),
            ('+', "new line".to_string()),
            (' ', "shared".to_string()),
        ]
    );
}

#[test]
fn review_queue_at_rest_counts_filters_and_rows() {
    fn app() -> Element {
        let item = |id: &str, risk: ReviewRisk, conf: f64, kind: &str| ReviewItem {
            item_id: id.into(),
            item_title: format!("Item {id}"),
            agent_name: "Release agent".into(),
            model_name: None,
            item_kind: kind.into(),
            risk_level: risk,
            confidence_score: conf,
            created_at: 1_000,
            agent_reasoning: None,
            body_heading: None,
            body_text: Some("Body".into()),
            body_format: BodyFormat::Text,
            flags: vec![],
            file_changes: vec![],
            plan_steps: vec![],
            cited_sources: vec![],
        };
        let items = vec![
            item("a", ReviewRisk::High, 0.5, "Plan"),
            item("b", ReviewRisk::High, 0.9, "Edit"),
            item("c", ReviewRisk::Medium, 0.6, "Refund"),
            item("d", ReviewRisk::Low, 0.95, "Message"),
            item("e", ReviewRisk::Low, 0.95, "Message"),
            item("f", ReviewRisk::Low, 0.95, "Message"),
            item("g", ReviewRisk::Low, 0.4, "Message"),
            item("h", ReviewRisk::Medium, 0.99, "Message"),
        ];
        rsx! {
            DecisionReviewQueue { review_items: items, reviewed_before: 24usize, current_time: 2_000 }
        }
    }
    let html = render(app);
    assert!(html.contains("24 of 32 reviewed"));
    assert_eq!(count(&html, "class=\"kit-drq__row\""), 8);
    assert!(html.contains("class=\"kit-drq__count\">8<"));
    assert!(html.contains("class=\"kit-drq__count\">2<"));
    assert!(html.contains("class=\"kit-drq__count\">3<"));
}

#[test]
fn run_control_marks_the_first_queued_message_as_next() {
    fn app() -> Element {
        rsx! {
            RunControlComposer {
                agent_identity: AgentIdentity { agent_id: "itinerary".into(), agent_name: "Itinerary agent".into() },
                request_text: Some("Plan a trip.".to_string()),
                answer_text: Some("Day one.".to_string()),
                run_state: RunState::Running,
                queued_messages: vec![QueuedMessage { queue_id: "q1".into(), message_text: "Add a ferry".into() }],
                context_share: Some(0.42),
            }
        }
    }
    let html = render(app);
    assert_eq!(count(&html, "class=\"kit-rcc__next\""), 1);
    assert!(html.contains("Queued 1"));
    assert!(html.contains("Stop</button>") || html.contains("kit-rcc__stop"));
    assert!(html.contains("42%"));
    assert!(html.contains("Itinerary agent"));
}

#[test]
fn working_trail_at_rest_is_working_with_no_completion_row() {
    fn app() -> Element {
        rsx! {
            WorkingTrailPanel {
                trail_steps: vec![TrailStep::Thinking { text: "Thinking about it.".into() }],
                auto_finish: false,
            }
        }
    }
    let html = render(app);
    assert!(html.contains("kit-trail__status\">Working<"));
    assert_eq!(count(&html, "class=\"kit-trail__done\""), 0);
}

#[test]
fn working_trail_at_rest_marks_the_active_step_and_only_offers_the_header_and_more_controls() {
    fn app() -> Element {
        rsx! {
            WorkingTrailPanel {
                trail_steps: vec![
                    TrailStep::Thinking { text: "Reading the question.".into() },
                    TrailStep::Search {
                        query_label: "pricing changes".into(),
                        sources: vec![
                            TrailSource { source_label: "Pricing page".into(), source_href: None, source_mark: None },
                            TrailSource { source_label: "Changelog".into(), source_href: None, source_mark: None },
                        ],
                        hidden_source_count: 1,
                    },
                    TrailStep::Thinking { text: "Comparing tiers.".into() },
                    TrailStep::Thinking { text: "Summarising changes.".into() },
                ],
                step_pause: 0,
                auto_finish: false,
            }
        }
    }
    let html = render(app);
    assert!(html.contains("kit-trail__status\">Working<"));
    assert_eq!(count(&html, "class=\"kit-trail__step is-active\""), 1);
    assert_eq!(count(&html, "class=\"kit-trail__done\""), 0);
    assert_eq!(count(&html, "<input"), 0);
    assert_eq!(count(&html, "<button"), 2);
    assert!(html.contains("1 more source"));
}

#[test]
fn working_trail_with_five_steps_folds_all_but_the_active_step_and_three_before_it() {
    fn app() -> Element {
        rsx! {
            WorkingTrailPanel {
                trail_steps: (1..=5)
                    .map(|n| TrailStep::Thinking { text: format!("Thought {n}") })
                    .collect::<Vec<_>>(),
                step_pause: 0,
                auto_finish: false,
            }
        }
    }
    let html = render(app);
    assert_eq!(count(&html, "class=\"kit-trail__node\""), 4);
    assert!(!html.contains("Thought 1<"));
    assert!(html.contains("Thought 2<"));
    assert!(html.contains("Thought 5<"));
    assert!(html.contains("Show 1 earlier step"));
}

#[test]
fn rail_start_keeps_four_steps_until_earlier_steps_are_unfolded() {
    assert_eq!(rail_start(4, false), 0);
    assert_eq!(rail_start(5, false), 1);
    assert_eq!(rail_start(9, false), 5);
    assert_eq!(rail_start(9, true), 0);
}

#[test]
fn fill_in_footer_states_how_many_blanks_are_filled() {
    fn app() -> Element {
        let mut specs = BTreeMap::new();
        specs.insert(
            "tone".to_string(),
            rv2_kit::components::fill_in_prompt_templates::BlankSpec {
                hint_text: Some("Tone".into()),
                fallback_value: Some("friendly".into()),
            },
        );
        rsx! {
            FillInPromptTemplates {
                template_list: vec![Template {
                    template_id: "t1".into(),
                    template_title: "Client summary".into(),
                    group_label: Some("Sales".into()),
                    group_key: Some("sales".into()),
                    template_body: "Summarise {client} for {account} in a {tone} tone, covering {topic}.".into(),
                    blank_specs: specs,
                }],
            }
        }
    }
    let html = render(app);
    assert!(html.contains("1 of 4 filled"));
    assert_eq!(count(&html, "class=\"kit-fpt__blank\""), 4);
}

#[test]
fn unfilled_blank_stays_visible_by_name_in_the_raw_prompt() {
    let body = "Draft a follow-up to {client} about {topic}.";
    let mut values = BTreeMap::new();
    values.insert("topic".to_string(), "pricing".to_string());
    assert_eq!(
        render_prompt(body, &values),
        "Draft a follow-up to {client} about pricing."
    );
}
