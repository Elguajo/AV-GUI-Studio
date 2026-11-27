//! EXUI and GDL commands shared by the desktop and the headless CLI.

use std::path::{Path, PathBuf};

use gui_studio_extron::Kit;

use serde_json::{Value, json};

use crate::cmd::CommandSpec;
use crate::file_access;
use crate::params::{boolean, f64_of, get, req_string, req_targets, string};
use crate::{DocState, Result, Session, bad, spec};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        spec!("extron.demo", "New 1025M demo", "{}", false, demo),
        spec!(
            "extron.profiles",
            "Touch panels",
            "{} → {profiles: [{id, name, partNumber, width, height}]} (the panels a project can target)",
            false,
            profiles
        ),
        spec!(
            "extron.newPanel",
            "New touch panel project…",
            "{name? (project name, default \"Untitled panel\"), profile? (extron.profiles id, default tlp-pro-1025m), kit? (extron.kits id, default aurora)} → a new document with an empty Home page of the panel's resolution",
            false,
            new_panel
        ),
        spec!(
            "extron.addPage",
            "Add page",
            "{name? (default \"Page N\"), x?, y? (canvas top-left; default right of everything), select? (default true)} → {id, name, key, objectId}: a panel-sized artboard, as the Artboard tool makes",
            true,
            add_page
        ),
        spec!(
            "extron.insert",
            "Insert Extron control",
            "{kind: button|slider|label|shape, panel? (page artboard; default the selection's, else the one at the view centre, else the start page), x?, y? (in the page, default centred), name?, text? (button or label caption), kit? (whose look; default the project's), select? (default true)} → {id, key, objectId, controlId}",
            true,
            insert
        ),
        spec!(
            "extron.assign",
            "Assign Extron control",
            "{kind: button|slider|label|shape, ids?} (layers directly in the panel: a control of the same kind keeps its IDs, a copy or another kind gets new ones; a button is an instance of a 3-variant set) → {assigned: [{id, key, objectId, controlId}]}",
            true,
            assign
        ),
        spec!(
            "extron.kits",
            "UI kits",
            "{} → {kits: [{id, name, description, author, builtIn, swatches, blocks: [{name, description, width, height}]}], project (the open project's kit id, or null), problems: [{file, error}] (files in the kits folder that aren't kits)}",
            false,
            kits
        ),
        spec!(
            "extron.installKit",
            "Install UI kit…",
            "{path (a gui-studio-kit/1 .json or .guikit file)} → {id, name, replaced}: copies the kit into the app's kits folder (for this session only when it has none)",
            false,
            install_kit
        ),
        spec!("extron.removeKit", "Remove UI kit", "{id} → {removed}: an installed kit (built-in kits stay)", false, remove_kit),
        spec!("extron.useKit", "Use UI kit", "{id} → {kit}: the open project's controls from now on take this kit's look", true, use_kit),
        spec!(
            "extron.insertBlock",
            "Insert UI kit block",
            "{block (name), kit? (default the project's), panel? (default as extron.insert), x?, y? (block's top-left in the page, default centred), select? (default true)} → {ids, controls: [{id, key, objectId, controlId}]}",
            true,
            insert_block
        ),
        spec!(
            "extron.setControl",
            "Set Extron control IDs",
            "{ids?, controlId? (1–65534, unique; one control at a time), min?, max? (a slider's range, min below max)} → {controls: [{id, key, objectId, controlId, min, max}]}",
            true,
            set_control
        ),
        spec!("extron.validate", "Validate screen", "{}", false, validate),
        spec!("extron.exportExui", "Export EXUI…", "{path? (.exui)} → {path, pages, elements}; returns base64 `data` without a path", false, export),
        spec!("extron.buildGdl", "Build GDL…", "{path}", false, build),
        spec!(
            "extron.exporter",
            "GDL exporter",
            "{} → {cli, registry, source (environment|settings|sibling)}: the Extron exporter a GDL build uses, or why none was found",
            false,
            exporter
        ),
        spec!(
            "extron.setExporter",
            "Choose GDL exporter…",
            "{cli (Extron.Cli.dll), registry (donors.json, with prepared donors)} → {cli, registry, source}: kept in the preferences for later builds",
            false,
            set_exporter
        ),
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
    let panel = project.pages[0].node;
    s.set_selection(vec![panel]);
    Ok(json!({"index": s.active, "panel": panel}))
}

fn profiles(_: &mut Session, _: &Value) -> Result<Value> {
    let profiles: Vec<Value> = gui_studio_extron::PROFILES
        .iter()
        .map(|p| json!({"id": p.id, "name": p.name, "partNumber": p.part_number, "width": p.width, "height": p.height}))
        .collect();
    Ok(json!({"profiles": profiles}))
}

fn new_panel(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "extron.newPanel";
    let name = string(p, "name").unwrap_or_else(|| "Untitled panel".into());
    let panel = match string(p, "profile") {
        None => gui_studio_extron::DEFAULT_PROFILE,
        Some(id) => gui_studio_extron::profile(&id).ok_or_else(|| {
            let known: Vec<_> = gui_studio_extron::PROFILES.iter().map(|p| p.id).collect();
            bad(cmd, format!("Unknown touch panel \"{id}\". Use {}.", known.join(" or ")))
        })?,
    };
    let kit = match string(p, "kit") {
        Some(id) => kit_by_id(s, cmd, &id)?,
        None => Kit::default_kit().map_err(|error| bad(cmd, error))?,
    };
    let doc = gui_studio_extron::panel(&name, panel, &kit).map_err(|error| bad(cmd, error))?;
    let panel = gui_studio_extron::panel_frame(&doc).map_err(|error| bad(cmd, error))?;
    s.docs.push(DocState::new(doc, None));
    s.active = s.docs.len() - 1;
    s.set_selection(vec![panel]);
    Ok(json!({"index": s.active, "panel": panel}))
}

fn add_page(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "extron.addPage";
    let at = match (coordinate(cmd, p, "x")?, coordinate(cmd, p, "y")?) {
        (Some(x), Some(y)) => Some([x, y]),
        (None, None) => None,
        _ => return Err(bad(cmd, "Give both `x` and `y`, or neither.")),
    };
    let doc = s.doc_mut()?;
    let id = gui_studio_extron::add_page(doc, gui_studio_extron::NewPage { name: string(p, "name"), at }).map_err(|error| bad(cmd, error))?;
    let node = doc.node(id).ok_or(crate::EngineError::NoNode(id))?;
    let name = node.name.clone();
    let data: Value = node.plugin_data.get("exui").and_then(|raw| serde_json::from_str(raw).ok()).unwrap_or_default();
    if boolean(p, "select").unwrap_or(true) {
        s.set_selection(vec![id]);
    }
    Ok(json!({"id": id, "name": name, "key": data["key"], "objectId": data["objectId"]}))
}

/// The page a control goes on when none is named: the selection's, else the one under the view
/// centre, else the start page.
fn target_panel(s: &Session) -> Option<figcraft_doc::NodeId> {
    let ds = s.ds().ok()?;
    let doc = &ds.doc;
    if let Some(panel) = ds.selection.iter().find_map(|id| gui_studio_extron::panel_of(doc, *id)) {
        return Some(panel);
    }
    let centre = figcraft_geom::Point::new(ds.view.cx, ds.view.cy);
    gui_studio_extron::panels(doc).into_iter().find(|id| doc.bounds_of(&[*id]).is_some_and(|b| b.contains(centre)))
}

fn insert(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "extron.insert";
    let kind = control_kind(cmd, p)?;
    let panel = panel_param(s, cmd, p)?;
    let kit = string(p, "kit").map(|id| kit_by_id(s, cmd, &id)).transpose()?;
    let spec = gui_studio_extron::Insert {
        panel,
        name: string(p, "name"),
        text: string(p, "text"),
        x: coordinate(cmd, p, "x")?,
        y: coordinate(cmd, p, "y")?,
        kit,
        ..Default::default()
    };
    let doc = s.doc_mut()?;
    let added = gui_studio_extron::insert(doc, kind, spec, crate::components::sync_instance).map_err(|error| bad(cmd, error))?;
    if boolean(p, "select").unwrap_or(true) {
        s.set_selection(vec![added.node]);
    }
    Ok(identity(&added))
}

fn assign(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "extron.assign";
    let kind = control_kind(cmd, p)?;
    let targets = req_targets(cmd, s, p)?;
    let doc = s.doc_mut()?;
    let mut assigned = Vec::new();
    for id in targets {
        let done = gui_studio_extron::assign(doc, id, kind).map_err(|error| bad(cmd, format!("node {id}: {error}")))?;
        assigned.push(identity(&done));
    }
    Ok(json!({"assigned": assigned}))
}

/// `panel` when given, else the page a new control goes on.
fn panel_param(s: &Session, cmd: &str, p: &Value) -> Result<Option<figcraft_doc::NodeId>> {
    match get(p, "panel") {
        None | Some(Value::Null) => Ok(target_panel(s)),
        Some(value) => Ok(Some(crate::params::id_of(value).ok_or_else(|| bad(cmd, "`panel` must be a node id"))?)),
    }
}

/// The installed Extron UI kits: files in the app's folder `dir`, or kept for the session when
/// there is none (the CLI, tests). The built-in kits come first and can't be replaced.
#[derive(Debug, Default)]
pub struct KitStore {
    pub dir: Option<PathBuf>,
    session: Vec<Kit>,
}

/// Read a kit from the kits folder, refusing a file larger than a kit can be before reading it.
#[allow(clippy::disallowed_methods)]
fn read_kit(path: &Path) -> std::result::Result<Kit, String> {
    let size = std::fs::metadata(path).map_err(|error| error.to_string())?.len();
    if size > gui_studio_extron::MAX_KIT_BYTES {
        return Err(format!("A UI kit file holds up to {} KiB.", gui_studio_extron::MAX_KIT_BYTES / 1024));
    }
    std::fs::read(path).map_err(|error| error.to_string()).and_then(|bytes| Kit::parse(&bytes))
}

fn kit_file(path: &Path) -> bool {
    path.extension().and_then(std::ffi::OsStr::to_str).is_some_and(|ext| ext.eq_ignore_ascii_case("json") || ext.eq_ignore_ascii_case("guikit"))
}

// The kits folder is the app's own (set by the desktop app, never by a command), so automation
// roots don't apply to it; a file the user installs from is read through `file_access`.
#[allow(clippy::disallowed_methods)]
impl KitStore {
    /// The installed kits by name, and the files in the folder that aren't kits.
    pub fn installed(&self) -> (Vec<Kit>, Vec<(PathBuf, String)>) {
        let Some(dir) = &self.dir else { return (self.session.clone(), Vec::new()) };
        let (mut kits, mut problems) = (Vec::new(), Vec::new());
        let Ok(entries) = std::fs::read_dir(dir) else { return (kits, problems) };
        let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).filter(|path| path.is_file() && kit_file(path)).collect();
        paths.sort();
        for path in paths {
            let read = read_kit(&path);
            match read {
                Ok(kit) if Kit::is_built_in(&kit.id) => problems.push((path, format!("\"{}\" is a built-in kit's id.", kit.id))),
                Ok(kit) => kits.push(kit),
                Err(error) => problems.push((path, error)),
            }
        }
        kits.sort_by_key(|kit| kit.name.to_lowercase());
        (kits, problems)
    }

    /// Every kit, built-in ones first, with whether it is built in.
    pub fn all(&self) -> Vec<(Kit, bool)> {
        let mut all: Vec<(Kit, bool)> = Kit::built_in().into_iter().map(|kit| (kit, true)).collect();
        all.extend(self.installed().0.into_iter().map(|kit| (kit, false)));
        all
    }

    pub fn find(&self, id: &str) -> Option<Kit> {
        self.all().into_iter().map(|(kit, _)| kit).find(|kit| kit.id == id)
    }

    /// Keep `kit`; true when it replaced an installed kit of the same id.
    fn install(&mut self, kit: &Kit) -> std::result::Result<bool, String> {
        if Kit::is_built_in(&kit.id) {
            return Err(format!("\"{}\" is a built-in kit's id. Give the kit another id.", kit.id));
        }
        let replaced = self.installed().0.iter().any(|k| k.id == kit.id);
        match &self.dir {
            Some(dir) => {
                std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
                let bytes = serde_json::to_vec_pretty(kit).map_err(|error| error.to_string())?;
                std::fs::write(dir.join(format!("{}.json", kit.id)), bytes).map_err(|error| error.to_string())?;
            }
            None => {
                self.session.retain(|k| k.id != kit.id);
                self.session.push(kit.clone());
            }
        }
        Ok(replaced)
    }

    fn remove(&mut self, id: &str) -> std::result::Result<(), String> {
        if Kit::is_built_in(id) {
            return Err("Built-in kits can't be removed.".into());
        }
        match &self.dir {
            Some(dir) => {
                let mut removed = false;
                for path in std::fs::read_dir(dir).map_err(|error| error.to_string())?.flatten().map(|entry| entry.path()) {
                    let owns = kit_file(&path) && read_kit(&path).is_ok_and(|kit| kit.id == id);
                    if owns {
                        std::fs::remove_file(&path).map_err(|error| error.to_string())?;
                        removed = true;
                    }
                }
                if !removed {
                    return Err(format!("No installed UI kit is called \"{id}\"."));
                }
            }
            None => {
                let before = self.session.len();
                self.session.retain(|k| k.id != id);
                if self.session.len() == before {
                    return Err(format!("No installed UI kit is called \"{id}\"."));
                }
            }
        }
        Ok(())
    }
}

fn kit_by_id(s: &Session, cmd: &str, id: &str) -> Result<Kit> {
    s.kits.find(id).ok_or_else(|| bad(cmd, format!("No UI kit \"{id}\" is installed.")))
}

fn kits(s: &mut Session, _: &Value) -> Result<Value> {
    let kits: Vec<Value> = s
        .kits
        .all()
        .into_iter()
        .map(|(kit, built_in)| {
            let blocks: Vec<Value> = kit
                .blocks
                .iter()
                .map(|b| json!({"name": b.name, "description": b.description, "width": b.size[0], "height": b.size[1]}))
                .collect();
            json!({"id": kit.id, "name": kit.name, "description": kit.description, "author": kit.author, "builtIn": built_in, "swatches": kit.swatches(), "blocks": blocks})
        })
        .collect();
    let problems: Vec<Value> = s.kits.installed().1.into_iter().map(|(file, error)| json!({"file": file, "error": error})).collect();
    let project = s.doc().ok().and_then(|doc| gui_studio_extron::project_kit(doc).ok()).map(|kit| kit.id);
    Ok(json!({"kits": kits, "project": project, "problems": problems}))
}

fn install_kit(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "extron.installKit";
    let path = PathBuf::from(req_string(cmd, p, "path")?);
    if !kit_file(&path) {
        return Err(bad(cmd, "Choose a UI kit file (.json or .guikit)."));
    }
    let bytes = file_access::read_capped(&path, gui_studio_extron::MAX_KIT_BYTES)?;
    let kit = Kit::parse(&bytes).map_err(|error| bad(cmd, error))?;
    let replaced = s.kits.install(&kit).map_err(|error| bad(cmd, error))?;
    Ok(json!({"id": kit.id, "name": kit.name, "replaced": replaced}))
}

fn remove_kit(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "extron.removeKit";
    let id = req_string(cmd, p, "id")?;
    s.kits.remove(&id).map_err(|error| bad(cmd, error))?;
    Ok(json!({"removed": id}))
}

fn use_kit(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "extron.useKit";
    let kit = kit_by_id(s, cmd, &req_string(cmd, p, "id")?)?;
    gui_studio_extron::set_kit(s.doc_mut()?, &kit).map_err(|error| bad(cmd, error))?;
    Ok(json!({"kit": kit.id}))
}

fn insert_block(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "extron.insertBlock";
    let block = req_string(cmd, p, "block")?;
    let panel = panel_param(s, cmd, p)?;
    let kit = match string(p, "kit") {
        Some(id) => kit_by_id(s, cmd, &id)?,
        None => gui_studio_extron::project_kit(s.doc()?).map_err(|error| bad(cmd, error))?,
    };
    let at = match (coordinate(cmd, p, "x")?, coordinate(cmd, p, "y")?) {
        (Some(x), Some(y)) => Some([x, y]),
        (None, None) => None,
        _ => return Err(bad(cmd, "Give both `x` and `y`, or neither.")),
    };
    let doc = s.doc_mut()?;
    let placed = gui_studio_extron::insert_block(doc, &kit, &block, panel, at, crate::components::sync_instance).map_err(|error| bad(cmd, error))?;
    let ids: Vec<_> = placed.iter().map(|a| a.node).collect();
    if boolean(p, "select").unwrap_or(true) {
        s.set_selection(ids.clone());
    }
    Ok(json!({"ids": ids, "controls": placed.iter().map(identity).collect::<Vec<_>>()}))
}

fn control_kind(cmd: &str, p: &Value) -> Result<gui_studio_extron::ControlKind> {
    req_string(cmd, p, "kind")?.parse().map_err(|error: String| bad(cmd, error))
}

/// An optional whole-pixel panel coordinate; a value that isn't a finite number is an error.
fn coordinate(cmd: &str, p: &Value, key: &str) -> Result<Option<i32>> {
    match get(p, key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => f64_of(value)
            .filter(|v| v.is_finite() && (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(v))
            .map(|v| Some(v.round() as i32))
            .ok_or_else(|| bad(cmd, format!("`{key}` must be a number"))),
    }
}

fn identity(item: &gui_studio_extron::Assigned) -> Value {
    json!({"id": item.node, "key": item.key, "objectId": item.object_id, "controlId": item.control_id})
}

fn set_control(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "extron.setControl";
    let targets = req_targets(cmd, s, p)?;
    let edit = gui_studio_extron::ControlEdit { control_id: whole(cmd, p, "controlId")?, min: whole(cmd, p, "min")?, max: whole(cmd, p, "max")? };
    if edit.control_id.is_some() && targets.len() > 1 {
        return Err(bad(cmd, "Each control needs its own Control ID: set it on one control at a time."));
    }
    if edit.control_id.is_none() && edit.min.is_none() && edit.max.is_none() {
        return Err(bad(cmd, "Give controlId, min or max."));
    }
    let doc = s.doc_mut()?;
    let mut controls = Vec::new();
    for id in targets {
        gui_studio_extron::edit_control(doc, id, edit).map_err(|error| bad(cmd, format!("node {id}: {error}")))?;
        let info = gui_studio_extron::control_info(doc, id).ok_or_else(|| bad(cmd, format!("node {id}: the control is missing")))?;
        controls.push(json!({"id": id, "key": info.key, "objectId": info.object_id, "controlId": info.control_id, "min": info.min, "max": info.max}));
    }
    Ok(json!({"controls": controls}))
}

/// An optional whole number from 0 to 65535; anything else is an error.
fn whole(cmd: &str, p: &Value, key: &str) -> Result<Option<u16>> {
    match get(p, key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => f64_of(value)
            .filter(|v| v.fract() == 0.0 && (0.0..=f64::from(u16::MAX)).contains(v))
            .map(|v| Some(v as u16))
            .ok_or_else(|| bad(cmd, format!("`{key}` must be a whole number from 0 to 65535"))),
    }
}

fn validate(s: &mut Session, _: &Value) -> Result<Value> {
    Ok(validation(s.doc()?))
}

/// The `extron.validate` result for `doc`, without running a command (which would count as an
/// edit for redraws): `screen` says whether the document has Extron assignments at all, and each
/// diagnostic names the layer to fix by `nodeId` and `layer`.
pub fn validation(doc: &figcraft_doc::Document) -> Value {
    let screen = gui_studio_extron::is_screen(doc);
    match gui_studio_extron::project(doc) {
        Ok(project) => json!({"screen": screen, "valid": true, "pages": project.pages.len(), "elements": elements(&project), "diagnostics": []}),
        Err(items) => json!({"screen": screen, "valid": false, "diagnostics": items}),
    }
}

/// Controls on every page.
fn elements(project: &gui_studio_extron::Project) -> usize {
    project.pages.iter().map(|page| page.elements.len()).sum()
}

/// One line per problem, naming the layer and node to fix.
fn diagnostics(items: &[gui_studio_extron::Diagnostic]) -> String {
    items.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")
}

fn output_path(cmd: &str, p: &Value, extension: &str) -> Result<PathBuf> {
    let path = PathBuf::from(req_string(cmd, p, "path")?);
    if !path.extension().and_then(std::ffi::OsStr::to_str).is_some_and(|ext| ext.eq_ignore_ascii_case(extension)) {
        return Err(bad(cmd, format!("Choose a .{extension} output path.")));
    }
    Ok(path)
}

fn export(s: &mut Session, p: &Value) -> Result<Value> {
    use base64::Engine as _;
    let cmd = "extron.exportExui";
    let path = if string(p, "path").is_some() { Some(output_path(cmd, p, "exui")?) } else { None };
    let project = gui_studio_extron::project(s.doc()?).map_err(|items| bad(cmd, diagnostics(&items)))?;
    let bytes = gui_studio_extron::package(&project).map_err(|error| bad(cmd, error))?;
    let (pages, elements) = (project.pages.len(), elements(&project));
    let Some(path) = path else {
        // Where files are saved as downloads (the browser), the bytes go back to the caller.
        let data = base64::engine::general_purpose::STANDARD.encode(&bytes);
        return Ok(json!({"format": "exui", "pages": pages, "elements": elements, "data": data}));
    };
    file_access::write(&path, &bytes)?;
    Ok(json!({"path": path, "pages": pages, "elements": elements}))
}

#[cfg(not(target_arch = "wasm32"))]
fn build(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "extron.buildGdl";
    if file_access::current().is_some() {
        return Err(bad(cmd, "GDL build requires the local desktop or an unconfined CLI run. Export EXUI from confined automation."));
    }
    let path = output_path(cmd, p, "gdl")?;
    let project = gui_studio_extron::project(s.doc()?).map_err(|items| bad(cmd, diagnostics(&items)))?;
    let exporter = find_exporter(s).map_err(|error| bad(cmd, error))?;
    gui_studio_extron::build(&project, &path, &exporter).map_err(|error| bad(cmd, error))
}

/// The exporter a GDL build uses ([`gui_studio_extron::find_exporter`]): the environment, the
/// preferences, then a sibling checkout next to the program or the working folder.
#[cfg(not(target_arch = "wasm32"))]
pub fn find_exporter(s: &Session) -> std::result::Result<gui_studio_extron::Exporter, String> {
    let near: Vec<PathBuf> = [std::env::current_exe().ok().and_then(|exe| exe.parent().map(Path::to_path_buf)), std::env::current_dir().ok()]
        .into_iter()
        .flatten()
        .collect();
    let settings = s.prefs.extron_exporter.as_ref().map(|(cli, registry)| (Path::new(cli.as_str()), Path::new(registry.as_str())));
    gui_studio_extron::find_exporter(settings, &near)
}

#[cfg(not(target_arch = "wasm32"))]
fn described(found: &gui_studio_extron::Exporter) -> Value {
    json!({"cli": found.cli, "registry": found.registry, "source": found.source.name()})
}

#[cfg(not(target_arch = "wasm32"))]
fn exporter(s: &mut Session, _: &Value) -> Result<Value> {
    find_exporter(s).map(|found| described(&found)).map_err(|error| crate::EngineError::Disabled("extron.exporter".into(), error))
}

#[cfg(not(target_arch = "wasm32"))]
fn set_exporter(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "extron.setExporter";
    let (cli, registry) = (req_string(cmd, p, "cli")?, req_string(cmd, p, "registry")?);
    let found = gui_studio_extron::check_exporter(Path::new(&cli), Path::new(&registry), gui_studio_extron::ExporterSource::Settings)
        .map_err(|error| bad(cmd, error))?;
    s.prefs.extron_exporter = Some((cli, registry));
    s.prefs_unsaved = true;
    Ok(described(&found))
}

#[cfg(target_arch = "wasm32")]
fn exporter(_: &mut Session, _: &Value) -> Result<Value> {
    Err(bad("extron.exporter", "GDL build is available in the desktop app and native CLI."))
}

#[cfg(target_arch = "wasm32")]
fn set_exporter(_: &mut Session, _: &Value) -> Result<Value> {
    Err(bad("extron.setExporter", "GDL build is available in the desktop app and native CLI."))
}

#[cfg(target_arch = "wasm32")]
fn build(_: &mut Session, _: &Value) -> Result<Value> {
    Err(bad("extron.buildGdl", "GDL build is available in the desktop app and native CLI."))
}
