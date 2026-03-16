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
pub fn draft_faces(brep: &BRep, params: &DraftParams) -> KernelResult<BRep> {
    if params.angle.abs() < 1e-12 {
        // Zero draft angle — return unchanged geometry
        let faces = extract_face_polygons(brep)?;
        return rebuild_brep_from_faces(&faces);
    }

    let pull = params.pull_direction.normalize();
    let tan_angle = params.angle.tan();

    let mut faces = extract_face_polygons(brep)?;

    for &face_idx in &params.face_indices {
        let fi = face_idx as usize;
        if fi >= faces.len() {
            return Err(KernelError::InvalidParameter {
                param: "face_indices".into(),
                value: format!("Face index {} out of range (0..{})", fi, faces.len()),
            });
        }

        let (ref mut points, ref mut normal) = faces[fi];

        // Find the neutral edge: the vertex with minimum projection along pull direction
        let min_h = points.iter()
            .map(|p| Vec3::new(p.x, p.y, p.z).dot(&pull))
            .fold(f64::INFINITY, f64::min);

        // Compute face centroid for outward direction reference
        let centroid = {
            let n = points.len() as f64;
            let sum = points.iter().fold(Vec3::new(0.0, 0.0, 0.0), |acc, p| {
                acc + Vec3::new(p.x, p.y, p.z)
            });
            sum / n
        };

        // For each vertex: displace laterally by h * tan(angle)
        for p in points.iter_mut() {
            let pv = Vec3::new(p.x, p.y, p.z);
            let h = pv.dot(&pull) - min_h;
            if h.abs() < 1e-12 { continue; } // Neutral edge vertex — no displacement

            // Lateral direction: away from centroid, projected onto plane perpendicular to pull
            let to_centroid = centroid - pv;
            let lateral = to_centroid - pull * to_centroid.dot(&pull);
            let lat_len = lateral.norm();
            if lat_len < 1e-12 { continue; }
            let lateral_dir = -lateral / lat_len; // Outward from centroid

            let displacement = lateral_dir * h * tan_angle;
            *p = Pt3::new(p.x + displacement.x, p.y + displacement.y, p.z + displacement.z);
        }

        // Recompute normal from modified vertices
        if points.len() >= 3 {
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
