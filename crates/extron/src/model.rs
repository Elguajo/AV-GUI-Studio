//! Native controls and validation for a touch panel project: one or more pages of one panel model.
//! Adapted from AV-GUI-Studio (3b7e5b7), src/project.rs.

use std::collections::{HashMap, HashSet};

use figcraft_doc::NodeId;
use serde::Serialize;

pub const MAX_ELEMENTS: usize = 64;
/// Pages in one EXUI/5 project (figma-extron-exporter `docs/exui-v2.md`).
pub const MAX_PAGES: usize = 16;

/// A touch panel model the exporter can build a GDL for. The catalog follows the profiles of the
/// exporter's EXUI/5 manifest schema and its panel table (figma-extron-exporter
/// `docs/extron-format.md`, the panel profile section).
#[derive(Debug, Eq, PartialEq)]
pub struct Profile {
    /// The EXUI/5 manifest `profile`, kept in each page's assignment.
    pub id: &'static str,
    pub name: &'static str,
    pub part_number: &'static str,
    pub width: i32,
    pub height: i32,
}

pub const PROFILES: [Profile; 2] = [
    Profile { id: "tlp-pro-1025m", name: "TLP Pro 1025M", part_number: "60-1566-02", width: 1280, height: 800 },
    Profile { id: "tlp-pro-300m-portrait", name: "TLP Pro 300M (portrait)", part_number: "60-1667-02", width: 320, height: 480 },
];

/// The panel a project gets when none is named: the TLP Pro 1025M.
pub const DEFAULT_PROFILE: &Profile = &PROFILES[0];

pub fn profile(id: &str) -> Option<&'static Profile> {
    PROFILES.iter().find(|p| p.id == id)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color(pub [u8; 3]);

impl Color {
    pub fn hex(self) -> String {
        let [r, g, b] = self.0;
        format!("#{r:02X}{g:02X}{b:02X}")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StateRole {
    Normal,
    Feedback,
    Pressed,
}

impl StateRole {
    pub const ALL: [Self; 3] = [Self::Normal, Self::Feedback, Self::Pressed];

    pub fn id(self) -> usize {
        match self {
            Self::Normal => 0,
            Self::Feedback => 1,
            Self::Pressed => 2,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Feedback => "Active",
            Self::Pressed => "Pressed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ButtonState {
    pub name: String,
    pub text: String,
    pub fill: Color,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub enum Control {
    Button { states: [ButtonState; 3], size: u8 },
    Slider { fill: Color, track: Color, thumb: Color, min: u16, max: u16 },
    Label { text: String, color: Color, size: u8 },
    Shape { fill: Color, radius: u8 },
}

impl Control {
    fn addressable(&self) -> bool {
        matches!(self, Self::Button { .. } | Self::Slider { .. })
    }

    fn kind(&self) -> &'static str {
        match self {
            Self::Button { .. } => "button",
            Self::Slider { .. } => "slider",
            Self::Label { .. } => "label",
            Self::Shape { .. } => "shape",
        }
    }

    fn state_count(&self) -> usize {
        match self {
            Self::Button { states, .. } => states.len(),
            _ => 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Identity {
    pub key: String,
    pub object_id: u16,
    pub control_id: Option<u16>,
}

#[derive(Clone, Debug)]
pub struct Element {
    pub node: NodeId,
    pub identity: Identity,
    pub name: String,
    pub bounds: [i32; 4],
    pub control: Control,
}

/// One page of the project: a panel-sized artboard on the canvas.
#[derive(Debug)]
pub struct Page {
    pub node: NodeId,
    pub name: String,
    pub identity: Identity,
    pub background: Color,
    pub elements: Vec<Element>,
}

/// A touch panel project. `pages[0]` is the start page.
#[derive(Debug)]
pub struct Project {
    pub name: String,
    pub profile: &'static Profile,
    pub pages: Vec<Page>,
}

/// A problem that blocks export, reported at the layer to fix (`layer` is its name).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub node_id: NodeId,
    pub layer: String,
    pub severity: &'static str,
    pub message: String,
}

impl Diagnostic {
    pub(crate) fn error(node_id: NodeId, layer: impl Into<String>, message: impl Into<String>) -> Self {
        Self { node_id, layer: layer.into(), severity: "error", message: message.into() }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\" (node {}): {}", self.layer, self.node_id, self.message)
    }
}

pub(crate) fn valid_name(value: &str) -> bool {
    !value.trim().is_empty() && value.encode_utf16().count() <= 256 && !value.contains('\0')
}

fn valid_key(key: &str) -> bool {
    key.len() <= 100
        && key.starts_with(|c: char| c.is_ascii_lowercase())
        && key.split(['.', '-']).all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()))
}

impl Project {
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        let mut result = Vec::new();
        let (width, height) = (self.profile.width, self.profile.height);
        // The first owner of each key, ID and page name, named in the message on a later copy.
        let mut keys: HashMap<&str, &str> = HashMap::new();
        let mut objects: HashMap<u16, &str> = HashMap::new();
        let mut controls: HashMap<u16, &Element> = HashMap::new();
        let mut names: HashSet<&str> = HashSet::new();
        for (index, page) in self.pages.iter().enumerate() {
            let error = |message: &str| Diagnostic::error(page.node, &page.name, message);
            if index >= MAX_PAGES {
                result.push(error(&format!("A project holds up to {MAX_PAGES} pages.")));
            }
            if !valid_name(&self.name) || !valid_name(&page.name) {
                result.push(error("Project and page names need 1–256 characters."));
            } else if !names.insert(page.name.trim()) {
                result.push(error(&format!("Page name \"{}\" is already used by another page.", page.name)));
            }
            if !valid_key(&page.identity.key) || page.identity.object_id == 0 || page.identity.control_id.is_some() {
                result.push(error("Invalid page identity."));
            } else if let Some(owner) = keys.get(page.identity.key.as_str()) {
                result.push(error(&format!("Semantic key \"{}\" is already used by \"{owner}\".", page.identity.key)));
            } else if let Some(owner) = objects.get(&page.identity.object_id) {
                result.push(error(&format!("Object ID {} is already used by \"{owner}\".", page.identity.object_id)));
            } else {
                keys.insert(page.identity.key.as_str(), page.name.as_str());
                objects.insert(page.identity.object_id, page.name.as_str());
            }
            if !(1..=MAX_ELEMENTS).contains(&page.elements.len()) {
                result.push(error(&format!("A screen needs 1–{MAX_ELEMENTS} elements.")));
            }
            for item in &page.elements {
                let mut error = |reason: String| result.push(Diagnostic::error(item.node, &item.name, reason));
                if !valid_name(&item.name) {
                    error("Name needs 1–256 characters.".into());
                }
                if !valid_key(&item.identity.key) {
                    error("Semantic key needs lowercase letters and digits separated by dots or hyphens.".into());
                } else if let Some(owner) = keys.get(item.identity.key.as_str()) {
                    error(format!("Semantic key \"{}\" is already used by \"{owner}\".", item.identity.key));
                } else {
                    keys.insert(item.identity.key.as_str(), item.name.as_str());
                }
                if item.identity.object_id == 0 {
                    error("Object ID must be 1 or higher.".into());
                } else if let Some(owner) = objects.get(&item.identity.object_id) {
                    error(format!("Object ID {} is already used by \"{owner}\".", item.identity.object_id));
                } else {
                    objects.insert(item.identity.object_id, item.name.as_str());
                }
                match item.identity.control_id {
                    None if item.control.addressable() => error(format!("A {} needs a Control ID.", item.control.kind())),
                    Some(_) if !item.control.addressable() => error(format!("A {} cannot have a Control ID.", item.control.kind())),
                    Some(0) => error("Control ID must be 1 or higher.".into()),
                    Some(id) => match controls.get(&id) {
                        Some(owner) => error(shared_control_id(id, owner, item)),
                        None => {
                            controls.insert(id, item);
                        }
                    },
                    None => {}
                }
                let [x, y, w, h] = item.bounds.map(i64::from);
                if x < 0 || y < 0 || w <= 0 || h <= 0 || x + w > i64::from(width) || y + h > i64::from(height) {
                    error(format!("Element must fit inside the {width} × {height} screen."));
                }
                match &item.control {
                    Control::Button { states, size } => {
                        if !(4..=200).contains(size)
                            || states
                                .iter()
                                .any(|state| !valid_name(&state.name) || state.text.encode_utf16().count() > 256 || state.text.contains('\0'))
                        {
                            error("Button states need names, captions up to 256 characters and a font size of 4–200.".into());
                        }
                        let same_appearance = |a: &ButtonState, b: &ButtonState| a.text == b.text && a.fill == b.fill && a.color == b.color;
                        if same_appearance(&states[0], &states[1])
                            || same_appearance(&states[0], &states[2])
                            || same_appearance(&states[1], &states[2])
                        {
                            error("Button states need distinct appearances.".into());
                        }
                    }
                    Control::Label { text, size, .. } if !valid_name(text) || !(4..=200).contains(size) => {
                        error("Label needs 1–256 characters and a font size of 4–200.".into());
                    }
                    Control::Slider { min, max, .. } if min >= max => {
                        error("Slider maximum must exceed its minimum.".into());
                    }
                    _ => {}
                }
            }
        }
        result
    }
}

/// Why `item` cannot reuse the Control ID `id` of `owner`, with GUI Designer's rules (study §13.2):
/// only controls of one type, and buttons with the same number of states, may share an ID there.
/// EXUI/5 export still needs every Control ID in the project to be unique.
fn shared_control_id(id: u16, owner: &Element, item: &Element) -> String {
    let owner_name = &owner.name;
    if owner.control.kind() != item.control.kind() {
        format!("Control ID {id} is already used by \"{owner_name}\", a {}. Only controls of the same type can share IDs.", owner.control.kind())
    } else if owner.control.state_count() != item.control.state_count() {
        format!("Control ID {id} is already used by \"{owner_name}\", a button with a different number of states.")
    } else {
        format!("Control ID {id} is shared with \"{owner_name}\". EXUI/5 export needs a unique Control ID for each control on the screen.")
    }
}
