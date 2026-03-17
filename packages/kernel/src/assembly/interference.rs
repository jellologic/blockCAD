//! Assembly interference detection — checks for overlapping components.
//!
//! Phase 1: Bounding box broad-phase pre-filtering.
//! Phase 2: Triangle-level narrow-phase intersection testing for true interference.

use crate::error::KernelResult;
use crate::tessellation::mesh::TriMesh;
use crate::topology::BRep;

/// An interference between two components (bbox-only, legacy).
#[derive(Debug, Clone)]
pub struct Interference {
    pub component_a: String,
    pub component_b: String,
    /// Estimated overlap distance (negative = penetration depth).
    pub overlap_distance: f64,
}

/// Detailed interference detection result with triangle-level accuracy.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InterferenceResult {
    pub pairs: Vec<InterferencePair>,
}

/// A single interference pair with contact geometry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InterferencePair {
    pub component_a: String,
    pub component_b: String,
    /// Estimated overlap volume (approximate, from bbox intersection).
    pub overlap_volume_estimate: f64,
    /// Contact points found at triangle-triangle intersections.
    pub contact_points: Vec<[f64; 3]>,
}

/// Check for interference between positioned component BReps (legacy bbox-only).
pub fn check_interference(
    components: &[(String, BRep)],
) -> KernelResult<Vec<Interference>> {
    let mut results = Vec::new();

    let bboxes: Vec<(String, [f64; 3], [f64; 3])> = components
        .iter()
        .map(|(id, brep)| {
            let (min, max) = compute_brep_bbox(brep);
            (id.clone(), min, max)
        })
        .collect();

    for i in 0..bboxes.len() {
        for j in (i + 1)..bboxes.len() {
            let (ref id_a, min_a, max_a) = bboxes[i];
            let (ref id_b, min_b, max_b) = bboxes[j];

            if bbox_overlaps(&min_a, &max_a, &min_b, &max_b) {
                let overlap = bbox_overlap_depth(&min_a, &max_a, &min_b, &max_b);
                results.push(Interference {
                    component_a: id_a.clone(),
                    component_b: id_b.clone(),
                    overlap_distance: overlap,
                });
            }
        }
    }

    Ok(results)
}

/// Enhanced interference detection with triangle-level narrow-phase.
///
/// Each component is provided as `(id, mesh, transform_4x4_col_major)`.
/// The mesh positions are assumed to already be in world space (pre-transformed).
///
/// Phase 1: Axis-aligned bounding box broad-phase.
/// Phase 2: Triangle-triangle intersection tests for overlapping pairs.
pub fn check_interference_detailed(
    components: &[(String, &TriMesh, [f64; 16])],
) -> InterferenceResult {
    let mut pairs = Vec::new();

    let bboxes: Vec<([f64; 3], [f64; 3])> = components
        .iter()
        .map(|(_, mesh, _)| compute_mesh_bbox(mesh))
        .collect();

    for i in 0..components.len() {
        for j in (i + 1)..components.len() {
            let (min_a, max_a) = &bboxes[i];
            let (min_b, max_b) = &bboxes[j];

            if !bbox_overlaps(min_a, max_a, min_b, max_b) {
                continue;
            }

            let (ref id_a, mesh_a, _) = components[i];
            let (ref id_b, mesh_b, _) = components[j];

            let contact_points = triangle_triangle_intersections(mesh_a, mesh_b);

            if !contact_points.is_empty() {
                let overlap_volume = bbox_overlap_volume(min_a, max_a, min_b, max_b);

                pairs.push(InterferencePair {
                    component_a: id_a.clone(),
                    component_b: id_b.clone(),
                    overlap_volume_estimate: overlap_volume,
                    contact_points,
                });
            }
        }
    }

    InterferenceResult { pairs }
}

// =============================================================================
// Bounding box helpers
// =============================================================================

fn compute_brep_bbox(brep: &BRep) -> ([f64; 3], [f64; 3]) {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];

    for (_, vertex) in brep.vertices.iter() {
        let p = vertex.point;
        let coords = [p.x, p.y, p.z];
        for i in 0..3 {
            if coords[i] < min[i] { min[i] = coords[i]; }
            if coords[i] > max[i] { max[i] = coords[i]; }
        }
    }

    if min[0].is_infinite() {
        return ([0.0; 3], [0.0; 3]);
    }

    (min, max)
}

fn compute_mesh_bbox(mesh: &TriMesh) -> ([f64; 3], [f64; 3]) {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];

    for v in 0..mesh.vertex_count() {
        for axis in 0..3 {
            let val = mesh.positions[v * 3 + axis] as f64;
            if val < min[axis] { min[axis] = val; }
            if val > max[axis] { max[axis] = val; }
        }
    }

    if min[0].is_infinite() {
        return ([0.0; 3], [0.0; 3]);
    }

    (min, max)
}

fn bbox_overlaps(min_a: &[f64; 3], max_a: &[f64; 3], min_b: &[f64; 3], max_b: &[f64; 3]) -> bool {
    for i in 0..3 {
        if max_a[i] < min_b[i] || max_b[i] < min_a[i] {
            return false;
        }
    }
    true
}

fn bbox_overlap_depth(min_a: &[f64; 3], max_a: &[f64; 3], min_b: &[f64; 3], max_b: &[f64; 3]) -> f64 {
    let mut min_overlap = f64::INFINITY;
    for i in 0..3 {
        let overlap = (max_a[i].min(max_b[i])) - (min_a[i].max(min_b[i]));
        if overlap < min_overlap {
            min_overlap = overlap;
        }
    }
    min_overlap
}

fn bbox_overlap_volume(min_a: &[f64; 3], max_a: &[f64; 3], min_b: &[f64; 3], max_b: &[f64; 3]) -> f64 {
    let mut volume = 1.0;
    for i in 0..3 {
        let overlap = (max_a[i].min(max_b[i])) - (min_a[i].max(min_b[i]));
        if overlap <= 0.0 {
            return 0.0;
        }
        volume *= overlap;
    }
    volume
}

// =============================================================================
// Triangle-triangle intersection (Moller's method)
// =============================================================================

type Tri = [[f64; 3]; 3];

fn get_triangle(mesh: &TriMesh, tri_idx: usize) -> Tri {
    let base = tri_idx * 3;
    let i0 = mesh.indices[base] as usize;
    let i1 = mesh.indices[base + 1] as usize;
    let i2 = mesh.indices[base + 2] as usize;
    [
        [mesh.positions[i0 * 3] as f64, mesh.positions[i0 * 3 + 1] as f64, mesh.positions[i0 * 3 + 2] as f64],
        [mesh.positions[i1 * 3] as f64, mesh.positions[i1 * 3 + 1] as f64, mesh.positions[i1 * 3 + 2] as f64],
        [mesh.positions[i2 * 3] as f64, mesh.positions[i2 * 3 + 1] as f64, mesh.positions[i2 * 3 + 2] as f64],
    ]
}

fn sub(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn add(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: &[f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn tri_bbox(tri: &Tri) -> ([f64; 3], [f64; 3]) {
    let mut min = tri[0];
    let mut max = tri[0];
    for v in &tri[1..] {
        for i in 0..3 {
            if v[i] < min[i] { min[i] = v[i]; }
            if v[i] > max[i] { max[i] = v[i]; }
        }
    }
    (min, max)
}

fn triangle_triangle_test(t1: &Tri, t2: &Tri) -> Option<[f64; 3]> {
    let (min1, max1) = tri_bbox(t1);
    let (min2, max2) = tri_bbox(t2);
    for i in 0..3 {
        if max1[i] < min2[i] - 1e-10 || max2[i] < min1[i] - 1e-10 {
            return None;
        }
    }

    let e1 = [sub(&t1[1], &t1[0]), sub(&t1[2], &t1[1]), sub(&t1[0], &t1[2])];
    let e2 = [sub(&t2[1], &t2[0]), sub(&t2[2], &t2[1]), sub(&t2[0], &t2[2])];

    let n1 = cross(&e1[0], &sub(&t1[2], &t1[0]));
    let n2 = cross(&e2[0], &sub(&t2[2], &t2[0]));

    let d1 = dot(&n1, &t1[0]);
    let dist2: [f64; 3] = [
        dot(&n1, &t2[0]) - d1,
        dot(&n1, &t2[1]) - d1,
        dot(&n1, &t2[2]) - d1,
    ];

    if dist2[0] > 1e-10 && dist2[1] > 1e-10 && dist2[2] > 1e-10 { return None; }
    if dist2[0] < -1e-10 && dist2[1] < -1e-10 && dist2[2] < -1e-10 { return None; }

    let d2 = dot(&n2, &t2[0]);
    let dist1: [f64; 3] = [
        dot(&n2, &t1[0]) - d2,
        dot(&n2, &t1[1]) - d2,
        dot(&n2, &t1[2]) - d2,
    ];

    if dist1[0] > 1e-10 && dist1[1] > 1e-10 && dist1[2] > 1e-10 { return None; }
    if dist1[0] < -1e-10 && dist1[1] < -1e-10 && dist1[2] < -1e-10 { return None; }

    let n1_len_sq = dot(&n1, &n1);
    if n1_len_sq < 1e-20 { return None; }

    let coplanar = dist2[0].abs() < 1e-10 && dist2[1].abs() < 1e-10 && dist2[2].abs() < 1e-10;
    if coplanar {
        return coplanar_tri_tri_test(t1, t2, &n1);
    }

    let line_dir = cross(&n1, &n2);
    let line_dir_len_sq = dot(&line_dir, &line_dir);
    if line_dir_len_sq < 1e-20 { return None; }

    let max_axis = if line_dir[0].abs() >= line_dir[1].abs() && line_dir[0].abs() >= line_dir[2].abs() {
        0
    } else if line_dir[1].abs() >= line_dir[2].abs() {
        1
    } else {
        2
    };

    let proj1: [f64; 3] = [t1[0][max_axis], t1[1][max_axis], t1[2][max_axis]];
    let proj2: [f64; 3] = [t2[0][max_axis], t2[1][max_axis], t2[2][max_axis]];

    let interval1 = compute_interval(&proj1, &dist1);
    let interval2 = compute_interval(&proj2, &dist2);

    let (int1_min, int1_max) = interval1?;
    let (int2_min, int2_max) = interval2?;

    if int1_max < int2_min - 1e-10 || int2_max < int1_min - 1e-10 { return None; }

    let c1 = scale(&add(&add(&t1[0], &t1[1]), &t1[2]), 1.0 / 3.0);
    let c2 = scale(&add(&add(&t2[0], &t2[1]), &t2[2]), 1.0 / 3.0);
    Some(scale(&add(&c1, &c2), 0.5))
}

fn compute_interval(proj: &[f64; 3], dist: &[f64; 3]) -> Option<(f64, f64)> {
    let mut on_line = Vec::new();

    for i in 0..3 {
        let j = (i + 1) % 3;
        if (dist[i] > 1e-10 && dist[j] < -1e-10) || (dist[i] < -1e-10 && dist[j] > 1e-10) {
            let t = dist[i] / (dist[i] - dist[j]);
            let p = proj[i] + t * (proj[j] - proj[i]);
            on_line.push(p);
        } else if dist[i].abs() <= 1e-10 {
            on_line.push(proj[i]);
        }
    }

    if on_line.len() < 2 {
        if on_line.len() == 1 {
            return Some((on_line[0], on_line[0]));
        }
        return None;
    }

    let mut min_val = on_line[0];
    let mut max_val = on_line[0];
    for &v in &on_line[1..] {
        if v < min_val { min_val = v; }
        if v > max_val { max_val = v; }
    }

    Some((min_val, max_val))
}

fn coplanar_tri_tri_test(t1: &Tri, t2: &Tri, normal: &[f64; 3]) -> Option<[f64; 3]> {
    let abs_n = [normal[0].abs(), normal[1].abs(), normal[2].abs()];
    let (ax1, ax2) = if abs_n[0] >= abs_n[1] && abs_n[0] >= abs_n[2] {
        (1, 2)
    } else if abs_n[1] >= abs_n[2] {
        (0, 2)
    } else {
        (0, 1)
    };

    let p1: [[f64; 2]; 3] = [
        [t1[0][ax1], t1[0][ax2]],
        [t1[1][ax1], t1[1][ax2]],
        [t1[2][ax1], t1[2][ax2]],
    ];
    let p2: [[f64; 2]; 3] = [
        [t2[0][ax1], t2[0][ax2]],
        [t2[1][ax1], t2[1][ax2]],
        [t2[2][ax1], t2[2][ax2]],
    ];

    let edges1 = [
        [p1[1][0] - p1[0][0], p1[1][1] - p1[0][1]],
        [p1[2][0] - p1[1][0], p1[2][1] - p1[1][1]],
        [p1[0][0] - p1[2][0], p1[0][1] - p1[2][1]],
    ];
    let edges2 = [
        [p2[1][0] - p2[0][0], p2[1][1] - p2[0][1]],
        [p2[2][0] - p2[1][0], p2[2][1] - p2[1][1]],
        [p2[0][0] - p2[2][0], p2[0][1] - p2[2][1]],
    ];

    for edge in edges1.iter().chain(edges2.iter()) {
        let axis = [-edge[1], edge[0]];
        let len_sq = axis[0] * axis[0] + axis[1] * axis[1];
        if len_sq < 1e-20 { continue; }

        let (min1, max1) = project_2d(&p1, &axis);
        let (min2, max2) = project_2d(&p2, &axis);

        if max1 < min2 - 1e-10 || max2 < min1 - 1e-10 {
            return None;
        }
    }

    let c1 = scale(&add(&add(&t1[0], &t1[1]), &t1[2]), 1.0 / 3.0);
    let c2 = scale(&add(&add(&t2[0], &t2[1]), &t2[2]), 1.0 / 3.0);
    Some(scale(&add(&c1, &c2), 0.5))
}

fn project_2d(tri: &[[f64; 2]; 3], axis: &[f64; 2]) -> (f64, f64) {
    let d0 = tri[0][0] * axis[0] + tri[0][1] * axis[1];
    let d1 = tri[1][0] * axis[0] + tri[1][1] * axis[1];
    let d2 = tri[2][0] * axis[0] + tri[2][1] * axis[1];
    let min = d0.min(d1).min(d2);
    let max = d0.max(d1).max(d2);
    (min, max)
}

fn triangle_triangle_intersections(mesh_a: &TriMesh, mesh_b: &TriMesh) -> Vec<[f64; 3]> {
    let tri_count_a = mesh_a.triangle_count();
    let tri_count_b = mesh_b.triangle_count();

    if tri_count_a == 0 || tri_count_b == 0 {
        return Vec::new();
    }

    let bboxes_b: Vec<([f64; 3], [f64; 3])> = (0..tri_count_b)
        .map(|t| tri_bbox(&get_triangle(mesh_b, t)))
        .collect();

    let mut contact_points = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let quant_scale = 1e4;

    for ta in 0..tri_count_a {
        let tri_a = get_triangle(mesh_a, ta);
        let (min_a, max_a) = tri_bbox(&tri_a);

        for tb in 0..tri_count_b {
            let (ref min_b, ref max_b) = bboxes_b[tb];

            let mut separated = false;
            for i in 0..3 {
                if max_a[i] < min_b[i] - 1e-10 || max_b[i] < min_a[i] - 1e-10 {
                    separated = true;
                    break;
                }
            }
            if separated { continue; }

            let tri_b = get_triangle(mesh_b, tb);

            if let Some(contact) = triangle_triangle_test(&tri_a, &tri_b) {
                let key = [
                    (contact[0] * quant_scale).round() as i64,
                    (contact[1] * quant_scale).round() as i64,
                    (contact[2] * quant_scale).round() as i64,
                ];
                if seen.insert(key) {
                    contact_points.push(contact);
                }
            }
        }
    }

    contact_points
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;
    use crate::geometry::transform;
    use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};
    use crate::tessellation::{tessellate_brep, TessellationParams};
    use crate::geometry::{Pt3, Vec3};

    fn make_box_at(x: f64, y: f64, z: f64, w: f64, h: f64, d: f64) -> BRep {
        let base = build_box_brep(w, h, d).unwrap();
        if x == 0.0 && y == 0.0 && z == 0.0 {
            return base;
        }
        let t = transform::translation(x, y, z);
        let polygons = extract_face_polygons(&base).unwrap();
        let transformed: Vec<_> = polygons.iter().map(|(pts, n)| {
            let new_pts: Vec<_> = pts.iter().map(|p| transform::transform_point(&t, p)).collect();
            let new_n = transform::transform_normal(&t, n);
            (new_pts, new_n)
        }).collect();
        rebuild_brep_from_faces(&transformed).unwrap()
    }

    fn make_box_mesh_at(x: f64, y: f64, z: f64, w: f64, h: f64, d: f64) -> TriMesh {
        let brep = make_box_at(x, y, z, w, h, d);
        let params = TessellationParams::default();
        tessellate_brep(&brep, &params).unwrap()
    }

    fn identity_transform() -> [f64; 16] {
        [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]
    }

    #[test]
    fn no_components_no_interference() {
        let result = check_interference(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn non_overlapping_boxes() {
        let a = make_box_at(0.0, 0.0, 0.0, 5.0, 5.0, 5.0);
        let b = make_box_at(10.0, 0.0, 0.0, 5.0, 5.0, 5.0);
        let result = check_interference(&[
            ("comp1".into(), a),
            ("comp2".into(), b),
        ]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn overlapping_boxes_detected() {
        let a = make_box_at(0.0, 0.0, 0.0, 5.0, 5.0, 5.0);
        let b = make_box_at(3.0, 0.0, 0.0, 5.0, 5.0, 5.0);
        let result = check_interference(&[
            ("comp1".into(), a),
            ("comp2".into(), b),
        ]).unwrap();
        assert_eq!(result.len(), 1);
        assert!((result[0].overlap_distance - 2.0).abs() < 0.1);
    }

    #[test]
    fn detailed_two_overlapping_boxes_detected() {
        let mesh_a = make_box_mesh_at(0.0, 0.0, 0.0, 5.0, 5.0, 5.0);
        let mesh_b = make_box_mesh_at(3.0, 0.0, 0.0, 5.0, 5.0, 5.0);

        let components = vec![
            ("comp1".to_string(), &mesh_a, identity_transform()),
            ("comp2".to_string(), &mesh_b, identity_transform()),
        ];

        let result = check_interference_detailed(&components);
        assert_eq!(result.pairs.len(), 1);
        assert!(!result.pairs[0].contact_points.is_empty());
    }

    #[test]
    fn detailed_two_separated_boxes_not_flagged() {
        let mesh_a = make_box_mesh_at(0.0, 0.0, 0.0, 5.0, 5.0, 5.0);
        let mesh_b = make_box_mesh_at(10.0, 0.0, 0.0, 5.0, 5.0, 5.0);

        let components = vec![
            ("comp1".to_string(), &mesh_a, identity_transform()),
            ("comp2".to_string(), &mesh_b, identity_transform()),
        ];

        let result = check_interference_detailed(&components);
        assert!(result.pairs.is_empty());
    }

    #[test]
    fn detailed_empty_components_no_interference() {
        let result = check_interference_detailed(&[]);
        assert!(result.pairs.is_empty());
    }
}
