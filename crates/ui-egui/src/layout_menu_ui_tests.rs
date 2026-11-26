//! The alignment row's menu for layers: Align to frame and the sizes of the first selected layer,
//! from the Design panel.

use egui::Pos2;
use egui::accesskit::Node;
use figcraft_doc::NodeId;
use figcraft_engine::Session;
use serde_json::json;

use crate::layers_drag_tests::Ui;

fn tree(u: &mut Ui) -> Vec<Node> {
    u.ctx.enable_accesskit();
    u.frame(vec![]);
    let out = u.frame_output(vec![]);
    out.platform_output.accesskit_update.expect("an AccessKit update").nodes.into_iter().map(|(_, n)| n).collect()
}

/// Click the first screen-reader node whose name starts with `label`.
fn click(u: &mut Ui, label: &str) {
    let nodes = tree(u);
    let n = nodes.iter().find(|n| n.label().is_some_and(|l| l.starts_with(label))).unwrap_or_else(|| panic!("no {label:?}"));
    let b = n.bounds().expect("bounds");
    let p = Pos2::new(((b.x0 + b.x1) / 2.0) as f32, ((b.y0 + b.y1) / 2.0) as f32);
    u.button(p, true);
    u.button(p, false);
    for _ in 0..3 {
        u.frame(vec![]);
    }
}

fn width(u: &Ui, n: NodeId) -> f64 {
    u.app.session.doc().unwrap().node(n).unwrap().width
}

#[test]
fn the_alignment_menu_turns_on_align_to_frame_and_sizes_layers_like_the_first_selected() {
    let mut s = Session::new();
    let id = |v: serde_json::Value| NodeId(v["id"].as_u64().unwrap());
    let frame = id(s.execute("node.create", &json!({"type": "frame", "x": 0, "y": 0, "width": 600, "height": 400})).unwrap());
    let a = id(s.execute("node.create", &json!({"type": "rectangle", "parent": frame, "x": 40, "y": 40, "width": 120, "height": 40})).unwrap());
    let b = id(s.execute("node.create", &json!({"type": "rectangle", "parent": frame, "x": 40, "y": 200, "width": 60, "height": 60})).unwrap());
    s.execute("select.set", &json!({"ids": [a, b]})).unwrap();
    let mut u = Ui::new(s);
    u.frame(vec![]);

    click(&mut u, "Alignment options");
    click(&mut u, "Align to frame");
    assert!(u.app.session.ds().unwrap().view.align_to_frame);
    click(&mut u, "Alignment options");
    click(&mut u, "Same width");
    assert_eq!(width(&u, b), 120.0, "b takes the width of a, selected first");
    assert!(u.app.ui.last_error.is_none(), "{:?}", u.app.ui.last_error);

    // With Align to frame on, Align right puts each against the frame's right edge.
    click(&mut u, "Align right");
    let doc = u.app.session.doc().unwrap();
    assert_eq!((doc.world_bounds(a).unwrap().x1, doc.world_bounds(b).unwrap().x1), (600.0, 600.0));
}
