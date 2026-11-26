//! What the editor shows and edits of one Extron layer: its type, IDs, slider range, a button's
//! state variants, and a page's touch panel.

use figcraft_doc::{Document, NodeId, NodeKind};

use crate::author::{NEXT_ID_KEY, panels};
use crate::metadata::{Kind, Metadata};
use crate::model::{Profile, StateRole, profile};

/// The highest Control ID a control can take: 65535 belongs to the donor's Offline popup
/// (figma-extron-exporter `docs/exui-v2.md`).
pub const MAX_CONTROL_ID: u16 = u16::MAX - 1;

/// An Extron layer as the inspector shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct ControlInfo {
    pub kind: Kind,
    pub key: Option<String>,
    pub object_id: Option<u16>,
    pub control_id: Option<u16>,
    /// A slider's range.
    pub min: Option<u16>,
    pub max: Option<u16>,
    /// A page's touch panel.
    pub profile: Option<&'static Profile>,
    /// A button's state variants, in Normal, Active, Pressed order, and the one its instance shows.
    pub states: Vec<(StateRole, NodeId)>,
    pub shown: Option<StateRole>,
}

/// The Extron assignment of `node`, or `None` for a layer without one (or with a copied one,
/// which needs its own IDs first).
pub fn control_info(doc: &Document, node: NodeId) -> Option<ControlInfo> {
    let layer = doc.node(node)?;
    let data = Metadata::read(layer).ok()?;
    if data.kind == Kind::State {
        return None;
    }
    let mut info = ControlInfo {
        kind: data.kind,
        key: data.key,
        object_id: data.object_id,
        control_id: data.control_id,
        min: data.min,
        max: data.max,
        profile: data.profile.as_deref().and_then(profile),
        states: Vec::new(),
        shown: None,
    };
    if let NodeKind::Instance { component, .. } = layer.kind
        && data.kind == Kind::Button
    {
        info.states = button_states(doc, component);
        info.shown = info.states.iter().find(|(_, id)| *id == component).map(|(role, _)| *role);
    }
    Some(info)
}

/// The state variants of the set `component` belongs to, by role.
fn button_states(doc: &Document, component: NodeId) -> Vec<(StateRole, NodeId)> {
    let Some(set) = doc.node(component).and_then(|c| c.parent).and_then(|id| doc.node(id)) else { return Vec::new() };
    if !matches!(set.kind, NodeKind::ComponentSet { .. }) {
        return Vec::new();
    }
    let mut states: Vec<(StateRole, NodeId)> = set
        .children
        .iter()
        .filter_map(|id| doc.node(*id))
        .filter_map(|variant| Some((Metadata::read(variant).ok().filter(|data| data.kind == Kind::State)?.role?, variant.id)))
        .collect();
    states.sort_by_key(|(role, _)| role.id());
    states
}

/// The addressable controls in `root`'s subtree that are shown, with their Control IDs.
pub fn control_ids(doc: &Document, root: NodeId) -> Vec<(NodeId, u16)> {
    doc.subtree(root)
        .into_iter()
        .filter_map(|id| doc.node(id))
        .filter(|n| n.visible && n.plugin_data.contains_key(crate::metadata::KEY))
        .filter_map(|n| Some((n.id, Metadata::read(n).ok().filter(|d| matches!(d.kind, Kind::Button | Kind::Slider))?.control_id?)))
        .collect()
}

/// What to change on a control; `None` leaves a value as it is.
#[derive(Clone, Copy, Debug, Default)]
pub struct ControlEdit {
    pub control_id: Option<u16>,
    pub min: Option<u16>,
    pub max: Option<u16>,
}

/// Change the Control ID or slider range of the control `node`. A Control ID another control has
/// is refused (EXUI/5 needs each to be unique), and every page's counter moves past a new one, so
/// a control inserted later never gets an ID a configuration may already use.
pub fn edit_control(doc: &mut Document, node: NodeId, edit: ControlEdit) -> Result<(), String> {
    let layer = doc.node(node).ok_or("The layer is missing.")?;
    let name = layer.name.clone();
    let mut data = Metadata::read(layer).map_err(|_| format!("\"{name}\" is not an Extron control."))?;
    if let Some(id) = edit.control_id {
        if !matches!(data.kind, Kind::Button | Kind::Slider) {
            return Err(format!("Only buttons and sliders have a Control ID; \"{name}\" is a {}.", kind_name(data.kind)));
        }
        if !(1..=MAX_CONTROL_ID).contains(&id) {
            return Err(format!("Control ID must be 1–{MAX_CONTROL_ID}."));
        }
        let owner = doc.nodes().filter(|n| n.id != node).find(|n| Metadata::read(n).is_ok_and(|d| d.control_id == Some(id)));
        if let Some(owner) = owner {
            return Err(format!("Control ID {id} is already used by \"{}\".", owner.name));
        }
        data.control_id = Some(id);
    }
    if edit.min.is_some() || edit.max.is_some() {
        if data.kind != Kind::Slider {
            return Err(format!("Only sliders have a range; \"{name}\" is a {}.", kind_name(data.kind)));
        }
        let (min, max) = (edit.min.or(data.min).unwrap_or(0), edit.max.or(data.max).unwrap_or(100));
        if min >= max {
            return Err(format!("Slider maximum must exceed its minimum ({min})."));
        }
        (data.min, data.max) = (Some(min), Some(max));
    }
    let layer = doc.node_mut(node).ok_or("The layer is missing.")?;
    data.write(layer).map_err(|e| e.to_string())?;
    if let Some(id) = edit.control_id {
        for panel in panels(doc) {
            let Some(frame) = doc.node_mut(panel) else { continue };
            let stored = frame.plugin_data.get(NEXT_ID_KEY).and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
            if stored <= u32::from(id) {
                frame.plugin_data.insert(NEXT_ID_KEY.into(), (u32::from(id) + 1).to_string());
            }
        }
    }
    Ok(())
}

/// The type of an Extron layer as messages and the inspector name it.
pub fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Page => "page",
        Kind::Button => "button",
        Kind::Slider => "slider",
        Kind::Label => "label",
        Kind::Shape => "shape",
        Kind::State => "button state",
    }
}
