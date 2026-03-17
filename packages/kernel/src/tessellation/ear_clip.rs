//! Optimized ear-clipping triangulation for 2D polygons.
//!
//! Works correctly for simple (non-self-intersecting) polygons,
//! both convex and concave. Uses pre-computed convexity and an ear
//! candidate list for improved average-case performance.

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
        // Skip degenerate triangle early
        let area = triangle_area_2x(points[0], points[1], points[2]);
        if area.abs() < 1e-20 {
            return vec![];
        }
        return vec![[0, 1, 2]];
    }

    // Ensure CCW winding
    let area = signed_area(points);
    let indices: Vec<usize> = if area >= 0.0 {
        (0..n).collect()
    } else {
        (0..n).rev().collect()
    };

    triangulate_indexed(points, indices)
}

/// Core triangulation on an index list — used by both `triangulate` and
/// `triangulate_with_holes` (after bridge construction).
fn triangulate_indexed(points: &[[f64; 2]], mut indices: Vec<usize>) -> Vec<[usize; 3]> {
    let mut remaining = indices.len();
    if remaining < 3 {
        return vec![];
    }

    let mut triangles = Vec::with_capacity(remaining - 2);

    // Pre-compute convexity for each vertex in the polygon
    let mut is_convex = vec![false; remaining];
    for i in 0..remaining {
        let prev = if i == 0 { remaining - 1 } else { i - 1 };
        let next = (i + 1) % remaining;
        is_convex[i] = cross_2d(
            points[indices[prev]],
            points[indices[i]],
            points[indices[next]],
        ) > 0.0;
    }

    // Build initial ear candidate list — only convex vertices can be ears
    let mut is_ear_flag = vec![false; remaining];
    for i in 0..remaining {
        if is_convex[i] {
            is_ear_flag[i] = check_ear(points, &indices, &is_convex, remaining, i);
        }
    }

    let max_attempts = remaining * remaining; // safety limit
    let mut attempts = 0;

    while remaining > 3 && attempts < max_attempts {
        // Find an ear to clip
        let mut found = None;
        for i in 0..remaining {
            if is_ear_flag[i] {
                found = Some(i);
                break;
            }
        }

        let ear_idx = match found {
            Some(i) => i,
            None => {
                // Fallback: try any convex vertex (degenerate polygon)
                attempts += remaining;
                // Force-find the best ear candidate by relaxing constraints
                let mut best = None;
                let mut best_area = -1.0;
                for i in 0..remaining {
                    let prev = if i == 0 { remaining - 1 } else { i - 1 };
                    let next = (i + 1) % remaining;
                    let a2x = triangle_area_2x(
                        points[indices[prev]],
                        points[indices[i]],
                        points[indices[next]],
                    );
                    if a2x > best_area {
                        best_area = a2x;
                        best = Some(i);
                    }
                }
                match best {
                    Some(i) => i,
                    None => break,
                }
            }
        };

        attempts += 1;

        let prev_idx = if ear_idx == 0 { remaining - 1 } else { ear_idx - 1 };
        let next_idx = (ear_idx + 1) % remaining;

        let prev = indices[prev_idx];
        let curr = indices[ear_idx];
        let next = indices[next_idx];

        // Skip degenerate triangles
        let area_2x = triangle_area_2x(points[prev], points[curr], points[next]);
        if area_2x.abs() > 1e-20 {
            triangles.push([prev, curr, next]);
        }

        // Remove the ear tip
        indices.remove(ear_idx);
        is_convex.remove(ear_idx);
        is_ear_flag.remove(ear_idx);
        remaining -= 1;

        if remaining <= 2 {
            break;
        }

        // Update only the two neighbors of the removed ear
        // Adjust neighbor positions after removal
        let new_prev = if ear_idx == 0 { remaining - 1 } else { ear_idx - 1 };
        let new_next = if ear_idx >= remaining { 0 } else { ear_idx };

        for &ni in &[new_prev, new_next] {
            let p = if ni == 0 { remaining - 1 } else { ni - 1 };
            let nx = (ni + 1) % remaining;
            is_convex[ni] = cross_2d(
                points[indices[p]],
                points[indices[ni]],
                points[indices[nx]],
            ) > 0.0;
            if is_convex[ni] {
                is_ear_flag[ni] = check_ear(points, &indices, &is_convex, remaining, ni);
            } else {
                is_ear_flag[ni] = false;
            }
        }
    }

    // Handle the last triangle
    if remaining == 3 {
        let area_2x = triangle_area_2x(
            points[indices[0]],
            points[indices[1]],
            points[indices[2]],
        );
        if area_2x.abs() > 1e-20 {
            triangles.push([indices[0], indices[1], indices[2]]);
        }
    }

    triangles
}

/// Check if vertex at position `i` in the polygon is an ear.
/// Only reflex (non-convex) vertices need to be tested for containment.
fn check_ear(
    points: &[[f64; 2]],
    indices: &[usize],
    is_convex: &[bool],
    remaining: usize,
    i: usize,
) -> bool {
    let prev = if i == 0 { remaining - 1 } else { i - 1 };
    let next = (i + 1) % remaining;

    let a = points[indices[prev]];
    let b = points[indices[i]];
    let c = points[indices[next]];

    // Skip degenerate triangles early
    let area_2x = triangle_area_2x(a, b, c);
    if area_2x.abs() < 1e-20 {
        return false;
    }

    // Only test reflex vertices for containment (convex vertices cannot be inside)
    for k in 0..remaining {
        if k == prev || k == i || k == next {
            continue;
        }
        // Only reflex vertices can be inside the ear triangle
        if is_convex[k] {
            continue;
        }
        let p = points[indices[k]];
        // Skip coincident vertices (bridge duplicates)
        if is_coincident(p, a) || is_coincident(p, b) || is_coincident(p, c) {
            continue;
        }
        if point_in_triangle_edge_func(p, a, b, c) {
            return false;
        }
    }

    true
}

/// Twice the signed area of triangle (a, b, c). Positive for CCW.
#[inline(always)]
fn triangle_area_2x(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
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

/// Cross product of vectors (a->b) and (a->c).
#[inline(always)]
fn cross_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

/// Edge-function-based point-in-triangle test (3 cross products).
/// Returns true if point p is strictly inside triangle (a, b, c) assumed CCW.
#[inline]
fn point_in_triangle_edge_func(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let d1 = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
    let d2 = (c[0] - b[0]) * (p[1] - b[1]) - (c[1] - b[1]) * (p[0] - b[0]);
    let d3 = (a[0] - c[0]) * (p[1] - c[1]) - (a[1] - c[1]) * (p[0] - c[0]);
    // All same sign means inside (for CCW triangle, all >= 0)
    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);
    !(has_neg && has_pos)
}

#[inline(always)]
fn is_coincident(p: [f64; 2], q: [f64; 2]) -> bool {
    (p[0] - q[0]).abs() < 1e-12 && (p[1] - q[1]).abs() < 1e-12
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
        assert_eq!(tris.len(), 4); // 6 vertices -> 4 triangles
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

    // --- Tests for optimized ear clipping ---

    #[test]
    fn convex_hexagon() {
        // Regular hexagon (convex)
        let pts: Vec<[f64; 2]> = (0..6)
            .map(|i| {
                let angle = std::f64::consts::PI / 3.0 * i as f64;
                [angle.cos(), angle.sin()]
            })
            .collect();
        let tris = triangulate(&pts);
        assert_eq!(tris.len(), 4); // 6 vertices -> 4 triangles
        // Verify all indices in bounds
        for tri in &tris {
            for &idx in tri {
                assert!(idx < pts.len());
            }
        }
    }

    #[test]
    fn concave_arrow_shape() {
        // Arrow/chevron shape (concave)
        let pts = [
            [0.0, 0.0],
            [2.0, 1.0],
            [0.0, 2.0],
            [0.5, 1.0], // reflex vertex
        ];
        let tris = triangulate(&pts);
        assert_eq!(tris.len(), 2); // 4 vertices -> 2 triangles
    }

    #[test]
    fn degenerate_collinear_points() {
        // Three collinear points should produce no triangles
        let pts = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
        let tris = triangulate(&pts);
        assert_eq!(tris.len(), 0);
    }

    #[test]
    fn less_than_three_points() {
        assert_eq!(triangulate(&[[0.0, 0.0], [1.0, 1.0]]).len(), 0);
        assert_eq!(triangulate(&[[0.0, 0.0]]).len(), 0);
        assert_eq!(triangulate(&([] as [[f64; 2]; 0])).len(), 0);
    }

    #[test]
    fn complex_concave_star() {
        // Star shape with alternating convex/reflex vertices
        let pts: Vec<[f64; 2]> = (0..10)
            .map(|i| {
                let angle = std::f64::consts::PI / 5.0 * i as f64 - std::f64::consts::FRAC_PI_2;
                let r = if i % 2 == 0 { 2.0 } else { 1.0 };
                [r * angle.cos(), r * angle.sin()]
            })
            .collect();
        let tris = triangulate(&pts);
        assert_eq!(tris.len(), 8); // 10 vertices -> 8 triangles
    }

    #[test]
    fn optimized_matches_triangle_count_for_polygon_with_hole() {
        // Verify polygon with hole produces valid triangulation
        let outer = vec![
            [0.0, 0.0],
            [20.0, 0.0],
            [20.0, 20.0],
            [0.0, 20.0],
        ];
        let hole = vec![
            [5.0, 5.0],
            [5.0, 15.0],
            [15.0, 15.0],
            [15.0, 5.0],
        ];
        let all_pts: Vec<[f64; 2]> = outer.iter().chain(hole.iter()).copied().collect();
        let tris = triangulate_with_holes(&outer, &[hole]);
        // Total area of triangles should approximate the frame area
        let total_area: f64 = tris
            .iter()
            .map(|tri| {
                triangle_area_2x(all_pts[tri[0]], all_pts[tri[1]], all_pts[tri[2]]).abs() * 0.5
            })
            .sum();
        let expected_area = 20.0 * 20.0 - 10.0 * 10.0; // 300
        assert!(
            (total_area - expected_area).abs() < 1.0,
            "Total triangle area {} should approximate frame area {}",
            total_area,
            expected_area
        );
    }
}
