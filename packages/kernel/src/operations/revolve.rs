use std::f64::consts::PI;

use crate::error::{KernelError, KernelResult};
use crate::geometry::surface::plane::Plane;
use crate::geometry::{Pt3, Vec3};
use crate::topology::body::Body;
use crate::topology::builders::make_planar_face;
use crate::topology::shell::Shell;
use crate::topology::solid::Solid;
use crate::topology::BRep;

use super::extrude::ExtrudeProfile;
use super::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RevolveParams {
    pub axis_origin: Pt3,
    pub axis_direction: Vec3,
    /// Angle of revolution in radians (2*PI for full revolution)
    pub angle: f64,
    /// Whether a second direction (reverse) revolution is enabled
    #[serde(default)]
    pub direction2_enabled: bool,
    /// Angle for reverse direction in radians
    #[serde(default)]
    pub angle2: f64,
    /// Whether to revolve symmetrically (Mid Plane: angle/2 in each direction)
    #[serde(default)]
    pub symmetric: bool,
    /// Whether thin feature (shell) is enabled
    #[serde(default)]
    pub thin_feature: bool,
    /// Wall thickness for thin feature
    #[serde(default)]
    pub thin_wall_thickness: f64,
    /// Flip side to cut (for cut revolve only)
    #[serde(default)]
    pub flip_side_to_cut: bool,
}

impl RevolveParams {
    pub fn full(axis_origin: Pt3, axis_direction: Vec3) -> Self {
        RevolveParams {
            axis_origin,
            axis_direction,
            angle: 2.0 * std::f64::consts::PI,
            direction2_enabled: false,
            angle2: 0.0,
            symmetric: false,
            thin_feature: false,
            thin_wall_thickness: 0.0,
            flip_side_to_cut: false,
        }
    }
}

#[derive(Debug)]
pub struct RevolveOp;

impl Operation for RevolveOp {
    type Params = RevolveParams;

    fn execute(&self, params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        if params.angle.abs() < 1e-12 {
            return Err(KernelError::InvalidParameter {
                param: "angle".into(),
                value: params.angle.to_string(),
            });
        }
        Err(KernelError::Operation {
            op: "revolve".into(),
            detail: "Use revolve_profile() for standalone revolution".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Revolve"
    }
}

/// Number of angular segments for revolution approximation.
const DEFAULT_SEGMENTS: usize = 36;

/// Revolve a closed planar profile around an axis to create a solid BRep.
///
/// The profile is rotated through `angle` radians around the axis defined by
/// `axis_origin` and `axis_direction`. For full revolutions (angle = 2*PI),
/// the resulting solid is closed. For partial revolutions, cap faces are added
/// at the start and end.
///
/// The profile points must not intersect the revolution axis.
pub fn revolve_profile(
    profile: &ExtrudeProfile,
    params: &RevolveParams,
) -> KernelResult<BRep> {
    if params.angle.abs() < 1e-12 {
        return Err(KernelError::InvalidParameter {
            param: "angle".into(),
            value: params.angle.to_string(),
        });
    }

    let n = profile.points.len();
    if n < 3 {
        return Err(KernelError::InvalidParameter {
            param: "profile".into(),
            value: format!("Need at least 3 points, got {}", n),
        });
    }

    let axis_origin = params.axis_origin;
    let axis_dir = params.axis_direction.normalize();

    // Compute start/end angles based on symmetric/direction2 settings
    let (start_angle, end_angle) = if params.symmetric {
        (-params.angle / 2.0, params.angle / 2.0)
    } else if params.direction2_enabled && params.angle2.abs() > 1e-12 {
        (-params.angle2, params.angle)
    } else {
        (0.0, params.angle)
    };
    let total_angle = end_angle - start_angle;

    let is_full = (total_angle.abs() - 2.0 * PI).abs() < 1e-6;

    // Determine number of angular segments
    let total_segments = if is_full {
        DEFAULT_SEGMENTS
    } else {
        let frac = total_angle.abs() / (2.0 * PI);
        (DEFAULT_SEGMENTS as f64 * frac).ceil().max(2.0) as usize
    };
    let angle_step = total_angle / total_segments as f64;

    let mut brep = BRep::new();

    // Generate rotated rings of points
    let mut rings: Vec<Vec<Pt3>> = Vec::with_capacity(total_segments + 1);
    for seg in 0..=total_segments {
        let theta = start_angle + angle_step * seg as f64;
        let ring: Vec<Pt3> = profile
            .points
            .iter()
            .map(|p| rotate_point_around_axis(*p, axis_origin, axis_dir, theta))
            .collect();
        rings.push(ring);
    }

    // Create side faces: for each profile edge, sweep it through the revolution
    let num_rings = if is_full {
        total_segments
    } else {
        total_segments + 1
    };

    for seg in 0..total_segments {
        let next_seg = if is_full {
            (seg + 1) % num_rings
        } else {
            seg + 1
        };

        for edge in 0..n {
            let next_edge = (edge + 1) % n;

            // Quad face: ring[seg][edge] → ring[seg][next_edge] → ring[next][next_edge] → ring[next][edge]
            let p0 = rings[seg][edge];
            let p1 = rings[seg][next_edge];
            let p2 = rings[next_seg][next_edge];
            let p3 = rings[next_seg][edge];

            // Compute normal from the first triangle of the quad to ensure
            // consistency with fix_winding (which checks per-triangle geometric normals).
            let e1 = p1 - p0;
            let e2 = p2 - p0;
            let normal = e1.cross(&e2).normalize();

            let side_plane = Plane {
                origin: p0,
                normal,
                u_axis: e1.normalize(),
                v_axis: (p3 - p0).normalize(),
            };
            make_planar_face(&mut brep, &[p0, p1, p2, p3], side_plane)?;
        }
    }

    // Add cap faces for partial revolutions
    if !is_full {
        // Start cap: the first ring (reversed winding for outward normal)
        let start_points: Vec<Pt3> = rings[0].iter().rev().copied().collect();
        let start_normal_rotated =
            rotate_point_around_axis(profile.plane.origin + profile.plane.normal, axis_origin, axis_dir, start_angle)
                - rotate_point_around_axis(profile.plane.origin, axis_origin, axis_dir, start_angle);
        let start_normal = -start_normal_rotated.normalize();
        let start_u =
            rotate_point_around_axis(profile.plane.origin + profile.plane.u_axis, axis_origin, axis_dir, start_angle)
                - rotate_point_around_axis(profile.plane.origin, axis_origin, axis_dir, start_angle);
        let start_v =
            rotate_point_around_axis(profile.plane.origin + profile.plane.v_axis, axis_origin, axis_dir, start_angle)
                - rotate_point_around_axis(profile.plane.origin, axis_origin, axis_dir, start_angle);
        let start_plane = Plane {
            origin: rotate_point_around_axis(profile.plane.origin, axis_origin, axis_dir, start_angle),
            normal: start_normal,
            u_axis: start_u.normalize(),
            v_axis: start_v.normalize(),
        };
        make_planar_face(&mut brep, &start_points, start_plane)?;

        // End cap: the last ring (rotated profile)
        let end_points: Vec<Pt3> = rings[total_segments].clone();
        let end_normal =
            rotate_point_around_axis(profile.plane.origin + profile.plane.normal, axis_origin, axis_dir, end_angle)
                - rotate_point_around_axis(profile.plane.origin, axis_origin, axis_dir, end_angle);
        let end_normal = end_normal.normalize();
        let end_u =
            rotate_point_around_axis(profile.plane.origin + profile.plane.u_axis, axis_origin, axis_dir, end_angle)
                - rotate_point_around_axis(profile.plane.origin, axis_origin, axis_dir, end_angle);
        let end_v =
            rotate_point_around_axis(profile.plane.origin + profile.plane.v_axis, axis_origin, axis_dir, end_angle)
                - rotate_point_around_axis(profile.plane.origin, axis_origin, axis_dir, end_angle);
        let end_plane = Plane {
            origin: rotate_point_around_axis(profile.plane.origin, axis_origin, axis_dir, end_angle),
            normal: end_normal,
            u_axis: end_u.normalize(),
            v_axis: end_v.normalize(),
        };
        make_planar_face(&mut brep, &end_points, end_plane)?;
    }

    // Collect all faces into a shell and solid
    let face_ids: Vec<_> = brep.faces.iter().map(|(id, _)| id).collect();
    let shell_id = brep.shells.insert(Shell::new(face_ids, is_full));
    let solid_id = brep.solids.insert(Solid::new(vec![shell_id]));
    brep.body = Body::Solid(solid_id);

    Ok(brep)
}

/// Rotate a point around an axis using Rodrigues' rotation formula.
pub(crate) fn rotate_point_around_axis(point: Pt3, axis_origin: Pt3, axis_dir: Vec3, angle: f64) -> Pt3 {
    let v = point - axis_origin;
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let k = axis_dir;

    // Rodrigues' formula: v_rot = v*cos(a) + (k×v)*sin(a) + k*(k·v)*(1-cos(a))
    let v_rot = v * cos_a + k.cross(&v) * sin_a + k * k.dot(&v) * (1.0 - cos_a);
    axis_origin + v_rot
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;
    use crate::tessellation::{tessellate_brep, TessellationParams};

    fn square_profile() -> ExtrudeProfile {
        // A small square in the XZ plane, offset from Y axis
        // (rectangle at x=2..4, z=0..2, y=0)
        ExtrudeProfile {
            points: vec![
                Pt3::new(2.0, 0.0, 0.0),
                Pt3::new(4.0, 0.0, 0.0),
                Pt3::new(4.0, 0.0, 2.0),
                Pt3::new(2.0, 0.0, 2.0),
            ],
            plane: Plane {
                origin: Pt3::new(2.0, 0.0, 0.0),
                normal: Vec3::new(0.0, -1.0, 0.0),
                u_axis: Vec3::new(1.0, 0.0, 0.0),
                v_axis: Vec3::new(0.0, 0.0, 1.0),
            },
        }
    }

    #[test]
    fn test_revolve_full_creates_solid() {
        let profile = square_profile();
        let params = RevolveParams::full(Pt3::origin(), Vec3::new(0.0, 0.0, 1.0));
        let brep = revolve_profile(&profile, &params).unwrap();

        // Full revolution of 4-edge profile with 36 segments:
        // 36 segments × 4 edges = 144 side faces, no caps
        assert_eq!(brep.faces.len(), 144);
        assert!(matches!(brep.body, Body::Solid(_)));

        // Tessellate and validate
        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        mesh.validate().unwrap();
        // 144 quad faces → 288 triangles
        assert_eq!(mesh.triangle_count(), 288);
    }

    #[test]
    fn test_revolve_half_creates_faces() {
        let profile = square_profile();
        let mut params = RevolveParams::full(Pt3::origin(), Vec3::new(0.0, 0.0, 1.0));
        params.angle = PI; // 180 degrees
        let brep = revolve_profile(&profile, &params).unwrap();

        // 18 segments × 4 edges = 72 side faces + 2 cap faces = 74
        assert_eq!(brep.faces.len(), 74);
        assert!(matches!(brep.body, Body::Solid(_)));
    }

    #[test]
    fn test_revolve_quarter() {
        let profile = square_profile();
        let mut params = RevolveParams::full(Pt3::origin(), Vec3::new(0.0, 0.0, 1.0));
        params.angle = PI / 2.0; // 90 degrees
        let brep = revolve_profile(&profile, &params).unwrap();

        // 9 segments × 4 edges = 36 side faces + 2 cap faces = 38
        assert_eq!(brep.faces.len(), 38);
    }

    #[test]
    fn test_revolve_half_watertight() {
        let profile = square_profile();
        let mut params = RevolveParams::full(Pt3::origin(), Vec3::new(0.0, 0.0, 1.0));
        params.angle = PI;
        let brep = revolve_profile(&profile, &params).unwrap();
        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        assert!(mesh.is_watertight(), "Half revolve mesh should be watertight");
        mesh.validate().unwrap();
    }

    #[test]
    fn test_revolve_zero_angle_rejected() {
        let profile = square_profile();
        let mut params = RevolveParams::full(Pt3::origin(), Vec3::new(0.0, 0.0, 1.0));
        params.angle = 0.0;
        let result = revolve_profile(&profile, &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_revolve_tessellation_valid() {
        let profile = square_profile();
        let params = RevolveParams::full(Pt3::origin(), Vec3::new(0.0, 0.0, 1.0));
        let brep = revolve_profile(&profile, &params).unwrap();

        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        mesh.validate().unwrap();

        // All normals should be unit vectors
        for i in (0..mesh.normals.len()).step_by(3) {
            let nx = mesh.normals[i] as f64;
            let ny = mesh.normals[i + 1] as f64;
            let nz = mesh.normals[i + 2] as f64;
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            assert!(
                (len - 1.0).abs() < 0.01,
                "Normal at vertex {} not unit: len={}",
                i / 3,
                len
            );
        }
    }

    #[test]
    fn test_revolve_symmetric() {
        let profile = square_profile();
        let mut params = RevolveParams::full(Pt3::origin(), Vec3::new(0.0, 0.0, 1.0));
        params.angle = PI; // 180 degrees total
        params.symmetric = true; // -90 to +90
        let brep = revolve_profile(&profile, &params).unwrap();
        assert!(matches!(brep.body, Body::Solid(_)));
        // Partial revolution: should have side faces + 2 caps
        assert!(brep.faces.len() > 2);
    }

    #[test]
    fn test_revolve_direction2() {
        let profile = square_profile();
        let mut params = RevolveParams::full(Pt3::origin(), Vec3::new(0.0, 0.0, 1.0));
        params.angle = PI / 2.0; // 90 degrees forward
        params.direction2_enabled = true;
        params.angle2 = PI / 2.0; // 90 degrees reverse
        let brep = revolve_profile(&profile, &params).unwrap();
        assert!(matches!(brep.body, Body::Solid(_)));
    }

    #[test]
    fn test_revolve_symmetric_ignores_direction2() {
        let profile = square_profile();
        let mut params = RevolveParams::full(Pt3::origin(), Vec3::new(0.0, 0.0, 1.0));
        params.angle = PI;
        params.symmetric = true;
        params.direction2_enabled = true; // should be ignored
        params.angle2 = PI; // should be ignored
        let brep = revolve_profile(&profile, &params).unwrap();
        assert!(matches!(brep.body, Body::Solid(_)));
    }
}
