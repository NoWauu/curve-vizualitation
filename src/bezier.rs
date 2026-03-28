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
