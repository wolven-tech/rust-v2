use dioxus::prelude::*;
use rv2_kit::screens::file_library::{Account, FileLibrary, Folder, Kind, Scope, Template};

const CSS: &str = include_str!("../assets/file-library.css");

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// The CSS block for one selector, from its opening brace to the closing brace.
fn rule<'a>(css: &'a str, selector: &str) -> &'a str {
    let start = css
        .find(&format!("{selector} {{"))
        .unwrap_or_else(|| panic!("no rule for {selector}"));
    let end = css[start..]
        .find('}')
        .map(|i| start + i)
        .unwrap_or(css.len());
    &css[start..end]
}

fn account() -> Account {
    Account {
        name: "Ada Lovelace".into(),
        avatar_url: None,
    }
}

fn rest_folders() -> Vec<Folder> {
    [
        ("f1", "Q3 Campaign", Scope::Shared, 12),
        ("f2", "Drafts", Scope::Private, 3),
        ("f3", "Board packs", Scope::Shared, 7),
        ("f4", "Hiring", Scope::Private, 5),
    ]
    .into_iter()
    .map(|(id, name, scope, file_count)| Folder {
        id: id.into(),
        name: name.into(),
        scope,
        file_count,
    })
    .collect()
}

fn rest_templates() -> Vec<Template> {
    [
        ("t1", "Brand guidelines", Kind::Doc, "Ada", Scope::Shared),
        ("t2", "Campaign budget", Kind::Sheet, "Lin", Scope::Mine),
        (
            "t3",
            "Client onboarding kit",
            Kind::Pdf,
            "Sam",
            Scope::Shared,
        ),
        ("t4", "Weekly status report", Kind::Doc, "Ada", Scope::Mine),
    ]
    .into_iter()
    .map(|(id, name, kind, owner, scope)| Template {
        id: id.into(),
        name: name.into(),
        kind,
        owner: owner.into(),
        scope,
    })
    .collect()
}

fn rest_app() -> Element {
    rsx! {
        FileLibrary {
            account: account(),
            folders: rest_folders(),
            templates: rest_templates(),
        }
    }
}

#[test]
fn rest_page_shows_four_folder_cards_and_four_template_cards() {
    let html = render(rest_app);
    assert_eq!(html.matches("kit-fl__card\"").count(), 8);
    assert_eq!(html.matches("kit-fl__item--template").count(), 4);
    assert_eq!(rest_folders().len(), 4);
    assert_eq!(rest_templates().len(), 4);
}

#[test]
fn rest_section_headings_count_their_cards() {
    let html = render(rest_app);
    assert!(html.contains("Folders "));
    assert!(html.contains("File templates "));
    assert!(html.matches("kit-fl__count\">4</span>").count() >= 2);
}

#[test]
fn rest_template_names_are_the_four_demo_names() {
    let names: Vec<String> = rest_templates().into_iter().map(|t| t.name).collect();
    assert_eq!(
        names,
        vec![
            "Brand guidelines",
            "Campaign budget",
            "Client onboarding kit",
            "Weekly status report",
        ]
    );
}

#[test]
fn template_card_names_its_owner_and_scope() {
    let html = render(rest_app);
    assert!(html.contains("aria-label=\"Brand guidelines, DOC, owned by Ada, Shared\""));
    assert!(html.contains("aria-label=\"Client onboarding kit, PDF, owned by Sam, Shared\""));
    assert!(html.contains("aria-label=\"Weekly status report, DOC, owned by Ada, Mine\""));
}

#[test]
fn folder_card_names_its_scope_and_file_count() {
    let html = render(rest_app);
    assert!(html.contains("aria-label=\"Q3 Campaign, Shared, 12 files\""));
    assert!(html.contains("aria-label=\"Hiring, Private, 5 files\""));
}

#[test]
fn template_names_wrap_before_they_truncate() {
    let name = rule(CSS, ".kit-fl__name");
    assert!(!name.contains("ellipsis"));
    assert!(!name.contains("nowrap"));
    assert!(name.contains("overflow-wrap: anywhere"));
}

#[test]
fn toolbar_stays_on_one_line_above_phone_width() {
    let toolbar = rule(CSS, ".kit-fl__toolbar");
    assert!(toolbar.contains("flex-wrap: nowrap"));
    assert!(CSS.contains(".kit-fl__toolbar {\n    flex-wrap: wrap;"));
}

#[test]
fn template_cards_sit_in_a_grid_of_at_least_240_pixel_columns() {
    let grid = rule(CSS, ".kit-fl__items--templates.is-grid");
    assert!(grid.contains("minmax(240px, 1fr)"));
}
