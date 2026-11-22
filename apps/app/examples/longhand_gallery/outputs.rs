use dioxus::prelude::*;
use rv2_kit::screens::file_library::{
    Account as LibraryAccount, FileLibrary, Folder, Kind, Scope, Template as LibraryTemplate,
};
use rv2_kit::screens::folder_detail::{
    Account as FolderAccount, FileItem, FileKind, FolderDetail, ShareState,
};

use super::Scene;

fn file(id: &str, name: &str, kind: FileKind, when: &str) -> FileItem {
    FileItem {
        id: id.into(),
        name: name.into(),
        kind,
        updated_at: when.into(),
        subtitle: format!("Saved by a workflow, {when}"),
        preview_url: None,
    }
}

fn folder_files(folder: &str) -> Vec<FileItem> {
    match folder {
        "pipeline" => vec![
            file(
                "p1",
                "Pipeline summary, run 12.pdf",
                FileKind::Pdf,
                "Nov 17",
            ),
            file(
                "p2",
                "Renewal risk list, run 12.xlsx",
                FileKind::Sheet,
                "Nov 17",
            ),
            file("p3", "Owner emails, run 12.docx", FileKind::Doc, "Nov 17"),
        ],
        "news" => vec![
            file("n1", "Newsletter, week 46.docx", FileKind::Doc, "Nov 15"),
            file("n2", "Newsletter, week 45.docx", FileKind::Doc, "Nov 8"),
            file("n3", "Hero image, week 46.png", FileKind::Image, "Nov 15"),
            file(
                "n4",
                "Open rate, weeks 40 to 46.xlsx",
                FileKind::Sheet,
                "Nov 16",
            ),
        ],
        _ => vec![],
    }
}

fn folder_name(folder: &str) -> &'static str {
    match folder {
        "pipeline" => "Monday pipeline review",
        "news" => "Weekly customer newsletter",
        _ => "Renewal follow-ups",
    }
}

#[component]
pub fn StoredOutputs() -> Element {
    let mut open_folder = use_signal(|| Some("news".to_string()));
    let mut shared = use_signal(|| false);
    let mut last = use_signal(|| "no action yet".to_string());
    let current = open_folder();
    let files = current.as_deref().map(folder_files).unwrap_or_default();
    let title = current
        .as_deref()
        .map(folder_name)
        .unwrap_or("Stored outputs");
    let share_state = if shared() {
        ShareState::Shared
    } else {
        ShareState::Private
    };

    rsx! {
        Scene {
            id: "outputs",
            title: "Stored outputs",
            story: "Lena finds last week's newsletter without remembering where it went. Outputs are filed under the workflow that made them, and she opens one folder to see every run.",
            div { class: "lh-stack",
                div { class: "lh-box", style: "height: 640px;",
                    FileLibrary {
                        account: LibraryAccount { name: "Lena Ruiz".into(), avatar_url: None },
                        folders: vec![
                            Folder { id: "pipeline".into(), name: "Monday pipeline review".into(), scope: Scope::Shared, file_count: 3 },
                            Folder { id: "news".into(), name: "Weekly customer newsletter".into(), scope: Scope::Shared, file_count: 4 },
                            Folder { id: "renewals".into(), name: "Renewal follow-ups".into(), scope: Scope::Private, file_count: 0 },
                        ],
                        templates: vec![
                            LibraryTemplate { id: "t1".into(), name: "Pipeline summary".into(), kind: Kind::Doc, owner: "Sam".into(), scope: Scope::Shared },
                            LibraryTemplate { id: "t2".into(), name: "Renewal risk list".into(), kind: Kind::Sheet, owner: "Priya".into(), scope: Scope::Mine },
                        ],
                        on_open_folder: move |id: String| {
                            last.set(format!("opened folder {id}"));
                            open_folder.set(Some(id));
                        },
                        on_use_template: move |id: String| last.set(format!("use template {id}")),
                        on_navigate: move |label: String| last.set(format!("navigate {label}")),
                    }
                }
                div { class: "lh-between",
                    h3 { class: "lh-h", "{title}" }
                    div { class: "lh-row",
                        button { class: "lh-btn", r#type: "button",
                            aria_pressed: "{shared()}",
                            onclick: move |_| {
                                let next = !shared();
                                shared.set(next);
                                last.set(if next { "folder shared with the team".to_string() } else { "folder is private".to_string() });
                            },
                            {if shared() { "Shared with team" } else { "Share with team" }}
                        }
                    }
                }
                if let Some(name) = current.clone() {
                    div { class: "lh-box", style: "height: 560px;",
                        FolderDetail {
                            folder_name: title,
                            crumbs: vec!["Workflows".to_string(), name],
                            account: FolderAccount { name: "Lena Ruiz".into(), avatar_url: None },
                            files: files,
                            share_state: share_state,
                            on_upload: move |names: Vec<String>| last.set(format!("uploaded {}", names.join(", "))),
                            on_share: move |_| shared.set(true),
                            on_open_file: move |id: String| last.set(format!("opened file {id}")),
                            on_remove: move |ids: Vec<String>| last.set(format!("removed {} files", ids.len())),
                            on_start_chat: move |_| last.set("started a chat about this folder".to_string()),
                            on_navigate: move |label: String| last.set(format!("navigate {label}")),
                        }
                    }
                } else {
                    p { class: "lh-empty", "Pick a folder to see the outputs of each run." }
                }
                p { class: "gl-out", "data-out": "outputs", "{last()}" }
            }
        }
    }
}
