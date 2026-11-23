use crate::project::{Control, Project, StudioResult, persist, temporary_file};
use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Output},
};
use zip::{ZipWriter, write::SimpleFileOptions};

pub fn save(project: &Project, path: &Path) -> StudioResult<()> {
    project.validate()?;
    if project.elements.is_empty() {
        return Err("Add at least one element before exporting.".into());
    }
    let manifest = json!({
        "schema": "exui/5",
        "profile": "tlp-pro-1025m",
        "screen": "screen.json",
        "resources": [],
    });
    let elements: Vec<_> = project.elements.iter().map(|element| {
        let mut body = json!({
            "key": element.key(),
            "objectId": element.id,
            "controlId": element.control_id(),
            "name": element.name,
            "rect": element.bounds.0,
        });
        let font = |size: u8| json!({ "family": "Arial", "size": size, "bold": false });
        let props = match &element.control {
            Control::Button { states, size } => {
                let states: Vec<Value> = states.iter().enumerate().map(|(id, state)| {
                    let name = ["Normal", "Active", "Pressed"][id];
                    let role = ["normal", "feedback", "pressed"][id];
                    json!({
                        "stateId": id, "name": name, "role": role,
                        "text": state.text, "fill": state.fill.hex(),
                        "textColor": state.color.hex(), "textAlign": "middle-center",
                    })
                }).collect();
                json!({ "kind": "button", "font": font(*size), "states": states, "press": "Pressed" })
            }
            Control::Slider { fill, track, thumb, min, max } => {
                let [_, _, w, h] = element.bounds.0;
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
                let [_, _, w, h] = element.bounds.0;
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
        "page": { "key": "page.home", "objectId": 100, "name": "Home", "background": project.background.hex() },
        "elements": elements, "assets": [],
    });
    let mut file = temporary_file(path)?;
    let mut archive = ZipWriter::new(file.as_file_mut());
    for (name, value) in [("manifest.json", manifest), ("screen.json", screen)] {
        archive
            .start_file(name, SimpleFileOptions::default())
            .map_err(|error| error.to_string())?;
        let bytes = serde_json::to_vec_pretty(&value).map_err(|error| error.to_string())?;
        archive
            .write_all(&bytes)
            .map_err(|error| error.to_string())?;
    }
    archive.finish().map_err(|error| error.to_string())?;
    persist(file, path)
}

pub fn build_gdl(
    project: &Project,
    output: &Path,
    cli: &Path,
    registry: &Path,
) -> StudioResult<String> {
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    let package = directory.path().join("screen.exui");
    save(project, &package)?;
    let mut command = Command::new("dotnet");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let result = command
        .arg(cli)
        .arg("convert-package")
        .arg(&package)
        .arg(output)
        .arg("--registry")
        .arg(registry)
        .output()
        .map_err(|error| format!("Could not launch the Extron exporter: {error}"))?;
    conversion_report(result)
}

fn conversion_report(result: Output) -> StudioResult<String> {
    let stdout = String::from_utf8_lossy(&result.stdout);
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!(
            "Extron exporter returned {}. {} {}",
            result.status,
            stderr.trim(),
            stdout.trim()
        ));
    }
    let report: Value = serde_json::from_str(&stdout)
        .map_err(|error| format!("Could not read the Extron conversion report: {error}"))?;
    let checks = report["checks"]
        .as_str()
        .ok_or("The Extron report has no verification results.")?;
    Ok(checks.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::Read};
    use zip::ZipArchive;

    #[test]
    fn writes_stable_native_identities() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("room.exui");
        let project = Project::demo();
        save(&project, &path).unwrap();
        let mut archive = ZipArchive::new(File::open(&path).unwrap()).unwrap();
        let mut bytes = String::new();
        archive
            .by_name("screen.json")
            .unwrap()
            .read_to_string(&mut bytes)
            .unwrap();
        let screen: Value = serde_json::from_str(&bytes).unwrap();
        let elements = screen["elements"].as_array().unwrap();
        let mute = elements
            .iter()
            .find(|element| element["name"] == "Mute")
            .unwrap();
        let project_mute = project
            .elements
            .iter()
            .find(|element| element.name == "Mute")
            .unwrap();
        assert_eq!(mute["objectId"], project_mute.id);
        assert_eq!(mute["controlId"], project_mute.control_id().unwrap());
        assert_eq!(mute["states"][1]["role"], "feedback");
        assert_eq!(mute["press"], "Pressed");
    }
}
