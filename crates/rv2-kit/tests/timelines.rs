use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;
use rv2_kit::components::annotation_sidebar::{
    AnnotationSidebar, Note, NoteFilter, NoteSort, NoteState, Writer, filter_counts, is_patch,
    relative_label, visible_notes,
};
use rv2_kit::components::background_jobs_popover::{
    BackgroundJob, BackgroundJobsPopover, JobCounts, JobState, NavKey, job_counts, next_index,
    progress as job_progress, ring_path, summary_text,
};
use rv2_kit::components::change_log_timeline::{
    Change, ChangeLogTimeline, ChangeWriter, KindStyle, KindTone, MessagePart, RowKey, flatten,
    format_utc, kind_label, kind_tone, looks_like_file, message_parts, next_row, relative_time,
};
use rv2_kit::components::change_replay::{
    ChangeEvent, ChangeReplay, ReplayAuthor, ReplayKey, clock, compact, default_axis, file_rows,
    scale_max, shown_events, split_path, step_index, totals,
};
use rv2_kit::components::deliberation_digest::{
    DeliberationDigest, DigestStep, Effort, Focus, builtin_kinds, format_count, format_duration,
    header_text, is_related, kind_totals, merged_kinds, segment_widths, total_time,
};
use rv2_kit::components::expandable_work_item_card::{
    ExpandableWorkItemCard, Subtask, default_footer_hint, initials, member_matches, progress,
    ring_segments, toggle_subtask,
};
use rv2_kit::components::inter_agent_message_feed::{
    AttachmentKind, Entry, EntryKind, FeedKey, InterAgentMessageFeed, Participant, active_mention,
    attachment_kind, complete_mention, is_awaiting_you, mention_candidates, next_entry,
    parse_recipients, quote_text, visible_indices,
};
use rv2_kit::components::quota_load_scheduler::{
    CurveEmphasis, DayKind, LoadStatus, PlannedRun, QuotaLoadScheduler, RunDays, active_at,
    clamp_start, clock as quota_clock, combined_curve, is_locked, move_run, overflow_path, peak_of,
    run_load, spread, status,
};
use rv2_kit::components::schedule_variance_timeline::{
    PlanStep, ScheduleVarianceTimeline, StepState, links_for, plan, project, signed, variance_words,
};
use rv2_kit::components::session_packing_planner::{
    ScheduledTask, SessionPackingPlanner, Slot, place, refusal, split_usage,
};
use rv2_kit::components::task_assignment_board::{
    TaskAssignmentBoard, WorkItem, Worker, assign, busy_until, least_busy, next_auto, queue_for,
    queued_cost, unassigned,
};
use rv2_kit::components::working_trail_panel::{
    TrailStep, WorkingTrailPanel, elapsed_label, visible_source_count,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

fn writer(name: &str) -> Writer {
    Writer {
        writer_name: name.to_string(),
        writer_mark: None,
    }
}

fn note(id: &str, text: &str, at: i64) -> Note {
    Note {
        note_id: id.to_string(),
        writer: writer("Ada"),
        posted_at: at,
        note_text: text.to_string(),
        note_state: NoteState::Open,
    }
}

#[test]
fn only_slash_patch_notes_are_patches() {
    assert!(is_patch("/patch rename heading"));
    assert!(!is_patch("please /patch later"));
}

#[test]
fn patches_filter_and_counts_split_by_prefix() {
    let notes = vec![note("a", "/patch x", 1), note("b", "why?", 2)];
    assert_eq!(filter_counts(&notes), (2, 1, 1));
    let shown = visible_notes(
        &notes,
        NoteFilter::Patches,
        NoteSort::Newest,
        &BTreeMap::new(),
        false,
    );
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].note_id, "a");
}

#[test]
fn newest_first_orders_by_time_and_hide_resolved_drops_them() {
    let notes = vec![note("old", "a", 1), note("new", "b", 9)];
    let mut overrides = BTreeMap::new();
    overrides.insert("old".to_string(), true);
    let newest = visible_notes(&notes, NoteFilter::All, NoteSort::Newest, &overrides, false);
    assert_eq!(newest[0].note_id, "new");
    let hidden = visible_notes(&notes, NoteFilter::All, NoteSort::Newest, &overrides, true);
    assert_eq!(hidden.len(), 1);
}

#[test]
fn relative_label_picks_a_unit() {
    assert_eq!(relative_label(1000, 1000 - 30), "just now");
    assert_eq!(relative_label(1000, 1000 - 600), "10 min ago");
    assert_eq!(relative_label(100_000, 0), "1 d ago");
}

#[test]
fn annotation_sidebar_renders_empty_line_and_no_items() {
    fn app() -> Element {
        rsx! { AnnotationSidebar { acting_writer: writer("Me") } }
    }
    let html = render(app);
    assert!(html.contains("No notes yet"));
    assert!(!html.contains("data-note-id"));
}

#[test]
fn annotation_sidebar_marks_only_patch_notes_runnable() {
    fn app() -> Element {
        let notes = vec![note("p", "/patch fix", 1), note("q", "hello", 2)];
        rsx! { AnnotationSidebar { notes, acting_writer: writer("Me"), now_secs: 10 } }
    }
    let html = render(app);
    assert_eq!(html.matches("Apply patch").count(), 1);
    assert!(html.contains("<time"));
}

#[test]
fn memory_summary_counts_paused_only_when_present() {
    use rv2_kit::components::assistant_memory_manager::{Memory, summary_line};
    let mk = |id: &str, paused: bool| Memory {
        memory_id: id.to_string(),
        memory_text: id.to_string(),
        memory_group: "g".to_string(),
        source_label: None,
        date_label: None,
        is_paused: paused,
    };
    assert_eq!(
        summary_line(&[mk("a", false), mk("b", false)]),
        "2 memories"
    );
    assert_eq!(
        summary_line(&[mk("a", true), mk("b", false)]),
        "2 memories · 1 paused"
    );
}

#[test]
fn paused_jobs_are_not_counted_as_running() {
    let jobs = vec![
        BackgroundJob {
            job_id: "1".into(),
            job_title: "a".into(),
            current_step: "s".into(),
            steps_done: 1,
            steps_total: 2,
            job_state: JobState::Running,
        },
        BackgroundJob {
            job_id: "2".into(),
            job_title: "b".into(),
            current_step: "s".into(),
            steps_done: 1,
            steps_total: 2,
            job_state: JobState::Running,
        },
        BackgroundJob {
            job_id: "3".into(),
            job_title: "c".into(),
            current_step: "s".into(),
            steps_done: 0,
            steps_total: 2,
            job_state: JobState::NeedsYou,
        },
        BackgroundJob {
            job_id: "4".into(),
            job_title: "d".into(),
            current_step: "s".into(),
            steps_done: 0,
            steps_total: 2,
            job_state: JobState::Paused,
        },
    ];
    let counts = job_counts(&jobs);
    assert_eq!(
        counts,
        JobCounts {
            running: 2,
            needs_you: 1,
            paused: 1
        }
    );
    let text = summary_text(counts, "running", "needs you", "paused");
    assert_eq!(text.as_deref(), Some("2 running, 1 needs you"));
}

#[test]
fn job_progress_is_zero_without_steps_and_ring_is_empty() {
    assert_eq!(job_progress(0, 0), 0.0);
    assert!(ring_path(0.0).is_empty());
    assert!(close(job_progress(1, 4), 0.25));
}

#[test]
fn job_keyboard_moves_clamp_and_wrap_at_edges() {
    assert_eq!(next_index(3, None, NavKey::Down), Some(0));
    assert_eq!(next_index(3, Some(2), NavKey::Down), Some(2));
    assert_eq!(next_index(3, Some(0), NavKey::Up), Some(0));
    assert_eq!(next_index(3, None, NavKey::End), Some(2));
    assert_eq!(next_index(0, None, NavKey::Home), None);
}

#[test]
fn background_jobs_trigger_has_full_accessible_summary() {
    fn app() -> Element {
        let jobs = vec![BackgroundJob {
            job_id: "1".into(),
            job_title: "Index".into(),
            current_step: "scan".into(),
            steps_done: 1,
            steps_total: 4,
            job_state: JobState::Running,
        }];
        rsx! { BackgroundJobsPopover { job_list: jobs } }
    }
    let html = render(app);
    assert!(html.contains("Background tasks: 1 running"));
    assert!(html.contains("aria-expanded=\"false\""));
}

#[test]
fn backticked_names_become_chips_and_extension_decides_file() {
    assert!(looks_like_file("lib.rs"));
    assert!(!looks_like_file("feature/login"));
    let parts = message_parts("touch `lib.rs` on `main`");
    assert_eq!(
        parts,
        vec![
            MessagePart::Text("touch ".into()),
            MessagePart::Name {
                name: "lib.rs".into(),
                is_file: true
            },
            MessagePart::Text(" on ".into()),
            MessagePart::Name {
                name: "main".into(),
                is_file: false
            },
        ]
    );
}

#[test]
fn unconfigured_kind_renders_neutral() {
    let overrides: BTreeMap<String, KindStyle> = BTreeMap::new();
    assert_eq!(kind_tone("mystery", &overrides), KindTone::Neutral);
    assert_eq!(kind_tone("speed", &overrides), KindTone::BuiltIn);
    assert_eq!(kind_label("new_capability", &overrides), "New capability");
}

#[test]
fn change_log_relative_time_and_utc_format() {
    assert_eq!(relative_time(1000, 1000 - 30), "just now");
    assert_eq!(relative_time(10_000, 10_000 - 7_200), "2 h ago");
    assert_eq!(format_utc(0), "1970-01-01 00:00 UTC");
}

fn change(id: &str, msg: &str, merged: Vec<Change>, in_progress: bool) -> Change {
    Change {
        change_id: id.to_string(),
        change_message: msg.to_string(),
        change_author: ChangeWriter {
            writer_name: "Ada".into(),
            is_agent: false,
            lane_color: None,
            writer_mark: None,
        },
        co_author: None,
        changed_at: None,
        change_kind: None,
        file_count: None,
        lines_added: None,
        lines_removed: None,
        change_body: None,
        merged_changes: merged,
        in_progress,
    }
}

#[test]
fn merge_folds_its_changes_until_expanded() {
    let changes = vec![change(
        "m",
        "Merge",
        vec![change("c1", "a", vec![], false)],
        false,
    )];
    let folded = flatten(&changes, &BTreeSet::new());
    assert_eq!(folded.len(), 1);
    let mut open = BTreeSet::new();
    open.insert("m".to_string());
    let expanded = flatten(&changes, &open);
    assert_eq!(expanded.len(), 2);
    assert_eq!(expanded[1].depth, 1);
}

#[test]
fn change_keyboard_moves_clamp() {
    assert_eq!(next_row(3, Some(1), RowKey::Up), Some(0));
    assert_eq!(next_row(3, Some(2), RowKey::Down), Some(2));
    assert_eq!(next_row(3, None, RowKey::End), Some(2));
}

#[test]
fn change_log_renders_in_progress_as_non_button() {
    fn app() -> Element {
        let changes = vec![change("w", "Writing the thing", vec![], true)];
        rsx! { ChangeLogTimeline { change_list: changes, current_branch: "main" } }
    }
    let html = render(app);
    assert!(html.contains("Writing…"));
    assert!(html.contains("kit-log__row is-progress"));
}

fn event(at: f64, file: &str, add: u32, rem: u32) -> ChangeEvent {
    ChangeEvent {
        at_offset: at,
        file_path: file.to_string(),
        lines_added: add,
        lines_removed: rem,
        line_details: vec![],
    }
}

#[test]
fn replay_totals_follow_shown_offset_and_net_is_difference() {
    let events = vec![event(10.0, "a.rs", 5, 1), event(70.0, "b.rs", 2, 4)];
    let all = totals(&events, None);
    assert_eq!((all.added, all.removed, all.net, all.files), (7, 5, 2, 2));
    let early = totals(&events, Some(30.0));
    assert_eq!((early.added, early.removed, early.files), (5, 1, 1));
    assert_eq!(shown_events(&events, Some(30.0)).len(), 1);
}

#[test]
fn replay_file_row_appears_at_first_event_and_keeps_position() {
    let events = vec![
        event(1.0, "a.rs", 1, 0),
        event(2.0, "b.rs", 1, 0),
        event(3.0, "a.rs", 1, 0),
    ];
    let rows = file_rows(&events, None);
    assert_eq!(
        rows.iter().map(|r| r.path.as_str()).collect::<Vec<_>>(),
        vec!["a.rs", "b.rs"]
    );
    assert_eq!(file_rows(&events, Some(1.5)).len(), 1);
}

#[test]
fn replay_axis_rounds_to_minute_and_scale_covers_all_events() {
    let events = vec![event(61.0, "a", 3, 9)];
    assert!(close(default_axis(&events), 120.0));
    assert!(close(scale_max(&events), 9.0));
}

#[test]
fn replay_arrows_step_and_compact_formats() {
    assert_eq!(step_index(3, None, ReplayKey::Right), Some(0));
    assert_eq!(step_index(3, Some(2), ReplayKey::Right), Some(2));
    assert_eq!(compact(4800), "4.8k");
    assert_eq!(clock(75.0), "1:15");
    assert_eq!(
        split_path("src/lib.rs"),
        ("src/".to_string(), "lib.rs".to_string())
    );
}

#[test]
fn change_replay_renders_status_for_in_progress() {
    fn app() -> Element {
        let events = vec![event(5.0, "src/main.rs", 2, 1)];
        rsx! {
            ChangeReplay {
                author: ReplayAuthor { author_id: "agent-1".into(), author_name: "Builder".into() },
                change_events: events,
                in_progress: true,
            }
        }
    }
    let html = render(app);
    assert!(html.contains("Editing src/main.rs"));
}

fn dstep(id: &str, kind: &str, secs: Option<f64>) -> DigestStep {
    DigestStep {
        step_id: id.to_string(),
        kind_key: kind.to_string(),
        step_label: id.to_string(),
        elapsed_time: secs,
        is_running: secs.is_none(),
        model_id: None,
    }
}

#[test]
fn digest_segments_are_proportional_and_bad_times_count_zero() {
    let steps = vec![
        dstep("a", "read", Some(2.0)),
        dstep("b", "write", Some(f64::NAN)),
        dstep("c", "tool", Some(-4.0)),
        dstep("d", "check", Some(6.0)),
    ];
    assert!(close(total_time(&steps), 8.0));
    let (widths, remainder) = segment_widths(&steps, None);
    assert!(close(widths[0], 25.0));
    assert!(close(widths[3], 75.0));
    assert!(close(remainder, 0.0));
}

#[test]
fn digest_header_reads_reasoning_then_summary() {
    assert_eq!(header_text(true, 0.0, 0), "Reasoning");
    assert_eq!(
        header_text(false, 11.0, 6),
        "Reasoned for 11 seconds, 6 steps"
    );
    assert_eq!(format_duration(60.0), "1 minute");
}

#[test]
fn digest_kind_totals_keep_first_appearance_and_unknown_uses_key() {
    let table = merged_kinds(&BTreeMap::new());
    assert!(table.contains_key("reasoning") && builtin_kinds().contains_key("read"));
    let steps = vec![
        dstep("a", "zebra", Some(1.0)),
        dstep("b", "read", Some(2.0)),
        dstep("c", "zebra", Some(3.0)),
    ];
    let totals = kind_totals(&steps, &table);
    assert_eq!(totals[0].key, "zebra");
    assert_eq!(totals[0].label, "zebra");
    assert!(close(totals[0].total, 4.0));
    assert_eq!(totals[0].count, 2);
}

#[test]
fn digest_focus_highlights_matching_steps_only() {
    let step = dstep("a", "read", Some(1.0));
    assert!(is_related(&step, &Focus::Kind("read".into())));
    assert!(!is_related(&step, &Focus::Kind("write".into())));
    assert_eq!(format_count(1234), "1,234");
    assert_eq!(format_count(12_400), "12.4k");
    assert_eq!(Effort::High.bars(), 3);
}

#[test]
fn digest_renders_reasoning_header_while_running() {
    fn app() -> Element {
        rsx! { DeliberationDigest { step_list: vec![], is_reasoning: true } }
    }
    assert!(render(app).contains("Reasoning"));
}

fn sub(id: &str, done: bool, who: Option<&str>) -> Subtask {
    Subtask {
        subtask_id: id.to_string(),
        subtask_text: id.to_string(),
        is_done: done,
        assignee_name: who.map(str::to_string),
    }
}

#[test]
fn work_item_progress_toggle_and_member_match() {
    let list = vec![
        sub("a", true, Some("Ada")),
        sub("b", false, Some("Bo")),
        sub("c", false, None),
    ];
    assert_eq!(progress(&list), (1, 3));
    let flipped = toggle_subtask(&list, "b");
    assert!(flipped[1].is_done);
    assert!(member_matches(&list[0], Some("Ada")));
    assert!(!member_matches(&list[1], Some("Ada")));
    assert!(member_matches(&list[2], None));
    assert_eq!(default_footer_hint(1, 3), "2 open");
    assert_eq!(default_footer_hint(3, 3), "all done");
    assert_eq!(initials("ada lovelace"), "AL");
}

#[test]
fn work_item_ring_has_one_segment_per_subtask() {
    let list = vec![
        sub("a", true, None),
        sub("b", false, None),
        sub("c", false, None),
    ];
    assert_eq!(ring_segments(&list).len(), 3);
    assert!(ring_segments(&list)[0].1);
}

#[test]
fn work_item_closed_card_renders_header_only() {
    fn app() -> Element {
        rsx! { ExpandableWorkItemCard { item_title: "Launch", subtask_list: vec![sub("a", false, None)] } }
    }
    let html = render(app);
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(!html.contains("kit-item__subtask"));
}

fn participant(id: &str, name: &str) -> Participant {
    Participant {
        participant_id: id.to_string(),
        display_name: name.to_string(),
        model_name: None,
        badge_mark: None,
        logo: None,
    }
}

fn entry(id: &str, sender: &str, to: &[&str], kind: EntryKind) -> Entry {
    Entry {
        entry_id: id.to_string(),
        sender_id: sender.to_string(),
        recipient_ids: to.iter().map(|s| s.to_string()).collect(),
        entry_kind: kind,
        body_text: format!("body {id}"),
        sent_at: None,
        answers_id: None,
        is_drafting: false,
        attachment_list: vec![],
        detail_card: None,
    }
}

#[test]
fn needs_you_point_stays_awaiting_until_answered() {
    let mut entries = vec![entry("q", "p1", &["you"], EntryKind::NeedsPerson)];
    assert!(is_awaiting_you(&entries, 0));
    let mut answer = entry("r", "you", &["p1"], EntryKind::Answer);
    answer.answers_id = Some("q".into());
    entries.push(answer);
    assert!(!is_awaiting_you(&entries, 0));
}

#[test]
fn mention_picks_recipient_and_completes_token() {
    let people = vec![participant("ada", "Ada Lovelace"), participant("bo", "Bo")];
    assert_eq!(active_mention("hi @ad"), Some("ad".to_string()));
    assert_eq!(mention_candidates(&people, "ad").len(), 1);
    assert_eq!(
        parse_recipients("@AdaLovelace check this", &people, None),
        vec!["ada".to_string()]
    );
    assert_eq!(
        parse_recipients("plain", &people, None),
        vec!["all".to_string()]
    );
    assert_eq!(
        parse_recipients("plain", &people, Some("bo")),
        vec!["bo".to_string()]
    );
    assert_eq!(complete_mention("hi @ad", &people[0]), "hi @AdaLovelace ");
}

#[test]
fn feed_participant_filter_and_roving_keys() {
    let entries = vec![
        entry("1", "ada", &["bo"], EntryKind::TaskRequest),
        entry("2", "cy", &["all"], EntryKind::Broadcast),
    ];
    assert_eq!(visible_indices(&entries, Some("bo")), vec![0, 1]);
    assert_eq!(visible_indices(&entries, Some("ada")), vec![0, 1]);
    assert_eq!(next_entry(2, 0, FeedKey::Down), Some(1));
    assert_eq!(next_entry(0, 0, FeedKey::Down), None);
}

#[test]
fn feed_attachment_kind_from_label() {
    assert_eq!(attachment_kind("src/", None), AttachmentKind::Folder);
    assert_eq!(attachment_kind("data.csv", None), AttachmentKind::Table);
    assert_eq!(attachment_kind("notes.md", None), AttachmentKind::File);
    assert_eq!(quote_text("abcdef", 3), "abc…");
}

#[test]
fn feed_renders_awaiting_tag() {
    fn app() -> Element {
        let people = vec![participant("p1", "Planner")];
        let entries = vec![entry("q", "p1", &["you"], EntryKind::NeedsPerson)];
        rsx! { InterAgentMessageFeed { participants: people, entries, feed_name: "Run" } }
    }
    assert!(render(app).contains("Awaiting you"));
}

fn run(id: &str, start: f64, peak: f64) -> PlannedRun {
    PlannedRun {
        run_id: id.to_string(),
        run_name: id.to_string(),
        start_minute: start,
        peak_tokens_per_min: peak,
        rise_minutes: 10.0,
        tail_half_life_minutes: 20.0,
        run_days: RunDays::Daily,
        curve_emphasis: CurveEmphasis::Normal,
    }
}

#[test]
fn quota_load_rises_then_halves_each_half_life() {
    let r = run("a", 100.0, 1000.0);
    assert!(close(run_load(&r, 100.0), 0.0));
    assert!(close(run_load(&r, 110.0), 1000.0));
    assert!(close(run_load(&r, 130.0), 500.0));
    assert!(close(run_load(&run("n", 0.0, f64::NAN), 5.0), 0.0));
}

#[test]
fn quota_status_thresholds_and_overflow() {
    assert_eq!(status(900.0, 1000.0, 0.75), LoadStatus::Close);
    assert_eq!(status(1200.0, 1000.0, 0.75), LoadStatus::Over);
    assert_eq!(status(500.0, 1000.0, 0.75), LoadStatus::Under);
    let curve = vec![0.0, 2000.0, 0.0];
    assert!(!overflow_path(&curve, 1000.0, 2000.0, 40.0).is_empty());
}

#[test]
fn quota_locked_runs_cannot_move_and_spread_never_raises_peak() {
    let runs = vec![run("early", 60.0, 1000.0), run("late", 600.0, 1000.0)];
    assert!(move_run(&runs, "early", 300.0, Some(120.0)).is_none());
    assert!(is_locked(&runs[0], Some(120.0)));
    let before = peak_of(&combined_curve(&runs, DayKind::Weekdays)).1;
    let starts = spread(&runs, DayKind::Weekdays, Some(0.0));
    let moved: Vec<PlannedRun> = runs
        .iter()
        .zip(starts.iter())
        .map(|(r, s)| PlannedRun {
            start_minute: *s,
            ..r.clone()
        })
        .collect();
    let after = peak_of(&combined_curve(&moved, DayKind::Weekdays)).1;
    assert!(after <= before + 1e-9);
}

#[test]
fn running_step_variance_is_projected_finish_against_plan() {
    let step = |start: Option<f64>| PlanStep {
        step_key: "s".into(),
        step_label: "Step".into(),
        planned_span: 10.0,
        waits_for: vec![],
        actual_start: start,
        actual_end: None,
        step_mark: None,
    };
    let early = project(&[step(Some(0.0))], 3.0);
    assert_eq!(early[0].state, StepState::Running);
    assert!(close(early[0].variance, 0.0), "{}", early[0].variance);
    let late = project(&[step(Some(0.0))], 12.0);
    assert!(close(late[0].variance, 2.0), "{}", late[0].variance);
}

#[test]
fn quota_weekend_runs_excluded_from_weekday_curve() {
    let mut weekend = run("w", 0.0, 1000.0);
    weekend.run_days = RunDays::Weekend;
    let curve = combined_curve(&[weekend], DayKind::Weekdays);
    assert!(curve.iter().all(|v| *v == 0.0));
    assert_eq!(quota_clock(65.0), "01:05");
    assert!(close(clamp_start(-5.0), 0.0));
    assert!(active_at(&[run("x", 0.0, 1.0)], 5.0).contains(&"x".to_string()));
}

#[test]
fn quota_renders_status_summary() {
    fn app() -> Element {
        rsx! { QuotaLoadScheduler { starting_runs: vec![run("nightly", 540.0, 5000.0)], provider_ceiling_per_min: 80000.0 } }
    }
    let html = render(app);
    assert!(html.contains("Under limit"));
    assert!(html.contains("nightly"));
}

fn pstep(key: &str, span: f64, waits: &[&str], start: Option<f64>, end: Option<f64>) -> PlanStep {
    PlanStep {
        step_key: key.to_string(),
        step_label: key.to_string(),
        planned_span: span,
        waits_for: waits.iter().map(|s| s.to_string()).collect(),
        actual_start: start,
        actual_end: end,
        step_mark: None,
    }
}

#[test]
fn waiting_step_has_no_bar_and_running_grows_with_clock() {
    let steps = vec![
        pstep("a", 10.0, &[], Some(0.0), Some(4.0)),
        pstep("b", 10.0, &["a"], None, None),
    ];
    let rows = project(&steps, 6.0);
    assert_eq!(rows[0].state, StepState::Done);
    assert_eq!(rows[1].state, StepState::Running);
    assert!(close(rows[1].bar_end - rows[1].start, 2.0));
    let waiting = project(&steps, 2.0);
    assert_eq!(waiting[1].state, StepState::Waiting);
}

#[test]
fn variance_words_and_signed_figures() {
    assert_eq!(variance_words(2.0), "two seconds behind plan");
    assert_eq!(variance_words(-1.0), "one second ahead of plan");
    assert_eq!(signed(2.0), "+2");
    assert_eq!(signed(-1.0), "−1");
    assert_eq!(variance_words(0.0), "on time");
}

#[test]
fn plan_starts_dependents_after_their_dependency_and_links_show_both_ways() {
    let steps = vec![
        pstep("a", 5.0, &[], None, None),
        pstep("b", 3.0, &["a"], None, None),
    ];
    let planned = plan(&steps);
    assert!(close(planned[0].0, 0.0));
    assert!(close(planned[1].0, 5.0));
    assert_eq!(links_for(&steps, "a").len(), 1);
    assert_eq!(links_for(&steps, "b").len(), 1);
}

#[test]
fn variance_timeline_renders_rows_with_state() {
    fn app() -> Element {
        rsx! { ScheduleVarianceTimeline { step_list: vec![pstep("fetch", 4.0, &[], Some(0.0), Some(6.0))], clock_point: 6.0 } }
    }
    let html = render(app);
    assert!(html.contains("fetch"));
    assert!(html.contains("done"));
}

fn task(id: &str, usage: f64) -> ScheduledTask {
    ScheduledTask {
        task_id: id.to_string(),
        task_title: id.to_string(),
        usage_share: usage,
        agent_name: None,
        task_detail: None,
        agent_logo: None,
        pinned_slot: None,
    }
}

#[test]
fn oversized_task_splits_and_parts_add_back() {
    let parts = split_usage(2.5, 1.0, 0.5);
    assert_eq!(parts.len(), 3);
    assert!(close(parts.iter().sum::<f64>(), 2.5));
    assert!(parts.iter().all(|p| *p >= 0.5 - 1e-9));
}

#[test]
fn placement_skips_elapsed_sessions_and_reports_unplaced() {
    let tasks = vec![task("big", 2.0), task("huge", 10.0)];
    let current = Some(Slot {
        day_index: 0,
        session_index: 1,
    });
    let (parts, unplaced) = place(&tasks, &[], (1, 2), 1.0, 0.1, current);
    assert!(parts.iter().all(|p| p.session_index >= 1));
    assert_eq!(unplaced, vec!["big".to_string(), "huge".to_string()]);
}

#[test]
fn refusal_names_reason_for_full_or_elapsed_session() {
    let slot = Slot {
        day_index: 0,
        session_index: 0,
    };
    assert!(refusal(slot, 0.5, &[], 1.0, None).is_none());
    assert!(
        refusal(slot, 2.0, &[], 1.0, None)
            .unwrap()
            .contains("Not enough room")
    );
    let later = Slot {
        day_index: 0,
        session_index: 2,
    };
    assert!(
        refusal(slot, 0.1, &[], 1.0, Some(later))
            .unwrap()
            .contains("already run")
    );
}

#[test]
fn packing_renders_grid_headers() {
    fn app() -> Element {
        rsx! { SessionPackingPlanner { task_queue: vec![task("t", 0.2)], day_headers: vec!["Mon".into()], row_headers: vec!["09:00".into()] } }
    }
    let html = render(app);
    assert!(html.contains("Mon"));
    assert!(html.contains("09:00"));
}

fn worker(id: &str, busy: f64, rate: f64) -> Worker {
    Worker {
        worker_id: id.to_string(),
        worker_name: id.to_string(),
        worker_model_name: None,
        worker_avatar: None,
        busy_minutes: busy,
        cost_per_minute: rate,
    }
}

fn item(id: &str, est: f64, who: Option<&str>) -> WorkItem {
    WorkItem {
        work_id: id.to_string(),
        work_title: id.to_string(),
        estimate_minutes: est,
        required_tools: vec![],
        assigned_worker_id: who.map(str::to_string),
    }
}

#[test]
fn busy_until_adds_busy_and_queue_estimates() {
    let w = worker("a", 10.0, 0.5);
    let items = vec![item("x", 5.0, Some("a")), item("y", 7.0, Some("a"))];
    let queue = queue_for(&items, "a");
    assert!(close(busy_until(100.0, &w, &queue), 122.0));
    assert!(close(queued_cost(&w, &queue), 11.0));
}

#[test]
fn automatic_assigns_first_unassigned_to_least_busy() {
    let workers = vec![worker("a", 30.0, 1.0), worker("b", 5.0, 1.0)];
    let items = vec![item("x", 3.0, None), item("y", 2.0, Some("a"))];
    assert_eq!(least_busy(&workers, &items), Some("b".to_string()));
    assert_eq!(
        next_auto(&workers, &items),
        Some(("x".to_string(), "b".to_string()))
    );
    let assigned = assign(&items, "x", Some("b"));
    assert_eq!(unassigned(&assigned, &workers).len(), 0);
}

#[test]
fn board_renders_overflow_chip_after_three_items() {
    fn app() -> Element {
        let workers = vec![worker("a", 0.0, 1.0)];
        let items = (0..5)
            .map(|i| item(&format!("i{i}"), 1.0, Some("a")))
            .collect();
        rsx! { TaskAssignmentBoard { worker_list: workers, work_items: items } }
    }
    let html = render(app);
    assert!(html.contains("+2"));
}

#[test]
fn elapsed_label_and_source_reveal_count() {
    assert_eq!(elapsed_label(7_000), "0:07");
    assert_eq!(visible_source_count(5, 2, false), 3);
    assert_eq!(visible_source_count(5, 2, true), 5);
}

#[test]
fn trail_renders_thinking_steps_and_timer() {
    fn app() -> Element {
        rsx! { WorkingTrailPanel { trail_steps: vec![TrailStep::Thinking { text: "Checking".into() }], step_pause: 0 } }
    }
    let html = render(app);
    assert!(html.contains("0:00"));
    assert!(html.contains("aria-expanded=\"true\""));
}
