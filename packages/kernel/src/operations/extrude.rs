use crate::error::{KernelError, KernelResult};
use crate::geometry::surface::plane::Plane;
use crate::geometry::{Pt3, Vec3};
use crate::topology::builders::make_planar_face;
use crate::topology::body::Body;
use crate::topology::shell::Shell;
use crate::topology::solid::Solid;
use crate::topology::BRep;

use super::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtrudeParams {
    /// Direction of extrusion (unit vector)
    pub direction: Vec3,
    /// Depth of extrusion
    pub depth: f64,
    /// Whether to extrude symmetrically in both directions
    pub symmetric: bool,
    /// Draft angle in radians (for tapered extrusions)
    pub draft_angle: f64,
}

/// Input profile for extrusion: an ordered list of 3D points forming a closed loop,
/// plus the plane they lie on.
#[derive(Debug, Clone)]
pub struct ExtrudeProfile {
    pub points: Vec<Pt3>,
    pub plane: Plane,
}

#[derive(Debug)]
pub struct ExtrudeOp;

impl Operation for ExtrudeOp {
    type Params = ExtrudeParams;

    fn execute(&self, params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        if params.depth <= 0.0 {
            return Err(KernelError::InvalidParameter {
                param: "depth".into(),
                value: params.depth.to_string(),
            });
        }
        // For now, delegate to extrude_profile for standalone use.
        // Full integration with input BRep (reading sketch profiles from it) comes later.
        Err(KernelError::Operation {
            op: "extrude".into(),
            detail: "Use extrude_profile() for standalone extrusion".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Extrude"
    }
}

/// Extrude a closed planar profile along a direction to create a solid BRep.
///
/// This is the core extrusion algorithm for the vertical slice.
/// Handles only linear edges (polygonal profiles).
pub fn extrude_profile(profile: &ExtrudeProfile, direction: Vec3, depth: f64) -> KernelResult<BRep> {
    if depth <= 0.0 {
        return Err(KernelError::InvalidParameter {
            param: "depth".into(),
            value: depth.to_string(),
        });
    }
    let n = profile.points.len();
    if n < 3 {
        return Err(KernelError::InvalidParameter {
            param: "profile".into(),
            value: format!("Need at least 3 points, got {}", n),
        });
    }

    let offset = direction.normalize() * depth;
    let mut brep = BRep::new();

    // Bottom face: profile points (reversed winding for outward normal pointing down)
    let bottom_points: Vec<Pt3> = profile.points.iter().rev().copied().collect();
    let bottom_normal = -profile.plane.normal;
    let bottom_plane = Plane {
        origin: profile.plane.origin,
        normal: bottom_normal,
        u_axis: profile.plane.u_axis,
        v_axis: profile.plane.v_axis,
    };
    make_planar_face(&mut brep, &bottom_points, bottom_plane)?;

    // Top face: translated profile points
    let top_points: Vec<Pt3> = profile.points.iter().map(|p| p + offset).collect();
    let top_plane = Plane {
        origin: profile.plane.origin + offset,
        normal: profile.plane.normal,
        u_axis: profile.plane.u_axis,
        v_axis: profile.plane.v_axis,
    };
    make_planar_face(&mut brep, &top_points, top_plane)?;

    // Side faces: one quad per profile edge
    for i in 0..n {
        let j = (i + 1) % n;
        let p0 = profile.points[i];
        let p1 = profile.points[j];
        let p2 = p1 + offset;
        let p3 = p0 + offset;

        // Compute outward normal for this side face
        let edge_dir = (p1 - p0).normalize();
        let side_normal = edge_dir.cross(&direction.normalize()).normalize();

        let side_plane = Plane {
            origin: p0,
            normal: side_normal,
            u_axis: edge_dir,
            v_axis: direction.normalize(),
        };
        make_planar_face(&mut brep, &[p0, p1, p2, p3], side_plane)?;
    }

    // Collect all faces into a shell and solid
    let face_ids: Vec<_> = brep.faces.iter().map(|(id, _)| id).collect();
    let shell_id = brep.shells.insert(Shell::new(face_ids, true));
    let solid_id = brep.solids.insert(Solid::new(vec![shell_id]));
    brep.body = Body::Solid(solid_id);

    Ok(brep)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;

    fn square_profile() -> ExtrudeProfile {
        ExtrudeProfile {
            points: vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(10.0, 0.0, 0.0),
                Pt3::new(10.0, 5.0, 0.0),
                Pt3::new(0.0, 5.0, 0.0),
            ],
            plane: Plane::xy(0.0),
        }
    }

    #[test]
    fn extrude_square_creates_six_faces() {
        let profile = square_profile();
        let brep = extrude_profile(&profile, Vec3::new(0.0, 0.0, 1.0), 3.0).unwrap();
        assert_eq!(brep.faces.len(), 6, "Box should have 6 faces");
    }

    #[test]
    fn extrude_square_has_solid_body() {
        let profile = square_profile();
        let brep = extrude_profile(&profile, Vec3::new(0.0, 0.0, 1.0), 3.0).unwrap();
        assert!(matches!(brep.body, Body::Solid(_)));
    }

    #[test]
    fn extrude_faces_have_four_edges_each() {
        let profile = square_profile();
        let brep = extrude_profile(&profile, Vec3::new(0.0, 0.0, 1.0), 3.0).unwrap();
        for (_id, face) in brep.faces.iter() {
            let loop_id = face.outer_loop.unwrap();
            let loop_ = brep.loops.get(loop_id).unwrap();
            assert_eq!(loop_.len(), 4);
        }
    }

    #[test]
    fn extrude_zero_depth_rejected() {
        let profile = square_profile();
        assert!(extrude_profile(&profile, Vec3::new(0.0, 0.0, 1.0), 0.0).is_err());
    }

    #[test]
    fn extrude_negative_depth_rejected() {
        let profile = square_profile();
        assert!(extrude_profile(&profile, Vec3::new(0.0, 0.0, 1.0), -1.0).is_err());
    }

    #[test]
    fn extrude_triangle() {
        let profile = ExtrudeProfile {
            points: vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(5.0, 0.0, 0.0),
                Pt3::new(2.5, 4.0, 0.0),
            ],
            plane: Plane::xy(0.0),
        };
        let brep = extrude_profile(&profile, Vec3::new(0.0, 0.0, 1.0), 2.0).unwrap();
        // Triangle extrusion: 2 caps + 3 sides = 5 faces
        assert_eq!(brep.faces.len(), 5);
    }
}
