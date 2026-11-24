//! File library: the top of a shared file area, with folders and file templates in two sections that share one
//! search, one sort and one grid/list choice.
//!
//! Design choice: the spec's "Updated" sort has no timestamps in its data, so it keeps the host's order.
//! Design choice: the grid/list choice is remembered in `localStorage` (try/catch), per browser, not per account.
//! Design choice: "/" focuses the search field, as the spec asks, and is ignored while a field has focus.
//! Design choice: the sidebar is drawn inline so the screen stands alone, without the workspace frame.

use dioxus::prelude::*;

use crate::internal::*;

const CSS: Asset = asset!("/assets/file-library.css");
const VIEW_KEY: &str = "kit-file-library-view";

#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub name: String,
    pub avatar_url: Option<String>,
}

/// Who a folder or template is shared with: the team, only the person, or the person's own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scope {
    Shared,
    #[default]
    Private,
    Mine,
}

impl Scope {
    pub fn label(self) -> &'static str {
        match self {
            Self::Shared => "Shared",
            Self::Private => "Private",
            Self::Mine => "Mine",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub scope: Scope,
    pub file_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    Doc,
    Sheet,
    Pdf,
    #[default]
    Other,
}

impl Kind {
    pub const ALL: [Self; 4] = [Self::Doc, Self::Sheet, Self::Pdf, Self::Other];

    pub fn code(self) -> &'static str {
        match self {
            Self::Doc => "DOC",
            Self::Sheet => "XLS",
            Self::Pdf => "PDF",
            Self::Other => "FILE",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub kind: Kind,
    pub owner: String,
    pub scope: Scope,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FolderFilter {
    #[default]
    All,
    Shared,
    Private,
}

impl FolderFilter {
    pub const ALL: [Self; 3] = [Self::All, Self::Shared, Self::Private];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Shared => "Shared",
            Self::Private => "Private",
        }
    }

    pub fn admits(self, scope: Scope) -> bool {
        match self {
            Self::All => true,
            Self::Shared => scope == Scope::Shared,
            Self::Private => scope == Scope::Private,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TemplateFilter {
    #[default]
    All,
    Mine,
    Shared,
}

impl TemplateFilter {
    pub const ALL: [Self; 3] = [Self::All, Self::Mine, Self::Shared];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Mine => "Mine",
            Self::Shared => "Shared with me",
        }
    }

    pub fn admits(self, scope: Scope) -> bool {
        match self {
            Self::All => true,
            Self::Mine => scope == Scope::Mine,
            Self::Shared => scope == Scope::Shared,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortKey {
    #[default]
    Name,
    Updated,
    Kind,
}

impl SortKey {
    pub const ALL: [Self; 3] = [Self::Name, Self::Updated, Self::Kind];

    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Updated => "Updated",
            Self::Kind => "Kind",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    Grid,
    List,
}

/// Whether a name matches the search text, ignoring case; an empty query matches everything.
pub fn matches_query(name: &str, query: &str) -> bool {
    let needle = query.trim().to_lowercase();
    needle.is_empty() || name.to_lowercase().contains(&needle)
}

/// Folders sorted by the key. `Updated` and `Kind` keep the host's order, since folders carry neither.
pub fn sort_folders(folders: &[Folder], key: SortKey) -> Vec<Folder> {
    let mut out = folders.to_vec();
    if key == SortKey::Name {
        out.sort_by_key(|f| f.name.to_lowercase());
    }
    out
}

/// Templates sorted by the key. `Updated` keeps the host's order; `Kind` sorts by kind code, then name.
pub fn sort_templates(templates: &[Template], key: SortKey) -> Vec<Template> {
    let mut out = templates.to_vec();
    match key {
        SortKey::Name => out.sort_by_key(|t| t.name.to_lowercase()),
        SortKey::Kind => out.sort_by_key(|t| (t.kind.code(), t.name.to_lowercase())),
        SortKey::Updated => {}
    }
    out
}

/// The action a template's menu asks the host to carry out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateAction {
    Rename,
    Duplicate,
    Remove,
}

/// The file library page: sidebar, title, folders and templates with their filters.
#[component]
pub fn FileLibrary(
    #[props(default = "Juniper Works".to_string(), into)] workspace_name: String,
    account: Account,
    #[props(default)] folders: Vec<Folder>,
    #[props(default)] templates: Vec<Template>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_create_folder: EventHandler<()>,
    #[props(default)] on_add_template: EventHandler<()>,
    #[props(default)] on_open_folder: EventHandler<String>,
    #[props(default)] on_use_template: EventHandler<String>,
    #[props(default)] on_template_action: EventHandler<(String, TemplateAction)>,
    #[props(default)] on_navigate: EventHandler<String>,
) -> Element {
    let base = use_hook(|| format!("kit-fl-{}", next_id()));
    let mut folder_filter = use_signal(FolderFilter::default);
    let mut template_filter = use_signal(TemplateFilter::default);
    let mut kind_filter = use_signal(|| None::<Kind>);
    let mut query = use_signal(String::new);
    let mut sort = use_signal(SortKey::default);
    let mut sort_open = use_signal(|| false);
    let mut view = use_signal(ViewMode::default);
    let mut menu_for = use_signal(|| None::<String>);
    let mut announce = use_signal(String::new);

    use_hook(move || {
        spawn(async move {
            let stored = document::eval(&format!(
                "try {{ return localStorage.getItem({VIEW_KEY:?}) || ''; }} catch (e) {{ return ''; }}"
            ))
            .await
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();
            if stored == "list" {
                view.set(ViewMode::List);
            }
        });
    });

    let folder_now = folder_filter();
    let template_now = template_filter();
    let kind_now = kind_filter();
    let search = query();
    let sort_key = sort();
    let visible_folders: Vec<Folder> = sort_folders(&folders, sort_key)
        .into_iter()
        .filter(|f| folder_now.admits(f.scope) && matches_query(&f.name, &search))
        .collect();
    let visible_templates: Vec<Template> = sort_templates(&templates, sort_key)
        .into_iter()
        .filter(|t| template_now.admits(t.scope))
        .filter(|t| kind_now.is_none_or(|k| k == t.kind))
        .filter(|t| matches_query(&t.name, &search))
        .collect();
    let folder_total = folders.len();
    let template_total = templates.len();
    let searching = !search.trim().is_empty();
    let folders_nothing = !folders.is_empty() && visible_folders.is_empty();
    let templates_nothing = !templates.is_empty() && visible_templates.is_empty();
    let nothing_at_all = searching && folders_nothing && templates_nothing;
    let is_grid = view() == ViewMode::Grid;
    let grid_class = if is_grid {
        "kit-fl__items is-grid"
    } else {
        "kit-fl__items is-list"
    };
    let sort_list_id = format!("{base}-sort-list");
    let sort_trigger_id = format!("{base}-sort");
    let search_id = format!("{base}-search");
    let folder_tabs: Vec<(FolderFilter, usize)> = FolderFilter::ALL
        .iter()
        .map(|f| (*f, folders.iter().filter(|x| f.admits(x.scope)).count()))
        .collect();
    let template_tabs: Vec<(TemplateFilter, usize)> = TemplateFilter::ALL
        .iter()
        .map(|f| (*f, templates.iter().filter(|x| f.admits(x.scope)).count()))
        .collect();
    let results_text = format!(
        "{} results",
        visible_folders.len() + visible_templates.len()
    );

    let on_page_keys = {
        let search_id = search_id.clone();
        move |event: KeyboardEvent| {
            if let Key::Character(c) = event.key() {
                if c == "/" {
                    event.prevent_default();
                    focus_after_frame(&search_id);
                }
            }
        }
    };
    let on_search_keys = move |event: KeyboardEvent| {
        event.stop_propagation();
        if matches!(event.key(), Key::Escape) {
            query.set(String::new());
        }
    };
    let on_sort_keys = {
        let sort_trigger_id = sort_trigger_id.clone();
        move |event: KeyboardEvent| {
            let current = SortKey::ALL.iter().position(|k| *k == sort()).unwrap_or(0);
            match event.key() {
                Key::ArrowDown => {
                    event.prevent_default();
                    sort.set(SortKey::ALL[(current + 1).min(SortKey::ALL.len() - 1)]);
                }
                Key::ArrowUp => {
                    event.prevent_default();
                    sort.set(SortKey::ALL[current.saturating_sub(1)]);
                }
                Key::Enter | Key::Escape => {
                    event.prevent_default();
                    sort_open.set(false);
                    focus_after_frame(&sort_trigger_id);
                }
                _ => {}
            }
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "kit-fl", "aria-labelledby": "{base}-title", onkeydown: on_page_keys,
            nav { class: "kit-fl__nav", "aria-label": "Sidebar",
                p { class: "kit-fl__ws", "{workspace_name}" }
                ul { class: "kit-fl__nav-list",
                    for item in ["Home", "Files", "Inbox", "Settings"] {
                        li { key: "{item}",
                            button { class: "kit-fl__nav-item", r#type: "button",
                                "aria-current": if item == "Files" { "page" } else { "false" },
                                onclick: move |_| on_navigate.call(item.to_string()),
                                "{item}"
                            }
                        }
                    }
                }
                div { class: "kit-fl__account",
                    span { class: "kit-fl__avatar", "aria-hidden": "true",
                        if let Some(src) = account.avatar_url.clone() {
                            img { src: "{src}", alt: "" }
                        } else {
                            "{account.name.chars().next().unwrap_or('?')}"
                        }
                    }
                    span { class: "kit-fl__account-name", "{account.name}" }
                }
            }

            div { class: "kit-fl__page",
                header { class: "kit-fl__head",
                    h1 { id: "{base}-title", class: "kit-fl__title", "Files" }
                    p { class: "kit-fl__lede",
                        "Folders and templates your team shares. "
                        button { class: "kit-fl__link", r#type: "button", onclick: move |_| on_navigate.call("Learn more".to_string()), "Learn more" }
                    }
                }

                div { class: "kit-fl__toolbar", role: "search",
                    label { class: "kit-fl__search",
                        span { class: "kit-sr", "Search folders and templates" }
                        input { id: "{search_id}", class: "kit-fl__input", r#type: "search", placeholder: "Search files and folders (/)",
                            value: "{search}", oninput: move |e| query.set(e.value()), onkeydown: on_search_keys }
                    }
                    div { class: "kit-fl__sort",
                        button { id: "{sort_trigger_id}", class: "kit-fl__btn", r#type: "button", "aria-haspopup": "listbox",
                            "aria-expanded": "{sort_open()}", "aria-controls": "{sort_list_id}",
                            onclick: {
                                let sort_list_id = sort_list_id.clone();
                                move |_| {
                                    let next = !sort_open();
                                    sort_open.set(next);
                                    if next {
                                        focus_after_frame(&sort_list_id);
                                    }
                                }
                            },
                            "Sort: {sort_key.label()}"
                        }
                        if sort_open() {
                            ul { id: "{sort_list_id}", class: "kit-fl__menu", role: "listbox", tabindex: "-1", "aria-label": "Sort by",
                                onkeydown: on_sort_keys,
                                for key in SortKey::ALL {
                                    li { key: "{key.label()}", role: "option",
                                        "aria-selected": if key == sort_key { "true" } else { "false" },
                                        class: if key == sort_key { "kit-fl__opt is-active" } else { "kit-fl__opt" },
                                        onclick: move |_| {
                                            sort.set(key);
                                            sort_open.set(false);
                                        },
                                        "{key.label()}"
                                    }
                                }
                            }
                        }
                    }
                    div { class: "kit-fl__segment", role: "group", "aria-label": "Layout",
                        button { class: "kit-fl__seg", r#type: "button", "aria-pressed": "{is_grid}",
                            onclick: move |_| {
                                view.set(ViewMode::Grid);
                                let _ = document::eval(&format!("try {{ localStorage.setItem({VIEW_KEY:?}, 'grid'); }} catch (e) {{}}"));
                            },
                            "Grid"
                        }
                        button { class: "kit-fl__seg", r#type: "button", "aria-pressed": "{!is_grid}",
                            onclick: move |_| {
                                view.set(ViewMode::List);
                                let _ = document::eval(&format!("try {{ localStorage.setItem({VIEW_KEY:?}, 'list'); }} catch (e) {{}}"));
                            },
                            "List"
                        }
                    }
                    p { class: "kit-sr", role: "status", "aria-live": "polite",
                        if searching { "{results_text}" }
                    }
                }

                if nothing_at_all {
                    div { class: "kit-fl__nomatch",
                        p { "Nothing matches" }
                        button { class: "kit-fl__btn", r#type: "button",
                            onclick: move |_| {
                                query.set(String::new());
                                folder_filter.set(FolderFilter::All);
                                template_filter.set(TemplateFilter::All);
                                kind_filter.set(None);
                            },
                            "Clear filters"
                        }
                    }
                }

                section { class: "kit-fl__section", "aria-labelledby": "{base}-folders",
                    div { class: "kit-fl__section-head",
                        h2 { id: "{base}-folders", class: "kit-fl__section-title",
                            "Folders "
                            span { class: "kit-fl__count", "{folder_total}" }
                        }
                        div { class: "kit-fl__section-actions",
                            div { class: "kit-fl__tabs", role: "tablist", "aria-label": "Folder scope",
                                for (f, n) in folder_tabs.iter() {
                                    button { key: "{f:?}", class: "kit-fl__tab", r#type: "button", role: "tab",
                                        "aria-selected": if *f == folder_now { "true" } else { "false" },
                                        tabindex: if *f == folder_now { "0" } else { "-1" },
                                        onclick: {
                                            let f = *f;
                                            move |_| folder_filter.set(f)
                                        },
                                        "{f.label()} "
                                        span { class: "kit-fl__count", "{n}" }
                                    }
                                }
                            }
                            button { class: "kit-fl__btn kit-fl__btn--primary", r#type: "button", onclick: move |_| on_create_folder.call(()), "Create folder" }
                        }
                    }
                    if is_loading {
                        div { class: "{grid_class}", "aria-busy": "true",
                            for i in 0..3 {
                                span { key: "{i}", class: "kit-fl__ph kit-fl__ph--card" }
                            }
                        }
                    } else if is_empty || folders.is_empty() {
                        div { class: "kit-fl__empty",
                            p { class: "kit-fl__empty-title", "No folders yet" }
                            button { class: "kit-fl__btn kit-fl__btn--primary", r#type: "button", onclick: move |_| on_create_folder.call(()), "New folder" }
                        }
                    } else if folders_nothing {
                        div { class: "kit-fl__empty",
                            p { class: "kit-fl__empty-title", "Nothing matches" }
                            button { class: "kit-fl__btn", r#type: "button", onclick: move |_| folder_filter.set(FolderFilter::All), "Show all folders" }
                        }
                    } else {
                        ul { class: "{grid_class}",
                            for folder in visible_folders.iter() {
                                li { key: "{folder.id}", class: "kit-fl__item",
                                    button { class: "kit-fl__card", r#type: "button",
                                        "aria-label": "{folder.name}, {folder.scope.label()}, {folder.file_count} files",
                                        onclick: {
                                            let id = folder.id.clone();
                                            move |_| on_open_folder.call(id.clone())
                                        },
                                        span { class: "kit-fl__glyph", "aria-hidden": "true", "▤" }
                                        span { class: "kit-fl__name", "{folder.name}" }
                                        span { class: "kit-fl__meta", "aria-hidden": "true",
                                            span { class: "kit-fl__scope", "{folder.scope.label()}" }
                                            " {folder.file_count} files"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                section { class: "kit-fl__section", "aria-labelledby": "{base}-templates",
                    div { class: "kit-fl__section-head",
                        h2 { id: "{base}-templates", class: "kit-fl__section-title",
                            "File templates "
                            span { class: "kit-fl__count", "{template_total}" }
                        }
                        div { class: "kit-fl__section-actions",
                            div { class: "kit-fl__tabs", role: "tablist", "aria-label": "Template owner",
                                for (f, n) in template_tabs.iter() {
                                    button { key: "{f:?}", class: "kit-fl__tab", r#type: "button", role: "tab",
                                        "aria-selected": if *f == template_now { "true" } else { "false" },
                                        tabindex: if *f == template_now { "0" } else { "-1" },
                                        onclick: {
                                            let f = *f;
                                            move |_| template_filter.set(f)
                                        },
                                        "{f.label()} "
                                        span { class: "kit-fl__count", "{n}" }
                                    }
                                }
                            }
                            select { class: "kit-fl__select", "aria-label": "Template type",
                                onchange: move |e| kind_filter.set(Kind::ALL.iter().copied().find(|k| k.code() == e.value())),
                                option { value: "", selected: kind_now.is_none(), "All types" }
                                for k in Kind::ALL {
                                    option { key: "{k.code()}", value: "{k.code()}", selected: kind_now == Some(k), "{k.code()}" }
                                }
                            }
                            button { class: "kit-fl__btn kit-fl__btn--primary", r#type: "button", onclick: move |_| on_add_template.call(()), "Add template" }
                        }
                    }
                    if is_loading {
                        div { class: "{grid_class}", "aria-busy": "true",
                            for i in 0..3 {
                                span { key: "{i}", class: "kit-fl__ph kit-fl__ph--row" }
                            }
                        }
                    } else if is_empty || templates.is_empty() {
                        div { class: "kit-fl__empty",
                            p { class: "kit-fl__empty-title", "No templates yet" }
                            button { class: "kit-fl__btn kit-fl__btn--primary", r#type: "button", onclick: move |_| on_add_template.call(()), "Add template" }
                        }
                    } else if templates_nothing {
                        div { class: "kit-fl__empty",
                            p { class: "kit-fl__empty-title", "Nothing matches" }
                            button { class: "kit-fl__btn", r#type: "button", onclick: move |_| template_filter.set(TemplateFilter::All), "Show all templates" }
                        }
                    } else {
                        ul { class: "{grid_class} kit-fl__items--templates",
                            for t in visible_templates.iter() {
                                li { key: "{t.id}", class: "kit-fl__item kit-fl__item--template",
                                    button { class: "kit-fl__card", r#type: "button",
                                        "aria-label": "{t.name}, {t.kind.code()}, owned by {t.owner}, {t.scope.label()}",
                                        onclick: {
                                            let id = t.id.clone();
                                            move |_| on_use_template.call(id.clone())
                                        },
                                        span { class: "kit-fl__kind", "aria-hidden": "true", "{t.kind.code()}" }
                                        span { class: "kit-fl__name", "{t.name}" }
                                        span { class: "kit-fl__meta", "aria-hidden": "true",
                                            "{t.owner} · "
                                            span { class: "kit-fl__scope", "{t.scope.label()}" }
                                        }
                                    }
                                    div { class: "kit-fl__more",
                                        button { class: "kit-fl__btn", r#type: "button", "aria-haspopup": "menu",
                                            "aria-label": "Actions for {t.name}",
                                            "aria-expanded": if menu_for() == Some(t.id.clone()) { "true" } else { "false" },
                                            onclick: {
                                                let id = t.id.clone();
                                                move |_| {
                                                    let next = if menu_for() == Some(id.clone()) { None } else { Some(id.clone()) };
                                                    menu_for.set(next);
                                                }
                                            },
                                            "More"
                                        }
                                        if menu_for() == Some(t.id.clone()) {
                                            div { class: "kit-fl__menu kit-fl__menu--row", role: "menu",
                                                onkeydown: move |event: KeyboardEvent| {
                                                    if matches!(event.key(), Key::Escape) {
                                                        menu_for.set(None);
                                                    }
                                                },
                                                for (label, action) in [("Rename", TemplateAction::Rename), ("Duplicate", TemplateAction::Duplicate), ("Remove", TemplateAction::Remove)] {
                                                    button { key: "{label}", class: "kit-fl__opt", r#type: "button", role: "menuitem",
                                                        onclick: {
                                                            let id = t.id.clone();
                                                            move |_| {
                                                                on_template_action.call((id.clone(), action));
                                                                announce.set(format!("{label} {id}"));
                                                                menu_for.set(None);
                                                            }
                                                        },
                                                        "{label}"
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
            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
