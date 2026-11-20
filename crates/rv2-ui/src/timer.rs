//! Running clocks: a timer readout and a timed section of a run.
//!
//! Drawn from Talk Run Card's rehearsal runner and jbt-fit's workout runner.
//! Both apps own their clock and pass the elapsed seconds in; the kit only
//! renders them, so it keeps no timer of its own.
//!
//! The digits sit in `role="timer"`, which is not announced as it changes. The
//! pace word sits in its own `role="status"` and changes only when the band
//! does, so a screen reader hears "Prepare to move", not every second.

use dioxus::prelude::*;

use crate::feedback::Verdict;
use crate::readout::Meter;

/// Where a timed run stands against its plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pace {
    OnPace,
    /// At or past the warning share of the plan.
    Closing,
    /// Past the plan.
    Over,
}

impl Pace {
    pub fn word(self) -> &'static str {
        match self {
            Self::OnPace => "On pace",
            Self::Closing => "Prepare to move on",
            Self::Over => "Over time",
        }
    }

    fn verdict(self) -> Verdict {
        match self {
            Self::OnPace => Verdict::Clear,
            Self::Closing => Verdict::Caution,
            Self::Over => Verdict::Stop,
        }
    }
}

/// The pace band for `elapsed` of `planned` seconds, closing from `warn_pct`
/// percent of the plan. Over means strictly past the plan.
pub fn pace(elapsed: u32, planned: u32, warn_pct: u8) -> Pace {
    if elapsed > planned {
        Pace::Over
    } else if u64::from(elapsed) * 100 >= u64::from(planned) * u64::from(warn_pct) {
        Pace::Closing
    } else {
        Pace::OnPace
    }
}

/// Which way a [`TimerReadout`] counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClockDirection {
    /// Time spent.
    #[default]
    Up,
    /// Time left; past zero it shows the overrun with a plus sign.
    Down,
}

/// Seconds as `m:ss`, or `h:mm:ss` from an hour.
pub fn clock(secs: u32) -> String {
    let (h, m, s) = (secs / 3600, secs / 60 % 60, secs % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// A clock and, when there is a plan, the pace word beside it.
///
/// Children are the controls: start, pause, skip.
#[component]
pub fn TimerReadout(
    label: String,
    elapsed_secs: u32,
    #[props(default)] planned_secs: Option<u32>,
    #[props(default)] direction: ClockDirection,
    #[props(default = 80)] warn_pct: u8,
    children: Element,
) -> Element {
    let shown = match (direction, planned_secs) {
        (ClockDirection::Down, Some(planned)) if elapsed_secs > planned => {
            format!("+{}", clock(elapsed_secs - planned))
        }
        (ClockDirection::Down, Some(planned)) => clock(planned - elapsed_secs),
        _ => clock(elapsed_secs),
    };
    let band = planned_secs.map(|planned| pace(elapsed_secs, planned, warn_pct));
    rsx! {
        div { class: "space-y-1",
            p { class: "font-mono text-xs font-medium uppercase tracking-widest text-ink-muted", "{label}" }
            p {
                class: "font-mono text-5xl font-semibold tabular-nums tracking-tight text-ink",
                role: "timer",
                aria_label: "{label}",
                "{shown}"
            }
            if let Some(band) = band {
                p { class: "text-sm font-semibold {band.verdict().ink()}", role: "status", "{band.word()}" }
            }
            div { class: "flex flex-wrap gap-2 pt-2", {children} }
        }
    }
}

/// One section of a timed run: where it sits in the run, what to say, what
/// comes next, and its clock against its plan.
///
/// Children are the run controls.
#[component]
pub fn SectionTimer(
    /// Zero-based.
    section: usize,
    sections: usize,
    title: String,
    #[props(default)] cue: Option<String>,
    #[props(default)] next: Option<String>,
    elapsed_secs: u32,
    planned_secs: u32,
    #[props(default = 80)] warn_pct: u8,
    children: Element,
) -> Element {
    let band = pace(elapsed_secs, planned_secs, warn_pct);
    let (warn_at, critical_at) = match band {
        Pace::Over => (None, Some(0.0)),
        _ => (Some(f64::from(warn_pct) / 100.0), None),
    };
    let position = format!("Section {} of {}", section + 1, sections);
    rsx! {
        section { class: "space-y-4", aria_label: "{position}: {title}",
            div {
                p { class: "font-mono text-xs font-medium uppercase tracking-widest text-ink-muted", "{position}" }
                h3 { class: "mt-1 text-xl font-semibold text-ink", "{title}" }
                if let Some(cue) = cue {
                    p { class: "mt-1 text-ink", "{cue}" }
                }
            }
            TimerReadout {
                label: "Section time",
                elapsed_secs,
                planned_secs,
                direction: ClockDirection::Down,
                warn_pct,
            }
            Meter {
                label: "Section progress",
                value: f64::from(elapsed_secs.min(planned_secs)),
                max: f64::from(planned_secs),
                readout: format!("{} of {}", clock(elapsed_secs), clock(planned_secs)),
                warn_at,
                critical_at,
            }
            if let Some(next) = next {
                p { class: "text-sm text-ink-muted", "Next · {next}" }
            }
            div { class: "flex flex-wrap gap-2", {children} }
        }
    }
}
