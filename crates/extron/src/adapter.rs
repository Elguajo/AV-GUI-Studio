//! Adapt editable design nodes to the EXUI model: every panel artboard on the design page is a page.

use figcraft_color::{BlendMode, PaintKind};
use figcraft_doc::{Document, Node, NodeId, NodeKind, TextAlignH, TextAlignV, TextData};

use crate::metadata::{KEY, Kind, Metadata};
use crate::model::{ButtonState, Color, Control, Diagnostic, Element, Page, Profile, Project, StateRole, profile};

pub fn project(doc: &Document) -> Result<Project, Vec<Diagnostic>> {
    let root = doc.current_page.or_else(|| doc.pages.first().copied()).unwrap_or(NodeId(0));
    let page_name = doc.node(root).map(|page| page.name.as_str()).unwrap_or_default();
    let error = |message: &str| vec![Diagnostic::error(root, page_name, message)];
    if doc.pages.len() != 1 {
        return Err(error("Extron export requires one design page."));
    }
    let page = doc.node(root).ok_or_else(|| error("The design page is missing."))?;
    let frames: Vec<&Node> = page.children.iter().filter_map(|id| doc.node(*id)).filter(|node| node.plugin_data.contains_key(KEY)).collect();
    if frames.is_empty() {
        return Err(error("The design page needs an Extron panel frame."));
    }
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut pages = Vec::new();
    for frame in &frames {
        let (mut screen, data) = match read_frame(frame) {
            Ok((screen, data, problems)) => {
                diagnostics.extend(problems);
                (screen, data)
            }
            Err(message) => {
                diagnostics.push(Diagnostic::error(frame.id, &frame.name, message));
                continue;
            }
        };
        for id in &frame.children {
            let Some(node) = doc.node(*id) else {
                diagnostics.push(Diagnostic::error(frame.id, &frame.name, "A panel layer is missing."));
                continue;
            };
            match read_element(doc, node) {
                Ok(item) => screen.elements.push(item),
                // A problem in a master shows once, at the master, however many controls use it.
                Err((at, message)) => {
                    let layer = doc.node(at).map_or(node.name.as_str(), |n| n.name.as_str());
                    if !diagnostics.iter().any(|d| d.node_id == at && d.message == message) {
                        diagnostics.push(Diagnostic::error(at, layer, message));
                    }
                }
            }
        }
        pages.push((screen, data));
    }
    let panels: Vec<NodeId> = frames.iter().map(|frame| frame.id).collect();
    let master = |node: &Node| matches!(node.kind, NodeKind::Component { .. } | NodeKind::ComponentSet { .. });
    for node in page.children.iter().filter(|id| !panels.contains(id)).filter_map(|id| doc.node(*id)).filter(|node| !master(node)) {
        diagnostics.push(Diagnostic::error(node.id, &node.name, "Only Extron panel artboards and master components may be top-level layers."));
    }
    // The start page is the one with the lowest Object ID (the first panel of the project); the
    // others follow in layer order. Every page shares its panel model and project name.
    let Some(start) = (0..pages.len()).min_by_key(|i| pages[*i].0.identity.object_id) else {
        return Err(diagnostics);
    };
    let (first, (panel, name)) = pages.remove(start);
    pages.insert(0, (first, (panel, name.clone())));
    for (screen, (other, _)) in pages.iter().skip(1) {
        if other.id != panel.id {
            diagnostics.push(Diagnostic::error(
                screen.node,
                &screen.name,
                format!("Every page needs the project's panel, {}. This one is a {}.", panel.name, other.name),
            ));
        }
    }
    let project = Project { name, profile: panel, pages: pages.into_iter().map(|(screen, _)| screen).collect() };
    diagnostics.extend(project.diagnostics());
    if diagnostics.is_empty() { Ok(project) } else { Err(diagnostics) }
}

/// Whether `doc` carries Extron assignments, so its export problems are worth showing.
pub fn is_screen(doc: &Document) -> bool {
    doc.nodes().any(|node| node.plugin_data.contains_key(KEY))
}

/// A page's touch panel model and the project name it carries.
type Target = (&'static Profile, String);

/// A panel artboard as an empty page, with its panel model and project name.
fn read_frame(frame: &Node) -> Result<(Page, Target, Vec<Diagnostic>), String> {
    let data = Metadata::read(frame)?;
    let panel = data.profile.as_deref().and_then(profile);
    let Some(panel) = panel.filter(|_| data.kind == Kind::Page && matches!(frame.kind, NodeKind::Frame)) else {
        return Err("The Extron panel must be a frame assigned to a supported touch panel.".into());
    };
    if frame.width != f64::from(panel.width) || frame.height != f64::from(panel.height) {
        return Err(format!("The {} panel frame must be {} × {}.", panel.name, panel.width, panel.height));
    }
    appearance(frame)?;
    let mut diagnostics = Vec::new();
    if frame.transform.as_coeffs()[..4] != [1.0, 0.0, 0.0, 1.0] {
        diagnostics.push(Diagnostic::error(frame.id, &frame.name, "The panel frame cannot be rotated, scaled or skewed."));
    }
    let name = data.project_name.clone().ok_or("Missing Extron project name.")?;
    let page = Page { node: frame.id, name: frame.name.clone(), identity: data.identity()?, background: solid(frame)?, elements: Vec::new() };
    Ok((page, (panel, name), diagnostics))
}

/// A problem and the layer to fix it on.
type Located = (NodeId, String);

fn read_element(doc: &Document, node: &Node) -> Result<Element, Located> {
    let here = |message: String| (node.id, message);
    let data = Metadata::read(node).map_err(here)?;
    appearance(node).map_err(here)?;
    let bounds = bounds(node).map_err(here)?;
    let control = match (&node.kind, data.kind) {
        (NodeKind::Text(_), Kind::Label) => {
            let (text, color, size) = text(node, TextAlignH::Left).map_err(here)?;
            Control::Label { text, color, size }
        }
        (NodeKind::Rectangle, Kind::Shape) => Control::Shape { fill: solid(node).map_err(here)?, radius: radius(node).map_err(here)? },
        (NodeKind::Frame | NodeKind::Instance { .. }, Kind::Slider) => slider(doc, node, &data).map_err(here)?,
        (NodeKind::Instance { component, .. }, Kind::Button) => button(doc, node, *component)?,
        _ => return Err(here("Extron assignment does not match a supported layer type.".into())),
    };
    Ok(Element { node: node.id, name: node.name.clone(), identity: data.identity().map_err(here)?, bounds, control })
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
    // Within the integer range: the screen check then names the 1280 × 800 bounds a layer leaves.
    if !rect.iter().all(|v| v.is_finite() && (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(v)) {
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

/// A button instance's states, from the Normal, Active and Pressed variants of its set. A caption
/// edited on the instance replaces the caption of every state; anything else about its look
/// belongs to the variants. Problems in the set are reported at the set or the variant.
fn button(doc: &Document, instance: &Node, component: NodeId) -> Result<Control, Located> {
    let here = |message: &str| (instance.id, message.to_owned());
    let master = doc.node(component).ok_or_else(|| here("Button component is missing."))?;
    let set = master
        .parent
        .and_then(|id| doc.node(id))
        .filter(|n| matches!(n.kind, NodeKind::ComponentSet { .. }))
        .ok_or_else(|| here("A button must reference a component variant set."))?;
    if !instance.overrides.iter().all(|key| matches!(key.as_str(), "size" | "position" | "name")) {
        return Err(here("Only the caption, size and position can change on a button. Edit its look on the master's state variants."));
    }
    let caption = instance_caption(doc, instance).map_err(|message| here(&message))?;
    let at_set = |message: &str| (set.id, message.to_owned());
    if set.children.len() != 3 {
        return Err(at_set("A button needs Normal, Active and Pressed variants."));
    }
    let mut states = [None, None, None];
    let mut size = None;
    for id in &set.children {
        let state = doc.node(*id).ok_or_else(|| at_set("Button state variant is missing."))?;
        let (role, mut appearance, font) = button_state(doc, state).map_err(|message| (state.id, message))?;
        if let Some(text) = &caption {
            appearance.text.clone_from(text);
        }
        if states[role.id()].replace(appearance).is_some() {
            return Err(at_set("Button state role is assigned more than once."));
        }
        if size.replace(font).is_some_and(|s| s != font) {
            return Err(at_set("Button states must share the same font size."));
        }
    }
    let [Some(normal), Some(active), Some(pressed)] = states else { return Err(at_set("Button state is missing.")) };
    Ok(Control::Button { states: [normal, active, pressed], size: size.ok_or_else(|| at_set("Missing button font size."))? })
}

/// The caption edited on a button instance, if any. Its layers follow the variant otherwise.
fn instance_caption(doc: &Document, instance: &Node) -> Result<Option<String>, String> {
    let mut caption = None;
    for layer in instance.children.iter().filter_map(|id| doc.node(*id)) {
        let text = matches!(layer.kind, NodeKind::Text(_));
        if !layer.overrides.iter().all(|key| text && key == "characters") {
            return Err("Only the caption, size and position can change on a button. Edit its look on the master's state variants.".into());
        }
        if let NodeKind::Text(data) = &layer.kind
            && layer.overrides.contains("characters")
        {
            caption = Some(data.characters.clone());
        }
    }
    Ok(caption)
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
