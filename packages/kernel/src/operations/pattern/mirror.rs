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

    let mut all_faces: Vec<(Vec<Pt3>, Vec3)> = Vec::new();

    // Keep original faces
    for (pts, normal) in &base_faces {
        all_faces.push((pts.clone(), *normal));
    }

    // Add mirrored faces
    for (pts, normal) in &base_faces {
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

        all_faces.push((mirrored_pts_reversed, mirrored_normal));
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
