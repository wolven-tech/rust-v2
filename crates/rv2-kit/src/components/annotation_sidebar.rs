//! Annotation sidebar: a column of notes beside a document, with filters, a
//! sort menu, a composer, and `/patch` notes that can be applied.
//!
//! Design choice: `on_run_patch` returns a boxed future, so the working and
//! failed states follow the outcome of the patch.
//! Design choice: "Queries" are the notes that are not patches.
//! Design choice: `now_secs` supplies the clock (wasm32 has no `SystemTime`)
//! and stamps notes created in the composer.
//! Design choice: a composed note stays listed until the host's `notes` carries the same `note_id`,
//! so the list is right with or without `on_note_added`.
//! Design choice: the `datetime` attribute carries epoch seconds, not ISO text.
//! Browser checks: the sheet layout at phone width, the fade-in of new notes,
//! focus return from the sort menu, and the polite announcements.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/annotation-sidebar.css");

/// The outcome of a patch run; `Err` carries the message shown under the note.
pub type PatchFuture = Pin<Box<dyn Future<Output = Result<(), String>>>>;

#[derive(Clone, PartialEq, Debug)]
pub struct Writer {
    pub writer_name: String,
    pub writer_mark: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NoteState {
    #[default]
    Open,
    Resolved,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Note {
    pub note_id: String,
    pub writer: Writer,
    /// Epoch seconds.
    pub posted_at: i64,
    pub note_text: String,
    pub note_state: NoteState,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NoteFilter {
    #[default]
    All,
    Queries,
    Patches,
}

impl NoteFilter {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Queries => "queries",
            Self::Patches => "patches",
        }
    }

    pub fn matches(self, note: &Note) -> bool {
        match self {
            Self::All => true,
            Self::Queries => !is_patch(&note.note_text),
            Self::Patches => is_patch(&note.note_text),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NoteSort {
    #[default]
    Newest,
    Oldest,
}

#[derive(Clone, PartialEq, Debug)]
pub enum PatchState {
    Running,
    Failed(String),
}

/// A note whose text begins with `/patch` is runnable.
pub fn is_patch(text: &str) -> bool {
    text.starts_with("/patch")
}

/// A note is resolved when the viewer toggled it, otherwise when its state says so.
pub fn is_resolved(note: &Note, overrides: &BTreeMap<String, bool>) -> bool {
    overrides
        .get(&note.note_id)
        .copied()
        .unwrap_or(note.note_state == NoteState::Resolved)
}

/// Counts for All, Queries and Patches, in that order.
pub fn filter_counts(notes: &[Note]) -> (usize, usize, usize) {
    let all = notes.len();
    let patches = notes.iter().filter(|n| is_patch(&n.note_text)).count();
    (all, all - patches, patches)
}

/// Filters, orders and optionally hides resolved notes. Ties keep input order.
pub fn visible_notes(
    notes: &[Note],
    filter: NoteFilter,
    sort: NoteSort,
    overrides: &BTreeMap<String, bool>,
    hide_resolved: bool,
) -> Vec<Note> {
    let mut out: Vec<Note> = notes
        .iter()
        .filter(|n| filter.matches(n))
        .filter(|n| !(hide_resolved && is_resolved(n, overrides)))
        .cloned()
        .collect();
    match sort {
        NoteSort::Newest => out.sort_by_key(|n| std::cmp::Reverse(n.posted_at.clone())),
        NoteSort::Oldest => out.sort_by_key(|n| n.posted_at.clone()),
    }
    out
}

/// Coarse age label; the caller supplies `now_secs`.
pub fn relative_label(now_secs: i64, posted_at: i64) -> String {
    let delta = now_secs - posted_at;
    if delta < 60 {
        "just now".to_string()
    } else if delta < 3_600 {
        format!("{} min ago", delta / 60)
    } else if delta < 86_400 {
        format!("{} h ago", delta / 3_600)
    } else {
        format!("{} d ago", delta / 86_400)
    }
}

/// Whether the list is scrolled away from its new-notes end: the top when newest
/// notes come first, the bottom otherwise.
async fn list_scrolled_away(id: String, toward_top: bool) -> bool {
    let js = if toward_top {
        format!("const el = document.getElementById({id:?}); return !!el && el.scrollTop > 24;")
    } else {
        format!(
            "const el = document.getElementById({id:?}); return !!el && el.scrollHeight - el.scrollTop - el.clientHeight > 24;"
        )
    };
    document::eval(&js)
        .await
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn jump_to_new_end(id: &str, toward_top: bool) {
    let target = if toward_top { "0" } else { "el.scrollHeight" };
    document::eval(&format!(
        "const el = document.getElementById({id:?}); if (el) el.scrollTop = {target};"
    ));
}

fn focus_id(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

/// Moves DOM focus to the next or previous `menuitemradio` inside the menu, wrapping at both ends.
fn step_menu_focus(menu_id: &str, step: i32) {
    document::eval(&format!(
        "const items = [...document.querySelectorAll('#{menu_id} [role=menuitemradio]')]; if (items.length) {{ const at = items.indexOf(document.activeElement); items[(at + {step} + items.length) % items.length].focus(); }}"
    ));
}

#[derive(Clone, PartialEq, Debug)]
struct NoteView {
    note: Note,
    resolved: bool,
    patch: bool,
    running: bool,
    error: Option<String>,
    relative: String,
}

/// A column of notes with a filter, a sort menu, runnable patch notes and a composer.
#[component]
pub fn AnnotationSidebar(
    #[props(default)] notes: Vec<Note>,
    acting_writer: Writer,
    #[props(default)] on_run_patch: Option<Callback<Note, PatchFuture>>,
    #[props(default)] on_note_added: Option<EventHandler<Note>>,
    #[props(default)] on_close_request: Option<EventHandler<()>>,
    #[props(default = "Notes".to_string(), into)] section_title: String,
    #[props(default)] now_secs: i64,
    #[props(default, into)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-annot-{}", next_id()));
    let mut filter = use_signal(NoteFilter::default);
    let mut sort = use_signal(NoteSort::default);
    let mut sort_open = use_signal(|| false);
    let mut hide_resolved = use_signal(|| false);
    let mut overrides = use_signal(BTreeMap::<String, bool>::new);
    let mut added = use_signal(Vec::<Note>::new);
    let mut patches = use_signal(BTreeMap::<String, PatchState>::new);
    let mut draft = use_signal(String::new);
    let mut announce = use_signal(String::new);
    let mut away = use_signal(|| false);
    let mut unseen = use_signal(|| 0_usize);
    let mut seen_len = use_signal(|| 0_usize);

    let all: Vec<Note> = notes
        .iter()
        .cloned()
        .chain(
            added()
                .into_iter()
                .filter(|composed| !notes.iter().any(|note| note.note_id == composed.note_id)),
        )
        .collect();
    let all_len = all.len();
    let toward_top = sort() == NoteSort::Newest;
    let list_dom = format!("{base}-list");
    let resume_label = format!("{} new notes · Show", unseen());
    use_effect(use_reactive((&all_len,), move |(len,)| {
        let previous = *seen_len.peek();
        if len > previous && *away.peek() {
            let next = *unseen.peek() + (len - previous);
            unseen.set(next);
        }
        seen_len.set(len);
    }));
    let overrides_now = overrides();
    let current_filter = filter();
    let current_sort = sort();
    let hiding = hide_resolved();
    let shown = visible_notes(&all, current_filter, current_sort, &overrides_now, hiding);
    let (n_all, n_queries, n_patches) = filter_counts(&all);
    let patch_states = patches();
    let rows: Vec<NoteView> = shown
        .iter()
        .map(|note| {
            let state = patch_states.get(&note.note_id);
            NoteView {
                note: note.clone(),
                resolved: is_resolved(note, &overrides_now),
                patch: is_patch(&note.note_text),
                running: matches!(state, Some(PatchState::Running)),
                error: match state {
                    Some(PatchState::Failed(message)) => Some(message.clone()),
                    _ => None,
                },
                relative: relative_label(now_secs, note.posted_at),
            }
        })
        .collect();
    let empty_line = if all.is_empty() {
        "No notes yet"
    } else {
        "No matching notes"
    };
    let tabs: Vec<(NoteFilter, &'static str, usize)> = vec![
        (NoteFilter::All, "All", n_all),
        (NoteFilter::Queries, "Queries", n_queries),
        (NoteFilter::Patches, "Patches", n_patches),
    ];
    let sort_items = [
        (NoteSort::Newest, "Newest first"),
        (NoteSort::Oldest, "Oldest first"),
    ];
    let sort_label = match current_sort {
        NoteSort::Newest => "Newest first",
        NoteSort::Oldest => "Oldest first",
    };
    let checked_index = sort_items
        .iter()
        .position(|(value, _)| *value == current_sort)
        .unwrap_or(0);
    let sort_item_prefix = format!("{base}-sort-item");
    let hide_label = if hiding {
        "Show resolved"
    } else {
        "Hide resolved"
    };
    let draft_text = draft();
    let send_disabled = draft_text.trim().is_empty();
    let sort_id = format!("{base}-sort");
    let menu_id = format!("{base}-menu");
    let root_cls = class_list("kit-annot", &root_class);
    let sort_open_now = sort_open();
    let sort_expanded = if sort_open_now { "true" } else { "false" };
    let hide_pressed = if hiding { "true" } else { "false" };

    let run_patch = move |note: Note| {
        let Some(handler) = on_run_patch else {
            return;
        };
        if matches!(patches.read().get(&note.note_id), Some(PatchState::Running)) {
            return;
        }
        let id = note.note_id.clone();
        patches.write().insert(id.clone(), PatchState::Running);
        announce.set("Applying patch".to_string());
        let outcome = handler.call(note);
        spawn(async move {
            match outcome.await {
                Ok(()) => {
                    patches.write().remove(&id);
                    announce.set("Patch applied".to_string());
                }
                Err(message) => {
                    patches.write().insert(id, PatchState::Failed(message));
                    announce.set("Patch failed".to_string());
                }
            }
        });
    };

    let send = move || {
        let text = draft().trim().to_string();
        if text.is_empty() {
            return;
        }
        let note = Note {
            note_id: format!("local-{}", next_id()),
            writer: acting_writer.clone(),
            posted_at: now_secs,
            note_text: text,
            note_state: NoteState::Open,
        };
        if let Some(handler) = on_note_added {
            handler.call(note.clone());
        }
        added.write().push(note);
        draft.set(String::new());
        announce.set("Note added".to_string());
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: root_cls, role: "group", "aria-labelledby": "{base}-title",
            header { class: "kit-annot__head",
                h2 { id: "{base}-title", class: "kit-annot__title", "{section_title}" }
                if let Some(close) = on_close_request {
                    button {
                        r#type: "button",
                        class: "kit-annot__icon",
                        "aria-label": "Close notes",
                        onclick: move |_| close.call(()),
                        "×"
                    }
                }
            }
            div { class: "kit-annot__bar",
                div { class: "kit-annot__tabs", role: "toolbar", "aria-label": "Filter notes",
                    for (value, label, count) in tabs {
                        {
                            let pressed = if value == current_filter { "true" } else { "false" };
                            let tab_text = format!("{label} {count}");
                            rsx! {
                                button {
                                    key: "{label}",
                                    r#type: "button",
                                    class: "kit-annot__tab",
                                    "aria-pressed": "{pressed}",
                                    "data-filter": value.as_str(),
                                    onclick: move |_| filter.set(value),
                                    "{tab_text}"
                                }
                            }
                        }
                    }
                }
                div { class: "kit-annot__sort",
                    button {
                        id: "{sort_id}",
                        r#type: "button",
                        class: "kit-annot__tab",
                        "aria-haspopup": "menu",
                        "aria-expanded": "{sort_expanded}",
                        "aria-controls": "{menu_id}",
                        onclick: move |_| sort_open.set(!sort_open()),
                        "{sort_label}"
                    }
                    if sort_open_now {
                        div {
                            id: "{menu_id}",
                            class: "kit-annot__menu",
                            role: "menu",
                            onmounted: {
                                let first = format!("{sort_item_prefix}-{checked_index}");
                                move |_: MountedEvent| focus_id(&first)
                            },
                            onkeydown: {
                                let sort_id = sort_id.clone();
                                let menu_id = menu_id.clone();
                                move |event: KeyboardEvent| match event.key() {
                                    Key::Escape => {
                                        event.prevent_default();
                                        sort_open.set(false);
                                        focus_id(&sort_id);
                                    }
                                    Key::ArrowDown => {
                                        event.prevent_default();
                                        step_menu_focus(&menu_id, 1);
                                    }
                                    Key::ArrowUp => {
                                        event.prevent_default();
                                        step_menu_focus(&menu_id, -1);
                                    }
                                    _ => {}
                                }
                            },
                            for (index, (value, label)) in sort_items.into_iter().enumerate() {
                                {
                                    let checked = if value == current_sort { "true" } else { "false" };
                                    let item_id = format!("{sort_item_prefix}-{index}");
                                    let sort_id = sort_id.clone();
                                    rsx! {
                                        button {
                                            key: "{label}",
                                            id: "{item_id}",
                                            r#type: "button",
                                            role: "menuitemradio",
                                            "aria-checked": "{checked}",
                                            class: "kit-annot__menuitem",
                                            onclick: move |_| {
                                                sort.set(value);
                                                sort_open.set(false);
                                                focus_id(&sort_id);
                                            },
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "kit-annot__tab",
                        "aria-pressed": "{hide_pressed}",
                        onclick: move |_| hide_resolved.set(!hide_resolved()),
                        "{hide_label}"
                    }
                }
            }
            if unseen() > 0 {
                button {
                    r#type: "button",
                    class: "kit-annot__resume",
                    onclick: {
                        let list_dom = list_dom.clone();
                        move |_| {
                            jump_to_new_end(&list_dom, toward_top);
                            unseen.set(0);
                            away.set(false);
                        }
                    },
                    "{resume_label}"
                }
            }
            div {
                id: "{list_dom}",
                class: "kit-annot__list",
                onscroll: {
                    let list_dom = list_dom.clone();
                    move |_| {
                        let list_dom = list_dom.clone();
                        spawn(async move {
                            let scrolled = list_scrolled_away(list_dom, toward_top).await;
                            away.set(scrolled);
                            if !scrolled {
                                unseen.set(0);
                            }
                        });
                    }
                },
                if rows.is_empty() {
                    p { class: "kit-annot__empty", "{empty_line}" }
                }
                ol { class: "kit-annot__items",
                    for row in rows {
                        {
                            let id = row.note.note_id.clone();
                            let resolved_attr = if row.resolved { "true" } else { "false" };
                            let item_class = if row.resolved {
                                "kit-annot__item is-resolved"
                            } else {
                                "kit-annot__item"
                            };
                            let apply_label = if row.running { "Applying…" } else { "Apply patch" };
                            let busy = if row.running { "true" } else { "false" };
                            let mut run = run_patch.clone();
                            let note_for_run = row.note.clone();
                            let note_text = row.note.note_text.clone();
                            let writer_name = row.note.writer.writer_name.clone();
                            let mark = row
                                .note
                                .writer
                                .writer_mark
                                .clone()
                                .unwrap_or_else(|| writer_name.chars().take(1).collect());
                            let time_value = row.note.posted_at.to_string();
                            let relative = row.relative.clone();
                            let toggle_id = id.clone();
                            let was_resolved = row.resolved;
                            let error = row.error.clone();
                            rsx! {
                                li {
                                    key: "{id}",
                                    class: item_class,
                                    "data-note-id": "{id}",
                                    header { class: "kit-annot__meta",
                                        span { class: "kit-annot__mark", "aria-hidden": "true", "{mark}" }
                                        span { class: "kit-annot__writer", "{writer_name}" }
                                        time { class: "kit-annot__time", "datetime": "{time_value}", "{relative}" }
                                    }
                                    p { class: "kit-annot__text", "{note_text}" }
                                    div { class: "kit-annot__actions",
                                        button {
                                            r#type: "button",
                                            class: "kit-annot__toggle",
                                            "aria-pressed": "{resolved_attr}",
                                            onclick: move |_| {
                                                overrides.write().insert(toggle_id.clone(), !was_resolved);
                                            },
                                            "Resolved"
                                        }
                                        if row.patch {
                                            button {
                                                r#type: "button",
                                                class: "kit-annot__apply",
                                                disabled: row.running,
                                                "aria-busy": "{busy}",
                                                onclick: move |_| run(note_for_run.clone()),
                                                if row.running {
                                                    span { class: "kit-annot__spinner", "aria-hidden": "true" }
                                                }
                                                "{apply_label}"
                                            }
                                        }
                                    }
                                    if let Some(message) = error {
                                        p { class: "kit-annot__error", role: "alert", "{message}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            form {
                class: "kit-annot__composer",
                onsubmit: {
                    let mut send = send.clone();
                    move |event: FormEvent| {
                        event.prevent_default();
                        send();
                    }
                },
                input {
                    r#type: "text",
                    class: "kit-annot__input",
                    "aria-label": "Write a note",
                    placeholder: "Add a note, or start with /patch",
                    value: "{draft_text}",
                    oninput: move |event: FormEvent| draft.set(event.value()),
                    onkeydown: {
                        let mut send = send.clone();
                        move |event: KeyboardEvent| {
                            if matches!(event.key(), Key::Enter) && !event.modifiers().shift() {
                                event.prevent_default();
                                send();
                            }
                        }
                    },
                }
                button {
                    r#type: "submit",
                    class: "kit-annot__send",
                    disabled: send_disabled,
                    "Send"
                }
            }
            div { class: "kit-annot__sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
