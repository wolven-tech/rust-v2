//! Parallel edit overlap grid: contributors editing one repository at once, flagged where their line spans overlap.
//!
//! Design choice: a row is a conflict when two edits by different contributors share a line. The caller's
//! `conflict` options are used when given; otherwise each overlapping contributor is offered as an option.
//! Design choice: picks are kept locally when `chosen_options` is omitted, and `on_choose_option` is always called.
//! Design choice: the grid is a native table, with contributors as column headers and files as row headers.
//! Browser checks: bar grow-in, sheen on active edits, glow on connectors, option sentence wrap at phone width.

use dioxus::prelude::*;

use crate::internal::*;

const CSS: Asset = asset!("/assets/parallel-edit-overlap-grid.css");
/// Width of the file column in the stylesheet; connector endpoints are placed past it.
const FILE_COLUMN_PCT: f64 = 30.0;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Contributor {
    pub contributor_id: String,
    pub contributor_name: String,
    pub model_name: Option<String>,
    pub branch_name: Option<String>,
    pub contributor_mark: Option<String>,
    pub is_active: bool,
}

/// Inclusive line span `(start, end)` that a contributor changed in one file.
#[derive(Clone, PartialEq, Debug)]
pub struct Edit {
    pub contributor_id: String,
    pub line_range: (u32, u32),
    pub is_editing: bool,
    pub edit_note: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ConflictOption {
    pub contributor_id: String,
    pub option_text: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FileRow {
    pub file_path: String,
    pub edits: Vec<Edit>,
    pub conflict: Option<Vec<ConflictOption>>,
}

/// Whether two inclusive line spans share at least one line.
pub fn spans_overlap(a: (u32, u32), b: (u32, u32)) -> bool {
    a.0 <= b.1 && b.0 <= a.1
}

/// Index pairs of edits by different contributors whose spans overlap.
pub fn overlapping_pairs(row: &FileRow) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for i in 0..row.edits.len() {
        for j in (i + 1)..row.edits.len() {
            let (a, b) = (&row.edits[i], &row.edits[j]);
            if a.contributor_id != b.contributor_id && spans_overlap(a.line_range, b.line_range) {
                pairs.push((i, j));
            }
        }
    }
    pairs
}

pub fn is_conflict(row: &FileRow) -> bool {
    !overlapping_pairs(row).is_empty()
}

/// Options offered on a conflicting row: the caller's list, else each contributor who overlaps.
pub fn options_for(row: &FileRow) -> Vec<ConflictOption> {
    if let Some(options) = &row.conflict {
        return options.clone();
    }
    let mut ids: Vec<String> = Vec::new();
    for (i, j) in overlapping_pairs(row) {
        for idx in [i, j] {
            let id = row.edits[idx].contributor_id.clone();
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    ids.into_iter()
        .map(|id| ConflictOption {
            option_text: format!("Keep {id}'s version"),
            contributor_id: id,
        })
        .collect()
}

/// Merge is open when every conflict has a pick and no contributor is still active.
pub fn can_merge(
    rows: &[FileRow],
    picks: &[(String, String)],
    contributors: &[Contributor],
) -> bool {
    contributors.iter().all(|c| !c.is_active)
        && rows
            .iter()
            .filter(|r| is_conflict(r))
            .all(|r| picks.iter().any(|(p, _)| *p == r.file_path))
}

/// Status word for the title bar.
pub fn status_word(
    rows: &[FileRow],
    picks: &[(String, String)],
    contributors: &[Contributor],
) -> String {
    let active = contributors.iter().filter(|c| c.is_active).count();
    if active > 0 {
        format!("{active} still active")
    } else if can_merge(rows, picks, contributors) {
        "Ready to combine".to_string()
    } else {
        "Conflicts open".to_string()
    }
}

/// Files, open conflicts, and files each contributor has kept.
pub fn footer_counts(
    rows: &[FileRow],
    picks: &[(String, String)],
    contributors: &[Contributor],
) -> (usize, usize, Vec<(String, usize)>) {
    let open = rows
        .iter()
        .filter(|r| is_conflict(r) && !picks.iter().any(|(p, _)| *p == r.file_path))
        .count();
    let kept = contributors
        .iter()
        .map(|c| {
            let n = picks
                .iter()
                .filter(|(_, cid)| *cid == c.contributor_id)
                .count();
            (c.contributor_id.clone(), n)
        })
        .collect();
    (rows.len(), open, kept)
}

/// Splits a path into its folder prefix and file name.
pub fn split_path(path: &str) -> (&str, &str) {
    match path.rfind('/') {
        Some(i) => (&path[..=i], &path[i + 1..]),
        None => ("", path),
    }
}

/// Contributors × files grid of line spans, with conflicts that need a choice before merge.
#[component]
pub fn ParallelEditOverlapGrid(
    contributors: Vec<Contributor>,
    file_rows: Vec<FileRow>,
    #[props(default = "Parallel work".to_string(), into)] section_title: String,
    #[props(default)] chosen_options: Option<Vec<(String, String)>>,
    #[props(default)] on_choose_option: Option<EventHandler<(String, String)>>,
    #[props(default)] on_merge_request: Option<EventHandler<()>>,
    #[props(default)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-peo-{}", next_id()));
    let mut local_picks = use_signal(Vec::<(String, String)>::new);
    let mut open_row = use_signal(|| None::<usize>);

    let uses_local_picks = chosen_options.is_none();
    let picks: Vec<(String, String)> = chosen_options.clone().unwrap_or_else(|| local_picks());
    let mergeable = can_merge(&file_rows, &picks, &contributors);
    let status = status_word(&file_rows, &picks, &contributors);
    let (files, open, kept) = footer_counts(&file_rows, &picks, &contributors);
    let class = class_list("kit-peo", &root_class);
    let n = contributors.len().max(1);
    let footer_line = format!("{files} files · {open} open conflicts");
    let merge_disabled = !mergeable;

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", "aria-labelledby": "{base}-title",
            header { class: "kit-peo__bar",
                h2 { id: "{base}-title", class: "kit-peo__title", "{section_title}" }
                p { class: "kit-peo__status", role: "status", "aria-live": "polite", "{status}" }
                button {
                    r#type: "button", class: "kit-peo__merge", "aria-disabled": "{merge_disabled}", disabled: merge_disabled,
                    onclick: move |_| {
                        if let Some(h) = on_merge_request {
                            h.call(());
                        }
                    },
                    "Merge"
                }
            }
            div { class: "kit-peo__scroll",
                table { class: "kit-peo__table", "aria-label": "Files by contributor",
                    thead {
                        tr {
                            th { scope: "col", "File" }
                            for c in contributors.iter() {
                                {
                                    let model = c.model_name.clone().or(c.branch_name.clone()).unwrap_or_default();
                                    let name = c.contributor_name.clone();
                                    let active = c.is_active;
                                    let mark = c.contributor_mark.clone().unwrap_or_else(|| name.chars().take(1).collect());
                                    rsx! {
                                        th { key: "{c.contributor_id}", scope: "col", "data-active": active.then_some("true"),
                                            span { class: "kit-peo__mark", "aria-hidden": "true", "{mark}" }
                                            span { class: "kit-peo__name", "{name}" }
                                            span { class: "kit-peo__meta", "{model}" }
                                            if active { span { class: "kit-peo__tag", "Active" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    tbody {
                        for (r, row) in file_rows.iter().enumerate() {
                            {
                                let (folder, file) = split_path(&row.file_path);
                                let conflict = is_conflict(row);
                                let pick = picks.iter().find(|(p, _)| *p == row.file_path).map(|(_, cid)| cid.clone());
                                let pairs = overlapping_pairs(row);
                                let options = options_for(row);
                                let is_open = open_row() == Some(r) && conflict;
                                let path = row.file_path.clone();
                                let folder_text = folder.to_string();
                                let file_text = file.to_string();
                                let cell_ids: Vec<String> = contributors.iter().map(|c| c.contributor_id.clone()).collect();
                                let connector = pairs.first().map(|(a, b)| {
                                    let col = |id: &str| cell_ids.iter().position(|c| c == id).unwrap_or(0);
                                    let x_of = |c: usize| FILE_COLUMN_PCT + (c as f64 + 0.5) / n as f64 * (100.0 - FILE_COLUMN_PCT);
                                    (x_of(col(&row.edits[*a].contributor_id)), x_of(col(&row.edits[*b].contributor_id)))
                                });
                                let conflict_id = format!("{base}-c-{r}");
                                let options_id = format!("{base}-o-{r}");
                                let focus_script = format!("(document.getElementById({options_id:?})?.querySelector('input:checked') ?? document.getElementById({options_id:?})?.querySelector('input'))?.focus();");
                                rsx! {
                                    tr { key: "{path}", class: "kit-peo__row", "data-conflict": conflict.then_some("true"),
                                        tabindex: conflict.then_some("0"),
                                        onkeydown: move |event: KeyboardEvent| {
                                            // Enter on the row and on its Conflict button both reach here; preventing the default stops the button's own click.
                                            if conflict && event.key() == Key::Enter {
                                                event.prevent_default();
                                                open_row.set(if open_row() == Some(r) { None } else { Some(r) });
                                            }
                                        },
                                        th { scope: "row", class: "kit-peo__file",
                                            span { class: "kit-peo__folder", "{folder_text}" }
                                            span { class: "kit-peo__filename", "{file_text}" }
                                            if conflict {
                                                button {
                                                    r#type: "button", class: "kit-peo__conflict", id: "{conflict_id}", "aria-expanded": "{is_open}",
                                                    onclick: move |_| open_row.set(if open_row() == Some(r) { None } else { Some(r) }),
                                                    "Conflict"
                                                }
                                            }
                                            if let Some(cid) = pick.clone() {
                                                span { class: "kit-peo__kept", "Kept {cid}" }
                                            }
                                        }
                                        for c in contributors.iter() {
                                            {
                                                let edit = row.edits.iter().find(|e| e.contributor_id == c.contributor_id).cloned();
                                                let kept_here = pick.as_deref() == Some(c.contributor_id.as_str());
                                                let (start, end) = edit.as_ref().map(|e| e.line_range).unwrap_or((0, 0));
                                                let span_text = format!("{start}–{end}");
                                                let active_edit = edit.as_ref().map(|e| e.is_editing).unwrap_or(false);
                                                let label = match &edit {
                                                    Some(_) => format!("{}, lines {span_text}{}", c.contributor_name, if kept_here { ", kept" } else { "" }),
                                                    None => format!("{}, no change", c.contributor_name),
                                                };
                                                let width = if edit.is_some() { format!("width: {:.0}%;", ((end - start + 1) as f64).min(200.0) / 2.0) } else { "width: 0%;".to_string() };
                                                let title = edit.as_ref().and_then(|e| e.edit_note.clone()).unwrap_or_else(|| label.clone());
                                                rsx! {
                                                    td { key: "{c.contributor_id}", class: "kit-peo__cell", "data-kept": kept_here.then_some("true"), "data-editing": active_edit.then_some("true"),
                                                        "aria-label": "{label}", title: "{title}",
                                                        if edit.is_some() {
                                                            span { class: "kit-peo__span", "{span_text}" }
                                                            span { class: "kit-peo__bar-track", "aria-hidden": "true",
                                                                span { class: "kit-peo__bar-fill", style: "{width}" }
                                                            }
                                                        }
                                                        if kept_here { span { class: "kit-peo__tag", "Kept" } }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if let Some((x1, x2)) = connector {
                                        tr { key: "{row.file_path}-link", class: "kit-peo__link-row", "aria-hidden": "true",
                                            td { colspan: "{n + 1}",
                                                svg { class: "kit-peo__link", view_box: "0 0 100 10", preserve_aspect_ratio: "none",
                                                    path { d: "M {x1:.2} 0 V 6 H {x2:.2} V 0", "data-open": (!mergeable || !picks.iter().any(|(p, _)| *p == row.file_path)).then_some("true") }
                                                }
                                            }
                                        }
                                    }
                                    if is_open {
                                        tr { key: "{row.file_path}-detail", class: "kit-peo__detail-row",
                                            td { colspan: "{n + 1}",
                                                fieldset { id: "{options_id}", class: "kit-peo__options", "aria-label": "Choose which version to keep",
                                                    onmounted: {
                                                        let script = focus_script.clone();
                                                        move |_| {
                                                            document::eval(&script);
                                                        }
                                                    },
                                                    onkeydown: {
                                                        let conflict_id = conflict_id.clone();
                                                        move |event: Event<KeyboardData>| {
                                                            if matches!(event.key(), Key::Escape) {
                                                                event.prevent_default();
                                                                open_row.set(None);
                                                                focus_after_frame(&conflict_id);
                                                            }
                                                        }
                                                    },
                                                    legend { "Keep one version of {file_text}" }
                                                    for option in options.iter() {
                                                        {
                                                            let cid = option.contributor_id.clone();
                                                            let text = option.option_text.clone();
                                                            let checked = pick.as_deref() == Some(cid.clone().as_str());
                                                            let path_for_pick = row.file_path.clone();
                                                            rsx! {
                                                                label { key: "{cid}", class: "kit-peo__option",
                                                                    input {
                                                                        r#type: "radio", name: "{base}-{r}", checked: checked,
                                                                        onchange: move |_| {
                                                                            if uses_local_picks {
                                                                                let mut next: Vec<(String, String)> = local_picks().into_iter().filter(|(p, _)| *p != path_for_pick).collect();
                                                                                next.push((path_for_pick.clone(), cid.clone()));
                                                                                local_picks.set(next);
                                                                            }
                                                                            if let Some(h) = on_choose_option {
                                                                                h.call((path_for_pick.clone(), cid.clone()));
                                                                            }
                                                                        },
                                                                    }
                                                                    span { "{text}" }
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
            }
            footer { class: "kit-peo__foot",
                p { "{footer_line}" }
                ul { class: "kit-peo__kept-list", "aria-label": "Files kept per contributor",
                    for (cid, count) in kept.iter() {
                        li { key: "{cid}", "{cid}: {count} kept" }
                    }
                }
            }
        }
    }
}
