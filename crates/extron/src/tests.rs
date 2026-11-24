use figcraft_doc::NodeKind;
use serde_json::Value;
use zip::ZipArchive;

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
    let panel = project(&doc).unwrap().page;
    let title = doc.nodes().find(|node| node.name == "Room title").unwrap().id;
    let copy = doc.duplicate_into(title, panel, None).unwrap();
    doc.node_mut(title).unwrap().transform = figcraft_geom::Affine::rotate(0.1);
    let errors = project(&doc).unwrap_err();
    assert!(errors.iter().any(|item| item.node_id == copy && item.message.contains("Copied Extron assignment")));
    assert!(errors.iter().any(|item| item.node_id == title && item.message.contains("Rotated")));
}
