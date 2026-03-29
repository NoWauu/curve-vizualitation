use nannou::prelude::*;

use crate::bezier::{self, bernstein_basis};
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