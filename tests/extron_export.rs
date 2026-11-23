use gui_studio::{
    exui,
    project::{Color, Control, Project},
};
use std::{env, path::PathBuf, process::Command};

#[test]
#[ignore = "requires the existing Extron exporter, donor registry and GUI Designer"]
fn verifies_both_versions_and_mute_control_identity() {
    let setting =
        |name: &str| PathBuf::from(env::var_os(name).unwrap_or_else(|| panic!("Set {name}")));
    let cli = setting("GUI_STUDIO_EXTRON_CLI");
    let registry = setting("GUI_STUDIO_DONOR_REGISTRY");
    let directory = tempfile::tempdir().unwrap();
    let mut project = Project::demo();
    let first = directory.path().join("aurora-v1.gdl");
    let second = directory.path().join("aurora-v2.gdl");

    let checked = exui::build_gdl(&project, &first, &cli, &registry).unwrap();
    assert!(checked.ends_with(" passed"), "{checked}");
    let mute = project
        .elements
        .iter_mut()
        .find(|object| object.name == "Mute")
        .unwrap();
    let Control::Button { states, .. } = &mut mute.control else {
        panic!("Expected Mute button")
    };
    states[1].fill = Color([55, 120, 165]);
    let checked = exui::build_gdl(&project, &second, &cli, &registry).unwrap();
    assert!(checked.ends_with(" passed"), "{checked}");

    let read_mute = |path: &std::path::Path| {
        let result = Command::new("dotnet")
            .arg(&cli)
            .arg("list-controls")
            .arg(path)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        report["model"]["Pages"][0]["Controls"]
            .as_array()
            .unwrap()
            .iter()
            .find(|control| control["Name"] == "Mute")
            .unwrap()
            .clone()
    };
    let original = read_mute(&first);
    let revised = read_mute(&second);
    assert_eq!(original["Type"], "PBButton");
    for field in ["ObjectId", "ControlId", "Comments", "PressFeedbackState"] {
        assert_eq!(original[field], revised[field], "{field}");
    }
    assert_ne!(original["States"], revised["States"]);
}
