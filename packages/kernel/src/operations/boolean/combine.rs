//! Combine Bodies operation — feature-tree-level boolean that merges multiple BRep bodies.
//!
//! Wraps the low-level CSG primitives (`csg_union`, `csg_subtract`, `csg_intersect`)
//! into a single parameterised operation suitable for the feature tree.

use serde::{Deserialize, Serialize};

use crate::error::{KernelError, KernelResult};
use crate::topology::body::Body;
use crate::topology::BRep;

use super::csg::{csg_intersect, csg_subtract, csg_union};

/// Which boolean combination to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CombineOperation {
    /// Union (merge bodies)
    Add,
    /// Subtract tool from target
    Subtract,
    /// Intersection (keep only overlapping volume)
    Common,
}

/// Parameters for the Combine Bodies feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombineParams {
    pub operation: CombineOperation,
}

/// Combine two BRep bodies using the specified boolean operation.
///
/// * `target` — the main body to operate on.
/// * `tool`   — the secondary body (added, subtracted, or intersected).
///
/// Returns an error if either body is empty.
pub fn combine_bodies(
    target: &BRep,
    tool: &BRep,
    params: &CombineParams,
) -> KernelResult<BRep> {
    if matches!(target.body, Body::Empty) {
        return Err(KernelError::Operation {
            op: "combine_bodies".into(),
            detail: "Target body is empty".into(),
        });
    }
    if matches!(tool.body, Body::Empty) {
        return Err(KernelError::Operation {
            op: "combine_bodies".into(),
            detail: "Tool body is empty".into(),
        });
    }

    match params.operation {
        CombineOperation::Add => csg_union(target, tool),
        CombineOperation::Subtract => csg_subtract(target, tool),
        CombineOperation::Common => csg_intersect(target, tool),
    }
}

// ─── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Pt3, Vec3};
    use crate::topology::builders::{build_box_brep, extract_face_polygons, rebuild_brep_from_faces};

    /// Helper: build a box offset along X so it partially overlaps the origin box.
    fn offset_box(dx: f64) -> BRep {
        let base = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let polys = extract_face_polygons(&base).unwrap();
        let shifted: Vec<(Vec<Pt3>, Vec3)> = polys
            .into_iter()
            .map(|(pts, n)| {
                (
                    pts.into_iter()
                        .map(|p| Pt3::new(p.x + dx, p.y, p.z))
                        .collect(),
                    n,
                )
            })
            .collect();
        rebuild_brep_from_faces(&shifted).unwrap()
    }

    #[test]
    fn combine_add_unions_two_boxes() {
        let a = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let b = offset_box(5.0);
        let params = CombineParams {
            operation: CombineOperation::Add,
        };
        let result = combine_bodies(&a, &b, &params).unwrap();
        // Union of two overlapping boxes should produce faces
        assert!(
            result.faces.len() >= 6,
            "Union should produce at least 6 faces, got {}",
            result.faces.len()
        );
    }

    #[test]
    fn combine_subtract_removes_volume() {
        let a = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let b = build_box_brep(5.0, 5.0, 20.0).unwrap(); // tall narrow through centre
        let params = CombineParams {
            operation: CombineOperation::Subtract,
        };
        let result = combine_bodies(&a, &b, &params).unwrap();
        assert!(
            result.faces.len() > 6,
            "Subtract should produce more than 6 faces, got {}",
            result.faces.len()
        );
    }

    #[test]
    fn combine_common_keeps_intersection() {
        let a = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let b = offset_box(5.0);
        let params = CombineParams {
            operation: CombineOperation::Common,
        };
        let result = combine_bodies(&a, &b, &params).unwrap();
        assert!(
            result.faces.len() >= 6,
            "Intersection should produce at least 6 faces, got {}",
            result.faces.len()
        );
    }

    #[test]
    fn combine_empty_target_errors() {
        let empty = BRep::new();
        let b = build_box_brep(5.0, 5.0, 5.0).unwrap();
        let params = CombineParams {
            operation: CombineOperation::Add,
        };
        let result = combine_bodies(&empty, &b, &params);
        assert!(result.is_err(), "Should error on empty target body");
    }

    #[test]
    fn combine_empty_tool_errors() {
        let a = build_box_brep(5.0, 5.0, 5.0).unwrap();
        let empty = BRep::new();
        let params = CombineParams {
            operation: CombineOperation::Subtract,
        };
        let result = combine_bodies(&a, &empty, &params);
        assert!(result.is_err(), "Should error on empty tool body");
    }
}
