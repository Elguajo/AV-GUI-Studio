use super::{ACCENT, MUTED};
use eframe::egui::{
    Align2, Color32, FontId, Painter, PointerButton, Pos2, Rect, Sense, Stroke, StrokeKind, Ui,
    Vec2, vec2,
};
use gui_studio::project::{Bounds, Color, Control, Element, HEIGHT, Project, WIDTH};
use std::collections::HashMap;

pub(super) struct Canvas {
    zoom: f32,
    pan: Vec2,
    pub preview: bool,
    drag: Option<Drag>,
    sliders: HashMap<u16, f32>,
    active: HashMap<u16, bool>,
}

struct Drag {
    id: u16,
    start: Pos2,
    bounds: Bounds,
    resize: Option<(bool, bool)>,
    before: Project,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            preview: false,
            drag: None,
            sliders: HashMap::new(),
            active: HashMap::new(),
        }
    }
}

impl Canvas {
    pub fn reset_view(&mut self) {
        self.zoom = 1.0;
        self.pan = Vec2::ZERO;
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        project: &mut Project,
        mut selected: Option<u16>,
        state: usize,
    ) -> (Option<u16>, Option<Project>) {
        let (area, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let viewport = area.rect;
        let fit = ((viewport.width() - 88.0) / WIDTH as f32)
            .min((viewport.height() - 108.0) / HEIGHT as f32)
            .max(0.08);
        if area.hovered() {
            self.zoom = (self.zoom * (ui.input(|input| input.smooth_scroll_delta.y) / 380.0).exp())
                .clamp(0.4, 4.5);
        }
        if area.dragged_by(PointerButton::Middle) {
            self.pan += area.drag_delta();
        }
        let scale = fit * self.zoom;
        let artboard = Rect::from_center_size(
            viewport.center() + self.pan,
            vec2(WIDTH as f32, HEIGHT as f32) * scale,
        );
        let local = |point: Pos2| {
            let offset = (point - artboard.min) / scale;
            Pos2::new(offset.x, offset.y)
        };
        let pointer = area.interact_pointer_pos();
        let mut before = None;
        if self.preview {
            if let Some(point) = pointer
                && let Some(element) = hit(project, point, artboard, scale)
            {
                match &element.control {
                    Control::Button { .. } if area.clicked() => {
                        let active = self.active.entry(element.id).or_default();
                        *active = !*active;
                    }
                    Control::Slider { .. }
                        if area.clicked() || area.is_pointer_button_down_on() =>
                    {
                        let rect = bounds_rect(element.bounds, artboard, scale);
                        self.sliders.insert(
                            element.id,
                            ((point.x - rect.left()) / rect.width()).clamp(0.0, 1.0),
                        );
                    }
                    _ => {}
                }
            }
        } else {
            if area.clicked()
                && let Some(point) = pointer
            {
                selected = hit(project, point, artboard, scale).map(|element| element.id);
            }
            if area.drag_started_by(PointerButton::Primary) {
                let origin = ui.input(|input| input.pointer.press_origin());
                if let Some(origin) = origin {
                    let handle = project
                        .elements
                        .iter()
                        .find(|element| Some(element.id) == selected)
                        .and_then(|element| {
                            corner_handles(bounds_rect(element.bounds, artboard, scale))
                                .into_iter()
                                .find(|(corner, _, _)| corner.distance(origin) <= 7.0)
                                .map(|(_, right, bottom)| (element, Some((right, bottom))))
                        });
                    if let Some((element, resize)) = handle.or_else(|| {
                        hit(project, origin, artboard, scale).map(|element| (element, None))
                    }) {
                        selected = Some(element.id);
                        self.drag = Some(Drag {
                            id: element.id,
                            start: local(origin),
                            bounds: element.bounds,
                            resize,
                            before: project.clone(),
                        });
                    }
                }
            }
            if let Some(drag) = &self.drag
                && let Some(point) = pointer
                && let Some(element) = project
                    .elements
                    .iter_mut()
                    .find(|element| element.id == drag.id)
            {
                let delta = local(point) - drag.start;

                let snap = ui.input(|input| input.modifiers.shift);
                let round = |value: f32| {
                    if snap {
                        (value / 8.0).round() as i32 * 8
                    } else {
                        value.round() as i32
                    }
                };
                element.bounds =
                    transformed_bounds(drag.bounds, round(delta.x), round(delta.y), drag.resize);
            }
            if area.drag_stopped_by(PointerButton::Primary) {
                before = self.drag.take().map(|drag| drag.before);
            }
        }

        painter.rect_filled(viewport, 0.0, Color32::from_rgb(12, 18, 25));
        let grid = Color32::from_rgb(34, 49, 59);
        let step = 20.0;
        for row in 0..(viewport.height() / step).ceil() as i32 {
            for column in 0..(viewport.width() / step).ceil() as i32 {
                painter.circle_filled(
                    viewport.min + vec2(column as f32 * step, row as f32 * step),
                    0.8,
                    grid,
                );
            }
        }
        painter.rect_filled(
            artboard.translate(vec2(0.0, 5.0)).expand(5.0),
            4.0,
            Color32::from_black_alpha(72),
        );
        painter.rect_filled(artboard, 0.0, rgb(project.background));
        painter.rect_stroke(
            artboard,
            0.0,
            Stroke::new(1.0_f32, Color32::from_rgb(65, 89, 100)),
            StrokeKind::Outside,
        );
        painter.text(
            artboard.left_top() - vec2(0.0, 12.0),
            Align2::LEFT_BOTTOM,
            "Home",
            FontId::proportional(13.0),
            MUTED,
        );
        painter.text(
            artboard.right_top() - vec2(0.0, 12.0),
            Align2::RIGHT_BOTTOM,
            format!("{:.0}%  ·  1280 × 800", scale * 100.0),
            FontId::monospace(11.0),
            MUTED,
        );

        let content = painter.with_clip_rect(artboard);
        let hovered = pointer
            .and_then(|point| hit(project, point, artboard, scale))
            .map(|element| element.id);
        for element in &project.elements {
            let view_state = if self.preview {
                if hovered == Some(element.id) && area.is_pointer_button_down_on() {
                    2
                } else {
                    usize::from(*self.active.get(&element.id).unwrap_or(&false))
                }
            } else if selected == Some(element.id) {
                state
            } else {
                0
            };
            let slider = *self.sliders.get(&element.id).unwrap_or(&0.57);
            draw(
                &content,
                element,
                bounds_rect(element.bounds, artboard, scale),
                scale,
                view_state,
                slider,
            );
        }
        if !self.preview
            && let Some(item) = project
                .elements
                .iter()
                .find(|element| Some(element.id) == selected)
        {
            let rect = bounds_rect(item.bounds, artboard, scale);
            painter.rect_stroke(rect, 0.0, Stroke::new(1.4_f32, ACCENT), StrokeKind::Outside);
            for (corner, _, _) in corner_handles(rect) {
                painter.rect_filled(Rect::from_center_size(corner, vec2(5.0, 5.0)), 0.0, ACCENT);
            }
        }
        (selected, before)
    }
}

fn corner_handles(rect: Rect) -> [(Pos2, bool, bool); 4] {
    [
        (rect.left_top(), false, false),
        (rect.left_bottom(), false, true),
        (rect.right_top(), true, false),
        (rect.right_bottom(), true, true),
    ]
}

fn transformed_bounds(bounds: Bounds, dx: i32, dy: i32, resize: Option<(bool, bool)>) -> Bounds {
    let [x, y, w, h] = bounds.0;
    let Some((right, bottom)) = resize else {
        return Bounds([x.saturating_add(dx), y.saturating_add(dy), w, h]).clamped();
    };
    fn edges(start: i32, size: i32, delta: i32, far: bool, limit: i32) -> (i32, i32) {
        let end = start + size;
        if far {
            (start, end.saturating_add(delta).clamp(start + 1, limit))
        } else {
            (start.saturating_add(delta).clamp(0, end - 1), end)
        }
    }
    let (left, right) = edges(x, w, dx, right, WIDTH);
    let (top, bottom) = edges(y, h, dy, bottom, HEIGHT);
    Bounds([left, top, right - left, bottom - top])
}

fn hit(project: &Project, point: Pos2, artboard: Rect, scale: f32) -> Option<&Element> {
    project
        .elements
        .iter()
        .rev()
        .find(|element| bounds_rect(element.bounds, artboard, scale).contains(point))
}

fn bounds_rect(bounds: Bounds, artboard: Rect, scale: f32) -> Rect {
    let [x, y, w, h] = bounds.0;
    Rect::from_min_size(
        artboard.min + vec2(x as f32, y as f32) * scale,
        vec2(w as f32, h as f32) * scale,
    )
}

fn draw(painter: &Painter, element: &Element, rect: Rect, scale: f32, state: usize, slider: f32) {
    let font = |size: u8| FontId::proportional(f32::from(size) * scale * 4.0 / 3.0);
    match &element.control {
        Control::Button { states, size } => {
            let theme = &states[state];
            painter.rect_filled(rect, 0.0, rgb(theme.fill));
            painter.with_clip_rect(rect).text(
                rect.center(),
                Align2::CENTER_CENTER,
                &theme.text,
                font(*size),
                rgb(theme.color),
            );
        }
        Control::Slider {
            fill, track, thumb, ..
        } => {
            let thickness = rect.height().min(12.0 * scale);
            let bar = Rect::from_center_size(rect.center(), vec2(rect.width(), thickness));
            let stop = bar.left() + bar.width() * slider;
            painter.rect_filled(bar, thickness / 2.0, rgb(*track));
            painter.rect_filled(
                Rect::from_min_max(bar.min, Pos2::new(stop, bar.bottom())),
                thickness / 2.0,
                rgb(*fill),
            );
            let radius = rect.height().min(rect.width()).min(26.0 * scale) / 2.0;
            let center = bar.left() + radius + slider * (bar.width() - 2.0 * radius).max(0.0);
            painter.circle_filled(Pos2::new(center, bar.center().y), radius, rgb(*thumb));
        }
        Control::Label { text, color, size } => {
            painter.with_clip_rect(rect).text(
                rect.left_center(),
                Align2::LEFT_CENTER,
                text,
                font(*size),
                rgb(*color),
            );
        }
        Control::Shape { fill, radius } => {
            painter.rect_filled(
                rect,
                (f32::from(*radius) * scale).min(rect.width().min(rect.height()) / 2.0),
                rgb(*fill),
            );
        }
    }
}

fn rgb(color: Color) -> Color32 {
    let [r, g, b] = color.0;
    Color32::from_rgb(r, g, b)
}
