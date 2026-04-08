use nannou::prelude::*;

/// Hermite basis function h00: weight of start position P0.
pub fn h00(t: f32) -> f32 {
    2.0 * t * t * t - 3.0 * t * t + 1.0
}

/// Hermite basis function h10: weight of start tangent M0.
pub fn h10(t: f32) -> f32 {
    t * t * t - 2.0 * t * t + t
}

/// Hermite basis function h01: weight of end position P1.
pub fn h01(t: f32) -> f32 {
    -2.0 * t * t * t + 3.0 * t * t
}

/// Hermite basis function h11: weight of end tangent M1.
pub fn h11(t: f32) -> f32 {
    t * t * t - t * t
}

/// Evaluates a single cubic Hermite segment at parameter t in [0, 1].
pub fn evaluate_segment(p0: Vec2, m0: Vec2, p1: Vec2, m1: Vec2, t: f32) -> Vec2 {
    h00(t) * p0 + h10(t) * m0 + h01(t) * p1 + h11(t) * m1
}

/// Converts a Hermite segment (p0, m0, p1, m1) to 4 Bezier control points.
pub fn hermite_to_bezier(p0: Vec2, m0: Vec2, p1: Vec2, m1: Vec2) -> [Vec2; 4] {
    [p0, p0 + m0 / 3.0, p1 - m1 / 3.0, p1]
}

/// Samples the full piecewise Hermite curve at evenly-spaced t values.
pub fn sample_curve(points: &[Vec2], tangents: &[Vec2], resolution: usize) -> Vec<Vec2> {
    if points.len() < 2 {
        return vec![];
    }
    let n_segs = points.len() - 1;
    let mut result = Vec::with_capacity(n_segs * resolution + 1);
    for seg in 0..n_segs {
        let steps = if seg == n_segs - 1 { resolution + 1 } else { resolution };
        for j in 0..steps {
            let t = j as f32 / resolution as f32;
            result.push(evaluate_segment(
                points[seg],
                tangents[seg],
                points[seg + 1],
                tangents[seg + 1],
                t,
            ));
        }
    }
    result
}

/// Evaluates the piecewise Hermite curve at global t in [0, 1].
pub fn evaluate_piecewise(points: &[Vec2], tangents: &[Vec2], global_t: f32) -> Vec2 {
    if points.len() < 2 {
        return points.first().copied().unwrap_or(Vec2::ZERO);
    }
    let (seg, local_t) = global_to_local_t(points.len(), global_t);
    evaluate_segment(
        points[seg],
        tangents[seg],
        points[seg + 1],
        tangents[seg + 1],
        local_t,
    )
}

/// Maps a global t in [0, 1] to (segment_index, local_t) for n_points.
pub fn global_to_local_t(n_points: usize, global_t: f32) -> (usize, f32) {
    if n_points < 2 {
        return (0, global_t);
    }
    let n_segs = n_points - 1;
    let scaled = (global_t * n_segs as f32).clamp(0.0, n_segs as f32);
    let seg = (scaled.floor() as usize).min(n_segs - 1);
    let local_t = (scaled - seg as f32).clamp(0.0, 1.0);
    (seg, local_t)
}
