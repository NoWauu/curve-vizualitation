use nannou::prelude::*;

fn lerp(p0: Vec2, p1: Vec2, t: f32) -> Vec2 {
    (1.0 - t) * p0 + t * p1
}

/// De Casteljau's algorithm — evaluates an n-degree Bezier curve at parameter `t`.
pub fn de_casteljau(points: &[Vec2], t: f32) -> Vec2 {
    if points.len() == 1 {
        return points[0];
    }

    let reduced: Vec<Vec2> = points
        .windows(2)
        .map(|w| lerp(w[0], w[1], t))
        .collect();

    de_casteljau(&reduced, t)
}

fn binomial(n: usize, k: usize) -> f32 {
    if k > n {
        return 0.0;
    }
    let k = k.min(n - k);
    let mut result = 1.0f32;
    for i in 0..k {
        result *= (n - i) as f32 / (i + 1) as f32;
    }
    result
}

/// Evaluates the i-th Bernstein basis polynomial of degree n at t.
/// This is the "influence" (weight) of the i-th control point.
pub fn bernstein_basis(n: usize, i: usize, t: f32) -> f32 {
    binomial(n, i) * t.powi(i as i32) * (1.0 - t).powi((n - i) as i32)
}

/// Samples the Bezier curve at `resolution + 1` evenly spaced t values.
pub fn sample_curve(control_points: &[Vec2], resolution: usize) -> Vec<Vec2> {
    (0..=resolution)
        .map(|i| {
            let t = i as f32 / resolution as f32;
            de_casteljau(control_points, t)
        })
        .collect()
}

// ── Piecewise spline ──────────────────────────────────────────────────────────

/// Splits n control points into cubic Bézier segments of at most 4 points.
/// Adjacent segments share their junction point (C0 continuity).
/// Returns a list of (start, end_exclusive) index pairs.
pub fn piecewise_segment_ranges(n: usize) -> Vec<(usize, usize)> {
    if n < 2 {
        return vec![];
    }
    let mut ranges = vec![];
    let mut start = 0;
    while start < n - 1 {
        let end = (start + 4).min(n);
        ranges.push((start, end));
        if end == n {
            break;
        }
        start = end - 1; // share the junction point
    }
    ranges
}

/// Maps a global t ∈ [0, 1] to `(segment_index, local_t ∈ [0, 1])`.
/// Each segment owns an equal fraction of the global t range.
pub fn global_to_local_t(ranges: &[(usize, usize)], global_t: f32) -> (usize, f32) {
    let n = ranges.len();
    if n == 0 {
        return (0, global_t);
    }
    let scaled = (global_t * n as f32).clamp(0.0, n as f32);
    let seg = (scaled.floor() as usize).min(n - 1);
    let local_t = (scaled - seg as f32).clamp(0.0, 1.0);
    (seg, local_t)
}

/// Evaluates the piecewise spline at a global t ∈ [0, 1].
pub fn evaluate_piecewise(points: &[Vec2], global_t: f32) -> Vec2 {
    let ranges = piecewise_segment_ranges(points.len());
    if ranges.is_empty() {
        return points.first().copied().unwrap_or(Vec2::ZERO);
    }
    let (seg, local_t) = global_to_local_t(&ranges, global_t);
    let (s, e) = ranges[seg];
    de_casteljau(&points[s..e], local_t)
}
