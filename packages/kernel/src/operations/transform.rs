use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};
use crate::topology::BRep;

use crate::operations::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScaleBodyParams {
    /// Uniform scale factor (must be > 0).
    pub scale_factor: f64,
    /// Center point for scaling. Defaults to origin if None.
    pub center: Option<Pt3>,
    /// Optional non-uniform scale factors (x, y, z). Overrides `scale_factor` when present.
    pub non_uniform: Option<Vec3>,
    /// If true, keep the original body and union the scaled copy with it.
    pub copy: bool,
}

#[derive(Debug)]
pub struct ScaleBodyOp;

impl Operation for ScaleBodyOp {
    type Params = ScaleBodyParams;

    fn execute(&self, params: &Self::Params, input: &BRep) -> KernelResult<BRep> {
        scale_body(input, params)
    }

    fn name(&self) -> &'static str {
        "Scale Body"
    }
}

/// Validate scale factors, returning the effective (sx, sy, sz) triple.
fn validate_scale(params: &ScaleBodyParams) -> KernelResult<(f64, f64, f64)> {
    if let Some(nu) = &params.non_uniform {
        if nu.x <= 0.0 || nu.y <= 0.0 || nu.z <= 0.0 {
            return Err(KernelError::InvalidParameter {
                param: "non_uniform".into(),
                value: format!("({}, {}, {})", nu.x, nu.y, nu.z),
            });
        }
        Ok((nu.x, nu.y, nu.z))
    } else {
        if params.scale_factor <= 0.0 {
            return Err(KernelError::InvalidParameter {
                param: "scale_factor".into(),
                value: format!("{}", params.scale_factor),
            });
        }
        let s = params.scale_factor;
        Ok((s, s, s))
    }
}

/// Scale a point about the given center with scale factors (sx, sy, sz).
/// v' = center + scale * (v - center)
fn scale_point(p: &Pt3, center: &Pt3, sx: f64, sy: f64, sz: f64) -> Pt3 {
    Pt3::new(
        center.x + sx * (p.x - center.x),
        center.y + sy * (p.y - center.y),
        center.z + sz * (p.z - center.z),
    )
}

/// Scale a normal vector under non-uniform scaling.
/// Under scaling (sx, sy, sz), normals transform by the inverse-transpose:
/// n' = normalize(nx/sx, ny/sy, nz/sz)
fn scale_normal(n: &Vec3, sx: f64, sy: f64, sz: f64) -> Vec3 {
    let raw = Vec3::new(n.x / sx, n.y / sy, n.z / sz);
    let len = raw.norm();
    if len < 1e-15 {
        *n // degenerate — keep original
    } else {
        raw / len
    }
}

pub fn scale_body(brep: &BRep, params: &ScaleBodyParams) -> KernelResult<BRep> {
    if brep.faces.is_empty() {
        return Err(KernelError::Operation {
            op: "scale_body".into(),
            detail: "Cannot scale: no existing geometry".into(),
        });
    }

    let (sx, sy, sz) = validate_scale(params)?;
    let center = params.center.unwrap_or_else(|| Pt3::new(0.0, 0.0, 0.0));

    let base_faces = extract_face_polygons(brep)?;

    // Build scaled faces
    let scaled_faces: Vec<(Vec<Pt3>, Vec3)> = base_faces
        .iter()
        .map(|(pts, normal)| {
            let scaled_pts: Vec<Pt3> = pts
                .iter()
                .map(|p| scale_point(p, &center, sx, sy, sz))
                .collect();
            let scaled_normal = scale_normal(normal, sx, sy, sz);
            (scaled_pts, scaled_normal)
        })
        .collect();

    if params.copy {
        // Union: keep original faces plus scaled faces
        let mut all_faces: Vec<(Vec<Pt3>, Vec3)> = base_faces;
        all_faces.extend(scaled_faces);
        rebuild_brep_from_faces(&all_faces)
    } else {
        rebuild_brep_from_faces(&scaled_faces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;
    use crate::topology::body::Body;

    /// Helper: compute volume of a closed BRep via the divergence theorem.
    /// For each triangulated face, sum signed volumes of tetrahedra with origin.
    fn compute_volume(brep: &BRep) -> f64 {
        let faces = extract_face_polygons(brep).unwrap();
        let mut volume = 0.0;
        for (pts, _normal) in &faces {
            // Fan-triangulate from pts[0]
            if pts.len() < 3 {
                continue;
            }
            for i in 1..pts.len() - 1 {
                let a = pts[0];
                let b = pts[i];
                let c = pts[i + 1];
                // Signed volume of tetrahedron with origin
                volume += a.coords.dot(&b.coords.cross(&c.coords));
            }
        }
        (volume / 6.0).abs()
    }

    #[test]
    fn uniform_scale_2x_volume_8x() {
        let brep = build_box_brep(2.0, 3.0, 4.0).unwrap();
        let original_vol = compute_volume(&brep);
        assert!((original_vol - 24.0).abs() < 0.1);

        let params = ScaleBodyParams {
            scale_factor: 2.0,
            center: None,
            non_uniform: None,
            copy: false,
        };
        let result = scale_body(&brep, &params).unwrap();
        let scaled_vol = compute_volume(&result);
        assert!(
            (scaled_vol / original_vol - 8.0).abs() < 0.01,
            "2x uniform scale should give 8x volume: got {} / {} = {}",
            scaled_vol,
            original_vol,
            scaled_vol / original_vol
        );
    }

    #[test]
    fn uniform_scale_half_volume_eighth() {
        let brep = build_box_brep(4.0, 4.0, 4.0).unwrap();
        let original_vol = compute_volume(&brep);

        let params = ScaleBodyParams {
            scale_factor: 0.5,
            center: None,
            non_uniform: None,
            copy: false,
        };
        let result = scale_body(&brep, &params).unwrap();
        let scaled_vol = compute_volume(&result);
        assert!(
            (scaled_vol / original_vol - 0.125).abs() < 0.001,
            "0.5x uniform scale should give 0.125x volume: got {}",
            scaled_vol / original_vol
        );
    }

    #[test]
    fn scale_about_centroid_preserves_center() {
        let brep = build_box_brep(2.0, 2.0, 2.0).unwrap();
        // Box is 0..2 in each axis, centroid at (1,1,1)
        let center = Pt3::new(1.0, 1.0, 1.0);

        let params = ScaleBodyParams {
            scale_factor: 3.0,
            center: Some(center),
            non_uniform: None,
            copy: false,
        };
        let result = scale_body(&brep, &params).unwrap();
        let faces = extract_face_polygons(&result).unwrap();

        // Compute bounding box of scaled result
        let mut min = Pt3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Pt3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        for (pts, _) in &faces {
            for p in pts {
                min.x = min.x.min(p.x);
                min.y = min.y.min(p.y);
                min.z = min.z.min(p.z);
                max.x = max.x.max(p.x);
                max.y = max.y.max(p.y);
                max.z = max.z.max(p.z);
            }
        }
        // Centroid of result should still be at (1,1,1)
        let cx = (min.x + max.x) / 2.0;
        let cy = (min.y + max.y) / 2.0;
        let cz = (min.z + max.z) / 2.0;
        assert!((cx - 1.0).abs() < 0.01, "centroid x: {}", cx);
        assert!((cy - 1.0).abs() < 0.01, "centroid y: {}", cy);
        assert!((cz - 1.0).abs() < 0.01, "centroid z: {}", cz);
    }

    #[test]
    fn scale_about_origin_shifts_center() {
        let brep = build_box_brep(2.0, 2.0, 2.0).unwrap();
        // Box is 0..2 in each axis

        let params = ScaleBodyParams {
            scale_factor: 2.0,
            center: None, // origin
            non_uniform: None,
            copy: false,
        };
        let result = scale_body(&brep, &params).unwrap();
        let faces = extract_face_polygons(&result).unwrap();

        let mut max = Pt3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        for (pts, _) in &faces {
            for p in pts {
                max.x = max.x.max(p.x);
                max.y = max.y.max(p.y);
                max.z = max.z.max(p.z);
            }
        }
        // Should now extend to 4.0 in each axis
        assert!((max.x - 4.0).abs() < 0.01);
        assert!((max.y - 4.0).abs() < 0.01);
        assert!((max.z - 4.0).abs() < 0.01);
    }

    #[test]
    fn non_uniform_scale_stretches_one_axis() {
        let brep = build_box_brep(2.0, 2.0, 2.0).unwrap();
        let original_vol = compute_volume(&brep);

        let params = ScaleBodyParams {
            scale_factor: 1.0, // ignored when non_uniform is set
            center: None,
            non_uniform: Some(Vec3::new(1.0, 1.0, 3.0)),
            copy: false,
        };
        let result = scale_body(&brep, &params).unwrap();
        let scaled_vol = compute_volume(&result);
        // Volume should be 3x (only z stretched)
        assert!(
            (scaled_vol / original_vol - 3.0).abs() < 0.01,
            "non-uniform (1,1,3) should give 3x volume: got {}",
            scaled_vol / original_vol
        );
    }

    #[test]
    fn scale_preserves_watertightness() {
        let brep = build_box_brep(5.0, 5.0, 5.0).unwrap();
        let params = ScaleBodyParams {
            scale_factor: 2.5,
            center: None,
            non_uniform: None,
            copy: false,
        };
        let result = scale_body(&brep, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
        // Same number of faces as original (scaling preserves topology)
        assert_eq!(result.faces.len(), 6);
        // Volume should be 2.5^3 * original = 15.625 * 125 = 1953.125
        let vol = compute_volume(&result);
        assert!((vol - 5.0 * 5.0 * 5.0 * 2.5_f64.powi(3)).abs() < 1.0);
    }

    #[test]
    fn invalid_scale_factor_zero() {
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        let params = ScaleBodyParams {
            scale_factor: 0.0,
            center: None,
            non_uniform: None,
            copy: false,
        };
        assert!(scale_body(&brep, &params).is_err());
    }

    #[test]
    fn invalid_scale_factor_negative() {
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        let params = ScaleBodyParams {
            scale_factor: -2.0,
            center: None,
            non_uniform: None,
            copy: false,
        };
        assert!(scale_body(&brep, &params).is_err());
    }

    #[test]
    fn invalid_non_uniform_negative() {
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        let params = ScaleBodyParams {
            scale_factor: 1.0,
            center: None,
            non_uniform: Some(Vec3::new(1.0, -1.0, 1.0)),
            copy: false,
        };
        assert!(scale_body(&brep, &params).is_err());
    }

    #[test]
    fn copy_mode_includes_original_and_scaled() {
        let brep = build_box_brep(2.0, 2.0, 2.0).unwrap();
        assert_eq!(brep.faces.len(), 6);

        let params = ScaleBodyParams {
            scale_factor: 3.0,
            center: Some(Pt3::new(100.0, 0.0, 0.0)), // far away so no coincident faces
            non_uniform: None,
            copy: true,
        };
        let result = scale_body(&brep, &params).unwrap();
        // 6 original + 6 scaled = 12 faces
        assert_eq!(result.faces.len(), 12);
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn scale_empty_brep_rejected() {
        let brep = BRep::new();
        let params = ScaleBodyParams {
            scale_factor: 2.0,
            center: None,
            non_uniform: None,
            copy: false,
        };
        assert!(scale_body(&brep, &params).is_err());
    }
}
