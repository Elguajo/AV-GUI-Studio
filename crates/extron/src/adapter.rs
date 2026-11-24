//! Adapt editable design nodes to the single-screen EXUI model.

use figcraft_color::{BlendMode, PaintKind};
use figcraft_doc::{Document, Node, NodeId, NodeKind, TextAlignH, TextAlignV, TextData};

use crate::metadata::{KEY, Kind, Metadata};
use crate::model::{ButtonState, Color, Control, Diagnostic, Element, HEIGHT, PROFILE, Project, StateRole, WIDTH};

pub fn project(doc: &Document) -> Result<Project, Vec<Diagnostic>> {
    let root = doc.current_page.or_else(|| doc.pages.first().copied()).unwrap_or(NodeId(0));
    let error = |message: &str| vec![Diagnostic::error(root, message)];
    if doc.pages.len() != 1 {
        return Err(error("Extron export requires one design page."));
    }
    let page = doc.node(root).ok_or_else(|| error("The design page is missing."))?;
    let mut frames = page.children.iter().filter_map(|id| doc.node(*id)).filter(|node| node.plugin_data.contains_key(KEY));
    let frame = frames.next().filter(|_| frames.next().is_none()).ok_or_else(|| error("The design page needs one Extron panel frame."))?;
    let (mut project, mut diagnostics) = read_frame(frame).map_err(|message| vec![Diagnostic::error(frame.id, message)])?;
    let mut component_sets = Vec::new();
    for id in &frame.children {
        let Some(node) = doc.node(*id) else {
            diagnostics.push(Diagnostic::error(frame.id, "A panel layer is missing."));
            continue;
        };
        match read_element(doc, node) {
            Ok((item, variants)) => {
                project.elements.push(item);
                component_sets.extend(variants);
            }
            Err(message) => diagnostics.push(Diagnostic::error(node.id, format!("{} — {message}", node.name))),
        }
    }
    for id in page.children.iter().filter(|id| **id != frame.id && !component_sets.contains(id)) {
        diagnostics.push(Diagnostic::error(*id, "Only the Extron panel and its button component sets may be top-level layers."));
    }
    diagnostics.extend(project.diagnostics());
    if diagnostics.is_empty() { Ok(project) } else { Err(diagnostics) }
}

fn read_frame(frame: &Node) -> Result<(Project, Vec<Diagnostic>), String> {
    let data = Metadata::read(frame)?;
    if data.kind != Kind::Page || data.profile.as_deref() != Some(PROFILE) || !matches!(frame.kind, NodeKind::Frame) {
        return Err("The Extron panel must be a frame assigned to TLP Pro 1025M.".into());
    }
    if frame.width != f64::from(WIDTH) || frame.height != f64::from(HEIGHT) {
        return Err("The Extron panel frame must be 1280 × 800.".into());
    }
    appearance(frame)?;
    let mut diagnostics = Vec::new();
    if frame.transform.as_coeffs()[..4] != [1.0, 0.0, 0.0, 1.0] {
        diagnostics.push(Diagnostic::error(frame.id, "The panel frame cannot be rotated, scaled or skewed."));
    }
    Ok((
        Project {
            page: frame.id,
            name: data.project_name.clone().ok_or("Missing Extron project name.")?,
            page_name: frame.name.clone(),
            identity: data.identity()?,
            background: solid(frame)?,
            elements: Vec::new(),
        },
        diagnostics,
    ))
}

fn read_element(doc: &Document, node: &Node) -> Result<(Element, Option<NodeId>), String> {
    let data = Metadata::read(node)?;
    appearance(node)?;
    let bounds = bounds(node)?;
    let (control, variants) = match (&node.kind, data.kind) {
        (NodeKind::Text(_), Kind::Label) => {
            let (text, color, size) = text(node, TextAlignH::Left)?;
            (Control::Label { text, color, size }, None)
        }
        (NodeKind::Rectangle, Kind::Shape) => (Control::Shape { fill: solid(node)?, radius: radius(node)? }, None),
        (NodeKind::Frame, Kind::Slider) => (slider(doc, node, &data)?, None),
        (NodeKind::Instance { component, .. }, Kind::Button) => button(doc, node, *component)?,
        _ => return Err("Extron assignment does not match a supported layer type.".into()),
    };
    Ok((Element { node: node.id, name: node.name.clone(), identity: data.identity()?, bounds, control }, variants))
}

fn appearance(node: &Node) -> Result<(), String> {
    if !node.visible
        || node.opacity != 1.0
        || node.is_mask
        || !matches!(node.blend, BlendMode::Normal | BlendMode::PassThrough)
        || !node.strokes.is_empty()
        || !node.effects.is_empty()
        || node.layout.is_some()
        || !node.bound.is_empty()
        || !node.interactions.is_empty()
    {
        Err("Hidden layers, opacity, masks, strokes, effects, auto layout, variables and prototype actions are unsupported in this export.".into())
    } else {
        Ok(())
    }
}

fn solid(node: &Node) -> Result<Color, String> {
    if let [paint] = node.fills.as_slice()
        && let PaintKind::Solid { color } = paint.kind
        && paint.visible
        && paint.opacity == 1.0
        && paint.blend == BlendMode::Normal
        && color.a == 1.0
    {
        let [r, g, b, _] = color.to_rgba8();
        Ok(Color([r, g, b]))
    } else {
        Err("Use one opaque solid fill for native export.".into())
    }
}

fn bounds(node: &Node) -> Result<[i32; 4], String> {
    let [a, b, c, d, x, y] = node.transform.as_coeffs();
    if [a, b, c, d] != [1.0, 0.0, 0.0, 1.0] {
        return Err("Rotated, scaled or skewed layers cannot be exported as native controls.".into());
    }
    let rect = [x, y, node.width, node.height];
    if !rect.iter().all(|v| v.is_finite() && (0.0..=f64::from(i32::MAX)).contains(v)) {
        return Err("Layer geometry is outside its container.".into());
    }
    Ok(rect.map(|v| v.round() as i32))
}

fn radius(node: &Node) -> Result<u8, String> {
    let [r, b, c, d] = node.corners.radii;
    if [b, c, d] != [r; 3] || node.corners.smoothing != 0.0 || !(0.0..=255.0).contains(&r) {
        return Err("Use the same unsmoothed corner radius (0–255) on all corners.".into());
    }
    Ok(r.round() as u8)
}

/// Canvas text sizes are px and Extron font sizes are pt, converted by the exporter's px × 0.75
/// rule (figma-extron-exporter `src/figma-plugin/NATIVE_TEXT.md`).
pub(crate) const PX_PER_PT: f64 = 4.0 / 3.0;

fn text(node: &Node, align: TextAlignH) -> Result<(String, Color, u8), String> {
    appearance(node)?;
    let NodeKind::Text(data) = &node.kind else { return Err("A native caption must be a text layer.".into()) };
    let mut supported = TextData::new(data.characters.clone());
    supported.style.font_family = "Arial".into();
    supported.style.font_style = "Regular".into();
    supported.style.font_size = data.style.font_size;
    supported.align_h = align;
    supported.align_v = TextAlignV::Middle;
    supported.auto_resize = data.auto_resize;
    supported.saved_breaks = data.saved_breaks.clone();
    let points = (data.style.font_size / PX_PER_PT).round();
    if &supported != data || !points.is_finite() || !(4.0..=200.0).contains(&points) {
        return Err("Use uniform Arial Regular at 4–200 pt (px × 0.75), middle aligned, with the native caption alignment.".into());
    }
    Ok((data.characters.clone(), solid(node)?, points as u8))
}

fn slider(doc: &Document, node: &Node, data: &Metadata) -> Result<Control, String> {
    let [track_id, fill_id, thumb_id] = node.children.as_slice() else {
        return Err("A slider needs a track, fill and thumb in that order.".into());
    };
    let get = |id| doc.node(id).ok_or("Missing slider layer.");
    let (track, fill, thumb) = (get(*track_id)?, get(*fill_id)?, get(*thumb_id)?);
    for part in [track, fill, thumb] {
        appearance(part)?;
        if !part.children.is_empty() {
            return Err("Slider parts must be simple shapes.".into());
        }
    }
    if !node.fills.is_empty()
        || !matches!(track.kind, NodeKind::Rectangle)
        || !matches!(fill.kind, NodeKind::Rectangle)
        || !matches!(thumb.kind, NodeKind::Ellipse { start: 0.0, inner: 0.0, .. })
    {
        return Err("Use a transparent slider frame with a rectangle track, rectangle fill and ellipse thumb.".into());
    }
    Ok(Control::Slider {
        fill: solid(fill)?,
        track: solid(track)?,
        thumb: solid(thumb)?,
        min: data.min.ok_or("Slider minimum is missing.")?,
        max: data.max.ok_or("Slider maximum is missing.")?,
    })
}

fn button(doc: &Document, instance: &Node, component: NodeId) -> Result<(Control, Option<NodeId>), String> {
    let master = doc.node(component).ok_or("Button component is missing.")?;
    let set = master
        .parent
        .and_then(|id| doc.node(id))
        .filter(|n| matches!(n.kind, NodeKind::ComponentSet { .. }))
        .ok_or("A button must reference a component variant set.")?;
    if !instance.overrides.iter().all(|key| matches!(key.as_str(), "size" | "position" | "name"))
        || instance.children.iter().any(|id| doc.node(*id).is_some_and(|n| !n.overrides.is_empty()))
    {
        return Err("Edit appearance or caption on the button state variants before exporting.".into());
    }
    if set.children.len() != 3 {
        return Err("A button needs Normal, Active and Pressed variants.".into());
    }
    let mut states = [None, None, None];
    let mut size = None;
    for id in &set.children {
        let state = doc.node(*id).ok_or("Button state variant is missing.")?;
        let (role, appearance, font) = button_state(doc, state).map_err(|err| format!("{} — {err}", state.name))?;
        if states[role.id()].replace(appearance).is_some() {
            return Err("Button state role is assigned more than once.".into());
        }
        if size.replace(font).is_some_and(|s| s != font) {
            return Err("Button states must share the same font size.".into());
        }
    }
    let [Some(normal), Some(active), Some(pressed)] = states else { return Err("Button state is missing.".into()) };
    Ok((Control::Button { states: [normal, active, pressed], size: size.ok_or("Missing button font size.")? }, Some(set.id)))
}

fn button_state(doc: &Document, state: &Node) -> Result<(StateRole, ButtonState, u8), String> {
    let data = Metadata::read(state)?;
    let role = data.role.filter(|r| Some(r.id()) == data.state_id && data.kind == Kind::State).ok_or("Invalid button state ID or role.")?;
    let [caption] = state.children.as_slice() else { return Err("A button state needs one native caption.".into()) };
    let caption = doc.node(*caption).ok_or("Missing button caption.")?;
    appearance(state)?;
    if !matches!(state.kind, NodeKind::Component { .. })
        || radius(state)? != 0
        || bounds(caption)? != [0, 0, state.width.round() as i32, state.height.round() as i32]
    {
        return Err("Use a rectangular variant with a caption filling the state bounds.".into());
    }
    let (text, color, size) = text(caption, TextAlignH::Center)?;
    let name = data.state_name.ok_or("Missing button state name.")?;
    Ok((role, ButtonState { name, text, fill: solid(state)?, color }, size))
}
