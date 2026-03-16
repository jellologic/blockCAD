use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::geometry::curve::Curve;
use crate::operations::extrude::ExtrudeProfile;
use crate::topology::BRep;
use crate::topology::builders::build_brep_from_rings;

use super::traits::Operation;

const DEFAULT_SEGMENTS: usize = 36;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SweepParams {
    /// Path curve defined as start/end points + optional intermediate points.
    /// For now: simplified as a line from origin along a direction with a length,
    /// or an arc defined by center/axis/angle.
    /// The evaluator provides the actual curve object separately.
    pub segments: Option<usize>,
    /// Twist angle along the sweep (radians)
    pub twist: f64,
}

impl Default for SweepParams {
    fn default() -> Self {
        Self { segments: None, twist: 0.0 }
    }
}

#[derive(Debug)]
pub struct SweepOp;

impl Operation for SweepOp {
    type Params = SweepParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "sweep".into(),
            detail: "Use sweep_profile() directly with profile and curve".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Sweep"
    }
}

/// Sweep a profile along a path curve, producing a solid BRep.
///
/// Algorithm:
/// 1. Discretize path into N stations
/// 2. Build parallel-transport frame at each station
/// 3. Place profile at each station using the frame
/// 4. Connect adjacent rings with quad faces
/// 5. Cap start/end for open paths
pub fn sweep_profile(
    profile: &ExtrudeProfile,
    path: &dyn Curve,
    params: &SweepParams,
) -> KernelResult<BRep> {
    let n_profile = profile.points.len();
    if n_profile < 3 {
        return Err(KernelError::InvalidParameter {
            param: "profile".into(),
            value: format!("Need at least 3 profile points, got {}", n_profile),
        });
    }

    let (t_start, t_end) = path.domain();
    let is_closed = path.is_closed();
    let n_segments = params.segments.unwrap_or(DEFAULT_SEGMENTS);

    // Generate stations along path
    let mut rings: Vec<Vec<Pt3>> = Vec::with_capacity(n_segments + 1);
    let num_stations = if is_closed { n_segments } else { n_segments + 1 };

    // Compute profile centroid in local 2D space
    let profile_centroid = {
        let sum = profile.points.iter().fold(Vec3::new(0.0, 0.0, 0.0), |acc, p| {
            acc + Vec3::new(p.x, p.y, p.z)
        });
        sum / n_profile as f64
    };

    // Profile points relative to centroid (local offsets)
    let local_offsets: Vec<Vec3> = profile.points.iter().map(|p| {
        Vec3::new(p.x, p.y, p.z) - profile_centroid
    }).collect();

    // Build frames using parallel transport
    let mut prev_u = profile.plane.u_axis;
    let mut prev_v = profile.plane.v_axis;

    for i in 0..num_stations {
        let t = t_start + (t_end - t_start) * i as f64 / n_segments as f64;
        let pos = path.point_at(t)?;
        let tangent = path.tangent_at(t)?.normalize();

        // Parallel transport: project previous U,V onto plane perpendicular to new tangent
        let u = (prev_u - tangent * prev_u.dot(&tangent));
        let u_len = u.norm();
        let u = if u_len > 1e-12 { u / u_len } else {
            // Fallback: pick arbitrary perpendicular
            let arbitrary = if tangent.x.abs() < 0.9 { Vec3::new(1.0, 0.0, 0.0) } else { Vec3::new(0.0, 1.0, 0.0) };
            tangent.cross(&arbitrary).normalize()
        };
        let v = tangent.cross(&u).normalize();

        // Apply twist
        let twist_angle = params.twist * i as f64 / n_segments as f64;
        let (cos_t, sin_t) = (twist_angle.cos(), twist_angle.sin());
        let u_twisted = u * cos_t + v * sin_t;
        let v_twisted = -u * sin_t + v * cos_t;

        // Place profile at this station
        let ring: Vec<Pt3> = local_offsets.iter().map(|offset| {
            let local_u = offset.dot(&profile.plane.u_axis);
            let local_v = offset.dot(&profile.plane.v_axis);
            Pt3::new(
                pos.x + u_twisted.x * local_u + v_twisted.x * local_v,
                pos.y + u_twisted.y * local_u + v_twisted.y * local_v,
                pos.z + u_twisted.z * local_u + v_twisted.z * local_v,
            )
        }).collect();

        rings.push(ring);

        prev_u = u_twisted;
        prev_v = v_twisted;
    }

    build_brep_from_rings(&rings, is_closed, !is_closed, !is_closed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;
    use crate::geometry::curve::line::Line3;
    use crate::geometry::curve::arc::Arc3;
    use crate::topology::body::Body;

    fn make_square_profile(size: f64) -> ExtrudeProfile {
        let half = size / 2.0;
        ExtrudeProfile {
            points: vec![
                Pt3::new(-half, -half, 0.0),
                Pt3::new(half, -half, 0.0),
                Pt3::new(half, half, 0.0),
                Pt3::new(-half, half, 0.0),
            ],
            plane: Plane::xy(0.0),
        }
    }

    #[test]
    fn sweep_along_straight_line() {
        let profile = make_square_profile(2.0);
        let path = Line3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(0.0, 0.0, 10.0)).unwrap();
        let params = SweepParams { segments: Some(10), twist: 0.0 };
        let result = sweep_profile(&profile, &path, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
        assert!(result.faces.len() > 0, "Should produce faces");
    }

    #[test]
    fn sweep_with_twist() {
        let profile = make_square_profile(2.0);
        let path = Line3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(0.0, 0.0, 10.0)).unwrap();
        let params = SweepParams { segments: Some(20), twist: std::f64::consts::PI };
        let result = sweep_profile(&profile, &path, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn sweep_too_few_profile_points() {
        let profile = ExtrudeProfile {
            points: vec![Pt3::new(0.0, 0.0, 0.0), Pt3::new(1.0, 0.0, 0.0)],
            plane: Plane::xy(0.0),
        };
        let path = Line3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(0.0, 0.0, 10.0)).unwrap();
        let params = SweepParams::default();
        assert!(sweep_profile(&profile, &path, &params).is_err());
    }
}
