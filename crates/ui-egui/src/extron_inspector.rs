//! The Extron section of the Design panel: what one selected Extron layer is to the touch panel.
//! A control shows its Object ID and an editable Control ID, a slider its range, a button a
//! Normal / Active / Pressed switch for the state the canvas shows, and a page its touch panel.

use std::sync::Arc;

use egui::{Id, Ui};
use figcraft_doc::{Document, NodeId};
use figcraft_engine::components::variant_props;
use figcraft_engine::gui_studio_extron::{ControlInfo, ElementKind, StateRole, control_info};
use serde_json::json;

use crate::FigApp;
use crate::i18n::{tr, trf};
use crate::right::row;
use crate::widgets::{self, Lead};

/// The section for a selection of one Extron layer; whether it drew anything.
pub(crate) fn section(app: &mut FigApp, ui: &mut Ui, doc: &Arc<Document>, sel: &[NodeId]) -> bool {
    let [id] = *sel else { return false };
    let Some(info) = crate::selection_cache::cached(ui.ctx(), "extron control", doc, sel, &[], || control_info(doc, id)) else {
        return false;
    };
    widgets::section_header(ui, title(info.kind), &[], false);
    match info.kind {
        ElementKind::Page => page(ui, &info),
        ElementKind::Button | ElementKind::Slider => {
            ids(app, ui, id, &info);
            if info.kind == ElementKind::Slider {
                range(app, ui, id, &info);
            } else {
                states(app, ui, doc, id, &info);
            }
        }
        ElementKind::Label | ElementKind::Shape => ids(app, ui, id, &info),
        ElementKind::State => {}
    }
    ui.add_space(8.0);
    true
}

fn title(kind: ElementKind) -> &'static str {
    match kind {
        ElementKind::Page => tr("Extron page"),
        ElementKind::Button => tr("Extron button"),
        ElementKind::Slider => tr("Extron slider"),
        ElementKind::Label => tr("Extron label"),
        ElementKind::Shape => tr("Extron shape"),
        ElementKind::State => tr("Extron button state"),
    }
}

/// Object ID, which the project hands out, beside the Control ID GCP addresses the control by.
fn ids(app: &mut FigApp, ui: &mut Ui, id: NodeId, info: &ControlInfo) {
    let addressable = matches!(info.kind, ElementKind::Button | ElementKind::Slider);
    widgets::sublabels(ui, tr("Object ID"), addressable.then(|| tr("Control ID")));
    let col = widgets::col_width(ui.available_width());
    row(ui, 32.0, |ui| {
        let object = info.object_id.map(f64::from);
        ui.add_enabled_ui(false, |ui| {
            widgets::number_field(ui, Id::unique(("extron_object_id", id)), Lead::Named(tr("Object ID")), object, "", col, 1.0)
        })
        .response
        .on_disabled_hover_text(tr("Object IDs are handed out by the project and never reused"));
        if addressable {
            let control = info.control_id.map(f64::from);
            let edit = widgets::number_field(ui, Id::unique(("extron_control_id", id)), Lead::Named(tr("Control ID")), control, "", col, 1.0);
            app.field_run(edit, "extron.setControl", |v| json!({"ids": [id], "controlId": v.round()}));
        }
    });
}

/// A slider's minimum and maximum level.
fn range(app: &mut FigApp, ui: &mut Ui, id: NodeId, info: &ControlInfo) {
    widgets::sublabels(ui, tr("Minimum"), Some(tr("Maximum")));
    let col = widgets::col_width(ui.available_width());
    row(ui, 32.0, |ui| {
        let edit = widgets::number_field(ui, Id::unique(("extron_min", id)), Lead::Named(tr("Minimum")), info.min.map(f64::from), "", col, 1.0);
        app.field_run(edit, "extron.setControl", |v| json!({"ids": [id], "min": v.round()}));
        let edit = widgets::number_field(ui, Id::unique(("extron_max", id)), Lead::Named(tr("Maximum")), info.max.map(f64::from), "", col, 1.0);
        app.field_run(edit, "extron.setControl", |v| json!({"ids": [id], "max": v.round()}));
    });
}

/// Which of a button's states the canvas shows: the instance takes that state's variant. Export
/// writes every state whichever one is shown.
fn states(app: &mut FigApp, ui: &mut Ui, doc: &Document, id: NodeId, info: &ControlInfo) {
    if info.states.len() != StateRole::ALL.len() {
        return;
    }
    widgets::sublabels(ui, tr("State on canvas"), None);
    let labels = [tr("Normal"), tr("Active"), tr("Pressed")];
    let active = info.shown.map_or(0, StateRole::id);
    let mut picked = None;
    row(ui, 32.0, |ui| {
        ui.add_space(widgets::PAD);
        ui.scope(|ui| {
            ui.set_max_width(ui.available_width() - widgets::PAD_RIGHT);
            picked = widgets::segmented_text(ui, &labels, active);
        })
        .response
        .on_hover_text(tr("The state this button shows on the canvas. Export includes every state."));
    });
    let Some(index) = picked.filter(|i| *i != active) else { return };
    let Some(variant) = info.states.get(index).and_then(|(_, variant)| doc.node(*variant)) else { return };
    app.run("instance.setProperties", json!({"ids": [id], "properties": variant_props(&variant.name)}));
}

/// A page's touch panel and its resolution.
fn page(ui: &mut Ui, info: &ControlInfo) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let text = match info.profile {
        Some(p) => trf("{panel} · {width} × {height}", &[("panel", &p.name), ("width", &p.width), ("height", &p.height)]),
        None => tr("Unknown touch panel").to_owned(),
    };
    widgets::sublabels(ui, tr("Touch panel"), None);
    widgets::note(ui, &text, t.text);
}
