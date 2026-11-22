//! Folder detail: one folder of a shared file area, its files as cards in a multi-select listbox, drop-to-upload,
//! and a breadcrumb trail.
//!
//! Design choice: `on_upload` receives file names. Web file handles are not carried across the crate boundary
//! (the kit has no web-sys dependency); the host reads the bytes from the name it was handed.
//! Design choice: the upload is a local progress animation that stands in for the host transfer; the host decides
//! the final state through `files`.
//! Design choice: Up and Down move one rendered row; the column count is read from the grid's computed tracks.
//! Design choice: the sidebar is drawn inline so the screen stands alone, without the workspace frame.

use std::collections::BTreeSet;

use dioxus::html::HasFileData;
use dioxus::prelude::*;

use crate::internal::*;

const CSS: Asset = asset!("/assets/folder-detail.css");

#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub name: String,
    pub avatar_url: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FileKind {
    Doc,
    Sheet,
    Pdf,
    Image,
    #[default]
    Other,
}

impl FileKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Doc => "Document",
            Self::Sheet => "Spreadsheet",
            Self::Pdf => "PDF",
            Self::Image => "Image",
            Self::Other => "File",
        }
    }

    pub fn glyph(self) -> &'static str {
        match self {
            Self::Doc => "DOC",
            Self::Sheet => "XLS",
            Self::Pdf => "PDF",
            Self::Image => "IMG",
            Self::Other => "FILE",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShareState {
    #[default]
    Private,
    Shared,
}

impl ShareState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Private => "Private",
            Self::Shared => "Shared",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FileItem {
    pub id: String,
    pub name: String,
    pub kind: FileKind,
    pub updated_at: String,
    pub subtitle: String,
    pub preview_url: Option<String>,
}

/// Resolves a shift-click or keyboard range between two indices, inclusive, whichever end is first.
pub fn range_ids(files: &[FileItem], from: usize, to: usize) -> Vec<String> {
    let (lo, hi) = if from <= to { (from, to) } else { (to, from) };
    files
        .iter()
        .enumerate()
        .filter(|(i, _)| *i >= lo && *i <= hi)
        .map(|(_, f)| f.id.clone())
        .collect()
}

/// Index of the first file whose name starts with `prefix`, ignoring case, searched from `start`.
pub fn type_ahead_index(files: &[FileItem], prefix: &str, start: usize) -> Option<usize> {
    let needle = prefix.to_lowercase();
    if needle.is_empty() || files.is_empty() {
        return None;
    }
    (0..files.len())
        .map(|offset| (start + offset) % files.len())
        .find(|i| files[*i].name.to_lowercase().starts_with(&needle))
}

/// Path of the breadcrumb at `index`, from the root down to it.
pub fn crumb_path(crumbs: &[String], index: usize) -> String {
    crumbs
        .iter()
        .take(index + 1)
        .cloned()
        .collect::<Vec<_>>()
        .join(" / ")
}

/// Upload progress for one file the person dropped, as a whole percentage.
#[derive(Clone, Debug, PartialEq)]
struct Upload {
    id: String,
    name: String,
    progress: u8,
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

/// Moves focus one row up or down from card `current`. The row is the grid's rendered column count, read from its
/// computed track list, so it follows the width. Sends the new card index back, clamped to `0..=last`.
fn row_step_script(base: &str, current: usize, last: usize, down: bool) -> String {
    format!(
        r#"(() => {{
const base = {base:?};
const card = document.getElementById(`${{base}}-card-{current}`);
const list = card ? card.closest('[role="listbox"]') : null;
const tracks = list ? getComputedStyle(list).gridTemplateColumns.split(' ').filter(Boolean).length : 1;
const next = Math.min({last}, Math.max(0, {current} + ({down} ? tracks : -tracks)));
document.getElementById(`${{base}}-card-${{next}}`)?.focus({{ preventScroll: true }});
dioxus.send(next);
}})();"#
    )
}

/// Folder page: breadcrumb, title row with actions, a drop target and the file grid.
#[component]
pub fn FolderDetail(
    #[props(default = "Q3 Campaign".to_string(), into)] folder_name: String,
    #[props(default)] crumbs: Vec<String>,
    #[props(default = "Juniper Works".to_string(), into)] workspace_name: String,
    account: Account,
    #[props(default)] files: Vec<FileItem>,
    #[props(default)] share_state: ShareState,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_upload: EventHandler<Vec<String>>,
    #[props(default)] on_share: EventHandler<()>,
    #[props(default)] on_open_file: EventHandler<String>,
    #[props(default)] on_remove: EventHandler<Vec<String>>,
    #[props(default)] on_start_chat: EventHandler<()>,
    #[props(default)] on_navigate: EventHandler<String>,
) -> Element {
    let base = use_hook(|| format!("kit-fd-{}", next_id()));
    let mut selected = use_signal(BTreeSet::<String>::new);
    let mut anchor = use_signal(|| 0_usize);
    let mut active = use_signal(|| 0_usize);
    let mut drop_active = use_signal(|| false);
    let mut uploads = use_signal(Vec::<Upload>::new);
    let mut typed = use_signal(String::new);
    let mut confirm_remove = use_signal(|| false);
    let mut announce = use_signal(String::new);
    let mut upload_seq = use_signal(|| 0_usize);

    let chosen = selected();
    let chosen_count = chosen.len();
    let upload_now = uploads();
    let file_count = files.len();
    let card_id = |i: usize| format!("{base}-card-{i}");
    let crumb_count = crumbs.len();
    let mid_crumbs: Vec<(usize, String)> = crumbs
        .iter()
        .enumerate()
        .skip(1)
        .take(crumb_count.saturating_sub(2))
        .map(|(i, c)| (i, c.clone()))
        .collect();
    let last_crumb = crumbs.len().saturating_sub(1);
    let share_label = share_state.label();
    let files_for_keys = files.clone();

    let base_for_keys = base.clone();
    let on_grid_keys = {
        let files = files_for_keys.clone();
        move |event: KeyboardEvent| {
            let card_id = |i: usize| format!("{base_for_keys}-card-{i}");
            let last = files.len().saturating_sub(1);
            let current = active();
            let mods = event.modifiers();
            let ctrl = mods.ctrl() || mods.meta();
            let key = event.key();
            match key {
                Key::ArrowRight => {
                    event.prevent_default();
                    let next = (current + 1).min(last);
                    active.set(next);
                    focus_after_frame(&card_id(next));
                }
                Key::ArrowLeft => {
                    event.prevent_default();
                    let next = current.saturating_sub(1);
                    active.set(next);
                    focus_after_frame(&card_id(next));
                }
                Key::ArrowDown | Key::ArrowUp => {
                    event.prevent_default();
                    let script = row_step_script(
                        &base_for_keys,
                        current,
                        last,
                        matches!(key, Key::ArrowDown),
                    );
                    spawn(async move {
                        let mut moved = document::eval(&script);
                        if let Ok(next) = moved.recv::<usize>().await {
                            active.set(next);
                        }
                    });
                }
                Key::Home => {
                    event.prevent_default();
                    active.set(0);
                    focus_after_frame(&card_id(0));
                }
                Key::End => {
                    event.prevent_default();
                    active.set(last);
                    focus_after_frame(&card_id(last));
                }
                Key::Escape => {
                    selected.set(BTreeSet::new());
                    announce.set("Selection cleared".to_string());
                }
                Key::Delete | Key::Backspace => {
                    if !selected().is_empty() {
                        event.prevent_default();
                        confirm_remove.set(true);
                    }
                }
                Key::Enter => {
                    if let Some(file) = files.get(current) {
                        on_open_file.call(file.id.clone());
                    }
                }
                Key::Character(c) if c == " " => {
                    event.prevent_default();
                    if let Some(file) = files.get(current) {
                        let id = file.id.clone();
                        selected.with_mut(|s| {
                            if !s.remove(&id) {
                                s.insert(id);
                            }
                        });
                        anchor.set(current);
                        announce.set(format!("{} selected", selected().len()));
                    }
                }
                Key::Character(c) if ctrl && (c == "a" || c == "A") => {
                    event.prevent_default();
                    selected.set(files.iter().map(|f| f.id.clone()).collect());
                    announce.set(format!("{} selected", files.len()));
                }
                Key::Character(c) if !ctrl && c.chars().count() == 1 => {
                    let mut buffer = typed();
                    buffer.push_str(&c);
                    let found = type_ahead_index(&files, &buffer, current);
                    typed.set(buffer);
                    if let Some(i) = found {
                        active.set(i);
                        focus_after_frame(&card_id(i));
                    }
                }
                _ => {
                    typed.set(String::new());
                }
            }
        }
    };

    let start_uploads = move |names: Vec<String>| {
        if names.is_empty() {
            return;
        }
        on_upload.call(names.clone());
        let mut seq = upload_seq();
        let mut batch = Vec::new();
        for name in names {
            seq += 1;
            batch.push(Upload {
                id: format!("up-{seq}"),
                name: name.clone(),
                progress: 0,
            });
        }
        upload_seq.set(seq);
        let started: Vec<String> = batch.iter().map(|u| u.id.clone()).collect();
        uploads.with_mut(|list| {
            let mut next = batch.clone();
            next.append(list);
            *list = next;
        });
        announce.set(format!("Uploading {} files", started.len()));
        for id in started {
            spawn(async move {
                for step in 1..=10_u8 {
                    sleep(120).await;
                    let value = step * 10;
                    uploads.with_mut(|list| {
                        if let Some(u) = list.iter_mut().find(|u| u.id == id) {
                            u.progress = value;
                        }
                    });
                }
                announce.set("Upload complete".to_string());
            });
        }
    };

    let on_drop = move |event: DragEvent| {
        event.prevent_default();
        drop_active.set(false);
        let names: Vec<String> = event.files().iter().map(|f| f.name()).collect();
        let mut start = start_uploads;
        start(names);
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "kit-fd", "aria-labelledby": "{base}-title",
            nav { class: "kit-fd__nav", "aria-label": "Sidebar",
                p { class: "kit-fd__ws", "{workspace_name}" }
                ul { class: "kit-fd__nav-list",
                    for item in ["Home", "Files", "Inbox", "Settings"] {
                        li { key: "{item}",
                            button { class: "kit-fd__nav-item", r#type: "button",
                                "aria-current": if item == "Files" { "page" } else { "false" },
                                onclick: move |_| on_navigate.call(item.to_string()),
                                "{item}"
                            }
                        }
                    }
                }
                div { class: "kit-fd__account",
                    span { class: "kit-fd__avatar", "aria-hidden": "true",
                        if let Some(src) = account.avatar_url.clone() {
                            img { src: "{src}", alt: "" }
                        } else {
                            "{initials(&account.name)}"
                        }
                    }
                    span { class: "kit-fd__account-name", "{account.name}" }
                }
            }

            div { class: "kit-fd__page",
                nav { class: "kit-fd__crumbs", "aria-label": "Breadcrumb",
                    ol {
                        for (i, crumb) in crumbs.iter().enumerate() {
                            if i == last_crumb {
                                li { key: "{i}", "aria-current": "page", "{crumb}" }
                            } else {
                                li { key: "{i}", class: if i == 0 { "" } else { "kit-fd__mid" },
                                    button { class: "kit-fd__crumb", r#type: "button",
                                        onclick: {
                                            let path = crumb_path(&crumbs, i);
                                            move |_| on_navigate.call(path.clone())
                                        },
                                        "{crumb}"
                                    }
                                }
                            }
                        }
                    }
                    if !mid_crumbs.is_empty() {
                        details { class: "kit-fd__crumb-more",
                            summary { "…" }
                            ul { class: "kit-fd__crumb-menu",
                                for (i, crumb) in mid_crumbs.iter() {
                                    li { key: "{i}",
                                        button { class: "kit-fd__crumb", r#type: "button",
                                            onclick: {
                                                let path = crumb_path(&crumbs, *i);
                                                move |_| on_navigate.call(path.clone())
                                            },
                                            "{crumb}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                header { class: "kit-fd__titlerow",
                    div { class: "kit-fd__titlebox",
                        h1 { id: "{base}-title", class: "kit-fd__title", "{folder_name}" }
                        span { class: "kit-fd__share", "data-state": if share_state == ShareState::Shared { "shared" } else { "private" }, "{share_label}" }
                    }
                    div { class: "kit-fd__actions",
                        button { class: "kit-fd__btn", r#type: "button", onclick: move |_| on_share.call(()), "Share" }
                        label { class: "kit-fd__btn kit-fd__btn--primary", "aria-label": "Upload files",
                            span { "Upload" }
                            input { class: "kit-fd__file", r#type: "file", multiple: true,
                                onchange: move |event: FormEvent| {
                                    let names: Vec<String> = event.files().iter().map(|f| f.name()).collect();
                                    let mut start = start_uploads;
                                    start(names);
                                },
                            }
                        }
                    }
                }

                div { class: "kit-fd__body",
                    ondragover: move |event: DragEvent| {
                        event.prevent_default();
                        drop_active.set(true);
                    },
                    ondragleave: move |_| drop_active.set(false),
                    ondrop: on_drop,
                    "data-drop": if drop_active() { "active" } else { "idle" },

                    if drop_active() {
                        div { class: "kit-fd__droparea", role: "status", "Drop files to upload" }
                    }

                    if is_loading {
                        div { class: "kit-fd__grid", "aria-busy": "true",
                            for i in 0..4 {
                                span { key: "{i}", class: "kit-fd__ph" }
                            }
                        }
                    } else if is_empty && upload_now.is_empty() {
                        div { class: "kit-fd__empty",
                            p { class: "kit-fd__empty-title", "No files yet" }
                            p { class: "kit-fd__empty-text", "Drop files here, or start a chat about this folder." }
                            button { class: "kit-fd__btn kit-fd__btn--primary", r#type: "button", onclick: move |_| on_start_chat.call(()), "Start a chat about this folder" }
                        }
                    } else {
                        ul { class: "kit-fd__grid", role: "listbox", "aria-label": "Files", "aria-multiselectable": "true",
                            "aria-activedescendant": "{card_id(active())}",
                            onkeydown: on_grid_keys,
                            for u in upload_now.iter() {
                                li { key: "{u.id}", class: "kit-fd__item kit-fd__item--uploading", role: "option", "aria-selected": "false",
                                    span { class: "kit-fd__thumb", "aria-hidden": "true", "UP" }
                                    span { class: "kit-fd__name", "{u.name}" }
                                    span { class: "kit-fd__progress", role: "progressbar", "aria-valuemin": "0", "aria-valuemax": "100", "aria-valuenow": "{u.progress}", "aria-label": "Uploading {u.name}",
                                        span { class: "kit-fd__progress-fill", style: "width: {u.progress}%;" }
                                    }
                                }
                            }
                            for (i, file) in files.iter().enumerate() {
                                li { key: "{file.id}", id: "{card_id(i)}", class: "kit-fd__item", role: "option",
                                    "aria-selected": if chosen.contains(&file.id) { "true" } else { "false" },
                                    tabindex: if i == active() { "0" } else { "-1" },
                                    "aria-label": "{file.name}, {file.kind.label()}",
                                    onfocus: move |_| active.set(i),
                                    onclick: {
                                        let id = file.id.clone();
                                        let files_range = files_for_keys.clone();
                                        move |event: MouseEvent| {
                                            let mods = event.modifiers();
                                            active.set(i);
                                            if mods.shift() {
                                                let ids = range_ids(&files_range, anchor(), i);
                                                selected.set(ids.into_iter().collect());
                                            } else if mods.ctrl() || mods.meta() {
                                                selected.with_mut(|s| {
                                                    if !s.remove(&id) {
                                                        s.insert(id.clone());
                                                    }
                                                });
                                                anchor.set(i);
                                            } else {
                                                selected.set(BTreeSet::from([id.clone()]));
                                                anchor.set(i);
                                            }
                                            announce.set(format!("{} selected", selected().len()));
                                        }
                                    },
                                    ondoubleclick: {
                                        let id = file.id.clone();
                                        move |_| on_open_file.call(id.clone())
                                    },
                                    span { class: "kit-fd__thumb", "aria-hidden": "true",
                                        if let Some(src) = file.preview_url.clone() {
                                            img { src: "{src}", alt: "" }
                                        } else {
                                            "{file.kind.glyph()}"
                                        }
                                    }
                                    span { class: "kit-fd__name", "{file.name}" }
                                    span { class: "kit-fd__kind", "{file.kind.label()}" }
                                    span { class: "kit-fd__sub", "{file.subtitle}" }
                                    span { class: "kit-fd__quick", "aria-hidden": "true", "{file.updated_at}" }
                                }
                            }
                        }
                    }

                    if chosen_count > 0 {
                        div { class: "kit-fd__selbar",
                            span { "{chosen_count} of {file_count} selected" }
                            if confirm_remove() {
                                span { class: "kit-fd__confirm", "Remove {chosen_count} files?" }
                                button { class: "kit-fd__btn kit-fd__btn--danger", r#type: "button",
                                    onclick: move |_| {
                                        let ids: Vec<String> = selected().into_iter().collect();
                                        on_remove.call(ids);
                                        selected.set(BTreeSet::new());
                                        confirm_remove.set(false);
                                    },
                                    "Remove"
                                }
                                button { class: "kit-fd__btn", r#type: "button", onclick: move |_| confirm_remove.set(false), "Keep" }
                            } else {
                                button { class: "kit-fd__btn", r#type: "button", onclick: move |_| confirm_remove.set(true), "Remove selected" }
                            }
                        }
                    }
                }
            }
            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
