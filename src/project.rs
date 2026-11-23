use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs::File, io::Read, path::Path};
use tempfile::NamedTempFile;

pub const WIDTH: i32 = 1280;
pub const HEIGHT: i32 = 800;
pub const MAX_ELEMENTS: usize = 64;
const PAGE_ID: u16 = 100;
const MAX_PROJECT_SIZE: u64 = 1024 * 1024;

pub type StudioResult<T> = Result<T, String>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub version: u8,
    pub name: String,
    pub background: Color,
    pub elements: Vec<Element>,
    next_id: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Color(pub [u8; 3]);

impl Color {
    pub fn hex(self) -> String {
        let [r, g, b] = self.0;
        format!("#{r:02X}{g:02X}{b:02X}")
    }
}

pub const INK: Color = Color([230, 244, 243]);
pub const MINT: Color = Color([122, 234, 188]);
pub const DIM: Color = Color([148, 176, 184]);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Element {
    pub id: u16,
    pub name: String,
    pub bounds: Bounds,
    pub control: Control,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

/// A validation finding, attached to an element when one is responsible.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub element: Option<u16>,
    pub severity: Severity,
    pub message: String,
}

impl Element {
    pub fn key(&self) -> String {
        format!("studio.element-{}", self.id)
    }

    pub fn control_id(&self) -> Option<u16> {
        self.control.kind().addressable().then_some(self.id)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Bounds(pub [i32; 4]);

impl Bounds {
    pub fn clamped(self) -> Self {
        let [x, y, w, h] = self.0;
        let w = w.clamp(1, WIDTH);
        let h = h.clamp(1, HEIGHT);
        Self([x.clamp(0, WIDTH - w), y.clamp(0, HEIGHT - h), w, h])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Button,
    Slider,
    Label,
    Shape,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Button => "Button",
            Self::Slider => "Slider",
            Self::Label => "Label",
            Self::Shape => "Shape",
        }
    }

    pub fn addressable(self) -> bool {
        matches!(self, Self::Button | Self::Slider)
    }

    pub fn default_size(self) -> [i32; 2] {
        match self {
            Self::Button => [190, 72],
            Self::Slider => [272, 48],
            Self::Label => [216, 44],
            Self::Shape => [256, 172],
        }
    }

    pub fn create(self) -> Control {
        match self {
            Self::Button => button("Button", false),
            Self::Slider => Control::Slider {
                fill: MINT,
                track: Color([37, 64, 73]),
                thumb: INK,
                min: 0,
                max: 100,
            },
            Self::Label => label("Label", 22, INK),
            Self::Shape => Control::Shape {
                fill: Color([19, 40, 48]),
                radius: 14,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Control {
    Button {
        states: [ButtonState; 3],
        size: u8,
    },
    Slider {
        fill: Color,
        track: Color,
        thumb: Color,
        min: u16,
        max: u16,
    },
    Label {
        text: String,
        color: Color,
        size: u8,
    },
    Shape {
        fill: Color,
        radius: u8,
    },
}

impl Control {
    pub fn kind(&self) -> Kind {
        match self {
            Self::Button { .. } => Kind::Button,
            Self::Slider { .. } => Kind::Slider,
            Self::Label { .. } => Kind::Label,
            Self::Shape { .. } => Kind::Shape,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ButtonState {
    pub text: String,
    pub fill: Color,
    pub color: Color,
}

fn button(text: &str, active: bool) -> Control {
    let states = [
        (Color([33, 61, 69]), INK),
        (MINT, Color([12, 34, 32])),
        (Color([82, 175, 145]), Color([12, 34, 32])),
    ];
    let mut states = states.map(|(fill, color)| ButtonState {
        text: text.into(),
        fill,
        color,
    });
    if active {
        states[0] = ButtonState {
            text: text.into(),
            fill: Color([40, 95, 82]),
            color: MINT,
        };
    }
    Control::Button { states, size: 21 }
}

fn label(text: &str, size: u8, color: Color) -> Control {
    Control::Label {
        text: text.into(),
        color,
        size,
    }
}

impl Project {
    pub fn restore_snapshot(&mut self, mut snapshot: Self) {
        snapshot.next_id = snapshot.next_id.max(self.next_id);
        *self = snapshot;
    }

    pub fn new(name: &str) -> Self {
        Self {
            version: 1,
            name: name.into(),
            background: Color([10, 27, 35]),
            elements: Vec::new(),
            next_id: 8101,
        }
    }

    pub fn demo() -> Self {
        let mut project = Self::new("Aurora Meeting Room");
        let card = Kind::Shape.create();
        let elements = [
            (
                "Section",
                [60, 45, 650, 36],
                label("AURORA   /   ROOM CONTROL", 18, MINT),
            ),
            (
                "Room title",
                [60, 100, 860, 75],
                label("Meeting Room", 50, INK),
            ),
            (
                "Intro",
                [60, 184, 1080, 39],
                label("Choose a source and adjust the room audio.", 22, DIM),
            ),
            ("Source card", [60, 270, 704, 348], card.clone()),
            ("Audio card", [792, 270, 428, 348], card),
            (
                "Source title",
                [91, 302, 510, 45],
                label("Input source", 28, INK),
            ),
            (
                "Source hint",
                [91, 355, 540, 36],
                label("Select what appears on the display.", 17, DIM),
            ),
            ("HDMI", [91, 428, 201, 82], button("HDMI", true)),
            ("USB-C", [311, 428, 201, 82], button("USB-C", false)),
            ("Wireless", [531, 428, 201, 82], button("Wireless", false)),
            (
                "Audio title",
                [824, 302, 332, 45],
                label("Room audio", 28, INK),
            ),
            (
                "Volume label",
                [824, 355, 330, 36],
                label("Volume", 17, DIM),
            ),
            ("Volume", [824, 404, 363, 45], Kind::Slider.create()),
            ("Mute", [824, 477, 363, 62], button("Mute", false)),
            (
                "Footer",
                [60, 702, 750, 36],
                label("AURORA   /   AV SYSTEMS", 16, DIM),
            ),
        ];
        for (name, bounds, control) in elements {
            project
                .insert(name.into(), Bounds(bounds), control)
                .expect("demo has capacity");
        }
        project
    }

    pub fn insert(&mut self, name: String, bounds: Bounds, control: Control) -> StudioResult<u16> {
        if self.elements.len() >= MAX_ELEMENTS {
            return Err(format!(
                "This screen supports up to {MAX_ELEMENTS} elements."
            ));
        }
        let id = u16::try_from(self.next_id).map_err(|_| "No object IDs remain.")?;
        self.next_id += 1;
        self.elements.push(Element {
            id,
            name,
            bounds: bounds.clamped(),
            control,
        });
        Ok(id)
    }

    /// Every validation finding. Errors block save and export; warnings do not.
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        let mut found = Vec::new();
        let mut project_error = |message: String| {
            found.push(Diagnostic {
                element: None,
                severity: Severity::Error,
                message,
            });
        };
        if self.version != 1 {
            project_error(format!(
                "Unsupported GUI Studio document version {}.",
                self.version
            ));
        }
        if !valid_name(&self.name) {
            project_error("Project name needs 1–256 characters.".into());
        }
        if self.elements.len() > MAX_ELEMENTS {
            project_error(format!(
                "This screen supports up to {MAX_ELEMENTS} elements."
            ));
        }
        if !(u32::from(PAGE_ID) + 1..=u32::from(u16::MAX) + 1).contains(&self.next_id) {
            project_error("Invalid next object ID.".into());
        }
        let mut ids = HashSet::from([PAGE_ID]);
        for element in &self.elements {
            let mut report = |severity, reason: &str| {
                found.push(Diagnostic {
                    element: Some(element.id),
                    severity,
                    message: format!("{} ({}) — {reason}", element.name, element.id),
                });
            };
            if !valid_name(&element.name) {
                report(Severity::Error, "Name needs 1–256 characters.");
            }
            if element.id <= PAGE_ID
                || u32::from(element.id) >= self.next_id
                || !ids.insert(element.id)
            {
                report(Severity::Error, "Duplicate or invalid object/control ID.");
            }
            if element.bounds.clamped() != element.bounds {
                report(
                    Severity::Error,
                    "The element must fit inside the 1280 × 800 screen.",
                );
            }
            match &element.control {
                Control::Button { states, size } => {
                    if !valid_font(*size)
                        || states.iter().any(|state| state.text.chars().count() > 256)
                    {
                        report(
                            Severity::Error,
                            "Button text must fit in 256 characters and use a font size of 4–200.",
                        );
                    }
                    if states[0] == states[1] || states[0] == states[2] || states[1] == states[2] {
                        report(Severity::Error, "Button states need distinct appearances.");
                    }
                }
                Control::Label { text, size, .. } if !valid_name(text) || !valid_font(*size) => {
                    report(
                        Severity::Error,
                        "Label needs 1–256 characters and a font size of 4–200.",
                    );
                }
                Control::Slider { min, max, .. } if min >= max => {
                    report(Severity::Error, "Slider maximum must exceed its minimum.");
                }
                _ => {}
            }
        }
        found
    }

    pub fn open(path: &Path) -> StudioResult<Self> {
        let file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let mut data = Vec::new();
        file.take(MAX_PROJECT_SIZE + 1)
            .read_to_end(&mut data)
            .map_err(|error| error.to_string())?;
        if data.len() as u64 > MAX_PROJECT_SIZE {
            return Err("Project file exceeds 1 MiB.".into());
        }
        let project: Self = serde_json::from_slice(&data)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        project.validate()?;
        Ok(project)
    }

    pub fn save(&self, path: &Path) -> StudioResult<()> {
        self.validate()?;
        let mut file = temporary_file(path)?;
        serde_json::to_writer_pretty(file.as_file_mut(), self)
            .map_err(|error| error.to_string())?;
        persist(file, path)
    }

    /// Fails with the first error from [`Project::diagnostics`].
    pub fn validate(&self) -> StudioResult<()> {
        match self
            .diagnostics()
            .into_iter()
            .find(|item| item.severity == Severity::Error)
        {
            Some(item) => Err(item.message),
            None => Ok(()),
        }
    }
}

fn valid_name(text: &str) -> bool {
    !text.trim().is_empty() && text.chars().count() <= 256
}

fn valid_font(size: u8) -> bool {
    (4..=200).contains(&size)
}

pub(crate) fn temporary_file(path: &Path) -> StudioResult<NamedTempFile> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    NamedTempFile::new_in(parent).map_err(|error| format!("{}: {error}", path.display()))
}

pub(crate) fn persist(file: NamedTempFile, path: &Path) -> StudioResult<()> {
    file.as_file()
        .sync_all()
        .map_err(|error| error.to_string())?;
    file.persist(path)
        .map(|_| ())
        .map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_reopen_and_restyle_retains_mute_identity() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("room.guistudio");
        let mut project = Project::demo();
        let mute = project
            .elements
            .iter()
            .find(|item| item.name == "Mute")
            .unwrap()
            .clone();
        project.elements.reverse();
        let edited = project
            .elements
            .iter_mut()
            .find(|item| item.id == mute.id)
            .unwrap();
        if let Control::Button { states, .. } = &mut edited.control {
            states[1].fill = Color([55, 120, 165]);
        }
        project.save(&path).unwrap();
        assert_eq!(Project::open(&path).unwrap(), project);
        let edited = project
            .elements
            .iter()
            .find(|item| item.id == mute.id)
            .unwrap();
        assert_eq!(edited.key(), mute.key());
        assert_eq!(edited.control_id(), mute.control_id());
        assert_ne!(edited.control, mute.control);
        project.elements.retain(|item| item.id != mute.id);
        assert_ne!(
            project
                .insert("Replacement".into(), mute.bounds, Kind::Button.create())
                .unwrap(),
            mute.id
        );
    }

    #[test]
    fn diagnostics_report_every_problem_at_its_element() {
        let mut project = Project::demo();
        project.elements[2].name = " ".into();
        project.elements[3].bounds = Bounds([1279, 0, 2, 2]);
        let found = project.diagnostics();
        for index in [2, 3] {
            assert!(
                found
                    .iter()
                    .any(|item| item.element == Some(project.elements[index].id)
                        && item.severity == Severity::Error)
            );
        }
        assert_eq!(project.validate().unwrap_err(), found[0].message);
    }

    #[test]
    fn rejects_duplicate_identity_and_off_screen_geometry() {
        let mut project = Project::demo();
        let first = project.elements[0].clone();
        project.elements.push(first);
        assert!(project.validate().unwrap_err().contains("Duplicate"));
        project.elements.pop();
        project.elements[0].bounds = Bounds([1279, 0, 2, 2]);
        assert!(project.validate().unwrap_err().contains("Section"));
    }
}
