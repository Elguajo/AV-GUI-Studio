//! Authoring a touch panel project: an empty first page, more panel artboards (pages), and native
//! controls inserted on a page or assigned to its layers. Presets and default sizes follow the
//! eframe prototype's `Kind::create` and `Kind::default_size` (tag `eframe-prototype`,
//! `src/project.rs`).
//!
//! Object and Control IDs come from one project-wide counter kept in every panel frame's
//! `plugin_data` ([`NEXT_ID_KEY`]), so an ID that belonged to a deleted control or page is never
//! handed out again. A control's Control ID equals its Object ID, as in the prototype and the demo.
//!
//! Buttons and sliders are instances of a master component beside the panels, one per control
//! kind and UI kit ([`MASTER_KEY`]). Editing a master restyles every control made from it; each
//! keeps its own key and IDs. A button's caption is an override on its instance.

use std::str::FromStr;

use figcraft_doc::{Document, Node, NodeId, NodeKind, TextAlignH};

use crate::demo;
use crate::kit::Kit;
use crate::metadata::{KEY, Kind, Metadata};
use crate::model::{MAX_ELEMENTS, MAX_PAGES, Profile, StateRole, profile};

/// The panel frames' `plugin_data` entry holding the next Object ID to hand out.
pub const NEXT_ID_KEY: &str = "guiStudio.nextObjectId";

/// The first page's own Object ID; controls and later pages get higher ones.
const PAGE_OBJECT_ID: u16 = 100;
/// The panel frames' `plugin_data` entry holding the project's UI kit (its JSON), so the project
/// keeps its look where the kit is not installed.
pub const KIT_KEY: &str = "guiStudio.kit";
/// The space a new page leaves to the right of what is already on the canvas.
const ARTBOARD_GAP: i32 = 160;
/// The `plugin_data` entry marking a master: `button:<kit id>` on a button's variant set,
/// `slider:<kit id>` on a slider component.
pub const MASTER_KEY: &str = "guiStudio.master";

/// Rebuilds an instance from its main component: the editor's component sync
/// (`figcraft_engine::components::sync_instance`), which this crate cannot depend on.
pub type SyncInstance = fn(&mut Document, NodeId);

/// A native control type that can be inserted or assigned.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlKind {
    Button,
    Slider,
    Label,
    Shape,
}

impl ControlKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Slider => "slider",
            Self::Label => "label",
            Self::Shape => "shape",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Button => "Button",
            Self::Slider => "Slider",
            Self::Label => "Label",
            Self::Shape => "Shape",
        }
    }

    pub fn default_size(self) -> [i32; 2] {
        match self {
            Self::Button => [190, 72],
            Self::Slider => [272, 48],
            Self::Label => [216, 44],
            Self::Shape => [256, 172],
        }
    }

    fn addressable(self) -> bool {
        matches!(self, Self::Button | Self::Slider)
    }

    fn metadata(self) -> Kind {
        match self {
            Self::Button => Kind::Button,
            Self::Slider => Kind::Slider,
            Self::Label => Kind::Label,
            Self::Shape => Kind::Shape,
        }
    }

    /// The layer type this control needs, as an error names it.
    fn layer(self) -> &'static str {
        match self {
            Self::Button => "an instance of a button variant set",
            Self::Slider => "a frame or an instance of a slider master",
            Self::Label => "a text layer",
            Self::Shape => "a rectangle",
        }
    }

    fn fits(self, kind: &NodeKind) -> bool {
        matches!(
            (self, kind),
            (Self::Button, NodeKind::Instance { .. })
                | (Self::Slider, NodeKind::Frame | NodeKind::Instance { .. })
                | (Self::Label, NodeKind::Text(_))
                | (Self::Shape, NodeKind::Rectangle)
        )
    }
}

impl FromStr for ControlKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, String> {
        match value {
            "button" => Ok(Self::Button),
            "slider" => Ok(Self::Slider),
            "label" => Ok(Self::Label),
            "shape" => Ok(Self::Shape),
            other => Err(format!("Unknown control kind \"{other}\". Use button, slider, label or shape.")),
        }
    }
}

/// What to insert besides the kind: everything is optional.
#[derive(Clone, Debug, Default)]
pub struct Insert {
    /// The page (panel frame) to insert on (default: the start page).
    pub panel: Option<NodeId>,
    /// Layer name (default: the kind, such as "Button").
    pub name: Option<String>,
    /// Caption of a button or label.
    pub text: Option<String>,
    /// Left edge in the panel (default: centred).
    pub x: Option<i32>,
    /// Top edge in the panel (default: centred).
    pub y: Option<i32>,
    /// Width and height (default: the kind's size).
    pub size: Option<[i32; 2]>,
    /// The kit whose look the control takes (default: the project's).
    pub kit: Option<Kit>,
    /// A label's size in points instead of the kit's.
    pub font: Option<u8>,
    /// A label in the kit's secondary text colour.
    pub muted: bool,
}

/// A new page: both fields are optional.
#[derive(Clone, Debug, Default)]
pub struct NewPage {
    /// Page name (default: the next free "Page N").
    pub name: Option<String>,
    /// Top-left corner on the canvas (default: right of everything on the canvas, level with the
    /// start page).
    pub at: Option<[i32; 2]>,
}

/// The identity a control got or kept.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assigned {
    pub node: NodeId,
    pub key: String,
    pub object_id: u16,
    pub control_id: Option<u16>,
}

/// A new document holding one empty page named "Home" for the touch panel `panel`, in a project
/// named `name` with the look of `kit`.
pub fn panel(name: &str, panel: &'static Profile, kit: &Kit) -> Result<Document, String> {
    if !crate::model::valid_name(name) {
        return Err("Project name needs 1–256 characters.".into());
    }
    let mut doc = Document::new(name);
    let root = doc.current_page.ok_or("Missing design page.")?;
    let mut frame = demo::node(&mut doc, NodeKind::Frame, "Home", [0, 0, panel.width, panel.height]);
    demo::fill(&mut frame, kit.background()?);
    frame.plugin_data.insert(KIT_KEY.into(), serde_json::to_string(kit).map_err(|e| e.to_string())?);
    let mut data = Metadata::new(&frame, Kind::Page);
    data.profile = Some(panel.id.into());
    data.project_name = Some(name.into());
    data.object_id = Some(PAGE_OBJECT_ID);
    data.key = Some("page.home".into());
    demo::assign(&mut frame, &data)?;
    frame.plugin_data.insert(NEXT_ID_KEY.into(), (PAGE_OBJECT_ID + 1).to_string());
    demo::insert(&mut doc, frame, root)?;
    Ok(doc)
}

/// The panel frames (pages) on the current page, in layer order.
pub fn panels(doc: &Document) -> Vec<NodeId> {
    let Some(page) = doc.current_page.and_then(|id| doc.node(id)) else { return Vec::new() };
    page.children
        .iter()
        .filter_map(|id| doc.node(*id))
        .filter(|node| matches!(node.kind, NodeKind::Frame) && Metadata::read(node).is_ok_and(|data| data.kind == Kind::Page))
        .map(|node| node.id)
        .collect()
}

/// The panel frame `node` is in (or is).
pub fn panel_of(doc: &Document, node: NodeId) -> Option<NodeId> {
    let panels = panels(doc);
    let mut at = Some(node);
    while let Some(id) = at {
        if panels.contains(&id) {
            return Some(id);
        }
        at = doc.node(id).and_then(|n| n.parent);
    }
    None
}

/// The start page's panel frame: the one with the lowest Object ID, as export orders them.
pub fn panel_frame(doc: &Document) -> Result<NodeId, String> {
    panels(doc)
        .into_iter()
        .filter_map(|id| Some((Metadata::read(doc.node(id)?).ok()?.object_id.unwrap_or(u16::MAX), id)))
        .min()
        .map(|(_, id)| id)
        .ok_or_else(|| "This page has no Extron panel. Start with a new touch panel project.".into())
}

/// The touch panel model of the project: the start page's.
pub fn project_profile(doc: &Document) -> Result<&'static Profile, String> {
    let start = panel_frame(doc)?;
    let data = doc.node(start).ok_or("The panel is missing.").and_then(|n| Metadata::read(n).map_err(|_| "The panel is missing."))?;
    data.profile.as_deref().and_then(profile).ok_or_else(|| "The start page has no supported touch panel.".into())
}

/// The project's UI kit, kept on its start page; Aurora for a project that has none (the demo, an
/// older file) or none that can be read.
pub fn project_kit(doc: &Document) -> Result<Kit, String> {
    let start = panel_frame(doc)?;
    match doc.node(start).and_then(|frame| frame.plugin_data.get(KIT_KEY)).and_then(|raw| Kit::parse(raw.as_bytes()).ok()) {
        Some(kit) => Ok(kit),
        None => Kit::default_kit(),
    }
}

/// Give the project the look of `kit` for the controls inserted from now on. Controls already
/// placed keep theirs.
pub fn set_kit(doc: &mut Document, kit: &Kit) -> Result<(), String> {
    let pages = panels(doc);
    if pages.is_empty() {
        return Err("This page has no Extron panel. Start with a new touch panel project.".into());
    }
    let raw = serde_json::to_string(kit).map_err(|e| e.to_string())?;
    for page in pages {
        if let Some(frame) = doc.node_mut(page) {
            frame.plugin_data.insert(KIT_KEY.into(), raw.clone());
        }
    }
    Ok(())
}

/// Add an empty page (a panel artboard) to the project, with the start page's panel model, project
/// name and background, and a new key and Object ID.
pub fn add_page(doc: &mut Document, spec: NewPage) -> Result<NodeId, String> {
    let start = panel_frame(doc)?;
    let existing = panels(doc);
    if existing.len() >= MAX_PAGES {
        return Err(format!("A project holds up to {MAX_PAGES} pages."));
    }
    let panel = project_profile(doc)?;
    let source = doc.node(start).ok_or("The panel is missing.")?;
    let project_name = Metadata::read(source).ok().and_then(|data| data.project_name).ok_or("Missing Extron project name.")?;
    let fills = source.fills.clone();
    let start_y = source.transform.as_coeffs()[5].round() as i32;
    let names: Vec<String> = existing.iter().filter_map(|id| doc.node(*id)).map(|n| n.name.clone()).collect();
    let name = match spec.name {
        Some(name) if !crate::model::valid_name(&name) => return Err("Page name needs 1–256 characters.".into()),
        Some(name) if names.iter().any(|n| n.trim() == name.trim()) => return Err(format!("Page name \"{name}\" is already used by another page.")),
        Some(name) => name,
        None => (2..).map(|n| format!("Page {n}")).find(|candidate| !names.contains(candidate)).unwrap_or_default(),
    };
    let root = doc.current_page.ok_or("Missing design page.")?;
    let [x, y] = spec.at.unwrap_or_else(|| [right_edge(doc, root) + ARTBOARD_GAP, start_y]);
    let object_id = allocate(doc)?;
    let mut frame = demo::node(doc, NodeKind::Frame, &name, [x, y, panel.width, panel.height]);
    frame.fills = fills;
    let mut data = Metadata::new(&frame, Kind::Page);
    data.profile = Some(panel.id.into());
    data.project_name = Some(project_name);
    data.object_id = Some(object_id);
    data.key = Some(format!("page.{object_id}"));
    demo::assign(&mut frame, &data)?;
    for key in [NEXT_ID_KEY, KIT_KEY] {
        if let Some(value) = doc.node(start).and_then(|n| n.plugin_data.get(key).cloned()) {
            frame.plugin_data.insert(key.into(), value);
        }
    }
    demo::insert(doc, frame, root)
}

/// The right edge of everything on the canvas page, in whole pixels (0 when it is empty).
fn right_edge(doc: &Document, root: NodeId) -> i32 {
    let children = doc.node(root).map(|page| page.children.clone()).unwrap_or_default();
    doc.bounds_of(&children).map_or(0, |b| b.x1.ceil() as i32)
}

/// Insert a `kind` control at the top of a page, with a new key and IDs. A button or slider is an
/// instance of the kit's master, which `sync` builds.
pub fn insert(doc: &mut Document, kind: ControlKind, spec: Insert, sync: SyncInstance) -> Result<Assigned, String> {
    let panel = match spec.panel {
        Some(id) if panels(doc).contains(&id) => id,
        Some(id) => return Err(format!("Node {id} is not an Extron panel artboard.")),
        None => panel_frame(doc)?,
    };
    let frame = doc.node(panel).ok_or("The panel is missing.")?;
    let controls = frame.children.iter().filter_map(|id| doc.node(*id)).filter(|n| n.plugin_data.contains_key(KEY)).count();
    if controls >= MAX_ELEMENTS {
        return Err(format!("This screen supports up to {MAX_ELEMENTS} elements."));
    }
    let (width, height) = (frame.width.round() as i32, frame.height.round() as i32);
    let kit = match spec.kit {
        Some(kit) => kit,
        None => project_kit(doc)?,
    };
    let [w, h] = spec.size.unwrap_or_else(|| kind.default_size());
    if w <= 0 || h <= 0 {
        return Err("A control needs a positive width and height.".into());
    }
    let [w, h] = [w.min(width), h.min(height)];
    let rect = [spec.x.unwrap_or((width - w) / 2), spec.y.unwrap_or((height - h) / 2), w, h];
    let name = spec.name.unwrap_or_else(|| kind.label().into());
    let layer = match kind {
        ControlKind::Button | ControlKind::Slider => {
            let id = instance(doc, kind, &kit, panel, &name, rect, sync)?;
            if kind == ControlKind::Button {
                caption(doc, id, &spec.text.unwrap_or_else(|| kind.label().into()))?;
            }
            return identify(doc, id, kind);
        }
        ControlKind::Label => {
            let text = spec.text.unwrap_or_else(|| kind.label().into());
            let (color, size) = kit.label(spec.muted, spec.font)?;
            demo::text_layer(doc, &name, rect, text, color, size, TextAlignH::Left)
        }
        ControlKind::Shape => {
            let mut shape = demo::node(doc, NodeKind::Rectangle, &name, rect);
            let (fill, radius) = kit.shape()?;
            demo::fill(&mut shape, fill);
            shape.corners.radii = [f64::from(radius); 4];
            shape
        }
    };
    let id = demo::insert(doc, layer, panel)?;
    identify(doc, id, kind)
}

/// An instance of the `kind` master of `kit` named `name` at `rect` in `panel`. A size other than
/// the master's is the instance's own (a `size` override), and its layers follow their constraints.
fn instance(
    doc: &mut Document,
    kind: ControlKind,
    kit: &Kit,
    panel: NodeId,
    name: &str,
    rect: [i32; 4],
    sync: SyncInstance,
) -> Result<NodeId, String> {
    let component = master(doc, kind, kit, panel)?;
    let main = doc.node(component).ok_or("The master component is missing.")?;
    let (mw, mh) = (main.width, main.height);
    let [x, y, w, h] = rect;
    let mut layer = demo::node(doc, NodeKind::Instance { component, props: Default::default() }, name, [x, y, mw.round() as i32, mh.round() as i32]);
    layer.source = Some(component);
    let id = demo::insert(doc, layer, panel)?;
    sync(doc, id);
    let (w, h) = (f64::from(w), f64::from(h));
    let resized = (w - mw).abs() > f64::EPSILON || (h - mh).abs() > f64::EPSILON;
    if resized {
        figcraft_layout::set_size(doc, id, w, h);
        figcraft_layout::apply_constraints(doc, id, mw, mh);
    }
    let layer = doc.node_mut(id).ok_or("The new control is missing.")?;
    if resized {
        layer.overrides.insert("size".into());
    }
    if layer.name != name {
        layer.name = name.into();
        layer.overrides.insert("name".into());
    }
    Ok(id)
}

/// Show `text` on button instance `id`: an override of its caption, unless the master already
/// says it. It applies to every state of the button.
fn caption(doc: &mut Document, id: NodeId, text: &str) -> Result<(), String> {
    let caption = doc.node(id).and_then(|n| n.children.iter().copied().find(|c| doc.node(*c).is_some_and(|c| matches!(c.kind, NodeKind::Text(_)))));
    let layer = caption.and_then(|c| doc.node_mut(c)).ok_or("The button master has no caption.")?;
    let NodeKind::Text(data) = &mut layer.kind else { return Err("The button master has no caption.".into()) };
    let text = text.replace("\r\n", "\n");
    if data.characters != text {
        let len = data.char_count();
        data.replace_range(0, len, &text);
        layer.overrides.insert("characters".into());
    }
    Ok(())
}

/// The component to instantiate for a `kind` control in the look of `kit` (a button's Normal
/// variant): the kit's master on the canvas page, made beside the panels the first time.
fn master(doc: &mut Document, kind: ControlKind, kit: &Kit, panel: NodeId) -> Result<NodeId, String> {
    let root = doc.current_page.ok_or("Missing design page.")?;
    let tag = format!("{}:{}", kind.name(), kit.id);
    let found = doc.node(root).into_iter().flat_map(|page| page.children.iter()).filter_map(|id| doc.node(*id)).find(|n| {
        n.plugin_data.get(MASTER_KEY) == Some(&tag)
            && matches!((kind, &n.kind), (ControlKind::Button, NodeKind::ComponentSet { .. }) | (ControlKind::Slider, NodeKind::Component { .. }))
    });
    if let Some(master) = found {
        if kind == ControlKind::Slider {
            return Ok(master.id);
        }
        let normal = master
            .children
            .iter()
            .copied()
            .find(|id| doc.node(*id).and_then(|n| Metadata::read(n).ok()).is_some_and(|data| data.role == Some(StateRole::Normal)));
        return normal.or_else(|| master.children.first().copied()).ok_or_else(|| format!("The \"{}\" master has no variants.", master.name));
    }
    let at = variant_spot(doc, root, panel);
    let [w, h] = kind.default_size();
    let name = format!("{} {}", kit.name, kind.name());
    let (id, component) = match kind {
        ControlKind::Button => {
            let crate::model::Control::Button { states, size } = kit.button(kind.label())? else { return Err("Missing button preset.".into()) };
            demo::button_set(doc, root, &name, [w, h], states, size, at)?
        }
        _ => {
            let kind = NodeKind::Component { description: String::new(), props: Default::default(), prop_order: Vec::new() };
            let mut layer = demo::node(doc, kind, &name, [at[0], at[1], w, h]);
            layer.fills.clear();
            let id = demo::insert(doc, layer, root)?;
            demo::slider(doc, id, [0, 0, w, h], kit.slider()?)?;
            (id, id)
        }
    };
    doc.node_mut(id).ok_or("The new master is missing.")?.plugin_data.insert(MASTER_KEY.into(), tag);
    Ok(component)
}

/// Insert the block `name` of `kit` on a page (default: the start page) with its top-left corner at
/// `at` (default: centred), each item a control with a new key and IDs. Its buttons and sliders are
/// instances of the kit's masters, which `sync` builds.
pub fn insert_block(
    doc: &mut Document,
    kit: &Kit,
    name: &str,
    panel: Option<NodeId>,
    at: Option<[i32; 2]>,
    sync: SyncInstance,
) -> Result<Vec<Assigned>, String> {
    let block = kit.block(name).ok_or_else(|| format!("The {} kit has no block \"{name}\".", kit.name))?.clone();
    let panel = match panel {
        Some(id) => id,
        None => panel_frame(doc)?,
    };
    let frame = doc.node(panel).ok_or_else(|| format!("Node {panel} is not an Extron panel artboard."))?;
    let (width, height) = (frame.width.round() as i32, frame.height.round() as i32);
    let [w, h] = block.size;
    if w > width || h > height {
        return Err(format!("\"{name}\" is {w} × {h} and does not fit the {width} × {height} page."));
    }
    let controls = frame.children.iter().filter_map(|id| doc.node(*id)).filter(|n| n.plugin_data.contains_key(KEY)).count();
    if controls + block.items.len() > MAX_ELEMENTS {
        return Err(format!("\"{name}\" adds {} controls; this screen supports up to {MAX_ELEMENTS} elements.", block.items.len()));
    }
    let [x, y] = at.unwrap_or([(width - w) / 2, (height - h) / 2]);
    let mut placed = Vec::new();
    for item in &block.items {
        let kind: ControlKind = item.kind.parse()?;
        let [ix, iy, iw, ih] = item.rect;
        let spec = Insert {
            panel: Some(panel),
            name: Some(item.name.clone()),
            text: Some(item.text.clone().unwrap_or_else(|| item.name.clone())),
            x: Some(x + ix),
            y: Some(y + iy),
            size: Some([iw, ih]),
            kit: Some(kit.clone()),
            font: item.size,
            muted: item.muted,
        };
        placed.push(insert(doc, kind, spec, sync)?);
    }
    Ok(placed)
}

/// Make the layer `node`, directly in the panel, a `kind` control. A layer that already is one
/// keeps its key and IDs; a copied assignment or a different kind gets new ones, so a replacement
/// never inherits another control's bindings. A button's variants get the Normal, Active and
/// Pressed roles in their order unless they already carry valid state assignments.
pub fn assign(doc: &mut Document, node: NodeId, kind: ControlKind) -> Result<Assigned, String> {
    let layer = doc.node(node).ok_or("The layer is missing.")?;
    if !layer.parent.is_some_and(|parent| panels(doc).contains(&parent)) {
        if panels(doc).is_empty() {
            return Err("This page has no Extron panel. Start with a new touch panel project.".into());
        }
        return Err(format!("\"{}\" must be directly inside an Extron panel.", layer.name));
    }
    if !kind.fits(&layer.kind) {
        return Err(format!("A {} must be {}.", kind.name(), kind.layer()));
    }
    let kept = Metadata::read(layer)
        .ok()
        .filter(|data| data.kind == kind.metadata())
        .and_then(|data| data.identity().ok())
        .filter(|identity| identity.control_id.is_some() == kind.addressable());
    if kind == ControlKind::Button {
        assign_states(doc, node)?;
    }
    match kept {
        Some(identity) => Ok(Assigned { node, key: identity.key, object_id: identity.object_id, control_id: identity.control_id }),
        None => identify(doc, node, kind),
    }
}

/// Give `node` a new key and IDs as a `kind` control.
fn identify(doc: &mut Document, node: NodeId, kind: ControlKind) -> Result<Assigned, String> {
    let object_id = allocate(doc)?;
    let layer = doc.node_mut(node).ok_or("The layer is missing.")?;
    let mut data = Metadata::new(layer, kind.metadata());
    let key = format!("studio.element-{object_id}");
    data.key = Some(key.clone());
    data.object_id = Some(object_id);
    data.control_id = kind.addressable().then_some(object_id);
    if kind == ControlKind::Slider {
        data.min = Some(0);
        data.max = Some(100);
    }
    demo::assign(layer, &data)?;
    Ok(Assigned { node, key, object_id, control_id: data.control_id })
}

/// The next Object ID: above every panel's stored counter and every ID in the document (a demo or
/// an older file has no counter yet). Every panel's counter then moves past it, so removing a page
/// never brings back the IDs it handed out. Once the counter reaches the top (a Control ID set
/// near 65534 moves it there), the lowest ID no layer holds is handed out instead.
fn allocate(doc: &mut Document) -> Result<u16, String> {
    let panels = panels(doc);
    let stored = panels
        .iter()
        .filter_map(|id| doc.node(*id))
        .filter_map(|frame| frame.plugin_data.get(NEXT_ID_KEY))
        .filter_map(|value| value.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    let used: std::collections::BTreeSet<u16> = doc
        .nodes()
        .filter_map(|n| n.plugin_data.get(KEY))
        .filter_map(|raw| serde_json::from_str::<Metadata>(raw).ok())
        .flat_map(|data| [data.object_id, data.control_id])
        .flatten()
        .collect();
    let highest = used.last().map_or(0, |id| u32::from(*id));
    let next = stored.max(highest + 1).max(u32::from(PAGE_OBJECT_ID) + 1);
    // 65535 is the donor's Offline popup (figma-extron-exporter `docs/exui-v2.md`).
    let Some(id) = u16::try_from(next).ok().filter(|id| *id < u16::MAX) else {
        return (PAGE_OBJECT_ID + 1..u16::MAX).find(|id| !used.contains(id)).ok_or_else(|| "No Object IDs remain in this project.".into());
    };
    for panel in panels {
        if let Some(frame) = doc.node_mut(panel) {
            frame.plugin_data.insert(NEXT_ID_KEY.into(), (next + 1).to_string());
        }
    }
    Ok(id)
}

/// Where a new button's variant set goes: right of its page, below the canvas's other layers.
fn variant_spot(doc: &Document, root: NodeId, panel: NodeId) -> [i32; 2] {
    let x = doc.node(panel).map_or(0, |frame| (frame.transform.as_coeffs()[4] + frame.width).ceil() as i32 + 96);
    let y = doc
        .node(root)
        .and_then(|page| {
            page.children
                .iter()
                .filter(|id| **id != panel)
                .filter_map(|id| doc.node(*id))
                .map(|n| (n.transform.as_coeffs()[5] + n.height).ceil() as i32 + 96)
                .max()
        })
        .unwrap_or(0);
    [x, y]
}

/// Give the variants of a button instance's set the three state roles, in order, where they lack
/// a valid state assignment.
fn assign_states(doc: &mut Document, instance: NodeId) -> Result<(), String> {
    let set = |doc: &Document| -> Option<Vec<NodeId>> {
        let NodeKind::Instance { component, .. } = doc.node(instance)?.kind else { return None };
        let set = doc.node(doc.node(component)?.parent?)?;
        matches!(set.kind, NodeKind::ComponentSet { .. }).then(|| set.children.clone())
    };
    let variants =
        set(doc).filter(|ids| ids.len() == 3 && ids.iter().all(|id| doc.node(*id).is_some_and(|n| matches!(n.kind, NodeKind::Component { .. }))));
    let Some(variants) = variants else {
        return Err("A button must be an instance of a variant set with three variants: Normal, Active and Pressed.".into());
    };
    for (role, id) in StateRole::ALL.into_iter().zip(variants) {
        let variant = doc.node_mut(id).ok_or("Button state variant is missing.")?;
        if Metadata::read(variant).is_ok_and(|data| data.kind == Kind::State && data.role.is_some_and(|r| Some(r.id()) == data.state_id)) {
            continue;
        }
        state(variant, role)?;
    }
    Ok(())
}

fn state(variant: &mut Node, role: StateRole) -> Result<(), String> {
    let mut data = Metadata::new(variant, Kind::State);
    data.role = Some(role);
    data.state_id = Some(role.id());
    data.state_name = Some(role.name().into());
    demo::assign(variant, &data)
}
