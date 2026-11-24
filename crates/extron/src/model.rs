//! Native controls and validation for a single TLP Pro 1025M screen.
//! Adapted from GUI-Studio (3b7e5b7), src/project.rs.

use std::collections::HashSet;

use figcraft_doc::NodeId;
use serde::Serialize;

pub const PROFILE: &str = "tlp-pro-1025m";
pub const WIDTH: i32 = 1280;
pub const HEIGHT: i32 = 800;
pub const MAX_ELEMENTS: usize = 64;

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

#[derive(Debug)]
pub struct Project {
    pub page: NodeId,
    pub name: String,
    pub page_name: String,
    pub identity: Identity,
    pub background: Color,
    pub elements: Vec<Element>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub node_id: NodeId,
    pub severity: &'static str,
    pub message: String,
}

impl Diagnostic {
    pub(crate) fn error(node_id: NodeId, message: impl Into<String>) -> Self {
        Self { node_id, severity: "error", message: message.into() }
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
        if !valid_name(&self.name) || !valid_name(&self.page_name) {
            result.push(Diagnostic::error(self.page, "Project and page names need 1–256 characters."));
        }
        if !valid_key(&self.identity.key) || self.identity.object_id == 0 || self.identity.control_id.is_some() {
            result.push(Diagnostic::error(self.page, "Invalid page identity."));
        }
        if !(1..=MAX_ELEMENTS).contains(&self.elements.len()) {
            result.push(Diagnostic::error(self.page, format!("A screen needs 1–{MAX_ELEMENTS} elements.")));
        }
        let mut keys = HashSet::from([self.identity.key.as_str()]);
        let mut objects = HashSet::from([self.identity.object_id]);
        let mut controls = HashSet::new();
        for item in &self.elements {
            let mut error = |reason| result.push(Diagnostic::error(item.node, format!("{} — {reason}", item.name)));
            if !valid_name(&item.name) {
                error("Name needs 1–256 characters.");
            }
            if !valid_key(&item.identity.key) || !keys.insert(item.identity.key.as_str()) {
                error("Duplicate or invalid semantic key.");
            }
            if item.identity.object_id == 0 || !objects.insert(item.identity.object_id) {
                error("Duplicate or invalid Object ID.");
            }
            match item.identity.control_id {
                Some(id) if item.control.addressable() && id != 0 && controls.insert(id) => {}
                None if !item.control.addressable() => {}
                _ => error("Duplicate, invalid or missing Control ID."),
            }
            let [x, y, w, h] = item.bounds.map(i64::from);
            if x < 0 || y < 0 || w <= 0 || h <= 0 || x + w > i64::from(WIDTH) || y + h > i64::from(HEIGHT) {
                error("Element must fit inside the 1280 × 800 screen.");
            }
            match &item.control {
                Control::Button { states, size } => {
                    if !(4..=200).contains(size)
                        || states.iter().any(|state| !valid_name(&state.name) || state.text.encode_utf16().count() > 256 || state.text.contains('\0'))
                    {
                        error("Button states need names, captions up to 256 characters and a font size of 4–200.");
                    }
                    let same_appearance = |a: &ButtonState, b: &ButtonState| a.text == b.text && a.fill == b.fill && a.color == b.color;
                    if same_appearance(&states[0], &states[1]) || same_appearance(&states[0], &states[2]) || same_appearance(&states[1], &states[2]) {
                        error("Button states need distinct appearances.");
                    }
                }
                Control::Label { text, size, .. } if !valid_name(text) || !(4..=200).contains(size) => {
                    error("Label needs 1–256 characters and a font size of 4–200.");
                }
                Control::Slider { min, max, .. } if min >= max => {
                    error("Slider maximum must exceed its minimum.");
                }
                _ => {}
            }
        }
        result
    }
}
