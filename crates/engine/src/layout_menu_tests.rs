//! GUI Designer's Layout menu on top of Figma's align row: Same width, height and size after the
//! first selected layer, and Align to frame, which aligns each layer to its own frame and spreads
//! the layers of one frame to its edges.

use serde_json::{Value, json};

use crate::{EngineError, NodeId, Session};

fn id(v: &Value) -> NodeId {
    crate::params::id_of(&v["id"]).unwrap()
}

fn rect(s: &mut Session, parent: Option<NodeId>, x: f64, y: f64, w: f64, h: f64) -> NodeId {
    let mut p = json!({"type": "rectangle", "x": x, "y": y, "width": w, "height": h});
    if let Some(parent) = parent {
        p["parent"] = json!(parent);
    }
    id(&s.execute("node.create", &p).unwrap())
}

fn frame(s: &mut Session, x: f64, y: f64, w: f64, h: f64) -> NodeId {
    id(&s.execute("node.create", &json!({"type": "frame", "x": x, "y": y, "width": w, "height": h})).unwrap())
}

fn bounds(s: &Session, n: NodeId) -> [f64; 4] {
    let b = s.doc().unwrap().world_bounds(n).unwrap();
    [b.x0, b.y0, b.width(), b.height()]
}

#[test]
fn same_size_takes_the_first_selected_layers_size_in_one_step() {
    let mut s = Session::new();
    let a = rect(&mut s, None, 0.0, 0.0, 100.0, 40.0);
    let b = rect(&mut s, None, 200.0, 0.0, 60.0, 80.0);
    let c = rect(&mut s, None, 400.0, 0.0, 30.0, 30.0);
    // Selected in click order: b first, so b sets the size.
    s.execute("select.set", &json!({"ids": [b]})).unwrap();
    s.execute("select.add", &json!({"ids": [a, c]})).unwrap();
    let depth = s.undo_depth().0;
    s.execute("arrange.sameWidth", &json!({})).unwrap();
    assert_eq!([bounds(&s, a)[2], bounds(&s, c)[2]], [60.0, 60.0]);
    assert_eq!([bounds(&s, a)[3], bounds(&s, c)[3]], [40.0, 30.0], "heights stay");
    assert_eq!(s.undo_depth().0, depth + 1);
    s.execute("arrange.sameHeight", &json!({"ids": [a, c], "reference": b})).unwrap();
    assert_eq!([bounds(&s, a)[3], bounds(&s, c)[3]], [80.0, 80.0]);
    s.execute("edit.undo", &json!({})).unwrap();
    s.execute("arrange.sameSize", &json!({"ids": [c, a]})).unwrap();
    assert_eq!(bounds(&s, a), [0.0, 0.0, 60.0, 30.0], "c (60 × 30 since Same width) first: a takes its size, keeping its corner");
    assert_eq!(bounds(&s, b), [200.0, 0.0, 60.0, 80.0], "b was not a target");

    let alone = s.execute("arrange.sameSize", &json!({"ids": [a]})).unwrap_err();
    assert!(matches!(alone, EngineError::Disabled(..)), "{alone}");
    assert!(alone.to_string().contains("the first one selected sets the size"), "{alone}");
}

#[test]
fn extron_buttons_take_the_first_ones_size_and_stay_exportable() {
    let mut s = Session::new();
    s.execute("extron.newPanel", &json!({})).unwrap();
    let wide = id(&s.execute("extron.insert", &json!({"kind": "button", "x": 40, "y": 100, "name": "HDMI", "text": "HDMI"})).unwrap());
    s.execute("node.set", &json!({"ids": [wide], "props": {"width": 260, "height": 90}})).unwrap();
    let mute = id(&s.execute("extron.insert", &json!({"kind": "button", "x": 40, "y": 300, "name": "Mute", "text": "Mute"})).unwrap());
    s.execute("arrange.sameSize", &json!({"ids": [wide, mute]})).unwrap();
    assert_eq!(bounds(&s, mute), [40.0, 300.0, 260.0, 90.0]);
    assert_eq!(s.execute("extron.validate", &json!({})).unwrap()["valid"], true);
}

#[test]
fn align_to_frame_aligns_each_layer_to_its_own_frame() {
    let mut s = Session::new();
    let left = frame(&mut s, 0.0, 0.0, 400.0, 300.0);
    let right = frame(&mut s, 600.0, 0.0, 200.0, 200.0);
    let a = rect(&mut s, Some(left), 50.0, 60.0, 100.0, 40.0);
    let b = rect(&mut s, Some(right), 20.0, 30.0, 40.0, 40.0);
    let loose = rect(&mut s, None, 1000.0, 500.0, 10.0, 10.0);
    let doc = s.doc().unwrap();
    assert!(crate::cmd::arrange::can_align(doc, &[a, b], true));
    assert!(!crate::cmd::arrange::can_align(doc, &[loose], true), "a layer on the page has no frame");

    // Without the setting, the two line up with each other.
    s.execute("align.right", &json!({"ids": [a, b]})).unwrap();
    assert_eq!(bounds(&s, a)[0] + 100.0, bounds(&s, b)[0] + 40.0);
    s.execute("edit.undo", &json!({})).unwrap();

    // `to: frame`, or the setting, puts each against its own frame; the loose one stays.
    s.execute("align.right", &json!({"ids": [a, b, loose], "to": "frame"})).unwrap();
    assert_eq!((bounds(&s, a)[0], bounds(&s, b)[0], bounds(&s, loose)[0]), (300.0, 760.0, 1000.0));
    assert_eq!(s.execute("view.alignToFrame", &json!({})).unwrap()["on"], true);
    s.execute("align.vcenter", &json!({"ids": [a, b]})).unwrap();
    assert_eq!((bounds(&s, a)[1], bounds(&s, b)[1]), (130.0, 80.0));
    s.execute("align.top", &json!({"ids": [a, b], "to": "selection"})).unwrap();
    assert_eq!(bounds(&s, a)[1], bounds(&s, b)[1], "`to: selection` overrides the setting");
    let wrong = s.execute("align.left", &json!({"ids": [a], "to": "page"})).unwrap_err();
    assert!(wrong.to_string().contains("`to` must be selection or frame"), "{wrong}");
}

#[test]
fn distribute_to_frame_spreads_the_layers_of_one_frame_to_its_edges() {
    let mut s = Session::new();
    let panel = frame(&mut s, 100.0, 100.0, 1000.0, 400.0);
    let a = rect(&mut s, Some(panel), 300.0, 10.0, 100.0, 50.0);
    let b = rect(&mut s, Some(panel), 100.0, 10.0, 200.0, 50.0);
    s.execute("distribute.horizontal", &json!({"ids": [a, b], "to": "frame"})).unwrap();
    // Two layers: b (leftmost) at the frame's left edge, a at its right edge.
    assert_eq!((bounds(&s, b)[0], bounds(&s, a)[0] + 100.0), (100.0, 1100.0));
    let c = rect(&mut s, Some(panel), 500.0, 10.0, 100.0, 50.0);
    s.execute("distribute.horizontal", &json!({"ids": [a, b, c], "to": "frame"})).unwrap();
    // 1000 − (200 + 100 + 100) = 600 over two gaps.
    let xs: Vec<f64> = [b, c, a].iter().map(|n| bounds(&s, *n)[0]).collect();
    assert_eq!(xs, [100.0, 600.0, 1000.0]);

    let other = frame(&mut s, 1500.0, 0.0, 300.0, 300.0);
    let d = rect(&mut s, Some(other), 0.0, 0.0, 10.0, 10.0);
    let mixed = s.execute("distribute.vertical", &json!({"ids": [a, d], "to": "frame"})).unwrap_err();
    assert!(mixed.to_string().contains("select layers of one frame"), "{mixed}");
}
