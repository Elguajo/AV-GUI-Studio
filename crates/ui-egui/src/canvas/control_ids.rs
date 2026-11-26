//! Extron Control ID badges (View ▸ Control ID badges): the Control ID of every button and slider
//! on the page, at a fixed screen size. A badge sits in its control's top-left corner, or above
//! the control when the corner would cover its caption or the control is too small to hold it.

use std::sync::Arc;

use egui::{Color32, CornerRadius, Pos2, Rect, pos2, vec2};
use figcraft_doc::{Document, NodeId, NodeKind};
use figcraft_engine::gui_studio_extron::control_ids;

use super::FigApp;
use crate::theme::semibold;

/// Inset from the control's corner, and the gap to a control a badge sits above.
const INSET: f32 = 3.0;

/// A control's badge, in page units: its box and the box of each caption's ink.
#[derive(Clone)]
struct Badge {
    id: u16,
    bounds: figcraft_geom::Rect,
    captions: Vec<figcraft_geom::Rect>,
}

/// The badges, when the view shows them.
pub(super) fn draw(app: &FigApp, painter: &egui::Painter, view: &figcraft_engine::ViewState, doc: &Arc<Document>, page: NodeId) {
    if !view.control_ids {
        return;
    }
    let badges = crate::selection_cache::cached(painter.ctx(), "control id badges", doc, &[], &[page.0], || Arc::new(badges(doc, page)));
    let fill = super::overlays::accent(painter);
    for b in badges.iter() {
        let control = screen_rect(app, b.bounds);
        if !control.expand(24.0).intersects(app.canvas.rect) {
            continue;
        }
        let galley = painter.layout_no_wrap(b.id.to_string(), semibold(10.0), Color32::WHITE);
        let size = vec2(galley.size().x + 8.0, galley.size().y + 2.0);
        let captions: Vec<Rect> = b.captions.iter().map(|r| screen_rect(app, *r)).collect();
        let at = place(control, size, &captions);
        let r = Rect::from_min_size(at, size);
        painter.rect_filled(r, CornerRadius::same(3), fill);
        painter.galley(r.center() - galley.size() / 2.0, galley, Color32::WHITE);
    }
}

/// Where a badge of `size` goes for a control drawn at `control`: its top-left corner, unless the
/// badge would cover a caption or not fit, then just above the control.
fn place(control: Rect, size: egui::Vec2, captions: &[Rect]) -> Pos2 {
    let inside = Rect::from_min_size(control.min + vec2(INSET, INSET), size);
    let fits = inside.max.x <= control.max.x - INSET && inside.max.y <= control.max.y - INSET;
    if fits && !captions.iter().any(|c| c.intersects(inside)) {
        return inside.min;
    }
    pos2(control.min.x, control.min.y - size.y - INSET)
}

fn screen_rect(app: &FigApp, r: figcraft_geom::Rect) -> Rect {
    Rect::from_two_pos(app.to_screen(r.x0, r.y0), app.to_screen(r.x1, r.y1))
}

fn badges(doc: &Document, page: NodeId) -> Vec<Badge> {
    control_ids(doc, page)
        .into_iter()
        .filter_map(|(node, id)| Some(Badge { id, bounds: doc.world_bounds(node)?, captions: captions(doc, node) }))
        .collect()
}

/// The page box of the ink of every shown text layer in `node`.
fn captions(doc: &Document, node: NodeId) -> Vec<figcraft_geom::Rect> {
    doc.subtree(node)
        .into_iter()
        .filter_map(|id| doc.node(id))
        .filter(|n| n.visible && matches!(n.kind, NodeKind::Text(_)))
        .filter_map(|n| {
            let (layout, dy) = figcraft_text::layout_node(n)?;
            let ink = layout.ink_bounds()?;
            let ink = figcraft_geom::Rect::new(ink.x0, ink.y0 + dy, ink.x1, ink.y1 + dy);
            Some(figcraft_geom::transform_rect_bbox(doc.world_transform(n.id), ink))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use egui::{Rect, pos2, vec2};

    use super::place;

    #[test]
    fn a_badge_takes_the_corner_unless_it_would_cover_the_caption_or_not_fit() {
        let button = Rect::from_min_size(pos2(100.0, 100.0), vec2(190.0, 72.0));
        let size = vec2(30.0, 14.0);
        // A caption centred in the button leaves the corner free.
        let centred = Rect::from_min_max(pos2(150.0, 126.0), pos2(240.0, 146.0));
        assert_eq!(place(button, size, &[centred]), pos2(103.0, 103.0));
        // A caption that reaches the corner pushes the badge above the button.
        let wide = Rect::from_min_max(pos2(102.0, 102.0), pos2(288.0, 170.0));
        assert_eq!(place(button, size, &[wide]), pos2(100.0, 83.0));
        // So does a control too small on screen to hold it.
        let small = Rect::from_min_size(pos2(100.0, 100.0), vec2(20.0, 10.0));
        assert_eq!(place(small, size, &[]), pos2(100.0, 83.0));
    }
}
