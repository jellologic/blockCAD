use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::operations::extrude::ExtrudeProfile;
use crate::topology::BRep;
use crate::topology::builders::build_brep_from_rings;

use super::traits::Operation;

const DEFAULT_SLICES_PER_SPAN: usize = 10;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoftParams {
    /// Number of intermediate slices between each profile pair.
    #[serde(default = "default_slices")]
    pub slices_per_span: usize,
    /// Whether the loft is closed (last profile connects to first).
    #[serde(default)]
    pub closed: bool,
}

fn default_slices() -> usize { DEFAULT_SLICES_PER_SPAN }

impl Default for LoftParams {
    fn default() -> Self {
        Self { slices_per_span: DEFAULT_SLICES_PER_SPAN, closed: false }
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

/// Loft between multiple profiles, creating a blended solid.
///
/// Algorithm:
/// 1. Resample all profiles to a common vertex count
/// 2. Align vertex orderings to minimize distance
/// 3. Linearly interpolate between adjacent profiles
/// 4. Connect rings with quad faces
/// 5. Cap start/end if not closed
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

    // Generate all rings by interpolating between adjacent profiles
    let mut rings: Vec<Vec<Pt3>> = Vec::new();
    let n_profiles = resampled.len();
    let n_spans = if params.closed { n_profiles } else { n_profiles - 1 };

    for span in 0..n_spans {
        let next = if params.closed { (span + 1) % n_profiles } else { span + 1 };
        let profile_a = &resampled[span];
        let profile_b = &resampled[next];

        let n_slices = params.slices_per_span;
        for s in 0..n_slices {
            let t = s as f64 / n_slices as f64;
            let ring: Vec<Pt3> = profile_a.iter().zip(profile_b.iter()).map(|(a, b)| {
                Pt3::new(
                    a.x + (b.x - a.x) * t,
                    a.y + (b.y - a.y) * t,
                    a.z + (b.z - a.z) * t,
                )
            }).collect();
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
}
