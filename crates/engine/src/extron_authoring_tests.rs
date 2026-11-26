use serde_json::json;

use crate::{EngineError, Session};

fn id(value: &serde_json::Value) -> crate::NodeId {
    crate::params::id_of(&value["id"]).unwrap()
}

#[test]
fn a_new_panel_takes_controls_as_undoable_edits_and_validates() {
    let mut s = Session::new();
    let opened = s.execute("extron.newPanel", &json!({"name": "Huddle Room"})).unwrap();
    assert_eq!(s.doc().unwrap().name, "Huddle Room");
    assert_eq!(s.selection(), vec![crate::params::id_of(&opened["panel"]).unwrap()]);
    assert_eq!(s.undo_depth(), (0, 0));

    let mute = s.execute("extron.insert", &json!({"kind": "button", "x": 60, "y": 300, "name": "Mute", "text": "Mute"})).unwrap();
    assert_eq!((mute["objectId"].as_u64(), mute["controlId"].as_u64(), mute["key"].as_str()), (Some(101), Some(101), Some("studio.element-101")));
    assert_eq!(s.selection(), vec![id(&mute)]);
    let label = s.execute("extron.insert", &json!({"kind": "label", "text": "Huddle Room", "select": false})).unwrap();
    assert_eq!(label["controlId"], serde_json::Value::Null);
    assert_eq!(s.selection(), vec![id(&mute)]);
    assert_eq!(s.undo_depth().0, 2);

    let valid = s.execute("extron.validate", &json!({})).unwrap();
    assert_eq!((valid["valid"].as_bool(), valid["elements"].as_u64()), (Some(true), Some(2)), "{valid}");

    s.execute("edit.undo", &json!({})).unwrap();
    s.execute("edit.undo", &json!({})).unwrap();
    let empty = s.execute("extron.validate", &json!({})).unwrap();
    assert_eq!(empty["diagnostics"][0]["message"], "A screen needs 1–64 elements.");
    s.execute("edit.redo", &json!({})).unwrap();
    assert_eq!(s.execute("extron.validate", &json!({})).unwrap()["valid"], true);
    // The counter came back with the redo, so the next control still gets a fresh ID.
    assert_eq!(s.execute("extron.insert", &json!({"kind": "shape"})).unwrap()["objectId"], 102);
}

#[test]
fn assign_turns_selected_panel_layers_into_controls_and_reports_the_layer_it_refuses() {
    let mut s = Session::new();
    let panel = crate::params::id_of(&s.execute("extron.newPanel", &json!({})).unwrap()["panel"]).unwrap();
    let text = id(&s
        .execute("node.create", &json!({"type": "text", "parent": panel, "x": 60, "y": 60, "width": 300, "height": 40, "text": "Hint"}))
        .unwrap());
    let assigned = s.execute("extron.assign", &json!({"kind": "label"})).unwrap();
    assert_eq!(assigned["assigned"][0]["id"], json!(text));
    assert_eq!(assigned["assigned"][0]["objectId"], 101);

    let undo = s.undo_depth().0;
    let refused = s.execute("extron.assign", &json!({"kind": "shape", "ids": [text]})).unwrap_err();
    assert!(refused.to_string().contains(&format!("node {text}: A shape must be a rectangle.")), "{refused}");
    assert_eq!(s.undo_depth().0, undo);

    let unknown = s.execute("extron.insert", &json!({"kind": "dial"})).unwrap_err();
    assert!(unknown.to_string().contains("Unknown control kind \"dial\""), "{unknown}");
    let offset = s.execute("extron.insert", &json!({"kind": "label", "x": "left"})).unwrap_err();
    assert!(offset.to_string().contains("`x` must be a number"), "{offset}");
    assert!(matches!(s.execute("extron.insert", &json!({"kind": "label", "colour": "red"})), Err(EngineError::BadParams { .. })));

    s.execute("file.new", &json!({})).unwrap();
    let plain = s.execute("extron.insert", &json!({"kind": "label"})).unwrap_err();
    assert!(plain.to_string().contains("no Extron panel"), "{plain}");
}

#[test]
fn a_project_targets_the_chosen_touch_panel_and_takes_more_pages() {
    let mut s = Session::new();
    let listed = s.execute("extron.profiles", &json!({})).unwrap();
    let ids: Vec<_> = listed["profiles"].as_array().unwrap().iter().map(|p| p["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["tlp-pro-1025m", "tlp-pro-300m-portrait"]);
    let unknown = s.execute("extron.newPanel", &json!({"profile": "tlp-pro-2000"})).unwrap_err();
    assert!(unknown.to_string().contains("Unknown touch panel \"tlp-pro-2000\""), "{unknown}");

    let home = crate::params::id_of(&s.execute("extron.newPanel", &json!({"name": "Lectern", "profile": "tlp-pro-300m-portrait"})).unwrap()["panel"])
        .unwrap();
    assert_eq!(s.doc().unwrap().node(home).unwrap().width, 320.0);
    let page = s.execute("extron.addPage", &json!({"name": "Audio"})).unwrap();
    assert_eq!((page["name"].as_str(), page["objectId"].as_u64(), page["key"].as_str()), (Some("Audio"), Some(101), Some("page.101")));
    let audio = id(&page);
    assert_eq!(s.selection(), vec![audio]);
    let node = s.doc().unwrap().node(audio).unwrap();
    assert_eq!((node.width, node.height, node.transform.as_coeffs()[4]), (320.0, 480.0, 480.0));

    // With the new page selected, a control goes on it; `panel` names another page.
    let mute = id(&s.execute("extron.insert", &json!({"kind": "button", "text": "Mute"})).unwrap());
    assert_eq!(gui_studio_extron::panel_of(s.doc().unwrap(), mute), Some(audio));
    let label = id(&s.execute("extron.insert", &json!({"kind": "label", "panel": home})).unwrap());
    assert_eq!(gui_studio_extron::panel_of(s.doc().unwrap(), label), Some(home));
    let valid = s.execute("extron.validate", &json!({})).unwrap();
    assert_eq!(valid["valid"], true, "{valid}");

    s.execute("edit.undo", &json!({})).unwrap();
    s.execute("edit.undo", &json!({})).unwrap();
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(gui_studio_extron::panels(s.doc().unwrap()), vec![home]);
    let half = s.execute("extron.addPage", &json!({"x": 10})).unwrap_err();
    assert!(half.to_string().contains("Give both `x` and `y`"), "{half}");
}

#[test]
fn the_artboard_tool_places_a_page_where_it_is_pressed_and_dragged() {
    use crate::{PointerEvent, PointerKind, ToolId};
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    s.execute("tool.artboard", &json!({})).unwrap();
    let refused = s.pointer(&PointerEvent::at(PointerKind::Down, 0.0, 0.0, 1.0)).unwrap_err();
    assert!(refused.to_string().contains("Start with a new touch panel project. The Artboard tool adds pages to it."), "{refused}");

    let home = crate::params::id_of(&s.execute("extron.newPanel", &json!({"name": "Huddle Room"})).unwrap()["panel"]).unwrap();
    s.execute("tool.artboard", &json!({})).unwrap();
    assert_eq!(s.tool.tool, ToolId::Artboard);
    // A click: the page's top-left corner where it was pressed (snapped to whole pixels).
    s.pointer(&PointerEvent::at(PointerKind::Down, 1500.4, 20.0, 1.0)).unwrap();
    s.pointer(&PointerEvent::at(PointerKind::Up, 1500.4, 20.0, 1.0)).unwrap();
    let panels = gui_studio_extron::panels(s.doc().unwrap());
    assert_eq!(panels.len(), 2);
    let clicked = panels[1];
    let node = s.doc().unwrap().node(clicked).unwrap();
    assert_eq!((node.name.as_str(), node.width, node.height), ("Page 2", 1280.0, 800.0));
    assert_eq!(node.transform.as_coeffs()[4..], [1500.0, 20.0]);
    assert_eq!(s.selection(), vec![clicked]);
    assert_eq!(s.tool.tool, ToolId::Move);
    assert_eq!(s.undo_depth().0, 1);

    // A drag: the page follows the pointer and lands where the button comes up, as one undo step.
    s.execute("tool.artboard", &json!({})).unwrap();
    s.pointer(&PointerEvent::at(PointerKind::Down, 0.0, 1000.0, 1.0)).unwrap();
    s.pointer(&PointerEvent::at(PointerKind::Move, 100.0, 1100.0, 1.0)).unwrap();
    s.pointer(&PointerEvent::at(PointerKind::Move, 200.0, 1200.0, 1.0)).unwrap();
    s.pointer(&PointerEvent::at(PointerKind::Up, 200.0, 1200.0, 1.0)).unwrap();
    let panels = gui_studio_extron::panels(s.doc().unwrap());
    assert_eq!(panels.len(), 3);
    assert_eq!(s.doc().unwrap().node(panels[2]).unwrap().transform.as_coeffs()[4..], [200.0, 1200.0]);
    assert_eq!(s.doc().unwrap().node(panels[2]).unwrap().name, "Page 3");
    assert_eq!(s.undo_depth().0, 2);
    assert_eq!(gui_studio_extron::panel_frame(s.doc().unwrap()).unwrap(), home);

    // Enter with the tool adds a page right of everything.
    s.execute("tool.artboard", &json!({})).unwrap();
    s.execute("tool.place", &json!({})).unwrap();
    let panels = gui_studio_extron::panels(s.doc().unwrap());
    assert_eq!(s.doc().unwrap().node(panels[3]).unwrap().transform.as_coeffs()[4..], [1500.0 + 1280.0 + 160.0, 0.0]);
}

fn kit_file(dir: &std::path::Path, id: &str, name: &str) -> std::path::PathBuf {
    let mut kit = serde_json::to_value(gui_studio_extron::Kit::default_kit().unwrap()).unwrap();
    kit["id"] = id.into();
    kit["name"] = name.into();
    kit["background"] = "#202020".into();
    let path = dir.join(format!("{id}.guikit"));
    std::fs::write(&path, serde_json::to_vec(&kit).unwrap()).unwrap();
    path
}

#[test]
fn kits_are_built_in_or_installed_from_files_and_style_new_projects_and_blocks() {
    let root = std::env::temp_dir().join(format!("gui-studio-kits-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    struct Files(std::path::PathBuf);
    impl Files {
        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }
    impl Drop for Files {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let files = Files(root);
    let mut s = Session::new();
    s.kits.dir = Some(files.path().join("UI Kits"));
    let listed = s.execute("extron.kits", &json!({})).unwrap();
    let ids: Vec<_> = listed["kits"].as_array().unwrap().iter().map(|k| k["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["aurora", "daylight", "graphite"]);
    assert_eq!(listed["kits"][0]["builtIn"], true);
    assert_eq!(listed["kits"][0]["blocks"][1]["name"], "Source selection");
    assert_eq!(listed["project"], serde_json::Value::Null);

    // Installing copies the file into the kits folder; the same id again replaces it.
    let source = kit_file(files.path(), "studio-dark", "Studio dark");
    let installed = s.execute("extron.installKit", &json!({"path": source})).unwrap();
    assert_eq!((installed["id"].as_str(), installed["replaced"].as_bool()), (Some("studio-dark"), Some(false)));
    assert!(files.path().join("UI Kits/studio-dark.json").is_file());
    assert_eq!(s.execute("extron.installKit", &json!({"path": source})).unwrap()["replaced"], true);
    let clash = s.execute("extron.installKit", &json!({"path": kit_file(files.path(), "aurora", "Mine")})).unwrap_err();
    assert!(clash.to_string().contains("built-in kit's id"), "{clash}");
    std::fs::write(files.path().join("UI Kits/broken.json"), b"{}").unwrap();
    let listed = s.execute("extron.kits", &json!({})).unwrap();
    assert_eq!(listed["kits"][3]["id"], "studio-dark");
    assert_eq!(listed["kits"][3]["builtIn"], false);
    assert!(listed["problems"][0]["file"].as_str().unwrap().ends_with("broken.json"));

    // A project in that kit takes its background; a block in another kit keeps that kit's look.
    let home = crate::params::id_of(&s.execute("extron.newPanel", &json!({"kit": "studio-dark"})).unwrap()["panel"]).unwrap();
    assert_eq!(s.execute("extron.kits", &json!({})).unwrap()["project"], "studio-dark");
    let fill = |s: &Session, id| s.doc().unwrap().node(id).unwrap().fills[0].kind.clone();
    let dark = fill(&s, home);
    let block = s.execute("extron.insertBlock", &json!({"block": "Power", "kit": "graphite", "x": 40, "y": 40})).unwrap();
    assert_eq!(block["ids"].as_array().unwrap().len(), 2);
    assert_eq!(s.selection().len(), 2);
    assert_eq!(s.undo_depth().0, 1);
    s.execute("extron.useKit", &json!({"id": "daylight"})).unwrap();
    assert_eq!(s.execute("extron.kits", &json!({})).unwrap()["project"], "daylight");
    assert_eq!(fill(&s, home), dark, "the pages keep their background");
    assert_eq!(s.execute("extron.validate", &json!({})).unwrap()["valid"], true);
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(s.execute("extron.kits", &json!({})).unwrap()["project"], "studio-dark");

    let missing = s.execute("extron.insertBlock", &json!({"block": "Power", "kit": "nope"})).unwrap_err();
    assert!(missing.to_string().contains("No UI kit \"nope\" is installed."), "{missing}");
    s.execute("extron.removeKit", &json!({"id": "studio-dark"})).unwrap();
    assert!(!files.path().join("UI Kits/studio-dark.json").exists());
    let builtin = s.execute("extron.removeKit", &json!({"id": "aurora"})).unwrap_err();
    assert!(builtin.to_string().contains("Built-in kits can't be removed."), "{builtin}");
    // The project still has its copy of the removed kit.
    assert_eq!(s.execute("extron.kits", &json!({})).unwrap()["project"], "studio-dark");
}

/// The solid fill of layer `id`, as `[r, g, b, a]`.
fn solid(s: &Session, id: crate::NodeId) -> [u8; 4] {
    let n = s.doc().unwrap().node(id).unwrap();
    let figcraft_doc::color::PaintKind::Solid { color } = n.fills[0].kind else { panic!("a solid fill") };
    color.to_rgba8()
}

#[test]
fn editing_a_master_restyles_every_control_and_undo_brings_the_look_back() {
    let mut s = Session::new();
    s.execute("extron.newPanel", &json!({"name": "Huddle Room"})).unwrap();
    let mut buttons = Vec::new();
    for (x, text) in [(40, "HDMI"), (260, "USB-C"), (480, "Mute")] {
        buttons.push(id(&s.execute("extron.insert", &json!({"kind": "button", "x": x, "y": 100, "name": text, "text": text})).unwrap()));
    }
    s.execute("extron.insertBlock", &json!({"block": "Volume", "x": 40, "y": 400})).unwrap();
    let doc = s.doc().unwrap();
    let masters: Vec<_> = doc.nodes().filter(|n| n.plugin_data.contains_key(gui_studio_extron::MASTER_KEY)).map(|n| n.id).collect();
    assert_eq!(masters.len(), 2, "one button and one slider master");
    let set = masters.iter().copied().find(|m| doc.node(*m).is_some_and(|n| matches!(n.kind, figcraft_doc::NodeKind::ComponentSet { .. }))).unwrap();
    let normal = doc.node(set).unwrap().children[0];
    let original = solid(&s, buttons[0]);
    assert_eq!(s.execute("extron.validate", &json!({})).unwrap()["valid"], true);

    // One edit to the master restyles every button on the canvas; captions and IDs stay.
    s.execute("node.set", &json!({"ids": [normal], "props": {"fills": ["#C82828"]}})).unwrap();
    for (button, text) in buttons.iter().zip(["HDMI", "USB-C", "Mute"]) {
        assert_eq!(solid(&s, *button), [0xC8, 0x28, 0x28, 0xFF], "{text} follows the master");
        let doc = s.doc().unwrap();
        let figcraft_doc::NodeKind::Text(data) = &doc.node(doc.node(*button).unwrap().children[0]).unwrap().kind else { panic!("a caption") };
        assert_eq!(data.characters, text);
    }
    let valid = s.execute("extron.validate", &json!({})).unwrap();
    assert_eq!((valid["valid"].as_bool(), valid["elements"].as_u64()), (Some(true), Some(6)), "{valid}");

    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(solid(&s, buttons[2]), original);
}
