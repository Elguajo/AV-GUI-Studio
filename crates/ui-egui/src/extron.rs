//! Extron actions for the keyboard-accessible File menu and Actions palette.

use serde_json::json;

use crate::{
    FigApp,
    i18n::{tr, trf},
};

pub(crate) fn demo(app: &mut FigApp) {
    if app.run("extron.demo", json!({})).is_some() {
        app.ui.home = false;
        app.run("view.zoomSelection", json!({}));
        app.run("select.none", json!({}));
    }
}

pub(crate) fn validate(app: &mut FigApp) {
    let Some(result) = app.run("extron.validate", json!({})) else { return };
    if result["valid"] == true {
        app.toast(tr("The screen is ready for Extron export."));
    } else if let Some(message) = result["diagnostics"][0]["message"].as_str() {
        app.toast(trf("Extron validation failed. {reason}", &[("reason", &message)]));
    }
}

pub(crate) fn save(app: &mut FigApp, id: &str) {
    let Some((id, ext, title)) = (match id {
        "extron.exportExui" => Some(("extron.exportExui", "exui", tr("Export EXUI"))),
        "extron.buildGdl" => Some(("extron.buildGdl", "gdl", tr("Build GDL"))),
        _ => None,
    }) else {
        return;
    };
    let Ok(doc) = app.session.doc() else {
        app.toast(tr("Open an Extron screen first."));
        return;
    };
    let name = figcraft_engine::cmd::file::safe_file_name(&doc.name);
    app.pick_save(title, &format!("{name}.{ext}"), &[ext], move |app, path| {
        let Some(result) = app.run(id, json!({"path": path.to_string_lossy()})) else { return };
        if let Some(checks) = result["checks"].as_str() {
            app.toast(trf("GDL built. Verification {checks}.", &[("checks", &checks)]));
        } else {
            app.toast(tr("EXUI exported."));
        }
    });
}
