//! File drop area: drop or browse for files, upload each, and track queued, uploading, done,
//! failed and rejected rows with a try-again for failures.
//!
//! Design choice: `upload_handler` returns a future and reports 0 to 1 through the `progress`
//! signal in its request. Without a handler the component simulates an upload. `FileItem` omits the
//! browser file handle because it has no `PartialEq`; the handler receives the `FileData`.
//!
//! Browser checks: drag over changes the border; Enter on the zone opens the picker; Try again and
//! Discard are reachable in order; long names truncate.

use std::{future::Future, pin::Pin};

use dioxus::html::{FileData, HasFileData};
use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id, sleep, use_now_ms};

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
    /// Time left on an upload in progress. The component computes it for uploads it runs; a host
    /// that seeds rows with `initial_items` supplies it.
    pub seconds_left: Option<u32>,
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
        return Some("Type not permitted".to_string());
    }
    if byte_size > max_bytes {
        return Some(format!("Too large (max {})", format_bytes(max_bytes)));
    }
    None
}

/// `512 B`, `1.5 KB`, `12 MB`.
pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    let value = bytes as f64;
    if value < KB {
        format!("{bytes} B")
    } else if value < KB * KB {
        format!("{} KB", one_decimal(value / KB))
    } else {
        format!("{} MB", one_decimal(value / (KB * KB)))
    }
}

fn one_decimal(value: f64) -> String {
    let text = format!("{value:.1}");
    match text.strip_suffix(".0") {
        Some(whole) => whole.to_string(),
        None => text,
    }
}

/// `(in progress, stored, refused, failed)` counts for the summary row. In progress covers queued
/// and uploading rows; refused is a rejected row; failed is an upload that errored.
pub fn summary(items: &[FileItem]) -> (usize, usize, usize, usize) {
    items.iter().fold(
        (0, 0, 0, 0),
        |(live, stored, refused, failed), item| match item.item_state {
            ItemState::Queued | ItemState::Uploading => (live + 1, stored, refused, failed),
            ItemState::Done => (live, stored + 1, refused, failed),
            ItemState::Rejected => (live, stored, refused + 1, failed),
            ItemState::Failed => (live, stored, refused, failed + 1),
        },
    )
}

/// `1 in progress · 1 stored · 1 refused`, with a failed count appended only when one exists.
pub fn summary_text(counts: (usize, usize, usize, usize)) -> String {
    let (live, stored, refused, failed) = counts;
    let mut text = format!("{live} in progress · {stored} stored · {refused} refused");
    if failed > 0 {
        text.push_str(&format!(" · {failed} failed"));
    }
    text
}

/// Seconds left for an upload that has run `elapsed_ms` and reached `progress`, or `None` before
/// any progress has been reported.
pub fn seconds_left_from(elapsed_ms: f64, progress: f64) -> Option<u32> {
    if elapsed_ms <= 0.0 || progress <= 0.0 || progress >= 1.0 {
        return None;
    }
    let remaining_ms = elapsed_ms * (1.0 - progress) / progress;
    Some((remaining_ms / 1000.0).ceil().max(1.0) as u32)
}

/// `about 6 seconds left`, or whole minutes from 60 seconds up.
pub fn time_left_text(seconds: u32) -> String {
    if seconds < 60 {
        let unit = if seconds == 1 { "second" } else { "seconds" };
        format!("about {seconds} {unit} left")
    } else {
        let minutes = seconds.div_ceil(60);
        let unit = if minutes == 1 { "minute" } else { "minutes" };
        format!("about {minutes} {unit} left")
    }
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
        ItemState::Uploading => {
            let transferred = format_bytes((item.byte_size as f64 * item.progress) as u64);
            let total = format_bytes(item.byte_size);
            match item.seconds_left {
                Some(seconds) => format!("{transferred} of {total} · {}", time_left_text(seconds)),
                None => format!("{transferred} of {total}"),
            }
        }
        ItemState::Done => "Stored".to_string(),
        ItemState::Failed | ItemState::Rejected => item.error_text.clone().unwrap_or_default(),
    }
}

#[derive(Clone)]
struct Entry {
    item: FileItem,
    file: Option<FileData>,
    progress: Signal<f64>,
    /// Epoch milliseconds when the handler was entered; `None` until the clock has been read.
    started_ms: Option<f64>,
    task: Option<dioxus::core::Task>,
}

fn seed_entries(items: &[FileItem]) -> Vec<Entry> {
    items
        .iter()
        .map(|item| Entry {
            item: item.clone(),
            file: None,
            progress: Signal::new(item.progress),
            started_ms: None,
            task: None,
        })
        .collect()
}

#[component]
pub fn FileDropArea(
    #[props(default)] accepted_extensions: Vec<String>,
    #[props(default = DEFAULT_MAX_BYTES)] max_bytes: u64,
    #[props(default)] upload_handler: Option<Callback<UploadRequest, UploadTask>>,
    #[props(default)] on_list_change: EventHandler<Vec<FileItem>>,
    #[props(default)] title_text: Option<String>,
    #[props(default, into)] hint_text: Option<String>,
    /// Rows present on mount, for a fixed list such as a gallery demo. They are not uploaded.
    #[props(default)]
    initial_items: Vec<FileItem>,
    #[props(default)] demo_run: bool,
    #[props(default, into)] extra_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-fda-{}", next_id()));
    let picker_id = format!("{id}-picker");
    let mut entries = use_signal(|| seed_entries(&initial_items));
    let mut dragging = use_signal(|| false);
    let mut notice = use_signal(String::new);
    let now_ms = use_now_ms(None, 500) as f64;

    let items: Vec<FileItem> = entries()
        .iter()
        .map(|entry| {
            let mut item = entry.item.clone();
            match item.item_state {
                ItemState::Uploading => {
                    item.progress = (entry.progress)();
                    if let Some(started) = entry.started_ms {
                        if let Some(seconds) = seconds_left_from(now_ms - started, item.progress) {
                            item.seconds_left = Some(seconds);
                        }
                    }
                }
                ItemState::Done => item.progress = 1.0,
                _ => {}
            }
            item
        })
        .collect();
    let counts = summary(&items);
    let summary_line = summary_text(counts);
    let title = title_text
        .clone()
        .unwrap_or_else(|| "Drop files here or browse".to_string());
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
                    Some(reason) => {
                        notice.set(format!("{name}: {reason}"));
                        (ItemState::Rejected, Some(reason), None)
                    }
                    None => (ItemState::Queued, None, Some(file.clone())),
                };
            let item = FileItem {
                item_id: item_id.clone(),
                file_name: name,
                byte_size: size,
                item_state: state,
                progress: 0.0,
                error_text,
                seconds_left: None,
            };
            let mut entry = Entry {
                item,
                file: keep.clone(),
                progress,
                started_ms: None,
                task: None,
            };
            if let Some(file) = keep {
                entry.task = Some(match handler_for_drop {
                    Some(handler) => spawn_upload(entries, handler, item_id, file, progress),
                    None => simulate_upload(entries, item_id, progress),
                });
            }
            next_entries.push(entry);
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
                    seconds_left: None,
                };
                entries.with_mut(|list| {
                    list.push(Entry {
                        item,
                        file: None,
                        progress,
                        started_ms: None,
                        task: None,
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
                svg { class: "kit-fda__symbol", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.75", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                    path { d: "M12 15V4" }
                    path { d: "M7.5 8.5 12 4l4.5 4.5" }
                    path { d: "M4 15v3.5A1.5 1.5 0 0 0 5.5 20h13a1.5 1.5 0 0 0 1.5-1.5V15" }
                }
                span { class: "kit-fda__title", "{title}" }
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
            div { class: "kit-fda__list",
                if !items.is_empty() {
                    div { class: "kit-fda__rows", role: "list",
                        for item in items.iter() {
                            {
                                let percent = progress_percent(item.progress);
                                let ext = extension_of(&item.file_name);
                                let group = file_group(&ext);
                                let status = status_text(item);
                                let try_id = item.item_id.clone();
                                let discard_id = item.item_id.clone();
                                let file_name = item.file_name.clone();
                                let show_try = item.item_state == ItemState::Failed;
                                let show_bar = item.item_state != ItemState::Rejected;
                                let show_mark = item.item_state == ItemState::Done;
                                let state_name = format!("{:?}", item.item_state).to_lowercase();
                                rsx! {
                                    div { key: "{item.item_id}", class: "kit-fda__row", role: "listitem", "data-state": "{state_name}",
                                        span { class: "kit-fda__badge", "data-group": group, "{ext}" }
                                        span { class: "kit-fda__meta",
                                            span { class: "kit-fda__name",
                                                span { class: "kit-fda__file", "{item.file_name}" }
                                                if show_mark {
                                                    svg { class: "kit-fda__mark", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                                                        path { d: "M3.5 8.5 6.5 11.5 12.5 4.5" }
                                                    }
                                                }
                                            }
                                            span { class: "kit-fda__size", "{format_bytes(item.byte_size)}" }
                                            if show_bar {
                                                span { class: "kit-fda__bar", role: "progressbar", "aria-label": "{file_name}", "aria-valuemin": "0", "aria-valuemax": "100", "aria-valuenow": "{percent}",
                                                    span { class: "kit-fda__fill", style: "width: {percent}%" }
                                                }
                                            }
                                            span { class: "kit-fda__status", "{status}" }
                                        }
                                        span { class: "kit-fda__actions",
                                            if show_try {
                                                button {
                                                    r#type: "button",
                                                    class: "kit-fda__action",
                                                    "aria-label": "Try again {file_name}",
                                                    onclick: move |_| {
                                                        let retry = entries().into_iter().find(|e| e.item.item_id == try_id);
                                                        if let (Some(entry), Some(handler)) = (retry, upload_handler) {
                                                            if let Some(file) = entry.file.clone() {
                                                                set_state(entries, &entry.item.item_id, ItemState::Uploading, None);
                                                                let mut progress_signal = entry.progress;
                                                                progress_signal.set(0.0);
                                                                let task = spawn_upload(entries, handler, entry.item.item_id.clone(), file, progress_signal);
                                                                set_started(entries, &entry.item.item_id, None);
                                                                entries.with_mut(|list| {
                                                                    for e in list.iter_mut().filter(|e| e.item.item_id == entry.item.item_id) {
                                                                        e.task = Some(task);
                                                                    }
                                                                });
                                                            }
                                                        }
                                                    },
                                                    "Try again"
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "kit-fda__action",
                                                "aria-label": "Discard {file_name}",
                                                onclick: move |_| {
                                                    let current = entries();
                                                    for entry in current.iter().filter(|e| e.item.item_id == discard_id) {
                                                        if let Some(task) = entry.task {
                                                            task.cancel();
                                                        }
                                                    }
                                                    entries.set(current.into_iter().filter(|e| e.item.item_id != discard_id).collect());
                                                },
                                                "Discard"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "kit-fda__summary", role: "status", "aria-live": "polite", "{summary_line}" }
            }
            div { class: "kit-fda__notice", role: "status", "aria-live": "polite", "{notice}" }
        }
    }
}

fn spawn_upload(
    entries: Signal<Vec<Entry>>,
    handler: Callback<UploadRequest, UploadTask>,
    item_id: String,
    file: FileData,
    progress: Signal<f64>,
) -> dioxus::core::Task {
    spawn(async move {
        let started = clock_ms().await;
        set_started(entries, &item_id, started);
        let request = UploadRequest {
            item_id: item_id.clone(),
            file,
            progress,
        };
        let outcome = handler.call(request).await;
        finish(entries, &item_id, outcome);
    })
}

/// Stands in for an upload when the host gives no handler: progress runs 0 to 1 and the row stores.
fn simulate_upload(
    entries: Signal<Vec<Entry>>,
    item_id: String,
    mut progress: Signal<f64>,
) -> dioxus::core::Task {
    spawn(async move {
        let started = clock_ms().await;
        set_started(entries, &item_id, started);
        for step in 1..=10 {
            sleep(120).await;
            progress.set(step as f64 / 10.0);
        }
        set_state(entries, &item_id, ItemState::Done, None);
    })
}

fn set_started(mut entries: Signal<Vec<Entry>>, item_id: &str, started: Option<f64>) {
    entries.with_mut(|list| {
        for entry in list.iter_mut().filter(|e| e.item.item_id == item_id) {
            entry.started_ms = started;
        }
    });
}

/// Epoch milliseconds from the browser; `None` with no renderer (SSR, tests).
async fn clock_ms() -> Option<f64> {
    document::eval("return Date.now();").await.ok()?.as_f64()
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
