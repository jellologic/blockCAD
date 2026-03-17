use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::operations::extrude::ExtrudeProfile;
use crate::topology::BRep;
use crate::topology::builders::build_brep_from_rings;

use super::traits::Operation;

const DEFAULT_SLICES_PER_SPAN: usize = 10;

/// A guide curve that controls the shape of a loft surface between profiles.
///
/// The curve is defined as a sequence of 3D points. The first point must lie on
/// the first profile, and the last point must lie on the last profile. The guide
/// curve influences the interpolated cross-sections so that the resulting surface
/// passes through the curve.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoftGuideCurve {
    pub points: Vec<Pt3>,
}

/// Tangency condition at a loft profile, controlling how the surface meets the profile plane.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TangencyCondition {
    /// No tangency constraint (default, sharp transition).
    None,
    /// Tangent to the profile plane normal (smooth G1 transition).
    Normal,
    /// Tangent in a specific direction.
    Direction(Vec3),
    /// Tangent with weight controlling influence distance.
    Weight { direction: Vec3, weight: f64 },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoftParams {
    /// Number of intermediate slices between each profile pair.
    #[serde(default = "default_slices")]
    pub slices_per_span: usize,
    /// Whether the loft is closed (last profile connects to first).
    #[serde(default)]
    pub closed: bool,
    /// Optional guide curves that control the loft surface shape.
    #[serde(default)]
    pub guide_curves: Option<Vec<LoftGuideCurve>>,
    /// Tangency condition at the start profile.
    #[serde(default)]
    pub start_tangency: Option<TangencyCondition>,
    /// Tangency condition at the end profile.
    #[serde(default)]
    pub end_tangency: Option<TangencyCondition>,
}

fn default_slices() -> usize { DEFAULT_SLICES_PER_SPAN }

impl Default for LoftParams {
    fn default() -> Self {
        Self {
            slices_per_span: DEFAULT_SLICES_PER_SPAN,
            closed: false,
            guide_curves: None,
            start_tangency: Option::None,
            end_tangency: Option::None,
        }
    }
}

#[derive(Debug)]
pub struct LoftOp;

impl Operation for LoftOp {
    type Params = LoftParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "loft".into(),
            detail: "Use loft_profiles() directly with profile list".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Loft"
    }
}

/// Validate guide curves against the given profiles.
///
/// Each guide curve must have at least 2 points. The first point must lie near
/// the first profile's plane, and the last point must lie near the last profile's
/// plane (within a tolerance).
fn validate_guide_curves(
    guides: &[LoftGuideCurve],
    profiles: &[ExtrudeProfile],
) -> KernelResult<()> {
    if profiles.len() < 2 {
        return Ok(());
    }

    let first_plane = &profiles[0].plane;
    let last_plane = &profiles[profiles.len() - 1].plane;
    let tol = 1.0; // generous tolerance for guide endpoints

    for (i, guide) in guides.iter().enumerate() {
        if guide.points.len() < 2 {
            return Err(KernelError::InvalidParameter {
                param: "guide_curves".into(),
                value: format!("Guide curve {} must have at least 2 points", i),
            });
        }

        // Check first point is near first profile plane
        let first_pt = &guide.points[0];
        let dist_first = (first_pt - first_plane.origin).dot(&first_plane.normal).abs();
        if dist_first > tol {
            return Err(KernelError::InvalidParameter {
                param: "guide_curves".into(),
                value: format!(
                    "Guide curve {} first point is {} from first profile plane (tolerance {})",
                    i, dist_first, tol,
                ),
            });
        }

        // Check last point is near last profile plane
        let last_pt = &guide.points[guide.points.len() - 1];
        let dist_last = (last_pt - last_plane.origin).dot(&last_plane.normal).abs();
        if dist_last > tol {
            return Err(KernelError::InvalidParameter {
                param: "guide_curves".into(),
                value: format!(
                    "Guide curve {} last point is {} from last profile plane (tolerance {})",
                    i, dist_last, tol,
                ),
            });
        }
    }

    Ok(())
}

/// Sample a guide curve at parameter t in [0, 1] using arc-length parameterization.
fn sample_guide_curve(guide: &LoftGuideCurve, t: f64) -> Pt3 {
    let n = guide.points.len();
    if n == 0 {
        return Pt3::origin();
    }
    if n == 1 || t <= 0.0 {
        return guide.points[0];
    }
    if t >= 1.0 {
        return guide.points[n - 1];
    }

    // Compute cumulative arc lengths
    let mut lengths = vec![0.0f64];
    for i in 1..n {
        let d = (guide.points[i] - guide.points[i - 1]).norm();
        lengths.push(lengths[i - 1] + d);
    }
    let total = lengths[n - 1];
    if total < 1e-12 {
        return guide.points[0];
    }

    let target_s = total * t;

    // Find segment
    let mut seg = 0;
    while seg < n - 2 && lengths[seg + 1] < target_s {
        seg += 1;
    }

    let seg_start = lengths[seg];
    let seg_end = lengths[seg + 1];
    let seg_len = seg_end - seg_start;
    let local_t = if seg_len > 1e-12 { (target_s - seg_start) / seg_len } else { 0.0 };

    let a = &guide.points[seg];
    let b = &guide.points[seg + 1];
    Pt3::new(
        a.x + (b.x - a.x) * local_t,
        a.y + (b.y - a.y) * local_t,
        a.z + (b.z - a.z) * local_t,
    )
}

/// Compute the centroid of a set of points.
fn centroid(pts: &[Pt3]) -> Pt3 {
    let n = pts.len() as f64;
    let sum = pts.iter().fold(Vec3::zeros(), |acc, p| acc + p.coords);
    Pt3::from(sum / n)
}

/// Apply guide curve displacements to a linearly interpolated ring.
///
/// For each guide curve, we compute the displacement between the linearly
/// interpolated guide position and the actual guide curve position at parameter t.
/// This displacement is then distributed to nearby ring vertices using inverse-distance
/// weighting, causing the ring to deform to pass through the guide curve points.
fn apply_guide_displacement(
    ring: &mut [Pt3],
    t: f64,
    guides: &[LoftGuideCurve],
    first_profile_centroid: &Pt3,
    last_profile_centroid: &Pt3,
) {
    if guides.is_empty() {
        return;
    }

    // Linearly interpolated centroid at parameter t
    let interp_centroid = Pt3::new(
        first_profile_centroid.x + (last_profile_centroid.x - first_profile_centroid.x) * t,
        first_profile_centroid.y + (last_profile_centroid.y - first_profile_centroid.y) * t,
        first_profile_centroid.z + (last_profile_centroid.z - first_profile_centroid.z) * t,
    );

    for guide in guides {
        // Where the guide curve actually is at parameter t
        let guide_pt = sample_guide_curve(guide, t);

        // Where the guide would be under linear interpolation between its endpoints
        let guide_start = &guide.points[0];
        let guide_end = &guide.points[guide.points.len() - 1];
        let guide_linear = Pt3::new(
            guide_start.x + (guide_end.x - guide_start.x) * t,
            guide_start.y + (guide_end.y - guide_start.y) * t,
            guide_start.z + (guide_end.z - guide_start.z) * t,
        );

        // The displacement caused by the guide curve deviating from linear
        let displacement = guide_pt - guide_linear;

        // Direction from centroid to the guide's linear position (for weighting)
        let guide_dir = guide_linear - interp_centroid;
        let guide_dist_from_center = guide_dir.norm();

        if guide_dist_from_center < 1e-12 {
            // Guide is at center; apply uniform displacement
            for pt in ring.iter_mut() {
                pt.x += displacement.x;
                pt.y += displacement.y;
                pt.z += displacement.z;
            }
        } else {
            let guide_dir_normalized = guide_dir / guide_dist_from_center;

            // Weight vertices by how aligned they are with the guide direction
            // from the centroid. Vertices on the same side as the guide get more
            // displacement.
            for pt in ring.iter_mut() {
                let vert_dir = *pt - interp_centroid;
                let vert_dist = vert_dir.norm();
                if vert_dist < 1e-12 {
                    continue;
                }
                let vert_dir_normalized = vert_dir / vert_dist;

                // Cosine similarity: 1.0 means same direction, -1.0 means opposite
                let cos_sim = vert_dir_normalized.dot(&guide_dir_normalized);

                // Weight: use (1 + cos_sim) / 2 to map [-1, 1] to [0, 1]
                let weight = ((1.0 + cos_sim) / 2.0).powi(2);

                pt.x += displacement.x * weight;
                pt.y += displacement.y * weight;
                pt.z += displacement.z * weight;
            }
        }
    }
}

/// Loft between multiple profiles, creating a blended solid.
///
/// Algorithm:
/// 1. Resample all profiles to a common vertex count
/// 2. Align vertex orderings to minimize distance
/// 3. Linearly interpolate between adjacent profiles
/// 4. If guide curves are present, apply displacement to interpolated rings
/// Resolve a TangencyCondition to a direction vector at a profile.
fn resolve_tangency(
    condition: &TangencyCondition,
    profile: &ExtrudeProfile,
) -> Option<Vec3> {
    match condition {
        TangencyCondition::None => None,
        TangencyCondition::Normal => Some(profile.plane.normal),
        TangencyCondition::Direction(dir) => Some(*dir),
        TangencyCondition::Weight { direction, weight } => {
            Some(*direction * *weight)
        }
    }
}

/// Compute per-profile tangent vectors for Hermite interpolation.
fn compute_profile_tangents(
    profiles: &[ExtrudeProfile],
    params: &LoftParams,
) -> Vec<Option<Vec3>> {
    let n = profiles.len();
    let mut tangents = vec![None; n];

    if let Some(ref cond) = params.start_tangency {
        tangents[0] = resolve_tangency(cond, &profiles[0]);
    }
    if let Some(ref cond) = params.end_tangency {
        tangents[n - 1] = resolve_tangency(cond, &profiles[n - 1]);
    }

    tangents
}

/// Cubic Hermite interpolation between two points with tangent vectors.
fn hermite_interpolate(a: Pt3, b: Pt3, tan_a: Vec3, tan_b: Vec3, t: f64) -> Pt3 {
    let t2 = t * t;
    let t3 = t2 * t;
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;

    Pt3::new(
        h00 * a.x + h10 * tan_a.x + h01 * b.x + h11 * tan_b.x,
        h00 * a.y + h10 * tan_a.y + h01 * b.y + h11 * tan_b.y,
        h00 * a.z + h10 * tan_a.z + h01 * b.z + h11 * tan_b.z,
    )
}

/// 5. Connect rings with quad faces
/// 6. Cap start/end if not closed
pub fn loft_profiles(
    profiles: &[ExtrudeProfile],
    params: &LoftParams,
) -> KernelResult<BRep> {
    if profiles.len() < 2 {
        return Err(KernelError::InvalidParameter {
            param: "profiles".into(),
            value: format!("Need at least 2 profiles for loft, got {}", profiles.len()),
        });
    }

    // Validate guide curves if present
    let guides = params.guide_curves.as_deref().unwrap_or(&[]);
    if !guides.is_empty() {
        validate_guide_curves(guides, profiles)?;
    }

    // Find the maximum vertex count across all profiles
    let max_verts = profiles.iter().map(|p| p.points.len()).max().unwrap();
    if max_verts < 3 {
        return Err(KernelError::InvalidParameter {
            param: "profile".into(),
            value: format!("Each profile needs at least 3 points"),
        });
    }

    // Resample all profiles to max_verts by arc-length interpolation
    let resampled: Vec<Vec<Pt3>> = profiles.iter().map(|p| {
        resample_profile(&p.points, max_verts)
    }).collect();

    // Compute centroids for guide curve displacement
    let first_centroid = centroid(&resampled[0]);
    let last_centroid = centroid(resampled.last().unwrap());

    // Resolve tangency vectors per-profile for Hermite interpolation.
    let tangent_vecs = compute_profile_tangents(profiles, params);

    // Generate all rings by interpolating between adjacent profiles
    let mut rings: Vec<Vec<Pt3>> = Vec::new();
    let n_profiles = resampled.len();
    let n_spans = if params.closed { n_profiles } else { n_profiles - 1 };
    let total_slices = n_spans * params.slices_per_span;

    for span in 0..n_spans {
        let next = if params.closed { (span + 1) % n_profiles } else { span + 1 };
        let profile_a = &resampled[span];
        let profile_b = &resampled[next];

        let tan_a = tangent_vecs[span];
        let tan_b = tangent_vecs[next];
        let use_hermite = tan_a.is_some() || tan_b.is_some();

        let n_slices = params.slices_per_span;
        for s in 0..n_slices {
            let local_t = s as f64 / n_slices as f64;
            let mut ring: Vec<Pt3> = profile_a.iter().zip(profile_b.iter()).map(|(a, b)| {
                if use_hermite {
                    let chord = Vec3::new(b.x - a.x, b.y - a.y, b.z - a.z);
                    let span_len = chord.norm().max(1e-12);
                    let ta = tan_a.unwrap_or(chord) * span_len;
                    let tb = tan_b.unwrap_or(chord) * span_len;
                    hermite_interpolate(*a, *b, ta, tb, local_t)
                } else {
                    Pt3::new(
                        a.x + (b.x - a.x) * local_t,
                        a.y + (b.y - a.y) * local_t,
                        a.z + (b.z - a.z) * local_t,
                    )
                }
            }).collect();

            // Apply guide curve displacement if guides are present
            if !guides.is_empty() {
                // Global parameter t across the entire loft
                let global_t = (span as f64 * n_slices as f64 + s as f64) / total_slices as f64;
                apply_guide_displacement(
                    &mut ring,
                    global_t,
                    guides,
                    &first_centroid,
                    &last_centroid,
                );
            }

            rings.push(ring);
        }
    }

    // Add the last profile as the final ring (for open lofts)
    if !params.closed {
        rings.push(resampled.last().unwrap().clone());
    }

    build_brep_from_rings(&rings, params.closed, !params.closed, !params.closed)
}

/// Resample a profile to have exactly `target_count` vertices by arc-length interpolation.
fn resample_profile(points: &[Pt3], target_count: usize) -> Vec<Pt3> {
    let n = points.len();
    if n == target_count {
        return points.to_vec();
    }
    if n == 0 || target_count == 0 {
        return Vec::new();
    }

    // Compute cumulative arc lengths
    let mut lengths = vec![0.0f64];
    for i in 1..n {
        let d = (points[i] - points[i - 1]).norm();
        lengths.push(lengths[i - 1] + d);
    }
    // Add closing edge
    let close_d = (points[0] - points[n - 1]).norm();
    let total_length = lengths[n - 1] + close_d;

    // Sample at uniform arc-length intervals
    let mut result = Vec::with_capacity(target_count);
    for i in 0..target_count {
        let target_s = total_length * i as f64 / target_count as f64;

        // Find segment containing target_s
        let mut seg = 0;
        while seg < n - 1 && lengths[seg + 1] < target_s {
            seg += 1;
        }

        // Interpolate within segment
        let seg_start = lengths[seg];
        let seg_next = if seg + 1 < n { seg + 1 } else { 0 };
        let seg_end = if seg + 1 < n { lengths[seg + 1] } else { total_length };
        let seg_len = seg_end - seg_start;
        let t = if seg_len > 1e-12 { (target_s - seg_start) / seg_len } else { 0.0 };

        let p0 = points[seg];
        let p1 = points[seg_next];
        result.push(Pt3::new(
            p0.x + (p1.x - p0.x) * t,
            p0.y + (p1.y - p0.y) * t,
            p0.z + (p1.z - p0.z) * t,
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;
    use crate::topology::body::Body;

    fn make_square_profile(size: f64, z: f64) -> ExtrudeProfile {
        let half = size / 2.0;
        ExtrudeProfile {
            points: vec![
                Pt3::new(-half, -half, z),
                Pt3::new(half, -half, z),
                Pt3::new(half, half, z),
                Pt3::new(-half, half, z),
            ],
            plane: Plane {
                origin: Pt3::new(0.0, 0.0, z),
                normal: Vec3::new(0.0, 0.0, 1.0),
                u_axis: Vec3::new(1.0, 0.0, 0.0),
                v_axis: Vec3::new(0.0, 1.0, 0.0),
            },
        }
    }

    #[test]
    fn loft_two_identical_profiles() {
        let p1 = make_square_profile(4.0, 0.0);
        let p2 = make_square_profile(4.0, 10.0);
        let result = loft_profiles(&[p1, p2], &LoftParams::default()).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
        assert!(result.faces.len() > 0);
    }

    #[test]
    fn loft_two_different_sizes() {
        let p1 = make_square_profile(4.0, 0.0);
        let p2 = make_square_profile(8.0, 10.0);
        let result = loft_profiles(&[p1, p2], &LoftParams::default()).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn loft_three_profiles() {
        let p1 = make_square_profile(4.0, 0.0);
        let p2 = make_square_profile(8.0, 5.0);
        let p3 = make_square_profile(2.0, 10.0);
        let result = loft_profiles(&[p1, p2, p3], &LoftParams::default()).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn loft_too_few_profiles() {
        let p1 = make_square_profile(4.0, 0.0);
        assert!(loft_profiles(&[p1], &LoftParams::default()).is_err());
    }

    #[test]
    fn resample_preserves_count() {
        let pts = vec![
            Pt3::new(0.0, 0.0, 0.0),
            Pt3::new(1.0, 0.0, 0.0),
            Pt3::new(1.0, 1.0, 0.0),
            Pt3::new(0.0, 1.0, 0.0),
        ];
        let resampled = resample_profile(&pts, 4);
        assert_eq!(resampled.len(), 4);
    }

    #[test]
    fn resample_upsamples() {
        let pts = vec![
            Pt3::new(0.0, 0.0, 0.0),
            Pt3::new(1.0, 0.0, 0.0),
            Pt3::new(1.0, 1.0, 0.0),
        ];
        let resampled = resample_profile(&pts, 6);
        assert_eq!(resampled.len(), 6);
    }

    fn make_circle_profile(radius: f64, z: f64, n_points: usize) -> ExtrudeProfile {
        use std::f64::consts::PI;
        let mut points = Vec::with_capacity(n_points);
        for i in 0..n_points {
            let angle = 2.0 * PI * i as f64 / n_points as f64;
            points.push(Pt3::new(radius * angle.cos(), radius * angle.sin(), z));
        }
        ExtrudeProfile {
            points,
            plane: Plane {
                origin: Pt3::new(0.0, 0.0, z),
                normal: Vec3::new(0.0, 0.0, 1.0),
                u_axis: Vec3::new(1.0, 0.0, 0.0),
                v_axis: Vec3::new(0.0, 1.0, 0.0),
            },
        }
    }

    #[test]
    fn loft_without_guide_curves_unchanged() {
        // Verify existing behavior is preserved when guide_curves is None
        let p1 = make_square_profile(4.0, 0.0);
        let p2 = make_square_profile(8.0, 10.0);
        let params = LoftParams { guide_curves: None, ..LoftParams::default() };
        let result = loft_profiles(&[p1, p2], &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
        assert!(result.faces.len() > 0);
    }

    #[test]
    fn loft_with_single_guide_curve() {
        // Two circles at z=0 and z=10, with a guide curve that bulges outward
        let p1 = make_circle_profile(2.0, 0.0, 16);
        let p2 = make_circle_profile(2.0, 10.0, 16);

        // Guide curve starts at (2,0,0) on the first profile, bulges to (4,0,5),
        // and ends at (2,0,10) on the last profile
        let guide = LoftGuideCurve {
            points: vec![
                Pt3::new(2.0, 0.0, 0.0),
                Pt3::new(4.0, 0.0, 5.0),
                Pt3::new(2.0, 0.0, 10.0),
            ],
        };

        let params = LoftParams {
            slices_per_span: 10,
            closed: false,
            guide_curves: Some(vec![guide]),
            ..LoftParams::default()
        };
        let result = loft_profiles(&[p1, p2], &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
        assert!(result.faces.len() > 0);
    }

    #[test]
    fn loft_with_two_guide_curves() {
        let p1 = make_circle_profile(2.0, 0.0, 16);
        let p2 = make_circle_profile(2.0, 10.0, 16);

        // Guide on +X side bulges out
        let guide1 = LoftGuideCurve {
            points: vec![
                Pt3::new(2.0, 0.0, 0.0),
                Pt3::new(4.0, 0.0, 5.0),
                Pt3::new(2.0, 0.0, 10.0),
            ],
        };

        // Guide on -X side stays straight (no bulge)
        let guide2 = LoftGuideCurve {
            points: vec![
                Pt3::new(-2.0, 0.0, 0.0),
                Pt3::new(-2.0, 0.0, 5.0),
                Pt3::new(-2.0, 0.0, 10.0),
            ],
        };

        let params = LoftParams {
            slices_per_span: 10,
            closed: false,
            guide_curves: Some(vec![guide1, guide2]),
            ..LoftParams::default()
        };
        let result = loft_profiles(&[p1, p2], &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
        assert!(result.faces.len() > 0);
    }

    #[test]
    fn loft_guide_curve_validation_too_few_points() {
        let p1 = make_circle_profile(2.0, 0.0, 16);
        let p2 = make_circle_profile(2.0, 10.0, 16);

        let guide = LoftGuideCurve {
            points: vec![Pt3::new(2.0, 0.0, 0.0)], // only 1 point
        };

        let params = LoftParams {
            slices_per_span: 10,
            closed: false,
            guide_curves: Some(vec![guide]),
            ..LoftParams::default()
        };
        assert!(loft_profiles(&[p1, p2], &params).is_err());
    }

    #[test]
    fn loft_guide_curve_validation_not_on_first_profile() {
        let p1 = make_circle_profile(2.0, 0.0, 16);
        let p2 = make_circle_profile(2.0, 10.0, 16);

        // First point at z=5, not on first profile plane (z=0)
        let guide = LoftGuideCurve {
            points: vec![
                Pt3::new(2.0, 0.0, 5.0),
                Pt3::new(2.0, 0.0, 10.0),
            ],
        };

        let params = LoftParams {
            slices_per_span: 10,
            closed: false,
            guide_curves: Some(vec![guide]),
            ..LoftParams::default()
        };
        assert!(loft_profiles(&[p1, p2], &params).is_err());
    }

    #[test]
    fn loft_guide_curve_validation_not_on_last_profile() {
        let p1 = make_circle_profile(2.0, 0.0, 16);
        let p2 = make_circle_profile(2.0, 10.0, 16);

        // Last point at z=5, not on last profile plane (z=10)
        let guide = LoftGuideCurve {
            points: vec![
                Pt3::new(2.0, 0.0, 0.0),
                Pt3::new(2.0, 0.0, 5.0),
            ],
        };

        let params = LoftParams {
            slices_per_span: 10,
            closed: false,
            guide_curves: Some(vec![guide]),
            ..LoftParams::default()
        };
        assert!(loft_profiles(&[p1, p2], &params).is_err());
    }

    #[test]
    fn loft_guide_curve_produces_bulge() {
        // Verify that a guide curve actually changes the geometry compared to no guide
        let p1 = make_circle_profile(2.0, 0.0, 16);
        let p2 = make_circle_profile(2.0, 10.0, 16);

        // Without guide
        let params_no_guide = LoftParams {
            slices_per_span: 10,
            closed: false,
            guide_curves: None,
            ..LoftParams::default()
        };
        let result_no_guide = loft_profiles(&[p1.clone(), p2.clone()], &params_no_guide).unwrap();

        // With bulging guide
        let guide = LoftGuideCurve {
            points: vec![
                Pt3::new(2.0, 0.0, 0.0),
                Pt3::new(5.0, 0.0, 5.0),
                Pt3::new(2.0, 0.0, 10.0),
            ],
        };
        let params_guide = LoftParams {
            slices_per_span: 10,
            closed: false,
            guide_curves: Some(vec![guide]),
            ..LoftParams::default()
        };
        let result_guide = loft_profiles(&[p1, p2], &params_guide).unwrap();

        // The guided loft should have different vertex positions
        let verts_no: Vec<_> = result_no_guide.vertices.iter().map(|(_, v)| v.point).collect();
        let verts_yes: Vec<_> = result_guide.vertices.iter().map(|(_, v)| v.point).collect();

        // Both should produce a valid solid with the same topology
        assert_eq!(verts_no.len(), verts_yes.len());

        // But the positions should differ (at least some vertex should be displaced)
        let mut max_diff = 0.0f64;
        for (a, b) in verts_no.iter().zip(verts_yes.iter()) {
            let diff = (a - b).norm();
            max_diff = max_diff.max(diff);
        }
        assert!(max_diff > 0.1, "Guide curve should displace vertices, max_diff={}", max_diff);
    }

    #[test]
    fn sample_guide_curve_endpoints() {
        let guide = LoftGuideCurve {
            points: vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(5.0, 0.0, 5.0),
                Pt3::new(0.0, 0.0, 10.0),
            ],
        };
        let start = sample_guide_curve(&guide, 0.0);
        let end = sample_guide_curve(&guide, 1.0);
        assert!((start - Pt3::new(0.0, 0.0, 0.0)).norm() < 1e-10);
        assert!((end - Pt3::new(0.0, 0.0, 10.0)).norm() < 1e-10);
    }

    #[test]
    fn sample_guide_curve_midpoint() {
        let guide = LoftGuideCurve {
            points: vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(10.0, 0.0, 10.0),
            ],
        };
        let mid = sample_guide_curve(&guide, 0.5);
        assert!((mid - Pt3::new(5.0, 0.0, 5.0)).norm() < 1e-10);
    }
}
