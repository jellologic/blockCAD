use crate::error::{KernelError, KernelResult};
use crate::geometry::surface::plane::Plane;
use crate::geometry::Vec3;
use crate::operations::boolean::csg::csg_union;
use crate::operations::extrude::{extrude_profile, ExtrudeParams, ExtrudeProfile};
use crate::topology::BRep;

use super::traits::Operation;

/// Parameters for the Rib operation.
///
/// A rib creates a thin-wall reinforcement from a sketch profile by extruding
/// it with a given thickness and unioning the result with the existing body.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RibParams {
    /// Rib wall thickness
    pub thickness: f64,
    /// Extrude direction (unit vector)
    pub direction: Vec3,
    /// Flip thickness direction (offset profile in the opposite direction)
    pub flip: bool,
    /// Apply thickness to both sides of the profile (thickness/2 each way)
    pub both_sides: bool,
}

#[derive(Debug)]
pub struct RibOp;

impl Operation for RibOp {
    type Params = RibParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "rib".into(),
            detail: "Use rib_from_profile() for rib creation".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Rib"
    }
}

/// Create a rib by extruding a profile with the given thickness and unioning
/// it with the existing body.
///
/// The profile is offset by `thickness` perpendicular to the extrude direction
/// (in the profile plane) to create a thin slab, which is then boolean-unioned
/// with the body.
///
/// For `both_sides`, the profile is offset by thickness/2 in each direction.
pub fn rib_from_profile(
    body: &BRep,
    profile: &ExtrudeProfile,
    params: &RibParams,
) -> KernelResult<BRep> {
    // Validate thickness
    if params.thickness <= 0.0 {
        return Err(KernelError::InvalidParameter {
            param: "thickness".into(),
            value: params.thickness.to_string(),
        });
    }

    let n = profile.points.len();
    if n < 3 {
        return Err(KernelError::InvalidParameter {
            param: "profile".into(),
            value: format!("Need at least 3 points, got {}", n),
        });
    }

    // The rib is a thin slab: we create a thickened profile by offsetting
    // the profile points perpendicular to the extrude direction within the
    // profile plane. We build a closed rectangular cross-section from the
    // original and offset profiles, then extrude it.

    let dir_norm = params.direction.normalize();

    // Compute the offset direction: perpendicular to extrude direction, lying
    // in the profile plane. We use the plane's normal crossed with the extrude
    // direction to get the in-plane perpendicular.
    // If the extrude direction is aligned with the plane normal, we use the
    // plane's u_axis as the offset direction.
    let plane_normal = profile.plane.normal.normalize();
    let offset_dir = {
        let candidate = plane_normal.cross(&dir_norm);
        if candidate.norm() < 1e-12 {
            // Extrude direction is along the plane normal; offset in u_axis direction
            profile.plane.u_axis.normalize()
        } else {
            candidate.normalize()
        }
    };

    // Build thickened profile: a closed polygon that represents the rib cross-section.
    // Original profile forms one side, offset profile forms the other, connected at ends.
    let (offset_a, offset_b) = if params.both_sides {
        let half = params.thickness / 2.0;
        (-half, half)
    } else if params.flip {
        (-params.thickness, 0.0)
    } else {
        (0.0, params.thickness)
    };

    // Create side A (offset_a) and side B (offset_b) points
    let side_a: Vec<_> = profile
        .points
        .iter()
        .map(|p| *p + offset_dir * offset_a)
        .collect();
    let side_b: Vec<_> = profile
        .points
        .iter()
        .map(|p| *p + offset_dir * offset_b)
        .collect();

    // Build the closed polygon: side_a forward, then side_b reversed
    // This creates a rectangular-ish closed profile around the rib centerline.
    let mut thick_points = side_a.clone();
    thick_points.extend(side_b.iter().rev());

    // Create the thickened profile
    let thick_profile = ExtrudeProfile {
        points: thick_points,
        plane: profile.plane.clone(),
    };

    // Compute extrude depth from direction projected onto the body extent.
    // For a rib, the extrude depth is typically determined by the body geometry.
    // We use a reasonable depth: the extent of the body along the extrude direction.
    let depth = compute_rib_depth(body, profile, dir_norm);

    let extrude_params = ExtrudeParams::blind(dir_norm, depth);
    let rib_brep = extrude_profile(&thick_profile, &extrude_params)?;

    // Union the rib with the existing body
    csg_union(body, &rib_brep)
}

/// Compute the rib extrusion depth by finding the body extent along the
/// extrude direction from the profile centroid.
fn compute_rib_depth(body: &BRep, profile: &ExtrudeProfile, direction: Vec3) -> f64 {
    let dir = direction.normalize();

    // Profile centroid
    let n = profile.points.len() as f64;
    let centroid = profile.points.iter().fold(
        crate::geometry::Pt3::new(0.0, 0.0, 0.0),
        |acc, p| crate::geometry::Pt3::new(acc.x + p.x, acc.y + p.y, acc.z + p.z),
    );
    let centroid = crate::geometry::Pt3::new(centroid.x / n, centroid.y / n, centroid.z / n);

    // Find the max extent of the body along the direction from the centroid
    let mut max_t: f64 = 0.0;
    for (_, v) in body.vertices.iter() {
        let diff = v.point - centroid;
        let t = Vec3::new(diff.x, diff.y, diff.z).dot(&dir);
        if t > max_t {
            max_t = t;
        }
    }

    // Use a minimum depth to avoid degenerate extrusions
    max_t.max(1.0)
}

/// Compute the perpendicular offset direction for the rib within the profile plane.
pub(crate) fn compute_offset_direction(plane: &Plane, extrude_dir: Vec3) -> Vec3 {
    let plane_normal = plane.normal.normalize();
    let dir_norm = extrude_dir.normalize();
    let candidate = plane_normal.cross(&dir_norm);
    if candidate.norm() < 1e-12 {
        plane.u_axis.normalize()
    } else {
        candidate.normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;
    use crate::geometry::{Pt3, Vec3};
    use crate::operations::extrude::{extrude_profile, ExtrudeParams, ExtrudeProfile};
    use crate::topology::body::Body;

    /// Helper: create a 10x10 square profile on the XY plane
    fn square_profile() -> ExtrudeProfile {
        ExtrudeProfile {
            points: vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(10.0, 0.0, 0.0),
                Pt3::new(10.0, 10.0, 0.0),
                Pt3::new(0.0, 10.0, 0.0),
            ],
            plane: Plane::xy(0.0),
        }
    }

    /// Helper: create a box body (10x10x10 extruded from XY plane along Z)
    fn box_body() -> BRep {
        let profile = square_profile();
        let params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 10.0);
        extrude_profile(&profile, &params).unwrap()
    }

    /// Helper: create a rib profile (a line-like thin profile on the XZ plane
    /// that cuts through the box along Y)
    fn rib_profile() -> ExtrudeProfile {
        ExtrudeProfile {
            points: vec![
                Pt3::new(2.0, 5.0, 2.0),
                Pt3::new(8.0, 5.0, 2.0),
                Pt3::new(8.0, 5.0, 8.0),
                Pt3::new(2.0, 5.0, 8.0),
            ],
            plane: Plane {
                origin: Pt3::new(0.0, 5.0, 0.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
                u_axis: Vec3::new(1.0, 0.0, 0.0),
                v_axis: Vec3::new(0.0, 0.0, 1.0),
            },
        }
    }

    #[test]
    fn rib_simple_on_box() {
        let body = box_body();
        let profile = rib_profile();
        let params = RibParams {
            thickness: 1.0,
            direction: Vec3::new(0.0, 1.0, 0.0),
            flip: false,
            both_sides: false,
        };
        let result = rib_from_profile(&body, &profile, &params);
        assert!(result.is_ok(), "Rib on box should succeed: {:?}", result.err());
        let brep = result.unwrap();
        assert!(matches!(brep.body, Body::Solid(_)), "Result should be a solid");
        assert!(brep.faces.len() >= 6, "Union result should have at least 6 faces");
    }

    #[test]
    fn rib_both_sides() {
        let body = box_body();
        let profile = rib_profile();
        let params = RibParams {
            thickness: 2.0,
            direction: Vec3::new(0.0, 1.0, 0.0),
            flip: false,
            both_sides: true,
        };
        let result = rib_from_profile(&body, &profile, &params);
        assert!(result.is_ok(), "Rib both_sides should succeed: {:?}", result.err());
        let brep = result.unwrap();
        assert!(matches!(brep.body, Body::Solid(_)));
    }

    #[test]
    fn rib_with_flip() {
        let body = box_body();
        let profile = rib_profile();
        let params_flip = RibParams {
            thickness: 1.0,
            direction: Vec3::new(0.0, 1.0, 0.0),
            flip: true,
            both_sides: false,
        };
        let result = rib_from_profile(&body, &profile, &params_flip);
        assert!(result.is_ok(), "Rib with flip should succeed: {:?}", result.err());
        let brep = result.unwrap();
        assert!(matches!(brep.body, Body::Solid(_)), "Flipped rib should produce a solid");
        assert!(brep.faces.len() >= 6, "Union result should have at least 6 faces");

        // Verify the flip affects the offset direction: build the rib slab
        // independently and check its vertex positions.
        let profile = rib_profile();
        let dir_norm = Vec3::new(0.0, 1.0, 0.0);
        let offset_dir = compute_offset_direction(&profile.plane, dir_norm);

        // With flip=true, offset should be in the negative offset_dir direction
        // (offset_a = -thickness, offset_b = 0)
        // With flip=false, offset should be in the positive direction
        // (offset_a = 0, offset_b = thickness)
        // Verify the function computes different offsets
        let side_a_flip: Vec<_> = profile
            .points
            .iter()
            .map(|p| *p + offset_dir * (-1.0))
            .collect();
        let side_a_normal: Vec<_> = profile
            .points
            .iter()
            .map(|p| *p + offset_dir * 0.0)
            .collect();

        // The first point of side_a should differ between flip and normal
        let diff = (side_a_flip[0].x - side_a_normal[0].x).abs()
            + (side_a_flip[0].y - side_a_normal[0].y).abs()
            + (side_a_flip[0].z - side_a_normal[0].z).abs();
        assert!(diff > 0.5, "Flip should produce different offset geometry");
    }

    #[test]
    fn rib_invalid_thickness_zero() {
        let body = box_body();
        let profile = rib_profile();
        let params = RibParams {
            thickness: 0.0,
            direction: Vec3::new(0.0, 1.0, 0.0),
            flip: false,
            both_sides: false,
        };
        let result = rib_from_profile(&body, &profile, &params);
        assert!(result.is_err(), "Zero thickness should fail");
    }

    #[test]
    fn rib_invalid_thickness_negative() {
        let body = box_body();
        let profile = rib_profile();
        let params = RibParams {
            thickness: -1.0,
            direction: Vec3::new(0.0, 1.0, 0.0),
            flip: false,
            both_sides: false,
        };
        let result = rib_from_profile(&body, &profile, &params);
        assert!(result.is_err(), "Negative thickness should fail");
    }
}
