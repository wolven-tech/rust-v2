//! Context treemap: an agent's context window as a treemap, one tile per block of context
//! and a dashed remainder for free space. A before and after view shows what a
//! compaction changed.
//!
//! Design choice: the map area has a fixed height, so a compaction note or a missing record
//! does not move the layout.
//! Design choice: the before view is `compaction_record.pre_blocks`; the after view is
//! `block_list`, which is the current state and already holds the digest.
//! Design choice: the agent menu is a native `select`. The bar is a plain toolbar, not a
//! menu component.
//! Design choice: a digest tile has no tokens of its own in the source data; the host
//! lists it in `block_list` with its `token_count`.
//! Design choice: each kind has one series tone, shared with `context_fill_grid`: conversation
//! 1, tool output 2, document 3, recall 4, digest 5. Instructions and capabilities share a
//! neutral system tone. Kinds draw from `--kit-series-*`, never the status colours.
//! Design choice: a tile is a button. Enter and Space hold its card open, a click holds
//! it too, and a second activation or Escape releases it.
//! Not implemented: the glide of tiles between sizes, the shine on the used figure, and the
//! roll of status text. Tiles re-lay out at their new places; CSS handles the shine and roll
//! when the host adds them. Block icons (`block_mark`) are not rendered.
//! Browser checks: tile glide on token change, the before and after switch animation, the
//! hover card staying on the screen edge, tiles too small for text relying on their card.

use dioxus::prelude::*;

use crate::components::series_trend_chart::{
    Rect, compact_number, focus_element, non_negative, share_label, treemap_layout, wrap_index,
};
use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/context-treemap.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlockKind {
    Instructions,
    Capabilities,
    Recall,
    Document,
    #[default]
    Conversation,
    ToolOutput,
    Digest,
}

impl BlockKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Instructions => "instructions",
            Self::Capabilities => "capabilities",
            Self::Recall => "recall",
            Self::Document => "document",
            Self::Conversation => "conversation",
            Self::ToolOutput => "tool_output",
            Self::Digest => "digest",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Instructions => "Instructions",
            Self::Capabilities => "Capabilities",
            Self::Recall => "Recall",
            Self::Document => "Document",
            Self::Conversation => "Conversation",
            Self::ToolOutput => "Tool output",
            Self::Digest => "Digest",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub block_key: String,
    pub block_label: String,
    pub block_kind: BlockKind,
    pub token_count: f64,
    pub block_note: Option<String>,
    pub sub_items: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentOption {
    pub agent_id: String,
    pub agent_name: String,
    pub window_size: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompactionRecord {
    pub pre_blocks: Vec<Block>,
    pub folded_keys: Vec<String>,
    pub removed_keys: Vec<String>,
    pub digest_text: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum View {
    Before,
    #[default]
    After,
}

impl View {
    fn as_str(self) -> &'static str {
        match self {
            Self::Before => "before",
            Self::After => "after",
        }
    }
}

/// Tile positions in percent of the map, one per block then the free space last.
pub fn tile_rects(tokens: &[f64], window: f64) -> Vec<Rect> {
    let used: f64 = tokens.iter().map(|t| non_negative(*t)).sum();
    let free = (window - used).max(0.0);
    let mut sizes: Vec<f64> = tokens.iter().map(|t| non_negative(*t)).collect();
    sizes.push(free);
    treemap_layout(
        &sizes,
        Rect {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 100.0,
        },
    )
}

/// Used tokens and the fill text state: over the window, `full`; otherwise the share.
pub fn fill_state(used: f64, window: f64) -> (String, bool) {
    if window <= 0.0 || used >= window {
        ("full".to_string(), true)
    } else {
        (share_label(used / window), false)
    }
}

/// Blocks shown for a view: the before view uses the pre-compaction snapshot.
pub fn visible_blocks<'a>(
    blocks: &'a [Block],
    record: Option<&'a CompactionRecord>,
    view: View,
) -> &'a [Block] {
    match (view, record) {
        (View::Before, Some(record)) => &record.pre_blocks,
        _ => blocks,
    }
}

/// A treemap of the context window with a hover and click card per block.
#[component]
pub fn ContextTreemap(
    block_list: Vec<Block>,
    window_size: f64,
    #[props(default = "Context".to_string(), into)] panel_title: String,
    #[props(default, into)] model_name: String,
    #[props(default)] agent_options: Vec<AgentOption>,
    #[props(default)] active_agent_id: Option<String>,
    #[props(default)] on_agent_change: Option<EventHandler<String>>,
    #[props(default)] compaction_record: Option<CompactionRecord>,
    #[props(default)] active_view: Option<View>,
    #[props(default)] on_view_change: Option<EventHandler<View>>,
    #[props(default)] status_text: Option<String>,
    #[props(default)] on_block_pick: Option<EventHandler<Option<Block>>>,
    #[props(default, into)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-tmap-{}", next_id()));
    let mut view_local = use_signal(View::default);
    let mut hovered = use_signal(|| None::<String>);
    let held = use_signal(|| None::<String>);

    let view = active_view.unwrap_or(view_local());
    let window = match active_agent_id
        .as_ref()
        .and_then(|id| agent_options.iter().find(|a| a.agent_id == *id))
    {
        Some(agent) => non_negative(agent.window_size),
        None => non_negative(window_size),
    };
    let record = compaction_record.as_ref();
    let blocks: Vec<Block> = visible_blocks(&block_list, record, view).to_vec();
    let tokens: Vec<f64> = blocks.iter().map(|b| non_negative(b.token_count)).collect();
    let used: f64 = tokens.iter().sum();
    let (fill_text, over) = fill_state(used, window);
    let rects = tile_rects(&tokens, window);
    let free_rect = rects.last().copied().unwrap_or_default();
    let class = class_list("kit-tmap", &root_class);
    let title = panel_title.clone();
    let used_text = format!(
        "{} of {} tokens",
        compact_number(used),
        compact_number(window)
    );
    let card_key = held().or(hovered());
    let card_block = card_key
        .as_ref()
        .and_then(|k| blocks.iter().find(|b| b.block_key == *k).cloned());
    let status = status_text.clone().unwrap_or_default();
    let record_line = match record {
        None => "No compaction has run".to_string(),
        Some(r) => format!(
            "{} folded, {} removed{}",
            r.folded_keys.len(),
            r.removed_keys.len(),
            if r.digest_text.is_some() {
                ", digest written"
            } else {
                ""
            }
        ),
    };
    let has_record = record.is_some();
    let view_name = view.as_str();
    let key_base = base.clone();
    let agent_select_value = active_agent_id.clone().unwrap_or_default();

    let mut set_view = move |next: View| {
        if active_view.is_none() {
            view_local.set(next);
        }
        if let Some(handler) = &on_view_change {
            handler.call(next);
        }
    };
    let pick = move |block: Option<Block>| {
        if let Some(handler) = &on_block_pick {
            handler.call(block);
        }
    };

    let mut tiles: Vec<Element> = Vec::new();
    for (idx, block) in blocks.iter().enumerate() {
        let rect = rects[idx];
        let key = block.block_key.clone();
        let label = block.block_label.clone();
        let kind = block.block_kind.as_str();
        let kind_label = block.block_kind.label();
        let token_text = format!("{:.0}", tokens[idx]);
        let style = format!(
            "left: {:.3}%; top: {:.3}%; width: {:.3}%; height: {:.3}%",
            rect.x, rect.y, rect.w, rect.h
        );
        let aria = format!("{label}, {kind_label}, {token_text} tokens");
        let is_held = held().as_deref() == Some(key.as_str());
        let small = rect.w * rect.h < 260.0;
        let hover_key = key.clone();
        let click_key = key.clone();
        let click_block = block.clone();
        let mut hold = held;
        let pick_click = pick;
        tiles.push(rsx! {
            button {
                key: "{idx}-{key}",
                r#type: "button",
                class: "kit-tmap__tile",
                style: style,
                "data-kind": kind,
                "data-held": is_held.then_some("true"),
                "data-small": small.then_some("true"),
                "aria-label": "{aria}",
                "aria-pressed": is_held.then_some("true"),
                onmouseenter: move |_| hovered.set(Some(hover_key.clone())),
                onclick: move |_| {
                    let next = if held().as_deref() == Some(click_key.as_str()) { None } else { Some(click_key.clone()) };
                    hold.set(next.clone());
                    pick_click(Some(click_block.clone()));
                },
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::Escape {
                        hold.set(None);
                    }
                },
                if small {
                    span { class: "kit-tmap__sr", "{label}" }
                } else {
                    span { class: "kit-tmap__tile-label", "{label}" }
                    span { class: "kit-tmap__tile-tokens", "{token_text}" }
                }
            }
        });
    }

    let free_style = format!(
        "left: {:.3}%; top: {:.3}%; width: {:.3}%; height: {:.3}%",
        free_rect.x, free_rect.y, free_rect.w, free_rect.h
    );
    let free_present = free_rect.w * free_rect.h > 0.0;
    let free_label = format!("Free space, {:.0}%", free_rect.w);
    let card_view = card_block.map(|b| {
        let rows = b.sub_items.clone();
        (
            b.block_label.clone(),
            b.block_kind.label(),
            format!("{:.0} tokens", b.token_count),
            rows,
            b.block_note.clone().unwrap_or_default(),
        )
    });
    let agent_list: Vec<(String, String)> = agent_options
        .iter()
        .map(|a| (a.agent_id.clone(), a.agent_name.clone()))
        .collect();
    let mut legend: Vec<(&'static str, &'static str)> = Vec::new();
    for block in &blocks {
        let entry = (block.block_kind.as_str(), block.block_kind.label());
        if !legend.contains(&entry) {
            legend.push(entry);
        }
    }
    let views = [View::Before, View::After];
    let selected_view_index = if view == View::Before { 0 } else { 1 };
    let mut hold_all = held;
    let key_for_views = key_base.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class, "aria-label": title.clone(), "data-view": view_name, "data-over": over.then_some("true"),
            header { class: "kit-tmap__bar",
                div { class: "kit-tmap__titles",
                    h3 { class: "kit-tmap__title", "{title}" }
                    if !model_name.is_empty() {
                        span { class: "kit-tmap__model", "{model_name}" }
                    }
                }
                if !agent_list.is_empty() {
                    select {
                        class: "kit-tmap__agent",
                        "aria-label": "Agent",
                        value: agent_select_value,
                        onchange: move |event: FormEvent| {
                            if let Some(handler) = &on_agent_change {
                                handler.call(event.value());
                            }
                        },
                        for (id, name) in agent_list.clone() {
                            option { key: "{id}", value: "{id}", "{name}" }
                        }
                    }
                }
                span { class: "kit-tmap__used", "data-over": over.then_some("true"), "{used_text}" }
                span { class: "kit-tmap__fill", "data-over": over.then_some("true"), "{fill_text}" }
                span { class: "kit-tmap__status", role: "status", "aria-live": "polite", "{status}" }
                if has_record {
                    div { class: "kit-tmap__views", role: "radiogroup", "aria-label": "Compaction view",
                        onkeydown: move |event: KeyboardEvent| {
                            let next = match event.key() {
                                Key::ArrowRight | Key::ArrowLeft => {
                                    let forward = event.key() == Key::ArrowRight;
                                    Some(views[wrap_index(selected_view_index, 2, forward)])
                                }
                                _ => None,
                            };
                            if let Some(next) = next {
                                event.prevent_default();
                                set_view(next);
                                focus_element(&format!("{key_for_views}-view-{}", if next == View::Before { 0 } else { 1 }));
                            }
                        },
                        for (i, option) in views.iter().copied().enumerate() {
                            button {
                                key: "{i}",
                                id: "{key_base}-view-{i}",
                                r#type: "button",
                                role: "radio",
                                class: "kit-tmap__view",
                                "aria-checked": if option == view { "true" } else { "false" },
                                tabindex: if option == view { "0" } else { "-1" },
                                onclick: move |_| set_view(option),
                                if option == View::Before { "Before" } else { "After" }
                            }
                        }
                    }
                }
            }
            div {
                class: "kit-tmap__map",
                role: "group",
                "aria-label": title.clone(),
                onmouseleave: move |_| hovered.set(None),
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::Escape {
                        hold_all.set(None);
                    }
                },
                div {
                    class: "kit-tmap__free",
                    style: free_style,
                    "data-present": free_present.then_some("true"),
                    onclick: move |_| {
                        hold_all.set(None);
                        pick(None);
                    },
                    span { class: "kit-tmap__sr", "{free_label}" }
                }
                {tiles.into_iter()}
                if let Some((label, kind, tokens_text, rows, note)) = card_view {
                    div { class: "kit-tmap__card", role: "status", "aria-live": "polite",
                        div { class: "kit-tmap__card-head",
                            span { class: "kit-tmap__card-label", "{label}" }
                            span { class: "kit-tmap__card-kind", "{kind}" }
                        }
                        span { class: "kit-tmap__card-tokens", "{tokens_text}" }
                        if !note.is_empty() {
                            p { class: "kit-tmap__card-note", "{note}" }
                        }
                        if !rows.is_empty() {
                            ul { class: "kit-tmap__card-items",
                                for (i, row) in rows.into_iter().enumerate() {
                                    li { key: "{i}", "{row}" }
                                }
                            }
                        }
                    }
                }
            }
            if !legend.is_empty() {
                ul { class: "kit-tmap__legend",
                    for (kind, label) in legend.clone() {
                        li { key: "{kind}", class: "kit-tmap__legend-row",
                            span { class: "kit-tmap__legend-swatch", "data-kind": kind, "aria-hidden": "true" }
                            "{label}"
                        }
                    }
                }
            }
            p { class: "kit-tmap__record", "{record_line}" }
        }
    }
}
