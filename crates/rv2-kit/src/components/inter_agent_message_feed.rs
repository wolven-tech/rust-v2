//! Inter-agent message feed: the messages agents exchange during a multi-agent
//! run, with participant tabs, a details card per selected message, a composer
//! with an `@` picker, and attachment previews.
//!
//! Design choice: a message is "received" by a participant when it is addressed
//! to that participant or to `all`; the participant tab shows both.
//! Design choice: a "needs you" point stays tagged until an entry from `you`
//! names it in `answers_id`.
//! Design choice: `@` mentions match a participant's display name with spaces
//! removed, case-insensitively; with no mention, a reply goes to the entry's
//! sender, and otherwise to `all`.
//! Design choice: the feed follows new entries while `is_running`; it does not
//! detect that the reader scrolled up.
//! Design choice: the composer is a native textarea; Shift+Enter inserts a line.
//! Browser checks: the preview dialog's focus trap, the flash on a quote jump,
//! the @ listbox rise, the wrap of bubbles at phone width, and the tab scroll.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/inter-agent-message-feed.css");

pub const YOU: &str = "you";
pub const ALL: &str = "all";

#[derive(Clone, PartialEq, Debug)]
pub struct Participant {
    pub participant_id: String,
    pub display_name: String,
    pub model_name: Option<String>,
    pub badge_mark: Option<String>,
    pub logo: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntryKind {
    TaskRequest,
    Answer,
    Handover,
    Broadcast,
    NeedsPerson,
    AgentStarted,
}

impl EntryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TaskRequest => "task_request",
            Self::Answer => "answer",
            Self::Handover => "handover",
            Self::Broadcast => "broadcast",
            Self::NeedsPerson => "needs_person",
            Self::AgentStarted => "agent_started",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::TaskRequest => "Task request",
            Self::Answer => "Answer",
            Self::Handover => "Handover",
            Self::Broadcast => "Broadcast",
            Self::NeedsPerson => "Needs you",
            Self::AgentStarted => "Started",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttachmentKind {
    File,
    Folder,
    Table,
}

impl AttachmentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Folder => "folder",
            Self::Table => "table",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Attachment {
    pub label: String,
    pub kind: Option<AttachmentKind>,
    pub preview: Option<String>,
    pub meta: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct DetailCard {
    pub notes: Option<String>,
    pub rows: Vec<(String, String)>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Entry {
    pub entry_id: String,
    pub sender_id: String,
    pub recipient_ids: Vec<String>,
    pub entry_kind: EntryKind,
    pub body_text: String,
    pub sent_at: Option<i64>,
    pub answers_id: Option<String>,
    pub is_drafting: bool,
    pub attachment_list: Vec<Attachment>,
    pub detail_card: Option<DetailCard>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ComposeMessage {
    pub text: String,
    pub recipient_ids: Vec<String>,
    pub answers_id: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FeedKey {
    Up,
    Down,
    Home,
    End,
}

/// Table extensions; a label with one of these is shown as a table.
const TABLE_EXTENSIONS: [&str; 6] = ["csv", "tsv", "xlsx", "xls", "parquet", "ods"];

/// An explicit kind wins; otherwise a trailing slash is a folder and a tabular extension a table.
pub fn attachment_kind(label: &str, explicit: Option<AttachmentKind>) -> AttachmentKind {
    if let Some(kind) = explicit {
        return kind;
    }
    if label.ends_with('/') {
        return AttachmentKind::Folder;
    }
    match label.rsplit_once('.').map(|(_, e)| e.to_lowercase()) {
        Some(ext) if TABLE_EXTENSIONS.contains(&ext.as_str()) => AttachmentKind::Table,
        _ => AttachmentKind::File,
    }
}

/// True when entry `index` is a point for the person that no entry from `you` has answered.
pub fn is_awaiting_you(entries: &[Entry], index: usize) -> bool {
    let Some(entry) = entries.get(index) else {
        return false;
    };
    if entry.entry_kind != EntryKind::NeedsPerson || !entry.recipient_ids.iter().any(|r| r == YOU) {
        return false;
    }
    !entries
        .iter()
        .any(|e| e.sender_id == YOU && e.answers_id.as_deref() == Some(entry.entry_id.as_str()))
}

/// Indices of entries shown for the focused participant, or all when none is focused.
pub fn visible_indices(entries: &[Entry], focused: Option<&str>) -> Vec<usize> {
    entries
        .iter()
        .enumerate()
        .filter(|(_, e)| match focused {
            None => true,
            Some(p) => e.sender_id == p || e.recipient_ids.iter().any(|r| r == p || r == ALL),
        })
        .map(|(i, _)| i)
        .collect()
}

/// The `@` token being typed at the end of `text`, without the `@`, if any.
pub fn active_mention(text: &str) -> Option<String> {
    let token_start = text.rfind(char::is_whitespace).map_or(0, |i| i + 1);
    text[token_start..].strip_prefix('@').map(str::to_string)
}

/// A display name as it is typed after `@`: spaces removed.
pub fn mention_token(name: &str) -> String {
    name.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Participants whose mention token starts with `query` (case-insensitive).
pub fn mention_candidates<'a>(
    participants: &'a [Participant],
    query: &str,
) -> Vec<&'a Participant> {
    let needle = query.to_lowercase();
    participants
        .iter()
        .filter(|p| {
            mention_token(&p.display_name)
                .to_lowercase()
                .starts_with(&needle)
        })
        .collect()
}

/// Replaces the trailing `@query` with `@Name `.
pub fn complete_mention(text: &str, participant: &Participant) -> String {
    let token_start = text.rfind(char::is_whitespace).map_or(0, |i| i + 1);
    format!(
        "{}@{} ",
        &text[..token_start],
        mention_token(&participant.display_name)
    )
}

/// Recipients named with `@` in `text`, else the reply target's sender, else `all`.
pub fn parse_recipients(
    text: &str,
    participants: &[Participant],
    reply_sender: Option<&str>,
) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        let Some(name) = word.strip_prefix('@') else {
            continue;
        };
        let name = name.trim_end_matches(|c: char| c.is_ascii_punctuation());
        let matched = participants
            .iter()
            .find(|p| mention_token(&p.display_name).eq_ignore_ascii_case(name));
        if let Some(p) = matched {
            if !found.contains(&p.participant_id) {
                found.push(p.participant_id.clone());
            }
        }
    }
    if !found.is_empty() {
        return found;
    }
    match reply_sender {
        Some(sender) => vec![sender.to_string()],
        None => vec![ALL.to_string()],
    }
}

/// The next entry index for a roving-focus key; `None` for an empty feed.
pub fn next_entry(len: usize, current: usize, key: FeedKey) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match key {
        FeedKey::Home => 0,
        FeedKey::End => len - 1,
        FeedKey::Up => current.saturating_sub(1),
        FeedKey::Down => (current + 1).min(len - 1),
    })
}

/// A quote of the answered text, cut to `limit` characters.
pub fn quote_text(body: &str, limit: usize) -> String {
    if body.chars().count() <= limit {
        body.to_string()
    } else {
        let cut: String = body.chars().take(limit).collect();
        format!("{cut}…")
    }
}

fn name_of(participants: &[Participant], id: &str) -> String {
    match id {
        YOU => "you".to_string(),
        ALL => "everyone".to_string(),
        _ => participants
            .iter()
            .find(|p| p.participant_id == id)
            .map(|p| p.display_name.clone())
            .unwrap_or_else(|| id.to_string()),
    }
}

/// `HH:MM` in UTC from epoch seconds.
fn clock_text(epoch_secs: i64) -> String {
    let day_secs = epoch_secs.rem_euclid(86_400);
    format!("{:02}:{:02}", day_secs / 3600, (day_secs % 3600) / 60)
}

fn focus_id(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

fn scroll_into_view(id: &str) {
    document::eval(&format!(
        "document.getElementById({id:?})?.scrollIntoView({{block: 'center', behavior: 'smooth'}});"
    ));
}

/// Whether the log is scrolled up from its newest message.
async fn log_scrolled_away(id: String) -> bool {
    let js = format!(
        "const el = document.getElementById({id:?}); return !!el && el.scrollHeight - el.scrollTop - el.clientHeight > 24;"
    );
    document::eval(&js)
        .await
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn scroll_to_end(id: &str) {
    document::eval(&format!(
        "const el = document.getElementById({id:?}); if (el) el.scrollTop = el.scrollHeight;"
    ));
}

/// A feed of agent messages with participant tabs, a details card and a composer.
#[component]
pub fn InterAgentMessageFeed(
    #[props(default)] participants: Vec<Participant>,
    #[props(default)] entries: Vec<Entry>,
    #[props(default)] run_subject: Option<String>,
    #[props(default)] is_running: bool,
    #[props(default)] picked_entry_id: Option<String>,
    #[props(default)] on_pick_change: Option<EventHandler<Option<String>>>,
    #[props(default)] focused_participant_id: Option<String>,
    #[props(default)] on_focus_change: Option<EventHandler<Option<String>>>,
    #[props(default)] on_compose: Option<EventHandler<ComposeMessage>>,
    #[props(default = "Write to a participant – @ to pick one".to_string(), into)]
    compose_hint: String,
    #[props(default = 480.0)] scroll_height: f64,
    #[props(default = "Agent messages".to_string(), into)] feed_name: String,
    #[props(default, into)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-feed-{}", next_id()));
    let mut internal_pick = use_signal(|| Option::<String>::None);
    let mut internal_focus = use_signal(|| Option::<String>::None);
    let mut reply_to = use_signal(|| Option::<String>::None);
    let mut draft = use_signal(String::new);
    let mut mention_dismissed = use_signal(|| false);
    let mut mention_index = use_signal(|| 0_usize);
    let mut preview = use_signal(|| Option::<Attachment>::None);
    let mut preview_return = use_signal(|| Option::<String>::None);
    let mut flash = use_signal(|| Option::<String>::None);
    let mut roving = use_signal(|| 0_usize);
    let mut announce = use_signal(String::new);
    let mut paused = use_signal(|| false);

    let pick_controlled = on_pick_change.is_some();
    let focus_controlled = on_focus_change.is_some();
    let picked_now: Option<String> = if pick_controlled {
        picked_entry_id.clone()
    } else {
        internal_pick()
    };
    let focused_now: Option<String> = if focus_controlled {
        focused_participant_id.clone()
    } else {
        internal_focus()
    };
    let visible = visible_indices(&entries, focused_now.as_deref());
    let roving_index = roving().min(visible.len().saturating_sub(1));
    let reply = reply_to();
    let reply_sender = reply.as_ref().and_then(|id| {
        entries
            .iter()
            .find(|e| &e.entry_id == id)
            .map(|e| e.sender_id.clone())
    });
    let reply_label = reply_sender
        .as_ref()
        .map(|sender| format!("Replying to {}", name_of(&participants, sender)));
    let text = draft();
    let mention = active_mention(&text).filter(|_| !mention_dismissed());
    let candidates: Vec<Participant> = mention
        .as_deref()
        .map(|q| {
            mention_candidates(&participants, q)
                .into_iter()
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let menu_open = !candidates.is_empty();
    let active_option = mention_index().min(candidates.len().saturating_sub(1));
    let preview_now = preview();
    let flashing = flash();
    let announcement = announce();
    let all_pressed = if focused_now.is_none() {
        "true"
    } else {
        "false"
    };
    let subject = run_subject.clone().unwrap_or_default();
    let root_cls = class_list("kit-feed", &root_class);
    let feed_dom = format!("{base}-log");
    let input_dom = format!("{base}-input");
    let send_dom = format!("{base}-send");
    let listbox_id = format!("{base}-mentions");
    let at_expanded = if menu_open { "true" } else { "false" };
    let aria_live = if is_running { "polite" } else { "off" };
    let max_height = format!("max-height: {scroll_height}px;");
    let send_disabled = text.trim().is_empty();
    let controls_attr = if menu_open {
        listbox_id.clone()
    } else {
        String::new()
    };

    let paused_now = paused();
    let feed_for_effect = feed_dom.clone();
    use_effect(use_reactive(
        (&entries.len(), &is_running, &paused_now),
        move |(_, running, held)| {
            if running && !held {
                scroll_to_end(&feed_for_effect);
            }
        },
    ));

    let send = {
        let participants = participants.clone();
        let reply_sender = reply_sender.clone();
        move || {
            let text = draft();
            if text.trim().is_empty() {
                return;
            }
            let recipient_ids = parse_recipients(&text, &participants, reply_sender.as_deref());
            let message = ComposeMessage {
                text,
                recipient_ids,
                answers_id: reply_to(),
            };
            draft.set(String::new());
            reply_to.set(None);
            mention_dismissed.set(false);
            announce.set("Message sent".to_string());
            if let Some(handler) = on_compose {
                handler.call(message);
            }
        }
    };

    let key_nav = {
        let visible = visible.clone();
        let picked_key = picked_now.clone();
        let focused_key = focused_now.clone();
        let feed_key = base.clone();
        move |event: KeyboardEvent| {
            let key = match event.key() {
                Key::ArrowDown => FeedKey::Down,
                Key::ArrowUp => FeedKey::Up,
                Key::Home => FeedKey::Home,
                Key::End => FeedKey::End,
                Key::Escape => {
                    event.prevent_default();
                    if picked_key.is_some() {
                        if !pick_controlled {
                            internal_pick.set(None);
                        }
                        if let Some(handler) = on_pick_change {
                            handler.call(None);
                        }
                    } else if focused_key.is_some() {
                        if !focus_controlled {
                            internal_focus.set(None);
                        }
                        if let Some(handler) = on_focus_change {
                            handler.call(None);
                        }
                    }
                    return;
                }
                _ => return,
            };
            event.prevent_default();
            if let Some(next) = next_entry(visible.len(), roving_index, key) {
                roving.set(next);
                focus_id(&format!("{feed_key}-entry-{}", visible[next]));
            }
        }
    };

    let key_composer = {
        let mut send = send.clone();
        let candidate_names = candidates.clone();
        move |event: KeyboardEvent| {
            if menu_open {
                match event.key() {
                    Key::ArrowDown => {
                        event.prevent_default();
                        mention_index.set((active_option + 1) % candidate_names.len());
                        return;
                    }
                    Key::ArrowUp => {
                        event.prevent_default();
                        let len = candidate_names.len();
                        mention_index.set((active_option + len - 1) % len);
                        return;
                    }
                    Key::Enter | Key::Tab => {
                        event.prevent_default();
                        let chosen = candidate_names[active_option].clone();
                        draft.set(complete_mention(&draft(), &chosen));
                        return;
                    }
                    Key::Escape => {
                        event.prevent_default();
                        mention_dismissed.set(true);
                        return;
                    }
                    _ => {}
                }
            }
            if matches!(event.key(), Key::Escape) && reply_to().is_some() {
                reply_to.set(None);
                return;
            }
            if matches!(event.key(), Key::Enter) && !event.modifiers().shift() {
                event.prevent_default();
                send();
            }
        }
    };

    let open_preview_return = preview_return();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: root_cls, "aria-label": "{feed_name}",
            header { class: "kit-feed__head",
                h2 { class: "kit-feed__subject", "{subject}" }
                if is_running {
                    span { class: "kit-feed__live",
                        span { class: "kit-feed__live-dot", "aria-hidden": "true" }
                        "Live"
                    }
                }
                div { class: "kit-feed__tabs", role: "toolbar", "aria-label": "Participants",
                    button {
                        r#type: "button",
                        class: "kit-feed__tab",
                        "aria-pressed": "{all_pressed}",
                        onclick: move |_| {
                            if !focus_controlled {
                                internal_focus.set(None);
                            }
                            if let Some(handler) = on_focus_change {
                                handler.call(None);
                            }
                        },
                        "All"
                    }
                    for participant in participants.iter() {
                        {
                            let id = participant.participant_id.clone();
                            let pressed = focused_now.as_deref() == Some(id.as_str());
                            let pressed_attr = if pressed { "true" } else { "false" };
                            let name = participant.display_name.clone();
                            let key = id.clone();
                            let was_focused = pressed;
                            rsx! {
                                button {
                                    key: "{key}",
                                    r#type: "button",
                                    class: "kit-feed__tab",
                                    "aria-pressed": "{pressed_attr}",
                                    onclick: move |_| {
                                        let next = if was_focused { None } else { Some(id.clone()) };
                                        if !focus_controlled {
                                            internal_focus.set(next.clone());
                                        }
                                        if let Some(handler) = on_focus_change {
                                            handler.call(next);
                                        }
                                    },
                                    "{name}"
                                }
                            }
                        }
                    }
                }
            }
            if paused_now && is_running {
                button {
                    r#type: "button",
                    class: "kit-feed__jump",
                    onclick: {
                        let feed = feed_dom.clone();
                        move |_| {
                            paused.set(false);
                            scroll_to_end(&feed);
                        }
                    },
                    "New messages · Jump to latest"
                }
            }
            div {
                id: "{feed_dom}",
                class: "kit-feed__log",
                role: "log",
                "aria-live": "{aria_live}",
                "aria-label": "{feed_name}",
                style: "{max_height}",
                onscroll: {
                    let feed = feed_dom.clone();
                    move |_| {
                        let feed = feed.clone();
                        spawn(async move {
                            paused.set(log_scrolled_away(feed).await);
                        });
                    }
                },
                onkeydown: key_nav,
                if visible.is_empty() {
                    p { class: "kit-feed__empty", "No messages yet" }
                }
                for (position, index) in visible.iter().copied().enumerate() {
                    {
                        let entry = entries[index].clone();
                        let entry_dom = format!("{base}-entry-{index}");
                        let entry_key = entry.entry_id.clone();
                        let is_picked = picked_now.as_deref() == Some(entry.entry_id.as_str());
                        let dimmed = picked_now.is_some() && !is_picked;
                        let dimmed_attr = if dimmed { "true" } else { "false" };
                        let tab_index = if position == roving_index { "0" } else { "-1" };
                        let sender = name_of(&participants, &entry.sender_id);
                        let to_names: Vec<String> = entry
                            .recipient_ids
                            .iter()
                            .map(|r| name_of(&participants, r))
                            .collect();
                        let to_text = to_names.join(", ");
                        let time = entry.sent_at.map(clock_text).unwrap_or_default();
                        let awaiting = is_awaiting_you(&entries, index);
                        let kind_name = entry.entry_kind.as_str();
                        let kind_label = entry.entry_kind.label();
                        let show_kind = entry.entry_kind != EntryKind::Answer;
                        let quote = entry.answers_id.as_ref().and_then(|answer| {
                            let target = entries.iter().position(|e| &e.entry_id == answer)?;
                            if target + 1 == index {
                                None
                            } else {
                                Some((
                                    format!("{base}-entry-{target}"),
                                    quote_text(&entries[target].body_text, 80),
                                ))
                            }
                        });
                        let drafting = entry.is_drafting;
                        let drafting_attr = if drafting { "true" } else { "false" };
                        let body = if drafting { "Drafting…".to_string() } else { entry.body_text.clone() };
                        let label = format!("{sender} to {to_text}, {time}: {body}");
                        let is_flashing = flashing.as_deref() == Some(entry_dom.as_str());
                        let bubble_class = if is_picked {
                            "kit-feed__entry is-picked"
                        } else if is_flashing {
                            "kit-feed__entry is-flash"
                        } else {
                            "kit-feed__entry"
                        };
                        let pick_value = entry.entry_id.clone();
                        let reply_value = entry.entry_id.clone();
                        let reply_focus = input_dom.clone();
                        let details = entry.detail_card.clone();
                        let attachments = entry.attachment_list.clone();
                        let mark = participants
                            .iter()
                            .find(|p| p.participant_id == entry.sender_id)
                            .and_then(|p| p.badge_mark.clone())
                            .unwrap_or_else(|| sender.chars().take(1).collect());
                        let mark_class = if drafting { "kit-feed__mark is-drafting" } else { "kit-feed__mark" };
                        let aria_selected = if is_picked { "true" } else { "false" };
                        let show_time = !time.is_empty();
                        rsx! {
                            div { key: "{entry_key}", class: "kit-feed__item", "data-dimmed": "{dimmed_attr}",
                                div {
                                    id: "{entry_dom}",
                                    class: bubble_class,
                                    "data-kind": kind_name,
                                    "data-drafting": "{drafting_attr}",
                                    tabindex: "{tab_index}",
                                    "aria-label": "{label}",
                                    "aria-selected": "{aria_selected}",
                                    onclick: move |_| {
                                        let next = if is_picked { None } else { Some(pick_value.clone()) };
                                        if !pick_controlled {
                                            internal_pick.set(next.clone());
                                        }
                                        if let Some(handler) = on_pick_change {
                                            handler.call(next);
                                        }
                                    },
                                    header { class: "kit-feed__meta",
                                        span { class: "kit-feed__badge", "aria-hidden": "true",
                                            span { class: "{mark_class}", "{mark}" }
                                        }
                                        span { class: "kit-feed__sender", "{sender}" }
                                        span { class: "kit-feed__to", "to {to_text}" }
                                        if show_time {
                                            time { class: "kit-feed__time", "datetime": "{time}", "{time}" }
                                        }
                                        if show_kind {
                                            span { class: "kit-feed__kind", "{kind_label}" }
                                        }
                                        if awaiting {
                                            span { class: "kit-feed__await", "Awaiting you" }
                                        }
                                    }
                                    if let Some((target_dom, quote_body)) = quote {
                                        button {
                                            r#type: "button",
                                            class: "kit-feed__quote",
                                            tabindex: "{tab_index}",
                                            onclick: move |event: MouseEvent| {
                                                event.stop_propagation();
                                                scroll_into_view(&target_dom);
                                                flash.set(Some(target_dom.clone()));
                                            },
                                            "↩ {quote_body}"
                                        }
                                    }
                                    if drafting {
                                        p { class: "kit-feed__drafting",
                                            span { class: "kit-feed__dots", "aria-hidden": "true" }
                                            "Drafting…"
                                        }
                                    } else {
                                        p { class: "kit-feed__text", "{body}" }
                                    }
                                    if !attachments.is_empty() {
                                        ul { class: "kit-feed__chips",
                                            for (chip_index, attachment) in attachments.iter().cloned().enumerate() {
                                                {
                                                    let kind = attachment_kind(&attachment.label, attachment.kind);
                                                    let kind_word = kind.as_str();
                                                    let chip_dom = format!("{entry_dom}-chip-{chip_index}");
                                                    let chip_label = attachment.label.clone();
                                                    let chip_aria = format!("{kind_word} {chip_label}");
                                                    let chip_key = chip_dom.clone();
                                                    let chip_return = chip_dom.clone();
                                                    rsx! {
                                                        li { key: "{chip_key}",
                                                            button {
                                                                id: "{chip_dom}",
                                                                r#type: "button",
                                                                class: "kit-feed__chip",
                                                                "data-kind": kind_word,
                                                                tabindex: "{tab_index}",
                                                                "aria-label": "{chip_aria}",
                                                                onclick: move |event: MouseEvent| {
                                                                    event.stop_propagation();
                                                                    preview_return.set(Some(chip_return.clone()));
                                                                    preview.set(Some(attachment.clone()));
                                                                },
                                                                "{chip_label}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    button {
                                        r#type: "button",
                                        class: "kit-feed__reply",
                                        tabindex: "{tab_index}",
                                        onclick: move |event: MouseEvent| {
                                            event.stop_propagation();
                                            reply_to.set(Some(reply_value.clone()));
                                            focus_id(&reply_focus);
                                        },
                                        "Reply"
                                    }
                                }
                                if is_picked {
                                    if let Some(card) = details {
                                        div { class: "kit-feed__details",
                                            if let Some(notes) = card.notes {
                                                p { class: "kit-feed__notes", "{notes}" }
                                            }
                                            dl {
                                                for (label, value) in card.rows {
                                                    {
                                                        let key = label.clone();
                                                        rsx! {
                                                            div { key: "{key}",
                                                                dt { "{label}" }
                                                                dd { "{value}" }
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
            if let Some(reply_text) = reply_label {
                div { class: "kit-feed__replying",
                    span { "{reply_text}" }
                    button {
                        r#type: "button",
                        class: "kit-feed__cancel-reply",
                        onclick: move |_| reply_to.set(None),
                        "Cancel"
                    }
                }
            }
            form {
                class: "kit-feed__composer",
                onsubmit: {
                    let mut send = send.clone();
                    move |event: FormEvent| {
                        event.prevent_default();
                        send();
                    }
                },
                div { class: "kit-feed__input-row",
                    textarea {
                        id: "{input_dom}",
                        class: "kit-feed__input",
                        rows: "2",
                        "aria-label": "Message",
                        "aria-autocomplete": "list",
                        "aria-controls": "{controls_attr}",
                        placeholder: "{compose_hint}",
                        value: "{text}",
                        oninput: move |event: FormEvent| {
                            draft.set(event.value());
                            mention_dismissed.set(false);
                            mention_index.set(0);
                        },
                        onkeydown: key_composer,
                    }
                    button {
                        r#type: "button",
                        class: "kit-feed__at",
                        "aria-label": "Mention a participant",
                        "aria-expanded": "{at_expanded}",
                        onclick: move |_| {
                            if menu_open {
                                mention_dismissed.set(true);
                            } else {
                                draft.set(format!("{}@", draft()));
                                mention_dismissed.set(false);
                            }
                        },
                        "@"
                    }
                    button {
                        id: "{send_dom}",
                        r#type: "submit",
                        class: "kit-feed__send",
                        disabled: send_disabled,
                        "Send"
                    }
                }
                if menu_open {
                    ul { id: "{listbox_id}", class: "kit-feed__mentions", role: "listbox", "aria-label": "Participants",
                        for (option_index, candidate) in candidates.iter().enumerate() {
                            {
                                let name = candidate.display_name.clone();
                                let selected = if option_index == active_option { "true" } else { "false" };
                                let chosen = candidate.clone();
                                let key = candidate.participant_id.clone();
                                rsx! {
                                    li {
                                        key: "{key}",
                                        role: "option",
                                        "aria-selected": "{selected}",
                                        onmousedown: move |event: MouseEvent| {
                                            event.prevent_default();
                                            draft.set(complete_mention(&draft(), &chosen));
                                        },
                                        "{name}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "kit-feed__sr", role: "status", "aria-live": "polite", "{announcement}" }
            if let Some(attachment) = preview_now {
                {
                    let preview_label = attachment.label.clone();
                    let preview_meta = attachment.meta.clone();
                    let preview_body = attachment.preview.clone().unwrap_or_default();
                    let close_return = open_preview_return.clone();
                    let key_return = open_preview_return.clone();
                    rsx! {
                        div { class: "kit-feed__preview-scrim", onclick: move |_| preview.set(None) }
                        div {
                            class: "kit-feed__preview",
                            role: "dialog",
                            "aria-modal": "true",
                            "aria-label": "{preview_label}",
                            onkeydown: move |event: KeyboardEvent| {
                                if matches!(event.key(), Key::Escape) {
                                    event.prevent_default();
                                    preview.set(None);
                                    if let Some(id) = key_return.clone() {
                                        focus_id(&id);
                                    }
                                }
                            },
                            header { class: "kit-feed__preview-head",
                                span { "{preview_label}" }
                                button {
                                    r#type: "button",
                                    class: "kit-feed__cancel-reply",
                                    "aria-label": "Close preview",
                                    onclick: move |_| {
                                        preview.set(None);
                                        if let Some(id) = close_return.clone() {
                                            focus_id(&id);
                                        }
                                    },
                                    "×"
                                }
                            }
                            if let Some(meta) = preview_meta {
                                p { class: "kit-feed__meta", "{meta}" }
                            }
                            pre { class: "kit-feed__preview-body", "{preview_body}" }
                        }
                    }
                }
            }
        }
    }
}
