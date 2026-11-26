//! Control identity across edits, ported from the eframe prototype
//! (`save_reopen_and_restyle_retains_mute_identity`, `tests/extron_export.rs`): a style-only change to
//! Mute keeps its key, Object ID, Control ID and state roles through reordering, saving and
//! reopening, and a copy or a replacement never inherits them. The GDL comparison runs only with
//! the Extron exporter at hand.

use serde_json::{Value, json};

use gui_studio_extron::{Color, Control, Element, Project};

use crate::{NodeId, Session};

fn id(v: &Value) -> NodeId {
    crate::params::id_of(&v["id"]).unwrap()
}

/// A TLP Pro 1025M project from the Aurora kit's masters: three source buttons, Mute, a volume
/// slider and a title. Returns Mute.
fn room(s: &mut Session) -> NodeId {
    s.execute("extron.newPanel", &json!({"name": "Huddle Room"})).unwrap();
    for (x, text) in [(40, "HDMI"), (260, "USB-C"), (480, "Laptop")] {
        s.execute("extron.insert", &json!({"kind": "button", "x": x, "y": 120, "name": text, "text": text})).unwrap();
    }
    let mute = id(&s.execute("extron.insert", &json!({"kind": "button", "x": 40, "y": 320, "name": "Mute", "text": "Mute"})).unwrap());
    s.execute("extron.insert", &json!({"kind": "slider", "x": 300, "y": 330, "name": "Volume"})).unwrap();
    s.execute("extron.insert", &json!({"kind": "label", "x": 40, "y": 40, "name": "Room title", "text": "Huddle Room"})).unwrap();
    mute
}

fn project(s: &Session) -> Project {
    gui_studio_extron::project(s.doc().unwrap()).unwrap_or_else(|items| panic!("{items:?}"))
}

fn element<'a>(project: &'a Project, name: &str) -> &'a Element {
    project.pages.iter().flat_map(|page| &page.elements).find(|e| e.name == name).unwrap_or_else(|| panic!("no {name}"))
}

/// Mute's state names and fills, Normal, Active and Pressed.
fn states(e: &Element) -> Vec<(String, Color)> {
    let Control::Button { states, .. } = &e.control else { panic!("{} is not a button", e.name) };
    states.iter().map(|state| (state.name.clone(), state.fill)).collect()
}

/// Restyle Mute's Active state on its master: a style-only change to every button of the kit.
fn restyle_active(s: &mut Session, mute: NodeId) {
    let active = gui_studio_extron::control_info(s.doc().unwrap(), mute).unwrap().states[1].1;
    s.execute("node.set", &json!({"ids": [active], "props": {"fills": ["#3778A5"]}})).unwrap();
}

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("gui-studio-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn restyle_reorder_and_reopen_keep_mutes_identity_and_a_copy_or_replacement_never_inherits_it() {
    let mut s = Session::new();
    let mute = room(&mut s);
    let before = element(&project(&s), "Mute").clone();

    // Style only, on the master; then a new layer order.
    restyle_active(&mut s, mute);
    s.execute("arrange.back", &json!({"ids": [mute]})).unwrap();
    let hdmi = s.doc().unwrap().nodes().find(|n| n.name == "HDMI").unwrap().id;
    s.execute("arrange.front", &json!({"ids": [hdmi]})).unwrap();
    let panel = s.doc().unwrap().node(mute).unwrap().parent.unwrap();
    assert_eq!(s.doc().unwrap().node(panel).unwrap().children[0], mute, "Mute is at the back now");

    let dir = scratch("identity");
    let path = dir.join("huddle-room.figcraft").to_string_lossy().to_string();
    s.execute("file.saveAs", &json!({"path": path})).unwrap();
    let mut t = Session::empty();
    t.execute("file.open", &json!({"path": path})).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    let reopened = project(&t);
    let after = element(&reopened, "Mute");
    assert_eq!(
        (&after.identity.key, after.identity.object_id, after.identity.control_id),
        (&before.identity.key, before.identity.object_id, before.identity.control_id)
    );
    let (was, now) = (states(&before), states(after));
    assert_eq!(now.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(), ["Normal", "Active", "Pressed"]);
    assert_eq!((now[0].1, now[2].1), (was[0].1, was[2].1), "Normal and Pressed keep their look");
    assert_eq!(now[1].1, Color([0x37, 0x78, 0xA5]), "Active takes the master's new fill");
    assert_ne!(now[1].1, was[1].1);
    let mute = after.node;

    // A copy carries Mute's assignment but not its identity: it blocks export until assigned,
    // and assigning gives it its own key and IDs.
    let copy = t.execute("edit.duplicate", &json!({"ids": [mute]})).unwrap();
    let copy = crate::params::id_of(&copy["ids"][0]).or_else(|| t.selection().first().copied()).unwrap();
    assert_ne!(copy, mute);
    let checked = t.execute("extron.validate", &json!({})).unwrap();
    assert_eq!(checked["valid"], false);
    let at_copy = checked["diagnostics"].as_array().unwrap().iter().find(|d| d["nodeId"] == json!(copy)).expect("a problem at the copy");
    assert_eq!(at_copy["message"], "Copied Extron assignment needs its own key and IDs.");
    let assigned = t.execute("extron.assign", &json!({"kind": "button", "ids": [copy]})).unwrap();
    let fresh = &assigned["assigned"][0];
    assert_ne!(fresh["objectId"], json!(before.identity.object_id));
    assert_ne!(fresh["controlId"], json!(before.identity.control_id));
    assert_ne!(fresh["key"], json!(before.identity.key));
    t.execute("edit.delete", &json!({"ids": [copy]})).unwrap();

    // Deleting Mute and putting a new button in its place gives new IDs, never Mute's.
    t.execute("edit.delete", &json!({"ids": [mute]})).unwrap();
    let replacement = t.execute("extron.insert", &json!({"kind": "button", "x": 40, "y": 320, "name": "Mute", "text": "Mute"})).unwrap();
    for (field, old) in
        [("objectId", json!(before.identity.object_id)), ("controlId", json!(before.identity.control_id)), ("key", json!(before.identity.key))]
    {
        assert_ne!(replacement[field], old, "{field}");
    }
    assert!(replacement["objectId"].as_u64() > fresh["objectId"].as_u64(), "IDs only go up, past the copy's too");
    assert_eq!(t.execute("extron.validate", &json!({})).unwrap()["valid"], true);
}

/// Build the project's GDL at `path` through the exporter and read Mute's control back with
/// `list-controls`.
fn built_mute(s: &Session, path: &std::path::Path) -> Value {
    let exporter = crate::cmd::extron::find_exporter(s).unwrap();
    let report = gui_studio_extron::build(&project(s), path, &exporter).unwrap();
    assert!(report["checks"].as_str().is_some_and(|c| c.ends_with(" passed")), "{report}");
    let out = std::process::Command::new("dotnet").arg(&exporter.cli).arg("list-controls").arg(path).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let listed: Value = serde_json::from_slice(&out.stdout).unwrap();
    let controls = listed["model"]["Pages"][0]["Controls"].as_array();
    let shown = || listed.to_string().chars().take(2000).collect::<String>();
    controls
        .unwrap_or_else(|| panic!("no controls on the first page: {}", shown()))
        .iter()
        .find(|c| c["Name"] == "Mute")
        .cloned()
        .expect("Mute in the GDL")
}

#[test]
#[ignore = "needs dotnet and the Extron exporter, found as Build GDL finds it (the environment, the preferences or a sibling checkout)"]
fn a_style_only_change_keeps_mutes_gdl_identity() {
    let mut s = Session::new();
    let mute = room(&mut s);
    let dir = scratch("gdl-identity");
    let original = built_mute(&s, &dir.join("huddle-room-v1.gdl"));
    restyle_active(&mut s, mute);
    let revised = built_mute(&s, &dir.join("huddle-room-v2.gdl"));
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(original["Type"], "PBButton");
    for field in ["ObjectId", "ControlId", "Comments", "PressFeedbackState"] {
        assert_eq!(original[field], revised[field], "{field}");
    }
    assert_ne!(original["States"], revised["States"], "the Active state's look changed");
}
