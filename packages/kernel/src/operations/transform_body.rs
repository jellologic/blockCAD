use crate::error::{KernelError, KernelResult};
use crate::geometry::transform::{
    compose, rotation_axis_angle, transform_normal, transform_point, translation,
};
use crate::geometry::{Mat4, Pt3, Vec3};
use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};
use crate::topology::BRep;

use super::traits::Operation;

/// The kind of spatial transformation to apply.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TransformKind {
    Translate {
        delta: Vec3,
    },
    Rotate {
        axis: Vec3,
        angle: f64,
        center: Pt3,
    },
    TranslateRotate {
        delta: Vec3,
        axis: Vec3,
        angle: f64,
        center: Pt3,
    },
}

/// Parameters for the Move/Copy Body operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MoveBodyParams {
    pub transform: TransformKind,
    /// If true, create a copy (union of original + transformed).
    /// If false, transform the body in place.
    #[serde(default)]
    pub copy: bool,
}

#[derive(Debug)]
pub struct MoveBodyOp;

impl Operation for MoveBodyOp {
    type Params = MoveBodyParams;

    fn execute(&self, params: &Self::Params, input: &BRep) -> KernelResult<BRep> {
        move_body(input, params)
    }

    fn name(&self) -> &'static str {
        "MoveBody"
    }
}

/// Build a 4x4 transformation matrix from a TransformKind.
pub fn build_transform_matrix(kind: &TransformKind) -> Mat4 {
    match kind {
        TransformKind::Translate { delta } => translation(delta.x, delta.y, delta.z),
        TransformKind::Rotate {
            axis,
            angle,
            center,
        } => {
            // Translate center to origin, rotate, translate back
            let to_origin = translation(-center.x, -center.y, -center.z);
            let rot = rotation_axis_angle(axis, *angle);
            let from_origin = translation(center.x, center.y, center.z);
            compose(&from_origin, &compose(&rot, &to_origin))
        }
        TransformKind::TranslateRotate {
            delta,
            axis,
            angle,
            center,
        } => {
            // Rotate about center, then translate
            let to_origin = translation(-center.x, -center.y, -center.z);
            let rot = rotation_axis_angle(axis, *angle);
            let from_origin = translation(center.x, center.y, center.z);
            let rotation_about_center = compose(&from_origin, &compose(&rot, &to_origin));
            let trans = translation(delta.x, delta.y, delta.z);
            compose(&trans, &rotation_about_center)
        }
    }
}

/// Apply a transformation matrix to a set of face polygons.
fn transform_faces(
    faces: &[(Vec<Pt3>, Vec3)],
    matrix: &Mat4,
) -> Vec<(Vec<Pt3>, Vec3)> {
    faces
        .iter()
        .map(|(pts, normal)| {
            let new_pts: Vec<Pt3> = pts.iter().map(|p| transform_point(matrix, p)).collect();
            let new_normal = transform_normal(matrix, normal);
            (new_pts, new_normal)
        })
        .collect()
}

/// Move or copy a body by applying a spatial transformation.
///
/// - If `params.copy` is false, transforms all geometry in place.
/// - If `params.copy` is true, creates a union of the original body
///   and the transformed copy.
pub fn move_body(brep: &BRep, params: &MoveBodyParams) -> KernelResult<BRep> {
    if brep.faces.is_empty() {
        return Err(KernelError::Operation {
            op: "move_body".into(),
            detail: "Cannot move/copy: no existing geometry".into(),
        });
    }

    let base_faces = extract_face_polygons(brep)?;
    let matrix = build_transform_matrix(&params.transform);
    let transformed_faces = transform_faces(&base_faces, &matrix);

    if params.copy {
        // Union: combine original faces and transformed faces
        let mut all_faces = base_faces;
        all_faces.extend(transformed_faces);
        rebuild_brep_from_faces(&all_faces)
    } else {
        // Move in place: just use the transformed faces
        rebuild_brep_from_faces(&transformed_faces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;
    use crate::topology::body::Body;

    fn make_box() -> BRep {
        build_box_brep(10.0, 10.0, 10.0).unwrap()
    }

    #[test]
    fn translate_body_shifts_vertices() {
        let brep = make_box();
        let params = MoveBodyParams {
            transform: TransformKind::Translate {
                delta: Vec3::new(10.0, 0.0, 0.0),
            },
            copy: false,
        };
        let result = move_body(&brep, &params).unwrap();

        // Same face count
        assert_eq!(result.faces.len(), 6);
        assert!(matches!(result.body, Body::Solid(_)));

        // All X coordinates should be shifted by 10
        let x_min = result
            .vertices
            .iter()
            .map(|(_, v)| v.point.x)
            .fold(f64::INFINITY, f64::min);
        let x_max = result
            .vertices
            .iter()
            .map(|(_, v)| v.point.x)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((x_min - 10.0).abs() < 1e-6, "x_min should be 10, got {}", x_min);
        assert!((x_max - 20.0).abs() < 1e-6, "x_max should be 20, got {}", x_max);
    }

    #[test]
    fn rotate_body_90_about_z() {
        let brep = make_box();
        let params = MoveBodyParams {
            transform: TransformKind::Rotate {
                axis: Vec3::new(0.0, 0.0, 1.0),
                angle: std::f64::consts::FRAC_PI_2,
                center: Pt3::new(0.0, 0.0, 0.0),
            },
            copy: false,
        };
        let result = move_body(&brep, &params).unwrap();

        assert_eq!(result.faces.len(), 6);
        assert!(matches!(result.body, Body::Solid(_)));

        // Original box: X in [0, 10], Y in [0, 10]
        // After 90° rotation about Z at origin: X in [-10, 0], Y in [0, 10]
        let x_min = result
            .vertices
            .iter()
            .map(|(_, v)| v.point.x)
            .fold(f64::INFINITY, f64::min);
        let x_max = result
            .vertices
            .iter()
            .map(|(_, v)| v.point.x)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((x_min - (-10.0)).abs() < 1e-6, "x_min should be -10, got {}", x_min);
        assert!(x_max.abs() < 1e-6, "x_max should be 0, got {}", x_max);
    }

    #[test]
    fn copy_body_doubles_faces() {
        let brep = make_box();
        let params = MoveBodyParams {
            transform: TransformKind::Translate {
                delta: Vec3::new(20.0, 0.0, 0.0),
            },
            copy: true,
        };
        let result = move_body(&brep, &params).unwrap();

        // Copy should produce 12 faces (6 original + 6 copy, no overlap)
        assert_eq!(result.faces.len(), 12);
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn move_preserves_face_count() {
        let brep = make_box();
        let original_face_count = brep.faces.len();
        let params = MoveBodyParams {
            transform: TransformKind::Translate {
                delta: Vec3::new(5.0, 5.0, 5.0),
            },
            copy: false,
        };
        let result = move_body(&brep, &params).unwrap();
        assert_eq!(result.faces.len(), original_face_count);
    }

    #[test]
    fn identity_transform_no_change() {
        let brep = make_box();
        let params = MoveBodyParams {
            transform: TransformKind::Translate {
                delta: Vec3::new(0.0, 0.0, 0.0),
            },
            copy: false,
        };
        let result = move_body(&brep, &params).unwrap();

        assert_eq!(result.faces.len(), 6);

        // Vertices should be in the same positions
        let orig_xs: Vec<f64> = {
            let mut xs: Vec<f64> = brep.vertices.iter().map(|(_, v)| v.point.x).collect();
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            xs
        };
        let result_xs: Vec<f64> = {
            let mut xs: Vec<f64> = result.vertices.iter().map(|(_, v)| v.point.x).collect();
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            xs
        };
        assert_eq!(orig_xs.len(), result_xs.len());
        for (a, b) in orig_xs.iter().zip(result_xs.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn move_empty_brep_rejected() {
        let brep = BRep::new();
        let params = MoveBodyParams {
            transform: TransformKind::Translate {
                delta: Vec3::new(1.0, 0.0, 0.0),
            },
            copy: false,
        };
        assert!(move_body(&brep, &params).is_err());
    }

    #[test]
    fn translate_rotate_combined() {
        let brep = make_box();
        let params = MoveBodyParams {
            transform: TransformKind::TranslateRotate {
                delta: Vec3::new(100.0, 0.0, 0.0),
                axis: Vec3::new(0.0, 0.0, 1.0),
                angle: std::f64::consts::FRAC_PI_2,
                center: Pt3::new(0.0, 0.0, 0.0),
            },
            copy: false,
        };
        let result = move_body(&brep, &params).unwrap();
        assert_eq!(result.faces.len(), 6);
        assert!(matches!(result.body, Body::Solid(_)));

        // After rotate 90° Z at origin: X in [-10, 0], then translate +100 X: X in [90, 100]
        let x_min = result
            .vertices
            .iter()
            .map(|(_, v)| v.point.x)
            .fold(f64::INFINITY, f64::min);
        let x_max = result
            .vertices
            .iter()
            .map(|(_, v)| v.point.x)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((x_min - 90.0).abs() < 1e-6, "x_min should be 90, got {}", x_min);
        assert!((x_max - 100.0).abs() < 1e-6, "x_max should be 100, got {}", x_max);
    }

    #[test]
    fn rotate_about_non_origin_center() {
        let brep = make_box();
        // Rotate 180° about Z axis at (5, 5, 0) — center of box's XY footprint
        let params = MoveBodyParams {
            transform: TransformKind::Rotate {
                axis: Vec3::new(0.0, 0.0, 1.0),
                angle: std::f64::consts::PI,
                center: Pt3::new(5.0, 5.0, 0.0),
            },
            copy: false,
        };
        let result = move_body(&brep, &params).unwrap();
        assert_eq!(result.faces.len(), 6);

        // 180° rotation about (5,5) maps (0,0) -> (10,10) and (10,10) -> (0,0)
        // So the bounding box should still be [0,10] x [0,10]
        let x_min = result.vertices.iter().map(|(_, v)| v.point.x).fold(f64::INFINITY, f64::min);
        let x_max = result.vertices.iter().map(|(_, v)| v.point.x).fold(f64::NEG_INFINITY, f64::max);
        assert!(x_min.abs() < 1e-6, "x_min should be 0, got {}", x_min);
        assert!((x_max - 10.0).abs() < 1e-6, "x_max should be 10, got {}", x_max);
    }
}
