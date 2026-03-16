use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};
use crate::topology::BRep;

use crate::operations::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MirrorParams {
    pub plane_origin: Pt3,
    pub plane_normal: Vec3,
}

#[derive(Debug)]
pub struct MirrorOp;

impl Operation for MirrorOp {
    type Params = MirrorParams;

    fn execute(&self, params: &Self::Params, input: &BRep) -> KernelResult<BRep> {
        mirror_brep(input, params)
    }

    fn name(&self) -> &'static str {
        "Mirror"
    }
}

/// Compute a canonical key for a face polygon: sort vertex coordinates so that
/// two faces with the same vertices (in any order) produce the same key.
fn face_vertex_key(pts: &[Pt3], tol: f64) -> Vec<[i64; 3]> {
    let mut quantized: Vec<[i64; 3]> = pts.iter()
        .map(|p| {
            let scale = 1.0 / tol;
            [(p.x * scale).round() as i64, (p.y * scale).round() as i64, (p.z * scale).round() as i64]
        })
        .collect();
    quantized.sort();
    quantized
}

pub fn mirror_brep(brep: &BRep, params: &MirrorParams) -> KernelResult<BRep> {
    if brep.faces.is_empty() {
        return Err(KernelError::Operation {
            op: "mirror".into(),
            detail: "Cannot mirror: no existing geometry".into(),
        });
    }

    let base_faces = extract_face_polygons(brep)?;
    let plane_normal = params.plane_normal.normalize();
    let plane_origin = params.plane_origin;

    let tol = 1e-6;

    // Build original and mirrored face lists
    let mut original_faces: Vec<(Vec<Pt3>, Vec3)> = Vec::new();
    let mut mirrored_faces: Vec<(Vec<Pt3>, Vec3)> = Vec::new();

    for (pts, normal) in &base_faces {
        original_faces.push((pts.clone(), *normal));

        let mirrored_pts: Vec<Pt3> = pts.iter()
            .map(|p| {
                let d = Vec3::new(p.x - plane_origin.x, p.y - plane_origin.y, p.z - plane_origin.z);
                let dist = d.dot(&plane_normal);
                Pt3::new(
                    p.x - 2.0 * dist * plane_normal.x,
                    p.y - 2.0 * dist * plane_normal.y,
                    p.z - 2.0 * dist * plane_normal.z,
                )
            })
            .collect();

        // Reverse winding order (mirror flips handedness)
        let mirrored_pts_reversed: Vec<Pt3> = mirrored_pts.into_iter().rev().collect();

        // Mirror the normal
        let d = normal.dot(&plane_normal);
        let mirrored_normal = Vec3::new(
            normal.x - 2.0 * d * plane_normal.x,
            normal.y - 2.0 * d * plane_normal.y,
            normal.z - 2.0 * d * plane_normal.z,
        );

        mirrored_faces.push((mirrored_pts_reversed, mirrored_normal));
    }

    // Detect coincident face pairs (original face i and mirrored face j share
    // the same vertices and have opposite normals). Both faces in such a pair
    // are internal and must be removed for a watertight result.
    let orig_keys: Vec<_> = original_faces.iter().map(|(pts, _)| face_vertex_key(pts, tol)).collect();
    let mirr_keys: Vec<_> = mirrored_faces.iter().map(|(pts, _)| face_vertex_key(pts, tol)).collect();

    let mut orig_remove = vec![false; original_faces.len()];
    let mut mirr_remove = vec![false; mirrored_faces.len()];

    for (i, ok) in orig_keys.iter().enumerate() {
        for (j, mk) in mirr_keys.iter().enumerate() {
            if ok == mk && !orig_remove[i] && !mirr_remove[j] {
                // Check normals are opposite
                let n1 = original_faces[i].1;
                let n2 = mirrored_faces[j].1;
                if (n1 + n2).norm() < tol {
                    orig_remove[i] = true;
                    mirr_remove[j] = true;
                }
            }
        }
    }

    let mut all_faces: Vec<(Vec<Pt3>, Vec3)> = Vec::new();
    for (i, face) in original_faces.into_iter().enumerate() {
        if !orig_remove[i] {
            all_faces.push(face);
        }
    }
    for (j, face) in mirrored_faces.into_iter().enumerate() {
        if !mirr_remove[j] {
            all_faces.push(face);
        }
    }

    rebuild_brep_from_faces(&all_faces)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;
    use crate::topology::body::Body;

    #[test]
    fn mirror_doubles_faces() {
        let brep = build_box_brep(5.0, 5.0, 5.0).unwrap();
        let params = MirrorParams {
            plane_origin: Pt3::new(10.0, 0.0, 0.0),
            plane_normal: Vec3::new(1.0, 0.0, 0.0),
        };
        let result = mirror_brep(&brep, &params).unwrap();
        assert_eq!(result.faces.len(), 12); // 6 original + 6 mirrored
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn mirror_empty_brep_rejected() {
        let brep = BRep::new();
        let params = MirrorParams {
            plane_origin: Pt3::new(0.0, 0.0, 0.0),
            plane_normal: Vec3::new(1.0, 0.0, 0.0),
        };
        assert!(mirror_brep(&brep, &params).is_err());
    }
}
