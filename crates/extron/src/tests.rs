use figcraft_doc::NodeKind;
use serde_json::Value;
use zip::ZipArchive;

use crate::model::Control;
use crate::{demo, package, project};

fn screen(doc: &figcraft_doc::Document) -> Value {
    let bytes = package(&project(doc).unwrap()).unwrap();
    let mut archive = ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    serde_json::from_reader(archive.by_name("screen.json").unwrap()).unwrap()
}

#[test]
fn demo_preserves_native_identity_after_an_edit() {
    let mut doc = demo().unwrap();
    let before = screen(&doc);
    let title = doc.nodes().find(|node| node.name == "Room title").unwrap().id;
    let NodeKind::Text(data) = &mut doc.node_mut(title).unwrap().kind else { panic!("title must be editable text") };
    data.characters = "Main Meeting Room".into();
    let after = screen(&doc);
    assert_eq!(before["page"], after["page"]);
    for (prev, next) in before["elements"].as_array().unwrap().iter().zip(after["elements"].as_array().unwrap()) {
        assert_eq!(prev["objectId"], next["objectId"]);
        assert_eq!(prev["controlId"], next["controlId"]);
        assert_eq!(prev["key"], next["key"]);
    }
    assert_eq!(after["elements"][1]["text"], "Main Meeting Room");
    let mute = &after["elements"][13];
    assert_eq!(mute["objectId"], 8114);
    assert_eq!(mute["controlId"], 8114);
    assert_eq!(mute["states"][1]["role"], "feedback");
    assert_eq!(mute["press"], "Pressed");
}

#[test]
fn canvas_text_is_px_and_exports_as_points() {
    let mut doc = demo().unwrap();
    let size = |doc: &figcraft_doc::Document, name: &str| match &doc.nodes().find(|node| node.name == name).unwrap().kind {
        NodeKind::Text(data) => data.style.font_size,
        _ => panic!("{name} must be text"),
    };
    // 50 pt on the panel is 66.67 px on the canvas; a 21 pt button caption is 28 px.
    assert!((size(&doc, "Room title") - 200.0 / 3.0).abs() < 1e-9);
    assert_eq!(size(&doc, "Caption"), 28.0);
    let exported = screen(&doc);
    assert_eq!(exported["elements"][1]["font"]["size"], 50);
    assert_eq!(exported["elements"][13]["font"]["size"], 21);

    let title = doc.nodes().find(|node| node.name == "Room title").unwrap().id;
    let resize = |doc: &mut figcraft_doc::Document, px: f64| {
        let NodeKind::Text(data) = &mut doc.node_mut(title).unwrap().kind else { panic!("title must be editable text") };
        data.style.font_size = px;
    };
    resize(&mut doc, 24.0);
    assert_eq!(screen(&doc)["elements"][1]["font"]["size"], 18);
    resize(&mut doc, 4.0); // 3 pt is below GUI Designer's 4 pt minimum
    let errors = project(&doc).unwrap_err();
    assert!(errors.iter().any(|e| e.node_id == title && e.message.contains("4–200 pt")), "{errors:?}");
}

#[test]
fn reports_copied_identity_and_unsupported_geometry_at_each_layer() {
    let mut doc = demo().unwrap();
    let panel = project(&doc).unwrap().pages[0].node;
    let title = doc.nodes().find(|node| node.name == "Room title").unwrap().id;
    let copy = doc.duplicate_into(title, panel, None).unwrap();
    doc.node_mut(title).unwrap().transform = figcraft_geom::Affine::rotate(0.1);
    let errors = project(&doc).unwrap_err();
    assert!(errors.iter().any(|item| item.node_id == copy && item.message.contains("Copied Extron assignment")));
    assert!(errors.iter().any(|item| item.node_id == title && item.layer == "Room title" && item.message.contains("Rotated")));
}

#[test]
fn a_layer_moved_off_the_screen_names_the_screen_bounds() {
    let mut doc = demo().unwrap();
    let title = doc.nodes().find(|node| node.name == "Room title").unwrap().id;
    doc.node_mut(title).unwrap().transform = figcraft_geom::Affine::translate((-10.0, 20.0));
    let errors = project(&doc).unwrap_err();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!((errors[0].node_id, errors[0].layer.as_str()), (title, "Room title"));
    assert_eq!(errors[0].message, "Element must fit inside the 1280 × 800 screen.");
    assert_eq!(errors[0].to_string(), format!("\"Room title\" (node {title}): Element must fit inside the 1280 × 800 screen."));
}

#[test]
fn shared_control_ids_follow_gui_designer_rules_and_name_the_first_owner() {
    let mut screen = project(&demo().unwrap()).unwrap();
    let index = |screen: &crate::Project, pick: fn(&Control) -> bool, nth: usize| {
        screen.pages[0].elements.iter().enumerate().filter(|(_, item)| pick(&item.control)).nth(nth).unwrap().0
    };
    let button = |c: &Control| matches!(c, Control::Button { .. });
    let (first, second) = (index(&screen, button, 0), index(&screen, button, 1));
    let slider = index(&screen, |c| matches!(c, Control::Slider { .. }), 0);
    let label = index(&screen, |c| matches!(c, Control::Label { .. }), 0);
    let shared = screen.pages[0].elements[first].identity.control_id;
    for item in [second, slider, label] {
        screen.pages[0].elements[item].identity.control_id = shared;
    }
    let owner = screen.pages[0].elements[first].name.clone();
    let errors = screen.diagnostics();
    let message = |item: usize| {
        let node = screen.pages[0].elements[item].node;
        let found: Vec<_> = errors.iter().filter(|e| e.node_id == node).collect();
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].layer, screen.pages[0].elements[item].name);
        found[0].message.clone()
    };
    let id = shared.unwrap();
    // Compatible buttons may share an ID in GUI Designer; EXUI/5 export still needs unique IDs.
    assert_eq!(
        message(second),
        format!("Control ID {id} is shared with \"{owner}\". EXUI/5 export needs a unique Control ID for each control on the screen.")
    );
    assert_eq!(message(slider), format!("Control ID {id} is already used by \"{owner}\", a button. Only controls of the same type can share IDs."));
    assert_eq!(message(label), "A label cannot have a Control ID.");
    assert_eq!(errors.len(), 3, "{errors:?}");
}
