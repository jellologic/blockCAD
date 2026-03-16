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
        let p = points[idx];
        // Skip vertices that coincide with triangle vertices (bridge duplicates)
        if (p[0] - a[0]).abs() < 1e-12 && (p[1] - a[1]).abs() < 1e-12 {
            continue;
        }
        if (p[0] - b[0]).abs() < 1e-12 && (p[1] - b[1]).abs() < 1e-12 {
            continue;
        }
        if (p[0] - c[0]).abs() < 1e-12 && (p[1] - c[1]).abs() < 1e-12 {
            continue;
        }
        if point_in_triangle(p, a, b, c) {
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

/// Triangulate a polygon with holes using the bridge technique.
/// `outer` is the outer boundary (CCW), `holes` is a list of hole boundaries (CW).
/// Returns triangle indices into a combined vertex array: [outer..., hole0..., hole1..., ...]
pub fn triangulate_with_holes(outer: &[[f64; 2]], holes: &[Vec<[f64; 2]>]) -> Vec<[usize; 3]> {
    if holes.is_empty() {
        return triangulate(outer);
    }

    // Build combined polygon with bridges
    // Track (global_index, point) pairs
    let mut combined: Vec<(usize, [f64; 2])> = outer.iter().enumerate().map(|(i, &p)| (i, p)).collect();

    // Sort holes by rightmost x (descending) for consistent bridge building
    let mut hole_data: Vec<(usize, &Vec<[f64; 2]>)> = holes.iter().enumerate().collect();
    hole_data.sort_by(|a, b| {
        let max_x_a = a.1.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
        let max_x_b = b.1.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
        max_x_b.partial_cmp(&max_x_a).unwrap()
    });

    for (hole_idx, hole) in hole_data {
        if hole.len() < 3 {
            continue;
        }

        let base_index = outer.len() + holes[..hole_idx].iter().map(|h| h.len()).sum::<usize>();

        // Find rightmost vertex in hole
        let (m_local, _) = hole
            .iter()
            .enumerate()
            .max_by(|a, b| a.1[0].partial_cmp(&b.1[0]).unwrap())
            .unwrap();
        let m_point = hole[m_local];
        let m_global = base_index + m_local;

        // Find the closest vertex in the combined polygon to the hole's rightmost point
        let (bridge_pos, _) = combined
            .iter()
            .enumerate()
            .min_by(|a, b| {
                let da = (a.1 .1[0] - m_point[0]).powi(2) + (a.1 .1[1] - m_point[1]).powi(2);
                let db = (b.1 .1[0] - m_point[0]).powi(2) + (b.1 .1[1] - m_point[1]).powi(2);
                da.partial_cmp(&db).unwrap()
            })
            .unwrap();

        let bridge_vertex = combined[bridge_pos];

        // Build hole vertices in order starting from m_local
        let mut hole_verts: Vec<(usize, [f64; 2])> = Vec::new();
        for k in 0..hole.len() {
            let idx = (m_local + k) % hole.len();
            hole_verts.push((base_index + idx, hole[idx]));
        }

        // Insert into combined: at bridge_pos+1, insert:
        // [hole starting from M, back to M, bridge point again]
        let mut insertion: Vec<(usize, [f64; 2])> = Vec::new();
        insertion.extend_from_slice(&hole_verts);
        insertion.push((m_global, m_point)); // back to M
        insertion.push(bridge_vertex); // bridge back

        combined.splice(bridge_pos + 1..bridge_pos + 1, insertion);
    }

    // Now triangulate the combined simple polygon
    let points_2d: Vec<[f64; 2]> = combined.iter().map(|(_, p)| *p).collect();
    let local_tris = triangulate(&points_2d);

    // Map local triangle indices back to global indices, filtering degenerate triangles
    // (where two global indices are the same due to bridge vertices)
    local_tris
        .iter()
        .map(|tri| [combined[tri[0]].0, combined[tri[1]].0, combined[tri[2]].0])
        .filter(|tri| tri[0] != tri[1] && tri[1] != tri[2] && tri[0] != tri[2])
        .collect()
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

    #[test]
    fn square_with_hole() {
        let outer = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let hole = vec![[3.0, 3.0], [3.0, 7.0], [7.0, 7.0], [7.0, 3.0]]; // CW
        let tris = triangulate_with_holes(&outer, &[hole]);
        // Should have triangles covering the frame area
        assert!(
            tris.len() >= 4,
            "Should have at least 4 triangles for frame, got {}",
            tris.len()
        );
        // All indices should be in bounds
        for tri in &tris {
            for &idx in tri {
                assert!(idx < 8, "Index {} out of bounds", idx);
            }
        }
    }

    #[test]
    fn no_holes_delegates_to_triangulate() {
        let outer = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let tris = triangulate_with_holes(&outer, &[]);
        assert_eq!(tris.len(), 2);
    }
}
