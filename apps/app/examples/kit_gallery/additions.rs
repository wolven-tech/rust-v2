use std::collections::BTreeMap;

use dioxus::prelude::*;

use super::{Item, Section};

#[component]
pub fn Additions() -> Element {
    rsx! {
        Section { id: "additions", title: "New items",
            AgentClashMapDemo {}
            RunProcessLanesDemo {}
            InterruptibleAnswerStreamDemo {}
        }
    }
}

#[component]
fn AgentClashMapDemo() -> Element {
    use rv2_kit::agents::agent_clash_map::{
        AgentClashMap, Clash, ClashWeight, Harness, Participant, Side,
    };
    let mut last = use_signal(String::new);
    let participants = vec![
        Participant {
            id: "ana".into(),
            name: "Ana Cruz".into(),
            note: Some("Invoice rounding, checkout side".into()),
        },
        Participant {
            id: "mira".into(),
            name: "Mira Okafor".into(),
            note: Some("Nightly release candidate".into()),
        },
        Participant {
            id: "dev".into(),
            name: "Dev Patel".into(),
            note: Some("Retry tests for webhooks".into()),
        },
        Participant {
            id: "sol".into(),
            name: "Sol Varga".into(),
            note: Some("Upgrade guide and changelog".into()),
        },
        Participant {
            id: "lee".into(),
            name: "Lee Park".into(),
            note: Some("Billing regression review".into()),
        },
        Participant {
            id: "ivo".into(),
            name: "Ivo Lind".into(),
            note: Some("Ledger schema migration".into()),
        },
        Participant {
            id: "rui".into(),
            name: "Rui Santos".into(),
            note: Some("Refund edge cases".into()),
        },
        Participant {
            id: "yuki".into(),
            name: "Yuki Tanaka".into(),
            note: Some("Webhook retries".into()),
        },
    ];
    let side = |person: &str, agent: &str, harness: Option<&str>| Side {
        participant_id: person.into(),
        agent_name: agent.into(),
        harness_key: harness.map(Into::into),
    };
    let clashes = vec![
        Clash {
            id: "files".into(),
            weight: ClashWeight::Decide,
            label: "Shared files".into(),
            summary: "Both agents are rewriting invoice.rs. One merge will conflict.".into(),
            sides: vec![
                side("ana", "Coder", Some("cli")),
                side("sol", "Docs writer", None),
            ],
        },
        Clash {
            id: "overwrite".into(),
            weight: ClashWeight::Decide,
            label: "Overwrite risk".into(),
            summary: "The ledger migration drops the column the refund fix still writes.".into(),
            sides: vec![
                side("ivo", "Migrator", Some("cli")),
                side("rui", "Fixer", Some("cloud")),
            ],
        },
        Clash {
            id: "refund".into(),
            weight: ClashWeight::Notice,
            label: "Contradiction".into(),
            summary: "The release notes say refunds round down; the new helper rounds to even."
                .into(),
            sides: vec![
                side("mira", "Release bot", Some("cloud")),
                side("lee", "Billing bot", Some("cli")),
            ],
        },
        Clash {
            id: "same-files".into(),
            weight: ClashWeight::Notice,
            label: "Same files".into(),
            summary: "Ana's reviewer and Lee's tester both edit the billing fixtures.".into(),
            sides: vec![
                side("ana", "Reviewer", Some("cli")),
                side("lee", "Tester", Some("cli")),
            ],
        },
        Clash {
            id: "retry".into(),
            weight: ClashWeight::Notice,
            label: "Duplicate fix".into(),
            summary: "Dev's tester and Rui's fixer both patch the webhook retry loop.".into(),
            sides: vec![
                side("dev", "Tester", Some("cli")),
                side("rui", "Fixer", Some("cloud")),
            ],
        },
        Clash {
            id: "plan".into(),
            weight: ClashWeight::Notice,
            label: "Stale plan".into(),
            summary: "Ivo's migration plan predates Lee's fixture change.".into(),
            sides: vec![
                side("ivo", "Migrator", Some("cli")),
                side("lee", "Tester", Some("cli")),
            ],
        },
        Clash {
            id: "guide".into(),
            weight: ClashWeight::Faint,
            label: "Same work".into(),
            summary: "Two agents are writing the same upgrade-guide section.".into(),
            sides: vec![
                side("dev", "Tester", Some("cli")),
                side("sol", "Docs writer", None),
            ],
        },
        Clash {
            id: "tests".into(),
            weight: ClashWeight::Faint,
            label: "Same tests".into(),
            summary: "Both agents are adding fixtures for the refund path.".into(),
            sides: vec![
                side("mira", "Release bot", Some("cloud")),
                side("rui", "Fixer", Some("cloud")),
            ],
        },
    ];
    let mut harnesses: BTreeMap<String, Harness> = BTreeMap::new();
    harnesses.insert(
        "cli".to_string(),
        Harness {
            display_name: Some("Terminal agent".into()),
            mark: Some(">_".into()),
        },
    );
    harnesses.insert(
        "cloud".to_string(),
        Harness {
            display_name: Some("Cloud run".into()),
            mark: Some("C".into()),
        },
    );
    rsx! {
        Item { slug: "agent-clash-map", name: "AgentClashMap",
            AgentClashMap {
                participants,
                clashes,
                harnesses,
                min_height: 340,
                on_select_clash: move |id: Option<String>| {
                    last.set(match id {
                        Some(id) => format!("clash open {id}"),
                        None => "clash closed".to_string(),
                    });
                },
                on_select_participant: move |id: Option<String>| {
                    last.set(match id {
                        Some(id) => format!("agent open for {id}"),
                        None => "agent closed".to_string(),
                    });
                },
            }
            div { class: "gl-out", "data-out": "agent-clash-map", "{last()}" }
        }
    }
}

#[component]
fn RunProcessLanesDemo() -> Element {
    use rv2_kit::agents::run_process_lanes::{
        Lane, LaneKind, RunProcessLanes, RunStatus, Step, StepShape, StepStatus,
    };
    let mut last = use_signal(String::new);
    let lane = |id: &str, name: &str, kind: LaneKind, role: &str| Lane {
        id: id.into(),
        name: name.into(),
        kind,
        role: Some(role.into()),
        mark: None,
        color: None,
        description: None,
    };
    let lanes = vec![
        lane("approver", "Approver", LaneKind::Person, "Approves"),
        lane("planner", "Planner", LaneKind::Agent, "Plans"),
        lane("coder", "Coder", LaneKind::Agent, "Writes"),
        lane("tests", "Test runner", LaneKind::Tool, "CI"),
        lane("release", "Release bot", LaneKind::Agent, "Ships"),
    ];
    let step = |id: &str, lane: &str, title: &str, status: StepStatus| Step {
        id: id.into(),
        lane_id: lane.into(),
        title: title.into(),
        status,
        shape: StepShape::Step,
        after: None,
        duration_label: None,
        detail: None,
    };
    let mut steps = use_signal(|| {
        let mut start = step("start", "approver", "Brief received", StepStatus::Done);
        start.shape = StepShape::Start;
        let mut plan = step("plan", "planner", "Plan the change", StepStatus::Done);
        plan.detail = Some("Moves rounding into one helper and lists the three call sites.".into());
        let mut write = step("write", "coder", "Write the helper", StepStatus::Done);
        write.duration_label = Some("3m 41s".into());
        let mut run = step("run", "tests", "Run tests", StepStatus::Running);
        run.detail = Some("412 of 438 cases passed so far.".into());
        let mut approve = step("approve", "approver", "Approve deploy", StepStatus::Waiting);
        approve.shape = StepShape::Approval;
        approve.after = Some(vec!["run".into()]);
        approve.detail = Some("Covers the rounding change and the changelog line.".into());
        let mut ship = step("ship", "release", "Ship build", StepStatus::Queued);
        ship.after = Some(vec!["approve".into()]);
        let mut legacy = step("legacy", "coder", "Legacy cleanup", StepStatus::Skipped);
        legacy.after = Some(vec!["write".into()]);
        let mut finish = step("finish", "release", "Published", StepStatus::Queued);
        finish.shape = StepShape::Finish;
        finish.after = Some(vec!["ship".into()]);
        vec![start, plan, write, run, approve, ship, legacy, finish]
    });
    let current = steps();
    let run_status = if current.iter().any(|s| s.status == StepStatus::Waiting) {
        RunStatus::Waiting
    } else {
        RunStatus::Running
    };
    rsx! {
        Item { slug: "run-process-lanes", name: "RunProcessLanes",
            RunProcessLanes {
                run_title: "Ship the invoice rounding fix",
                lanes,
                steps: current,
                run_status,
                on_select: move |step: Option<Step>| {
                    last.set(match step {
                        Some(step) => format!("open {}", step.title),
                        None => "closed".to_string(),
                    });
                },
                on_approve: move |step: Step| {
                    steps.with_mut(|list| {
                        for item in list.iter_mut() {
                            if item.id == step.id {
                                item.status = StepStatus::Done;
                            }
                            if item.id == "ship" {
                                item.status = StepStatus::Running;
                            }
                        }
                    });
                    last.set(format!("approved {}", step.title));
                },
                on_reject: move |step: Step| {
                    last.set(format!("rejected {}", step.title));
                },
            }
            div { class: "gl-out", "data-out": "run-process-lanes", "{last()}" }
        }
    }
}

const ANSWER: &str = "Rounding moves to one helper, so every invoice total is computed the same way. The helper rounds half to even at the last step, which matches the ledger export. Tests cover the .5 boundary and negative refunds.";
const QUESTION: &str = "How does rounding change across invoices?";

#[component]
fn InterruptibleAnswerStreamDemo() -> Element {
    use rv2_kit::components::interruptible_answer_stream::{
        AgentRef, InterruptibleAnswerStream, StreamMeta, StreamStatus,
    };
    let mut status = use_signal(|| StreamStatus::Streaming);
    let mut shown = use_signal(|| 0usize);
    let mut released = use_signal(|| false);
    let total = ANSWER.chars().count();
    let hold = ANSWER
        .find("refunds")
        .map_or(total, |byte| ANSWER[..byte].chars().count() + 3);
    let mut last = use_signal(String::new);

    use_hook(move || {
        spawn(async move {
            loop {
                if document::eval(
                    "await new Promise((resolve) => setTimeout(resolve, 90)); return true;",
                )
                .await
                .is_err()
                {
                    break;
                }
                if status() == StreamStatus::Streaming {
                    let limit = if released() { total } else { hold };
                    if shown() < limit {
                        shown.set((shown() + 4).min(limit));
                    } else if released() {
                        status.set(StreamStatus::Done);
                    }
                }
            }
        });
    });

    let text: String = ANSWER.chars().take(shown()).collect();
    let meta = StreamMeta {
        tokens: Some(1_240),
        duration_ms: Some(4_200),
    };
    let agent = AgentRef {
        id: "writer".into(),
        name: "Writer".into(),
        color: None,
    };
    rsx! {
        Item { slug: "interruptible-answer-stream", name: "InterruptibleAnswerStream",
            InterruptibleAnswerStream {
                text,
                question: QUESTION.to_string(),
                status: status(),
                agent,
                meta: Some(meta),
                error_message: "The connection dropped after 212 words.".to_string(),
                on_stop: move |_| {
                    last.set("stopped".to_string());
                    status.set(StreamStatus::Stopped);
                },
                on_continue: move |_| {
                    last.set("continued".to_string());
                    released.set(true);
                    status.set(StreamStatus::Streaming);
                },
                on_retry: move |_| {
                    last.set("retry".to_string());
                    released.set(true);
                    status.set(StreamStatus::Streaming);
                },
                on_dismiss: move |_| {
                    last.set("dismissed, restarting".to_string());
                    shown.set(0);
                    released.set(false);
                    status.set(StreamStatus::Streaming);
                },
                on_copy: move |plain: String| {
                    last.set(format!("copied {} characters", plain.chars().count()));
                },
                on_regenerate: move |_| {
                    last.set("regenerate".to_string());
                    shown.set(0);
                    released.set(false);
                    status.set(StreamStatus::Streaming);
                },
            }
            div { class: "gl-out", "data-out": "interruptible-answer-stream", "{last()}" }
            div { class: "gl-row",
                button { class: "gl-btn", r#type: "button", onclick: move |_| {
                    released.set(true);
                    status.set(StreamStatus::Failed);
                }, "Fail" }
                button { class: "gl-btn", r#type: "button", onclick: move |_| {
                    released.set(true);
                    shown.set(total);
                    status.set(StreamStatus::Done);
                }, "Finish" }
                button { class: "gl-btn", r#type: "button", onclick: move |_| {
                    shown.set(0);
                    released.set(false);
                    status.set(StreamStatus::Streaming);
                }, "Restart" }
            }
        }
    }
}
