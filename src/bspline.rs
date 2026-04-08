use nannou::prelude::*;

/// Generates a clamped uniform knot vector for a degree-3 B-spline with `n` control points.
/// Returns `n + 4` knots.
pub fn clamped_knots(n: usize) -> Vec<f32> {
    let p = 3;
    let m = n + p; // last knot index
    let n_spans = n - p;
    (0..=m)
        .map(|i| {
            if i <= p {
                0.0
            } else if i >= n {
                1.0
            } else {
                (i - p) as f32 / n_spans as f32
            }
        })
        .collect()
}

/// De Boor's algorithm: evaluates a cubic (degree-3) B-spline at parameter `t`.
pub fn de_boor(points: &[Vec2], knots: &[f32], t: f32) -> Vec2 {
    let n = points.len();
    let p = 3;
    let t = t.clamp(knots[p], knots[n] - 1e-7);

    // Find knot span k such that t ∈ [knots[k], knots[k+1])
    let mut k = p;
    for i in p..n {
        if t >= knots[i] && t < knots[i + 1] {
            k = i;
            break;
        }
    }

    let mut d: Vec<Vec2> = (0..=p).map(|j| points[k - p + j]).collect();

    for r in 1..=p {
        for j in (r..=p).rev() {
            let idx = k - p + j;
            let denom = knots[idx + p + 1 - r] - knots[idx];
            let alpha = if denom.abs() < 1e-10 {
                0.0
            } else {
                (t - knots[idx]) / denom
            };
            d[j] = (1.0 - alpha) * d[j - 1] + alpha * d[j];
        }
    }

    d[p]
}

/// Cox-de Boor recursion: evaluates the i-th B-spline basis function of degree `p` at `t`.
pub fn basis_function(i: usize, p: usize, knots: &[f32], t: f32) -> f32 {
    if p == 0 {
        return if t >= knots[i] && t < knots[i + 1] {
            1.0
        } else {
            0.0
        };
    }

    let left_denom = knots[i + p] - knots[i];
    let left = if left_denom.abs() < 1e-10 {
        0.0
    } else {
        (t - knots[i]) / left_denom * basis_function(i, p - 1, knots, t)
    };

    let right_denom = knots[i + p + 1] - knots[i + 1];
    let right = if right_denom.abs() < 1e-10 {
        0.0
    } else {
        (knots[i + p + 1] - t) / right_denom * basis_function(i + 1, p - 1, knots, t)
    };

    left + right
}

/// Samples the full B-spline curve at `resolution + 1` evenly spaced parameter values.
pub fn sample_curve(points: &[Vec2], resolution: usize) -> Vec<Vec2> {
    if points.len() < 4 {
        return vec![];
    }
    let knots = clamped_knots(points.len());
    (0..=resolution)
        .map(|i| {
            let t = i as f32 / resolution as f32;
            de_boor(points, &knots, t)
        })
        .collect()
}

/// Evaluates the B-spline at global t ∈ [0, 1].
pub fn evaluate(points: &[Vec2], t: f32) -> Vec2 {
    if points.len() < 4 {
        return points.first().copied().unwrap_or(Vec2::ZERO);
    }
    let knots = clamped_knots(points.len());
    de_boor(points, &knots, t)
}

/// Converts each B-spline span into 4 Bezier control points (exact for cubics).
/// Uses 4-point sampling and linear solve.
#[allow(dead_code)]
pub fn spans_to_bezier(points: &[Vec2]) -> Vec<Vec<Vec2>> {
    if points.len() < 4 {
        return vec![];
    }
    let knots = clamped_knots(points.len());
    let n_spans = points.len() - 3;

    (0..n_spans)
        .map(|span| {
            let t0 = knots[span + 3];
            let t1 = knots[span + 4];

            let p0 = de_boor(points, &knots, t0);
            let p_13 = de_boor(points, &knots, t0 + (t1 - t0) / 3.0);
            let p_23 = de_boor(points, &knots, t0 + (t1 - t0) * 2.0 / 3.0);
            let p3 = de_boor(points, &knots, t1 - 1e-7);

            // Solve for interior Bezier control points from cubic interpolation
            let a = p_13 - p0 * (8.0 / 27.0) - p3 * (1.0 / 27.0);
            let b = p_23 - p0 * (1.0 / 27.0) - p3 * (8.0 / 27.0);
            let b1 = a * 3.0 - b * 1.5;
            let b2 = b * 3.0 - a * 1.5;

            vec![p0, b1, b2, p3]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    

    const EPS: f32 = 1e-4;

    fn approx(a: f32, b: f32) -> bool { (a - b).abs() < EPS }
    fn approx_v2(a: Vec2, b: Vec2) -> bool { (a - b).length() < EPS }

    // ── clamped_knots ─────────────────────────────────────────────────────────

    #[test]
    fn knots_length() {
        // n control points → n + 4 knots
        for n in 4..=8 {
            assert_eq!(clamped_knots(n).len(), n + 4);
        }
    }

    #[test]
    fn knots_n4_is_all_clamped() {
        // [0,0,0,0, 1,1,1,1]
        let k = clamped_knots(4);
        assert_eq!(&k[..4], &[0.0, 0.0, 0.0, 0.0]);
        assert_eq!(&k[4..], &[1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn knots_n5_has_one_interior() {
        // [0,0,0,0, 0.5, 1,1,1,1]
        let k = clamped_knots(5);
        assert!(approx(k[4], 0.5));
        assert_eq!(&k[5..], &[1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn knots_n6_interior_spacing() {
        // [0,0,0,0, 1/3, 2/3, 1,1,1,1]
        let k = clamped_knots(6);
        assert!(approx(k[4], 1.0 / 3.0));
        assert!(approx(k[5], 2.0 / 3.0));
    }

    #[test]
    fn knots_start_and_end_clamped() {
        for n in 4..=9 {
            let k = clamped_knots(n);
            assert_eq!(&k[..4], &[0.0, 0.0, 0.0, 0.0]);
            assert_eq!(&k[n..], &[1.0, 1.0, 1.0, 1.0]);
        }
    }

    // ── de_boor ───────────────────────────────────────────────────────────────

    #[test]
    fn de_boor_n4_equals_de_casteljau() {
        // With n=4 and knots [0,0,0,0,1,1,1,1] the B-spline is a plain cubic Bézier.
        let pts = vec![vec2(0.0,0.0), vec2(1.0,2.0), vec2(3.0,2.0), vec2(4.0,0.0)];
        let knots = clamped_knots(4);
        for i in 0..=20 {
            let t = i as f32 / 20.0;
            let bs = de_boor(&pts, &knots, t);
            let bz = crate::bezier::de_casteljau(&pts, t);
            assert!(approx_v2(bs, bz), "t={t}: de_boor={bs:?} de_casteljau={bz:?}");
        }
    }

    #[test]
    fn clamped_start_interpolates_first_point() {
        let pts = vec![vec2(1.0,2.0), vec2(3.0,4.0), vec2(5.0,1.0), vec2(2.0,-1.0), vec2(0.0,3.0)];
        assert!(approx_v2(evaluate(&pts, 0.0), pts[0]));
    }

    #[test]
    fn clamped_end_interpolates_last_point() {
        let pts = vec![vec2(1.0,2.0), vec2(3.0,4.0), vec2(5.0,1.0), vec2(2.0,-1.0), vec2(0.0,3.0)];
        // endpoint is approximate due to the 1e-7 clamp inside de_boor
        assert!(approx_v2(evaluate(&pts, 1.0), *pts.last().unwrap()));
    }

    // ── basis_function ────────────────────────────────────────────────────────

    #[test]
    fn basis_partition_of_unity() {
        let n = 6;
        let knots = clamped_knots(n);
        // avoid t=1 because of the open-right-interval convention
        for i in 1..=19 {
            let t = i as f32 / 20.0;
            let sum: f32 = (0..n).map(|j| basis_function(j, 3, &knots, t)).sum();
            assert!(approx(sum, 1.0), "t={t} sum={sum}");
        }
    }

    #[test]
    fn basis_non_negative() {
        let n = 5;
        let knots = clamped_knots(n);
        for i in 0..n {
            for j in 0..20 {
                let t = j as f32 / 20.0;
                assert!(basis_function(i, 3, &knots, t) >= -1e-6,
                    "basis[{i}] < 0 at t={t}");
            }
        }
    }

    #[test]
    fn basis_sum_matches_de_boor_evaluation() {
        // Sum of basis-weighted control points must equal de_boor at every t.
        let pts = vec![
            vec2(0.0, 0.0), vec2(1.0, 2.0), vec2(3.0, 1.0),
            vec2(4.0, 3.0), vec2(5.0, 0.0),
        ];
        let knots = clamped_knots(pts.len());
        for i in 1..=19 {
            let t = i as f32 / 20.0;
            let from_boor = de_boor(&pts, &knots, t);
            let from_basis = pts.iter().enumerate().fold(Vec2::ZERO, |acc, (j, &p)| {
                acc + p * basis_function(j, 3, &knots, t)
            });
            assert!(approx_v2(from_boor, from_basis), "t={t}");
        }
    }

    // ── spans_to_bezier ───────────────────────────────────────────────────────

    #[test]
    fn spans_count() {
        // n control points → n-3 spans
        let pts5: Vec<Vec2> = (0..5).map(|i| vec2(i as f32, 0.0)).collect();
        let pts7: Vec<Vec2> = (0..7).map(|i| vec2(i as f32, 0.0)).collect();
        assert_eq!(spans_to_bezier(&pts5).len(), 2);
        assert_eq!(spans_to_bezier(&pts7).len(), 4);
    }

    #[test]
    fn spans_adjacent_share_junction() {
        let pts = vec![
            vec2(0.0,0.0), vec2(1.0,1.0), vec2(3.0,1.0),
            vec2(4.0,0.0), vec2(5.0,-1.0),
        ];
        let segs = spans_to_bezier(&pts);
        let last = *segs[0].last().unwrap();
        let first = segs[1][0];
        assert!(approx_v2(last, first), "junction mismatch: {last:?} vs {first:?}");
    }

    #[test]
    fn spans_bezier_matches_bspline_samples() {
        let pts = vec![
            vec2(0.0,0.0), vec2(1.0,2.0), vec2(3.0,2.0),
            vec2(4.0,0.0), vec2(5.0,1.0),
        ];
        let knots = clamped_knots(pts.len());
        let segs = spans_to_bezier(&pts);
        let n_spans = segs.len();
        for span in 0..n_spans {
            let t0 = knots[span + 3];
            let t1 = knots[span + 4];
            for j in 0..=8 {
                let lt = j as f32 / 8.0;
                let global_t = t0 + lt * (t1 - t0);
                let from_bspline = de_boor(&pts, &knots, global_t);
                let from_bezier  = crate::bezier::de_casteljau(&segs[span], lt);
                assert!(approx_v2(from_bspline, from_bezier),
                    "span={span} lt={lt:.3}: bspline={from_bspline:?} bezier={from_bezier:?}");
            }
        }
    }

    #[test]
    fn spans_too_few_points_returns_empty() {
        for n in 0..4 {
            let pts: Vec<Vec2> = (0..n).map(|i| vec2(i as f32, 0.0)).collect();
            assert!(spans_to_bezier(&pts).is_empty());
        }
    }
}
