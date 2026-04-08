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
