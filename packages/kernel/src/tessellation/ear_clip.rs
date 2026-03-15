//! Simple ear-clipping triangulation for 2D polygons.
//!
//! Works correctly for simple (non-self-intersecting) polygons,
//! both convex and concave. O(n²) worst case.

/// Triangulate a simple 2D polygon into triangles.
/// Returns triangle indices into the input point array.
///
/// Points must be in counter-clockwise order for correct triangle winding.
pub fn triangulate(points: &[[f64; 2]]) -> Vec<[usize; 3]> {
    let n = points.len();
    if n < 3 {
        return vec![];
    }
    if n == 3 {
        return vec![[0, 1, 2]];
    }

    // Ensure CCW winding
    let area = signed_area(points);
    let mut indices: Vec<usize> = if area >= 0.0 {
        (0..n).collect()
    } else {
        (0..n).rev().collect()
    };

    let mut triangles = Vec::with_capacity(n - 2);
    let mut remaining = indices.len();

    let mut i = 0;
    let mut attempts = 0;
    let max_attempts = remaining * remaining; // safety limit

    while remaining > 2 && attempts < max_attempts {
        attempts += 1;
        let prev = indices[if i == 0 { remaining - 1 } else { i - 1 }];
        let curr = indices[i % remaining];
        let next = indices[(i + 1) % remaining];

        if is_ear(&points, &indices, remaining, prev, curr, next) {
            triangles.push([prev, curr, next]);
            // Remove the ear tip
            indices.remove(i % remaining);
            remaining -= 1;
            if i >= remaining && remaining > 0 {
                i = 0;
            }
        } else {
            i = (i + 1) % remaining;
        }
    }

    triangles
}

fn signed_area(points: &[[f64; 2]]) -> f64 {
    let n = points.len();
    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += points[i][0] * points[j][1];
        area -= points[j][0] * points[i][1];
    }
    area * 0.5
}

fn is_ear(
    points: &[[f64; 2]],
    indices: &[usize],
    remaining: usize,
    prev: usize,
    curr: usize,
    next: usize,
) -> bool {
    let a = points[prev];
    let b = points[curr];
    let c = points[next];

    // Must be convex (CCW triangle)
    if cross_2d(a, b, c) <= 0.0 {
        return false;
    }

    // No other vertex inside this triangle
    for k in 0..remaining {
        let idx = indices[k];
        if idx == prev || idx == curr || idx == next {
            continue;
        }
        if point_in_triangle(points[idx], a, b, c) {
            return false;
        }
    }

    true
}

fn cross_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn point_in_triangle(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let d1 = cross_2d(a, b, p);
    let d2 = cross_2d(b, c, p);
    let d3 = cross_2d(c, a, p);
    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);
    !(has_neg && has_pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_returns_one_tri() {
        let pts = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let tris = triangulate(&pts);
        assert_eq!(tris.len(), 1);
    }

    #[test]
    fn square_returns_two_tris() {
        let pts = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let tris = triangulate(&pts);
        assert_eq!(tris.len(), 2);
    }

    #[test]
    fn pentagon_returns_three_tris() {
        let pts = [
            [0.0, 0.0],
            [2.0, 0.0],
            [3.0, 1.5],
            [1.0, 3.0],
            [-1.0, 1.5],
        ];
        let tris = triangulate(&pts);
        assert_eq!(tris.len(), 3);
    }

    #[test]
    fn clockwise_winding_handled() {
        // CW square — should still triangulate correctly
        let pts = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
        let tris = triangulate(&pts);
        assert_eq!(tris.len(), 2);
    }

    #[test]
    fn all_indices_in_bounds() {
        let pts = [[0.0, 0.0], [5.0, 0.0], [5.0, 3.0], [0.0, 3.0]];
        let tris = triangulate(&pts);
        for tri in &tris {
            for &idx in tri {
                assert!(idx < pts.len());
            }
        }
    }

    #[test]
    fn l_shape_concave() {
        // L-shape (concave polygon)
        let pts = [
            [0.0, 0.0],
            [2.0, 0.0],
            [2.0, 1.0],
            [1.0, 1.0],
            [1.0, 2.0],
            [0.0, 2.0],
        ];
        let tris = triangulate(&pts);
        assert_eq!(tris.len(), 4); // 6 vertices → 4 triangles
    }
}
