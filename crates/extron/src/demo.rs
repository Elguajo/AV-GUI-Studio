//! Editable Aurora Meeting Room example, with the original GUI Studio Object and Control IDs.

use figcraft_color::{Color as CanvasColor, Paint};
use figcraft_doc::{Document, Node, NodeId, NodeKind, TextAlignH, TextAlignV, TextAutoResize, TextData};
use figcraft_geom::Affine;

use crate::metadata::{Kind, Metadata};
use crate::model::{Color, Control, HEIGHT, PROFILE, StateRole, WIDTH};

const INK: Color = Color([230, 244, 243]);
const MINT: Color = Color([122, 234, 188]);
const DIM: Color = Color([148, 176, 184]);

pub fn document() -> Result<Document, String> {
    let name = "Aurora Meeting Room";
    let mut doc = Document::new(name);
    let root = doc.current_page.ok_or("Missing design page.")?;
    let mut frame = node(&mut doc, NodeKind::Frame, "Home", [0, 0, WIDTH, HEIGHT]);
    fill(&mut frame, Color([10, 27, 35]));
    let mut data = Metadata::new(&frame, Kind::Page);
    data.profile = Some(PROFILE.into());
    data.project_name = Some(name.into());
    data.object_id = Some(100);
    data.key = Some("page.home".into());
    assign(&mut frame, &data)?;
    let panel = insert(&mut doc, frame, root)?;

    let mut variant_y = 0;
    for (index, (name, rect, control)) in screen().into_iter().enumerate() {
        let object_id = 8101 + u16::try_from(index).map_err(|err| err.to_string())?;
        let (mut layer, kind) = match control.clone() {
            Control::Label { text, color, size } => (text_layer(&mut doc, name, rect, text, color, size, TextAlignH::Left), Kind::Label),
            Control::Shape { fill: color, radius } => {
                let mut shape = node(&mut doc, NodeKind::Rectangle, name, rect);
                fill(&mut shape, color);
                shape.corners.radii = [f64::from(radius); 4];
                (shape, Kind::Shape)
            }
            Control::Button { states, size } => {
                let appearance = ButtonVariants { name, rect, states, size, y: variant_y };
                let layer = button(&mut doc, root, appearance)?;
                variant_y += rect[3] + 96;
                (layer, Kind::Button)
            }
            Control::Slider { .. } => (node(&mut doc, NodeKind::Frame, name, rect), Kind::Slider),
        };
        let mut data = Metadata::new(&layer, kind);
        data.key = Some(format!("studio.element-{object_id}"));
        data.object_id = Some(object_id);
        data.control_id = matches!(kind, Kind::Button | Kind::Slider).then_some(object_id);
        if let Control::Slider { min, max, .. } = control {
            data.min = Some(min);
            data.max = Some(max);
            layer.fills.clear();
        }
        assign(&mut layer, &data)?;
        let id = insert(&mut doc, layer, panel)?;
        if let Control::Slider { fill, track, thumb, .. } = control {
            slider(&mut doc, id, rect, [track, fill, thumb])?;
        }
    }
    Ok(doc)
}

fn node(doc: &mut Document, kind: NodeKind, name: &str, rect: [i32; 4]) -> Node {
    let [x, y, w, h] = rect.map(f64::from);
    let mut node = Node::new(doc.alloc_id(), kind, name, w, h);
    node.transform = Affine::translate((x, y));
    node
}

fn insert(doc: &mut Document, layer: Node, parent: NodeId) -> Result<NodeId, String> {
    doc.insert(layer, parent, None).map_err(|err| err.to_string())
}

fn assign(node: &mut Node, data: &Metadata) -> Result<(), String> {
    data.write(node).map_err(|err| err.to_string())
}

fn fill(node: &mut Node, color: Color) {
    let [r, g, b] = color.0;
    node.fills = vec![Paint::solid(CanvasColor::from_rgb8_const(r, g, b))];
}

fn text_layer(doc: &mut Document, name: &str, rect: [i32; 4], caption: String, color: Color, size: u8, align: TextAlignH) -> Node {
    let mut data = TextData::new(caption);
    data.style.font_family = "Arial".into();
    data.style.font_style = "Regular".into();
    data.style.font_size = f64::from(size);
    data.auto_resize = TextAutoResize::None;
    data.align_h = align;
    data.align_v = TextAlignV::Middle;
    let mut layer = node(doc, NodeKind::Text(data), name, rect);
    fill(&mut layer, color);
    layer
}

struct ButtonVariants<'a> {
    name: &'a str,
    rect: [i32; 4],
    states: [crate::model::ButtonState; 3],
    size: u8,
    y: i32,
}

fn button(doc: &mut Document, root: NodeId, variants: ButtonVariants<'_>) -> Result<Node, String> {
    let ButtonVariants { name, rect, states, size, y } = variants;
    let [_, _, w, h] = rect;
    let set = node(doc, NodeKind::ComponentSet { description: String::new() }, name, [WIDTH + 96, y, 3 * (w + 12), h + 24]);
    let set_id = insert(doc, set, root)?;
    let mut first = None;
    for (role, state) in StateRole::ALL.into_iter().zip(states) {
        let variant = NodeKind::Component { description: String::new(), props: Default::default(), prop_order: Vec::new() };
        let x = 12 + role.id() as i32 * (w + 12);
        let mut layer = node(doc, variant, &format!("State={}", state.name), [x, 12, w, h]);
        fill(&mut layer, state.fill);
        let mut data = Metadata::new(&layer, Kind::State);
        data.role = Some(role);
        data.state_id = Some(role.id());
        data.state_name = Some(state.name);
        assign(&mut layer, &data)?;
        let id = insert(doc, layer, set_id)?;
        first.get_or_insert((id, state.fill));
        let caption = text_layer(doc, "Caption", [0, 0, w, h], state.text, state.color, size, TextAlignH::Center);
        insert(doc, caption, id)?;
    }
    let (component, color) = first.ok_or("Missing Normal variant.")?;
    let mut layer = node(doc, NodeKind::Instance { component, props: Default::default() }, name, rect);
    layer.source = Some(component);
    fill(&mut layer, color);
    Ok(layer)
}

fn slider(doc: &mut Document, id: NodeId, rect: [i32; 4], colors: [Color; 3]) -> Result<(), String> {
    let [_, _, w, h] = rect;
    let track_y = (h - 12) / 2;
    let parts = [
        ("Track", NodeKind::Rectangle, [0, track_y, w, 12], 6.0),
        ("Fill", NodeKind::Rectangle, [0, track_y, w / 2, 12], 6.0),
        ("Thumb", NodeKind::Ellipse { start: 0.0, sweep: std::f64::consts::TAU, inner: 0.0 }, [(w - 26) / 2, (h - 26) / 2, 26, 26], 0.0),
    ];
    for ((name, kind, rect, radius), color) in parts.into_iter().zip(colors) {
        let mut layer = node(doc, kind, name, rect);
        fill(&mut layer, color);
        layer.corners.radii = [radius; 4];
        insert(doc, layer, id)?;
    }
    Ok(())
}

fn label(text: &str, size: u8, color: Color) -> Control {
    Control::Label { text: text.into(), color, size }
}

fn source(text: &str, active: bool) -> Control {
    let normal = if active { (Color([40, 95, 82]), MINT) } else { (Color([33, 61, 69]), INK) };
    let on = Color([12, 34, 32]);
    let states = StateRole::ALL.map(|role| {
        let (fill, color) = match role {
            StateRole::Normal => normal,
            StateRole::Feedback => (MINT, on),
            StateRole::Pressed => (Color([82, 175, 145]), on),
        };
        crate::model::ButtonState { name: role.name().into(), text: text.into(), fill, color }
    });
    Control::Button { states, size: 21 }
}

fn screen() -> [(&'static str, [i32; 4], Control); 15] {
    let card = Control::Shape { fill: Color([19, 40, 48]), radius: 14 };
    [
        ("Section", [60, 45, 650, 36], label("AURORA   /   ROOM CONTROL", 18, MINT)),
        ("Room title", [60, 100, 860, 75], label("Meeting Room", 50, INK)),
        ("Intro", [60, 184, 1080, 39], label("Choose a source and adjust the room audio.", 22, DIM)),
        ("Source card", [60, 270, 704, 348], card.clone()),
        ("Audio card", [792, 270, 428, 348], card),
        ("Source title", [91, 302, 510, 45], label("Input source", 28, INK)),
        ("Source hint", [91, 355, 540, 36], label("Select what appears on the display.", 17, DIM)),
        ("HDMI", [91, 428, 201, 82], source("HDMI", true)),
        ("USB-C", [311, 428, 201, 82], source("USB-C", false)),
        ("Wireless", [531, 428, 201, 82], source("Wireless", false)),
        ("Audio title", [824, 302, 332, 45], label("Room audio", 28, INK)),
        ("Volume label", [824, 355, 330, 36], label("Volume", 17, DIM)),
        ("Volume", [824, 404, 363, 45], Control::Slider { fill: MINT, track: Color([37, 64, 73]), thumb: INK, min: 0, max: 100 }),
        ("Mute", [824, 477, 363, 62], source("Mute", false)),
        ("Footer", [60, 702, 750, 36], label("AURORA   /   AV SYSTEMS", 16, DIM)),
    ]
}
