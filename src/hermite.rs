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

#[cfg(test)]
mod tests {
    use super::*;
    

    const EPS: f32 = 1e-5;

    fn approx(a: f32, b: f32) -> bool { (a - b).abs() < EPS }
    fn approx_v2(a: Vec2, b: Vec2) -> bool { (a - b).length() < EPS }

    // ── basis functions ───────────────────────────────────────────────────────

    #[test]
    fn basis_endpoint_values() {
        assert!(approx(h00(0.0), 1.0));
        assert!(approx(h00(1.0), 0.0));
        assert!(approx(h10(0.0), 0.0));
        assert!(approx(h10(1.0), 0.0));
        assert!(approx(h01(0.0), 0.0));
        assert!(approx(h01(1.0), 1.0));
        assert!(approx(h11(0.0), 0.0));
        assert!(approx(h11(1.0), 0.0));
    }

    #[test]
    fn position_basis_sums_to_one() {
        // h00 + h01 = 1 everywhere (they blend the two endpoint positions)
        for i in 0..=20 {
            let t = i as f32 / 20.0;
            assert!(approx(h00(t) + h01(t), 1.0), "t={t}");
        }
    }

    #[test]
    fn h10_peak_is_positive_h11_trough_is_negative() {
        // h10 peaks at t=1/3, h11 troughs at t=2/3
        let peak = h10(1.0 / 3.0);
        let trough = h11(2.0 / 3.0);
        assert!(peak > 0.0);
        assert!(trough < 0.0);
    }

    // ── evaluate_segment ─────────────────────────────────────────────────────

    #[test]
    fn segment_interpolates_endpoints() {
        let p0 = vec2(1.0, 2.0);
        let m0 = vec2(0.5, -1.0);
        let p1 = vec2(4.0, -1.0);
        let m1 = vec2(1.0, 2.0);
        assert!(approx_v2(evaluate_segment(p0, m0, p1, m1, 0.0), p0));
        assert!(approx_v2(evaluate_segment(p0, m0, p1, m1, 1.0), p1));
    }

    #[test]
    fn segment_zero_tangents_is_smooth_interpolation() {
        // Zero tangents: curve still interpolates endpoints
        let p0 = vec2(0.0, 0.0);
        let p1 = vec2(4.0, 0.0);
        assert!(approx_v2(evaluate_segment(p0, Vec2::ZERO, p1, Vec2::ZERO, 0.0), p0));
        assert!(approx_v2(evaluate_segment(p0, Vec2::ZERO, p1, Vec2::ZERO, 1.0), p1));
        // Midpoint is average when tangents are zero (symmetry)
        let mid = evaluate_segment(p0, Vec2::ZERO, p1, Vec2::ZERO, 0.5);
        assert!(approx_v2(mid, vec2(2.0, 0.0)));
    }

    #[test]
    fn segment_coincident_points_zero_tangents() {
        let p = vec2(3.0, 5.0);
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            assert!(approx_v2(evaluate_segment(p, Vec2::ZERO, p, Vec2::ZERO, t), p));
        }
    }

    // ── hermite_to_bezier ─────────────────────────────────────────────────────

    #[test]
    fn bezier_conversion_endpoints_and_handles() {
        let p0 = vec2(0.0, 0.0);
        let m0 = vec2(3.0, 0.0);
        let p1 = vec2(6.0, 0.0);
        let m1 = vec2(3.0, 0.0);
        let bez = hermite_to_bezier(p0, m0, p1, m1);
        assert!(approx_v2(bez[0], p0));
        assert!(approx_v2(bez[3], p1));
        assert!(approx_v2(bez[1], p0 + m0 / 3.0));
        assert!(approx_v2(bez[2], p1 - m1 / 3.0));
    }

    #[test]
    fn bezier_conversion_matches_hermite_evaluation() {
        let p0 = vec2(0.0, 0.0);
        let m0 = vec2(2.0, 1.0);
        let p1 = vec2(3.0, 2.0);
        let m1 = vec2(-1.0, 0.5);
        let bez = hermite_to_bezier(p0, m0, p1, m1);
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            let h = evaluate_segment(p0, m0, p1, m1, t);
            let b = crate::bezier::de_casteljau(&bez, t);
            assert!(approx_v2(h, b), "t={t}: hermite={h:?} bezier={b:?}");
        }
    }

    // ── global_to_local_t ─────────────────────────────────────────────────────

    #[test]
    fn global_to_local_single_segment() {
        let (seg, lt) = global_to_local_t(2, 0.0);
        assert_eq!(seg, 0); assert!(approx(lt, 0.0));
        let (seg, lt) = global_to_local_t(2, 1.0);
        assert_eq!(seg, 0); assert!(approx(lt, 1.0));
        let (seg, lt) = global_to_local_t(2, 0.5);
        assert_eq!(seg, 0); assert!(approx(lt, 0.5));
    }

    #[test]
    fn global_to_local_two_segments() {
        // 3 points → 2 segments, each spans half the global range
        let (seg, lt) = global_to_local_t(3, 0.25);
        assert_eq!(seg, 0); assert!(approx(lt, 0.5));
        let (seg, lt) = global_to_local_t(3, 0.75);
        assert_eq!(seg, 1); assert!(approx(lt, 0.5));
        let (seg, lt) = global_to_local_t(3, 1.0);
        assert_eq!(seg, 1); assert!(approx(lt, 1.0));
    }

    #[test]
    fn global_to_local_degenerate_single_point() {
        // n_points < 2 → falls back to (0, t)
        let (seg, lt) = global_to_local_t(1, 0.7);
        assert_eq!(seg, 0); assert!(approx(lt, 0.7));
    }

    // ── sample_curve / evaluate_piecewise consistency ─────────────────────────

    #[test]
    fn piecewise_endpoints_match_control_points() {
        let pts = vec![vec2(0.0,0.0), vec2(2.0,3.0), vec2(5.0,1.0)];
        let tans = vec![vec2(1.0,1.0), vec2(2.0,0.0), vec2(1.0,-1.0)];
        let start = evaluate_piecewise(&pts, &tans, 0.0);
        let end   = evaluate_piecewise(&pts, &tans, 1.0);
        assert!(approx_v2(start, pts[0]));
        assert!(approx_v2(end,   *pts.last().unwrap()));
    }
}
