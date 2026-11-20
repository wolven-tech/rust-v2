//! An assistant answer shown as it arrives, with its sources up front and `[n]` markers that point back to them.
//!
//! Design choice: the answer grammar is a blank line for a paragraph, `- ` for a list item, `**text**` for
//! emphasis, and `[n]` or `[n,m]` for citations. A citation to a missing source is dropped, not rendered.
//! Design choice: the parser has no sentence model, so hovering a source highlights its whole paragraph.
//! Design choice: the reveal shows whole words at `reveal_speed` per second. A change that is not an
//! extension of the text restarts it.
//! Design choice: `on_copy_requested` receives the plain answer and its numbered sources; the component
//! also writes the same text to the clipboard.
//! Browser checks: word reveal and the reading pulse; the body keeps its height as text grows; Escape
//! closes the preview and returns focus to its opener; copy writes to the clipboard.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, copy_to_clipboard, next_id, sleep};

const CSS: Asset = asset!("/assets/grounded-reply.css");
/// Source cards shown before the "+N" tile.
pub const VISIBLE_CARDS: usize = 3;

#[derive(Clone, PartialEq, Debug)]
pub struct SourceEntry {
    pub entry_id: String,
    pub chip_text: String,
    pub link_target: Option<String>,
    pub leading_mark: Option<String>,
    pub snippet: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vote {
    Up,
    Down,
}

#[derive(Clone, PartialEq, Debug)]
pub struct VoteEvent {
    pub vote: Vote,
    pub reason: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Inline {
    Text(String),
    Strong(String),
    /// 1-based source numbers as written in the answer.
    Cite(Vec<usize>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Block {
    Paragraph(Vec<Inline>),
    ListItem(Vec<Inline>),
}

/// Splits an answer into blocks. Blank lines separate paragraphs; a `- ` line is a list item.
pub fn parse_answer(text: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut paragraph: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush_paragraph(&mut paragraph, &mut blocks);
        } else if let Some(item) = trimmed.strip_prefix("- ") {
            flush_paragraph(&mut paragraph, &mut blocks);
            blocks.push(Block::ListItem(parse_inline(item)));
        } else {
            paragraph.push(trimmed.to_string());
        }
    }
    flush_paragraph(&mut paragraph, &mut blocks);
    blocks
}

fn flush_paragraph(paragraph: &mut Vec<String>, blocks: &mut Vec<Block>) {
    if !paragraph.is_empty() {
        blocks.push(Block::Paragraph(parse_inline(&paragraph.join(" "))));
        paragraph.clear();
    }
}

/// Inline runs: `**strong**` and `[n]` / `[n,m]` citations. Any other bracket is literal text.
pub fn parse_inline(text: &str) -> Vec<Inline> {
    let mut out = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        let next = match (rest.find("**"), rest.find('[')) {
            (Some(s), Some(c)) => Some(s.min(c)),
            (Some(s), None) => Some(s),
            (None, Some(c)) => Some(c),
            (None, None) => None,
        };
        let Some(at) = next else {
            out.push(Inline::Text(rest.to_string()));
            break;
        };
        if at > 0 {
            out.push(Inline::Text(rest[..at].to_string()));
        }
        let tail = &rest[at..];
        if let Some(body) = tail.strip_prefix("**") {
            if let Some(end) = body.find("**") {
                out.push(Inline::Strong(body[..end].to_string()));
                rest = &body[end + 2..];
                continue;
            }
        } else if let Some(end) = tail.find(']') {
            let numbers: Option<Vec<usize>> = tail[1..end]
                .split(',')
                .map(|part| part.trim().parse::<usize>().ok())
                .collect();
            if let Some(numbers) = numbers.filter(|list| !list.is_empty()) {
                out.push(Inline::Cite(numbers));
                rest = &tail[end + 1..];
                continue;
            }
        }
        let width = tail.chars().next().map_or(1, char::len_utf8);
        out.push(Inline::Text(tail[..width].to_string()));
        rest = &tail[width..];
    }
    out
}

/// The first `count` whitespace-separated words of `text`, with the spaces between them kept.
pub fn take_words(text: &str, count: usize) -> &str {
    let mut seen = 0;
    let mut in_word = false;
    for (index, ch) in text.char_indices() {
        if ch.is_whitespace() {
            if in_word {
                seen += 1;
                if seen == count {
                    return &text[..index];
                }
            }
            in_word = false;
        } else {
            in_word = true;
        }
    }
    text
}

pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Plain answer text with a numbered source list after it.
pub fn plain_copy(answer: &str, sources: &[SourceEntry]) -> String {
    let mut out = String::new();
    for block in parse_answer(answer) {
        let (prefix, inlines) = match &block {
            Block::Paragraph(inlines) => ("", inlines),
            Block::ListItem(inlines) => ("- ", inlines),
        };
        out.push_str(prefix);
        for inline in inlines {
            match inline {
                Inline::Text(text) | Inline::Strong(text) => out.push_str(text),
                Inline::Cite(_) => {}
            }
        }
        out.push_str("\n\n");
    }
    out.push_str("Sources\n");
    for (index, source) in sources.iter().enumerate() {
        out.push_str(&format!("{}. {}\n", index + 1, source.chip_text));
    }
    out.trim_end().to_string()
}

/// Citation numbers that point at a real source; anything else is not rendered.
pub fn valid_cites(numbers: &[usize], source_count: usize) -> Vec<usize> {
    numbers
        .iter()
        .copied()
        .filter(|n| *n >= 1 && *n <= source_count)
        .collect()
}

fn focus_element(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

/// Answer with sources, streamed reveal, follow-ups and an action bar.
#[component]
pub fn GroundedReply(
    #[props(default, into)] heading_text: String,
    source_entries: Vec<SourceEntry>,
    #[props(default, into)] answer_text: String,
    #[props(default)] is_still_writing: bool,
    #[props(default = 24.0)] reveal_speed: f64,
    #[props(default)] next_questions: Vec<String>,
    #[props(default)] on_next_question_chosen: Option<EventHandler<String>>,
    #[props(default)] on_retry_requested: Option<EventHandler<()>>,
    #[props(default)] on_halt_requested: Option<EventHandler<()>>,
    #[props(default)] on_vote: Option<EventHandler<VoteEvent>>,
    #[props(default = vec!["Inaccurate".to_string(), "Wrong citation".to_string(), "Too long".to_string(), "Outdated".to_string()])]
    vote_down_reasons: Vec<String>,
    #[props(default)] on_copy_requested: Option<EventHandler<String>>,
    #[props(default = true)] reveal_animated: bool,
    #[props(default = true)] raised_surface: bool,
    #[props(default, into)] root_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-gr-{}", next_id()));
    let mut shown_words = use_signal(|| 0_usize);
    let mut cleared = use_signal(|| false);
    let mut open_source = use_signal(|| Option::<usize>::None);
    let mut opener = use_signal(String::new);
    let mut hover_source = use_signal(|| Option::<usize>::None);
    let mut vote = use_signal(|| Option::<Vote>::None);
    let mut menu_open = use_signal(|| false);
    let mut copied = use_signal(|| false);

    let total_words = word_count(&answer_text);
    {
        let text_key = answer_text.clone();
        use_effect(use_reactive((&text_key,), move |(text,)| {
            cleared.set(false);
            if *shown_words.peek() > word_count(&text) {
                shown_words.set(0);
            }
        }));
    }
    {
        let speed = reveal_speed.max(1.0);
        use_effect(use_reactive(
            (&total_words, &reveal_animated, &speed),
            move |(total, animated, speed)| {
                // Peek: the reveal task writes shown_words; tracking it here would
                // spawn another reveal loop on every word.
                if !animated || *shown_words.peek() >= total {
                    return;
                }
                spawn(async move {
                    while shown_words() < total {
                        sleep((1000.0 / speed) as u32).await;
                        shown_words.with_mut(|count| *count = (*count + 1).min(total));
                    }
                });
            },
        ));
    }

    let body_text = if cleared() {
        String::new()
    } else if reveal_animated {
        take_words(&answer_text, shown_words()).to_string()
    } else {
        answer_text.clone()
    };
    let blocks = parse_answer(&body_text);
    let reading = body_text.is_empty();
    let source_count = source_entries.len();
    let reading_label = format!("Reading {source_count} sources");
    let overflow = source_count.saturating_sub(VISIBLE_CARDS);
    let overflow_label = format!("+{overflow}");
    let heading_count = format!("{source_count} sources");
    let streaming = is_still_writing || (reveal_animated && shown_words() < total_words);
    let finished = !streaming && !reading;
    let root_class = if root_class.is_empty() {
        "kit-gr".to_string()
    } else {
        format!("kit-gr {root_class}")
    };
    let surface_attr = if raised_surface { "raised" } else { "inherit" };
    let current_open = open_source();
    let cards: Vec<(String, String, String, String, bool, bool)> = source_entries
        .iter()
        .enumerate()
        .take(VISIBLE_CARDS)
        .map(|(index, entry)| {
            (
                entry.entry_id.clone(),
                format!("{}", index + 1),
                entry.chip_text.clone(),
                format!("Source {}: {}", index + 1, entry.chip_text),
                current_open == Some(index),
                hover_source() == Some(index),
            )
        })
        .collect();
    let preview = current_open.and_then(|index| {
        source_entries
            .get(index)
            .cloned()
            .map(|entry| (index, entry))
    });
    let preview_region = preview.map(|(index, entry)| {
        let title = format!("Source {}: {}", index + 1, entry.chip_text);
        let snippet = entry.snippet.clone().unwrap_or_default();
        let has_snippet = entry.snippet.is_some();
        let link_href = entry.link_target.clone().unwrap_or_default();
        let has_link = entry.link_target.is_some();
        let link_text = format!("Open {}", entry.chip_text);
        rsx! {
            section { class: "kit-gr__preview", role: "region", aria_label: "{title}",
                p { class: "kit-gr__preview-title", "{title}" }
                if has_snippet {
                    blockquote { class: "kit-gr__snippet", "{snippet}" }
                }
                if has_link {
                    a { class: "kit-gr__link", href: "{link_href}", target: "_blank", rel: "noopener noreferrer", "{link_text}" }
                }
            }
        }
    });

    let rendered_blocks: Vec<Element> = blocks
        .into_iter()
        .enumerate()
        .map(|(block_index, block)| {
            render_block(
                block,
                block_index,
                source_count,
                &source_entries,
                &id,
                open_source,
                opener,
                hover_source,
            )
        })
        .collect();

    let copy_sources = source_entries.clone();
    let copy_answer = answer_text.clone();
    let copy_handler = on_copy_requested.clone();
    let retry_handler = on_retry_requested.clone();
    let halt_handler = on_halt_requested.clone();
    let vote_handler = on_vote.clone();
    let vote_reasons = vote_down_reasons.clone();
    let follow_handler = on_next_question_chosen.clone();
    let followups: Vec<String> = if finished {
        next_questions.clone()
    } else {
        Vec::new()
    };
    let key_focus_id = id.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        article {
            id: "{id}",
            class: "{root_class}",
            "data-surface": surface_attr,
            "data-state": if reading { "reading" } else if streaming { "streaming" } else { "finished" },
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape && open_source().is_some() {
                    event.prevent_default();
                    open_source.set(None);
                    focus_element(&opener());
                }
            },
            header { class: "kit-gr__head",
                h2 { class: "kit-gr__question", "{heading_text}" }
                span { class: "kit-gr__count", "{heading_count}" }
            }
            ul { class: "kit-gr__sources", aria_label: "Sources",
                for (entry_key, number, chip, card_label, expanded, highlighted) in cards {
                    li { key: "{entry_key}",
                        button {
                            r#type: "button",
                            id: "{key_focus_id}-src-{number}",
                            class: "kit-gr__card",
                            aria_label: "{card_label}",
                            "aria-expanded": if expanded { "true" } else { "false" },
                            "data-highlight": if highlighted { "true" } else { "false" },
                            onmouseenter: {
                                let index: usize = number.parse::<usize>().unwrap_or(1) - 1;
                                move |_| hover_source.set(Some(index))
                            },
                            onmouseleave: move |_| hover_source.set(None),
                            onclick: {
                                let index: usize = number.parse::<usize>().unwrap_or(1) - 1;
                                let card_id = format!("{key_focus_id}-src-{index}");
                                move |_| {
                                    if open_source() == Some(index) {
                                        open_source.set(None);
                                    } else {
                                        opener.set(card_id.clone());
                                        open_source.set(Some(index));
                                    }
                                }
                            },
                            span { class: "kit-gr__card-chip", "{number}" }
                            span { class: "kit-gr__card-text", "{chip}" }
                        }
                    }
                }
                if overflow > 0 {
                    li { class: "kit-gr__more", span { "{overflow_label}" } }
                }
            }
            if let Some(region) = preview_region {
                {region}
            }
            div { class: "kit-gr__body", "aria-busy": streaming.then_some("true"), role: "region", aria_label: "Answer",
                p { class: "kit-gr__reading", role: "status", "aria-live": "polite",
                    if reading {
                        span { class: "kit-gr__pulse", aria_hidden: "true" }
                        "{reading_label}"
                    }
                }
                for node in rendered_blocks {
                    {node}
                }
                if streaming && !reading {
                    span { class: "kit-gr__caret", aria_hidden: "true" }
                }
            }
            if !followups.is_empty() {
                ul { class: "kit-gr__followups", aria_label: "Follow-up questions",
                    for question in followups {
                        li { key: "{question}",
                            button {
                                r#type: "button",
                                class: "kit-gr__followup",
                                onclick: {
                                    let text = question.clone();
                                    let handler = follow_handler.clone();
                                    move |_| {
                                        if let Some(h) = &handler {
                                            h.call(text.clone());
                                        }
                                    }
                                },
                                "{question}"
                            }
                        }
                    }
                }
            }
            footer { class: "kit-gr__actions",
                if streaming {
                    button {
                        r#type: "button",
                        class: "kit-gr__stop",
                        onclick: move |_| {
                            if let Some(h) = &halt_handler {
                                h.call(());
                            }
                        },
                        "Stop"
                    }
                } else {
                    button {
                        r#type: "button",
                        class: "kit-gr__copy",
                        "data-copied": copied().then_some("true"),
                        onclick: move |_| {
                            let text = plain_copy(&copy_answer, &copy_sources);
                            copy_to_clipboard(&text);
                            if let Some(h) = &copy_handler {
                                h.call(text);
                            }
                            copied.set(true);
                            spawn(async move {
                                sleep(1500).await;
                                copied.set(false);
                            });
                        },
                        if copied() { "Copied" } else { "Copy answer" }
                    }
                    button {
                        r#type: "button",
                        class: "kit-gr__retry",
                        disabled: !finished,
                        onclick: move |_| {
                            cleared.set(true);
                            shown_words.set(0);
                            if let Some(h) = &retry_handler {
                                h.call(());
                            }
                        },
                        "Regenerate"
                    }
                    button {
                        r#type: "button",
                        class: "kit-gr__up",
                        "aria-pressed": if vote() == Some(Vote::Up) { "true" } else { "false" },
                        onclick: {
                            let handler = vote_handler.clone();
                            move |_| {
                                vote.set(Some(Vote::Up));
                                menu_open.set(false);
                                if let Some(h) = &handler {
                                    h.call(VoteEvent { vote: Vote::Up, reason: None });
                                }
                            }
                        },
                        "Good answer"
                    }
                    button {
                        r#type: "button",
                        class: "kit-gr__down",
                        "aria-pressed": if vote() == Some(Vote::Down) { "true" } else { "false" },
                        "aria-expanded": if menu_open() { "true" } else { "false" },
                        onclick: move |_| menu_open.set(!menu_open()),
                        "Bad answer"
                    }
                    if menu_open() {
                        div { class: "kit-gr__reasons", role: "menu", aria_label: "Why was it wrong?",
                            for reason in vote_reasons {
                                button {
                                    key: "{reason}",
                                    r#type: "button",
                                    role: "menuitem",
                                    class: "kit-gr__reason",
                                    onclick: {
                                        let picked = reason.clone();
                                        let handler = vote_handler.clone();
                                        move |_| {
                                            vote.set(Some(Vote::Down));
                                            menu_open.set(false);
                                            if let Some(h) = &handler {
                                                h.call(VoteEvent { vote: Vote::Down, reason: Some(picked.clone()) });
                                            }
                                        }
                                    },
                                    "{reason}"
                                }
                            }
                        }
                    }
                }
            }
            p { class: "kit-gr__live", role: "status", "aria-live": "polite",
                if finished { "Answer complete" }
            }
        }
    }
}

/// One paragraph or list item. Markers are buttons; a source hover highlights the paragraph.
#[allow(clippy::too_many_arguments)]
fn render_block(
    block: Block,
    block_index: usize,
    source_count: usize,
    sources: &[SourceEntry],
    id: &str,
    open_source: Signal<Option<usize>>,
    opener: Signal<String>,
    hover_source: Signal<Option<usize>>,
) -> Element {
    let (is_item, runs) = match block {
        Block::Paragraph(runs) => (false, runs),
        Block::ListItem(runs) => (true, runs),
    };
    let cited: Vec<usize> = runs
        .iter()
        .filter_map(|run| match run {
            Inline::Cite(numbers) => Some(numbers.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    let highlighted =
        hover_source().is_some_and(|hover| cited.iter().any(|n| n.checked_sub(1) == Some(hover)));
    let class = if highlighted {
        "kit-gr__para is-highlighted"
    } else {
        "kit-gr__para"
    };
    let key = format!("{id}-b{block_index}");
    let mut nodes: Vec<Element> = Vec::new();
    for (run_index, run) in runs.into_iter().enumerate() {
        let run_key = format!("{key}-{run_index}");
        match run {
            Inline::Text(text) => nodes.push(rsx! { span { key: "{run_key}", "{text}" } }),
            Inline::Strong(text) => nodes.push(rsx! { strong { key: "{run_key}", "{text}" } }),
            Inline::Cite(numbers) => {
                for number in valid_cites(&numbers, source_count) {
                    nodes.push(render_marker(number, sources, id, open_source, opener));
                }
            }
        }
    }
    if is_item {
        rsx! { ul { key: "{key}", class: "kit-gr__list", li { class: "{class}", for node in nodes { {node} } } } }
    } else {
        rsx! { p { key: "{key}", class: "{class}", for node in nodes { {node} } } }
    }
}

fn render_marker(
    number: usize,
    sources: &[SourceEntry],
    id: &str,
    mut open_source: Signal<Option<usize>>,
    mut opener: Signal<String>,
) -> Element {
    let index = number - 1;
    let label = format!("Source {number}: {}", sources[index].chip_text);
    let marker_id = format!("{id}-cite-{number}-{index}");
    let opener_id = format!("{id}-src-{index}");
    let text = number.to_string();
    let expanded = if open_source() == Some(index) {
        "true"
    } else {
        "false"
    };
    rsx! {
        button {
            r#type: "button",
            class: "kit-gr__marker",
            id: "{marker_id}",
            aria_label: "{label}",
            "aria-expanded": expanded,
            onclick: move |_| {
                if open_source() == Some(index) {
                    open_source.set(None);
                } else {
                    opener.set(opener_id.clone());
                    open_source.set(Some(index));
                }
            },
            "{text}"
        }
    }
}
