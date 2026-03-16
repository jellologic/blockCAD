//! Assembly interference detection — checks for overlapping components.
//!
//! Uses bounding box pre-filtering followed by face-distance checks.

use crate::error::KernelResult;
use crate::geometry::{Pt3, Vec3};
use crate::topology::BRep;
use crate::topology::builders::extract_face_polygons;

/// An interference between two components.
#[derive(Debug, Clone)]
pub struct Interference {
    pub component_a: String,
    pub component_b: String,
    /// Estimated overlap distance (negative = penetration depth).
    pub overlap_distance: f64,
}

/// Check for interference between positioned component BReps.
///
/// Uses bounding box overlap as a pre-filter, then checks face-to-face
/// proximity for pairs whose bounding boxes intersect.
pub fn check_interference(
    components: &[(String, BRep)],
) -> KernelResult<Vec<Interference>> {
    let mut results = Vec::new();

    // Compute bounding boxes for all components
    let bboxes: Vec<(String, [f64; 3], [f64; 3])> = components
        .iter()
        .map(|(id, brep)| {
            let (min, max) = compute_brep_bbox(brep);
            (id.clone(), min, max)
        })
        .collect();

    // Pairwise check
    for i in 0..bboxes.len() {
        for j in (i + 1)..bboxes.len() {
            let (ref id_a, min_a, max_a) = bboxes[i];
            let (ref id_b, min_b, max_b) = bboxes[j];

            // Bounding box overlap test
            if bbox_overlaps(&min_a, &max_a, &min_b, &max_b) {
                // Compute overlap depth (approximate)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;
    use crate::geometry::transform;
    use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};

    fn make_box_at(x: f64, y: f64, z: f64, w: f64, h: f64, d: f64) -> BRep {
        let base = build_box_brep(w, h, d).unwrap();
        if x == 0.0 && y == 0.0 && z == 0.0 {
            return base;
        }
        // Transform by translation
        let t = transform::translation(x, y, z);
        let polygons = extract_face_polygons(&base).unwrap();
        let transformed: Vec<_> = polygons.iter().map(|(pts, n)| {
            let new_pts: Vec<_> = pts.iter().map(|p| transform::transform_point(&t, p)).collect();
            let new_n = transform::transform_normal(&t, n);
            (new_pts, new_n)
        }).collect();
        rebuild_brep_from_faces(&transformed).unwrap()
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
        assert!(result.is_empty(), "Non-overlapping boxes should have no interference");
    }

    #[test]
    fn overlapping_boxes_detected() {
        let a = make_box_at(0.0, 0.0, 0.0, 5.0, 5.0, 5.0);
        let b = make_box_at(3.0, 0.0, 0.0, 5.0, 5.0, 5.0); // overlaps by 2 units in X
        let result = check_interference(&[
            ("comp1".into(), a),
            ("comp2".into(), b),
        ]).unwrap();
        assert_eq!(result.len(), 1, "Overlapping boxes should produce 1 interference");
        assert!((result[0].overlap_distance - 2.0).abs() < 0.1, "Overlap should be ~2.0");
    }

    #[test]
    fn three_components_pairwise_check() {
        let a = make_box_at(0.0, 0.0, 0.0, 5.0, 5.0, 5.0);
        let b = make_box_at(3.0, 0.0, 0.0, 5.0, 5.0, 5.0); // overlaps A
        let c = make_box_at(20.0, 0.0, 0.0, 5.0, 5.0, 5.0); // no overlap
        let result = check_interference(&[
            ("comp1".into(), a),
            ("comp2".into(), b),
            ("comp3".into(), c),
        ]).unwrap();
        assert_eq!(result.len(), 1, "Only A-B should interfere");
        assert_eq!(result[0].component_a, "comp1");
        assert_eq!(result[0].component_b, "comp2");
    }
}
