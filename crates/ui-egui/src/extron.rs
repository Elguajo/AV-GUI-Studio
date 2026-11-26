//! Extron actions for the keyboard-accessible File menu and Actions palette, and the Problems
//! section under the layers: every export problem of the open screen, kept current as it is
//! edited. Activating a problem selects its layer and brings it into view.

use std::sync::{Arc, Weak};

use egui::text::{LayoutJob, TextWrapping};
use egui::{Align2, Context, CornerRadius, Galley, Rect, RichText, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use figcraft_doc::{Document, NodeId};
use serde_json::{Value, json};

use figcraft_engine::gui_studio_extron::{DEFAULT_KIT, Kit, PROFILES, Profile};

use crate::FigApp;
use crate::a11y;
use crate::i18n::{tr, trf, trn};
use crate::theme::{Tokens, regular, semibold};

const HEADER: f32 = 40.0;

/// The diagnostics of the document snapshot they were made from (held weakly: an edit is a new
/// snapshot, so a different address means the list is out of date).
#[derive(Default)]
pub(crate) struct Problems {
    doc: Weak<Document>,
    /// The document has Extron assignments; other documents show no section.
    pub(crate) screen: bool,
    pub(crate) items: Vec<Problem>,
    pub(crate) collapsed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Problem {
    pub(crate) node: NodeId,
    pub(crate) layer: String,
    pub(crate) error: bool,
    pub(crate) message: String,
}

impl Problem {
    fn from_json(v: &Value) -> Option<Self> {
        Some(Self {
            node: NodeId(v["nodeId"].as_u64()?),
            layer: v["layer"].as_str().unwrap_or_default().to_owned(),
            error: v["severity"] != "warning",
            message: v["message"].as_str()?.to_owned(),
        })
    }

    /// The row's name for screen readers (and tests): the layer, then what to fix.
    pub(crate) fn label(&self) -> String {
        format!("{}: {}", self.layer, self.message)
    }
}

/// Validate the open document again when it changed since the list was made (or always, with
/// `force`). It reads the document directly: running `extron.validate` would count as an edit for
/// redraws on every change.
pub(crate) fn refresh(app: &mut FigApp, force: bool) {
    let Ok(ds) = app.session.ds() else {
        app.ui.problems = Problems { collapsed: app.ui.problems.collapsed, ..Problems::default() };
        return;
    };
    if !force && std::ptr::eq(app.ui.problems.doc.as_ptr(), Arc::as_ptr(&ds.doc)) {
        return;
    }
    let result = figcraft_engine::cmd::extron::validation(&ds.doc);
    let problems = &mut app.ui.problems;
    problems.doc = Arc::downgrade(&ds.doc);
    problems.screen = result["screen"] == true;
    problems.items = result["diagnostics"].as_array().map(|items| items.iter().filter_map(Problem::from_json).collect()).unwrap_or_default();
}

/// What the Problems section lists, for `ui.inspect`.
pub(crate) fn inspect(app: &FigApp) -> Value {
    let p = &app.ui.problems;
    let items: Vec<Value> = p
        .items
        .iter()
        .map(|i| json!({"nodeId": i.node.0, "layer": i.layer, "severity": if i.error { "error" } else { "warning" }, "message": i.message}))
        .collect();
    json!({"screen": p.screen, "collapsed": p.collapsed, "items": items})
}

/// Run `extron.demo` or `extron.newPanel` and show the new screen whole.
pub(crate) fn open_new(app: &mut FigApp, id: &str, params: Value) {
    if app.run(id, params).is_some() {
        app.ui.home = false;
        app.run("view.zoomSelection", json!({}));
        app.run("select.none", json!({}));
    }
}

pub(crate) fn validate(app: &mut FigApp) {
    if app.session.doc().is_err() {
        app.toast(tr("Open an Extron screen first."));
        return;
    }
    refresh(app, true);
    let problems = &mut app.ui.problems;
    let message = match problems.items.first() {
        None => tr("The screen is ready for Extron export.").to_owned(),
        Some(_) if problems.screen => {
            problems.collapsed = false;
            trn("{n} Extron problems. Select one in Problems to find its layer.", problems.items.len(), &[])
        }
        // Without Extron assignments there is no Problems section: the toast says what's missing.
        Some(first) => trf("Extron validation failed. {reason}", &[("reason", &first.message)]),
    };
    app.toast(message);
}

/// Select the layer of a problem and bring it into view: centred at the zoom the canvas has, or
/// zoomed to fit when it is larger than the view. A problem with the page itself opens the page.
pub(crate) fn reveal(app: &mut FigApp, node: NodeId) {
    let Ok(doc) = app.session.doc() else { return };
    if doc.pages.contains(&node) {
        app.run("page.select", json!({"id": node}));
        app.run("select.none", json!({}));
        return;
    }
    if app.run("select.set", json!({"ids": [node]})).is_none() {
        return;
    }
    let Ok(ds) = app.session.ds() else { return };
    let Some(b) = ds.doc.bounds_of(&[node]) else { return };
    let v = &ds.view;
    let zoom = v.zoom.max(1e-6);
    let inset = v.bottom_inset.clamp(0.0, v.vh.max(0.0));
    let (left, right) = (v.cx - v.vw / 2.0 / zoom, v.cx + v.vw / 2.0 / zoom);
    let (top, bottom) = (v.cy - v.vh / 2.0 / zoom, v.cy + (v.vh / 2.0 - inset) / zoom);
    if b.x0 >= left && b.x1 <= right && b.y0 >= top && b.y1 <= bottom {
        return;
    }
    if b.width() <= right - left && b.height() <= bottom - top {
        let center = b.center();
        app.run("view.set", json!({"cx": center.x, "cy": center.y + inset / 2.0 / zoom}));
    } else {
        app.run("view.zoomSelection", json!({}));
    }
}

/// The Problems section, docked under the layers of an Extron screen.
pub(crate) fn section(app: &mut FigApp, ui: &mut Ui) {
    refresh(app, false);
    if !app.ui.problems.screen {
        return;
    }
    let t = Tokens::get(ui.ctx());
    let w = ui.available_width();
    let text_w = (w - 48.0).max(40.0);
    let open = !app.ui.problems.collapsed;
    let rows: Vec<(Arc<Galley>, Arc<Galley>)> = if open {
        let painter = ui.painter();
        let galleys = |p: &Problem| {
            let mut name = LayoutJob::simple_singleline(p.layer.clone(), semibold(11.0), t.text);
            name.wrap = TextWrapping::truncate_at_width(text_w);
            (painter.layout_job(name), painter.layout(p.message.clone(), regular(11.0), t.text_secondary, text_w))
        };
        app.ui.problems.items.iter().map(galleys).collect()
    } else {
        Vec::new()
    };
    let row_h = |(name, message): &(Arc<Galley>, Arc<Galley>)| 6.0 + name.size().y + 2.0 + message.size().y + 8.0;
    let content = match (open, rows.is_empty()) {
        (false, _) => 0.0,
        (true, true) => 32.0,
        (true, false) => rows.iter().map(row_h).sum(),
    };
    let list_h = content.min((ui.available_height() * 0.4).max(96.0));
    let sel = app.session.ds().map(|ds| ds.selection.clone()).unwrap_or_default();
    let mut activated = None;
    egui::Panel::bottom("extron_problems").resizable(false).exact_size(HEADER + list_h).frame(egui::Frame::NONE).show(ui, |ui| {
        let rtl = crate::widgets::rtl(ui);
        let (head, resp) = ui.allocate_exact_size(vec2(w, HEADER), Sense::click());
        let fx = |x: f32| crate::widgets::flip_x(rtl, head, x);
        let (start, end) =
            if rtl { (egui::Align2::RIGHT_CENTER, egui::Align2::LEFT_CENTER) } else { (egui::Align2::LEFT_CENTER, egui::Align2::RIGHT_CENTER) };
        ui.painter().line_segment([head.left_top(), head.right_top()], Stroke::new(1.0, t.border));
        let n = app.ui.problems.items.len();
        ui.painter().text(pos2(fx(head.min.x + 16.0), head.center().y), start, tr("Problems"), semibold(11.0), t.text);
        let count = if n == 0 { t.text_secondary } else { t.danger };
        ui.painter().text(pos2(fx(head.max.x - 16.0), head.center().y), end, n.to_string(), semibold(11.0), count);
        a11y::disclosure(&resp, &trf("Problems: {n}", &[("n", &n)]), open);
        if resp.clicked() {
            app.ui.problems.collapsed = open;
        }
        if !open {
            return;
        }
        egui::ScrollArea::vertical().id_salt("extron_problems_rows").max_height(list_h).auto_shrink([false, false]).show(ui, |ui| {
            if rows.is_empty() {
                let (r, _) = ui.allocate_exact_size(vec2(w, 32.0), Sense::hover());
                let none = tr("No problems. The screen is ready for Extron export.");
                ui.painter().text(pos2(fx(r.min.x + 16.0), r.center().y), start, none, regular(11.0), t.text_secondary);
                return;
            }
            for (p, galleys) in app.ui.problems.items.iter().zip(&rows) {
                let (r, resp) = ui.allocate_exact_size(vec2(w, row_h(galleys)), Sense::click());
                if sel.contains(&p.node) {
                    ui.painter().rect_filled(r, CornerRadius::ZERO, t.selected_row);
                } else if resp.hovered() {
                    ui.painter().rect_filled(r, CornerRadius::ZERO, t.hover);
                }
                let (name, message) = galleys;
                let text_x = |g: &Galley| if rtl { r.max.x - 32.0 - g.size().x } else { r.min.x + 32.0 };
                let dot = pos2(fx(r.min.x + 20.0), r.min.y + 6.0 + name.size().y / 2.0);
                ui.painter().circle_filled(dot, 3.5, if p.error { t.danger } else { t.warning });
                ui.painter().galley(pos2(text_x(name), r.min.y + 6.0), name.clone(), t.text);
                ui.painter().galley(pos2(text_x(message), r.min.y + 8.0 + name.size().y), message.clone(), t.text_secondary);
                a11y::named_button(&resp, &p.label());
                if resp.clicked() {
                    activated = Some(p.node);
                }
            }
        });
    });
    if let Some(node) = activated {
        reveal(app, node);
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
    if app.ui.gdl_build.is_some() {
        app.toast(tr("A GDL is being built. Wait for it to finish."));
        return;
    }
    let Ok(doc) = app.session.doc() else {
        app.toast(tr("Open an Extron screen first."));
        return;
    };
    let name = figcraft_engine::cmd::file::safe_file_name(&doc.name);
    if id == "extron.buildGdl" {
        #[cfg(target_arch = "wasm32")]
        {
            app.toast(tr("GDL build is available in the desktop app."));
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Err(why) = figcraft_engine::cmd::extron::find_exporter(&app.session) {
            choose_exporter(app, true, Some(why));
            return;
        }
    }
    app.pick_save(title, &format!("{name}.{ext}"), &[ext], move |app, path| {
        #[cfg(not(target_arch = "wasm32"))]
        if id == "extron.buildGdl" {
            start_build(app, path);
            return;
        }
        if app.run(id, json!({"path": path.to_string_lossy()})).is_some() {
            app.toast(tr("EXUI exported."));
        }
    });
}

/// A GDL being built on a thread of its own: the file's name and the build.
pub struct GdlBuild {
    pub(crate) name: String,
    handle: std::thread::JoinHandle<Result<Value, String>>,
}

/// Build the open project's GDL at `path` on a worker thread: the exporter runs for several
/// seconds, and the window keeps working meanwhile. [`build_tick`] reports the result.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn start_build(app: &mut FigApp, path: std::path::PathBuf) {
    let Ok(doc) = app.session.doc() else { return };
    let project = match figcraft_engine::gui_studio_extron::project(doc) {
        Ok(project) => project,
        Err(items) => {
            refresh(app, true);
            let reason = items.first().map(ToString::to_string).unwrap_or_default();
            app.toast(trf("GDL build failed. {reason}", &[("reason", &reason)]));
            return;
        }
    };
    let exporter = match figcraft_engine::cmd::extron::find_exporter(&app.session) {
        Ok(exporter) => exporter,
        Err(why) => return app.toast(why),
    };
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let build = move || figcraft_engine::gui_studio_extron::build(&project, &path, &exporter);
    match std::thread::Builder::new().name("gui-studio-gdl".into()).spawn(build) {
        Ok(handle) => app.ui.gdl_build = Some(GdlBuild { name, handle }),
        Err(error) => app.toast(trf("GDL build failed. {reason}", &[("reason", &error)])),
    }
}

/// Once a frame: report a finished GDL build, or look again shortly while one runs.
pub(crate) fn build_tick(app: &mut FigApp, ctx: &Context) {
    let Some(build) = app.ui.gdl_build.take_if(|b| b.handle.is_finished()) else {
        if app.ui.gdl_build.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }
        return;
    };
    match build.handle.join() {
        Ok(Ok(result)) => {
            let checks = result["checks"].as_str().unwrap_or_default().to_owned();
            app.toast(trf("{name} built. Verification {checks}.", &[("name", &build.name), ("checks", &checks)]));
        }
        Ok(Err(reason)) => app.toast(trf("GDL build failed. {reason}", &[("reason", &reason)])),
        Err(_) => app.toast(trf("GDL build failed. {reason}", &[("reason", &"the build stopped unexpectedly")])),
    }
}

/// While a GDL is being built: a banner over the canvas saying so.
pub(crate) fn build_banner(app: &FigApp, ctx: &Context, canvas: Rect) {
    let Some(build) = &app.ui.gdl_build else { return };
    let t = Tokens::get(ctx);
    let text = trf("Building {name}…", &[("name", &build.name)]);
    widgets_banner(ctx, canvas, |ui| {
        ui.add(egui::Spinner::new().size(14.0));
        ui.label(RichText::new(text).font(regular(11.0)).color(t.text));
    });
}

fn widgets_banner(ctx: &Context, canvas: Rect, add: impl FnOnce(&mut Ui)) {
    crate::widgets::canvas_banner(ctx, egui::Id::unique("extron_gdl_build"), canvas, add);
}

/// File ▸ Extron ▸ Choose GDL exporter…: pick the exporter's Extron.Cli.dll, then its donor
/// registry, and keep them. With `then_build`, Build GDL goes on once they are chosen; `why` says
/// first why the exporter is asked for.
pub(crate) fn choose_exporter(app: &mut FigApp, then_build: bool, why: Option<String>) {
    if let Some(why) = why {
        app.toast(why);
    }
    app.pick_open(tr("Choose the Extron exporter (Extron.Cli.dll)"), &["dll"], move |app, cli| {
        app.pick_open(tr("Choose the exporter's donor registry (donors.json)"), &["json"], move |app, registry| {
            let params = json!({"cli": cli.to_string_lossy(), "registry": registry.to_string_lossy()});
            if app.run("extron.setExporter", params).is_none() {
                return;
            }
            if then_build {
                save(app, "extron.buildGdl");
            } else {
                app.toast(tr("GDL exporter chosen."));
            }
        });
    });
}

/// The New touch panel project dialog: GUI Designer's first wizard step, which picks the panel.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NewProject {
    pub(crate) name: String,
    /// Index into [`PROFILES`].
    pub(crate) panel: usize,
    /// The id of the UI kit the project starts with.
    pub(crate) kit: String,
}

/// File ▸ Extron ▸ New touch panel project… and the home screen's card.
pub(crate) fn open_new_project(app: &mut FigApp) {
    app.ui.new_project = Some(NewProject { name: tr("Untitled panel").to_owned(), panel: 0, kit: DEFAULT_KIT.to_owned() });
}

/// The panel's card: its screen drawn to scale (portrait or landscape), name, resolution and part
/// number. True when clicked.
fn panel_card(ui: &mut Ui, t: &Tokens, panel: &Profile, chosen: bool) -> bool {
    let (r, resp) = ui.allocate_exact_size(vec2(200.0, 176.0), Sense::click());
    a11y::named_button(&resp, panel.name);
    let painter = ui.painter();
    let fill = if chosen {
        t.selected_row
    } else if resp.hovered() {
        t.hover
    } else {
        t.panel
    };
    painter.rect(
        r,
        CornerRadius::same(8),
        fill,
        Stroke::new(if chosen { 2.0 } else { 1.0 }, if chosen { t.accent } else { t.border }),
        StrokeKind::Inside,
    );
    // The screen, to scale in a 120 × 84 box.
    let (w, h) = (panel.width as f32, panel.height as f32);
    let scale = (160.0 / w).min(84.0 / h);
    let screen = Rect::from_center_size(pos2(r.center().x, r.min.y + 16.0 + 42.0), vec2(w * scale, h * scale));
    painter.rect(screen, CornerRadius::same(3), t.text, Stroke::NONE, StrokeKind::Inside);
    painter.rect(screen.shrink(3.0), CornerRadius::same(2), t.accent, Stroke::NONE, StrokeKind::Inside);
    painter.text(pos2(r.center().x, r.min.y + 124.0), Align2::CENTER_CENTER, panel.name, semibold(12.0), t.text);
    let size = format!("{} × {}", panel.width, panel.height);
    painter.text(pos2(r.center().x, r.min.y + 142.0), Align2::CENTER_CENTER, size, regular(11.0), t.text_secondary);
    painter.text(pos2(r.center().x, r.min.y + 158.0), Align2::CENTER_CENTER, panel.part_number, regular(11.0), t.text_secondary);
    resp.clicked()
}

/// Pick the touch panel and name the project, then open it with an empty Home page.
pub(crate) fn new_project_dialog(app: &mut FigApp, ctx: &Context) {
    let Some(mut d) = app.ui.new_project.clone() else { return };
    let t = Tokens::get(ctx);
    d.panel = d.panel.min(PROFILES.len() - 1);
    let ready = !d.name.trim().is_empty();
    let kits = app.session.kits.all();
    if !kits.iter().any(|(kit, _)| kit.id == d.kit) {
        d.kit = DEFAULT_KIT.to_owned();
    }
    let (mut open, mut cancel, mut create) = (true, false, false);
    crate::dialogs::window(tr("New touch panel project"), ctx).id(egui::Id::unique("extron_new_project")).open(&mut open).show(ctx, |ui| {
        ui.set_width(416.0);
        ui.label(RichText::new(tr("Choose the touch panel. Every page of the project has its resolution.")).color(t.text_secondary));
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(16.0, 16.0);
            for (index, panel) in PROFILES.iter().enumerate() {
                if panel_card(ui, &t, panel, index == d.panel) {
                    d.panel = index;
                }
            }
        });
        ui.add_space(12.0);
        ui.label(tr("UI kit"));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            for (kit, _) in &kits {
                if kit_chip(ui, &t, kit, kit.id == d.kit) {
                    d.kit = kit.id.clone();
                }
            }
        });
        ui.add_space(12.0);
        ui.label(tr("Project name"));
        let name = ui.add(egui::TextEdit::singleline(&mut d.name).desired_width(400.0));
        crate::widgets::select_all_on_focus(ui, &name, &d.name);
        ui.add_space(4.0);
        ui.label(RichText::new(tr("Add more pages with the Artboard tool (Shift+O).")).color(t.text_secondary));
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            cancel = crate::widgets::button(ui, tr("Cancel"), false, true).clicked();
            let enter = name.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            create = (crate::widgets::button(ui, tr("Create"), true, ready).clicked() || enter) && ready;
        });
    });
    app.ui.new_project = (open && !cancel && !create).then(|| d.clone());
    if create {
        let profile = PROFILES[d.panel].id;
        open_new(app, "extron.newPanel", json!({"name": d.name.trim(), "profile": profile, "kit": d.kit}));
    }
}

/// `#RRGGBB` for painting; grey when it isn't one (a checked kit always has valid colours).
fn swatch(hex: &str) -> egui::Color32 {
    let channel = |i: usize| hex.get(i..i + 2).and_then(|c| u8::from_str_radix(c, 16).ok()).unwrap_or(128);
    egui::Color32::from_rgb(channel(1), channel(3), channel(5))
}

/// A kit's colours side by side in `r`.
fn paint_swatches(ui: &Ui, t: &Tokens, kit: &Kit, r: Rect) {
    let colors = kit.swatches();
    let w = r.width() / colors.len() as f32;
    for (i, hex) in colors.iter().enumerate() {
        let cell = Rect::from_min_size(pos2(r.min.x + w * i as f32, r.min.y), vec2(w, r.height()));
        ui.painter().rect_filled(cell, CornerRadius::ZERO, swatch(hex));
    }
    ui.painter().rect_stroke(r, CornerRadius::same(3), Stroke::new(1.0, t.border), StrokeKind::Inside);
}

/// A kit's name over its colours; true when clicked.
fn kit_chip(ui: &mut Ui, t: &Tokens, kit: &Kit, chosen: bool) -> bool {
    let (r, resp) = ui.allocate_exact_size(vec2(128.0, 56.0), Sense::click());
    a11y::named_button(&resp, &kit.name);
    let fill = if chosen {
        t.selected_row
    } else if resp.hovered() {
        t.hover
    } else {
        t.panel
    };
    let stroke = Stroke::new(if chosen { 2.0 } else { 1.0 }, if chosen { t.accent } else { t.border });
    ui.painter().rect(r, CornerRadius::same(6), fill, stroke, StrokeKind::Inside);
    paint_swatches(ui, t, kit, Rect::from_min_size(r.min + vec2(10.0, 10.0), vec2(r.width() - 20.0, 16.0)));
    let mut name = LayoutJob::simple_singleline(kit.name.clone(), semibold(11.0), t.text);
    name.wrap = TextWrapping::truncate_at_width(r.width() - 20.0);
    let galley = ui.painter().layout_job(name);
    ui.painter().galley(pos2(r.min.x + 10.0, r.min.y + 32.0), galley, t.text);
    resp.clicked()
}

/// The UI kits window: the kits to choose from, and the chosen one's controls and blocks to
/// insert on the page in front.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct KitsWindow {
    /// The id of the kit shown.
    pub(crate) kit: String,
}

/// File ▸ Extron ▸ UI kits…: opens on the project's kit.
pub(crate) fn open_kits(app: &mut FigApp) {
    let kit = app.session.doc().ok().and_then(|doc| figcraft_engine::gui_studio_extron::project_kit(doc).ok()).map(|kit| kit.id);
    app.ui.kits_window = Some(KitsWindow { kit: kit.unwrap_or_else(|| DEFAULT_KIT.to_owned()) });
}

/// Install a kit from a file the user picks, then show it.
fn install_kit(app: &mut FigApp) {
    app.pick_open(tr("Install UI kit"), &["guikit", "json"], |app, path| {
        let Some(result) = app.run("extron.installKit", json!({"path": path.to_string_lossy()})) else { return };
        let name = result["name"].as_str().unwrap_or_default().to_owned();
        if let (Some(window), Some(id)) = (app.ui.kits_window.as_mut(), result["id"].as_str()) {
            window.kit = id.to_owned();
        }
        app.toast(trf("Installed the {name} UI kit.", &[("name", &name)]));
    });
}

pub(crate) fn kits_dialog(app: &mut FigApp, ctx: &Context) {
    let Some(mut w) = app.ui.kits_window.clone() else { return };
    let t = Tokens::get(ctx);
    let kits = app.session.kits.all();
    let Some((kit, built_in)) = kits.iter().find(|(kit, _)| kit.id == w.kit).or_else(|| kits.first()).cloned() else { return };
    w.kit = kit.id.clone();
    let project = app
        .session
        .doc()
        .ok()
        .filter(|doc| figcraft_engine::gui_studio_extron::panel_frame(doc).is_ok())
        .and_then(|doc| figcraft_engine::gui_studio_extron::project_kit(doc).ok());
    let in_project = project.is_some();
    let current = project.as_ref().map(|k| k.id.clone());
    let mut open = true;
    let mut action: Option<(&'static str, Value)> = None;
    let (mut install, mut remove) = (false, false);
    crate::dialogs::floating(tr("UI kits"), ctx).id(egui::Id::unique("extron_kits")).open(&mut open).show(ctx, |ui| {
        ui.set_width(560.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            for (each, _) in &kits {
                if kit_chip(ui, &t, each, each.id == w.kit) {
                    w.kit = each.id.clone();
                }
            }
        });
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(&kit.name).strong());
            let tag = if built_in { tr("Built-in") } else { tr("Installed") };
            ui.label(RichText::new(tag).color(t.text_secondary));
            if current.as_deref() == Some(kit.id.as_str()) {
                ui.label(RichText::new(tr("This project's kit")).color(t.accent));
            }
        });
        if !kit.description.is_empty() {
            ui.label(RichText::new(&kit.description).color(t.text_secondary));
        }
        if !in_project {
            ui.label(RichText::new(tr("Open a touch panel project to insert from a kit.")).color(t.text_secondary));
        }
        ui.add_space(8.0);
        ui.label(tr("Controls"));
        ui.horizontal(|ui| {
            for (kind, label) in [("button", tr("Button")), ("slider", tr("Slider")), ("label", tr("Label")), ("shape", tr("Shape"))] {
                if crate::widgets::button(ui, label, false, in_project).clicked() {
                    action = Some(("extron.insert", json!({"kind": kind, "kit": kit.id})));
                }
            }
        });
        ui.add_space(8.0);
        ui.label(tr("Blocks"));
        egui::ScrollArea::vertical().id_salt("extron_kit_blocks").max_height(260.0).show(ui, |ui| {
            for block in &kit.blocks {
                ui.horizontal(|ui| {
                    if crate::widgets::button(ui, tr("Insert"), false, in_project).clicked() {
                        action = Some(("extron.insertBlock", json!({"block": block.name, "kit": kit.id})));
                    }
                    ui.vertical(|ui| {
                        ui.label(format!("{} · {} × {}", block.name, block.size[0], block.size[1]));
                        if !block.description.is_empty() {
                            ui.label(RichText::new(&block.description).color(t.text_secondary));
                        }
                    });
                });
                ui.add_space(4.0);
            }
        });
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            install = crate::widgets::button(ui, tr("Install kit…"), false, true).clicked();
            remove = crate::widgets::button(ui, tr("Remove kit"), false, !built_in).clicked();
            let use_it = in_project && current.as_deref() != Some(kit.id.as_str());
            if crate::widgets::button(ui, tr("Use for this project"), true, use_it).clicked() {
                action = Some(("extron.useKit", json!({"id": kit.id})));
            }
        });
    });
    app.ui.kits_window = open.then_some(w);
    if let Some((id, params)) = action
        && app.run(id, params).is_some()
        && id == "extron.useKit"
    {
        app.toast(trf("New controls take the {name} look. Placed controls keep theirs.", &[("name", &kit.name)]));
    }
    if install {
        install_kit(app);
    }
    if remove && app.run("extron.removeKit", json!({"id": kit.id})).is_some() {
        app.toast(trf("Removed the {name} UI kit.", &[("name", &kit.name)]));
        if let Some(window) = app.ui.kits_window.as_mut() {
            window.kit = DEFAULT_KIT.to_owned();
        }
    }
}
