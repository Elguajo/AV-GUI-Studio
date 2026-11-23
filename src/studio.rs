mod canvas;

use eframe::egui::{self, Color32, Context, Frame, Key, Margin, RichText, Ui};
use gui_studio::{
    exui,
    project::{Bounds, Color, Control, HEIGHT, Kind, Project, StudioResult, WIDTH},
};
use rfd::FileDialog;
use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    thread,
};

const TEXT: Color32 = Color32::from_rgb(234, 242, 241);
const MUTED: Color32 = Color32::from_rgb(152, 173, 184);
const ACCENT: Color32 = Color32::from_rgb(124, 239, 189);
const PANEL: Color32 = Color32::from_rgb(20, 27, 35);
const INSET: Color32 = Color32::from_rgb(32, 44, 54);
const KINDS: [Kind; 4] = [Kind::Button, Kind::Slider, Kind::Label, Kind::Shape];

pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1420.0, 900.0])
            .with_min_inner_size([1020.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "GUI Studio",
        options,
        Box::new(|creation| {
            let context = &creation.egui_ctx;
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = PANEL;
            visuals.window_fill = PANEL;
            visuals.extreme_bg_color = Color32::from_rgb(13, 19, 26);
            visuals.override_text_color = Some(TEXT);
            visuals.selection.bg_fill = Color32::from_rgb(39, 78, 72);
            visuals.selection.stroke = egui::Stroke::new(1.0_f32, ACCENT);
            visuals.widgets.inactive.bg_fill = INSET;
            visuals.widgets.hovered.bg_fill = Color32::from_rgb(46, 63, 73);
            visuals.widgets.active.bg_fill = Color32::from_rgb(46, 79, 76);
            context.set_visuals(visuals);
            context.style_mut(|style| {
                style.spacing.item_spacing = egui::vec2(8.0, 8.0);
                style.spacing.button_padding = egui::vec2(10.0, 5.0);
            });
            Ok(Box::new(Studio::new()))
        }),
    )
}

#[derive(Clone, Copy)]
enum Replace {
    New,
    Open,
    Close,
}

struct Exporter {
    cli: PathBuf,
    registry: PathBuf,
}

impl Exporter {
    fn discover() -> Option<Self> {
        if let (Some(cli), Some(registry)) = (
            std::env::var_os("GUI_STUDIO_EXTRON_CLI"),
            std::env::var_os("GUI_STUDIO_DONOR_REGISTRY"),
        ) {
            let exporter = Self {
                cli: cli.into(),
                registry: registry.into(),
            };
            return (exporter.cli.is_file() && exporter.registry.is_file()).then_some(exporter);
        }
        std::env::current_exe()
            .ok()?
            .ancestors()
            .find_map(|parent| {
                let sibling = parent.join("figma-extron-exporter");
                let cli = sibling.join("output/releases/0.8.8/smoke-kit/cli/Extron.Cli.dll");
                let registry = sibling.join("samples/donors.json");
                (cli.is_file() && registry.is_file()).then_some(Self { cli, registry })
            })
    }

    fn choose() -> Option<Self> {
        let cli = FileDialog::new()
            .set_title("Select Extron.Cli.dll")
            .add_filter("Extron CLI", &["dll"])
            .pick_file()?;
        let registry = FileDialog::new()
            .set_title("Select the Extron donor registry")
            .add_filter("Donor registry", &["json"])
            .pick_file()?;
        Some(Self { cli, registry })
    }
}

struct Studio {
    project: Project,
    saved: Project,
    path: Option<PathBuf>,
    selection: Option<u16>,
    state: usize,
    canvas: canvas::Canvas,
    undo: Vec<Project>,
    redo: Vec<Project>,
    pending: Option<Replace>,
    allow_close: bool,
    status: String,
    error: Option<String>,
    exporter: Option<Exporter>,
    conversion: Option<Receiver<StudioResult<String>>>,
}

impl Studio {
    fn new() -> Self {
        let project = Project::demo();
        let selection = project
            .elements
            .iter()
            .find(|element| element.name == "Mute")
            .map(|element| element.id);
        Self {
            saved: project.clone(),
            project,
            path: None,
            selection,
            state: 0,
            canvas: canvas::Canvas::default(),
            undo: Vec::new(),
            redo: Vec::new(),
            pending: None,
            allow_close: false,
            status: "Demo project ready".into(),
            error: None,
            exporter: Exporter::discover(),
            conversion: None,
        }
    }

    fn select(&mut self, selection: Option<u16>) {
        if self.selection != selection {
            self.state = 0;
            self.selection = selection;
        }
    }

    fn remember(&mut self, before: Project) {
        if before != self.project {
            if self.undo.len() == 60 {
                self.undo.remove(0);
            }
            self.undo.push(before);
            self.redo.clear();
        }
    }

    fn history(&mut self, undo: bool) {
        let (from, to) = if undo {
            (&mut self.undo, &mut self.redo)
        } else {
            (&mut self.redo, &mut self.undo)
        };
        if let Some(project) = from.pop() {
            to.push(self.project.clone());
            self.project.restore_snapshot(project);
            if !self
                .project
                .elements
                .iter()
                .any(|element| Some(element.id) == self.selection)
            {
                self.select(None);
            }
        }
    }

    fn replace(&mut self, intent: Replace, context: &Context) {
        if self.project != self.saved {
            self.pending = Some(intent);
        } else {
            self.run_replace(intent, context);
        }
    }

    fn run_replace(&mut self, intent: Replace, context: &Context) {
        let (project, path) = match intent {
            Replace::New => (Project::new("Untitled panel"), None),
            Replace::Open => {
                let Some(path) = project_dialog().pick_file() else {
                    return;
                };
                match Project::open(&path) {
                    Ok(project) => (project, Some(path)),
                    Err(error) => {
                        self.error = Some(error);
                        return;
                    }
                }
            }
            Replace::Close => {
                self.allow_close = true;
                context.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
        };
        self.saved = project.clone();
        self.project = project;
        self.path = path;
        self.select(None);
        self.canvas = canvas::Canvas::default();
        self.undo.clear();
        self.redo.clear();
        self.status = "Project ready".into();
    }

    fn add(&mut self, kind: Kind) {
        let previous = self.project.clone();
        let [w, h] = kind.default_size();
        let offset = 14 * (self.project.elements.len() as i32 % 8);
        let bounds = Bounds([
            WIDTH / 2 - w / 2 + offset,
            HEIGHT / 2 - h / 2 + offset,
            w,
            h,
        ]);
        match self
            .project
            .insert(kind.label().into(), bounds, kind.create())
        {
            Ok(id) => {
                self.select(Some(id));
                self.remember(previous);
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn remove(&mut self) {
        if let Some(id) = self.selection {
            let before = self.project.clone();
            self.project.elements.retain(|element| element.id != id);
            self.select(None);
            self.remember(before);
        }
    }

    fn save(&mut self, save_as: bool) -> bool {
        let path = match (&self.path, save_as) {
            (Some(path), false) => Some(path.clone()),
            _ => project_dialog()
                .set_file_name(file_name(&self.project.name, "guistudio"))
                .save_file(),
        };
        let Some(path) = path else { return false };
        match self.project.save(&path) {
            Ok(()) => {
                self.saved = self.project.clone();
                self.status = format!("Saved {}", path.display());
                self.path = Some(path);
                true
            }
            Err(error) => {
                self.error = Some(error);
                false
            }
        }
    }

    fn export(&mut self, context: &Context, gdl: bool) {
        let extension = if gdl { "gdl" } else { "exui" };
        let Some(path) = FileDialog::new()
            .add_filter(extension.to_uppercase(), &[extension])
            .set_file_name(file_name(&self.project.name, extension))
            .save_file()
        else {
            return;
        };
        if !gdl {
            match exui::save(&self.project, &path) {
                Ok(()) => self.status = format!("Exported {}", path.display()),
                Err(error) => self.error = Some(error),
            }
            return;
        }
        if path.exists() {
            self.error = Some("Choose a new GDL file name. Existing files are preserved.".into());
            return;
        }
        if self.exporter.is_none() {
            self.exporter = Exporter::choose();
        }
        let Some(exporter) = self.exporter.as_ref() else {
            return;
        };
        let (sender, receiver) = mpsc::channel();
        let cli = exporter.cli.clone();
        let registry = exporter.registry.clone();
        let project = self.project.clone();
        let context = context.clone();
        thread::spawn(move || {
            let result = exui::build_gdl(&project, &path, &cli, &registry)
                .map(|checks| format!("Built {} · {checks}", path.display()));
            let _ = sender.send(result);
            context.request_repaint();
        });
        self.conversion = Some(receiver);
        self.status = "Building and verifying GDL…".into();
    }

    fn shortcuts(&mut self, context: &Context) {
        use egui::Modifiers;
        if self.pending.is_some() || self.error.is_some() {
            return;
        }
        let command = |key| context.input_mut(|input| input.consume_key(Modifiers::COMMAND, key));
        if command(Key::S) {
            self.save(false);
        }
        if command(Key::O) {
            self.replace(Replace::Open, context);
        }
        if !context.wants_keyboard_input() && !self.canvas.preview {
            let redo = context.input_mut(|input| {
                input.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z)
            });
            if redo || command(Key::Y) {
                self.history(false);
            } else if command(Key::Z) {
                self.history(true);
            }
            if context.input(|input| input.key_pressed(Key::Delete)) {
                self.remove();
            }
        }
    }

    fn header(&mut self, ui: &mut Ui, context: &Context) {
        ui.horizontal(|ui| {
            let (mark, _) = ui.allocate_exact_size(egui::vec2(19.0, 19.0), egui::Sense::hover());
            ui.painter().rect_filled(mark, 4.0, ACCENT);
            ui.painter().text(
                mark.center(),
                egui::Align2::CENTER_CENTER,
                "G",
                egui::FontId::proportional(13.0),
                PANEL,
            );
            ui.label(RichText::new("GUI Studio").size(17.0).strong());
            ui.add_space(9.0);
            ui.separator();
            ui.label(RichText::new(&self.project.name).strong());
            if self.project != self.saved {
                ui.label(RichText::new("●").small().color(ACCENT))
                    .on_hover_text("Unsaved changes");
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_enabled_ui(self.conversion.is_none(), |ui| {
                    ui.menu_button(RichText::new("Export").strong().color(ACCENT), |ui| {
                        for (label, gdl) in [("Build GDL…", true), ("Export EXUI…", false)] {
                            if ui.button(label).clicked() {
                                ui.close();
                                self.export(context, gdl);
                            }
                        }
                        if ui.button("Choose Extron exporter…").clicked() {
                            ui.close();
                            if let Some(exporter) = Exporter::choose() {
                                self.exporter = Some(exporter);
                            }
                        }
                    });
                });
                if ui.button("Save").on_hover_text("Ctrl+S").clicked() {
                    self.save(false);
                }
                ui.menu_button("File", |ui| {
                    for (label, action) in [
                        ("New project", Replace::New),
                        ("Open project…", Replace::Open),
                    ] {
                        if ui.button(label).clicked() {
                            ui.close();
                            self.replace(action, context);
                        }
                    }
                    if ui.button("Save as…").clicked() {
                        ui.close();
                        self.save(true);
                    }
                });
            });
        });
    }

    fn layers(&mut self, ui: &mut Ui) {
        section(ui, "LAYERS");
        if ui
            .selectable_label(self.selection.is_none(), "Home")
            .clicked()
        {
            self.select(None);
        }
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut selected = self.selection;
            for element in self.project.elements.iter().rev() {
                let icon = match element.control.kind() {
                    Kind::Button => "B",
                    Kind::Slider => "S",
                    Kind::Label => "T",
                    Kind::Shape => "R",
                };
                let row = egui::Button::new(format!("{icon}    {}", element.name))
                    .selected(selected == Some(element.id))
                    .min_size(egui::vec2(ui.available_width(), 29.0));
                if ui.add(row).clicked() {
                    selected = Some(element.id);
                }
            }
            self.select(selected);
        });
    }

    fn inspector(&mut self, ui: &mut Ui) {
        section(ui, "INSPECTOR");
        let before = self.project.clone();
        if let Some(index) = self
            .project
            .elements
            .iter()
            .position(|element| Some(element.id) == self.selection)
        {
            let element = &mut self.project.elements[index];
            ui.label(
                RichText::new(element.control.kind().label())
                    .size(19.0)
                    .strong(),
            );
            field(ui, "Layer name", &mut element.name);
            ui.separator();
            egui::Grid::new("geometry")
                .num_columns(4)
                .spacing([6.0, 7.0])
                .show(ui, |ui| {
                    number(ui, "X", &mut element.bounds.0[0], 0..=WIDTH - 1);
                    number(ui, "Y", &mut element.bounds.0[1], 0..=HEIGHT - 1);
                    ui.end_row();
                    number(ui, "W", &mut element.bounds.0[2], 1..=WIDTH);
                    number(ui, "H", &mut element.bounds.0[3], 1..=HEIGHT);
                    ui.end_row();
                });
            element.bounds = element.bounds.clamped();
            ui.separator();
            match &mut element.control {
                Control::Button { states, size } => {
                    section(ui, "BUTTON STATE");
                    ui.horizontal(|ui| {
                        for (index, label) in ["Normal", "Active", "Pressed"].iter().enumerate() {
                            ui.selectable_value(&mut self.state, index, *label);
                        }
                    });
                    let state = &mut states[self.state];
                    field(ui, "Text", &mut state.text);
                    color(ui, "Fill", &mut state.fill);
                    color(ui, "Text color", &mut state.color);
                    number(ui, "Font size", size, 4..=200);
                }
                Control::Slider {
                    fill,
                    track,
                    thumb,
                    min,
                    max,
                } => {
                    color(ui, "Fill", fill);
                    color(ui, "Track", track);
                    color(ui, "Thumb", thumb);
                    number(ui, "Minimum", min, 0..=max.saturating_sub(1));
                    number(ui, "Maximum", max, min.saturating_add(1)..=u16::MAX);
                }
                Control::Label {
                    text,
                    color: ink,
                    size,
                } => {
                    field(ui, "Content", text);
                    color(ui, "Text color", ink);
                    number(ui, "Font size", size, 4..=200);
                }
                Control::Shape { fill, radius } => {
                    color(ui, "Fill", fill);
                    number(ui, "Radius", radius, 0..=u8::MAX);
                }
            }
            ui.separator();
            ui.label(RichText::new(format!("Object ID   {}", element.id)).color(MUTED));
            if let Some(id) = element.control_id() {
                ui.label(RichText::new(format!("Control ID  {id}")).color(ACCENT));
            }
            let length = self.project.elements.len();
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(index + 1 < length, egui::Button::new("Raise"))
                    .clicked()
                {
                    self.project.elements.swap(index, index + 1);
                }
                if ui
                    .add_enabled(index > 0, egui::Button::new("Lower"))
                    .clicked()
                {
                    self.project.elements.swap(index, index - 1);
                }
                if ui.button("Delete").clicked() {
                    self.project.elements.remove(index);
                    self.select(None);
                }
            });
        } else {
            ui.label(RichText::new("Home").size(19.0).strong());
            field(ui, "Project name", &mut self.project.name);
            color(ui, "Background", &mut self.project.background);
            ui.label(RichText::new("Extron TLP Pro 1025M").color(MUTED));
            ui.label(RichText::new("1280 × 800 px").color(MUTED));
        }
        self.remember(before);
    }

    fn toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for kind in KINDS {
                if ui
                    .add_enabled(
                        !self.canvas.preview,
                        egui::Button::new(format!("+ {}", kind.label())),
                    )
                    .clicked()
                {
                    self.add(kind);
                }
            }
            ui.separator();
            if ui
                .add_enabled(
                    !self.canvas.preview && !self.undo.is_empty(),
                    egui::Button::new("Undo"),
                )
                .clicked()
            {
                self.history(true);
            }
            if ui
                .add_enabled(
                    !self.canvas.preview && !self.redo.is_empty(),
                    egui::Button::new("Redo"),
                )
                .clicked()
            {
                self.history(false);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.toggle_value(&mut self.canvas.preview, "Preview");
                if ui.button("Fit").clicked() {
                    self.canvas.reset_view();
                }
            });
        });
    }

    fn overlays(&mut self, context: &Context) {
        if let Some(error) = self.error.clone() {
            egui::Modal::new(egui::Id::new("error")).show(context, |ui| {
                ui.set_max_width(520.0);
                ui.heading("Action could not be completed");
                ui.label(&error);
                if ui.button("Close").clicked() {
                    self.error = None;
                }
            });
        } else if let Some(intent) = self.pending {
            egui::Modal::new(egui::Id::new("discard")).show(context, |ui| {
                ui.heading("Save your changes?");
                ui.label(&self.project.name);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() && self.save(false) {
                        self.pending = None;
                        self.run_replace(intent, context);
                    }
                    if ui.button("Discard").clicked() {
                        self.pending = None;
                        self.run_replace(intent, context);
                    }
                    if ui.button("Cancel").clicked() {
                        self.pending = None;
                    }
                });
            });
        }
    }
}

impl eframe::App for Studio {
    fn update(&mut self, context: &Context, _: &mut eframe::Frame) {
        if !self.allow_close
            && self.project != self.saved
            && context.input(|input| input.viewport().close_requested())
        {
            context.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending = Some(Replace::Close);
        }
        if let Some(receiver) = &self.conversion {
            match receiver.try_recv() {
                Ok(Ok(status)) => {
                    self.status = status;
                    self.conversion = None;
                }
                Ok(Err(error)) => {
                    self.status = "GDL build failed".into();
                    self.error = Some(error);
                    self.conversion = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.status = "GDL build disconnected".into();
                    self.conversion = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        self.shortcuts(context);
        let frame = |x, y| {
            Frame::new()
                .fill(PANEL)
                .inner_margin(Margin::symmetric(x, y))
        };
        egui::TopBottomPanel::top("header")
            .frame(frame(18, 12))
            .show(context, |ui| self.header(ui, context));
        egui::TopBottomPanel::bottom("status")
            .frame(frame(15, 7))
            .show(context, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("TLP PRO 1025M")
                            .small()
                            .strong()
                            .color(ACCENT),
                    );
                    ui.label(RichText::new("1280 × 800").small().color(MUTED));
                    ui.separator();
                    ui.label(RichText::new(&self.status).small().color(MUTED))
                        .on_hover_text(&self.status);
                });
            });
        egui::SidePanel::left("layers")
            .exact_width(193.0)
            .resizable(false)
            .frame(frame(14, 16))
            .show(context, |ui| self.layers(ui));
        egui::SidePanel::right("inspector")
            .exact_width(245.0)
            .resizable(false)
            .frame(frame(16, 16))
            .show(context, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_enabled_ui(!self.canvas.preview, |ui| self.inspector(ui));
                });
            });
        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(PANEL))
            .show(context, |ui| {
                Frame::NONE
                    .inner_margin(Margin::symmetric(11, 8))
                    .show(ui, |ui| self.toolbar(ui));
                let (selection, before) =
                    self.canvas
                        .show(ui, &mut self.project, self.selection, self.state);
                self.select(selection);
                if let Some(before) = before {
                    self.remember(before);
                }
            });
        self.overlays(context);
    }
}

fn project_dialog() -> FileDialog {
    FileDialog::new().add_filter("GUI Studio project", &["guistudio"])
}

fn file_name(name: &str, extension: &str) -> String {
    let cleaned: String = name
        .chars()
        .take(96)
        .map(|char| {
            if char.is_control() || "/\\:*?\"<>|".contains(char) {
                '-'
            } else {
                char
            }
        })
        .collect();
    let stem = Path::new(cleaned.trim())
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
    format!("{stem}.{extension}")
}

fn section(ui: &mut Ui, name: &str) {
    ui.label(RichText::new(name).size(11.0).strong().color(MUTED));
}

fn field(ui: &mut Ui, label: &str, value: &mut String) {
    ui.label(RichText::new(label).color(MUTED));
    ui.add(
        egui::TextEdit::singleline(value)
            .char_limit(256)
            .desired_width(f32::INFINITY),
    );
}

fn color(ui: &mut Ui, label: &str, color: &mut Color) {
    ui.horizontal(|ui| {
        ui.color_edit_button_srgb(&mut color.0);
        ui.label(RichText::new(label).color(MUTED));
        ui.label(RichText::new(color.hex()).monospace().small());
    });
}

fn number<N: egui::emath::Numeric>(
    ui: &mut Ui,
    label: &str,
    value: &mut N,
    range: std::ops::RangeInclusive<N>,
) {
    ui.label(RichText::new(label).color(MUTED));
    ui.add(egui::DragValue::new(value).range(range));
}
