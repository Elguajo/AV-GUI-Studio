//! UI kits: the look of native controls (button states, slider, label, shape and the page
//! background) and ready-made blocks of controls, in one `gui-studio-kit/1` JSON file.
//!
//! A kit holds only what a native Extron control keeps in a GDL, so everything it makes exports.
//! Three kits are built in; others are installed from files. A project keeps a copy of its kit on
//! its pages, so it opens with the same look where that kit is not installed.

use serde::{Deserialize, Serialize};

use crate::model::{ButtonState, Color, Control, StateRole, valid_name};

pub const KIT_SCHEMA: &str = "gui-studio-kit/1";
/// The largest kit file read.
pub const MAX_KIT_BYTES: u64 = 256 * 1024;
const MAX_BLOCKS: usize = 32;
const MAX_BLOCK_ITEMS: usize = 64;

/// The kit a project gets when none is chosen.
pub const DEFAULT_KIT: &str = "aurora";

const BUILT_IN: [&str; 3] = [include_str!("../kits/aurora.json"), include_str!("../kits/daylight.json"), include_str!("../kits/graphite.json")];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Kit {
    pub schema: String,
    /// Lowercase letters, digits and hyphens; names the installed file.
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub author: String,
    /// Page background, `#RRGGBB`.
    pub background: String,
    pub button: ButtonStyle,
    pub slider: SliderStyle,
    pub label: LabelStyle,
    pub shape: ShapeStyle,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StateStyle {
    pub fill: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ButtonStyle {
    pub normal: StateStyle,
    pub active: StateStyle,
    pub pressed: StateStyle,
    /// Caption size in points.
    pub size: u8,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SliderStyle {
    pub track: String,
    pub fill: String,
    pub thumb: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LabelStyle {
    pub color: String,
    /// The colour of secondary text (a block item with `muted`).
    pub muted: String,
    /// Text size in points.
    pub size: u8,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ShapeStyle {
    pub fill: String,
    pub radius: u8,
}

/// Controls placed together, such as a card with a title and source buttons.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Width and height; every item lies inside.
    pub size: [i32; 2],
    pub items: Vec<BlockItem>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BlockItem {
    /// button, slider, label or shape.
    pub kind: String,
    pub name: String,
    /// Caption of a button or label (default: the layer name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// x, y (from the block's top left), width, height.
    pub rect: [i32; 4],
    /// A label's size in points instead of the kit's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u8>,
    /// A label in the kit's secondary text colour.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub muted: bool,
}

/// `#RRGGBB` as a colour.
pub(crate) fn color(hex: &str) -> Result<Color, String> {
    let digits = hex.strip_prefix('#').filter(|d| d.len() == 6 && d.bytes().all(|b| b.is_ascii_hexdigit()));
    let digits = digits.ok_or_else(|| format!("\"{hex}\" is not a #RRGGBB colour."))?;
    let channel = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).map_err(|e| e.to_string());
    Ok(Color([channel(0)?, channel(2)?, channel(4)?]))
}

fn font(size: u8, what: &str) -> Result<u8, String> {
    if (4..=200).contains(&size) { Ok(size) } else { Err(format!("{what} needs a size of 4–200 pt.")) }
}

impl Kit {
    /// Read and check a kit file.
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() as u64 > MAX_KIT_BYTES {
            return Err(format!("A UI kit file holds up to {} KiB.", MAX_KIT_BYTES / 1024));
        }
        let kit: Self = serde_json::from_slice(bytes).map_err(|error| format!("This is not a UI kit file: {error}"))?;
        kit.check()?;
        Ok(kit)
    }

    /// The built-in kits, in the order they are offered.
    pub fn built_in() -> Vec<Self> {
        BUILT_IN.iter().filter_map(|raw| Self::parse(raw.as_bytes()).ok()).collect()
    }

    pub fn is_built_in(id: &str) -> bool {
        Self::built_in().iter().any(|kit| kit.id == id)
    }

    /// The default kit, Aurora.
    pub fn default_kit() -> Result<Self, String> {
        Self::built_in().into_iter().find(|kit| kit.id == DEFAULT_KIT).ok_or_else(|| "The built-in Aurora kit is missing.".into())
    }

    pub fn check(&self) -> Result<(), String> {
        if self.schema != KIT_SCHEMA {
            return Err(format!("Unsupported UI kit schema \"{}\". This version reads {KIT_SCHEMA}.", self.schema));
        }
        let id_ok = (1..=64).contains(&self.id.len())
            && self.id.starts_with(|c: char| c.is_ascii_lowercase())
            && self.id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if !id_ok {
            return Err("A UI kit id needs 1–64 lowercase letters, digits and hyphens, starting with a letter.".into());
        }
        if !valid_name(&self.name) {
            return Err("A UI kit name needs 1–256 characters.".into());
        }
        color(&self.background)?;
        self.states()?;
        font(self.button.size, "A button caption")?;
        for hex in [&self.slider.track, &self.slider.fill, &self.slider.thumb, &self.label.color, &self.label.muted, &self.shape.fill] {
            color(hex)?;
        }
        font(self.label.size, "A label")?;
        if self.blocks.len() > MAX_BLOCKS {
            return Err(format!("A UI kit holds up to {MAX_BLOCKS} blocks."));
        }
        let mut names = std::collections::HashSet::new();
        for block in &self.blocks {
            let err = |why: String| Err(format!("Block \"{}\": {why}", block.name));
            if !valid_name(&block.name) || !names.insert(block.name.as_str()) {
                return Err("Every block needs its own name of 1–256 characters.".into());
            }
            let [w, h] = block.size;
            if w <= 0 || h <= 0 {
                return err("its size must be positive.".into());
            }
            if !(1..=MAX_BLOCK_ITEMS).contains(&block.items.len()) {
                return err(format!("it needs 1–{MAX_BLOCK_ITEMS} items."));
            }
            for item in &block.items {
                if !matches!(item.kind.as_str(), "button" | "slider" | "label" | "shape") {
                    return err(format!("\"{}\" is not a control kind. Use button, slider, label or shape.", item.kind));
                }
                if !valid_name(&item.name) || item.text.as_deref().is_some_and(|t| !valid_name(t)) {
                    return err("item names and captions need 1–256 characters.".into());
                }
                let [x, y, iw, ih] = item.rect;
                if x < 0 || y < 0 || iw <= 0 || ih <= 0 || x + iw > w || y + ih > h {
                    return err(format!("\"{}\" must lie inside the {w} × {h} block.", item.name));
                }
                if let Some(size) = item.size {
                    font(size, "A block label")?;
                }
            }
        }
        Ok(())
    }

    /// The three button states with `caption`.
    fn states(&self) -> Result<[ButtonState; 3], String> {
        let b = &self.button;
        let states = [&b.normal, &b.active, &b.pressed];
        let mut result = Vec::new();
        for (role, style) in StateRole::ALL.into_iter().zip(states) {
            result.push(ButtonState { name: role.name().into(), text: String::new(), fill: color(&style.fill)?, color: color(&style.text)? });
        }
        let same = |a: &ButtonState, b: &ButtonState| a.fill == b.fill && a.color == b.color;
        if same(&result[0], &result[1]) || same(&result[0], &result[2]) || same(&result[1], &result[2]) {
            return Err("The Normal, Active and Pressed button states need different colours.".into());
        }
        result.try_into().map_err(|_| "Missing button state.".into())
    }

    /// A button in this kit's look.
    pub(crate) fn button(&self, caption: &str) -> Result<Control, String> {
        let mut states = self.states()?;
        for state in &mut states {
            state.text = caption.into();
        }
        Ok(Control::Button { states, size: self.button.size })
    }

    pub(crate) fn background(&self) -> Result<Color, String> {
        color(&self.background)
    }

    /// Track, fill and thumb.
    pub(crate) fn slider(&self) -> Result<[Color; 3], String> {
        Ok([color(&self.slider.track)?, color(&self.slider.fill)?, color(&self.slider.thumb)?])
    }

    /// A label's colour and size.
    pub(crate) fn label(&self, muted: bool, size: Option<u8>) -> Result<(Color, u8), String> {
        let hex = if muted { &self.label.muted } else { &self.label.color };
        Ok((color(hex)?, size.unwrap_or(self.label.size)))
    }

    pub(crate) fn shape(&self) -> Result<(Color, u8), String> {
        Ok((color(&self.shape.fill)?, self.shape.radius))
    }

    pub fn block(&self, name: &str) -> Option<&Block> {
        self.blocks.iter().find(|block| block.name == name)
    }

    /// Colours that show the kit at a glance: background, button Normal and Active, shape.
    pub fn swatches(&self) -> [String; 4] {
        [self.background.clone(), self.button.normal.fill.clone(), self.button.active.fill.clone(), self.shape.fill.clone()]
    }
}
