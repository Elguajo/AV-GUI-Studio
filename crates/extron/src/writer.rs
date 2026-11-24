//! EXUI/5 writer adapted from GUI-Studio (3b7e5b7), src/exui.rs.

use std::io::{Cursor, Write};

use serde_json::{Value, json};
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::model::{Control, PROFILE, Project, StateRole};

pub fn package(project: &Project) -> Result<Vec<u8>, String> {
    if let Some(error) = project.diagnostics().into_iter().next() {
        return Err(error.message);
    }
    let manifest = json!({
        "schema": "exui/5",
        "profile": PROFILE,
        "screen": "screen.json",
        "resources": [],
    });
    let elements: Vec<_> = project.elements.iter().map(|element| {
        let mut body = json!({
            "key": element.identity.key,
            "objectId": element.identity.object_id,
            "controlId": element.identity.control_id,
            "name": element.name,
            "rect": element.bounds,
        });
        let font = |size: u8| json!({ "family": "Arial", "size": size, "bold": false });
        let props = match &element.control {
            Control::Button { states, size } => {
                let states: Vec<Value> = states.iter().enumerate().map(|(id, state)| {
                    let role = StateRole::ALL[id];
                    json!({
                        "stateId": id, "name": state.name, "role": role,
                        "text": state.text, "fill": state.fill.hex(),
                        "textColor": state.color.hex(), "textAlign": "middle-center",
                    })
                }).collect();
                json!({ "kind": "button", "font": font(*size), "states": states, "press": states[2]["name"] })
            }
            Control::Slider { fill, track, thumb, min, max } => {
                let [_, _, w, h] = element.bounds;
                let thumb_size = 26.min(w).min(h);
                let track_width = 12.min(h);
                json!({
                    "kind": "slider", "orientation": "left-to-right",
                    "min": min, "max": max, "fillColor": fill.hex(),
                    "native": {
                        "trackColor": track.hex(), "trackWidth": track_width,
                        "thumbColor": thumb.hex(), "thumbWidth": thumb_size,
                        "thumbHeight": thumb_size, "thumbType": "ellipse", "radius": track_width / 2,
                    },
                })
            }
            Control::Label { text, color, size } => json!({
                "kind": "label",
                "text": text, "font": font(*size), "textColor": color.hex(),
                "textAlign": "middle-left",
            }),
            Control::Shape { fill, radius } => {
                let [_, _, w, h] = element.bounds;
                json!({
                    "kind": "shape", "fillColor": fill.hex(),
                    "native": { "shape": "rectangle", "radius": i32::from(*radius).min(w.min(h) / 2), "strokeColor": "#00000000", "thickness": 0 },
                })
            }
        };
        if let (Some(target), Value::Object(fields)) = (body.as_object_mut(), props) {
            target.extend(fields);
        }
        body
    }).collect();
    let screen = json!({
        "schema": "exui-screen/5",
        "project": { "name": project.name },
        "page": { "key": project.identity.key, "objectId": project.identity.object_id, "name": project.page_name, "background": project.background.hex() },
        "elements": elements, "assets": [],
    });

    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, value) in [("manifest.json", manifest), ("screen.json", screen)] {
        archive.start_file(name, SimpleFileOptions::default()).map_err(|error| error.to_string())?;
        let bytes = serde_json::to_vec_pretty(&value).map_err(|error| error.to_string())?;
        archive.write_all(&bytes).map_err(|error| error.to_string())?;
    }
    archive.finish().map(Cursor::into_inner).map_err(|error| error.to_string())
}
