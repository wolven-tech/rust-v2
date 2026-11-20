//! A finished answer whose sentences are tied to the source passages they came from, checked by hovering or opening them.
//!
//! Design choice: `trailing_action_text` and `on_trailing_action` stand in for the spec's
//! `trailing_action: { action_text, on_activate }`, because an `EventHandler` is not a data field.
//! Design choice: a passage is marked against the words of the segments that cite it. Words are compared
//! lowercased, without punctuation, and only when longer than three characters. An `exact_phrase`
//! replaces that comparison.
//! Design choice: chips beyond four collapse into a "+N" chip, a fixed cut rather than a measured one.
//! Browser checks: hovering a chip tints its citing segments; the preview cross-fades when switching;
//! the Open reference action navigates only when no handler is given.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, copy_to_clipboard, next_id, sleep};

const CSS: Asset = asset!("/assets/source-linked-answer.css");
/// Chips shown before the "+N" chip.
pub const VISIBLE_CHIPS: usize = 4;

#[derive(Clone, PartialEq, Debug)]
pub struct Segment {
    pub text: String,
    pub cited_ids: Vec<String>,
    pub starts_paragraph: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Reference {
    pub ref_id: String,
    pub headline: String,
    pub origin_site: String,
    pub link_target: Option<String>,
    pub published_on: Option<String>,
    pub passage: Option<String>,
    pub exact_phrase: Option<String>,
    pub leading_mark: Option<String>,
}

fn normalise_word(word: &str) -> String {
    word.chars()
        .filter(|ch| ch.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Splits a passage into runs, flagging the runs that match the citing text.
/// An `exact_phrase` found in the passage is the only match when given.
pub fn mark_passage(passage: &str, citing: &str, exact: Option<&str>) -> Vec<(String, bool)> {
    if let Some(phrase) = exact.filter(|p| !p.is_empty()) {
        return match passage.find(phrase) {
            Some(at) => vec![
                (passage[..at].to_string(), false),
                (passage[at..at + phrase.len()].to_string(), true),
                (passage[at + phrase.len()..].to_string(), false),
            ],
            None => vec![(passage.to_string(), false)],
        };
    }
    let citing_words: Vec<String> = citing
        .split_whitespace()
        .map(normalise_word)
        .filter(|word| word.chars().count() > 3)
        .collect();
    let mut runs: Vec<(String, bool)> = Vec::new();
    for (index, word) in passage.split(' ').enumerate() {
        let lead = if index > 0 { " " } else { "" };
        let norm = normalise_word(word);
        let matched = norm.chars().count() > 3 && citing_words.contains(&norm);
        match runs.last_mut() {
            Some((text, flag)) if *flag == matched => {
                text.push_str(lead);
                text.push_str(word);
            }
            _ => runs.push((format!("{lead}{word}"), matched)),
        }
    }
    runs
}

/// The text of every segment that cites `ref_id`, joined: the sentence a passage must match.
pub fn citing_text(segments: &[Segment], ref_id: &str) -> String {
    segments
        .iter()
        .filter(|segment| segment.cited_ids.iter().any(|id| id == ref_id))
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn reference_label(index: usize, reference: &Reference) -> String {
    format!(
        "Reference {}: {} – {}",
        index + 1,
        reference.origin_site,
        reference.headline
    )
}

pub fn valid_ids(ids: &[String], references: &[Reference]) -> Vec<String> {
    ids.iter()
        .filter(|id| references.iter().any(|reference| reference.ref_id == **id))
        .cloned()
        .collect()
}

fn focus_element(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

/// Finished answer with citation markers, reference chips and an inline preview.
#[component]
pub fn SourceLinkedAnswer(
    answer_segments: Vec<Segment>,
    reference_list: Vec<Reference>,
    #[props(default)] pages_consulted: usize,
    #[props(default)] start_open_ref: Option<String>,
    #[props(default)] on_open_request: Option<EventHandler<Reference>>,
    #[props(default)] on_preview_change: Option<EventHandler<Option<String>>>,
    #[props(default)] trailing_action_text: Option<String>,
    #[props(default)] on_trailing_action: Option<EventHandler<()>>,
    #[props(default, into)] root_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-sla-{}", next_id()));
    let mut open_ref = use_signal(|| start_open_ref.clone());
    let hover_ref = use_signal(|| Option::<String>::None);
    let status = use_signal(String::new);
    let opener = use_signal(String::new);

    let current_open = open_ref();
    let hovered = hover_ref();
    let root_class = if root_class.is_empty() {
        "kit-sla".to_string()
    } else {
        format!("kit-sla {root_class}")
    };
    let header_text = format!(
        "{} sources · {pages_consulted} pages read",
        reference_list.len()
    );
    let overflow = reference_list.len().saturating_sub(VISIBLE_CHIPS);
    let overflow_text = format!("+{overflow}");

    let set_open = {
        let handler = on_preview_change.clone();
        move |next: Option<String>| {
            open_ref.set(next.clone());
            if let Some(h) = &handler {
                h.call(next);
            }
        }
    };

    let segment_nodes: Vec<Element> = answer_segments
        .iter()
        .enumerate()
        .map(|(index, segment)| {
            let cited = valid_ids(&segment.cited_ids, &reference_list);
            let tinted = hovered.as_ref().is_some_and(|hover| cited.contains(hover));
            let dimmed = hovered.is_some() && !tinted;
            let tint = if tinted { "true" } else { "false" };
            let dim = if dimmed { "true" } else { "false" };
            let text = segment.text.clone();
            let class = if segment.starts_paragraph {
                "kit-sla__segment is-paragraph"
            } else {
                "kit-sla__segment"
            };
            let key = format!("{id}-s{index}");
            let markers: Vec<Element> = cited
                .iter()
                .filter_map(|ref_id| {
                    let position = reference_list
                        .iter()
                        .position(|reference| reference.ref_id == *ref_id)?;
                    Some(render_marker(
                        &id,
                        &reference_list[position],
                        position,
                        index,
                        current_open.as_deref() == Some(ref_id.as_str()),
                        set_open.clone(),
                        opener,
                        hover_ref,
                    ))
                })
                .collect();
            rsx! {
                span { key: "{key}", class: "{class}", "data-tint": tint, "data-dim": dim,
                    "{text} "
                    for marker in markers {
                        {marker}
                    }
                }
            }
        })
        .collect();

    let chip_nodes: Vec<Element> = reference_list
        .iter()
        .enumerate()
        .take(VISIBLE_CHIPS)
        .map(|(position, reference)| {
            render_chip(
                &id,
                reference,
                position,
                current_open.as_deref() == Some(reference.ref_id.as_str()),
                set_open.clone(),
                opener,
                hover_ref,
            )
        })
        .collect();

    let preview_node = current_open.as_ref().and_then(|ref_id| {
        let position = reference_list
            .iter()
            .position(|reference| reference.ref_id == *ref_id)?;
        let reference = reference_list[position].clone();
        let citing = citing_text(&answer_segments, ref_id);
        Some(render_preview(
            &id,
            reference,
            position,
            &citing,
            on_open_request.clone(),
            status,
        ))
    });

    let mut close = set_open.clone();
    let trailing = on_trailing_action.clone();
    let trailing_label = trailing_action_text.clone();
    let status_text = status();
    let has_open = current_open.is_some();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        article {
            id: "{id}",
            class: "{root_class}",
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape && has_open {
                    event.prevent_default();
                    let back = opener();
                    close(None);
                    focus_element(&back);
                }
            },
            header { class: "kit-sla__head",
                span { class: "kit-sla__summary", "{header_text}" }
            }
            p { class: "kit-sla__body", role: "group", aria_label: "Answer",
                for node in segment_nodes {
                    {node}
                }
            }
            div { class: "kit-sla__chips", role: "group", aria_label: "References",
                for node in chip_nodes {
                    {node}
                }
                if overflow > 0 {
                    span { class: "kit-sla__more", "{overflow_text}" }
                }
            }
            if let Some(node) = preview_node {
                {node}
            }
            footer { class: "kit-sla__foot",
                span { class: "kit-sla__status", role: "status", "aria-live": "polite", "{status_text}" }
                if let Some(label) = trailing_label {
                    button {
                        r#type: "button",
                        class: "kit-sla__trailing",
                        onclick: move |_| {
                            if let Some(h) = &trailing {
                                h.call(());
                            }
                        },
                        "{label}"
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_marker(
    id: &str,
    reference: &Reference,
    position: usize,
    segment_index: usize,
    open_now: bool,
    mut set_open: impl FnMut(Option<String>) + Clone + 'static,
    mut opener: Signal<String>,
    mut hover_ref: Signal<Option<String>>,
) -> Element {
    let label = reference_label(position, reference);
    let number = (position + 1).to_string();
    let marker_id = format!("{id}-m{segment_index}-{position}");
    let ref_key = reference.ref_id.clone();
    let expanded = if open_now { "true" } else { "false" };
    let hover_on = ref_key.clone();
    let focus_on = ref_key.clone();
    let open_for_click = ref_key.clone();
    let opener_for_click = marker_id.clone();
    rsx! {
        button {
            r#type: "button",
            id: "{marker_id}",
            class: "kit-sla__marker",
            aria_label: "{label}",
            "aria-expanded": expanded,
            "aria-controls": "{id}-preview",
            onmouseenter: move |_| hover_ref.set(Some(hover_on.clone())),
            onmouseleave: move |_| hover_ref.set(None),
            onfocus: move |_| hover_ref.set(Some(focus_on.clone())),
            onblur: move |_| hover_ref.set(None),
            onclick: move |_| {
                if open_now {
                    set_open(None);
                } else {
                    opener.set(opener_for_click.clone());
                    set_open(Some(open_for_click.clone()));
                }
            },
            "{number}"
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_chip(
    id: &str,
    reference: &Reference,
    position: usize,
    open_now: bool,
    mut set_open: impl FnMut(Option<String>) + Clone + 'static,
    mut opener: Signal<String>,
    mut hover_ref: Signal<Option<String>>,
) -> Element {
    let label = reference_label(position, reference);
    let number = (position + 1).to_string();
    let site = reference.origin_site.clone();
    let headline = reference.headline.clone();
    let chip_id = format!("{id}-chip-{position}");
    let ref_key = reference.ref_id.clone();
    let expanded = if open_now { "true" } else { "false" };
    let hover_on = ref_key.clone();
    let focus_on = ref_key.clone();
    let open_for_click = ref_key.clone();
    let opener_for_click = chip_id.clone();
    rsx! {
        button {
            r#type: "button",
            id: "{chip_id}",
            class: "kit-sla__chip",
            aria_label: "{label}",
            "aria-expanded": expanded,
            "aria-controls": "{id}-preview",
            "data-open": expanded,
            onmouseenter: move |_| hover_ref.set(Some(hover_on.clone())),
            onmouseleave: move |_| hover_ref.set(None),
            onfocus: move |_| hover_ref.set(Some(focus_on.clone())),
            onblur: move |_| hover_ref.set(None),
            onclick: move |_| {
                if open_now {
                    set_open(None);
                } else {
                    opener.set(opener_for_click.clone());
                    set_open(Some(open_for_click.clone()));
                }
            },
            span { class: "kit-sla__chip-num", "{number}" }
            span { class: "kit-sla__chip-site", "{site}" }
            span { class: "kit-sla__chip-head", "{headline}" }
        }
    }
}

fn render_preview(
    id: &str,
    reference: Reference,
    position: usize,
    citing: &str,
    on_open_request: Option<EventHandler<Reference>>,
    mut status: Signal<String>,
) -> Element {
    let title = reference_label(position, &reference);
    let meta = match &reference.published_on {
        Some(date) => format!("{} · {date}", reference.origin_site),
        None => reference.origin_site.clone(),
    };
    let headline = reference.headline.clone();
    let passage = reference.passage.clone().unwrap_or_default();
    let has_passage = reference.passage.is_some();
    let runs = mark_passage(&passage, citing, reference.exact_phrase.as_deref());
    let link = reference.link_target.clone();
    let has_link = link.is_some();
    let link_href = link.unwrap_or_default();
    let open_reference = reference.clone();
    let copy_text = passage.clone();
    let open_label = "Open reference";
    let preview_id = format!("{id}-preview");
    rsx! {
        section { id: "{preview_id}", class: "kit-sla__preview", role: "region", aria_label: "{title}",
            header { class: "kit-sla__preview-head",
                span { class: "kit-sla__preview-meta", "{meta}" }
                h4 { class: "kit-sla__preview-title", "{headline}" }
            }
            if has_passage {
                blockquote { class: "kit-sla__passage",
                    for (run, flagged) in runs {
                        if flagged {
                            mark { "{run}" }
                        } else {
                            span { "{run}" }
                        }
                    }
                }
            }
            div { class: "kit-sla__preview-actions",
                if let Some(handler) = on_open_request {
                    button {
                        r#type: "button",
                        class: "kit-sla__open",
                        onclick: move |_| handler.call(open_reference.clone()),
                        "{open_label}"
                    }
                } else if has_link {
                    a { class: "kit-sla__open", href: "{link_href}", target: "_blank", rel: "noopener noreferrer", "{open_label}" }
                }
                if has_passage {
                    button {
                        r#type: "button",
                        class: "kit-sla__copy",
                        onclick: move |_| {
                            copy_to_clipboard(&copy_text);
                            status.set("Passage copied".to_string());
                            spawn(async move {
                                sleep(2000).await;
                                status.set(String::new());
                            });
                        },
                        "Copy passage"
                    }
                }
            }
        }
    }
}
