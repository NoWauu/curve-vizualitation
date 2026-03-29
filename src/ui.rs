use nannou::prelude::*;

use crate::bezier::bernstein_basis;
use crate::model::ControlPoint;

// ── Influence graph ───────────────────────────────────────────────────────────

const PANEL_W: f32 = 220.0;
const PANEL_H: f32 = 140.0;
const PANEL_MARGIN: f32 = 12.0;
const PADDING: f32 = 14.0;
const RESOLUTION: usize = 120;

/// Returns the top-left corner of the inner plot area and its (width, height).
fn plot_area(win: Rect) -> (Vec2, f32, f32) {
    let panel_x = win.right() - PANEL_MARGIN - PANEL_W / 2.0;
    let panel_y = win.bottom() + PANEL_MARGIN + PANEL_H / 2.0 + SLIDER_BOTTOM_MARGIN + SLIDER_HANDLE_R;
    let origin = vec2(panel_x - PANEL_W / 2.0 + PADDING, panel_y - PANEL_H / 2.0 + PADDING);
    (origin, PANEL_W - PADDING * 2.0, PANEL_H - PADDING * 2.0)
}

/// Draws the Bernstein influence graph in the bottom-right corner.
/// In piecewise mode, pass the active segment's points, its local t, seg_idx, and total_segs.
/// In full-Bézier mode, pass all points with global t, seg_idx=0, total_segs=1.
pub fn draw_influence_graph(
    draw: &Draw,
    win: Rect,
    points: &[ControlPoint],
    t: f32,
    seg_idx: usize,
    total_segs: usize,
) {
    if points.len() < 2 {
        return;
    }

    let (origin, plot_w, plot_h) = plot_area(win);
    let panel_cx = origin.x - PADDING + PANEL_W / 2.0;
    let panel_cy = origin.y - PADDING + PANEL_H / 2.0;

    // Background + border
    draw.rect()
        .x_y(panel_cx, panel_cy)
        .w_h(PANEL_W, PANEL_H)
        .color(rgba(0.05, 0.05, 0.05, 0.85));
    draw.rect()
        .x_y(panel_cx, panel_cy)
        .w_h(PANEL_W, PANEL_H)
        .no_fill()
        .stroke_weight(1.0)
        .stroke(rgba(1.0f32, 1.0, 1.0, 0.2));

    // Segment label (piecewise mode only)
    if total_segs > 1 {
        draw.text(&format!("Seg {}/{}", seg_idx + 1, total_segs))
            .x_y(panel_cx, panel_cy + PANEL_H / 2.0 - 9.0)
            .font_size(10)
            .color(rgba(1.0f32, 1.0, 1.0, 0.55));
    }

    // Axes
    let axis_color = rgba(1.0f32, 1.0, 1.0, 0.25);
    draw.line()
        .start(origin)
        .end(pt2(origin.x + plot_w, origin.y))
        .color(axis_color);
    draw.line()
        .start(origin)
        .end(pt2(origin.x, origin.y + plot_h))
        .color(axis_color);

    // Bernstein basis curves + dot at current_t
    let degree = points.len() - 1;
    for (i, point) in points.iter().enumerate() {
        let curve_pts: Vec<Vec2> = (0..=RESOLUTION)
            .map(|j| {
                let s = j as f32 / RESOLUTION as f32;
                let influence = bernstein_basis(degree, i, s);
                vec2(origin.x + s * plot_w, origin.y + influence * plot_h)
            })
            .collect();

        draw.polyline().weight(1.5).points(curve_pts).color(point.color);

        // Dot at current t
        let influence_at_t = bernstein_basis(degree, i, t);
        let dot = vec2(origin.x + t * plot_w, origin.y + influence_at_t * plot_h);
        draw.ellipse().xy(dot).radius(3.5).color(point.color);
    }

    // Vertical playhead line
    let playhead_x = origin.x + t * plot_w;
    draw.line()
        .start(pt2(playhead_x, origin.y))
        .end(pt2(playhead_x, origin.y + plot_h))
        .weight(1.0)
        .color(rgba(1.0f32, 1.0, 1.0, 0.5));
}

// ── Slider ────────────────────────────────────────────────────────────────────

pub const SLIDER_W: f32 = 380.0;
pub const SLIDER_HANDLE_R: f32 = 8.0;
const SLIDER_TRACK_H: f32 = 3.0;
const SLIDER_BOTTOM_MARGIN: f32 = 24.0;

/// Center of the slider track in window space.
pub fn slider_track_center(win: Rect) -> Vec2 {
    vec2(win.x(), win.bottom() + SLIDER_BOTTOM_MARGIN + SLIDER_HANDLE_R)
}

/// Clamps mouse x onto the track and returns the corresponding t ∈ [0, 1].
pub fn t_from_mouse_x(win: Rect, mouse_x: f32) -> f32 {
    let center = slider_track_center(win);
    let track_left = center.x - SLIDER_W / 2.0;
    ((mouse_x - track_left) / SLIDER_W).clamp(0.0, 1.0)
}

/// Returns true if `pos` is close enough to the slider handle to start a drag.
pub fn hits_slider(win: Rect, t: f32, pos: Vec2) -> bool {
    let center = slider_track_center(win);
    let handle = vec2(center.x - SLIDER_W / 2.0 + t * SLIDER_W, center.y);
    // Also allow clicking anywhere on the track bar
    let on_track = pos.x >= handle.x - SLIDER_W * t
        && pos.x <= handle.x + SLIDER_W * (1.0 - t)
        && (pos.y - center.y).abs() < SLIDER_HANDLE_R * 1.5;
    pos.distance(handle) < SLIDER_HANDLE_R + 4.0 || on_track
}

pub fn draw_slider(draw: &Draw, win: Rect, t: f32) {
    let center = slider_track_center(win);
    let track_left = center.x - SLIDER_W / 2.0;
    let track_right = center.x + SLIDER_W / 2.0;
    let handle_x = track_left + t * SLIDER_W;

    // Full track (dim)
    draw.line()
        .start(pt2(track_left, center.y))
        .end(pt2(track_right, center.y))
        .weight(SLIDER_TRACK_H)
        .color(rgba(1.0f32, 1.0, 1.0, 0.2));

    // Filled portion (t)
    draw.line()
        .start(pt2(track_left, center.y))
        .end(pt2(handle_x, center.y))
        .weight(SLIDER_TRACK_H)
        .color(STEELBLUE);

    // Handle
    draw.ellipse()
        .x_y(handle_x, center.y)
        .radius(SLIDER_HANDLE_R)
        .color(WHITE);

    // t label above the handle
    draw.text(&format!("t = {:.2}", t))
        .x_y(handle_x, center.y + SLIDER_HANDLE_R + 10.0)
        .font_size(11)
        .color(rgba(1.0f32, 1.0, 1.0, 0.7));
}
