//! Agent clash map: where several people's agents collide. Agents are chips on
//! the two edges, clashes are labelled badges in the middle, and each
//! clash-and-agent pair is one curve. Pointing at a clash or an agent brings its
//! curves forward and lets the rest recede.
//!
//! Design choice: chips fill the left edge first, then the right; badges keep
//! clash order. Geometry is percent of width for x and px for y, so curves need
//! no measurement. The one measured input is the container width, which only
//! switches to the narrow layout.
//! Design choice: a controlled `selected_*` prop wins while it is `Some`; the
//! map's own state applies otherwise.
//! Design choice: outside-click is a document listener. Chips and badges are
//! exempt, so a click on another one moves the card instead of closing it.
//! Browser checks: curves draw in, badges settle in turn, decide badges breathe,
//! pointing emphasises one clash, Escape returns focus, a keyboard open moves
//! focus into the card, and reduced motion removes all of it.

use std::collections::BTreeMap;

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, focus_after_frame, next_id, use_element_width};

const CSS: Asset = asset!("/assets/agent-clash-map.css");

/// Container width below which every chip shares the left edge.
pub const NARROW_PX: f64 = 520.0;
const ROW_PX: f64 = 64.0;
const CHIP_PX: f64 = 52.0;
const EDGE_PAD_PX: f64 = 12.0;
const WIDE_LEFT_X: f64 = 28.0;
const WIDE_RIGHT_X: f64 = 72.0;
const WIDE_BADGE_X: f64 = 50.0;
const NARROW_CHIP_X: f64 = 46.0;
const NARROW_BADGE_X: f64 = 54.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClashWeight {
    Decide,
    Notice,
    Faint,
}

impl ClashWeight {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Decide => "decide",
            Self::Notice => "notice",
            Self::Faint => "faint",
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::Decide => "Decide",
            Self::Notice => "Notice",
            Self::Faint => "Faint",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Side {
    pub participant_id: String,
    pub agent_name: String,
    pub harness_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Clash {
    pub id: String,
    pub weight: ClashWeight,
    pub label: String,
    pub summary: String,
    pub sides: Vec<Side>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Harness {
    pub display_name: Option<String>,
    pub mark: Option<String>,
}

/// What a card or an emphasis points at: a clash, or one agent of one person.
#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    Clash(String),
    Agent(String, String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emphasis {
    Normal,
    Full,
    Faded,
}

impl Emphasis {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Full => "full",
            Self::Faded => "faded",
        }
    }
}

/// One agent chip: a (person, agent name) pair that appears in at least one clash.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentChip {
    pub participant_id: String,
    pub person: String,
    pub note: Option<String>,
    pub agent_name: String,
    pub harness_key: Option<String>,
}

/// The clashes, chips and curves left after normalisation.
#[derive(Clone, Debug, PartialEq)]
pub struct Map {
    pub clashes: Vec<Clash>,
    pub agents: Vec<AgentChip>,
    /// (clash index, agent index), one per clash-and-agent pair.
    pub edges: Vec<(usize, usize)>,
    /// Full names of participants who appear in no clash.
    pub in_sync: Vec<String>,
}

impl Map {
    pub fn agent_index(&self, participant_id: &str, agent_name: &str) -> Option<usize> {
        self.agents
            .iter()
            .position(|a| a.participant_id == participant_id && a.agent_name == agent_name)
    }

    pub fn clash_index(&self, clash_id: &str) -> Option<usize> {
        self.clashes.iter().position(|c| c.id == clash_id)
    }

    pub fn agent_clashes(&self, agent: usize) -> Vec<usize> {
        self.edges
            .iter()
            .filter(|(_, a)| *a == agent)
            .map(|(c, _)| *c)
            .collect()
    }

    pub fn clash_agents(&self, clash: usize) -> Vec<usize> {
        self.edges
            .iter()
            .filter(|(c, _)| *c == clash)
            .map(|(_, a)| *a)
            .collect()
    }
}

/// Applies the spec's normalisation: unknown sides dropped, a repeated agent in
/// one clash counted once, clashes left empty removed, and chips shared across
/// clashes.
///
/// Chips are ordered so that agents which lead a clash (its first side) come
/// first, in clash order, and agents that only ever answer come after. The left
/// edge takes the first half, so the leading side of each clash sits on the left
/// and its answering side on the right.
pub fn normalise(participants: &[Participant], clashes: &[Clash]) -> Map {
    let mut map = Map {
        clashes: Vec::new(),
        agents: Vec::new(),
        edges: Vec::new(),
        in_sync: Vec::new(),
    };
    let mut leads: Vec<bool> = Vec::new();
    let mut in_a_clash: Vec<&str> = Vec::new();
    for clash in clashes {
        let mut sides: Vec<Side> = Vec::new();
        for side in &clash.sides {
            let Some(person) = participants.iter().find(|p| p.id == side.participant_id) else {
                continue;
            };
            let repeat = sides.iter().any(|s| {
                s.participant_id == side.participant_id && s.agent_name == side.agent_name
            });
            if repeat {
                continue;
            }
            let leading = sides.is_empty();
            sides.push(side.clone());
            let agent = match map.agent_index(&side.participant_id, &side.agent_name) {
                Some(index) => index,
                None => {
                    map.agents.push(AgentChip {
                        participant_id: person.id.clone(),
                        person: person.name.clone(),
                        note: person.note.clone(),
                        agent_name: side.agent_name.clone(),
                        harness_key: side.harness_key.clone(),
                    });
                    leads.push(false);
                    map.agents.len() - 1
                }
            };
            leads[agent] |= leading;
            map.edges.push((map.clashes.len(), agent));
            in_a_clash.push(&person.id);
        }
        if sides.is_empty() {
            continue;
        }
        map.clashes.push(Clash {
            sides,
            ..clash.clone()
        });
    }
    let mut order: Vec<usize> = (0..map.agents.len()).collect();
    order.sort_by_key(|&a| !leads[a]);
    let mut position = vec![0; order.len()];
    for (new, &old) in order.iter().enumerate() {
        position[old] = new;
    }
    map.agents = order.iter().map(|&old| map.agents[old].clone()).collect();
    for edge in &mut map.edges {
        edge.1 = position[edge.1];
    }
    map.in_sync = participants
        .iter()
        .filter(|p| !in_a_clash.contains(&p.id.as_str()))
        .map(|p| p.name.clone())
        .collect();
    map
}

/// The first word with a letter, with leading symbols and emoji removed. Falls
/// back to the whole name when no word has a letter.
pub fn first_name(full: &str) -> String {
    full.split_whitespace()
        .find(|word| word.chars().any(char::is_alphabetic))
        .map(|word| word.trim_start_matches(|c: char| !c.is_alphanumeric()))
        .filter(|word| !word.is_empty())
        .map_or_else(|| full.trim().to_string(), str::to_string)
}

/// `1 clash`, `2 clashes`.
pub fn count_line(count: usize) -> String {
    if count == 1 {
        "1 clash".to_string()
    } else {
        format!("{count} clashes")
    }
}

/// The chip's accessible name: person, agent, and how many clashes it is in.
pub fn agent_label(person: &str, agent: &str, clashes: usize) -> String {
    format!("{person}, {agent}, {}", count_line(clashes))
}

/// Chips on the left edge in a wide layout: the first half, rounded up.
pub fn left_count(chips: usize) -> usize {
    chips.div_ceil(2)
}

/// Stage height: enough rows for the busiest column, never below `min_height`.
pub fn stage_height(left: usize, right: usize, badges: usize, min_height: f64) -> f64 {
    let rows = left.max(right).max(badges).max(1) as f64;
    (rows * ROW_PX + 2.0 * EDGE_PAD_PX).max(min_height)
}

/// Vertical centres of `count` slots spread evenly down `height`. With
/// `height >= count * ROW_PX`, neighbouring chips never overlap.
pub fn slot_centres(count: usize, height: f64) -> Vec<f64> {
    (0..count)
        .map(|i| (i as f64 + 0.5) * height / count as f64)
        .collect()
}

/// Cubic curve from an agent to a badge, in the stage's percent-x and px-y units.
pub fn curve_path(from_x: f64, from_y: f64, to_x: f64, to_y: f64) -> String {
    let mid = (from_x + to_x) / 2.0;
    format!("M {from_x} {from_y} C {mid} {from_y} {mid} {to_y} {to_x} {to_y}")
}

/// Emphasis of one curve or badge. `members` are the (person, agent) pairs the
/// element joins; a clash focus lights the clash's own curves and badge.
pub fn emphasis(focus: Option<&Target>, clash_id: &str, members: &[(&str, &str)]) -> Emphasis {
    let Some(focus) = focus else {
        return Emphasis::Normal;
    };
    let hit = match focus {
        Target::Clash(id) => id == clash_id,
        Target::Agent(person, agent) => members.iter().any(|(p, a)| p == person && a == agent),
    };
    if hit { Emphasis::Full } else { Emphasis::Faded }
}

/// Whether a chip or card target still names something in the map.
pub fn target_exists(map: &Map, target: &Target) -> bool {
    match target {
        Target::Clash(id) => map.clash_index(id).is_some(),
        Target::Agent(person, agent) => map.agent_index(person, agent).is_some(),
    }
}

/// `In sync: Ana, Ben`, or nothing when every participant is in a clash.
pub fn sync_line(names: &[String]) -> Option<String> {
    (!names.is_empty()).then(|| format!("In sync: {}", names.join(", ")))
}

/// Identity slot (1..=6) per key, so one agent keeps its colour across renders.
pub fn accent_slot(key: &str) -> u32 {
    key.bytes().fold(0u32, |hash, b| {
        hash.wrapping_mul(31).wrapping_add(u32::from(b))
    }) % 6
        + 1
}

/// First letter of a name, uppercased, for an agent without a harness mark.
pub fn monogram(name: &str) -> String {
    name.chars()
        .find(|c| c.is_alphabetic())
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().to_string())
}

fn outside_script(root: &str) -> String {
    format!(
        r#"const key = '__kitAcm{root}';
window[key]?.();
const onDown = (event) => {{
    if (event.target.closest?.('[data-acm-card="{root}"], [data-acm-node="{root}"]')) return;
    cleanup();
    dioxus.send(true);
}};
const cleanup = () => {{ document.removeEventListener('pointerdown', onDown, true); delete window[key]; }};
window[key] = () => {{ cleanup(); dioxus.send(false); }};
document.addEventListener('pointerdown', onDown, true);
await new Promise(() => {{}});"#
    )
}

fn cleanup_script(root: &str) -> String {
    format!("window['__kitAcm{root}']?.();")
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ChipGeom {
    is_left: bool,
    y: f64,
    anchor_x: f64,
}

#[derive(Clone)]
struct Hooks {
    open: Signal<Option<Target>>,
    opener: Signal<String>,
    keyboard: Signal<bool>,
    focus_card: Signal<bool>,
    hover: Signal<Option<Target>>,
    on_clash: Option<EventHandler<Option<String>>>,
    on_person: Option<EventHandler<Option<String>>>,
    on_hover: Option<EventHandler<Option<String>>>,
}

impl Hooks {
    fn emit(&self, target: &Target, open: bool) {
        match target {
            Target::Clash(id) => {
                if let Some(h) = &self.on_clash {
                    h.call(open.then(|| id.clone()));
                }
            }
            Target::Agent(person, _) => {
                if let Some(h) = &self.on_person {
                    h.call(open.then(|| person.clone()));
                }
            }
        }
    }

    fn activate(&self, target: Target, dom_id: String) {
        let mut open = self.open;
        let previous = open.peek().clone();
        let next = if previous.as_ref() == Some(&target) {
            None
        } else {
            Some(target)
        };
        let mut keyboard = self.keyboard;
        let from_keys = keyboard();
        keyboard.set(false);
        let mut focus_card = self.focus_card;
        focus_card.set(from_keys && next.is_some());
        let mut opener = self.opener;
        opener.set(dom_id);
        if previous != next {
            if let Some(prev) = &previous {
                self.emit(prev, false);
            }
            if let Some(target) = &next {
                self.emit(target, true);
            }
        }
        open.set(next);
    }

    fn close(&self) {
        let mut open = self.open;
        let previous = open.peek().clone();
        open.set(None);
        if let Some(prev) = previous {
            self.emit(&prev, false);
        }
    }

    fn keyed(&self, event: &KeyboardEvent) {
        if event.key() == Key::Enter || event.key() == Key::Character(" ".to_string()) {
            let mut keyboard = self.keyboard;
            keyboard.set(true);
        }
    }

    fn enter(&self, target: Target) {
        if let (Target::Clash(id), Some(h)) = (&target, &self.on_hover) {
            h.call(Some(id.clone()));
        }
        let mut hover = self.hover;
        hover.set(Some(target));
    }

    fn leave(&self, target: &Target) {
        let mut hover = self.hover;
        if hover.peek().clone().as_ref() == Some(target) {
            hover.set(None);
        }
        if let (Target::Clash(_), Some(h)) = (target, &self.on_hover) {
            h.call(None);
        }
    }
}

fn clash_card(map: &Map, clash: usize) -> Element {
    let item = &map.clashes[clash];
    let kicker = format!("{} clash", item.weight.word());
    let weight = item.weight.as_str();
    let title = item.label.clone();
    let summary = item.summary.clone();
    let sides: Vec<(String, String)> = item
        .sides
        .iter()
        .map(|side| {
            let person = map
                .agents
                .iter()
                .find(|a| a.participant_id == side.participant_id)
                .map_or_else(String::new, |a| a.person.clone());
            (person, side.agent_name.clone())
        })
        .collect();
    rsx! {
        p { class: "kit-acm__kicker", "data-weight": "{weight}", "{kicker}" }
        h3 { class: "kit-acm__card-title", "{title}" }
        p { class: "kit-acm__card-text", "{summary}" }
        ul { class: "kit-acm__sides", "aria-label": "Sides",
            for (person, agent) in sides {
                li { class: "kit-acm__side",
                    span { class: "kit-acm__side-who", "{person}" }
                    span { class: "kit-acm__side-what", "{agent}" }
                }
            }
        }
    }
}

fn agent_card(map: &Map, harnesses: &BTreeMap<String, Harness>, agent: usize) -> Element {
    let chip = &map.agents[agent];
    let harness = chip.harness_key.as_ref().and_then(|k| harnesses.get(k));
    let heading = match harness.and_then(|h| h.display_name.clone()) {
        Some(name) => format!("{} · {name}", chip.agent_name),
        None => chip.agent_name.clone(),
    };
    let person = chip.person.clone();
    let note = chip.note.clone().unwrap_or_default();
    let has_note = chip.note.is_some();
    let rows: Vec<(String, String, String, String)> = map
        .agent_clashes(agent)
        .into_iter()
        .map(|c| {
            let clash = &map.clashes[c];
            (
                clash.weight.as_str().to_string(),
                clash.weight.word().to_string(),
                clash.label.clone(),
                clash.summary.clone(),
            )
        })
        .collect();
    rsx! {
        p { class: "kit-acm__kicker", "Agent" }
        h3 { class: "kit-acm__card-title", "{heading}" }
        p { class: "kit-acm__card-meta", "{person}" }
        if has_note {
            p { class: "kit-acm__note", "{note}" }
        }
        ul { class: "kit-acm__list", "aria-label": "Clashes",
            for (weight, word, label, summary) in rows {
                li { class: "kit-acm__row", "data-weight": "{weight}",
                    span { class: "kit-acm__row-tag", "{word}" }
                    span { class: "kit-acm__row-label", "{label}" }
                    span { class: "kit-acm__row-text", "{summary}" }
                }
            }
        }
    }
}

struct ChipView {
    dom_id: String,
    target: Target,
    is_left: bool,
    top: f64,
    mark: String,
    slot: u32,
    display: String,
    person: String,
    aria: String,
    expanded: bool,
    wave: bool,
}

struct BadgeView {
    dom_id: String,
    target: Target,
    weight: &'static str,
    word: &'static str,
    label: String,
    aria: String,
    style: String,
    expanded: bool,
    emph: Emphasis,
}

struct CurveView {
    key: String,
    d: String,
    weight: &'static str,
    emph: Emphasis,
}

/// The agents of several people and the clashes between them, drawn as one map.
/// A card opens beside the chip or badge that opened it.
#[component]
pub fn AgentClashMap(
    participants: Vec<Participant>,
    clashes: Vec<Clash>,
    #[props(default)] harnesses: BTreeMap<String, Harness>,
    #[props(default)] selected_clash_id: Option<String>,
    #[props(default)] selected_participant_id: Option<String>,
    #[props(default)] highlighted_clash_id: Option<String>,
    #[props(default)] on_select_clash: Option<EventHandler<Option<String>>>,
    #[props(default)] on_select_participant: Option<EventHandler<Option<String>>>,
    #[props(default)] on_highlight_clash: Option<EventHandler<Option<String>>>,
    #[props(default)] min_height: Option<u32>,
    #[props(default = "Agent clashes".to_string(), into)] heading: String,
    #[props(default)] extra_class: String,
) -> Element {
    let root_id = use_hook(|| format!("kit-acm-{}", next_id()));
    let width = use_element_width(root_id.clone());
    let narrow = width.is_some_and(|w| w < NARROW_PX);

    let internal = use_signal(|| None::<Target>);
    let hover = use_signal(|| None::<Target>);
    let opener = use_signal(String::new);
    let keyboard = use_signal(|| false);
    let focus_card = use_signal(|| false);

    let map = normalise(&participants, &clashes);
    let controlled = selected_clash_id.clone().map(Target::Clash).or_else(|| {
        let person = selected_participant_id.as_deref()?;
        map.agents
            .iter()
            .find(|a| a.participant_id == person)
            .map(|a| Target::Agent(a.participant_id.clone(), a.agent_name.clone()))
    });
    let open_card = controlled
        .or_else(|| internal())
        .filter(|t| target_exists(&map, t));
    let focus = highlighted_clash_id
        .clone()
        .map(Target::Clash)
        .or_else(|| hover());
    let focused_clash = match &focus {
        Some(Target::Clash(id)) => map.clash_index(id),
        _ => None,
    };

    let hooks = Hooks {
        open: internal,
        opener,
        keyboard,
        focus_card,
        hover,
        on_clash: on_select_clash,
        on_person: on_select_participant,
        on_hover: on_highlight_clash,
    };

    let is_open = open_card.is_some();
    {
        let hooks = hooks.clone();
        let root = root_id.clone();
        use_effect(use_reactive((&is_open,), move |(open,)| {
            if open {
                let script = outside_script(&root);
                let hooks = hooks.clone();
                spawn(async move {
                    let mut eval = document::eval(&script);
                    if let Ok(true) = eval.recv::<bool>().await {
                        hooks.close();
                    }
                });
            } else {
                document::eval(&cleanup_script(&root));
            }
        }));
    }

    let class = class_list("kit-acm", &extra_class);
    let count = count_line(map.clashes.len());
    let sync = sync_line(&map.in_sync);
    let announce = match &open_card {
        Some(Target::Clash(id)) => map
            .clash_index(id)
            .map(|c| format!("Details for {} open.", map.clashes[c].label)),
        Some(Target::Agent(_, agent)) => Some(format!("Details for {agent} open.")),
        None => None,
    };

    if map.clashes.is_empty() {
        return rsx! {
            document::Link { rel: "stylesheet", href: KIT_CSS }
            document::Link { rel: "stylesheet", href: CSS }
            section { id: "{root_id}", class, "aria-label": "{heading}",
                header { class: "kit-acm__head",
                    h2 { class: "kit-acm__title", "{heading}" }
                    span { class: "kit-acm__count", "{count}" }
                }
                p { class: "kit-acm__empty", "Everyone is in sync." }
                if let Some(line) = sync {
                    p { class: "kit-acm__sync", "{line}" }
                }
            }
        };
    }

    let total = map.agents.len();
    let left_n = if narrow { total } else { left_count(total) };
    let right_n = total - left_n;
    let badge_count = map.clashes.len();
    let height = stage_height(
        left_n,
        right_n,
        badge_count,
        f64::from(min_height.unwrap_or(0)),
    );
    let left_ys = slot_centres(left_n, height);
    let right_ys = slot_centres(right_n, height);
    let badge_ys = slot_centres(badge_count, height);
    let badge_x = if narrow { NARROW_BADGE_X } else { WIDE_BADGE_X };

    let geom: Vec<ChipGeom> = (0..total)
        .map(|index| {
            let (is_left, y) = if index < left_n {
                (true, left_ys[index])
            } else {
                (false, right_ys[index - left_n])
            };
            let anchor_x = match (narrow, is_left) {
                (true, _) => NARROW_CHIP_X,
                (false, true) => WIDE_LEFT_X,
                (false, false) => WIDE_RIGHT_X,
            };
            ChipGeom {
                is_left,
                y,
                anchor_x,
            }
        })
        .collect();

    let curves: Vec<CurveView> = map
        .edges
        .iter()
        .map(|&(c, a)| {
            let clash = &map.clashes[c];
            let agent = &map.agents[a];
            let members = [(agent.participant_id.as_str(), agent.agent_name.as_str())];
            CurveView {
                key: format!("{c}-{a}"),
                d: curve_path(geom[a].anchor_x, geom[a].y, badge_x, badge_ys[c]),
                weight: clash.weight.as_str(),
                emph: emphasis(focus.as_ref(), &clash.id, &members),
            }
        })
        .collect();

    let chips: Vec<ChipView> = map
        .agents
        .iter()
        .enumerate()
        .map(|(a, agent)| {
            let harness = agent.harness_key.as_ref().and_then(|k| harnesses.get(k));
            let target = Target::Agent(agent.participant_id.clone(), agent.agent_name.clone());
            let wave = focused_clash.is_some_and(|c| map.edges.contains(&(c, a)));
            ChipView {
                dom_id: format!("{root_id}-agent-{a}"),
                expanded: open_card.as_ref() == Some(&target),
                target,
                is_left: geom[a].is_left,
                top: geom[a].y - CHIP_PX / 2.0,
                mark: harness
                    .and_then(|h| h.mark.clone())
                    .unwrap_or_else(|| monogram(&agent.agent_name)),
                slot: accent_slot(agent.harness_key.as_deref().unwrap_or(&agent.agent_name)),
                display: agent.agent_name.clone(),
                person: first_name(&agent.person),
                aria: agent_label(&agent.person, &agent.agent_name, map.agent_clashes(a).len()),
                wave,
            }
        })
        .collect();

    let badges: Vec<BadgeView> = map
        .clashes
        .iter()
        .enumerate()
        .map(|(c, clash)| {
            let members: Vec<(&str, &str)> = map
                .clash_agents(c)
                .into_iter()
                .map(|a| {
                    (
                        map.agents[a].participant_id.as_str(),
                        map.agents[a].agent_name.as_str(),
                    )
                })
                .collect();
            let target = Target::Clash(clash.id.clone());
            BadgeView {
                dom_id: format!("{root_id}-clash-{c}"),
                expanded: open_card.as_ref() == Some(&target),
                target,
                weight: clash.weight.as_str(),
                word: clash.weight.word(),
                label: clash.label.clone(),
                aria: format!("{}. {}", clash.label, clash.summary),
                style: if narrow {
                    format!("top: {}px; --acm-i: {c};", badge_ys[c])
                } else {
                    format!("left: {badge_x}%; top: {}px; --acm-i: {c};", badge_ys[c])
                },
                emph: emphasis(focus.as_ref(), &clash.id, &members),
            }
        })
        .collect();

    let card_view = open_card.as_ref().map(|target| {
        let (anchor_x, anchor_y, title, body) = match target {
            Target::Clash(id) => {
                let c = map.clash_index(id).unwrap_or(0);
                (
                    badge_x,
                    badge_ys[c],
                    map.clashes[c].label.clone(),
                    clash_card(&map, c),
                )
            }
            Target::Agent(person, agent_name) => {
                let a = map.agent_index(person, agent_name).unwrap_or(0);
                let x = match (narrow, geom[a].is_left) {
                    (true, _) => NARROW_CHIP_X / 2.0,
                    (false, true) => 14.0,
                    (false, false) => 86.0,
                };
                (
                    x,
                    geom[a].y,
                    agent_name.clone(),
                    agent_card(&map, &harnesses, a),
                )
            }
        };
        let vertical = if anchor_y > height / 2.0 {
            format!("bottom: {}px", height - anchor_y + CHIP_PX / 2.0 + 8.0)
        } else {
            format!("top: {}px", anchor_y + CHIP_PX / 2.0 + 8.0)
        };
        let style = format!(
            "left: clamp(0px, calc({anchor_x}% - 9rem), calc(100% - min(18rem, 100%))); {vertical}"
        );
        (style, title, body)
    });

    let left_chips: Vec<&ChipView> = chips.iter().filter(|c| c.is_left).collect();
    let right_chips: Vec<&ChipView> = chips.iter().filter(|c| !c.is_left).collect();
    let stage_narrow = narrow.to_string();
    let card_id = format!("{root_id}-card");
    let escape_hooks = hooks.clone();
    let card_hooks = hooks.clone();
    let heading_label = heading.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            id: "{root_id}",
            class,
            "aria-label": "{heading_label}",
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape && escape_hooks.open.peek().is_some() {
                    event.prevent_default();
                    let id = escape_hooks.opener.peek().clone();
                    escape_hooks.close();
                    focus_after_frame(&id);
                }
            },
            header { class: "kit-acm__head",
                h2 { class: "kit-acm__title", "{heading}" }
                span { class: "kit-acm__count", "{count}" }
                ul { class: "kit-acm__legend", "aria-label": "Weights",
                    li { "data-weight": "decide",
                        span { class: "kit-acm__swatch", "aria-hidden": "true" }
                        "Decide: needs a person now"
                    }
                    li { "data-weight": "notice",
                        span { class: "kit-acm__swatch", "aria-hidden": "true" }
                        "Notice: a heads-up"
                    }
                    li { "data-weight": "faint",
                        span { class: "kit-acm__swatch", "aria-hidden": "true" }
                        "Faint: background"
                    }
                }
            }
            div {
                class: "kit-acm__stage",
                "data-narrow": "{stage_narrow}",
                style: "height: {height}px;",
                svg {
                    class: "kit-acm__curves",
                    "viewBox": "0 0 100 {height}",
                    "preserveAspectRatio": "none",
                    "aria-hidden": "true",
                    for curve in curves {
                        path {
                            key: "{curve.key}",
                            class: "kit-acm__curve",
                            "data-weight": "{curve.weight}",
                            "data-emph": "{curve.emph.as_str()}",
                            d: "{curve.d}",
                        }
                    }
                }
                for chip in left_chips {
                    button {
                        key: "{chip.dom_id}",
                        id: "{chip.dom_id}",
                        r#type: "button",
                        class: "kit-acm__chip",
                        "data-side": "left",
                        "data-acm-node": "{root_id}",
                        "aria-haspopup": "dialog",
                        "aria-expanded": if chip.expanded { "true" } else { "false" },
                        "aria-label": "{chip.aria}",
                        style: "top: {chip.top}px;",
                        onclick: {
                            let hooks = hooks.clone();
                            let target = chip.target.clone();
                            let dom_id = chip.dom_id.clone();
                            move |_| hooks.activate(target.clone(), dom_id.clone())
                        },
                        onkeydown: {
                            let hooks = hooks.clone();
                            move |event: KeyboardEvent| hooks.keyed(&event)
                        },
                        onfocus: {
                            let hooks = hooks.clone();
                            let target = chip.target.clone();
                            move |_| hooks.enter(target.clone())
                        },
                        onblur: {
                            let hooks = hooks.clone();
                            let target = chip.target.clone();
                            move |_| hooks.leave(&target)
                        },
                        onmouseenter: {
                            let hooks = hooks.clone();
                            let target = chip.target.clone();
                            move |_| hooks.enter(target.clone())
                        },
                        onmouseleave: {
                            let hooks = hooks.clone();
                            let target = chip.target.clone();
                            move |_| hooks.leave(&target)
                        },
                        span {
                            class: if chip.wave { "kit-acm__mark is-wave" } else { "kit-acm__mark" },
                            "aria-hidden": "true",
                            style: "--acm-slot: var(--kit-id-{chip.slot}); --acm-slot-soft: var(--kit-id-{chip.slot}-soft);",
                            "{chip.mark}"
                        }
                        span { class: "kit-acm__who",
                            span { class: "kit-acm__agent", "{chip.display}" }
                            span { class: "kit-acm__person", "{chip.person}" }
                        }
                    }
                }
                for badge in badges {
                    button {
                        key: "{badge.dom_id}",
                        id: "{badge.dom_id}",
                        r#type: "button",
                        class: "kit-acm__badge",
                        "data-weight": "{badge.weight}",
                        "data-emph": "{badge.emph.as_str()}",
                        "data-acm-node": "{root_id}",
                        "aria-haspopup": "dialog",
                        "aria-expanded": if badge.expanded { "true" } else { "false" },
                        "aria-label": "{badge.aria}",
                        style: "{badge.style}",
                        onclick: {
                            let hooks = hooks.clone();
                            let target = badge.target.clone();
                            let dom_id = badge.dom_id.clone();
                            move |_| hooks.activate(target.clone(), dom_id.clone())
                        },
                        onkeydown: {
                            let hooks = hooks.clone();
                            move |event: KeyboardEvent| hooks.keyed(&event)
                        },
                        onfocus: {
                            let hooks = hooks.clone();
                            let target = badge.target.clone();
                            move |_| hooks.enter(target.clone())
                        },
                        onblur: {
                            let hooks = hooks.clone();
                            let target = badge.target.clone();
                            move |_| hooks.leave(&target)
                        },
                        onmouseenter: {
                            let hooks = hooks.clone();
                            let target = badge.target.clone();
                            move |_| hooks.enter(target.clone())
                        },
                        onmouseleave: {
                            let hooks = hooks.clone();
                            let target = badge.target.clone();
                            move |_| hooks.leave(&target)
                        },
                        span { class: "kit-acm__weight", "{badge.word}" }
                        span { class: "kit-acm__label", "{badge.label}" }
                    }
                }
                for chip in right_chips {
                    button {
                        key: "{chip.dom_id}",
                        id: "{chip.dom_id}",
                        r#type: "button",
                        class: "kit-acm__chip",
                        "data-side": "right",
                        "data-acm-node": "{root_id}",
                        "aria-haspopup": "dialog",
                        "aria-expanded": if chip.expanded { "true" } else { "false" },
                        "aria-label": "{chip.aria}",
                        style: "top: {chip.top}px;",
                        onclick: {
                            let hooks = hooks.clone();
                            let target = chip.target.clone();
                            let dom_id = chip.dom_id.clone();
                            move |_| hooks.activate(target.clone(), dom_id.clone())
                        },
                        onkeydown: {
                            let hooks = hooks.clone();
                            move |event: KeyboardEvent| hooks.keyed(&event)
                        },
                        onfocus: {
                            let hooks = hooks.clone();
                            let target = chip.target.clone();
                            move |_| hooks.enter(target.clone())
                        },
                        onblur: {
                            let hooks = hooks.clone();
                            let target = chip.target.clone();
                            move |_| hooks.leave(&target)
                        },
                        onmouseenter: {
                            let hooks = hooks.clone();
                            let target = chip.target.clone();
                            move |_| hooks.enter(target.clone())
                        },
                        onmouseleave: {
                            let hooks = hooks.clone();
                            let target = chip.target.clone();
                            move |_| hooks.leave(&target)
                        },
                        span {
                            class: if chip.wave { "kit-acm__mark is-wave" } else { "kit-acm__mark" },
                            "aria-hidden": "true",
                            style: "--acm-slot: var(--kit-id-{chip.slot}); --acm-slot-soft: var(--kit-id-{chip.slot}-soft);",
                            "{chip.mark}"
                        }
                        span { class: "kit-acm__who",
                            span { class: "kit-acm__agent", "{chip.display}" }
                            span { class: "kit-acm__person", "{chip.person}" }
                        }
                    }
                }
                if let Some((style, title, body)) = card_view {
                    section {
                        id: "{card_id}",
                        class: "kit-acm__card",
                        role: "region",
                        "aria-label": "{title}",
                        tabindex: "-1",
                        "data-acm-card": "{root_id}",
                        style: "{style}",
                        onmounted: {
                            let hooks = card_hooks.clone();
                            move |event: MountedEvent| {
                                let mut focus = hooks.focus_card;
                                if focus() {
                                    focus.set(false);
                                    let node = event.data();
                                    spawn(async move {
                                        let _ = node.set_focus(true).await;
                                    });
                                }
                            }
                        },
                        {body}
                    }
                }
            }
            div { class: "kit-acm__sr", role: "status", "aria-live": "polite",
                {announce.unwrap_or_default()}
            }
            if let Some(line) = sync {
                p { class: "kit-acm__sync", "{line}" }
            }
        }
    }
}
