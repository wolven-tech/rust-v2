use dioxus::prelude::*;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

const NOW: u64 = 10_000_000_000;
const MIN: u64 = 60_000;

fn workspace_app() -> Element {
    use rv2_kit::blocks::client_workspace::{
        ClientWorkspace, ItemKind, Reference, StatCard, TimelineItem, WorkspaceInfo,
        WorkspaceOption, WorkspaceSection,
    };
    let stat = |id: &str, label: &str| StatCard {
        id: id.into(),
        label: label.into(),
        value: "€1".into(),
        delta_percent: Some(1.0),
        series: vec![1.0, 2.0, 3.0],
    };
    rsx! {
        ClientWorkspace {
            workspace: WorkspaceInfo {
                id: "acme".into(),
                name: "Larch & Co Ltd".into(),
                meta: vec!["Berlin".into(), "EUR".into()],
                synced_at_ms: NOW - 3 * MIN,
            },
            workspace_options: vec![WorkspaceOption { id: "acme".into(), name: "Larch & Co Ltd".into() }],
            stat_cards: vec![
                stat("receivables", "Receivables"),
                stat("bank", "Bank balance"),
                stat("margin", "Gross margin"),
                stat("burn", "Monthly burn"),
                stat("extra", "Dropped"),
            ],
            sections: vec![
                WorkspaceSection { id: "context".into(), label: "Context".into(), count: None },
                WorkspaceSection { id: "reports".into(), label: "Reports".into(), count: None },
                WorkspaceSection { id: "financials".into(), label: "Financials".into(), count: None },
                WorkspaceSection { id: "settings".into(), label: "Settings".into(), count: None },
            ],
            timeline_items: vec![
                TimelineItem {
                    id: "n1".into(),
                    kind: ItemKind::Note,
                    author: "Ana Ruiz".into(),
                    at_ms: NOW - 40 * MIN,
                    title: None,
                    text: "Bank lines are in.".into(),
                    references: vec![Reference { label: "Q3 cash report".into(), target: "r1".into() }],
                },
                TimelineItem {
                    id: "a1".into(),
                    kind: ItemKind::App,
                    author: String::new(),
                    at_ms: NOW - 7 * 24 * 60 * MIN,
                    title: None,
                    text: "Connected ledger app".into(),
                    references: vec![],
                },
            ],
            kind_options: ItemKind::ALL.to_vec(),
            now_ms: NOW,
        }
    }
}

#[test]
fn client_workspace_at_rest_shows_at_most_four_stat_cards() {
    let html = render(workspace_app);
    assert_eq!(html.matches("class=\"kit-ws__stat\"").count(), 4);
    assert!(!html.contains("Dropped"));
}

#[test]
fn client_workspace_section_tabs_read_in_order_with_context_selected() {
    let html = render(workspace_app);
    let at = |word: &str| {
        html.find(&format!(">{word}<"))
            .unwrap_or_else(|| panic!("{word} tab missing"))
    };
    assert!(at("Context") < at("Reports"));
    assert!(at("Reports") < at("Financials"));
    assert!(at("Financials") < at("Settings"));
    assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
}

#[test]
fn client_workspace_timeline_ends_with_a_connected_app_line_and_a_composer_avatar() {
    let html = render(workspace_app);
    assert!(html.contains("data-kind=\"app\""));
    assert!(html.contains("Connected ledger app"));
    assert!(html.contains("kit-ws__compose-row"));
    assert!(html.contains("kit-ws__app-mark"));
    assert!(html.contains("Switch client"));
    assert!(
        html.contains("Co Ltd"),
        "the workspace name heads the header"
    );
}

fn invoice_app() -> Element {
    use rv2_kit::blocks::invoice_credit_pair::{
        Invoice, InvoiceCreditPair, InvoiceStatus, Line, LineKind, Party,
    };
    let party = |name: &str| Party {
        name: name.into(),
        address: vec!["12 Harbour Road".into()],
        registration_id: None,
    };
    rsx! {
        InvoiceCreditPair {
            invoice: Invoice {
                number: "INV-2107".into(),
                issued_on: "1 Oct 2026".into(),
                due_on: "31 Oct 2026".into(),
                period_from: "1 Sep 2026".into(),
                period_to: "30 Sep 2026".into(),
                currency: "EUR".into(),
                issuer: party("Marlow Works Ltd"),
                recipient: party("Larch & Co Ltd"),
                lines: vec![
                    Line { id: "l1".into(), name: "Team plan".into(), quantity: 1.0, unit_amount: 980.0, tax_percent: 20.0, total: 980.0, kind: LineKind::Fixed },
                    Line { id: "l2".into(), name: "Extra storage".into(), quantity: 4.0, unit_amount: 25.0, tax_percent: 20.0, total: 100.0, kind: LineKind::Fixed },
                    Line { id: "l3".into(), name: "Image renders".into(), quantity: 9640.0, unit_amount: 0.3, tax_percent: 20.0, total: 2892.0, kind: LineKind::Metered },
                ],
                tax_percent: Some(20.0),
                status: InvoiceStatus::Open,
            },
        }
    }
}

#[test]
fn invoice_at_rest_has_totals_a_metered_group_and_no_invoice_status_label() {
    let html = render(invoice_app);
    assert_eq!(html.matches("Metered usage").count(), 1);
    assert_eq!(html.matches("kit-inv__line--metered").count(), 1);
    assert!(html.contains("Subtotal") && html.contains("Tax") && html.contains(">Total<"));
    assert!(html.contains("EUR 4766.40"));
    assert_eq!(
        html.matches("class=\"kit-inv__status\"").count(),
        1,
        "only the credit note carries a status"
    );
    assert!(html.contains("Pay online"));
}

fn metric_app() -> Element {
    use rv2_kit::blocks::metric_overview_board::{MetricOverviewBoard, OverviewSeries, StatMetric};
    let stat = |id: &str, label: &str| StatMetric {
        id: id.into(),
        label: label.into(),
        total: 42.1,
        format_total: |v| format!("{v:.1}"),
        current_series: vec![1.0, 2.0, 3.0, 4.0],
        previous_series: vec![1.0, 1.5, 2.0, 2.5],
        lower_is_better: false,
    };
    rsx! {
        MetricOverviewBoard {
            stat_cards: vec![stat("a", "Revenue"), stat("b", "Sign-ups"), stat("c", "Refunds")],
            overview_series: vec![OverviewSeries { label: "All".into(), values: vec![1.0, 2.0, 3.0] }],
            axis_labels: vec!["1 Nov".into(), "15 Nov".into(), "Today".into()],
            overview_previous: vec![1.0, 1.0, 2.0],
        }
    }
}

#[test]
fn metric_board_at_rest_draws_no_tooltip_and_says_vs_last_period_after_each_badge() {
    let html = render(metric_app);
    assert_eq!(html.matches("kit-metric-card__tooltip").count(), 0);
    assert_eq!(html.matches("vs last period").count(), 3);
}

fn run_app() -> Element {
    use rv2_kit::blocks::run_tree_inspector::{
        CallKind, CallStatus, RunAgent, RunCall, RunInfo, RunStatus, RunTreeInspector,
    };
    rsx! {
        RunTreeInspector {
            run: RunInfo {
                id: "r".into(),
                title: "Ship the spring newsletter".into(),
                status: RunStatus::Running,
                duration_secs: 520.0,
                cost_usd: 0.31,
                token_count: 96_000,
                model_call_count: 4,
                tool_call_count: 6,
            },
            agents: vec![RunAgent {
                id: "planner".into(),
                name: "Planner".into(),
                model_label: "Large model".into(),
                status: RunStatus::Running,
                parent_id: None,
                cost_usd: 0.19,
                elapsed_secs: 520.0,
                current_step: None,
                faded: false,
            }],
            calls: vec![RunCall {
                id: "c1".into(),
                agent_id: "planner".into(),
                kind: CallKind::Model,
                name: "Draft the send plan".into(),
                status: CallStatus::Ok,
                duration_secs: 96.0,
                error_message: None,
            }],
        }
    }
}

#[test]
fn run_tree_at_rest_marks_no_call_as_an_error_and_counts_zero_errors() {
    let html = render(run_app);
    assert_eq!(html.matches("data-status=\"error\"").count(), 0);
    assert!(html.contains("Errors"));
    assert!(
        html.contains("kit-rti__more"),
        "each card footer carries its disclosure count"
    );
    assert!(html.contains("kit-rti__foot"));
}

#[test]
fn run_tree_call_names_stay_on_at_most_two_lines_by_clamp_not_by_letter() {
    let css = include_str!("../assets/run-tree-inspector.css");
    let rule = css
        .split(".kit-rti__call-name {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .expect("call-name rule");
    assert!(rule.contains("-webkit-line-clamp: 2"));
    assert!(!rule.contains("break-all") && !rule.contains("anywhere"));
}

fn timeline_app() -> Element {
    use rv2_kit::blocks::agent_fleet_overview::{AgentStatus, StatusFilter};
    use rv2_kit::blocks::agent_timeline_overview::{
        AgentTimelineOverview, TimelineAgent, TimelineEvent, TimelineSpan,
    };
    let agent = |id: &str, status: AgentStatus| TimelineAgent {
        id: id.into(),
        name: id.into(),
        model_label: "Mid model".into(),
        status,
    };
    let event = |at: u64, status: AgentStatus| TimelineEvent { at_ms: at, status };
    rsx! {
        AgentTimelineOverview {
            agents: vec![
                agent("planner", AgentStatus::Running),
                agent("scout", AgentStatus::Running),
                agent("drafter", AgentStatus::Done),
                agent("publisher", AgentStatus::Waiting),
                agent("support", AgentStatus::Running),
                agent("invoicer", AgentStatus::Failed),
                agent("monitor", AgentStatus::Waiting),
            ],
            spans: vec![TimelineSpan {
                id: "a".into(),
                agent_id: "planner".into(),
                label: "Outline the launch".into(),
                start_at: NOW - 25 * MIN,
                end_at: None,
                status: AgentStatus::Running,
                error_line: None,
            }],
            inbox_items: vec![],
            window_from: NOW - 30 * MIN,
            window_to: NOW,
            now_ms: NOW,
            last_seen_at: NOW - 25 * MIN,
            activity: vec![
                event(NOW - 3 * MIN, AgentStatus::Done),
                event(NOW - 10 * MIN, AgentStatus::Done),
                event(NOW - 8 * MIN, AgentStatus::Done),
                event(NOW - 15 * MIN, AgentStatus::Done),
                event(NOW - 9 * MIN, AgentStatus::Waiting),
                event(NOW - 5 * MIN, AgentStatus::Waiting),
                event(NOW - 6 * MIN, AgentStatus::Failed),
            ],
            status_filter: StatusFilter::All,
        }
    }
}

#[test]
fn timeline_draws_one_lane_per_agent_and_one_curve_dot_per_event() {
    let html = render(timeline_app);
    assert_eq!(html.matches("class=\"kit-tl__lane\"").count(), 7);
    assert_eq!(html.matches("class=\"kit-tl__dot\"").count(), 7);
    assert!(html.contains("4 done"));
    assert!(html.contains("2 waiting"));
    assert!(html.contains("1 failed"));
    assert!(html.contains("Waiting 2"));
    assert!(html.contains("All 7"));
}

#[test]
fn timeline_summary_states_when_the_look_started_and_its_elapsed_time() {
    let html = render(timeline_app);
    assert!(html.contains("Since you looked"));
    assert!(html.contains("Started 25 min ago"));
    assert!(html.contains("kit-tl__pill"));
}

fn glow_empty_app() -> Element {
    use rv2_kit::motion::glow_floor_backdrop::{GlowAnchor, GlowFloorBackdrop, GlowIntensity};
    rsx! {
        GlowFloorBackdrop { intensity: GlowIntensity::Standard, anchor: GlowAnchor::Bottom, {rsx! {}} }
    }
}

#[test]
fn glow_stage_holds_no_content_at_rest() {
    let html = render(glow_empty_app);
    assert!(html.contains("kit-glow__backdrop"));
    assert!(html.contains("kit-glow__content"));
    assert!(!html.contains("<h3") && !html.contains("<button"));
}
