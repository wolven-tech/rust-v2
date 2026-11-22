use dioxus::prelude::*;
use rv2_kit::components::connector_picker::ConnectorPicker;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn picker() -> Element {
    rsx! {
        ConnectorPicker { product_name: "Ledger" }
    }
}

#[test]
fn connector_picker_opens_on_the_service_list_with_a_back_action_in_the_detail() {
    let html = render(picker);
    assert!(html.contains("data-pane=\"list\""));
    assert!(html.contains("class=\"kit-cp__pane-back\""));
}
