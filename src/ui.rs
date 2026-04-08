use nannou::prelude::*;

use crate::bezier::{self, bernstein_basis};
use crate::hermite;
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

/// Returns the top-left corner of the velocity panel's inner plot area, placed above the influence graph.
fn velocity_plot_area(win: Rect) -> (Vec2, f32, f32) {
    let panel_x = win.right() - PANEL_MARGIN - PANEL_W / 2.0;
    let influence_cy = win.bottom() + PANEL_MARGIN + PANEL_H / 2.0 + SLIDER_BOTTOM_MARGIN + SLIDER_HANDLE_R;
    let panel_y = influence_cy + PANEL_H + PANEL_MARGIN;
    let origin = vec2(panel_x - PANEL_W / 2.0 + PADDING, panel_y - PANEL_H / 2.0 + PADDING);
    (origin, PANEL_W - PADDING * 2.0, PANEL_H - PADDING * 2.0)
}

/// Returns the inner plot area for the G1 (tangent angle) panel on the left side.
fn g1_plot_area(win: Rect) -> (Vec2, f32, f32) {
    let panel_x = win.left() + PANEL_MARGIN + PANEL_W / 2.0;
    let panel_y = win.bottom() + PANEL_MARGIN + PANEL_H / 2.0 + SLIDER_BOTTOM_MARGIN + SLIDER_HANDLE_R;
    let origin = vec2(panel_x - PANEL_W / 2.0 + PADDING, panel_y - PANEL_H / 2.0 + PADDING);
    (origin, PANEL_W - PADDING * 2.0, PANEL_H - PADDING * 2.0)
}

/// Returns the inner plot area for the G2 (curvature) panel, placed above the G1 panel.
fn g2_plot_area(win: Rect) -> (Vec2, f32, f32) {
    let panel_x = win.left() + PANEL_MARGIN + PANEL_W / 2.0;
    let g1_cy = win.bottom() + PANEL_MARGIN + PANEL_H / 2.0 + SLIDER_BOTTOM_MARGIN + SLIDER_HANDLE_R;
    let panel_y = g1_cy + PANEL_H + PANEL_MARGIN;
    let origin = vec2(panel_x - PANEL_W / 2.0 + PADDING, panel_y - PANEL_H / 2.0 + PADDING);
    (origin, PANEL_W - PADDING * 2.0, PANEL_H - PADDING * 2.0)
}

/// Returns the top-left corner of the acceleration panel's inner plot area, placed above the velocity graph.
fn acceleration_plot_area(win: Rect) -> (Vec2, f32, f32) {
    let panel_x = win.right() - PANEL_MARGIN - PANEL_W / 2.0;
    let influence_cy = win.bottom() + PANEL_MARGIN + PANEL_H / 2.0 + SLIDER_BOTTOM_MARGIN + SLIDER_HANDLE_R;
    let vel_cy = influence_cy + PANEL_H + PANEL_MARGIN;
    let panel_y = vel_cy + PANEL_H + PANEL_MARGIN;
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

/// `segments` is one entry per spline segment (single entry for full Bézier mode).
pub fn draw_velocity_graph(draw: &Draw, win: Rect, segments: &[Vec<Vec2>], current_t: f32) {
    if segments.is_empty() {
        return;
    }

    let (origin, plot_w, plot_h) = velocity_plot_area(win);
    let panel_cx = origin.x - PADDING + PANEL_W / 2.0;
    let panel_cy = origin.y - PADDING + PANEL_H / 2.0;
    let n_segs = segments.len();

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
        .stroke(rgba(0.2f32, 0.8, 0.2, 0.35));

    // Label
    draw.text("Velocity")
        .x_y(panel_cx, panel_cy + PANEL_H / 2.0 - 9.0)
        .font_size(10)
        .color(rgba(0.2f32, 0.8, 0.2, 0.65));

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

    // Dim segment boundary lines
    if n_segs > 1 {
        for i in 1..n_segs {
            let bx = origin.x + (i as f32 / n_segs as f32) * plot_w;
            draw.line()
                .start(pt2(bx, origin.y))
                .end(pt2(bx, origin.y + plot_h))
                .weight(0.5)
                .color(rgba(1.0f32, 1.0, 1.0, 0.1));
        }
    }

    // Compute velocity magnitudes per segment using the derivative control polygon
    let per_seg: Vec<Vec<f32>> = segments
        .iter()
        .map(|seg| {
            if seg.len() < 2 {
                return vec![0.0; RESOLUTION + 1];
            }
            let vel_pts = bezier::bezier_derivative_points(seg);
            (0..=RESOLUTION)
                .map(|j| {
                    let lt = j as f32 / RESOLUTION as f32;
                    bezier::de_casteljau(&vel_pts, lt).length()
                })
                .collect()
        })
        .collect();

    let max_vel = per_seg.iter().flatten().cloned().fold(0.0f32, f32::max).max(0.01);

    // Draw per-segment velocity curves
    for (i, samples) in per_seg.iter().enumerate() {
        let gx0 = i as f32 / n_segs as f32;
        let gx1 = (i + 1) as f32 / n_segs as f32;
        let pts: Vec<Vec2> = samples
            .iter()
            .enumerate()
            .map(|(j, &v)| {
                let frac = j as f32 / RESOLUTION as f32;
                let gx = gx0 + frac * (gx1 - gx0);
                vec2(origin.x + gx * plot_w, origin.y + (v / max_vel) * plot_h)
            })
            .collect();
        draw.polyline().weight(1.5).points(pts).color(rgba(0.2, 0.8, 0.2, 0.9));
    }

    // C1 jump markers at junctions
    for i in 0..n_segs.saturating_sub(1) {
        let left_v = *per_seg[i].last().unwrap_or(&0.0);
        let right_v = per_seg[i + 1].first().copied().unwrap_or(0.0);
        let jx = (i + 1) as f32 / n_segs as f32;
        let px = origin.x + jx * plot_w;
        let ly = origin.y + (left_v / max_vel) * plot_h;
        let ry = origin.y + (right_v / max_vel) * plot_h;

        if (left_v - right_v).abs() / max_vel > 0.01 {
            draw.line()
                .start(pt2(px, ly))
                .end(pt2(px, ry))
                .weight(1.5)
                .color(rgba(1.0, 0.3, 0.3, 0.85));
            draw.ellipse().xy(pt2(px, ly)).radius(2.5).color(rgba(1.0, 0.3, 0.3, 1.0));
            draw.ellipse().xy(pt2(px, ry)).radius(2.5).color(rgba(1.0, 0.3, 0.3, 1.0));
        }
    }

    // Playhead
    let dot_x = origin.x + current_t * plot_w;
    let seg_i = ((current_t * n_segs as f32).floor() as usize).min(n_segs.saturating_sub(1));
    let current_vel = if segments[seg_i].len() >= 2 {
        let lt = (current_t * n_segs as f32 - seg_i as f32).clamp(0.0, 1.0);
        let vel_pts = bezier::bezier_derivative_points(&segments[seg_i]);
        bezier::de_casteljau(&vel_pts, lt).length()
    } else {
        0.0
    };
    let dot_y = origin.y + (current_vel / max_vel) * plot_h;
    draw.line()
        .start(pt2(dot_x, origin.y))
        .end(pt2(dot_x, origin.y + plot_h))
        .weight(1.0)
        .color(rgba(0.2, 0.8, 0.2, 0.5));
    draw.ellipse().xy(pt2(dot_x, dot_y)).radius(3.5).color(rgba(0.2, 0.8, 0.2, 1.0));
}

/// `segments` is one entry per spline segment (single entry for full Bézier mode).
pub fn draw_acceleration_graph(draw: &Draw, win: Rect, segments: &[Vec<Vec2>], current_t: f32) {
    if segments.is_empty() {
        return;
    }

    let (origin, plot_w, plot_h) = acceleration_plot_area(win);
    let panel_cx = origin.x - PADDING + PANEL_W / 2.0;
    let panel_cy = origin.y - PADDING + PANEL_H / 2.0;
    let n_segs = segments.len();

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
        .stroke(rgba(1.0f32, 0.6, 0.1, 0.35));

    // Label
    draw.text("Acceleration")
        .x_y(panel_cx, panel_cy + PANEL_H / 2.0 - 9.0)
        .font_size(10)
        .color(rgba(1.0f32, 0.6, 0.1, 0.65));

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

    // Dim segment boundary lines
    if n_segs > 1 {
        for i in 1..n_segs {
            let bx = origin.x + (i as f32 / n_segs as f32) * plot_w;
            draw.line()
                .start(pt2(bx, origin.y))
                .end(pt2(bx, origin.y + plot_h))
                .weight(0.5)
                .color(rgba(1.0f32, 1.0, 1.0, 0.1));
        }
    }

    // Compute acceleration magnitudes per segment using double derivative control polygon
    let per_seg: Vec<Vec<f32>> = segments
        .iter()
        .map(|seg| {
            if seg.len() < 3 {
                return vec![0.0; RESOLUTION + 1];
            }
            let vel_pts = bezier::bezier_derivative_points(seg);
            let acc_pts = bezier::bezier_derivative_points(&vel_pts);
            if acc_pts.is_empty() {
                return vec![0.0; RESOLUTION + 1];
            }
            (0..=RESOLUTION)
                .map(|j| {
                    let lt = j as f32 / RESOLUTION as f32;
                    bezier::de_casteljau(&acc_pts, lt).length()
                })
                .collect()
        })
        .collect();

    let max_acc = per_seg.iter().flatten().cloned().fold(0.0f32, f32::max).max(0.01);

    // Draw per-segment acceleration curves
    for (i, samples) in per_seg.iter().enumerate() {
        let gx0 = i as f32 / n_segs as f32;
        let gx1 = (i + 1) as f32 / n_segs as f32;
        let pts: Vec<Vec2> = samples
            .iter()
            .enumerate()
            .map(|(j, &a)| {
                let frac = j as f32 / RESOLUTION as f32;
                let gx = gx0 + frac * (gx1 - gx0);
                vec2(origin.x + gx * plot_w, origin.y + (a / max_acc) * plot_h)
            })
            .collect();
        draw.polyline().weight(1.5).points(pts).color(rgba(1.0, 0.6, 0.1, 0.9));
    }

    // C2 jump markers at junctions
    for i in 0..n_segs.saturating_sub(1) {
        let left_a = *per_seg[i].last().unwrap_or(&0.0);
        let right_a = per_seg[i + 1].first().copied().unwrap_or(0.0);
        let jx = (i + 1) as f32 / n_segs as f32;
        let px = origin.x + jx * plot_w;
        let ly = origin.y + (left_a / max_acc) * plot_h;
        let ry = origin.y + (right_a / max_acc) * plot_h;

        if (left_a - right_a).abs() / max_acc > 0.01 {
            draw.line()
                .start(pt2(px, ly))
                .end(pt2(px, ry))
                .weight(1.5)
                .color(rgba(1.0, 0.3, 0.3, 0.85));
            draw.ellipse().xy(pt2(px, ly)).radius(2.5).color(rgba(1.0, 0.3, 0.3, 1.0));
            draw.ellipse().xy(pt2(px, ry)).radius(2.5).color(rgba(1.0, 0.3, 0.3, 1.0));
        }
    }

    // Playhead
    let dot_x = origin.x + current_t * plot_w;
    let seg_i = ((current_t * n_segs as f32).floor() as usize).min(n_segs.saturating_sub(1));
    let current_acc = if segments[seg_i].len() >= 3 {
        let lt = (current_t * n_segs as f32 - seg_i as f32).clamp(0.0, 1.0);
        let vel_pts = bezier::bezier_derivative_points(&segments[seg_i]);
        let acc_pts = bezier::bezier_derivative_points(&vel_pts);
        if acc_pts.is_empty() { 0.0 } else { bezier::de_casteljau(&acc_pts, lt).length() }
    } else {
        0.0
    };
    let dot_y = origin.y + (current_acc / max_acc) * plot_h;
    draw.line()
        .start(pt2(dot_x, origin.y))
        .end(pt2(dot_x, origin.y + plot_h))
        .weight(1.0)
        .color(rgba(1.0, 0.6, 0.1, 0.5));
    draw.ellipse().xy(pt2(dot_x, dot_y)).radius(3.5).color(rgba(1.0, 0.6, 0.1, 1.0));
}

/// Draws the G1 continuity graph (tangent direction angle) on the left side.
pub fn draw_g1_graph(draw: &Draw, win: Rect, segments: &[Vec<Vec2>], current_t: f32) {
    if segments.is_empty() {
        return;
    }

    let (origin, plot_w, plot_h) = g1_plot_area(win);
    let panel_cx = origin.x - PADDING + PANEL_W / 2.0;
    let panel_cy = origin.y - PADDING + PANEL_H / 2.0;
    let n_segs = segments.len();

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
        .stroke(rgba(0.2f32, 0.8, 1.0, 0.35));

    draw.text("G1 — Tangent Angle")
        .x_y(panel_cx, panel_cy + PANEL_H / 2.0 - 9.0)
        .font_size(10)
        .color(rgba(0.2f32, 0.8, 1.0, 0.65));

    // Axes
    let axis_color = rgba(1.0f32, 1.0, 1.0, 0.25);
    draw.line().start(origin).end(pt2(origin.x + plot_w, origin.y)).color(axis_color);
    draw.line().start(origin).end(pt2(origin.x, origin.y + plot_h)).color(axis_color);

    // Zero line at mid height
    let mid_y = origin.y + plot_h * 0.5;
    draw.line()
        .start(pt2(origin.x, mid_y))
        .end(pt2(origin.x + plot_w, mid_y))
        .weight(0.5)
        .color(rgba(1.0f32, 1.0, 1.0, 0.15));

    // Segment boundary lines
    if n_segs > 1 {
        for i in 1..n_segs {
            let bx = origin.x + (i as f32 / n_segs as f32) * plot_w;
            draw.line()
                .start(pt2(bx, origin.y))
                .end(pt2(bx, origin.y + plot_h))
                .weight(0.5)
                .color(rgba(1.0f32, 1.0, 1.0, 0.1));
        }
    }

    // Compute per-segment tangent angles
    let per_seg: Vec<Vec<f32>> = segments
        .iter()
        .map(|seg| {
            if seg.len() < 2 {
                return vec![0.0; RESOLUTION + 1];
            }
            let vel_pts = bezier::bezier_derivative_points(seg);
            (0..=RESOLUTION)
                .map(|j| {
                    let lt = j as f32 / RESOLUTION as f32;
                    let vel = bezier::de_casteljau(&vel_pts, lt);
                    vel.y.atan2(vel.x)
                })
                .collect()
        })
        .collect();

    let max_angle = per_seg
        .iter()
        .flatten()
        .map(|a| a.abs())
        .fold(0.0f32, f32::max)
        .max(0.01);

    for (i, samples) in per_seg.iter().enumerate() {
        let gx0 = i as f32 / n_segs as f32;
        let gx1 = (i + 1) as f32 / n_segs as f32;
        let pts: Vec<Vec2> = samples
            .iter()
            .enumerate()
            .map(|(j, &a)| {
                let frac = j as f32 / RESOLUTION as f32;
                let gx = gx0 + frac * (gx1 - gx0);
                let norm = (a / max_angle) * 0.5 + 0.5;
                vec2(origin.x + gx * plot_w, origin.y + norm * plot_h)
            })
            .collect();
        draw.polyline().weight(1.5).points(pts).color(rgba(0.2, 0.8, 1.0, 0.9));
    }

    // G1 jump markers at junctions
    for i in 0..n_segs.saturating_sub(1) {
        let left_a = *per_seg[i].last().unwrap_or(&0.0);
        let right_a = per_seg[i + 1].first().copied().unwrap_or(0.0);
        let diff = (left_a - right_a).abs();
        let diff = diff.min(std::f32::consts::TAU - diff);
        let jx = (i + 1) as f32 / n_segs as f32;
        let px = origin.x + jx * plot_w;
        let ly = origin.y + ((left_a / max_angle) * 0.5 + 0.5) * plot_h;
        let ry = origin.y + ((right_a / max_angle) * 0.5 + 0.5) * plot_h;

        if diff > 0.05 {
            draw.line()
                .start(pt2(px, ly))
                .end(pt2(px, ry))
                .weight(1.5)
                .color(rgba(1.0, 0.3, 0.3, 0.85));
            draw.ellipse().xy(pt2(px, ly)).radius(2.5).color(rgba(1.0, 0.3, 0.3, 1.0));
            draw.ellipse().xy(pt2(px, ry)).radius(2.5).color(rgba(1.0, 0.3, 0.3, 1.0));
        }
    }

    // Playhead
    let dot_x = origin.x + current_t * plot_w;
    let seg_i = ((current_t * n_segs as f32).floor() as usize).min(n_segs.saturating_sub(1));
    let current_angle = if segments[seg_i].len() >= 2 {
        let lt = (current_t * n_segs as f32 - seg_i as f32).clamp(0.0, 1.0);
        let vel_pts = bezier::bezier_derivative_points(&segments[seg_i]);
        let vel = bezier::de_casteljau(&vel_pts, lt);
        vel.y.atan2(vel.x)
    } else {
        0.0
    };
    let dot_y = origin.y + ((current_angle / max_angle) * 0.5 + 0.5) * plot_h;
    draw.line()
        .start(pt2(dot_x, origin.y))
        .end(pt2(dot_x, origin.y + plot_h))
        .weight(1.0)
        .color(rgba(0.2, 0.8, 1.0, 0.5));
    draw.ellipse().xy(pt2(dot_x, dot_y)).radius(3.5).color(rgba(0.2, 0.8, 1.0, 1.0));
}

/// Draws the G2 continuity graph (signed curvature) on the left side, above the G1 panel.
pub fn draw_g2_graph(draw: &Draw, win: Rect, segments: &[Vec<Vec2>], current_t: f32) {
    if segments.is_empty() {
        return;
    }

    let (origin, plot_w, plot_h) = g2_plot_area(win);
    let panel_cx = origin.x - PADDING + PANEL_W / 2.0;
    let panel_cy = origin.y - PADDING + PANEL_H / 2.0;
    let n_segs = segments.len();

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
        .stroke(rgba(0.8f32, 0.3, 1.0, 0.35));

    draw.text("G2 — Curvature")
        .x_y(panel_cx, panel_cy + PANEL_H / 2.0 - 9.0)
        .font_size(10)
        .color(rgba(0.8f32, 0.3, 1.0, 0.65));

    // Axes
    let axis_color = rgba(1.0f32, 1.0, 1.0, 0.25);
    draw.line().start(origin).end(pt2(origin.x + plot_w, origin.y)).color(axis_color);
    draw.line().start(origin).end(pt2(origin.x, origin.y + plot_h)).color(axis_color);

    // Zero line at mid height
    let mid_y = origin.y + plot_h * 0.5;
    draw.line()
        .start(pt2(origin.x, mid_y))
        .end(pt2(origin.x + plot_w, mid_y))
        .weight(0.5)
        .color(rgba(1.0f32, 1.0, 1.0, 0.15));

    // Segment boundary lines
    if n_segs > 1 {
        for i in 1..n_segs {
            let bx = origin.x + (i as f32 / n_segs as f32) * plot_w;
            draw.line()
                .start(pt2(bx, origin.y))
                .end(pt2(bx, origin.y + plot_h))
                .weight(0.5)
                .color(rgba(1.0f32, 1.0, 1.0, 0.1));
        }
    }

    // Compute per-segment signed curvature κ = (x'y'' − y'x'') / |P'|³
    let per_seg: Vec<Vec<f32>> = segments
        .iter()
        .map(|seg| {
            if seg.len() < 3 {
                return vec![0.0; RESOLUTION + 1];
            }
            let vel_pts = bezier::bezier_derivative_points(seg);
            let acc_pts = bezier::bezier_derivative_points(&vel_pts);
            if acc_pts.is_empty() {
                return vec![0.0; RESOLUTION + 1];
            }
            (0..=RESOLUTION)
                .map(|j| {
                    let lt = j as f32 / RESOLUTION as f32;
                    let vel = bezier::de_casteljau(&vel_pts, lt);
                    let acc = bezier::de_casteljau(&acc_pts, lt);
                    let speed_sq = vel.length_squared();
                    if speed_sq > 1e-6 {
                        (vel.x * acc.y - vel.y * acc.x) / speed_sq.powf(1.5)
                    } else {
                        0.0
                    }
                })
                .collect()
        })
        .collect();

    let max_k = per_seg
        .iter()
        .flatten()
        .map(|k| k.abs())
        .fold(0.0f32, f32::max)
        .max(0.01);

    for (i, samples) in per_seg.iter().enumerate() {
        let gx0 = i as f32 / n_segs as f32;
        let gx1 = (i + 1) as f32 / n_segs as f32;
        let pts: Vec<Vec2> = samples
            .iter()
            .enumerate()
            .map(|(j, &k)| {
                let frac = j as f32 / RESOLUTION as f32;
                let gx = gx0 + frac * (gx1 - gx0);
                let norm = (k / max_k) * 0.5 + 0.5;
                vec2(origin.x + gx * plot_w, origin.y + norm * plot_h)
            })
            .collect();
        draw.polyline().weight(1.5).points(pts).color(rgba(0.8, 0.3, 1.0, 0.9));
    }

    // G2 jump markers at junctions
    for i in 0..n_segs.saturating_sub(1) {
        let left_k = *per_seg[i].last().unwrap_or(&0.0);
        let right_k = per_seg[i + 1].first().copied().unwrap_or(0.0);
        let jx = (i + 1) as f32 / n_segs as f32;
        let px = origin.x + jx * plot_w;
        let ly = origin.y + ((left_k / max_k) * 0.5 + 0.5) * plot_h;
        let ry = origin.y + ((right_k / max_k) * 0.5 + 0.5) * plot_h;

        if (left_k - right_k).abs() / max_k > 0.01 {
            draw.line()
                .start(pt2(px, ly))
                .end(pt2(px, ry))
                .weight(1.5)
                .color(rgba(1.0, 0.3, 0.3, 0.85));
            draw.ellipse().xy(pt2(px, ly)).radius(2.5).color(rgba(1.0, 0.3, 0.3, 1.0));
            draw.ellipse().xy(pt2(px, ry)).radius(2.5).color(rgba(1.0, 0.3, 0.3, 1.0));
        }
    }

    // Playhead
    let dot_x = origin.x + current_t * plot_w;
    let seg_i = ((current_t * n_segs as f32).floor() as usize).min(n_segs.saturating_sub(1));
    let current_k = if segments[seg_i].len() >= 3 {
        let lt = (current_t * n_segs as f32 - seg_i as f32).clamp(0.0, 1.0);
        let vel_pts = bezier::bezier_derivative_points(&segments[seg_i]);
        let acc_pts = bezier::bezier_derivative_points(&vel_pts);
        if acc_pts.is_empty() {
            0.0
        } else {
            let vel = bezier::de_casteljau(&vel_pts, lt);
            let acc = bezier::de_casteljau(&acc_pts, lt);
            let speed_sq = vel.length_squared();
            if speed_sq > 1e-6 {
                (vel.x * acc.y - vel.y * acc.x) / speed_sq.powf(1.5)
            } else {
                0.0
            }
        }
    } else {
        0.0
    };
    let dot_y = origin.y + ((current_k / max_k) * 0.5 + 0.5) * plot_h;
    draw.line()
        .start(pt2(dot_x, origin.y))
        .end(pt2(dot_x, origin.y + plot_h))
        .weight(1.0)
        .color(rgba(0.8, 0.3, 1.0, 0.5));
    draw.ellipse().xy(pt2(dot_x, dot_y)).radius(3.5).color(rgba(0.8, 0.3, 1.0, 1.0));
}

/// Computes the parametric (C) and geometric (G) continuity levels across all segment junctions.
/// Returns (c_level, g_level) where levels are 0, 1, or 2.
pub fn compute_continuity(segments: &[Vec<Vec2>]) -> (u8, u8) {
    let n_segs = segments.len();
    if n_segs <= 1 {
        return (2, 2);
    }

    let mut c_level: u8 = 2;
    let mut g_level: u8 = 2;

    for i in 0..n_segs - 1 {
        let seg_l = &segments[i];
        let seg_r = &segments[i + 1];
        if seg_l.len() < 2 || seg_r.len() < 2 {
            continue;
        }

        // C0 / G0: position match
        let pos_l = bezier::de_casteljau(seg_l, 1.0);
        let pos_r = bezier::de_casteljau(seg_r, 0.0);
        if pos_l.distance(pos_r) > 1.0 {
            c_level = 0;
            g_level = 0;
            continue;
        }

        // Velocity vectors at junction
        let vel_pts_l = bezier::bezier_derivative_points(seg_l);
        let vel_pts_r = bezier::bezier_derivative_points(seg_r);
        let vel_l = bezier::de_casteljau(&vel_pts_l, 1.0);
        let vel_r = bezier::de_casteljau(&vel_pts_r, 0.0);

        // C1: velocity vectors match exactly
        let vel_scale = vel_l.length().max(vel_r.length()).max(1.0);
        if vel_l.distance(vel_r) / vel_scale > 0.02 {
            c_level = c_level.min(0);
        }

        // G1: tangent directions match (angle difference)
        let angle_l = vel_l.y.atan2(vel_l.x);
        let angle_r = vel_r.y.atan2(vel_r.x);
        let angle_diff = (angle_l - angle_r).abs();
        let angle_diff = angle_diff.min(std::f32::consts::TAU - angle_diff);
        if angle_diff > 0.05 {
            g_level = g_level.min(0);
        }

        // C2 / G2: need at least cubic segments
        if seg_l.len() >= 3 && seg_r.len() >= 3 {
            let acc_pts_l = bezier::bezier_derivative_points(&vel_pts_l);
            let acc_pts_r = bezier::bezier_derivative_points(&vel_pts_r);
            if !acc_pts_l.is_empty() && !acc_pts_r.is_empty() {
                let acc_l = bezier::de_casteljau(&acc_pts_l, 1.0);
                let acc_r = bezier::de_casteljau(&acc_pts_r, 0.0);

                // C2: acceleration vectors match
                let acc_scale = acc_l.length().max(acc_r.length()).max(1.0);
                if acc_l.distance(acc_r) / acc_scale > 0.02 {
                    c_level = c_level.min(1);
                }

                // G2: curvature values match
                let speed_l_sq = vel_l.length_squared();
                let speed_r_sq = vel_r.length_squared();
                let curv_l = if speed_l_sq > 1e-6 {
                    (vel_l.x * acc_l.y - vel_l.y * acc_l.x) / speed_l_sq.powf(1.5)
                } else {
                    0.0
                };
                let curv_r = if speed_r_sq > 1e-6 {
                    (vel_r.x * acc_r.y - vel_r.y * acc_r.x) / speed_r_sq.powf(1.5)
                } else {
                    0.0
                };
                let curv_scale = curv_l.abs().max(curv_r.abs()).max(0.001);
                if (curv_l - curv_r).abs() / curv_scale > 0.05 {
                    g_level = g_level.min(1);
                }
            }
        } else {
            c_level = c_level.min(1);
            g_level = g_level.min(1);
        }
    }

    (c_level, g_level)
}

/// Draws the parametric (C) and geometric (G) continuity labels at the top of the screen.
pub fn draw_continuity_labels(draw: &Draw, win: Rect, segments: &[Vec<Vec2>]) {
    if segments.is_empty() {
        return;
    }

    let (c_level, g_level) = compute_continuity(segments);

    let level_color = |level: u8| -> Rgba<f32> {
        match level {
            0 => rgba(1.0, 0.35, 0.35, 0.95),
            1 => rgba(1.0, 0.85, 0.3, 0.95),
            _ => rgba(0.3, 1.0, 0.45, 0.95),
        }
    };

    let c_text = format!("C{}", c_level);
    let g_text = format!("G{}", g_level);
    let margin = 20.0;
    let top_y = win.top() - margin;

    // Top-left: geometric continuity (aligns with G1/G2 graphs on the left)
    draw.text(&g_text)
        .x_y(win.left() + margin + 14.0, top_y)
        .font_size(18)
        .color(level_color(g_level));

    // Top-right: parametric continuity (aligns with velocity/acceleration graphs on the right)
    draw.text(&c_text)
        .x_y(win.right() - margin - 14.0, top_y)
        .font_size(18)
        .color(level_color(c_level));
}

/// Draws the B-spline basis function graph in the bottom-right corner.
pub fn draw_bspline_influence_graph(
    draw: &Draw,
    win: Rect,
    points: &[ControlPoint],
    current_t: f32,
) {
    if points.len() < 4 {
        return;
    }

    let (origin, plot_w, plot_h) = plot_area(win);
    let panel_cx = origin.x - PADDING + PANEL_W / 2.0;
    let panel_cy = origin.y - PADDING + PANEL_H / 2.0;
    let n = points.len();
    let knots = crate::bspline::clamped_knots(n);

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
        .stroke(rgba(0.3f32, 0.7, 1.0, 0.3));

    draw.text("B-Spline Basis")
        .x_y(panel_cx, panel_cy + PANEL_H / 2.0 - 9.0)
        .font_size(10)
        .color(rgba(0.3f32, 0.7, 1.0, 0.65));

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

    // Draw each basis function N_{i,3}(t)
    for i in 0..n {
        let color = points[i].color;
        let curve_pts: Vec<Vec2> = (0..=RESOLUTION)
            .map(|j| {
                let s = j as f32 / RESOLUTION as f32;
                let t_val = s.clamp(0.0, 1.0 - 1e-7);
                let influence = crate::bspline::basis_function(i, 3, &knots, t_val);
                vec2(origin.x + s * plot_w, origin.y + influence * plot_h)
            })
            .collect();
        draw.polyline().weight(1.5).points(curve_pts).color(color);

        // Dot at current t
        let t_val = current_t.clamp(0.0, 1.0 - 1e-7);
        let influence_at_t = crate::bspline::basis_function(i, 3, &knots, t_val);
        let dot = vec2(
            origin.x + current_t * plot_w,
            origin.y + influence_at_t * plot_h,
        );
        draw.ellipse().xy(dot).radius(3.5).color(color);
    }

    // Vertical playhead line
    let playhead_x = origin.x + current_t * plot_w;
    draw.line()
        .start(pt2(playhead_x, origin.y))
        .end(pt2(playhead_x, origin.y + plot_h))
        .weight(1.0)
        .color(rgba(1.0f32, 1.0, 1.0, 0.5));
}

/// Draws the Hermite basis function influence graph in the bottom-right corner.
pub fn draw_hermite_influence_graph(
    draw: &Draw,
    win: Rect,
    seg_idx: usize,
    total_segs: usize,
    t: f32,
    p0_color: Rgb<f32>,
    p1_color: Rgb<f32>,
) {
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
        .stroke(rgba(1.0f32, 0.85, 0.3, 0.3));

    // Title + segment label
    let label = if total_segs > 1 {
        format!("Hermite Basis — Seg {}/{}", seg_idx + 1, total_segs)
    } else {
        "Hermite Basis".to_string()
    };
    draw.text(&label)
        .x_y(panel_cx, panel_cy + PANEL_H / 2.0 - 9.0)
        .font_size(10)
        .color(rgba(1.0f32, 0.85, 0.3, 0.65));

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

    // h00/h01 range [0,1], h10 range [0, 4/27], h11 range [-4/27, 0]
    let min_val = -0.2f32;
    let max_val = 1.05f32;
    let range = max_val - min_val;

    // Zero line
    let zero_y = origin.y + (-min_val / range) * plot_h;
    draw.line()
        .start(pt2(origin.x, zero_y))
        .end(pt2(origin.x + plot_w, zero_y))
        .weight(0.5)
        .color(rgba(1.0f32, 1.0, 1.0, 0.15));

    let basis_fns: [fn(f32) -> f32; 4] = [hermite::h00, hermite::h10, hermite::h01, hermite::h11];
    let tangent_color = rgba(1.0, 1.0, 0.4, 0.9);
    let tangent_color_dim = rgba(0.7, 0.7, 0.3, 0.9);
    let colors = [
        rgba(p0_color.red, p0_color.green, p0_color.blue, 1.0),
        tangent_color,
        rgba(p1_color.red, p1_color.green, p1_color.blue, 1.0),
        tangent_color_dim,
    ];

    for (i, basis_fn) in basis_fns.iter().enumerate() {
        let curve_pts: Vec<Vec2> = (0..=RESOLUTION)
            .map(|j| {
                let s = j as f32 / RESOLUTION as f32;
                let val = basis_fn(s);
                let y = (val - min_val) / range;
                vec2(origin.x + s * plot_w, origin.y + y * plot_h)
            })
            .collect();

        draw.polyline().weight(1.5).points(curve_pts).color(colors[i]);

        // Dot at current t
        let val_at_t = basis_fn(t);
        let dot_y = (val_at_t - min_val) / range;
        let dot = vec2(origin.x + t * plot_w, origin.y + dot_y * plot_h);
        draw.ellipse().xy(dot).radius(3.5).color(colors[i]);
    }

    // Vertical playhead line
    let playhead_x = origin.x + t * plot_w;
    draw.line()
        .start(pt2(playhead_x, origin.y))
        .end(pt2(playhead_x, origin.y + plot_h))
        .weight(1.0)
        .color(rgba(1.0f32, 1.0, 1.0, 0.5));
}

#[cfg(test)]
mod tests {
    use super::*;
    

    fn seg(pts: &[(f32, f32)]) -> Vec<Vec2> {
        pts.iter().map(|&(x, y)| vec2(x, y)).collect()
    }

    // ── compute_continuity ────────────────────────────────────────────────────

    #[test]
    fn single_segment_is_c2_g2() {
        let s = seg(&[(0.0,0.0),(1.0,1.0),(2.0,1.0),(3.0,0.0)]);
        assert_eq!(compute_continuity(&[s]), (2, 2));
    }

    #[test]
    fn empty_is_c2_g2() {
        assert_eq!(compute_continuity(&[]), (2, 2));
    }

    #[test]
    fn collinear_segments_equal_velocity_are_c2() {
        // Two straight-line cubics with uniform spacing → velocity matches at junction
        let s1 = seg(&[(0.0,0.0),(1.0,0.0),(2.0,0.0),(3.0,0.0)]);
        let s2 = seg(&[(3.0,0.0),(4.0,0.0),(5.0,0.0),(6.0,0.0)]);
        let (c, g) = compute_continuity(&[s1, s2]);
        assert!(c >= 2, "expected C2, got C{c}");
        assert!(g >= 2, "expected G2, got G{g}");
    }

    #[test]
    fn perpendicular_tangents_at_junction_are_c0_only() {
        // Segment 1 ends moving right, segment 2 starts moving up → G1 breaks
        let s1 = seg(&[(0.0,0.0),(0.33,0.0),(0.67,0.0),(1.0,0.0)]);
        let s2 = seg(&[(1.0,0.0),(1.0,0.33),(1.0,0.67),(1.0,1.0)]);
        let (c, g) = compute_continuity(&[s1, s2]);
        assert_eq!(c, 0, "expected C0");
        assert_eq!(g, 0, "expected G0");
    }

    #[test]
    fn matching_direction_different_speed_is_g1_not_c1() {
        // Same direction, different speed → G1 but not C1
        // vel at junction: s1 ends with (0.9,0) scaled ×3, s2 starts with (3,0) scaled ×3
        let s1 = seg(&[(0.0,0.0),(0.1,0.0),(0.2,0.0),(0.3,0.0)]);
        let s2 = seg(&[(0.3,0.0),(1.3,0.0),(2.3,0.0),(3.3,0.0)]);
        let (c, g) = compute_continuity(&[s1, s2]);
        assert_eq!(c, 0, "expected C0 (speeds differ)");
        assert!(g >= 1, "expected at least G1 (directions match)");
    }

    #[test]
    fn c1_but_not_c2_curvature_jump() {
        // seg1: [(-3,0),(-2,0),(-1,0),(0,0)] → vel at t=1: (3,0), accel: (0,0)
        // seg2: [(0,0),(1,0),(1,1),(2,1)]     → vel at t=0: (3,0), accel: 6*(-1,1)
        // → velocity matches (C1), acceleration doesn't (not C2)
        let s1 = seg(&[(-3.0,0.0),(-2.0,0.0),(-1.0,0.0),(0.0,0.0)]);
        let s2 = seg(&[(0.0,0.0),(1.0,0.0),(1.0,1.0),(2.0,1.0)]);
        let (c, g) = compute_continuity(&[s1, s2]);
        assert_eq!(c, 1, "expected C1");
        assert_eq!(g, 1, "expected G1");
    }
}