//! The Extron Problems section under the layers: it lists every export problem of the open
//! screen, follows edits and undo, and activating a problem selects its layer, reveals it in
//! Layers and brings it into view. Documents without Extron assignments show no section.

use egui::{Event, Modifiers, Pos2, pos2};
use figcraft_doc::NodeId;
use figcraft_engine::Session;
use serde_json::json;

use crate::layers_drag_tests::Ui;

fn settle(u: &mut Ui) {
    for _ in 0..10 {
        u.frame(vec![]);
    }
}

/// The centre of the screen-reader node named `label`, when one is on screen.
fn node(u: &mut Ui, label: &str) -> Option<Pos2> {
    let nodes = u.frame_output(vec![]).platform_output.accesskit_update?.nodes;
    let b = nodes.iter().find(|(_, n)| n.label() == Some(label)).and_then(|(_, n)| n.bounds())?;
    Some(pos2(((b.x0 + b.x1) / 2.0) as f32, ((b.y0 + b.y1) / 2.0) as f32))
}

fn click(u: &mut Ui, p: Pos2) {
    let button = |pressed| Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed, modifiers: Modifiers::NONE };
    u.frame(vec![Event::PointerMoved(p)]);
    u.frame(vec![button(true)]);
    u.frame(vec![button(false)]);
    settle(u);
}

fn demo() -> (Ui, NodeId) {
    let mut s = Session::new();
    s.execute("extron.demo", &json!({})).unwrap();
    s.execute("select.none", &json!({})).unwrap();
    let title = s.doc().unwrap().nodes().find(|n| n.name == "Room title").unwrap().id;
    let mut u = Ui::new(s);
    u.ctx.enable_accesskit();
    settle(&mut u);
    (u, title)
}

#[test]
fn problems_follow_edits_and_a_problem_selects_and_reveals_its_layer() {
    let (mut u, title) = demo();
    assert!(u.app.ui.problems.screen);
    assert_eq!(u.app.ui.problems.items, []);
    assert!(node(&mut u, "Problems: 0").is_some(), "the section shows under the layers");
    // Keeping the list current only reads the document: it is no edit to redraw for.
    let revision = u.app.session.ds().unwrap().revision;
    u.app.run("node.set", json!({"ids": [title], "props": {"x": 40}}));
    let edited = u.app.session.ds().unwrap().revision;
    assert_ne!(edited, revision);
    settle(&mut u);
    assert_eq!(u.app.session.ds().unwrap().revision, edited, "validating after an edit doesn't bump the revision");
    // An invalid edit: the title leaves the panel. The list follows without validating by hand.
    u.app.run("node.set", json!({"ids": [title], "props": {"x": -10}}));
    // Far from the title, so activating the problem has to bring it into view.
    u.app.run("view.set", json!({"cx": 5000, "cy": 5000}));
    settle(&mut u);
    let problem = u.app.ui.problems.items.iter().find(|p| p.node == title).cloned().expect("a problem at the title");
    assert_eq!((problem.layer.as_str(), problem.message.as_str()), ("Room title", "Element must fit inside the 1280 × 800 screen."));
    assert_eq!(u.app.ui.problems.items.len(), 1, "{:?}", u.app.ui.problems.items);
    assert!(node(&mut u, "Problems: 1").is_some());
    // Activating the problem selects the title, reveals its row in Layers and centres it.
    let row = node(&mut u, &problem.label()).expect("the problem's row");
    click(&mut u, row);
    let ds = u.app.session.ds().unwrap();
    assert_eq!(ds.selection, [title]);
    assert_eq!(u.app.ui.layers_seen_sel, [title], "Layers revealed the selected layer");
    let b = ds.doc.bounds_of(&[title]).unwrap();
    let v = &ds.view;
    assert!((v.cx - b.center().x).abs() < 1.0, "the canvas centres the layer: {} vs {}", v.cx, b.center().x);
    assert!(u.app.ui.last_error.is_none(), "{:?}", u.app.ui.last_error);
    // Undo puts the title back; the list empties again.
    u.app.run("edit.undo", json!({}));
    settle(&mut u);
    assert_eq!(u.app.ui.problems.items, []);
}

#[test]
fn an_invalid_screen_stays_blocked_and_names_the_layer_to_fix() {
    let (mut u, title) = demo();
    u.app.run("node.set", json!({"ids": [title], "props": {"x": -10}}));
    let dir = std::env::temp_dir().join(format!("gui-studio-problems-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("blocked.exui");
    let error = u.app.session.execute("extron.exportExui", &json!({"path": path.to_string_lossy()})).unwrap_err().to_string();
    assert!(error.contains(&format!("\"Room title\" (node {title}): Element must fit inside the 1280 × 800 screen.")), "{error}");
    assert!(!path.exists(), "a failed export writes nothing");
    std::fs::remove_dir_all(&dir).ok();
    // Validate from the menu opens a collapsed section and says how many problems there are.
    u.app.ui.problems.collapsed = true;
    crate::extron::validate(&mut u.app);
    assert!(!u.app.ui.problems.collapsed);
    assert_eq!(u.app.ui.toast.as_ref().map(|t| t.0.as_str()), Some("1 Extron problem. Select it in Problems to find its layer."));
}

#[test]
fn documents_without_extron_assignments_show_no_problems_section() {
    let mut u = Ui::new(Session::new());
    u.ctx.enable_accesskit();
    settle(&mut u);
    assert!(!u.app.ui.problems.screen);
    assert!(node(&mut u, "Problems: 0").is_none());
}

/// File ▸ Extron ▸ New touch panel project… asks for the panel first, as GUI Designer's wizard
/// does: a click on the TLP Pro 300M card and Create open a project of its portrait resolution.
#[test]
fn a_new_project_starts_with_the_touch_panel_chosen_in_the_dialog() {
    let mut u = Ui::new(Session::new());
    u.ctx.enable_accesskit();
    crate::menus::invoke(&mut u.app, "extron.newPanel");
    settle(&mut u);
    assert_eq!(u.app.ui.new_project.as_ref().map(|d| d.panel), Some(0));
    let card = node(&mut u, "TLP Pro 300M (portrait)").expect("a card per touch panel");
    click(&mut u, card);
    assert_eq!(u.app.ui.new_project.as_ref().map(|d| d.panel), Some(1));
    let create = node(&mut u, "Create").expect("the Create button");
    click(&mut u, create);
    assert!(u.app.ui.new_project.is_none());
    let doc = u.app.session.doc().unwrap();
    let home = figcraft_engine::gui_studio_extron::panel_frame(doc).unwrap();
    let frame = doc.node(home).unwrap();
    assert_eq!((doc.name.as_str(), frame.width, frame.height), ("Untitled panel", 320.0, 480.0));
    assert!(!u.app.ui.home);
}

/// File ▸ Extron ▸ UI kits…: choosing a kit and Use for this project restyles what is inserted
/// next; a block's Insert places its controls on the page.
#[test]
fn the_ui_kits_window_inserts_blocks_and_switches_the_projects_kit() {
    let mut s = Session::new();
    s.execute("extron.newPanel", &json!({"name": "Huddle Room"})).unwrap();
    let mut u = Ui::new(s);
    u.ctx.enable_accesskit();
    crate::menus::invoke(&mut u.app, "extron.kitsWindow");
    settle(&mut u);
    assert_eq!(u.app.ui.kits_window.as_ref().map(|w| w.kit.as_str()), Some("aurora"));
    let graphite = node(&mut u, "Graphite").expect("a chip per kit");
    click(&mut u, graphite);
    assert_eq!(u.app.ui.kits_window.as_ref().map(|w| w.kit.as_str()), Some("graphite"));
    let use_it = node(&mut u, "Use for this project").expect("Use for this project");
    click(&mut u, use_it);
    assert_eq!(figcraft_engine::gui_studio_extron::project_kit(u.app.session.doc().unwrap()).unwrap().id, "graphite");
    // One of the blocks' Insert buttons: its controls land on the page as they are in the kit.
    let insert = node(&mut u, "Insert").expect("an Insert button per block");
    click(&mut u, insert);
    let screen = figcraft_engine::gui_studio_extron::project(u.app.session.doc().unwrap()).unwrap();
    let names: Vec<_> = screen.pages[0].elements.iter().map(|e| e.name.clone()).collect();
    let kit = figcraft_engine::gui_studio_extron::Kit::built_in().into_iter().find(|k| k.id == "graphite").unwrap();
    let block = kit.blocks.iter().find(|b| b.items.iter().map(|i| i.name.clone()).collect::<Vec<_>>() == names);
    assert!(block.is_some(), "{names:?} is a block of the kit");
    assert!(u.app.ui.kits_window.is_some(), "the window stays open for more");
}
