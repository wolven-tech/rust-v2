//! Co-editing presence: agents working inside a shared document, shown as
//! in-text pointers, suggested edits and margin notes. Every suggested edit
//! waits for the person to accept or decline it.
//!
//! Design choice: a pointer is a zero-width caret inside the text at its
//! character offset, so it stays on its character at any width and rewraps
//! with the text. No measuring pass is needed.
//! Design choice: `followed_id` seeds the local follow state; the host learns
//! of changes through `on_follow_change`.
//! Design choice: spend reads in dollars, as in `delegation_card_tree`.
//! Design choice: the kit has no hue tokens, so palette keys mark a participant
//! by data attribute only; marks use the ok and fault tones and the caret ink.
//! Design choice: the live line names the latest proposal that arrived after
//! mount. Proposals present at mount are not announced.
//! Not implemented: text selections (the `selection` field is not carried),
//! image resize handles (`handle`), and the `in_proposal` position flag.
//! Browser checks: pointers glide between offsets and fade on removal; a
//! click outside an open window closes it; Escape returns focus to the mark;
//! image resize updates `column_fraction`; reduced motion applies changes at
//! once; the page fades toward `page_height`.

use std::collections::BTreeMap;

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id};

/// Gap between a pin and its note window, in px.
const PIN_WINDOW_GAP_PX: f64 = 24.0;

/// Top of a note window in px from its pin's top edge. Below the pin when it fits
/// in the page, else above it, else pulled up so its bottom meets the page's.
pub fn pin_window_top(page_h: f64, pin_top: f64, window_h: f64) -> f64 {
    let below = PIN_WINDOW_GAP_PX;
    if pin_top + below + window_h <= page_h {
        return below;
    }
    let above = -(PIN_WINDOW_GAP_PX + window_h);
    if pin_top + above >= 0.0 {
        return above;
    }
    (page_h - window_h).max(0.0) - pin_top
}

const CSS: Asset = asset!("/assets/co-editing-presence.css");

#[derive(Clone, Debug, PartialEq)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub palette_key: Option<String>,
    pub picture: Option<String>,
    pub wash_key: Option<String>,
    pub spend: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Title,
    Heading,
    Paragraph,
    Item,
    Code,
    Image,
    Table,
}

impl BlockKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Heading => "heading",
            Self::Paragraph => "paragraph",
            Self::Item => "item",
            Self::Code => "code",
            Self::Image => "image",
            Self::Table => "table",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WrapSide {
    Left,
    Right,
}

impl WrapSide {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImageBlock {
    pub column_fraction: f64,
    pub aspect: f64,
    pub wrap_side: WrapSide,
    pub is_loading: bool,
    pub source_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TableCell {
    pub text: String,
    pub badge_key: Option<String>,
}

/// A document block. The text is the caption for an image.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub id: String,
    pub kind: BlockKind,
    pub text: String,
    pub image: Option<ImageBlock>,
    /// Rows of cells; the first row names the columns. Empty for other kinds.
    pub cell_grid: Vec<Vec<TableCell>>,
}

/// Where a participant is. `offset` is a character index into the block text.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Position {
    pub block_id: String,
    pub offset: Option<usize>,
    pub in_proposal: Option<String>,
    pub activity: Option<String>,
    pub is_typing: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pin {
    pub id: String,
    pub participant_id: String,
    pub block_id: String,
    /// Character anchor. Without one, the pin sits on the block's top edge at
    /// `horizontal_fraction` across the column.
    pub offset: Option<usize>,
    pub horizontal_fraction: Option<f64>,
    pub vertical_offset: Option<f64>,
    pub body: String,
    pub is_open: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Proposal {
    pub id: String,
    pub participant_id: String,
    pub block_id: String,
    pub offset: usize,
    pub removal: Option<String>,
    pub insertion: Option<String>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct View {
    pub id: String,
    pub caption: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presence {
    Absent,
    Idle,
    Typing,
}

impl Presence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Idle => "idle",
            Self::Typing => "typing",
        }
    }
}

/// The result of accepting a proposal: the edited blocks and the proposals
/// that remain pending.
#[derive(Clone, Debug, PartialEq)]
pub struct AppliedEdit {
    pub blocks: Vec<Block>,
    pub proposals: Vec<Proposal>,
}

/// One run of rendered text, or a point marker, inside a block.
#[derive(Clone, Debug, PartialEq)]
pub enum MarkSegment {
    Text(String),
    Removed {
        proposal_id: String,
        participant_id: String,
        text: String,
        focusable: bool,
    },
    Inserted {
        proposal_id: String,
        participant_id: String,
        text: String,
        focusable: bool,
    },
    Caret {
        participant_id: String,
    },
    Pin {
        pin_id: String,
    },
}

/// Fraction clamped to 0..=1; NaN reads as 0.
pub fn unit_fraction(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

/// Dollars to two places; a positive amount under one cent reads as
/// "less than one cent".
pub fn format_spend(amount: f64) -> String {
    if !amount.is_finite() || amount <= 0.0 {
        "$0.00".to_string()
    } else if amount < 0.01 {
        "less than one cent".to_string()
    } else {
        format!("${amount:.2}")
    }
}

/// Offset clamped to the character length of `text`.
pub fn clamp_offset(offset: usize, text: &str) -> usize {
    offset.min(text.chars().count())
}

pub fn presence(position: Option<&Position>) -> Presence {
    match position {
        None => Presence::Absent,
        Some(p) if p.is_typing => Presence::Typing,
        Some(_) => Presence::Idle,
    }
}

/// The face's accessible name: the participant and its current activity.
pub fn face_label(name: &str, position: Option<&Position>) -> String {
    match position {
        None => format!("{name}, away"),
        Some(p) => match p.activity.as_deref().filter(|a| !a.is_empty()) {
            Some(activity) => format!("{name}: {activity}"),
            None => format!("{name}, in document"),
        },
    }
}

/// Pointer flag text: name, activity and spend, joined by a middle dot.
pub fn pointer_flag(name: &str, activity: Option<&str>, spend: Option<f64>) -> String {
    let mut parts = vec![name.to_string()];
    if let Some(activity) = activity.filter(|a| !a.is_empty()) {
        parts.push(activity.to_string());
    }
    if let Some(amount) = spend {
        parts.push(format_spend(amount));
    }
    parts.join(" · ")
}

/// Single-letter marker for a participant with no picture.
pub fn glyph_for(name: &str) -> String {
    name.chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string())
}

/// A pin window opens left when its anchor sits near the right edge.
pub fn pin_window_side(horizontal_fraction: f64) -> &'static str {
    if unit_fraction(horizontal_fraction) > 0.8 {
        "left"
    } else {
        "right"
    }
}

/// Clicking a face follows it, or stops following when it is already followed.
pub fn followed_after_click(current: Option<&str>, clicked: &str) -> Option<String> {
    if current == Some(clicked) {
        None
    } else {
        Some(clicked.to_string())
    }
}

/// The accessible name of a proposal, for example "Analyst proposes inserting: 38%".
pub fn proposal_label(author: &str, proposal: &Proposal) -> String {
    let removal = proposal.removal.as_deref().filter(|r| !r.is_empty());
    let insertion = proposal.insertion.as_deref().filter(|i| !i.is_empty());
    match (removal, insertion) {
        (Some(r), Some(i)) => format!("{author} proposes replacing {r} with {i}"),
        (None, Some(i)) => format!("{author} proposes inserting: {i}"),
        (Some(r), None) => format!("{author} proposes removing: {r}"),
        (None, None) => format!("{author} proposes an edit"),
    }
}

/// The polite announcement for a new proposal, with its reason.
pub fn announce_proposal(author: &str, proposal: &Proposal) -> String {
    let label = proposal_label(author, proposal);
    match proposal.reason.as_deref().filter(|r| !r.is_empty()) {
        Some(reason) => format!("{label}. Reason: {reason}"),
        None => format!("{label}."),
    }
}

/// Proposals whose ids were not in `known`, in their order.
pub fn new_proposals<'a>(known: &[String], proposals: &'a [Proposal]) -> Vec<&'a Proposal> {
    proposals
        .iter()
        .filter(|p| !known.contains(&p.id))
        .collect()
}

/// Blocks in reading order, with each run of consecutive items in one group.
#[derive(Debug, PartialEq)]
pub enum BlockGroup<'a> {
    Single(&'a Block),
    Items(Vec<&'a Block>),
}

pub fn group_blocks(blocks: &[Block]) -> Vec<BlockGroup<'_>> {
    let mut groups: Vec<BlockGroup<'_>> = Vec::new();
    for block in blocks {
        if block.kind == BlockKind::Item {
            if let Some(BlockGroup::Items(run)) = groups.last_mut() {
                run.push(block);
                continue;
            }
            groups.push(BlockGroup::Items(vec![block]));
        } else {
            groups.push(BlockGroup::Single(block));
        }
    }
    groups
}

enum Event<'a> {
    Point(MarkSegment),
    Removed { proposal: &'a Proposal, end: usize },
    Inserted(&'a Proposal),
}

/// Splits `text` around each proposal and each point (carets and pins), in
/// reading order. A removal is struck through; its insertion follows it.
/// Points inside a removal are kept, placed after the removed text.
pub fn mark_segments(
    text: &str,
    proposals: &[&Proposal],
    points: &[(usize, MarkSegment)],
) -> Vec<MarkSegment> {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let slice = |from: usize, to: usize| -> String { chars[from..to].iter().collect() };

    let mut events: Vec<(usize, u8, Event<'_>)> = Vec::new();
    for (offset, segment) in points {
        events.push((
            clamp_offset(*offset, text),
            0,
            Event::Point(segment.clone()),
        ));
    }
    for proposal in proposals {
        let start = proposal.offset.min(len);
        let removal_len = proposal.removal.as_deref().map_or(0, |r| r.chars().count());
        let end = (start + removal_len).min(len);
        if removal_len > 0 {
            events.push((start, 2, Event::Removed { proposal, end }));
        }
        if proposal.insertion.as_deref().is_some_and(|i| !i.is_empty()) {
            events.push((end, 1, Event::Inserted(proposal)));
        }
    }
    events.sort_by_key(|(pos, rank, _)| (*pos, *rank));

    let mut out = Vec::new();
    let mut cursor = 0usize;
    for (pos, _, event) in events {
        match event {
            Event::Point(segment) => {
                if pos > cursor {
                    out.push(MarkSegment::Text(slice(cursor, pos)));
                    cursor = pos;
                }
                out.push(segment);
            }
            _ if pos < cursor => continue,
            Event::Removed { proposal, end } => {
                if pos > cursor {
                    out.push(MarkSegment::Text(slice(cursor, pos)));
                    cursor = pos;
                }
                out.push(MarkSegment::Removed {
                    proposal_id: proposal.id.clone(),
                    participant_id: proposal.participant_id.clone(),
                    text: slice(cursor, end),
                    focusable: true,
                });
                cursor = end;
            }
            Event::Inserted(proposal) => {
                if pos > cursor {
                    out.push(MarkSegment::Text(slice(cursor, pos)));
                    cursor = pos;
                }
                out.push(MarkSegment::Inserted {
                    proposal_id: proposal.id.clone(),
                    participant_id: proposal.participant_id.clone(),
                    text: proposal.insertion.clone().unwrap_or_default(),
                    focusable: proposal.removal.is_none(),
                });
            }
        }
    }
    if cursor < len {
        out.push(MarkSegment::Text(slice(cursor, len)));
    }
    out
}

/// Writes an accepted proposal into its block and drops it from the list.
/// Other proposals in the same block move with the text: those after the
/// edit shift by its length change, those inside the removed span snap to its
/// start. An unknown id returns the inputs unchanged.
pub fn apply_proposal(blocks: &[Block], proposals: &[Proposal], id: &str) -> AppliedEdit {
    let Some(target) = proposals.iter().find(|p| p.id == id) else {
        return AppliedEdit {
            blocks: blocks.to_vec(),
            proposals: proposals.to_vec(),
        };
    };
    let mut blocks = blocks.to_vec();
    let mut rest: Vec<Proposal> = proposals.iter().filter(|p| p.id != id).cloned().collect();
    if let Some(block) = blocks.iter_mut().find(|b| b.id == target.block_id) {
        let chars: Vec<char> = block.text.chars().collect();
        let start = target.offset.min(chars.len());
        let removal_len = target.removal.as_deref().map_or(0, |r| r.chars().count());
        let end = (start + removal_len).min(chars.len());
        let insertion = target.insertion.clone().unwrap_or_default();
        let inserted_len = insertion.chars().count();

        let mut text: String = chars[..start].iter().collect();
        text.push_str(&insertion);
        text.extend(chars[end..].iter());
        block.text = text;

        for other in rest.iter_mut().filter(|p| p.block_id == block.id) {
            if other.offset >= end {
                other.offset = other.offset - (end - start) + inserted_len;
            } else if other.offset > start {
                other.offset = start;
            }
        }
    }
    AppliedEdit {
        blocks,
        proposals: rest,
    }
}

struct Face {
    id: String,
    palette: String,
    presence: &'static str,
    label: String,
    glyph: String,
    picture: Option<String>,
}

struct Ctx<'a> {
    participants: &'a [Participant],
    positions: &'a BTreeMap<String, Position>,
    proposals: &'a [Proposal],
    pins: &'a [Pin],
    open: Signal<Option<String>>,
    followed: Option<String>,
    on_apply: Option<EventHandler<Proposal>>,
    on_dismiss: Option<EventHandler<Proposal>>,
    on_pin_close: Option<EventHandler<Pin>>,
    on_pin_reply: Option<EventHandler<(Pin, String)>>,
}

fn find_participant<'a>(ctx: &Ctx<'a>, id: &str) -> Option<&'a Participant> {
    ctx.participants.iter().find(|p| p.id == id)
}

fn name_for(ctx: &Ctx<'_>, id: &str) -> String {
    find_participant(ctx, id).map_or_else(|| id.to_string(), |p| p.name.clone())
}

fn mark_label(ctx: &Ctx<'_>, proposal_id: &str, participant_id: &str) -> Option<String> {
    ctx.proposals
        .iter()
        .find(|p| p.id == proposal_id)
        .map(|p| proposal_label(&name_for(ctx, participant_id), p))
}

fn focus_proposal_mark(proposal_id: &str) {
    document::eval(&format!(
        "requestAnimationFrame(() => requestAnimationFrame(() => [...document.querySelectorAll('[data-proposal]')].find((el) => el.dataset.proposal === {proposal_id:?})?.focus()));"
    ));
}

fn palette_for(ctx: &Ctx<'_>, id: &str) -> String {
    find_participant(ctx, id)
        .and_then(|p| p.palette_key.clone())
        .unwrap_or_else(|| "neutral".to_string())
}

fn segment_node(ctx: &Ctx<'_>, segment: MarkSegment) -> Element {
    let mut open = ctx.open;
    match segment {
        MarkSegment::Text(text) => rsx! { span { "{text}" } },
        MarkSegment::Removed {
            proposal_id,
            participant_id,
            text,
            focusable,
        } => {
            let palette = palette_for(ctx, &participant_id);
            let tab = focusable.then_some("0");
            let role = focusable.then_some("button");
            let aria = focusable
                .then(|| mark_label(ctx, &proposal_id, &participant_id))
                .flatten();
            let click_id = proposal_id.clone();
            let key_id = proposal_id.clone();
            rsx! {
                span {
                    class: "kit-cocopy__removed",
                    "data-palette": "{palette}",
                    "data-proposal": "{proposal_id}",
                    tabindex: tab,
                    role: role,
                    "aria-label": aria,
                    onclick: move |_| open.set(Some(click_id.clone())),
                    onkeydown: move |event: KeyboardEvent| {
                        if matches!(event.key(), Key::Enter) || event.key() == Key::Character(" ".to_string()) {
                            event.prevent_default();
                            open.set(Some(key_id.clone()));
                        }
                    },
                    "{text}"
                }
            }
        }
        MarkSegment::Inserted {
            proposal_id,
            participant_id,
            text,
            focusable,
        } => {
            let palette = palette_for(ctx, &participant_id);
            let tab = focusable.then_some("0");
            let role = focusable.then_some("button");
            let aria = focusable
                .then(|| mark_label(ctx, &proposal_id, &participant_id))
                .flatten();
            let click_id = proposal_id.clone();
            let key_id = proposal_id.clone();
            rsx! {
                span {
                    class: "kit-cocopy__inserted",
                    "data-palette": "{palette}",
                    "data-proposal": "{proposal_id}",
                    tabindex: tab,
                    role: role,
                    "aria-label": aria,
                    onclick: move |_| open.set(Some(click_id.clone())),
                    onkeydown: move |event: KeyboardEvent| {
                        if matches!(event.key(), Key::Enter) || event.key() == Key::Character(" ".to_string()) {
                            event.prevent_default();
                            open.set(Some(key_id.clone()));
                        }
                    },
                    "{text}"
                }
            }
        }
        MarkSegment::Caret { participant_id } => {
            let Some(participant) = find_participant(ctx, &participant_id) else {
                return rsx! {};
            };
            let position = ctx.positions.get(&participant_id);
            let activity = position.and_then(|p| p.activity.clone());
            let typing = position.is_some_and(|p| p.is_typing);
            let flag = pointer_flag(&participant.name, activity.as_deref(), participant.spend);
            let faded = ctx
                .followed
                .as_deref()
                .is_some_and(|f| f != participant_id.as_str());
            let palette = palette_for(ctx, &participant_id);
            rsx! {
                span {
                    class: "kit-cocopy__caret",
                    "aria-hidden": "true",
                    "data-participant": "{participant_id}",
                    "data-palette": "{palette}",
                    "data-typing": typing.to_string(),
                    "data-faded": faded.to_string(),
                    span { class: "kit-cocopy__flag", "{flag}" }
                }
            }
        }
        MarkSegment::Pin { pin_id } => {
            let Some(pin) = ctx.pins.iter().find(|p| p.id == pin_id) else {
                return rsx! {};
            };
            let author = name_for(ctx, &pin.participant_id);
            let glyph = glyph_for(&author);
            rsx! {
                PinMarker {
                    pin: pin.clone(),
                    author,
                    glyph,
                    style: String::new(),
                    on_close: ctx.on_pin_close,
                    on_reply: ctx.on_pin_reply,
                }
            }
        }
    }
}

fn proposal_window(ctx: &Ctx<'_>, proposal: &Proposal) -> Element {
    let author = name_for(ctx, &proposal.participant_id);
    let label = proposal_label(&author, proposal);
    let reason = proposal.reason.clone().unwrap_or_default();
    let apply_proposal = proposal.clone();
    let dismiss_proposal = proposal.clone();
    let on_apply = ctx.on_apply;
    let on_dismiss = ctx.on_dismiss;
    let mut open = ctx.open;
    let mark_id = proposal.id.clone();
    rsx! {
        div {
            class: "kit-cocopy__window",
            role: "dialog",
            "aria-label": "{label}",
            tabindex: "-1",
            onmounted: move |event: MountedEvent| {
                let node = event.data();
                spawn(async move {
                    let _ = node.set_focus(true).await;
                });
            },
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape {
                    event.prevent_default();
                    open.set(None);
                    focus_proposal_mark(&mark_id);
                }
            },
            if !reason.is_empty() {
                p { class: "kit-cocopy__reason", "{reason}" }
            }
            div { class: "kit-cocopy__actions",
                button {
                    r#type: "button",
                    class: "kit-cocopy__apply",
                    onclick: move |_| {
                        if let Some(handler) = on_apply {
                            handler.call(apply_proposal.clone());
                        }
                        open.set(None);
                    },
                    "Apply"
                }
                button {
                    r#type: "button",
                    class: "kit-cocopy__dismiss",
                    onclick: move |_| {
                        if let Some(handler) = on_dismiss {
                            handler.call(dismiss_proposal.clone());
                        }
                        open.set(None);
                    },
                    "Dismiss"
                }
            }
        }
    }
}

fn image_figure(block: &Block, caption: Element) -> Element {
    let Some(image) = block.image.as_ref() else {
        return rsx! { figure { class: "kit-cocopy__image", figcaption { {caption} } } };
    };
    let fraction = unit_fraction(image.column_fraction) * 100.0;
    let ratio = if image.aspect.is_finite() && image.aspect > 0.0 {
        image.aspect
    } else {
        1.0
    };
    let style = format!("width: {fraction:.1}%; aspect-ratio: {ratio}");
    let wrap = image.wrap_side.as_str();
    let loading = image.is_loading.to_string();
    let sweep = image.is_loading || image.source_url.is_none();
    let shown_src = if image.is_loading {
        None
    } else {
        image.source_url.clone()
    };
    let alt = image.description.clone().unwrap_or_default();
    rsx! {
        figure {
            class: "kit-cocopy__image",
            "data-wrap": wrap,
            "aria-busy": loading,
            style: "{style}",
            if sweep {
                span { class: "kit-cocopy__sweep", "aria-hidden": "true" }
            }
            if let Some(src) = shown_src {
                img { src: "{src}", alt: "{alt}" }
            }
            figcaption { {caption} }
        }
    }
}

fn table_body(block: &Block) -> Element {
    let mut rows = block.cell_grid.iter();
    let header: Vec<TableCell> = rows.next().cloned().unwrap_or_default();
    let body: Vec<Vec<TableCell>> = rows.cloned().collect();
    rsx! {
        table { class: "kit-cocopy__table",
            thead {
                tr {
                    for cell in header {
                        th { {cell.text.clone()} }
                    }
                }
            }
            tbody {
                for row in body {
                    tr {
                        for cell in row {
                            td {
                                if let Some(key) = cell.badge_key {
                                    span { class: "kit-cocopy__badge", "data-badge": "{key}", {cell.text.clone()} }
                                } else {
                                    {cell.text.clone()}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn block_element(ctx: &Ctx<'_>, block: &Block, is_item: bool) -> Element {
    let proposals_here: Vec<&Proposal> = ctx
        .proposals
        .iter()
        .filter(|p| p.block_id == block.id)
        .collect();

    let mut points: Vec<(usize, MarkSegment)> = Vec::new();
    for (participant_id, position) in ctx.positions {
        if position.block_id == block.id && find_participant(ctx, participant_id).is_some() {
            let offset = position.offset.unwrap_or(0);
            points.push((
                offset,
                MarkSegment::Caret {
                    participant_id: participant_id.clone(),
                },
            ));
        }
    }
    for pin in ctx.pins {
        if let Some(offset) = pin.offset.filter(|_| pin.block_id == block.id) {
            points.push((
                offset,
                MarkSegment::Pin {
                    pin_id: pin.id.clone(),
                },
            ));
        }
    }

    let segments = mark_segments(&block.text, &proposals_here, &points);
    let marks: Vec<Element> = segments.into_iter().map(|s| segment_node(ctx, s)).collect();
    let caption = rsx! {
        span { class: "kit-cocopy__text",
            for node in marks {
                {node}
            }
        }
    };

    let mut overlays: Vec<Element> = Vec::new();
    for pin in ctx.pins {
        if pin.block_id == block.id && pin.offset.is_none() {
            let left = unit_fraction(pin.horizontal_fraction.unwrap_or(0.0)) * 100.0;
            let top = pin.vertical_offset.unwrap_or(0.0).max(0.0);
            let author = name_for(ctx, &pin.participant_id);
            let glyph = glyph_for(&author);
            let style = format!("left: {left:.2}%; top: {top:.0}px");
            overlays.push(rsx! {
                PinMarker {
                    pin: pin.clone(),
                    author,
                    glyph,
                    style,
                    on_close: ctx.on_pin_close,
                    on_reply: ctx.on_pin_reply,
                }
            });
        }
    }
    let open_now = (ctx.open)();
    let window = open_now
        .as_deref()
        .and_then(|id| proposals_here.iter().find(|p| p.id == id).copied());
    if let Some(proposal) = window {
        overlays.push(proposal_window(ctx, proposal));
    }

    let body = match block.kind {
        BlockKind::Title => rsx! { h1 { class: "kit-cocopy__title", {caption} } },
        BlockKind::Heading => rsx! { h2 { class: "kit-cocopy__heading", {caption} } },
        BlockKind::Paragraph => rsx! { p { class: "kit-cocopy__para", {caption} } },
        BlockKind::Item => rsx! { {caption} },
        BlockKind::Code => rsx! { pre { class: "kit-cocopy__code", code { {caption} } } },
        BlockKind::Image => image_figure(block, caption),
        BlockKind::Table => table_body(block),
    };
    let id = block.id.clone();
    let kind = block.kind.as_str();
    if is_item {
        rsx! {
            li { class: "kit-cocopy__block", "data-block-id": "{id}", "data-kind": kind,
                {body}
                for node in overlays {
                    {node}
                }
            }
        }
    } else {
        rsx! {
            div { class: "kit-cocopy__block", "data-block-id": "{id}", "data-kind": kind,
                {body}
                for node in overlays {
                    {node}
                }
            }
        }
    }
}

/// A margin note: a marker bearing the author's glyph, opening into a window
/// on hover, focus, or when `pin.is_open` is set.
#[component]
fn PinMarker(
    pin: Pin,
    author: String,
    glyph: String,
    style: String,
    on_close: Option<EventHandler<Pin>>,
    on_reply: Option<EventHandler<(Pin, String)>>,
) -> Element {
    let mut hovered = use_signal(|| false);
    let mut draft = use_signal(String::new);
    let open = pin.is_open || hovered();
    let side = pin_window_side(pin.horizontal_fraction.unwrap_or(0.0));
    let id = use_hook(|| format!("kit-cocopy-pin-{}", next_id()));
    let mut window_top = use_signal(|| None::<f64>);
    let window_id = format!("{id}-window");
    let measured_id = id.clone();
    let measured_window = window_id.clone();
    use_effect(use_reactive!(|(open,)| {
        if open {
            let pin_id = measured_id.clone();
            let note_id = measured_window.clone();
            spawn(async move {
                let script = format!(
                    "await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))); \
                     const pin = document.getElementById({pin_id:?}); \
                     const note = document.getElementById({note_id:?}); \
                     if (!pin || !note) return null; \
                     const page = (pin.closest('.kit-cocopy__page') || document.documentElement).getBoundingClientRect(); \
                     return [page.height, pin.getBoundingClientRect().top - page.top, note.offsetHeight];"
                );
                if let Ok(value) = document::eval(&script).await {
                    if let Some(values) = value.as_array() {
                        let measured: Vec<f64> = values.iter().filter_map(|v| v.as_f64()).collect();
                        if measured.len() == 3 {
                            window_top.set(Some(pin_window_top(
                                measured[0],
                                measured[1],
                                measured[2],
                            )));
                        }
                    }
                }
            });
        }
    }));
    let window_style = window_top()
        .map(|top| format!("top: {top}px; bottom: auto;"))
        .unwrap_or_default();
    let body = pin.body.clone();
    let close_pin = pin.clone();
    let reply_pin = pin.clone();
    rsx! {
        span {
            id: "{id}",
            class: "kit-cocopy__pin",
            style: "{style}",
            "data-side": side,
            "data-open": open.then_some("true"),
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            button {
                r#type: "button",
                class: "kit-cocopy__pin-marker",
                "aria-expanded": open.to_string(),
                "aria-label": "Note from {author}",
                onfocus: move |_| hovered.set(true),
                onblur: move |_| hovered.set(false),
                "{glyph}"
            }
            if open {
                span { id: "{window_id}", class: "kit-cocopy__pin-window", role: "dialog", "aria-label": "Note from {author}",
                    style: "{window_style}",
                    span { class: "kit-cocopy__pin-body", "{body}" }
                    if let Some(close) = on_close {
                        button {
                            r#type: "button",
                            class: "kit-cocopy__pin-close",
                            onclick: move |_| close.call(close_pin.clone()),
                            "Close"
                        }
                    }
                    if let Some(reply) = on_reply {
                        input {
                            r#type: "text",
                            class: "kit-cocopy__pin-reply",
                            "aria-label": "Reply to {author}",
                            placeholder: "Reply",
                            value: "{draft}",
                            oninput: move |event: FormEvent| draft.set(event.value()),
                            onkeydown: move |event: KeyboardEvent| {
                                if event.key() == Key::Enter {
                                    let text = draft().trim().to_string();
                                    if !text.is_empty() {
                                        reply.call((reply_pin.clone(), text));
                                        draft.set(String::new());
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

/// Agents working in a shared document: pointers, suggested edits and margin
/// notes, with each suggestion waiting for the person's decision.
#[component]
pub fn CoEditingPresence(
    participants: Vec<Participant>,
    content: Vec<Block>,
    #[props(default)] positions: BTreeMap<String, Position>,
    #[props(default)] pins: Vec<Pin>,
    #[props(default)] proposals: Vec<Proposal>,
    #[props(default = "Document".to_string(), into)] heading: String,
    #[props(default)] views: Vec<View>,
    #[props(default)] active_view: Option<String>,
    #[props(default)] on_view_change: Option<EventHandler<String>>,
    #[props(default)] followed_id: Option<String>,
    #[props(default)] on_follow_change: Option<EventHandler<Option<String>>>,
    #[props(default)] page_height: Option<u32>,
    #[props(default)] on_apply: Option<EventHandler<Proposal>>,
    #[props(default)] on_dismiss: Option<EventHandler<Proposal>>,
    #[props(default)] on_apply_all: Option<EventHandler<()>>,
    #[props(default)] on_pin_close: Option<EventHandler<Pin>>,
    #[props(default)] on_pin_reply: Option<EventHandler<(Pin, String)>>,
    #[props(default)] extra_class: String,
) -> Element {
    let mut local_view = use_signal(|| None::<String>);
    let mut followed = use_signal(|| followed_id.clone());
    let open = use_signal(|| None::<String>);
    let known = use_hook(|| {
        proposals
            .iter()
            .map(|p| p.id.clone())
            .collect::<Vec<String>>()
    });

    let ctx = Ctx {
        participants: &participants,
        positions: &positions,
        proposals: &proposals,
        pins: &pins,
        open,
        followed: followed(),
        on_apply,
        on_dismiss,
        on_pin_close,
        on_pin_reply,
    };

    let faces: Vec<Face> = participants
        .iter()
        .map(|p| Face {
            id: p.id.clone(),
            palette: palette_for(&ctx, &p.id),
            presence: presence(positions.get(&p.id)).as_str(),
            label: face_label(&p.name, positions.get(&p.id)),
            glyph: glyph_for(&p.name),
            picture: p.picture.clone(),
        })
        .collect();
    let live = new_proposals(&known, &proposals)
        .last()
        .map(|p| announce_proposal(&name_for(&ctx, &p.participant_id), p))
        .unwrap_or_default();
    let active = local_view()
        .or(active_view)
        .or_else(|| views.first().map(|v| v.id.clone()));
    let following = ctx.followed.is_some();
    let pending = proposals.len();
    let class = class_list("kit-cocopy", &extra_class);
    let page_style = page_height.map(|h| format!("max-height: {h}px"));
    let faded = page_height.is_some().then_some("true");

    let block_nodes: Vec<Element> = group_blocks(&content)
        .into_iter()
        .map(|group| match group {
            BlockGroup::Single(block) => block_element(&ctx, block, false),
            BlockGroup::Items(items) => {
                let lis: Vec<Element> = items
                    .into_iter()
                    .map(|block| block_element(&ctx, block, true))
                    .collect();
                rsx! {
                    ul { class: "kit-cocopy__list",
                        for li in lis {
                            {li}
                        }
                    }
                }
            }
        })
        .collect();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            class,
            "aria-label": "{heading}",
            "data-following": following.to_string(),
            header { class: "kit-cocopy__head",
                h2 { class: "kit-cocopy__name", "{heading}" }
                if !views.is_empty() {
                    div { class: "kit-cocopy__views", role: "group", "aria-label": "Views",
                        for view in views.iter() {
                            button {
                                r#type: "button",
                                class: "kit-cocopy__view",
                                "aria-pressed": (active.as_deref() == Some(view.id.as_str())).to_string(),
                                onclick: {
                                    let id = view.id.clone();
                                    move |_| {
                                        local_view.set(Some(id.clone()));
                                        if let Some(handler) = on_view_change {
                                            handler.call(id.clone());
                                        }
                                    }
                                },
                                {view.caption.clone()}
                            }
                        }
                    }
                }
                div { class: "kit-cocopy__faces", role: "group", "aria-label": "Participants",
                    for face in faces {
                        button {
                            r#type: "button",
                            class: "kit-cocopy__face",
                            "data-palette": face.palette,
                            "data-presence": face.presence,
                            "aria-pressed": (followed().as_deref() == Some(face.id.as_str())).to_string(),
                            "aria-label": face.label,
                            onclick: {
                                let id = face.id.clone();
                                move |_| {
                                    let next = followed_after_click(followed().as_deref(), &id);
                                    followed.set(next.clone());
                                    if let Some(handler) = on_follow_change {
                                        handler.call(next);
                                    }
                                }
                            },
                            if let Some(url) = face.picture {
                                img { class: "kit-cocopy__picture", src: url, alt: "" }
                            } else {
                                span { class: "kit-cocopy__glyph", "aria-hidden": "true", {face.glyph} }
                            }
                        }
                    }
                }
                div { class: "kit-cocopy__pending",
                    span { class: "kit-cocopy__count", "{pending} pending" }
                    if pending > 1 {
                        button {
                            r#type: "button",
                            class: "kit-cocopy__apply-all",
                            onclick: move |_| {
                                if let Some(handler) = on_apply_all {
                                    handler.call(());
                                }
                            },
                            "Apply all"
                        }
                    }
                }
            }
            div { class: "kit-cocopy__sr", role: "status", "aria-live": "polite", "{live}" }
            div { class: "kit-cocopy__page", style: page_style, "data-faded": faded,
                for node in block_nodes {
                    {node}
                }
            }
        }
    }
}
