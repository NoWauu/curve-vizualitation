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

/// Returns the control points of the Bézier derivative curve (degree n−1).
/// For a degree-n curve with n+1 control points, returns n control points Q_i = n*(P_{i+1}−P_i).
pub fn bezier_derivative_points(pts: &[Vec2]) -> Vec<Vec2> {
    let n = pts.len().saturating_sub(1);
    (0..n).map(|i| (pts[i + 1] - pts[i]) * n as f32).collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    

    const EPS: f32 = 1e-5;

    fn approx(a: f32, b: f32) -> bool { (a - b).abs() < EPS }
    fn approx_v2(a: Vec2, b: Vec2) -> bool { (a - b).length() < EPS }

    // ── de_casteljau ──────────────────────────────────────────────────────────

    #[test]
    fn de_casteljau_single_point() {
        let pts = vec![vec2(3.0, 7.0)];
        assert!(approx_v2(de_casteljau(&pts, 0.5), vec2(3.0, 7.0)));
    }

    #[test]
    fn de_casteljau_linear_endpoints_and_midpoint() {
        let pts = vec![vec2(0.0, 0.0), vec2(4.0, 2.0)];
        assert!(approx_v2(de_casteljau(&pts, 0.0), vec2(0.0, 0.0)));
        assert!(approx_v2(de_casteljau(&pts, 1.0), vec2(4.0, 2.0)));
        assert!(approx_v2(de_casteljau(&pts, 0.5), vec2(2.0, 1.0)));
    }

    #[test]
    fn de_casteljau_cubic_midpoint() {
        // (0,0),(1,0),(1,1),(2,1) at t=0.5 → (1.0, 0.5) by hand
        let pts = vec![vec2(0.0,0.0), vec2(1.0,0.0), vec2(1.0,1.0), vec2(2.0,1.0)];
        assert!(approx_v2(de_casteljau(&pts, 0.5), vec2(1.0, 0.5)));
    }

    #[test]
    fn de_casteljau_interpolates_endpoints() {
        let pts = vec![vec2(1.0,2.0), vec2(3.0,4.0), vec2(5.0,-1.0), vec2(-2.0,3.0)];
        assert!(approx_v2(de_casteljau(&pts, 0.0), pts[0]));
        assert!(approx_v2(de_casteljau(&pts, 1.0), *pts.last().unwrap()));
    }

    // ── bernstein_basis ───────────────────────────────────────────────────────

    #[test]
    fn bernstein_partition_of_unity() {
        let ts = [0.0f32, 0.25, 0.5, 0.75, 1.0];
        for &t in &ts {
            for n in 1..=5 {
                let sum: f32 = (0..=n).map(|i| bernstein_basis(n, i, t)).sum();
                assert!(approx(sum, 1.0), "n={n} t={t} sum={sum}");
            }
        }
    }

    #[test]
    fn bernstein_non_negative() {
        for i in 0..=3 {
            for j in 0..=10 {
                assert!(bernstein_basis(3, i, j as f32 / 10.0) >= 0.0);
            }
        }
    }

    #[test]
    fn bernstein_endpoint_values() {
        // only B_{n,0}(0) == 1
        assert!(approx(bernstein_basis(3, 0, 0.0), 1.0));
        for i in 1..=3 { assert!(approx(bernstein_basis(3, i, 0.0), 0.0)); }
        // only B_{n,n}(1) == 1
        assert!(approx(bernstein_basis(3, 3, 1.0), 1.0));
        for i in 0..=2 { assert!(approx(bernstein_basis(3, i, 1.0), 0.0)); }
    }

    // ── piecewise_segment_ranges ──────────────────────────────────────────────

    #[test]
    fn piecewise_ranges_small_inputs() {
        assert!(piecewise_segment_ranges(0).is_empty());
        assert!(piecewise_segment_ranges(1).is_empty());
        assert_eq!(piecewise_segment_ranges(2), vec![(0, 2)]);
        assert_eq!(piecewise_segment_ranges(4), vec![(0, 4)]);
    }

    #[test]
    fn piecewise_ranges_junction_shared() {
        // 5 points → [(0,4),(3,5)]: point at index 3 is shared
        let r = piecewise_segment_ranges(5);
        assert_eq!(r, vec![(0, 4), (3, 5)]);
        assert_eq!(r[0].1 - 1, r[1].0); // shared junction
    }

    #[test]
    fn piecewise_ranges_7_points() {
        let r = piecewise_segment_ranges(7);
        assert_eq!(r, vec![(0, 4), (3, 7)]);
    }

    // ── global_to_local_t ─────────────────────────────────────────────────────

    #[test]
    fn global_to_local_boundaries() {
        let ranges = vec![(0usize, 4usize), (3, 7)];
        let (seg, lt) = global_to_local_t(&ranges, 0.0);
        assert_eq!(seg, 0);
        assert!(approx(lt, 0.0));
        let (seg, lt) = global_to_local_t(&ranges, 1.0);
        assert_eq!(seg, 1);
        assert!(approx(lt, 1.0));
    }

    #[test]
    fn global_to_local_midpoints() {
        let ranges = vec![(0usize, 4usize), (3, 7)];
        // t=0.25 → first quarter → seg 0, local 0.5
        let (seg, lt) = global_to_local_t(&ranges, 0.25);
        assert_eq!(seg, 0);
        assert!(approx(lt, 0.5));
        // t=0.75 → third quarter → seg 1, local 0.5
        let (seg, lt) = global_to_local_t(&ranges, 0.75);
        assert_eq!(seg, 1);
        assert!(approx(lt, 0.5));
    }

    // ── bezier_derivative_points ──────────────────────────────────────────────

    #[test]
    fn derivative_points_linear() {
        // degree 1: Q_0 = 1*(P1-P0)
        let pts = vec![vec2(0.0, 0.0), vec2(3.0, 4.0)];
        let d = bezier_derivative_points(&pts);
        assert_eq!(d.len(), 1);
        assert!(approx_v2(d[0], vec2(3.0, 4.0)));
    }

    #[test]
    fn derivative_points_quadratic() {
        // degree 2: Q_i = 2*(P_{i+1}-P_i)
        let pts = vec![vec2(0.0,0.0), vec2(1.0,2.0), vec2(3.0,0.0)];
        let d = bezier_derivative_points(&pts);
        assert_eq!(d.len(), 2);
        assert!(approx_v2(d[0], vec2(2.0, 4.0)));
        assert!(approx_v2(d[1], vec2(4.0, -4.0)));
    }

    #[test]
    fn derivative_points_constant_curve_is_zero() {
        // All coincident points → derivative is zero
        let p = vec2(5.0, 3.0);
        let pts = vec![p, p, p, p];
        let d = bezier_derivative_points(&pts);
        for q in &d { assert!(approx_v2(*q, Vec2::ZERO)); }
    }

    // ── evaluate_piecewise ────────────────────────────────────────────────────

    #[test]
    fn evaluate_piecewise_endpoints() {
        let pts = vec![vec2(0.0,0.0), vec2(1.0,1.0), vec2(2.0,1.0), vec2(3.0,0.0)];
        assert!(approx_v2(evaluate_piecewise(&pts, 0.0), pts[0]));
        assert!(approx_v2(evaluate_piecewise(&pts, 1.0), *pts.last().unwrap()));
    }

    #[test]
    fn evaluate_piecewise_matches_de_casteljau_single_segment() {
        let pts = vec![vec2(0.0,0.0), vec2(1.0,2.0), vec2(3.0,2.0), vec2(4.0,0.0)];
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            assert!(approx_v2(evaluate_piecewise(&pts, t), de_casteljau(&pts, t)),
                "mismatch at t={t}");
        }
    }
}
