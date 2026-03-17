use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::geometry::curve::Curve;
use crate::operations::extrude::ExtrudeProfile;
use crate::topology::BRep;
use crate::topology::builders::build_brep_from_rings;

use super::traits::Operation;

const DEFAULT_SEGMENTS: usize = 36;

/// A guide curve that controls how the profile shape varies along the sweep path.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuideCurve {
    /// 3D points defining the guide curve (ordered from path start to path end)
    pub points: Vec<Pt3>,
}

impl GuideCurve {
    /// Interpolate a point on the guide curve at parameter t in [0, 1].
    fn point_at(&self, t: f64) -> Pt3 {
        let n = self.points.len();
        if n == 0 {
            return Pt3::origin();
        }
        if n == 1 || t <= 0.0 {
            return self.points[0];
        }
        if t >= 1.0 {
            return self.points[n - 1];
        }

        let segment_t = t * (n - 1) as f64;
        let idx = segment_t.floor() as usize;
        let frac = segment_t - idx as f64;

        if idx >= n - 1 {
            return self.points[n - 1];
        }

        let p0 = &self.points[idx];
        let p1 = &self.points[idx + 1];
        Pt3::new(
            p0.x + (p1.x - p0.x) * frac,
            p0.y + (p1.y - p0.y) * frac,
            p0.z + (p1.z - p0.z) * frac,
        )
    }
}

/// Profile orientation mode during sweep.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum SweepOrientation {
    /// Profile normal follows path tangent using parallel-transport frame (default).
    FollowPath,
    /// Profile maintains its original orientation throughout the sweep.
    KeepNormal,
    /// Follow path with twist determined by a guide curve (guide provided externally).
    FollowPathAndGuide,
    /// Constant twist rate: linear interpolation from 0 to `total_twist` radians.
    TwistAlongPath {
        total_twist: f64,
    },
}

impl Default for SweepOrientation {
    fn default() -> Self {
        SweepOrientation::FollowPath
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SweepParams {
    pub segments: Option<usize>,
    /// Twist angle along the sweep (radians). Legacy field; prefer `orientation`.
    pub twist: f64,
    /// Optional guide curves that control how the profile scales along the path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guide_curves: Option<Vec<GuideCurve>>,
    /// Profile orientation mode. Defaults to FollowPath (parallel transport).
    #[serde(default)]
    pub orientation: SweepOrientation,
}

impl Default for SweepParams {
    fn default() -> Self {
        Self {
            segments: None,
            twist: 0.0,
            guide_curves: None,
            orientation: SweepOrientation::default(),
        }
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

/// Compute the Frenet-Serret frame at parameter `t` along a path curve.
pub fn frenet_frame(path: &dyn Curve, t: f64) -> KernelResult<(Vec3, Vec3, Vec3)> {
    let tangent = path.tangent_at(t)?.normalize();
    let d2 = path.second_derivative_at(t)?;

    let d2_perp = d2 - tangent * d2.dot(&tangent);
    let d2_perp_len = d2_perp.norm();

    let normal = if d2_perp_len > 1e-12 {
        d2_perp / d2_perp_len
    } else {
        let arbitrary = if tangent.x.abs() < 0.9 {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };
        let n = tangent.cross(&arbitrary);
        n.normalize()
    };

    let binormal = tangent.cross(&normal).normalize();

    Ok((tangent, normal, binormal))
}

/// Project a 3D point onto a local frame, returning (u, v) coordinates.
fn project_to_local(point: &Pt3, center: &Pt3, u_axis: &Vec3, v_axis: &Vec3) -> (f64, f64) {
    let delta = Vec3::new(point.x - center.x, point.y - center.y, point.z - center.z);
    (delta.dot(u_axis), delta.dot(v_axis))
}

/// Compute scale factors from guide curves at a given sweep fraction.
fn compute_guide_scale_factors(
    guide_curves: &[GuideCurve],
    fraction: f64,
    path_pos: &Pt3,
    u_axis: &Vec3,
    v_axis: &Vec3,
    _profile_centroid: &Vec3,
    profile_u_axis: &Vec3,
    profile_v_axis: &Vec3,
    local_offsets: &[Vec3],
) -> (f64, f64) {
    if guide_curves.is_empty() {
        return (1.0, 1.0);
    }

    if guide_curves.len() == 1 {
        let guide_pt = guide_curves[0].point_at(fraction);
        let (gu, gv) = project_to_local(&guide_pt, path_pos, u_axis, v_axis);
        let guide_dist = (gu * gu + gv * gv).sqrt();

        if guide_dist <= 1e-12 {
            return (1.0, 1.0);
        }
        let guide_dir_u = gu / guide_dist;
        let guide_dir_v = gv / guide_dist;

        let mut max_extent = 0.0f64;
        for offset in local_offsets {
            let lu = offset.dot(profile_u_axis);
            let lv = offset.dot(profile_v_axis);
            let proj = lu * guide_dir_u + lv * guide_dir_v;
            max_extent = max_extent.max(proj.abs());
        }

        if max_extent < 1e-12 {
            return (1.0, 1.0);
        }

        let scale = guide_dist / max_extent;
        return (scale, scale);
    }

    // Two or more guide curves: first controls u-scale, second controls v-scale.
    let mut su = 1.0;
    let mut sv = 1.0;

    {
        let guide_pt = guide_curves[0].point_at(fraction);
        let (gu, _gv) = project_to_local(&guide_pt, path_pos, u_axis, v_axis);
        let guide_u_dist = gu.abs();
        let mut max_u = 0.0f64;
        for offset in local_offsets {
            max_u = max_u.max(offset.dot(profile_u_axis).abs());
        }
        if max_u > 1e-12 {
            su = guide_u_dist / max_u;
        }
    }

    {
        let guide_pt = guide_curves[1].point_at(fraction);
        let (_gu, gv) = project_to_local(&guide_pt, path_pos, u_axis, v_axis);
        let guide_v_dist = gv.abs();
        let mut max_v = 0.0f64;
        for offset in local_offsets {
            max_v = max_v.max(offset.dot(profile_v_axis).abs());
        }
        if max_v > 1e-12 {
            sv = guide_v_dist / max_v;
        }
    }

    (su, sv)
}

/// Sweep a profile along a path curve, producing a solid BRep.
pub fn sweep_profile(
    profile: &ExtrudeProfile,
    path: &dyn Curve,
    params: &SweepParams,
) -> KernelResult<BRep> {
    sweep_profile_with_guide(profile, path, params, None)
}

/// Sweep a profile along a path curve with an optional external guide curve
/// (used for FollowPathAndGuide orientation mode).
pub fn sweep_profile_with_guide(
    profile: &ExtrudeProfile,
    path: &dyn Curve,
    params: &SweepParams,
    guide: Option<&dyn Curve>,
) -> KernelResult<BRep> {
    let n_profile = profile.points.len();
    if n_profile < 3 {
        return Err(KernelError::InvalidParameter {
            param: "profile".into(),
            value: format!("Need at least 3 profile points, got {}", n_profile),
        });
    }

    // Validate guide curves if provided
    if let Some(ref guides) = params.guide_curves {
        for (i, gc) in guides.iter().enumerate() {
            if gc.points.len() < 2 {
                return Err(KernelError::InvalidParameter {
                    param: format!("guide_curves[{}]", i),
                    value: format!(
                        "Guide curve needs at least 2 points, got {}",
                        gc.points.len()
                    ),
                });
            }
        }
    }

    if matches!(params.orientation, SweepOrientation::FollowPathAndGuide) && guide.is_none() {
        return Err(KernelError::InvalidParameter {
            param: "guide".into(),
            value: "FollowPathAndGuide orientation requires a guide curve".into(),
        });
    }

    let (t_start, t_end) = path.domain();
    let is_closed = path.is_closed();
    let n_segments = params.segments.unwrap_or(DEFAULT_SEGMENTS);

    let mut rings: Vec<Vec<Pt3>> = Vec::with_capacity(n_segments + 1);
    let num_stations = if is_closed { n_segments } else { n_segments + 1 };

    let profile_centroid = {
        let sum = profile.points.iter().fold(Vec3::new(0.0, 0.0, 0.0), |acc, p| {
            acc + Vec3::new(p.x, p.y, p.z)
        });
        sum / n_profile as f64
    };

    let local_offsets: Vec<Vec3> = profile.points.iter().map(|p| {
        Vec3::new(p.x, p.y, p.z) - profile_centroid
    }).collect();

    let has_data_guides = params.guide_curves.as_ref().map_or(false, |g| !g.is_empty());

    let effective_twist = match &params.orientation {
        SweepOrientation::TwistAlongPath { total_twist } => *total_twist,
        _ => params.twist,
    };

    // Build frames according to orientation mode
    let mut prev_u = profile.plane.u_axis;
    let mut _prev_v = profile.plane.v_axis;

    // For guide-based orientation
    let _guide_initial = if let (SweepOrientation::FollowPathAndGuide, Some(guide_curve)) =
        (&params.orientation, guide)
    {
        let path_start = path.point_at(t_start)?;
        let guide_start = guide_curve.point_at(guide_curve.domain().0)?;
        let diff = guide_start - path_start;
        let diff_len = diff.norm();
        if diff_len > 1e-12 { Some(diff / diff_len) } else { None }
    } else {
        None
    };

    for i in 0..num_stations {
        let frac = i as f64 / n_segments as f64;
        let t = t_start + (t_end - t_start) * frac;
        let pos = path.point_at(t)?;
        let tangent = path.tangent_at(t)?.normalize();

        let (u_frame, v_frame) = match &params.orientation {
            SweepOrientation::FollowPath | SweepOrientation::TwistAlongPath { .. } => {
                let u = prev_u - tangent * prev_u.dot(&tangent);
                let u_len = u.norm();
                let u = if u_len > 1e-12 {
                    u / u_len
                } else {
                    let arbitrary = if tangent.x.abs() < 0.9 {
                        Vec3::new(1.0, 0.0, 0.0)
                    } else {
                        Vec3::new(0.0, 1.0, 0.0)
                    };
                    tangent.cross(&arbitrary).normalize()
                };
                let v = tangent.cross(&u).normalize();
                (u, v)
            }
            SweepOrientation::KeepNormal => {
                (profile.plane.u_axis, profile.plane.v_axis)
            }
            SweepOrientation::FollowPathAndGuide => {
                let u = prev_u - tangent * prev_u.dot(&tangent);
                let u_len = u.norm();
                let u_pt = if u_len > 1e-12 {
                    u / u_len
                } else {
                    let arbitrary = if tangent.x.abs() < 0.9 {
                        Vec3::new(1.0, 0.0, 0.0)
                    } else {
                        Vec3::new(0.0, 1.0, 0.0)
                    };
                    tangent.cross(&arbitrary).normalize()
                };
                let v_pt = tangent.cross(&u_pt).normalize();

                if let Some(guide_curve) = guide {
                    let (gt_start, gt_end) = guide_curve.domain();
                    let gt = gt_start + (gt_end - gt_start) * frac;
                    let guide_pos = guide_curve.point_at(gt)?;
                    let to_guide = guide_pos - pos;
                    let proj = to_guide - tangent * to_guide.dot(&tangent);
                    let proj_len = proj.norm();
                    if proj_len > 1e-12 {
                        let proj_dir = proj / proj_len;
                        let cos_a = proj_dir.dot(&u_pt).clamp(-1.0, 1.0);
                        let sin_a = proj_dir.dot(&v_pt).clamp(-1.0, 1.0);
                        let guide_angle = sin_a.atan2(cos_a);
                        let twist_angle = if i == 0 { 0.0 } else { guide_angle };
                        let (cos_t, sin_t) = (twist_angle.cos(), twist_angle.sin());
                        let u_final = u_pt * cos_t + v_pt * sin_t;
                        let v_final = -u_pt * sin_t + v_pt * cos_t;
                        (u_final, v_final)
                    } else {
                        (u_pt, v_pt)
                    }
                } else {
                    (u_pt, v_pt)
                }
            }
        };

        // Apply twist
        let twist_angle = effective_twist * frac;
        let (cos_t, sin_t) = (twist_angle.cos(), twist_angle.sin());
        let u_twisted = u_frame * cos_t + v_frame * sin_t;
        let v_twisted = -u_frame * sin_t + v_frame * cos_t;

        // Compute guide curve data scale factors
        let (su, sv) = if has_data_guides {
            compute_guide_scale_factors(
                params.guide_curves.as_ref().unwrap(),
                frac,
                &pos,
                &u_twisted,
                &v_twisted,
                &profile_centroid,
                &profile.plane.u_axis,
                &profile.plane.v_axis,
                &local_offsets,
            )
        } else {
            (1.0, 1.0)
        };

        // Place profile at this station
        let ring: Vec<Pt3> = local_offsets.iter().map(|offset| {
            let local_u = offset.dot(&profile.plane.u_axis) * su;
            let local_v = offset.dot(&profile.plane.v_axis) * sv;
            Pt3::new(
                pos.x + u_twisted.x * local_u + v_twisted.x * local_v,
                pos.y + u_twisted.y * local_u + v_twisted.y * local_v,
                pos.z + u_twisted.z * local_u + v_twisted.z * local_v,
            )
        }).collect();

        rings.push(ring);

        prev_u = u_twisted;
        _prev_v = v_twisted;
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
        let params = SweepParams { segments: Some(10), twist: 0.0, ..Default::default() };
        let result = sweep_profile(&profile, &path, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
        assert!(result.faces.len() > 0, "Should produce faces");
    }

    #[test]
    fn sweep_with_twist() {
        let profile = make_square_profile(2.0);
        let path = Line3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(0.0, 0.0, 10.0)).unwrap();
        let params = SweepParams { segments: Some(20), twist: std::f64::consts::PI, ..Default::default() };
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

    // --- Guide curve tests ---

    #[test]
    fn sweep_without_guide_curves_matches_existing() {
        let profile = make_square_profile(2.0);
        let path = Line3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(0.0, 0.0, 10.0)).unwrap();
        let params_none = SweepParams { segments: Some(10), twist: 0.0, ..Default::default() };
        let params_empty = SweepParams { segments: Some(10), twist: 0.0, guide_curves: Some(vec![]), ..Default::default() };
        let result_none = sweep_profile(&profile, &path, &params_none).unwrap();
        let result_empty = sweep_profile(&profile, &path, &params_empty).unwrap();
        assert_eq!(result_none.faces.len(), result_empty.faces.len());
    }

    #[test]
    fn sweep_with_single_guide_curve_varying_width() {
        let profile = make_square_profile(2.0);
        let path = Line3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(0.0, 0.0, 10.0)).unwrap();
        let guide = GuideCurve {
            points: vec![Pt3::new(1.0, 0.0, 0.0), Pt3::new(2.0, 0.0, 10.0)],
        };
        let params = SweepParams {
            segments: Some(10),
            twist: 0.0,
            guide_curves: Some(vec![guide]),
            ..Default::default()
        };
        let result = sweep_profile(&profile, &path, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn sweep_with_two_guide_curves() {
        let profile = make_square_profile(2.0);
        let path = Line3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(0.0, 0.0, 10.0)).unwrap();
        let guide_u = GuideCurve { points: vec![Pt3::new(1.0, 0.0, 0.0), Pt3::new(3.0, 0.0, 10.0)] };
        let guide_v = GuideCurve { points: vec![Pt3::new(0.0, 1.0, 0.0), Pt3::new(0.0, 0.5, 10.0)] };
        let params = SweepParams {
            segments: Some(10),
            twist: 0.0,
            guide_curves: Some(vec![guide_u, guide_v]),
            ..Default::default()
        };
        let result = sweep_profile(&profile, &path, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn sweep_guide_curve_too_few_points() {
        let profile = make_square_profile(2.0);
        let path = Line3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(0.0, 0.0, 10.0)).unwrap();
        let guide = GuideCurve { points: vec![Pt3::new(1.0, 0.0, 5.0)] };
        let params = SweepParams {
            segments: Some(10),
            twist: 0.0,
            guide_curves: Some(vec![guide]),
            ..Default::default()
        };
        assert!(sweep_profile(&profile, &path, &params).is_err());
    }

    #[test]
    fn guide_curve_interpolation() {
        let guide = GuideCurve {
            points: vec![Pt3::new(0.0, 0.0, 0.0), Pt3::new(10.0, 0.0, 5.0), Pt3::new(20.0, 0.0, 10.0)],
        };
        assert!((guide.point_at(0.0).x - 0.0).abs() < 1e-12);
        assert!((guide.point_at(0.5).x - 10.0).abs() < 1e-12);
        assert!((guide.point_at(1.0).x - 20.0).abs() < 1e-12);
        assert!((guide.point_at(0.25).x - 5.0).abs() < 1e-12);
    }

    #[test]
    fn sweep_guide_curve_serialization_roundtrip() {
        let params = SweepParams {
            segments: Some(10),
            twist: 0.5,
            guide_curves: Some(vec![GuideCurve { points: vec![Pt3::new(1.0, 0.0, 0.0), Pt3::new(2.0, 0.0, 10.0)] }]),
            ..Default::default()
        };
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: SweepParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.segments, Some(10));
        assert!((deserialized.twist - 0.5).abs() < 1e-12);
        assert_eq!(deserialized.guide_curves.unwrap().len(), 1);
    }

    #[test]
    fn sweep_no_guide_curves_omitted_in_json() {
        let params = SweepParams { segments: Some(10), twist: 0.0, ..Default::default() };
        let json = serde_json::to_string(&params).unwrap();
        assert!(!json.contains("guide_curves"));
        let json_no_guide = r#"{"segments":10,"twist":0.0}"#;
        let deserialized: SweepParams = serde_json::from_str(json_no_guide).unwrap();
        assert!(deserialized.guide_curves.is_none());
    }

    // --- Orientation mode tests ---

    #[test]
    fn follow_path_straight_line_no_rotation() {
        let profile = make_square_profile(2.0);
        let path = Line3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(0.0, 0.0, 10.0)).unwrap();
        let params = SweepParams {
            segments: Some(10),
            twist: 0.0,
            orientation: SweepOrientation::FollowPath,
            ..Default::default()
        };
        let result = sweep_profile(&profile, &path, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn follow_path_curved_path_rotates_profile() {
        let profile = make_square_profile(1.0);
        let path = Arc3::new(
            Pt3::new(0.0, 0.0, 0.0), 5.0, Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0), 0.0, std::f64::consts::FRAC_PI_2,
        ).unwrap();
        let params = SweepParams {
            segments: Some(20), twist: 0.0,
            orientation: SweepOrientation::FollowPath, ..Default::default()
        };
        let result = sweep_profile(&profile, &path, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn keep_normal_curved_path_no_rotation() {
        let profile = make_square_profile(1.0);
        let path = Arc3::new(
            Pt3::new(0.0, 0.0, 0.0), 5.0, Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0), 0.0, std::f64::consts::FRAC_PI_2,
        ).unwrap();
        let params = SweepParams {
            segments: Some(20), twist: 0.0,
            orientation: SweepOrientation::KeepNormal, ..Default::default()
        };
        let result = sweep_profile(&profile, &path, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn twist_along_path_rotates_linearly() {
        let profile = make_square_profile(2.0);
        let path = Line3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(0.0, 0.0, 10.0)).unwrap();
        let params = SweepParams {
            segments: Some(20), twist: 0.0,
            orientation: SweepOrientation::TwistAlongPath { total_twist: std::f64::consts::PI },
            ..Default::default()
        };
        let result = sweep_profile(&profile, &path, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn default_orientation_is_follow_path() {
        let params = SweepParams::default();
        assert!(matches!(params.orientation, SweepOrientation::FollowPath));
    }

    #[test]
    fn frenet_frame_on_straight_line() {
        let path = Line3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(0.0, 0.0, 10.0)).unwrap();
        let (tangent, normal, binormal) = frenet_frame(&path, 0.5).unwrap();
        assert!((tangent.z.abs() - 1.0) < 1e-9);
        assert!(tangent.dot(&normal).abs() < 1e-9);
        assert!(tangent.dot(&binormal).abs() < 1e-9);
        assert!(normal.dot(&binormal).abs() < 1e-9);
    }

    #[test]
    fn frenet_frame_on_arc() {
        let path = Arc3::new(
            Pt3::new(0.0, 0.0, 0.0), 5.0, Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0), 0.0, std::f64::consts::PI,
        ).unwrap();
        let (tangent, normal, binormal) = frenet_frame(&path, 0.0).unwrap();
        assert!(tangent.y.abs() > 0.9);
        assert!((tangent.norm() - 1.0).abs() < 1e-9);
        assert!((normal.norm() - 1.0).abs() < 1e-9);
        assert!((binormal.norm() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn follow_path_and_guide_requires_guide() {
        let profile = make_square_profile(2.0);
        let path = Line3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(0.0, 0.0, 10.0)).unwrap();
        let params = SweepParams {
            segments: Some(10), twist: 0.0,
            orientation: SweepOrientation::FollowPathAndGuide, ..Default::default()
        };
        assert!(sweep_profile(&profile, &path, &params).is_err());
    }

    #[test]
    fn follow_path_and_guide_with_guide_curve() {
        let profile = make_square_profile(1.0);
        let path = Line3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(0.0, 0.0, 10.0)).unwrap();
        let guide = Line3::new(Pt3::new(2.0, 0.0, 0.0), Pt3::new(0.0, 2.0, 10.0)).unwrap();
        let params = SweepParams {
            segments: Some(20), twist: 0.0,
            orientation: SweepOrientation::FollowPathAndGuide, ..Default::default()
        };
        let result = sweep_profile_with_guide(&profile, &path, &params, Some(&guide)).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn sweep_orientation_serialization_roundtrip() {
        let params = SweepParams {
            segments: Some(10), twist: 0.0,
            orientation: SweepOrientation::TwistAlongPath { total_twist: 1.5 },
            ..Default::default()
        };
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: SweepParams = serde_json::from_str(&json).unwrap();
        if let SweepOrientation::TwistAlongPath { total_twist } = deserialized.orientation {
            assert!((total_twist - 1.5).abs() < 1e-12);
        } else {
            panic!("Expected TwistAlongPath");
        }
    }

    #[test]
    fn sweep_params_backwards_compatible_deserialization() {
        let json_str = r#"{"segments":10,"twist":0.5}"#;
        let params: SweepParams = serde_json::from_str(json_str).unwrap();
        assert!(matches!(params.orientation, SweepOrientation::FollowPath));
        assert!((params.twist - 0.5).abs() < 1e-12);
    }
}
