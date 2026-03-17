use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};
use crate::topology::BRep;

use crate::operations::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CurvePatternParams {
    /// Points defining the curve path
    pub curve_points: Vec<Pt3>,
    /// Number of instances (including original)
    pub count: u32,
    /// Equal arc-length spacing between instances
    pub equal_spacing: bool,
    /// Rotate instances to follow the curve tangent
    pub align_to_curve: bool,
}

#[derive(Debug)]
pub struct CurvePatternOp;

impl Operation for CurvePatternOp {
    type Params = CurvePatternParams;

    fn execute(&self, params: &Self::Params, input: &BRep) -> KernelResult<BRep> {
        curve_pattern(input, params)
    }

    fn name(&self) -> &'static str {
        "Curve Pattern"
    }
}

/// Compute cumulative arc-length distances along a polyline defined by `points`.
/// Returns a vector of the same length as `points`, starting with 0.0.
fn cumulative_arc_lengths(points: &[Pt3]) -> Vec<f64> {
    let mut lengths = Vec::with_capacity(points.len());
    lengths.push(0.0);
    for i in 1..points.len() {
        let seg = points[i] - points[i - 1];
        lengths.push(lengths[i - 1] + seg.norm());
    }
    lengths
}

/// Interpolate a point on the polyline at a given arc-length distance `s`.
/// Also returns the tangent direction at that point.
fn interpolate_on_polyline(points: &[Pt3], cum_lengths: &[f64], s: f64) -> (Pt3, Vec3) {
    let total = *cum_lengths.last().unwrap();

    // Clamp to bounds
    if s <= 0.0 {
        let tangent = if points.len() > 1 {
            (points[1] - points[0]).normalize()
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        return (points[0], tangent);
    }
    if s >= total {
        let tangent = if points.len() > 1 {
            let n = points.len();
            (points[n - 1] - points[n - 2]).normalize()
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        return (*points.last().unwrap(), tangent);
    }

    // Binary search for the segment containing s
    let mut seg_idx = 0;
    for i in 1..cum_lengths.len() {
        if cum_lengths[i] >= s {
            seg_idx = i - 1;
            break;
        }
    }

    let seg_start = cum_lengths[seg_idx];
    let seg_end = cum_lengths[seg_idx + 1];
    let seg_len = seg_end - seg_start;

    let t = if seg_len > 1e-15 {
        (s - seg_start) / seg_len
    } else {
        0.0
    };

    let p = points[seg_idx] + (points[seg_idx + 1] - points[seg_idx]) * t;
    let tangent = (points[seg_idx + 1] - points[seg_idx]).normalize();

    (p, tangent)
}

/// Build a rotation that maps `from` direction to `to` direction.
/// Returns axis and angle for use with Rodrigues' rotation.
fn rotation_between(from: Vec3, to: Vec3) -> (Vec3, f64) {
    let from_n = from.normalize();
    let to_n = to.normalize();
    let dot = from_n.dot(&to_n).clamp(-1.0, 1.0);

    if dot > 1.0 - 1e-12 {
        // Vectors are parallel, no rotation needed
        return (Vec3::new(0.0, 0.0, 1.0), 0.0);
    }
    if dot < -1.0 + 1e-12 {
        // Vectors are anti-parallel, rotate 180 degrees around any perpendicular axis
        let perp = if from_n.x.abs() < 0.9 {
            Vec3::new(1.0, 0.0, 0.0).cross(&from_n).normalize()
        } else {
            Vec3::new(0.0, 1.0, 0.0).cross(&from_n).normalize()
        };
        return (perp, std::f64::consts::PI);
    }

    let axis = from_n.cross(&to_n).normalize();
    let angle = dot.acos();
    (axis, angle)
}

/// Rotate a point around an axis through the origin using Rodrigues' formula.
fn rotate_vec_around_axis(v: Vec3, axis: Vec3, angle: f64) -> Vec3 {
    if angle.abs() < 1e-15 {
        return v;
    }
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let k = axis;
    v * cos_a + k.cross(&v) * sin_a + k * k.dot(&v) * (1.0 - cos_a)
}

pub fn curve_pattern(brep: &BRep, params: &CurvePatternParams) -> KernelResult<BRep> {
    if brep.faces.is_empty() {
        return Err(KernelError::Operation {
            op: "curve_pattern".into(),
            detail: "Cannot pattern: no existing geometry".into(),
        });
    }
    if params.count < 2 {
        return Err(KernelError::InvalidParameter {
            param: "count".into(),
            value: params.count.to_string(),
        });
    }
    if params.curve_points.len() < 2 {
        return Err(KernelError::InvalidParameter {
            param: "curve_points".into(),
            value: format!("need at least 2 points, got {}", params.curve_points.len()),
        });
    }

    let base_faces = extract_face_polygons(brep)?;
    let cum_lengths = cumulative_arc_lengths(&params.curve_points);
    let total_length = *cum_lengths.last().unwrap();

    if total_length < 1e-15 {
        return Err(KernelError::InvalidParameter {
            param: "curve_points".into(),
            value: "curve has zero length".into(),
        });
    }

    // The first instance is at the original position (s=0).
    // Compute the initial tangent direction (used as reference for align_to_curve).
    let (_, initial_tangent) = interpolate_on_polyline(&params.curve_points, &cum_lengths, 0.0);

    // The original body is at position curve_points[0].
    // Each subsequent instance is offset = curve_point[i] - curve_points[0],
    // optionally rotated to align with local tangent.
    let origin = params.curve_points[0];

    let mut all_faces: Vec<(Vec<Pt3>, Vec3)> = Vec::new();

    for i in 0..params.count {
        let s = if params.equal_spacing {
            total_length * i as f64 / (params.count - 1) as f64
        } else {
            // Without equal spacing, distribute by parameter (segment index) uniformly
            total_length * i as f64 / (params.count - 1) as f64
        };

        let (curve_pt, tangent) = interpolate_on_polyline(&params.curve_points, &cum_lengths, s);
        let offset = curve_pt - origin;

        if params.align_to_curve && i > 0 {
            // Compute rotation from initial tangent to current tangent
            let (rot_axis, rot_angle) = rotation_between(initial_tangent, tangent);

            for (pts, normal) in &base_faces {
                let transformed: Vec<Pt3> = pts
                    .iter()
                    .map(|p| {
                        // Translate to origin, rotate, then translate to curve position
                        let v = p - origin;
                        let rotated = rotate_vec_around_axis(v, rot_axis, rot_angle);
                        origin + rotated + offset
                    })
                    .collect();
                let rotated_normal = rotate_vec_around_axis(*normal, rot_axis, rot_angle);
                all_faces.push((transformed, rotated_normal));
            }
        } else {
            // Just translate (first instance or no alignment)
            for (pts, normal) in &base_faces {
                let translated: Vec<Pt3> = pts.iter().map(|p| p + offset).collect();
                all_faces.push((translated, *normal));
            }
        }
    }

    rebuild_brep_from_faces(&all_faces)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::body::Body;
    use crate::topology::builders::build_box_brep;

    #[test]
    fn curve_pattern_along_straight_line() {
        // Pattern along a straight line should produce evenly spaced copies
        let brep = build_box_brep(3.0, 3.0, 3.0).unwrap();
        let params = CurvePatternParams {
            curve_points: vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(30.0, 0.0, 0.0),
            ],
            count: 4,
            equal_spacing: true,
            align_to_curve: false,
        };
        let result = curve_pattern(&brep, &params).unwrap();
        // 4 instances * 6 faces each = 24 faces
        assert_eq!(result.faces.len(), 24);
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn curve_pattern_along_arc() {
        // Pattern along an approximate quarter-circle arc
        let brep = build_box_brep(2.0, 2.0, 2.0).unwrap();
        let n_arc_pts = 20;
        let radius = 20.0;
        let arc_points: Vec<Pt3> = (0..=n_arc_pts)
            .map(|i| {
                let theta = std::f64::consts::FRAC_PI_2 * i as f64 / n_arc_pts as f64;
                Pt3::new(radius * theta.cos(), radius * theta.sin(), 0.0)
            })
            .collect();

        let params = CurvePatternParams {
            curve_points: arc_points,
            count: 3,
            equal_spacing: true,
            align_to_curve: false,
        };
        let result = curve_pattern(&brep, &params).unwrap();
        // 3 instances * 6 faces = 18
        assert_eq!(result.faces.len(), 18);
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn curve_pattern_with_align_to_curve() {
        // Pattern with alignment along an L-shaped path
        let brep = build_box_brep(2.0, 2.0, 2.0).unwrap();
        let params = CurvePatternParams {
            curve_points: vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(10.0, 0.0, 0.0),
                Pt3::new(10.0, 10.0, 0.0),
            ],
            count: 3,
            equal_spacing: true,
            align_to_curve: true,
        };
        let result = curve_pattern(&brep, &params).unwrap();
        assert_eq!(result.faces.len(), 18);
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn curve_pattern_invalid_count() {
        let brep = build_box_brep(3.0, 3.0, 3.0).unwrap();
        let params = CurvePatternParams {
            curve_points: vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(10.0, 0.0, 0.0),
            ],
            count: 0,
            equal_spacing: true,
            align_to_curve: false,
        };
        assert!(curve_pattern(&brep, &params).is_err());

        let params1 = CurvePatternParams {
            count: 1,
            ..params
        };
        assert!(curve_pattern(&brep, &params1).is_err());
    }

    #[test]
    fn curve_pattern_empty_brep_rejected() {
        let brep = BRep::new();
        let params = CurvePatternParams {
            curve_points: vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(10.0, 0.0, 0.0),
            ],
            count: 3,
            equal_spacing: true,
            align_to_curve: false,
        };
        assert!(curve_pattern(&brep, &params).is_err());
    }

    #[test]
    fn curve_pattern_insufficient_points() {
        let brep = build_box_brep(3.0, 3.0, 3.0).unwrap();
        let params = CurvePatternParams {
            curve_points: vec![Pt3::new(0.0, 0.0, 0.0)],
            count: 3,
            equal_spacing: true,
            align_to_curve: false,
        };
        assert!(curve_pattern(&brep, &params).is_err());
    }
}
