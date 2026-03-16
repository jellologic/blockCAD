use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::topology::BRep;
use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};

use super::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DraftParams {
    pub face_indices: Vec<u32>,
    pub pull_direction: Vec3,
    /// Draft angle in radians
    pub angle: f64,
}

#[derive(Debug)]
pub struct DraftOp;

impl Operation for DraftOp {
    type Params = DraftParams;

    fn execute(&self, params: &Self::Params, input: &BRep) -> KernelResult<BRep> {
        draft_faces(input, params)
    }

    fn name(&self) -> &'static str {
        "Draft"
    }
}

/// Apply draft (taper) to specified faces of a BRep.
///
/// For each face in `face_indices`, vertices are displaced laterally
/// proportional to their height along `pull_direction`, creating a taper
/// at the specified angle.
///
/// This operates at the global vertex level: each unique vertex position that
/// belongs to a drafted face is displaced once, and that displacement is
/// propagated consistently to all faces sharing that vertex, maintaining
/// watertight topology.
pub fn draft_faces(brep: &BRep, params: &DraftParams) -> KernelResult<BRep> {
    if params.angle.abs() < 1e-12 {
        // Zero draft angle — return unchanged geometry
        let faces = extract_face_polygons(brep)?;
        return rebuild_brep_from_faces(&faces);
    }

    let pull = params.pull_direction.normalize();
    let tan_angle = params.angle.tan();

    let mut faces = extract_face_polygons(brep)?;

    // Validate face indices first
    for &face_idx in &params.face_indices {
        let fi = face_idx as usize;
        if fi >= faces.len() {
            return Err(KernelError::InvalidParameter {
                param: "face_indices".into(),
                value: format!("Face index {} out of range (0..{})", fi, faces.len()),
            });
        }
    }

    // Compute the global neutral height (minimum along pull across all drafted faces)
    let drafted_set: std::collections::HashSet<usize> =
        params.face_indices.iter().map(|&i| i as usize).collect();
    let global_min_h = params.face_indices.iter()
        .flat_map(|&fi| faces[fi as usize].0.iter())
        .map(|p| Vec3::new(p.x, p.y, p.z).dot(&pull))
        .fold(f64::INFINITY, f64::min);

    // Compute the body centroid (across all faces) for consistent outward direction
    let body_centroid = {
        let mut total = Vec3::new(0.0, 0.0, 0.0);
        let mut count = 0usize;
        for (pts, _) in faces.iter() {
            for p in pts {
                total = total + Vec3::new(p.x, p.y, p.z);
                count += 1;
            }
        }
        if count > 0 { total / (count as f64) } else { total }
    };

    // Build a displacement map: for each unique vertex on drafted faces,
    // compute the displaced position once.
    let eps = 1e-9;
    let mut displacement_map: Vec<(Pt3, Pt3)> = Vec::new();

    // Helper: check if a position already has a displacement entry
    let find_displacement = |map: &[(Pt3, Pt3)], p: &Pt3| -> Option<Pt3> {
        for (old, new_pt) in map {
            let dx = p.x - old.x;
            let dy = p.y - old.y;
            let dz = p.z - old.z;
            if dx * dx + dy * dy + dz * dz < eps {
                return Some(*new_pt);
            }
        }
        None
    };

    // First pass: compute displacements for all vertices on drafted faces
    for &face_idx in &params.face_indices {
        let fi = face_idx as usize;
        let points = &faces[fi].0;
        for p in points.iter() {
            // Skip if already computed
            if find_displacement(&displacement_map, p).is_some() {
                continue;
            }

            let pv = Vec3::new(p.x, p.y, p.z);
            let h = pv.dot(&pull) - global_min_h;
            if h.abs() < 1e-12 { continue; } // Neutral edge vertex

            // Lateral direction: away from body centroid, perpendicular to pull
            let to_centroid = body_centroid - pv;
            let lateral = to_centroid - pull * to_centroid.dot(&pull);
            let lat_len = lateral.norm();
            if lat_len < 1e-12 { continue; }
            let lateral_dir = -lateral / lat_len; // Outward from body centroid

            let disp = lateral_dir * h * tan_angle;
            let new_pt = Pt3::new(p.x + disp.x, p.y + disp.y, p.z + disp.z);
            displacement_map.push((*p, new_pt));
        }
    }

    // Second pass: apply displacements to ALL faces (drafted and non-drafted)
    for (fi, (ref mut points, ref mut normal)) in faces.iter_mut().enumerate() {
        let mut modified = false;
        for p in points.iter_mut() {
            if let Some(new_pt) = find_displacement(&displacement_map, p) {
                *p = new_pt;
                modified = true;
            }
        }
        // Recompute normal if any vertices were updated
        if modified && points.len() >= 3 {
            let e1 = points[1] - points[0];
            let e2 = points[2] - points[0];
            let new_normal = e1.cross(&e2);
            let len = new_normal.norm();
            if len > 1e-12 {
                *normal = new_normal / len;
            }
        }
    }

    rebuild_brep_from_faces(&faces)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;
    use crate::topology::body::Body;

    #[test]
    fn draft_single_face() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = DraftParams {
            face_indices: vec![2], // Front face
            pull_direction: Vec3::new(0.0, 0.0, 1.0),
            angle: 0.1, // ~5.7 degrees
        };
        let result = draft_faces(&brep, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
        assert_eq!(result.faces.len(), 6);
    }

    #[test]
    fn draft_zero_angle_unchanged() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = DraftParams {
            face_indices: vec![0, 1],
            pull_direction: Vec3::new(0.0, 0.0, 1.0),
            angle: 0.0,
        };
        let result = draft_faces(&brep, &params).unwrap();
        assert_eq!(result.faces.len(), 6);
    }

    #[test]
    fn draft_invalid_face_index() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = DraftParams {
            face_indices: vec![99],
            pull_direction: Vec3::new(0.0, 0.0, 1.0),
            angle: 0.1,
        };
        assert!(draft_faces(&brep, &params).is_err());
    }
}
