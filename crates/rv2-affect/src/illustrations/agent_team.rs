//! A prompt types itself into a pill and is sent. Dashed lines branch out from it to three
//! agents, who pop in one after another with their role and the tools they use, then the scene
//! starts over. Laid out on a 600 × 400 grid inside a 3 : 2 frame.
//!
//! The first render is the finished scene, so a still frame shows the team; the loop waits `HOLD`
//! before it types again.
//!
//! Port losses: the choreography runs as a `spawn` loop over `sleep`, with the timeline built by
//! `schedule`. Reduced motion is read once on mount through `document::eval`; the finished scene
//! is then shown and no loop starts.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, prefers_motion_script, sleep};

const CSS: Asset = asset!("/assets/agent-team.css");
const SPRITE: &str = include_str!("../../assets/agent-team.svg");

/// Choreography, in ms.
const TYPE: u32 = 22;
const SEND: u32 = 420;
const SPREAD: u32 = 700;
const STAGGER: u32 = 160;
const HOLD: u32 = 6000;
const RESET: u32 = 600;
const LEAD_IN: u32 = 500;

const DEFAULT_PROMPT: &str = "Build a team to work my inbound pipeline";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Typing,
    Sent,
    Team,
    Reset,
}

impl Phase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Typing => "typing",
            Self::Sent => "sent",
            Self::Team => "team",
            Self::Reset => "reset",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// The prompt shows this many characters.
    Typed(usize),
    Phase(Phase),
}

/// Length of one full cycle, in ms: typing, sending, the team, the hold, then the reset.
pub fn cycle_len(prompt_len: usize) -> u32 {
    let typed_at = LEAD_IN + prompt_len as u32 * TYPE;
    let team_done = typed_at + SEND + SPREAD + STAGGER * 3 + LEAD_IN;
    team_done + HOLD + RESET
}

/// The steps of one cycle with their times in ms, in order. The cycle ends at `cycle_len`.
pub fn schedule(prompt_len: usize) -> Vec<(u32, Step)> {
    let mut out: Vec<(u32, Step)> = (1..=prompt_len)
        .map(|i| (LEAD_IN + i as u32 * TYPE, Step::Typed(i)))
        .collect();
    let typed_at = LEAD_IN + prompt_len as u32 * TYPE;
    let team_done = typed_at + SEND + SPREAD + STAGGER * 3 + LEAD_IN;
    out.push((typed_at + SEND, Step::Phase(Phase::Sent)));
    out.push((
        typed_at + SEND + (SPREAD as f64 * 0.55) as u32,
        Step::Phase(Phase::Team),
    ));
    out.push((team_done + HOLD, Step::Phase(Phase::Reset)));
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentTool {
    LinkedIn,
    HubSpot,
    Slack,
    Gmail,
    Salesforce,
}

impl AgentTool {
    fn symbol(self) -> &'static str {
        match self {
            Self::LinkedIn => "linkedin",
            Self::HubSpot => "hubspot",
            Self::Slack => "slack",
            Self::Gmail => "gmail",
            Self::Salesforce => "salesforce",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentTeamMember {
    /// Role shown in the pill.
    pub role: String,
    /// Square portrait, shown in a circle.
    pub avatar: String,
    /// One or two tools, drawn as overlapping logo circles.
    pub tools: Vec<AgentTool>,
}

/// The sprite's symbol with this id, as inner SVG markup.
fn symbol(id: &str) -> &'static str {
    let needle = format!("<symbol id=\"{id}\"");
    let Some(start) = SPRITE.find(&needle) else {
        return "";
    };
    let body = start + SPRITE[start..].find('>').unwrap_or(0) + 1;
    let end = SPRITE[body..]
        .find("</symbol>")
        .map_or(body, |offset| body + offset);
    &SPRITE[body..end]
}

const LINES: [&str; 3] = [
    "M300 110C300 162 102 152 102 196",
    "M300 110V196",
    "M300 110C300 162 498 152 498 196",
];

#[component]
pub fn AgentTeam(
    #[props(default = DEFAULT_PROMPT.to_string(), into)] prompt: String,
    members: [AgentTeamMember; 3],
    #[props(default, into)] class: String,
    #[props(default)] label: Option<String>,
) -> Element {
    let id = use_hook(|| format!("ui-atm{}", next_id()));
    let mut typed = use_signal(|| prompt.chars().count());
    let mut phase = use_signal(|| Phase::Team);
    let mut task = use_signal(|| None::<dioxus::core::Task>);

    use_effect(use_reactive((&prompt,), move |(prompt,)| {
        if let Some(running) = task.peek().as_ref() {
            running.cancel();
        }
        let length = prompt.chars().count();
        let mut channel = document::eval(&format!("dioxus.send({});", prefers_motion_script()));
        let handle = spawn(async move {
            if !channel.recv::<bool>().await.unwrap_or(true) {
                typed.set(length);
                phase.set(Phase::Team);
                return;
            }
            let steps = schedule(length);
            let cycle = cycle_len(length);
            sleep(HOLD).await;
            loop {
                phase.set(Phase::Typing);
                typed.set(0);
                let mut previous = 0;
                for (at, step) in steps.iter().copied() {
                    sleep(at - previous).await;
                    previous = at;
                    match step {
                        Step::Typed(count) => typed.set(count),
                        Step::Phase(next) => phase.set(next),
                    }
                }
                sleep(cycle - previous).await;
            }
        });
        task.set(Some(handle));
    }));

    let shown_text: String = prompt.chars().take(typed()).collect();
    let roles: Vec<&str> = members.iter().map(|m| m.role.as_str()).collect();
    let label = label.unwrap_or_else(|| {
        format!(
            "A prompt, “{prompt}”, builds a team of three agents: {}",
            roles.join(", ")
        )
    });
    let class = class_list("ui-atm", &class);
    let phase_attr = phase().as_str();

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { id: "{id}", class, role: "img", "aria-label": "{label}", "data-phase": phase_attr,
            svg { class: "ui-atm-lines", "viewBox": "0 0 600 400", "aria-hidden": "true",
                defs {
                    for (i, d) in LINES.iter().copied().enumerate() {
                        mask { key: "{i}", id: "{id}-reveal-{i}", "maskUnits": "userSpaceOnUse", x: "0", y: "0", width: "600", height: "400",
                            path { class: "ui-atm-reveal", d, "pathLength": "1" }
                        }
                    }
                }
                for (i, d) in LINES.iter().copied().enumerate() {
                    path { key: "{i}", class: "ui-atm-line", d, mask: "url(#{id}-reveal-{i})" }
                }
            }

            div { class: "ui-atm-halo", "aria-hidden": "true" }

            div { class: "ui-atm-prompt", "aria-hidden": "true",
                span { class: "ui-atm-text",
                    bdi { class: "ui-atm-typed",
                        "{shown_text}"
                        span { class: "ui-atm-caret" }
                    }
                }
                span { class: "ui-atm-send",
                    svg { "viewBox": "0 0 24 24", "aria-hidden": "true",
                        path { d: "M12 18.5v-13M6.5 11 12 5.5l5.5 5.5" }
                    }
                }
            }

            for (i, member) in members.iter().enumerate() {
                {agent(i, member.clone())}
            }
        }
    }
}

fn agent(index: usize, member: AgentTeamMember) -> Element {
    let left = 102 + index * 198;
    let style = format!("left: calc({left} * var(--u)); --i: {index}");
    rsx! {
        div { class: "ui-atm-agent", style, "aria-hidden": "true",
            div { class: "ui-atm-node",
                span { class: "ui-atm-face",
                    img { src: "{member.avatar}", alt: "", draggable: "false" }
                }
                span { class: "ui-atm-dot" }
            }
            div { class: "ui-atm-role",
                if !member.tools.is_empty() {
                    span { class: "ui-atm-tools",
                        for tool in member.tools.iter().take(2).copied() {
                            span { key: "{tool.symbol()}", class: "ui-atm-tool",
                                svg { "viewBox": "0 0 24 24", "aria-hidden": "true", dangerous_inner_html: symbol(tool.symbol()) }
                            }
                        }
                    }
                }
                span { "{member.role}" }
            }
        }
    }
}
