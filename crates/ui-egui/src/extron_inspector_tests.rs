//! The Extron section of the Design panel and the Control ID badges: a button's Control ID is
//! edited in its field, its state switch changes what the canvas shows, a label shows only its
//! Object ID, a page its touch panel, and View ▸ Control ID badges turns on with its key.

use egui::{Key, Pos2, accesskit::Role, pos2};
use figcraft_doc::NodeId;
use figcraft_engine::Session;
use figcraft_engine::gui_studio_extron::{StateRole, control_info};
use serde_json::json;

use crate::field_tests::{field, type_into, undo_steps};
use crate::layers_drag_tests::Ui;

fn settle(u: &mut Ui) {
    for _ in 0..6 {
        u.frame(vec![]);
    }
}

/// The screen-reader nodes with `role` named `label`, and where the first one is.
fn find(u: &mut Ui, role: Role, label: &str) -> Option<Pos2> {
    let nodes = u.frame_output(vec![]).platform_output.accesskit_update?.nodes;
    let b = nodes.iter().find(|(_, n)| n.role() == role && n.label() == Some(label)).and_then(|(_, n)| n.bounds())?;
    Some(pos2(((b.x0 + b.x1) / 2.0) as f32, ((b.y0 + b.y1) / 2.0) as f32))
}

fn project() -> (Ui, NodeId, NodeId, NodeId) {
    let mut s = Session::new();
    let panel = figcraft_engine::params::id_of(&s.execute("extron.newPanel", &json!({})).unwrap()["panel"]).unwrap();
    let id = |v: serde_json::Value| figcraft_engine::params::id_of(&v["id"]).unwrap();
    let label = id(s.execute("extron.insert", &json!({"kind": "label", "text": "Room", "x": 40, "y": 40})).unwrap());
    let mute = id(s.execute("extron.insert", &json!({"kind": "button", "name": "Mute", "text": "Mute"})).unwrap());
    let mut u = Ui::new(s);
    u.ctx.enable_accesskit();
    settle(&mut u);
    (u, panel, label, mute)
}

#[test]
fn a_buttons_control_id_is_edited_in_its_field_and_its_switch_picks_the_state_shown() {
    let (mut u, _, _, mute) = project();
    // The page is 100 and the label 101.
    assert_eq!(field(&mut u, "Object ID").1, "102");
    assert_eq!(field(&mut u, "Control ID").1, "102");

    let steps = undo_steps(&u);
    type_into(&mut u, "Control ID", "250", Key::Enter);
    let doc = u.app.session.doc().unwrap();
    assert_eq!(control_info(doc, mute).unwrap().control_id, Some(250));
    assert_eq!(undo_steps(&u), steps + 1, "one undo step");
    assert!(u.app.ui.last_error.is_none(), "{:?}", u.app.ui.last_error);

    let active = find(&mut u, Role::Button, "Active").expect("the Active state");
    u.button(active, true);
    u.button(active, false);
    settle(&mut u);
    let doc = u.app.session.doc().unwrap();
    assert_eq!(control_info(doc, mute).unwrap().shown, Some(StateRole::Feedback));
    assert!(u.app.ui.last_error.is_none(), "{:?}", u.app.ui.last_error);
}

#[test]
fn a_label_shows_only_its_object_id_and_a_page_its_touch_panel() {
    let (mut u, panel, label, _) = project();
    u.app.run("select.set", json!({"ids": [label]}));
    settle(&mut u);
    assert_eq!(field(&mut u, "Object ID").1, "101");
    assert!(find(&mut u, Role::SpinButton, "Control ID").is_none(), "a label has no Control ID");

    u.app.run("select.set", json!({"ids": [panel]}));
    settle(&mut u);
    let nodes = u.frame_output(vec![]).platform_output.accesskit_update.unwrap().nodes;
    assert!(nodes.iter().any(|(_, n)| n.label().or(n.value()).is_some_and(|l| l == "TLP Pro 1025M · 1280 × 800")), "the page shows its touch panel");
}

#[test]
fn control_id_badges_turn_on_with_their_key_and_draw_with_the_page() {
    let (mut u, ..) = project();
    assert_eq!(crate::shortcuts::shortcut("view.controlIds"), Some("Alt+I"));
    assert!(!u.app.session.ds().unwrap().view.control_ids);
    u.app.run("view.controlIds", json!({}));
    settle(&mut u);
    assert!(u.app.session.ds().unwrap().view.control_ids);
    assert!(u.app.ui.last_error.is_none(), "{:?}", u.app.ui.last_error);
}
