use dioxus::prelude::*;
use rv2_kit::components::connector_picker::{
    Connector, ConnectorLabels, ConnectorPicker, count_text, grouped_matches, header_title,
    toggle_connection,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn conn(id: &str, name: &str, category: &str, connected: bool) -> Connector {
    Connector {
        connector_id: id.into(),
        connector_name: name.into(),
        connector_domain: format!("{id}.example.com"),
        connector_category: category.into(),
        connector_summary: format!("Keep {name} records in step."),
        connector_features: vec![
            "Two-way sync".into(),
            "Scheduled refresh".into(),
            "Field mapping".into(),
        ],
        connector_mark: None,
        is_connected: connected,
    }
}

fn services() -> Vec<Connector> {
    vec![
        conn("files", "Files", "Storage", true),
        conn("calendar", "Calendar", "Planning", true),
        conn("mail", "Mail", "Messaging", true),
        conn("warehouse", "Warehouse", "Data", true),
        conn("drive", "Drive", "Storage", false),
        conn("planner", "Planner", "Planning", false),
        conn("chat", "Chat", "Messaging", false),
        conn("sheets", "Sheets", "Data", false),
        conn("notes", "Notes", "Storage", false),
        conn("board", "Board", "Planning", false),
    ]
}

fn panel() -> String {
    render(|| {
        rsx! {
            ConnectorPicker {
                connector_list: Some(services()),
                product_name: "Harbor",
                label_text: ConnectorLabels {
                    explore: Some("Open settings".into()),
                    ..ConnectorLabels::default()
                },
            }
        }
    })
}

#[test]
fn header_counts_connected_services_out_of_the_total() {
    assert_eq!(count_text(&services()), "4 of 10 connected");
    assert!(panel().contains("4 of 10 connected"));
}

#[test]
fn list_pane_shows_four_connected_rows_then_six_available() {
    let html = panel();
    assert_eq!(html.matches("class=\"kit-cp__row\"").count(), 10);
    let connected = html
        .find("aria-label=\"Connected\"")
        .expect("connected group");
    let available = html
        .find("aria-label=\"Available\"")
        .expect("available group");
    assert!(connected < available);
    let (connected_rows, available_rows) = grouped_matches(&services(), "");
    assert_eq!(connected_rows.len(), 4);
    assert_eq!(available_rows.len(), 6);
}

#[test]
fn title_names_the_product_and_the_picked_first_connected_service() {
    assert_eq!(
        header_title("Harbor", "Files", &ConnectorLabels::default()),
        "Harbor + Files"
    );
    assert!(panel().contains("Harbor + Files"));
}

#[test]
fn detail_pane_shows_domain_status_summary_and_three_features() {
    let html = panel();
    assert!(html.contains("files.example.com"));
    assert!(!html.contains("Not connected"));
    assert_eq!(html.matches("<li>").count(), 3);
    assert!(html.contains("Keep Files records in step."));
}

#[test]
fn explore_switch_sits_in_the_detail_head_beside_the_title() {
    let html = panel();
    let head = html.find("kit-cp__detail-head").expect("detail head");
    let status = html.find("kit-cp__status").expect("status");
    assert!(html[head..status].contains("checked"));
}

#[test]
fn explore_action_reads_the_explore_wording() {
    assert!(panel().contains("Open settings"));
}

#[test]
fn search_typing_hides_non_matching_rows_in_both_groups() {
    let (connected, available) = grouped_matches(&services(), "data");
    assert_eq!(connected.len(), 1);
    assert_eq!(available.len(), 1);
}

#[test]
fn switching_a_connector_flips_its_state_and_reports_it() {
    let mut list = services();
    assert_eq!(toggle_connection(&mut list, "drive"), Some(true));
    assert_eq!(toggle_connection(&mut list, "files"), Some(false));
    assert_eq!(toggle_connection(&mut list, "missing"), None);
}
