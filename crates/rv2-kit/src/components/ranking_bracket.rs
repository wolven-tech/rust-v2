//! Ranking bracket: a judge picks the strongest of several answers, pairing by pairing, to one final winner.
//!
//! Design choice: a verdict counts only when both entrants of its pairing are known and its winner is one of them.
//! Anything else is ignored, so a stray verdict never advances an entrant who did not reach the pairing.
//! Design choice: the layout is `agent_context_map::bracket_centres`, so every pairing sits midway between the two
//! groups it joins.
//! Design choice: each round is its own column element, so the narrow layout lists one round after another in order.
//! Browser checks: winner glide between rounds, route draw-in, detail easing, narrow list-per-round reflow.

use dioxus::prelude::*;

use crate::components::agent_context_map::{bracket_centres, focus_on_mount};
use crate::internal::*;

const CSS: Asset = asset!("/assets/ranking-bracket.css");

/// Height of one entrant row in the bracket, design units.
pub const ROW_H: f64 = 64.0;
/// Width of one round column, design units.
pub const COL_W: f64 = 220.0;
/// Pairing box size in px; must match `.kit-rb__pair` in the stylesheet.
const PAIR_W: f64 = 180.0;
const PAIR_H: f64 = 72.0;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Entrant {
    pub entrant_id: String,
    pub entrant_name: String,
    pub entrant_mark: Option<String>,
    pub entrant_meta: Option<String>,
    pub entrant_text: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Verdict {
    pub winner_id: String,
    pub rationale: Option<String>,
    /// Scores for the upper and lower entrant of the pairing.
    pub scores: Option<(f64, f64)>,
}

/// One pairing with both slots resolved, and its winner once a valid verdict exists.
#[derive(Clone, PartialEq, Debug)]
pub struct Pairing {
    pub round: usize,
    pub index: usize,
    pub upper: Option<Entrant>,
    pub lower: Option<Entrant>,
    pub winner: Option<String>,
    /// The verdict behind `winner`, when one is valid.
    pub verdict: Option<Verdict>,
}

/// Key used by the verdict map: `"round:pairing"`, zero-based.
pub fn pairing_key(round: usize, index: usize) -> String {
    format!("{round}:{index}")
}

/// Inverse of `pairing_key`.
pub fn parse_pairing_key(key: &str) -> Option<(usize, usize)> {
    let (round, index) = key.split_once(':')?;
    Some((round.parse().ok()?, index.parse().ok()?))
}

/// Round name from the number of rounds: the last is the final, then semifinal and quarterfinal.
pub fn round_name(round: usize, rounds: usize) -> String {
    match rounds.saturating_sub(round + 1) {
        0 => "Final".to_string(),
        1 => "Semifinal".to_string(),
        2 => "Quarterfinal".to_string(),
        _ => format!("Round {}", round + 1),
    }
}

/// A verdict counts only when the winner is one of the two entrants.
pub fn valid_winner(verdict: &Verdict, upper: &Entrant, lower: &Entrant) -> Option<String> {
    (verdict.winner_id == upper.entrant_id || verdict.winner_id == lower.entrant_id)
        .then(|| verdict.winner_id.clone())
}

/// Resolves every pairing round by round. Round zero pairs adjacent seeds; later rounds take the winners of the
/// two pairings that feed them. Returns no rounds when the entrant count is not a power of two of at least two.
pub fn resolve_bracket(entrants: &[Entrant], verdicts: &[(String, Verdict)]) -> Vec<Vec<Pairing>> {
    if bracket_centres(entrants.len(), 1.0, 1.0).is_empty() {
        return Vec::new();
    }
    let mut rounds: Vec<Vec<Pairing>> = Vec::new();
    let mut feeders: Vec<Option<Entrant>> = entrants.iter().map(|e| Some(e.clone())).collect();
    for round in 0..entrants.len().trailing_zeros() as usize {
        let mut pairings = Vec::new();
        let mut next: Vec<Option<Entrant>> = Vec::new();
        for index in 0..feeders.len() / 2 {
            let upper = feeders[2 * index].clone();
            let lower = feeders[2 * index + 1].clone();
            let verdict = match (&upper, &lower) {
                (Some(u), Some(l)) => verdicts
                    .iter()
                    .find(|(k, _)| *k == pairing_key(round, index))
                    .map(|(_, v)| v)
                    .filter(|v| valid_winner(v, u, l).is_some())
                    .cloned(),
                _ => None,
            };
            let winner = verdict.as_ref().map(|v| v.winner_id.clone());
            next.push(winner.as_ref().and_then(|w| {
                upper
                    .iter()
                    .chain(lower.iter())
                    .find(|e| e.entrant_id == *w)
                    .cloned()
            }));
            pairings.push(Pairing {
                round,
                index,
                upper,
                lower,
                winner,
                verdict,
            });
        }
        rounds.push(pairings);
        feeders = next;
    }
    rounds
}

/// The overall winner, once the final has a valid verdict.
pub fn champion(rounds: &[Vec<Pairing>]) -> Option<String> {
    rounds
        .last()
        .and_then(|r| r.first())
        .and_then(|p| p.winner.clone())
}

/// Pairing a pairing flows into: the next round, index halved.
pub fn flows_into(round: usize, index: usize, rounds: usize) -> Option<(usize, usize)> {
    (round + 1 < rounds).then_some((round + 1, index / 2))
}

/// Earliest pairing that flows into this one, in the round below.
pub fn feeds_from(round: usize, index: usize) -> Option<(usize, usize)> {
    round.checked_sub(1).map(|r| (r, index * 2))
}

/// Pairing an arrow key moves to from `at`, given the pairing count of each round. Left and right
/// follow the bracket's feeds; up and down move within the round.
pub fn pair_step(sizes: &[usize], at: (usize, usize), key: &Key) -> Option<(usize, usize)> {
    let (r, i) = at;
    match key {
        Key::ArrowRight => flows_into(r, i, sizes.len()).filter(|(nr, ni)| *ni < sizes[*nr]),
        Key::ArrowLeft => feeds_from(r, i).filter(|(pr, pi)| *pi < sizes[*pr]),
        Key::ArrowDown => (i + 1 < sizes[r]).then_some((r, i + 1)),
        Key::ArrowUp => i.checked_sub(1).map(|p| (r, p)),
        _ => None,
    }
}

/// Status line text: the pairing under judgement, or the result.
pub fn status_line(in_flight: Option<&str>, champion: Option<&str>, judge: &str) -> String {
    match (in_flight, champion) {
        (Some(title), _) => format!("Judging {title} · {judge}"),
        (None, Some(_)) => "Complete".to_string(),
        _ => format!("Waiting · {judge}"),
    }
}

/// Pairing name for the status line: the two entrants, or the key while either slot is still empty.
pub fn pairing_title(pairing: &Pairing) -> String {
    match (&pairing.upper, &pairing.lower) {
        (Some(u), Some(l)) => format!("{} against {}", u.entrant_name, l.entrant_name),
        _ => pairing_key(pairing.round, pairing.index),
    }
}

/// Knockout bracket of candidate answers, judged pairing by pairing. Opening a pairing shows both answers.
#[component]
pub fn RankingBracket(
    entrants: Vec<Entrant>,
    #[props(default)] verdicts: Vec<(String, Verdict)>,
    #[props(default)] pairing_in_flight: Option<String>,
    /// Pairing whose answers are open at mount, by key.
    #[props(default)]
    open_pairing: Option<String>,
    #[props(default)] judge_label: String,
    #[props(default)] brief: String,
    #[props(default)] on_adopt: Option<EventHandler<Entrant>>,
    #[props(default = "Judge".to_string(), into)] heading: String,
    #[props(default)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-rb-{}", next_id()));
    let mut open = use_signal(|| open_pairing.as_deref().and_then(parse_pairing_key));

    let rounds = resolve_bracket(&entrants, &verdicts);
    let rounds_total = rounds.len();
    let champ = champion(&rounds);
    let champ_entrant = champ
        .as_ref()
        .and_then(|id| entrants.iter().find(|e| e.entrant_id == *id).cloned());
    let in_flight_title = pairing_in_flight
        .as_deref()
        .and_then(parse_pairing_key)
        .and_then(|(r, i)| rounds.get(r)?.get(i))
        .map(pairing_title);
    let status = status_line(in_flight_title.as_deref(), champ.as_deref(), &judge_label);
    let class = class_list("kit-rb", &extra_class);
    let centres = bracket_centres(entrants.len(), ROW_H, COL_W);
    let width = COL_W * rounds_total.max(1) as f64;
    let height = ROW_H * entrants.len().max(1) as f64;
    let box_style = format!("width: {width}px; height: {height}px;");
    let open_pair = open().and_then(|(r, i)| rounds.get(r).and_then(|round| round.get(i)).cloned());
    let adopt_label = champ_entrant
        .as_ref()
        .map(|e| format!("Use {}", e.entrant_name));
    let sizes: Vec<usize> = rounds.iter().map(Vec::len).collect();
    let link_paths: Vec<(String, bool)> = rounds
        .iter()
        .enumerate()
        .flat_map(|(r, round)| {
            (0..round.len())
                .filter_map(|i| {
                    let (cx, cy) = *centres.get(r)?.get(i)?;
                    let (px, py) = *centres.get(r + 1)?.get(i / 2)?;
                    let x1 = cx + PAIR_W;
                    let xm = (x1 + px) / 2.0;
                    Some((
                        format!("M {x1} {cy} H {xm} V {py} H {px}"),
                        round[i].winner.is_some(),
                    ))
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let detail_base = base.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", "aria-labelledby": "{base}-title", "data-complete": champ.is_some().then_some("true"),
            header { class: "kit-rb__head",
                div { class: "kit-rb__heading",
                    h2 { id: "{base}-title", class: "kit-rb__title", "{heading}" }
                    if !brief.is_empty() { p { class: "kit-rb__brief", "{brief}" } }
                    if !judge_label.is_empty() { p { class: "kit-rb__judge", "{judge_label}" } }
                }
                p { class: "kit-rb__status", role: "status", "aria-live": "polite", "{status}" }
            }
            div { class: "kit-rb__scroll",
                div { class: "kit-rb__bracket", style: "{box_style}", role: "group", "aria-label": "Knockout rounds",
                    svg { class: "kit-rb__links", width: "{width}", height: "{height}", view_box: "0 0 {width} {height}", "aria-hidden": "true",
                        for (d, route) in link_paths {
                            path { key: "{d}", class: "kit-rb__link", "data-route": route.then_some("true"), d: "{d}" }
                        }
                    }
                    for (r, round) in rounds.iter().enumerate() {
                        {
                            let label = round_name(r, rounds_total);
                            let col_style = format!("left: {}px; width: {COL_W}px; height: {height}px;", r as f64 * COL_W);
                            rsx! {
                                div { key: "col{r}", class: "kit-rb__col", style: "{col_style}",
                                    span { class: "kit-rb__round", "{label}" }
                                    for (i, pairing) in round.iter().enumerate() {
                                        {
                                            let (_, cy) = centres[r][i];
                                            let key = pairing_key(r, i);
                                            let in_flight = pairing_in_flight.as_deref() == Some(key.as_str());
                                            let settled = pairing.winner.is_some();
                                            let style = format!("left: 0px; top: {:.1}px;", cy - PAIR_H / 2.0);
                                            let upper = pairing.upper.as_ref();
                                            let lower = pairing.lower.as_ref();
                                            let upper_name = upper.map(|e| e.entrant_name.clone()).unwrap_or_else(|| "Waiting".into());
                                            let lower_name = lower.map(|e| e.entrant_name.clone()).unwrap_or_else(|| "Waiting".into());
                                            let upper_meta = upper.and_then(|e| e.entrant_meta.clone());
                                            let lower_meta = lower.and_then(|e| e.entrant_meta.clone());
                                            let upper_win = settled && upper.is_some_and(|e| Some(&e.entrant_id) == pairing.winner.as_ref());
                                            let lower_win = settled && lower.is_some_and(|e| Some(&e.entrant_id) == pairing.winner.as_ref());
                                            let upper_lose = settled && upper.is_some() && !upper_win;
                                            let lower_lose = settled && lower.is_some() && !lower_win;
                                            let scores = pairing.verdict.as_ref().and_then(|v| v.scores);
                                            let upper_score = scores.map(|(u, _)| format!("{u:.0}"));
                                            let lower_score = scores.map(|(_, l)| format!("{l:.0}"));
                                            let winner_name = if upper_win { upper_name.clone() } else { lower_name.clone() };
                                            let state = if settled {
                                                format!("winner {winner_name}")
                                            } else if in_flight {
                                                "being judged".to_string()
                                            } else {
                                                "waiting".to_string()
                                            };
                                            let label = format!("{}, {upper_name} against {lower_name}, {state}", round_name(r, rounds_total));
                                            let can_open = (settled || in_flight) && upper.is_some() && lower.is_some();
                                            let is_open = open() == Some((r, i));
                                            let dom_id = format!("{base}-p-{r}-{i}");
                                            let nav_sizes = sizes.clone();
                                            let nav_base = base.clone();
                                            rsx! {
                                                button {
                                                    key: "{key}", id: "{dom_id}", r#type: "button", class: "kit-rb__pair", style: "{style}",
                                                    "data-in-flight": in_flight.then_some("true"), "data-settled": settled.then_some("true"),
                                                    "data-openable": can_open.then_some("true"),
                                                    "aria-label": "{label}", "aria-expanded": can_open.then(|| is_open.to_string()),
                                                    onclick: move |_| {
                                                        if !can_open { return; }
                                                        open.set(if open() == Some((r, i)) { None } else { Some((r, i)) });
                                                    },
                                                    onkeydown: move |event: KeyboardEvent| {
                                                        if event.key() == Key::Escape && is_open {
                                                            event.prevent_default();
                                                            open.set(None);
                                                            return;
                                                        }
                                                        if let Some((nr, ni)) = pair_step(&nav_sizes, (r, i), &event.key()) {
                                                            event.prevent_default();
                                                            let target = format!("{nav_base}-p-{nr}-{ni}");
                                                            document::eval(&format!("document.getElementById({target:?})?.focus();"));
                                                        }
                                                    },
                                                    span { class: "kit-rb__slot", "data-win": upper_win.then_some("true"), "data-lose": upper_lose.then_some("true"),
                                                        span { class: "kit-rb__slot-text",
                                                            span { class: "kit-rb__name", "{upper_name}" }
                                                            if let Some(meta) = upper_meta { span { class: "kit-rb__slot-meta", "{meta}" } }
                                                        }
                                                        if upper_win {
                                                            span { class: "kit-rb__figure",
                                                                if let Some(s) = upper_score { span { class: "kit-rb__score", "{s}" } }
                                                                span { class: "kit-rb__mark", "aria-hidden": "true", "✓" }
                                                            }
                                                        }
                                                    }
                                                    span { class: "kit-rb__slot", "data-win": lower_win.then_some("true"), "data-lose": lower_lose.then_some("true"),
                                                        span { class: "kit-rb__slot-text",
                                                            span { class: "kit-rb__name", "{lower_name}" }
                                                            if let Some(meta) = lower_meta { span { class: "kit-rb__slot-meta", "{meta}" } }
                                                        }
                                                        if lower_win {
                                                            span { class: "kit-rb__figure",
                                                                if let Some(s) = lower_score { span { class: "kit-rb__score", "{s}" } }
                                                                span { class: "kit-rb__mark", "aria-hidden": "true", "✓" }
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
            }
            if let (Some((open_r, open_i)), Some(pairing)) = (open(), open_pair) {
                section { class: "kit-rb__detail", "aria-label": "Pairing answers", tabindex: "-1",
                    onmounted: focus_on_mount,
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape {
                            event.prevent_default();
                            open.set(None);
                            let target = format!("{detail_base}-p-{open_r}-{open_i}");
                            document::eval(&format!("document.getElementById({target:?})?.focus();"));
                        }
                    },
                    for (side, entrant) in [("upper", pairing.upper.clone()), ("lower", pairing.lower.clone())] {
                        if let Some(e) = entrant {
                            {
                                let scores = pairing.verdict.as_ref().and_then(|v| v.scores).map(|(upper, lower)| {
                                    if side == "upper" { upper } else { lower }
                                });
                                let text = e.entrant_text.clone().unwrap_or_default();
                                let meta = e.entrant_meta.clone().unwrap_or_default();
                                rsx! {
                                    article { key: "{side}", class: "kit-rb__answer",
                                        header { class: "kit-rb__answer-head",
                                            span { class: "kit-rb__name", "{e.entrant_name}" }
                                            span { class: "kit-rb__meta", "{meta}" }
                                            if let Some(s) = scores { span { class: "kit-rb__score", "Score {s:.0}" } }
                                        }
                                        pre { class: "kit-rb__text", "{text}" }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(rationale) = pairing.verdict.as_ref().and_then(|v| v.rationale.clone()) {
                        p { class: "kit-rb__rationale", "{rationale}" }
                    }
                }
            }
            if let Some(entrant) = champ_entrant.clone() {
                div { class: "kit-rb__champion",
                    svg { class: "kit-rb__trophy", width: "18", height: "18", view_box: "0 0 24 24", "aria-hidden": "true",
                        fill: "none", stroke: "currentColor", "stroke-width": "1.8", "stroke-linecap": "round", "stroke-linejoin": "round",
                        path { d: "M8 4h8v5a4 4 0 0 1-8 0Z M8 6H5v1a3 3 0 0 0 3 3 M16 6h3v1a3 3 0 0 1-3 3 M12 13v4 M8 20h8 M9 17h6" }
                    }
                    span { class: "kit-rb__champ-name", "Winner {entrant.entrant_name}" }
                    if let (Some(handler), Some(label)) = (on_adopt, adopt_label.clone()) {
                        button { r#type: "button", class: "kit-rb__adopt", onclick: move |_| handler.call(entrant.clone()), "{label}" }
                    }
                }
            }
        }
    }
}
