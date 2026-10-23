use std::{cell::RefCell, rc::Rc};

use dioxus::{document::Document, prelude::*};
use rv2_ui::{ConsentFooter, consent_footer::*};

type HeadElement = (String, Vec<(String, String)>);

#[derive(Default)]
struct CapturedHead(RefCell<Vec<HeadElement>>);

impl Document for CapturedHead {
    fn eval(&self, js: String) -> document::Eval {
        document::NoOpDocument.eval(js)
    }

    fn create_head_element(&self, name: &str, attrs: &[(&str, String)], _: Option<String>) {
        self.0.borrow_mut().push((
            name.into(),
            attrs
                .iter()
                .map(|(k, v)| ((*k).into(), v.clone()))
                .collect(),
        ));
    }
}

fn fixture() -> Element {
    rsx! { ConsentFooter {} ConsentFooter {} }
}

#[test]
fn multiple_component_mounts_load_one_canonical_script_and_stylesheet() {
    let captured = Rc::new(CapturedHead::default());
    let document: Rc<dyn Document> = captured.clone();
    let mut dom = VirtualDom::new(fixture);
    dom.insert_any_root_context(Box::new(document));
    dom.rebuild_in_place();
    let head = captured.0.borrow();
    assert_eq!(head.len(), 2, "shared assets must be deduplicated");
    assert!(head.iter().any(|(tag, attrs)| tag == "script"
        && attrs.contains(&("src".into(), CONSENT_FOOTER_SCRIPT.to_string()))));
    assert!(head.iter().any(|(tag, attrs)| tag == "link"
        && attrs.contains(&("href".into(), CONSENT_FOOTER_STYLESHEET.to_string()))));
    assert!(!CONSENT_FOOTER_JS.is_empty());
    assert!(!CONSENT_FOOTER_CSS.is_empty());
}
