//! EXUI and GDL commands shared by the desktop and the headless CLI.

use std::path::PathBuf;

use serde_json::{Value, json};

use crate::cmd::CommandSpec;
use crate::file_access;
use crate::params::req_string;
use crate::{DocState, Result, Session, bad, spec};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        spec!("extron.demo", "New 1025M demo", "{}", false, demo),
        spec!("extron.validate", "Validate screen", "{}", false, validate),
        spec!("extron.exportExui", "Export EXUI…", "{path}", false, export),
        spec!("extron.buildGdl", "Build GDL…", "{path}", false, build),
    ]
}

fn demo(s: &mut Session, _: &Value) -> Result<Value> {
    let cmd = "extron.demo";
    let mut doc = gui_studio_extron::demo().map_err(|error| bad(cmd, error))?;
    let instances: Vec<_> = doc.nodes().filter(|node| matches!(node.kind, figcraft_doc::NodeKind::Instance { .. })).map(|node| node.id).collect();
    for id in instances {
        crate::components::sync_instance(&mut doc, id);
    }
    let project = gui_studio_extron::project(&doc).map_err(|items| bad(cmd, diagnostics(&items)))?;
    s.docs.push(DocState::new(doc, None));
    s.active = s.docs.len() - 1;
    s.set_selection(vec![project.page]);
    Ok(json!({"index": s.active, "panel": project.page}))
}

fn validate(s: &mut Session, _: &Value) -> Result<Value> {
    Ok(match gui_studio_extron::project(s.doc()?) {
        Ok(project) => json!({"valid": true, "elements": project.elements.len(), "diagnostics": []}),
        Err(items) => json!({"valid": false, "diagnostics": items}),
    })
}

fn diagnostics(items: &[gui_studio_extron::Diagnostic]) -> String {
    items.iter().map(|item| item.message.as_str()).collect::<Vec<_>>().join("\n")
}

fn output_path(cmd: &str, p: &Value, extension: &str) -> Result<PathBuf> {
    let path = PathBuf::from(req_string(cmd, p, "path")?);
    if !path.extension().and_then(std::ffi::OsStr::to_str).is_some_and(|ext| ext.eq_ignore_ascii_case(extension)) {
        return Err(bad(cmd, format!("Choose a .{extension} output path.")));
    }
    Ok(path)
}

fn export(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "extron.exportExui";
    let path = output_path(cmd, p, "exui")?;
    let project = gui_studio_extron::project(s.doc()?).map_err(|items| bad(cmd, diagnostics(&items)))?;
    let bytes = gui_studio_extron::package(&project).map_err(|error| bad(cmd, error))?;
    file_access::write(&path, &bytes)?;
    Ok(json!({"path": path, "elements": project.elements.len()}))
}

#[cfg(not(target_arch = "wasm32"))]
fn build(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "extron.buildGdl";
    if file_access::current().is_some() {
        return Err(bad(cmd, "GDL build requires the local desktop or an unconfined CLI run. Export EXUI from confined automation."));
    }
    let path = output_path(cmd, p, "gdl")?;
    let project = gui_studio_extron::project(s.doc()?).map_err(|items| bad(cmd, diagnostics(&items)))?;
    gui_studio_extron::build(&project, &path).map_err(|error| bad(cmd, error))
}

#[cfg(target_arch = "wasm32")]
fn build(_: &mut Session, _: &Value) -> Result<Value> {
    Err(bad("extron.buildGdl", "GDL build is available in the desktop app and native CLI."))
}
