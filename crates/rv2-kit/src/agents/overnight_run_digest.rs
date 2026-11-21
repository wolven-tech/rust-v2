//! Overnight run digest: the first screen on returning to agents that kept
//! working. Decisions come first, then errors that can be retried, then what
//! finished, with a chart of activity since the person left.
//!
//! Design choice: no wall clock, since `SystemTime` panics on
//! `wasm32-unknown-unknown`. `now_at` defaults to the latest event time, or to
//! `away_since` when no event has a readable time.
//! Design choice: times are shown in UTC, with the machine-readable ISO string
//! on the `<time>` element. `display_locale` is accepted but does not yet change
//! the figures.
//! Design choice: without `on_act`, a main action completes at once. A
//! successful action moves the card to its past-tense state here; a rejection
//! leaves it where it was and says why.
//! Design choice: revert and reapply move the card here only when their handler
//! is absent. With a handler, the host decides the new state and lists the
//! reverted cards in `reverted_keys`.
//! Design choice: after a revert, focus moves to reapply, which replaces the
//! main action on that card.
//! Design choice: `event_mark` is not taken; each lane carries its own mark.
//! Design choice: lane cards are newest first. Each heading counts every event
//! in its lane, not only the cards shown.
//! Browser checks: the intro sweep and its off switch; chart snap and card
//! highlight on pointer and focus; pending state and disabled siblings; focus
//! move after a main action; lanes stacking in a narrow container; reduced
//! motion removes the intro.

use std::pin::Pin;

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, focus_after_frame, next_id};

const CSS: Asset = asset!("/assets/overnight-run-digest.css");

/// Chart drawing box, in its own units; the SVG scales to the container.
const CHART_W: f64 = 600.0;
const CHART_H: f64 = 120.0;

/// The eventual result of a main action, resolved by the host.
pub type ActFuture = Pin<Box<dyn std::future::Future<Output = Result<(), String>>>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DigestLane {
    AwaitingDecision,
    Errored,
    Completed,
}

impl DigestLane {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AwaitingDecision => "awaiting-decision",
            Self::Errored => "errored",
            Self::Completed => "completed",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::AwaitingDecision => "Decisions",
            Self::Errored => "Errors",
            Self::Completed => "Completed",
        }
    }

    fn empty_text(self) -> &'static str {
        match self {
            Self::AwaitingDecision => "Nothing is waiting on you.",
            Self::Errored => "No errors since you left.",
            Self::Completed => "Nothing finished since you left.",
        }
    }
}

/// Where a card sits: its lane's own stage, or one a person has moved it to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardStage {
    Awaiting,
    Errored,
    Completed,
    Acted,
    Reverted,
}

impl CardStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Awaiting => "awaiting",
            Self::Errored => "errored",
            Self::Completed => "completed",
            Self::Acted => "acted",
            Self::Reverted => "reverted",
        }
    }

    fn tag(self) -> &'static str {
        match self {
            Self::Awaiting => "Needs a decision",
            Self::Errored => "Failed",
            Self::Completed => "Finished",
            Self::Acted => "Done",
            Self::Reverted => "Reverted",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionWhich {
    Main,
    Inspect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transition {
    Act,
    Revert,
    Reapply,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MainAction {
    pub action_label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DigestEvent {
    pub event_key: String,
    /// Unix milliseconds. `None` is an unreadable time; such events are omitted.
    pub happened_at: Option<u64>,
    pub event_lane: DigestLane,
    pub event_title: String,
    pub event_detail: Option<String>,
    pub repeat_count: Option<u32>,
    pub main_action: Option<MainAction>,
    pub inspect_available: bool,
    pub past_tense_label: Option<String>,
    pub reversible: bool,
}

/// A card a person has moved from its lane's stage. `prior` is what a
/// reapply restores.
#[derive(Clone, Debug, PartialEq)]
pub struct CardOverride {
    pub event_key: String,
    pub stage: CardStage,
    pub prior: CardStage,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChartPoint {
    pub event_key: String,
    pub x: f64,
    pub y: f64,
}

/// The stage a card starts in, before any person has acted on it.
pub fn base_stage(event: &DigestEvent) -> CardStage {
    match event.event_lane {
        DigestLane::AwaitingDecision => CardStage::Awaiting,
        DigestLane::Errored => CardStage::Errored,
        DigestLane::Completed => CardStage::Completed,
    }
}

/// `(stage, prior)` for an event, from its override if it has one.
pub fn stage_of(event: &DigestEvent, overrides: &[CardOverride]) -> (CardStage, CardStage) {
    match overrides.iter().find(|o| o.event_key == event.event_key) {
        Some(o) => (o.stage, o.prior),
        None => {
            let base = base_stage(event);
            (base, base)
        }
    }
}

/// The `(stage, prior)` after a transition, or `None` when it does not apply.
/// A completed card reverts only when it is reversible.
pub fn transition(
    stage: CardStage,
    prior: CardStage,
    which: Transition,
    reversible: bool,
) -> Option<(CardStage, CardStage)> {
    match (which, stage) {
        (Transition::Act, CardStage::Awaiting | CardStage::Errored) => {
            Some((CardStage::Acted, stage))
        }
        (Transition::Revert, CardStage::Acted) => Some((CardStage::Reverted, stage)),
        (Transition::Revert, CardStage::Completed) if reversible => {
            Some((CardStage::Reverted, stage))
        }
        (Transition::Reapply, CardStage::Reverted) => Some((prior, CardStage::Reverted)),
        _ => None,
    }
}

/// Events with a readable time, the only ones the digest can place.
pub fn visible_events(events: &[DigestEvent]) -> Vec<DigestEvent> {
    events
        .iter()
        .filter(|e| e.happened_at.is_some())
        .cloned()
        .collect()
}

/// How many visible events a lane holds, cards shown or not.
pub fn lane_total(events: &[DigestEvent], lane: DigestLane) -> usize {
    events.iter().filter(|e| e.event_lane == lane).count()
}

/// The newest `limit` events in a lane, newest first.
pub fn lane_cards(events: &[DigestEvent], lane: DigestLane, limit: usize) -> Vec<DigestEvent> {
    let mut in_lane: Vec<&DigestEvent> = events.iter().filter(|e| e.event_lane == lane).collect();
    in_lane.sort_by_key(|e| std::cmp::Reverse(e.happened_at.unwrap_or(0)));
    in_lane.into_iter().take(limit).cloned().collect()
}

/// Where `at` falls in `[away_since, now_at]`, clamped to the chart's edges.
pub fn fraction(at: u64, away_since: u64, now_at: u64) -> f64 {
    let span = now_at.saturating_sub(away_since);
    if span == 0 {
        return 1.0;
    }
    (at.saturating_sub(away_since) as f64 / span as f64).clamp(0.0, 1.0)
}

fn weight(event: &DigestEvent) -> u64 {
    u64::from(event.repeat_count.unwrap_or(1).max(1))
}

/// One point per event, in time order. The height is the running total of
/// events, so a merged card (`repeat_count`) raises its point further.
pub fn chart_points(
    events: &[DigestEvent],
    away_since: u64,
    now_at: u64,
    width: f64,
    height: f64,
) -> Vec<ChartPoint> {
    let mut timed: Vec<(&DigestEvent, u64)> = events
        .iter()
        .filter_map(|e| e.happened_at.map(|at| (e, at)))
        .collect();
    timed.sort_by_key(|(_, at)| *at);
    let total: u64 = timed.iter().map(|(e, _)| weight(e)).sum();
    let mut running = 0u64;
    timed
        .iter()
        .map(|(event, at)| {
            running += weight(event);
            ChartPoint {
                event_key: event.event_key.clone(),
                x: fraction(*at, away_since, now_at) * width,
                y: height - running as f64 / total as f64 * height,
            }
        })
        .collect()
}

/// `(line, area)` SVG path data: the trace, and the trace closed to the baseline.
pub fn chart_paths(points: &[ChartPoint], height: f64) -> (String, String) {
    let line = points
        .iter()
        .enumerate()
        .map(|(i, p)| format!("{} {:.1} {:.1}", if i == 0 { "M" } else { "L" }, p.x, p.y))
        .collect::<Vec<_>>()
        .join(" ");
    match (points.first(), points.last()) {
        (Some(first), Some(last)) => {
            let area = format!(
                "{line} L {:.1} {height:.1} L {:.1} {height:.1} Z",
                last.x, first.x
            );
            (line, area)
        }
        _ => (String::new(), String::new()),
    }
}

/// The horizontal band each point owns: from the midpoint to its left
/// neighbour to the midpoint to its right. Pointing in a band snaps to its event.
pub fn band_edges(xs: &[f64], width: f64) -> Vec<(f64, f64)> {
    xs.iter()
        .enumerate()
        .map(|(i, x)| {
            let left = if i == 0 { 0.0 } else { (xs[i - 1] + x) / 2.0 };
            let right = match xs.get(i + 1) {
                Some(next) => (x + next) / 2.0,
                None => width,
            };
            (left, right)
        })
        .collect()
}

/// `(year, month, day, hour, minute, second)` in UTC, from Unix milliseconds.
fn civil_utc(ms: u64) -> (i64, i64, i64, i64, i64, i64) {
    let secs = (ms / 1000) as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day, rem / 3600, rem / 60 % 60, rem % 60)
}

/// `YYYY-MM-DDTHH:MM:SSZ`, for the `datetime` attribute.
pub fn iso_utc(ms: u64) -> String {
    let (y, mo, d, h, mi, s) = civil_utc(ms);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// `HH:MM UTC`, for the card's time.
pub fn clock_utc(ms: u64) -> String {
    let (_, _, _, h, mi, _) = civil_utc(ms);
    format!("{h:02}:{mi:02} UTC")
}

/// A DOM-safe fragment of an event key, for element ids.
pub fn dom_slug(key: &str) -> String {
    key.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// Text for a card's main action: its own label, or "Retry" for an errored
/// event without one. `None` when there is nothing to do.
pub fn main_label(event: &DigestEvent, stage: CardStage) -> Option<String> {
    match (stage, &event.main_action) {
        (CardStage::Awaiting | CardStage::Errored, Some(action)) => {
            Some(action.action_label.clone())
        }
        (CardStage::Errored, None) => Some("Retry".to_string()),
        _ => None,
    }
}

fn set_override(
    mut overrides: Signal<Vec<CardOverride>>,
    event_key: String,
    stage: CardStage,
    prior: CardStage,
) {
    overrides.with_mut(|list| {
        list.retain(|o| o.event_key != event_key);
        list.push(CardOverride {
            event_key,
            stage,
            prior,
        });
    });
}

#[derive(Clone, Debug, PartialEq)]
struct CardView {
    event: DigestEvent,
    title: String,
    slug: String,
    stage: CardStage,
    tag: &'static str,
    time_text: String,
    iso: String,
    merged_text: Option<String>,
    main_text: Option<String>,
    past_text: Option<String>,
    revert: bool,
    reapply: bool,
    working: bool,
    other_busy: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct LaneView {
    key: &'static str,
    heading: String,
    heading_id: String,
    empty: &'static str,
    cards: Vec<CardView>,
}

/// The digest of what happened since the person last looked: a chart of
/// activity over the absence, and three lanes of cards to act on.
#[component]
pub fn OvernightRunDigest(
    digest_events: Vec<DigestEvent>,
    away_since: u64,
    #[props(default)] now_at: Option<u64>,
    #[props(default = 3)] cards_per_lane: usize,
    #[props(default = true)] replay_on_open: bool,
    #[props(default = "Since you last checked".to_string(), into)] section_heading: String,
    #[props(default)] display_locale: Option<String>,
    #[props(default)] on_act: Option<Callback<(DigestEvent, ActionWhich), ActFuture>>,
    #[props(default)] on_revert: Option<EventHandler<DigestEvent>>,
    #[props(default)] on_reapply: Option<EventHandler<DigestEvent>>,
    #[props(default)] reverted_keys: Option<Vec<String>>,
    #[props(default)] extra_class: String,
) -> Element {
    let _ = display_locale;
    let uid = use_hook(next_id);
    let heading_id = format!("kit-digest-{uid}-head");
    let overrides = use_signal(Vec::<CardOverride>::new);
    let mut pending = use_signal(|| None::<(String, ActionWhich)>);
    let mut announce = use_signal(String::new);
    let mut hover = use_signal(|| None::<String>);

    let reverted = reverted_keys.unwrap_or_default();
    let visible = visible_events(&digest_events);
    let now = now_at.unwrap_or_else(|| {
        visible
            .iter()
            .filter_map(|e| e.happened_at)
            .max()
            .unwrap_or(away_since)
    });
    let current = overrides();
    let pending_now = pending();
    let busy = pending_now.is_some();
    let hot = hover();
    let class = class_list("kit-digest", &extra_class);
    let line = announce();
    let replay = if replay_on_open { "true" } else { "false" };

    let points = chart_points(&visible, away_since, now, CHART_W, CHART_H);
    let (trace_d, area_d) = chart_paths(&points, CHART_H);
    let xs: Vec<f64> = points.iter().map(|p| p.x).collect();
    let bands: Vec<(String, f64, f64)> = points
        .iter()
        .zip(band_edges(&xs, CHART_W))
        .map(|(p, (left, right))| (p.event_key.clone(), left, right - left))
        .collect();
    let hot_point = hot
        .as_ref()
        .and_then(|key| points.iter().find(|p| &p.event_key == key))
        .map(|p| (p.x, p.y));
    let view_box = format!("0 0 {CHART_W} {CHART_H}");
    let guide_x = hot_point.map(|(x, _)| format!("{x:.1}"));
    let dot_y = hot_point.map(|(_, y)| format!("{y:.1}"));
    let tip = hot.as_ref().and_then(|key| {
        let event = visible.iter().find(|e| &e.event_key == key)?;
        let point = points.iter().find(|p| &p.event_key == key)?;
        Some((
            event.event_title.clone(),
            format!("{:.2}", point.x / CHART_W * 100.0),
        ))
    });
    let tip_visible = tip.is_some();
    let (tip_title, tip_left) = tip.unwrap_or_default();
    let chart_h = format!("{CHART_H}");

    let lanes: Vec<LaneView> = [
        DigestLane::AwaitingDecision,
        DigestLane::Errored,
        DigestLane::Completed,
    ]
    .into_iter()
    .map(|lane| {
        let total = lane_total(&visible, lane);
        let cards = lane_cards(&visible, lane, cards_per_lane)
            .into_iter()
            .map(|event| {
                let (stage, _) = stage_of(&event, &current);
                let stage = if reverted.contains(&event.event_key) {
                    CardStage::Reverted
                } else {
                    stage
                };
                let working = pending_now
                    .as_ref()
                    .is_some_and(|(key, _)| *key == event.event_key);
                let time_ms = event.happened_at.unwrap_or(0);
                CardView {
                    title: event.event_title.clone(),
                    slug: dom_slug(&event.event_key),
                    stage,
                    tag: stage.tag(),
                    time_text: clock_utc(time_ms),
                    iso: iso_utc(time_ms),
                    merged_text: event
                        .repeat_count
                        .filter(|n| *n > 1)
                        .map(|n| format!("{n} similar events merged")),
                    main_text: main_label(&event, stage),
                    past_text: (stage == CardStage::Acted).then(|| {
                        event
                            .past_tense_label
                            .clone()
                            .unwrap_or_else(|| "Done".to_string())
                    }),
                    revert: stage == CardStage::Acted
                        || (stage == CardStage::Completed && event.reversible),
                    reapply: stage == CardStage::Reverted,
                    working,
                    other_busy: busy && !working,
                    event,
                }
            })
            .collect();
        LaneView {
            key: lane.as_str(),
            heading: format!("{} ({total})", lane.title()),
            heading_id: format!("kit-digest-{uid}-{}", lane.as_str()),
            empty: lane.empty_text(),
            cards,
        }
    })
    .collect();

    let run = move |(event, which): (DigestEvent, ActionWhich)| {
        if which == ActionWhich::Inspect {
            if let Some(handler) = on_act {
                let fut = handler.call((event, ActionWhich::Inspect));
                spawn(async move {
                    let _ = fut.await;
                });
            }
            return;
        }
        let (stage, prior) = stage_of(&event, &overrides.peek());
        let Some((next, next_prior)) = transition(stage, prior, Transition::Act, event.reversible)
        else {
            return;
        };
        let key = event.event_key.clone();
        let title = event.event_title.clone();
        let label = main_label(&event, stage).unwrap_or_default();
        let revert_id = format!("kit-digest-{uid}-{}-revert", dom_slug(&key));
        let Some(handler) = on_act else {
            set_override(overrides, key, next, next_prior);
            announce.set(format!("{label} done: {title}."));
            focus_after_frame(&revert_id);
            return;
        };
        pending.set(Some((key.clone(), ActionWhich::Main)));
        let fut = handler.call((event, ActionWhich::Main));
        spawn(async move {
            let result = fut.await;
            pending.set(None);
            match result {
                Ok(()) => {
                    set_override(overrides, key, next, next_prior);
                    announce.set(format!("{label} done: {title}."));
                    focus_after_frame(&revert_id);
                }
                Err(message) => announce.set(format!("Could not {label}: {title}. {message}")),
            }
        });
    };

    let revert = move |event: DigestEvent| {
        let reapply_id = format!("kit-digest-{uid}-{}-reapply", dom_slug(&event.event_key));
        if let Some(handler) = on_revert {
            announce.set(format!("Reverted: {}.", event.event_title));
            handler.call(event);
            focus_after_frame(&reapply_id);
            return;
        }
        let (stage, prior) = stage_of(&event, &overrides.peek());
        let Some((next, next_prior)) =
            transition(stage, prior, Transition::Revert, event.reversible)
        else {
            return;
        };
        announce.set(format!("Reverted: {}.", event.event_title));
        set_override(overrides, event.event_key, next, next_prior);
        focus_after_frame(&reapply_id);
    };

    let reapply = move |event: DigestEvent| {
        if let Some(handler) = on_reapply {
            announce.set(format!("Reapplied: {}.", event.event_title));
            handler.call(event);
            return;
        }
        let (stage, prior) = stage_of(&event, &overrides.peek());
        let Some((next, next_prior)) =
            transition(stage, prior, Transition::Reapply, event.reversible)
        else {
            return;
        };
        announce.set(format!("Reapplied: {}.", event.event_title));
        set_override(overrides, event.event_key, next, next_prior);
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            class,
            "data-replay": replay,
            "aria-labelledby": "{heading_id}",
            h2 { id: "{heading_id}", class: "kit-digest__heading", "{section_heading}" }
            p { class: "kit-digest__sr", role: "status", "aria-live": "polite", "{line}" }
            div { class: "kit-digest__chart", "aria-hidden": "true",
                svg {
                    class: "kit-digest__svg",
                    view_box: "{view_box}",
                    preserve_aspect_ratio: "none",
                    onmouseleave: move |_| hover.set(None),
                    path { class: "kit-digest__area", d: "{area_d}" }
                    path { class: "kit-digest__trace", d: "{trace_d}", "pathLength": "1" }
                    if let Some(x) = guide_x.clone() {
                        line { class: "kit-digest__guide", x1: "{x}", x2: "{x}", y1: "0", y2: "{chart_h}" }
                    }
                    if let Some(y) = dot_y.clone() {
                        if let Some(x) = guide_x.clone() {
                            circle { class: "kit-digest__dot", "data-hot": "true", cx: "{x}", cy: "{y}", r: "5" }
                        }
                    }
                    for point in points.iter() {
                        circle {
                            class: "kit-digest__dot",
                            "data-hot": (hot.as_deref() == Some(point.event_key.as_str())).then_some("true"),
                            cx: format!("{:.1}", point.x),
                            cy: format!("{:.1}", point.y),
                            r: "3",
                        }
                    }
                    for (key, left, width) in bands {
                        rect {
                            class: "kit-digest__band",
                            x: format!("{left:.1}"),
                            y: "0",
                            width: format!("{width:.1}"),
                            height: "{chart_h}",
                            onmouseenter: {
                                let key = key.clone();
                                move |_| hover.set(Some(key.clone()))
                            },
                        }
                    }
                }
                if tip_visible {
                    div { class: "kit-digest__tip", style: "left: {tip_left}%", "{tip_title}" }
                }
            }
            div { class: "kit-digest__lanes",
                for LaneView { key, heading, heading_id, empty, cards } in lanes {
                    section {
                        class: "kit-digest__lane",
                        "data-lane": key,
                        "aria-labelledby": "{heading_id}",
                        h3 { id: "{heading_id}", class: "kit-digest__lane-title", "{heading}" }
                        if cards.is_empty() {
                            p { class: "kit-digest__empty", "{empty}" }
                        }
                        div { class: "kit-digest__cards",
                            for CardView {
                                event, title, slug, stage, tag, time_text, iso, merged_text, main_text,
                                past_text, revert: can_revert, reapply: can_reapply, working,
                                other_busy, ..
                            } in cards
                            {
                                article {
                                    class: "kit-digest__card",
                                    "data-stage": stage.as_str(),
                                    "data-hot": (hot.as_deref() == Some(event.event_key.as_str())).then_some("true"),
                                    "aria-busy": working.then_some("true"),
                                    onmouseenter: {
                                        let key = event.event_key.clone();
                                        move |_| hover.set(Some(key.clone()))
                                    },
                                    onmouseleave: move |_| hover.set(None),
                                    header { class: "kit-digest__card-head",
                                        span { class: "kit-digest__tag", "data-stage": stage.as_str(), "{tag}" }
                                        time { class: "kit-digest__time", datetime: "{iso}", "{time_text}" }
                                    }
                                    h4 { class: "kit-digest__title", "{title}" }
                                    if let Some(detail) = event.event_detail.clone() {
                                        p { class: "kit-digest__detail", "{detail}" }
                                    }
                                    if let Some(merged) = merged_text {
                                        p { class: "kit-digest__merged", "{merged}" }
                                    }
                                    footer { class: "kit-digest__actions",
                                        if event.inspect_available {
                                            button {
                                                r#type: "button",
                                                class: "kit-digest__action",
                                                "data-action": "Inspect",
                                                disabled: other_busy,
                                                "aria-label": "Inspect: {title}",
                                                onclick: {
                                                    let event = event.clone();
                                                    move |_| {
                                                        let mut run = run;
                                                        run((event.clone(), ActionWhich::Inspect))
                                                    }
                                                },
                                                "Inspect"
                                            }
                                        }
                                        if let Some(label) = main_text {
                                            button {
                                                r#type: "button",
                                                id: "kit-digest-{uid}-{slug}-main",
                                                class: "kit-digest__action kit-digest__action--main",
                                                disabled: other_busy || working,
                                                "aria-label": "{label}: {title}",
                                                onclick: {
                                                    let event = event.clone();
                                                    move |_| {
                                                        let mut run = run;
                                                        run((event.clone(), ActionWhich::Main))
                                                    }
                                                },
                                                if working {
                                                    span { class: "kit-digest__spin", "aria-hidden": "true" }
                                                }
                                                "{label}"
                                            }
                                        }
                                        if let Some(past) = past_text {
                                            span { class: "kit-digest__past", "{past}" }
                                        }
                                        if can_revert {
                                            button {
                                                r#type: "button",
                                                id: "kit-digest-{uid}-{slug}-revert",
                                                class: "kit-digest__action",
                                                disabled: other_busy,
                                                "aria-label": "Revert: {title}",
                                                onclick: {
                                                    let event = event.clone();
                                                    move |_| {
                                                        let mut revert = revert;
                                                        revert(event.clone())
                                                    }
                                                },
                                                "Revert"
                                            }
                                        }
                                        if can_reapply {
                                            button {
                                                r#type: "button",
                                                id: "kit-digest-{uid}-{slug}-reapply",
                                                class: "kit-digest__action",
                                                disabled: other_busy,
                                                "aria-label": "Reapply: {title}",
                                                onclick: {
                                                    let event = event.clone();
                                                    move |_| {
                                                        let mut reapply = reapply;
                                                        reapply(event.clone())
                                                    }
                                                },
                                                "Reapply"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
