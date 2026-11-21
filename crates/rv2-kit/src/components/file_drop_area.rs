//! File drop area: drop or browse for files, upload each, and track queued, uploading, done,
//! failed and rejected rows with a retry for failures.
//!
//! Design choice: `upload_handler` returns a future and reports 0 to 1 through the `progress`
//! signal in its request. Without a handler the component simulates an upload. `FileItem` omits the
//! browser file handle because it has no `PartialEq`; the handler receives the `FileData`.
//!
//! Browser checks: drag over changes the border and the icon fan; Enter on the zone opens the
//! picker; Retry and Remove are reachable in order; long names truncate.

use std::{future::Future, pin::Pin};

use dioxus::html::{FileData, HasFileData};
use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/file-drop-area.css");
pub const DEFAULT_MAX_BYTES: u64 = 25 * 1024 * 1024;

pub type UploadTask = Pin<Box<dyn Future<Output = Result<(), String>>>>;

/// What an upload handler receives for one file.
#[derive(Clone)]
pub struct UploadRequest {
    pub item_id: String,
    pub file: FileData,
    /// Written by the handler: 0 to 1.
    pub progress: Signal<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemState {
    Queued,
    Uploading,
    Done,
    Failed,
    Rejected,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FileItem {
    pub item_id: String,
    pub file_name: String,
    pub byte_size: u64,
    pub item_state: ItemState,
    pub progress: f64,
    pub error_text: Option<String>,
}

/// `.pdf` for `report.PDF`; empty when there is none.
pub fn extension_of(name: &str) -> String {
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && !ext.is_empty() => {
            format!(".{}", ext.to_lowercase())
        }
        _ => String::new(),
    }
}

/// `.pdf` for both `pdf` and `.PDF`, so accepted lists may be written either way.
pub fn normal_extension(raw: &str) -> String {
    let lower = raw.trim().to_lowercase();
    if lower.starts_with('.') {
        lower
    } else {
        format!(".{lower}")
    }
}

/// The reason a file cannot be uploaded, or `None` when it can.
pub fn rejection(
    name: &str,
    byte_size: u64,
    accepted: &[String],
    max_bytes: u64,
) -> Option<String> {
    let ext = extension_of(name);
    if !accepted.is_empty() && !accepted.iter().any(|raw| normal_extension(raw) == ext) {
        return Some("Unsupported file type".to_string());
    }
    if byte_size > max_bytes {
        return Some(format!("Too large (max {})", format_bytes(max_bytes)));
    }
    None
}

/// `512 B`, `1.5 KB`, `12.0 MB`.
pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    let value = bytes as f64;
    if value < KB {
        format!("{bytes} B")
    } else if value < KB * KB {
        format!("{:.1} KB", value / KB)
    } else {
        format!("{:.1} MB", value / (KB * KB))
    }
}

/// `(uploading, done, failed)` counts for the summary row.
pub fn summary(items: &[FileItem]) -> (usize, usize, usize) {
    items.iter().fold((0, 0, 0), |(up, done, failed), item| {
        match item.item_state {
            ItemState::Uploading => (up + 1, done, failed),
            ItemState::Done => (up, done + 1, failed),
            ItemState::Failed => (up, done, failed + 1),
            _ => (up, done, failed),
        }
    })
}

/// The tint group of a file-type badge.
pub fn file_group(extension: &str) -> &'static str {
    match extension {
        ".png" | ".jpg" | ".jpeg" | ".gif" | ".svg" | ".webp" => "image",
        ".pdf" | ".doc" | ".docx" | ".txt" | ".md" => "document",
        ".csv" | ".xls" | ".xlsx" | ".json" => "data",
        _ => "other",
    }
}

pub fn progress_percent(progress: f64) -> u32 {
    (progress.clamp(0.0, 1.0) * 100.0).round() as u32
}

fn status_text(item: &FileItem) -> String {
    match item.item_state {
        ItemState::Queued => "Starting".to_string(),
        ItemState::Uploading => format!(
            "{} of {}",
            format_bytes((item.byte_size as f64 * item.progress) as u64),
            format_bytes(item.byte_size)
        ),
        ItemState::Done => "Uploaded".to_string(),
        ItemState::Failed | ItemState::Rejected => item.error_text.clone().unwrap_or_default(),
    }
}

#[derive(Clone)]
struct Entry {
    item: FileItem,
    file: Option<FileData>,
    progress: Signal<f64>,
}

#[component]
pub fn FileDropArea(
    #[props(default)] accepted_extensions: Vec<String>,
    #[props(default = DEFAULT_MAX_BYTES)] max_bytes: u64,
    #[props(default)] upload_handler: Option<Callback<UploadRequest, UploadTask>>,
    #[props(default)] on_list_change: EventHandler<Vec<FileItem>>,
    #[props(default, into)] hint_text: Option<String>,
    #[props(default)] demo_run: bool,
    #[props(default, into)] extra_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-fda-{}", next_id()));
    let picker_id = format!("{id}-picker");
    let mut entries = use_signal(Vec::<Entry>::new);
    let mut dragging = use_signal(|| false);

    let items: Vec<FileItem> = entries()
        .iter()
        .map(|entry| {
            let mut item = entry.item.clone();
            match item.item_state {
                ItemState::Uploading => item.progress = (entry.progress)(),
                ItemState::Done => item.progress = 1.0,
                _ => {}
            }
            item
        })
        .collect();
    let (uploading, done, failed) = summary(&items);
    let hint = hint_text.clone().unwrap_or_else(|| {
        let formats = if accepted_extensions.is_empty() {
            "Any file type".to_string()
        } else {
            accepted_extensions.join(", ")
        };
        format!("{formats} · up to {}", format_bytes(max_bytes))
    });

    let accepted_for_drop = accepted_extensions.clone();
    let handler_for_drop = upload_handler;
    let id_for_files = id.clone();
    let add_files = use_callback(move |files: Vec<FileData>| {
        let mut next_entries = entries();
        for file in files {
            let name = file.name();
            let size = file.size();
            let item_id = format!("{id_for_files}-{}", next_id());
            let progress = Signal::new(0.0_f64);
            let (state, error_text, keep) =
                match rejection(&name, size, &accepted_for_drop, max_bytes) {
                    Some(reason) => (ItemState::Rejected, Some(reason), None),
                    None => (ItemState::Queued, None, Some(file.clone())),
                };
            let item = FileItem {
                item_id: item_id.clone(),
                file_name: name,
                byte_size: size,
                item_state: state,
                progress: 0.0,
                error_text,
            };
            next_entries.push(Entry {
                item,
                file: keep.clone(),
                progress,
            });
            if let (Some(file), Some(handler)) = (keep, handler_for_drop) {
                let progress_signal = progress;
                spawn(async move {
                    let request = UploadRequest {
                        item_id: item_id.clone(),
                        file,
                        progress: progress_signal,
                    };
                    let outcome = handler.call(request).await;
                    finish(entries, &item_id, outcome);
                });
            }
        }
        entries.set(next_entries);
    });

    let mut demo_started = use_signal(|| false);
    let id_demo = id.clone();
    use_effect(move || {
        if !demo_run || *demo_started.peek() {
            return;
        }
        demo_started.set(true);
        let demo_id = id_demo.clone();
        spawn(async move {
            let samples = [
                ("brief.pdf", 240_000_u64),
                ("logo.png", 88_000),
                ("budget.xlsx", 1_200_000),
                ("notes.md", 4_000),
            ];
            for (index, (name, size)) in samples.into_iter().enumerate() {
                let mut progress = Signal::new(0.0_f64);
                let item_id = format!("{demo_id}-demo-{index}");
                let item = FileItem {
                    item_id: item_id.clone(),
                    file_name: name.to_string(),
                    byte_size: size,
                    item_state: ItemState::Uploading,
                    progress: 0.0,
                    error_text: None,
                };
                entries.with_mut(|list| {
                    list.push(Entry {
                        item,
                        file: None,
                        progress,
                    })
                });
                for step in 1..=10 {
                    sleep(120).await;
                    progress.set(step as f64 / 10.0);
                }
                set_state(entries, &item_id, ItemState::Done, None);
            }
        });
    });

    use_effect(move || {
        let snapshot: Vec<FileItem> = entries()
            .iter()
            .map(|entry| {
                let mut item = entry.item.clone();
                item.progress = (entry.progress)();
                item
            })
            .collect();
        on_list_change.call(snapshot);
    });

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: class_list("kit-fda", &extra_class), "data-dragging": dragging().then_some("true"),
            label {
                class: "kit-fda__zone",
                role: "button",
                tabindex: "0",
                r#for: "{picker_id}",
                "aria-describedby": "{id}-hint",
                onkeydown: {
                    let picker = picker_id.clone();
                    move |event: KeyboardEvent| {
                        if matches!(event.key(), Key::Enter) || event.key() == Key::Character(" ".into()) {
                            event.prevent_default();
                            // Deferred: a synchronous click() re-enters Dioxus event dispatch and panics.
                            document::eval(&format!("setTimeout(() => document.getElementById({picker:?})?.click(), 0);"));
                        }
                    }
                },
                ondragover: move |event| {
                    event.prevent_default();
                    dragging.set(true);
                },
                ondragleave: move |_| dragging.set(false),
                ondrop: move |event| {
                    event.prevent_default();
                    dragging.set(false);
                    add_files.call(event.data().files());
                },
                span { class: "kit-fda__title", "Drop files here or browse" }
                span { id: "{id}-hint", class: "kit-fda__hint", "{hint}" }
                input {
                    id: "{picker_id}",
                    class: "kit-fda__picker",
                    r#type: "file",
                    multiple: true,
                    tabindex: "-1",
                    "aria-hidden": "true",
                    onchange: move |event| add_files.call(event.files()),
                }
            }
            if !items.is_empty() {
                div { class: "kit-fda__list", role: "list",
                    for item in items.iter() {
                        {
                            let percent = progress_percent(item.progress);
                            let ext = extension_of(&item.file_name);
                            let group = file_group(&ext);
                            let status = status_text(item);
                            let retry_id = item.item_id.clone();
                            let remove_id = item.item_id.clone();
                            let remove_name = item.file_name.clone();
                            let retry_name = item.file_name.clone();
                            let show_retry = item.item_state == ItemState::Failed;
                            let state_name = format!("{:?}", item.item_state).to_lowercase();
                            rsx! {
                                div { key: "{item.item_id}", class: "kit-fda__row", role: "listitem", "data-state": "{state_name}",
                                    span { class: "kit-fda__badge", "data-group": group, "{ext}" }
                                    span { class: "kit-fda__meta",
                                        span { class: "kit-fda__name", "{item.file_name}" }
                                        span { class: "kit-fda__size", "{format_bytes(item.byte_size)}" }
                                        span { class: "kit-fda__bar", role: "progressbar", "aria-label": "{retry_name}", "aria-valuemin": "0", "aria-valuemax": "100", "aria-valuenow": "{percent}",
                                            span { class: "kit-fda__fill", style: "width: {percent}%" }
                                        }
                                        span { class: "kit-fda__status", "{status}" }
                                    }
                                    span { class: "kit-fda__actions",
                                        if show_retry {
                                            button {
                                                r#type: "button",
                                                class: "kit-fda__action",
                                                "aria-label": "Retry {retry_name}",
                                                onclick: move |_| {
                                                    let retry = entries().into_iter().find(|e| e.item.item_id == retry_id);
                                                    if let (Some(entry), Some(handler)) = (retry, upload_handler) {
                                                        if let Some(file) = entry.file.clone() {
                                                            set_state(entries, &entry.item.item_id, ItemState::Uploading, None);
                                                            let progress_signal = entry.progress;
                                                            let item_id = entry.item.item_id.clone();
                                                            spawn(async move {
                                                                let request = UploadRequest { item_id: item_id.clone(), file, progress: progress_signal };
                                                                let outcome = handler.call(request).await;
                                                                finish(entries, &item_id, outcome);
                                                            });
                                                        }
                                                    }
                                                },
                                                "Retry"
                                            }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "kit-fda__action",
                                            "aria-label": "Remove {remove_name}",
                                            onclick: move |_| {
                                                let next: Vec<Entry> = entries().into_iter().filter(|e| e.item.item_id != remove_id).collect();
                                                entries.set(next);
                                            },
                                            "Remove"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "kit-fda__summary", role: "status", "aria-live": "polite",
                span { "{uploading} uploading" }
                span { "{done} done" }
                span { "{failed} failed" }
            }
        }
    }
}

fn set_state(
    mut entries: Signal<Vec<Entry>>,
    item_id: &str,
    state: ItemState,
    error: Option<String>,
) {
    entries.with_mut(|list| {
        for entry in list.iter_mut().filter(|e| e.item.item_id == item_id) {
            entry.item.item_state = state;
            entry.item.error_text = error.clone();
        }
    });
}

fn finish(entries: Signal<Vec<Entry>>, item_id: &str, outcome: Result<(), String>) {
    match outcome {
        Ok(()) => set_state(entries, item_id, ItemState::Done, None),
        Err(message) => set_state(entries, item_id, ItemState::Failed, Some(message)),
    }
}
