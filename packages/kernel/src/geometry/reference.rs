//! Reference geometry types — construction axes, points, and coordinate systems.

use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::topology::BRep;

/// A construction axis defined by an origin point and direction vector.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReferenceAxis {
    pub origin: Pt3,
    pub direction: Vec3,
}

/// A construction point in 3D space.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReferencePoint {
    pub position: Pt3,
}

/// A local coordinate system defined by origin and three orthonormal axes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoordinateSystem {
    pub origin: Pt3,
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub z_axis: Vec3,
}

impl ReferenceAxis {
    /// Create a reference axis passing through two points.
    /// Direction goes from `p1` toward `p2`.
    pub fn from_two_points(p1: Pt3, p2: Pt3) -> KernelResult<Self> {
        let dir = p2 - p1;
        let len = dir.norm();
        if len < 1e-12 {
            return Err(KernelError::InvalidParameter {
                param: "points".into(),
                value: "Two points are coincident — cannot define an axis".into(),
            });
        }
        Ok(Self {
            origin: p1,
            direction: dir / len,
        })
    }

    /// Create a reference axis from a BRep edge (straight-line edges only).
    /// Uses the edge's start and end vertex positions.
    pub fn from_edge(brep: &BRep, edge_index: usize) -> KernelResult<Self> {
        let (_, edge) = brep.edges.iter().nth(edge_index).ok_or_else(|| {
            KernelError::NotFound(format!("Edge index {}", edge_index))
        })?;

        let start_pt = brep.vertices.get(edge.start)?.point;
        let end_pt = brep.vertices.get(edge.end)?.point;

        Self::from_two_points(start_pt, end_pt)
    }
}

impl CoordinateSystem {
    /// Create a coordinate system from three points:
    /// - `origin`: the coordinate system origin
    /// - `x_point`: a point along the desired X axis
    /// - `xy_point`: a point in the desired XY plane (used to derive Y axis)
    pub fn from_three_points(origin: Pt3, x_point: Pt3, xy_point: Pt3) -> KernelResult<Self> {
        let x_dir = x_point - origin;
        let x_len = x_dir.norm();
        if x_len < 1e-12 {
            return Err(KernelError::InvalidParameter {
                param: "x_point".into(),
                value: "x_point is coincident with origin".into(),
            });
        }
        let x_axis = x_dir / x_len;

        let xy_dir = xy_point - origin;
        let z_axis = x_axis.cross(&xy_dir);
        let z_len = z_axis.norm();
        if z_len < 1e-12 {
            return Err(KernelError::InvalidParameter {
                param: "xy_point".into(),
                value: "Three points are collinear — cannot define a coordinate system".into(),
            });
        }
        let z_axis = z_axis / z_len;
        let y_axis = z_axis.cross(&x_axis).normalize();

        Ok(Self {
            origin,
            x_axis,
            y_axis,
            z_axis,
        })
    }

    /// The identity (world) coordinate system at the origin.
    pub fn identity() -> Self {
        Self {
            origin: Pt3::new(0.0, 0.0, 0.0),
            x_axis: Vec3::new(1.0, 0.0, 0.0),
            y_axis: Vec3::new(0.0, 1.0, 0.0),
            z_axis: Vec3::new(0.0, 0.0, 1.0),
        }
    }

    /// Check whether the axes form an orthonormal basis (within tolerance).
    pub fn is_orthonormal(&self, tol: f64) -> bool {
        let x_unit = (self.x_axis.norm() - 1.0).abs() < tol;
        let y_unit = (self.y_axis.norm() - 1.0).abs() < tol;
        let z_unit = (self.z_axis.norm() - 1.0).abs() < tol;
        let xy_ortho = self.x_axis.dot(&self.y_axis).abs() < tol;
        let xz_ortho = self.x_axis.dot(&self.z_axis).abs() < tol;
        let yz_ortho = self.y_axis.dot(&self.z_axis).abs() < tol;
        x_unit && y_unit && z_unit && xy_ortho && xz_ortho && yz_ortho
    }
}

/// Parameters for a reference axis feature.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceAxisKind {
    /// Axis through two explicit points.
    TwoPoints { p1: [f64; 3], p2: [f64; 3] },
    /// Axis along a BRep edge.
    Edge { edge_index: usize },
    /// Axis with explicit origin and direction.
    Explicit { origin: [f64; 3], direction: [f64; 3] },
}

/// Parameters for a reference axis feature.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReferenceAxisParams {
    pub kind: ReferenceAxisKind,
}

/// Parameters for a reference point feature.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReferencePointParams {
    pub position: [f64; 3],
}

/// Parameters for a coordinate system feature.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateSystemKind {
    /// Defined by three points (origin, x-point, xy-point).
    ThreePoints {
        origin: [f64; 3],
        x_point: [f64; 3],
        xy_point: [f64; 3],
    },
    /// The identity (world) coordinate system.
    Identity,
}

/// Parameters for a coordinate system feature.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoordinateSystemParams {
    pub kind: CoordinateSystemKind,
}

/// Compute a reference axis from its parameters.
pub fn compute_reference_axis(
    kind: &ReferenceAxisKind,
    brep: Option<&BRep>,
) -> KernelResult<ReferenceAxis> {
    match kind {
        ReferenceAxisKind::TwoPoints { p1, p2 } => {
            let pt1 = Pt3::new(p1[0], p1[1], p1[2]);
            let pt2 = Pt3::new(p2[0], p2[1], p2[2]);
            ReferenceAxis::from_two_points(pt1, pt2)
        }
        ReferenceAxisKind::Edge { edge_index } => {
            let brep = brep.ok_or_else(|| KernelError::InvalidParameter {
                param: "brep".into(),
                value: "Edge axis requires existing geometry".into(),
            })?;
            ReferenceAxis::from_edge(brep, *edge_index)
        }
        ReferenceAxisKind::Explicit { origin, direction } => {
            let dir = Vec3::new(direction[0], direction[1], direction[2]);
            let len = dir.norm();
            if len < 1e-12 {
                return Err(KernelError::InvalidParameter {
                    param: "direction".into(),
                    value: "Direction vector is zero-length".into(),
                });
            }
            Ok(ReferenceAxis {
                origin: Pt3::new(origin[0], origin[1], origin[2]),
                direction: dir / len,
            })
        }
    }
}

/// Compute a reference point from its parameters.
pub fn compute_reference_point(params: &ReferencePointParams) -> ReferencePoint {
    ReferencePoint {
        position: Pt3::new(params.position[0], params.position[1], params.position[2]),
    }
}

/// Compute a coordinate system from its parameters.
pub fn compute_coordinate_system(
    kind: &CoordinateSystemKind,
) -> KernelResult<CoordinateSystem> {
    match kind {
        CoordinateSystemKind::ThreePoints {
            origin,
            x_point,
            xy_point,
        } => {
            let o = Pt3::new(origin[0], origin[1], origin[2]);
            let x = Pt3::new(x_point[0], x_point[1], x_point[2]);
            let xy = Pt3::new(xy_point[0], xy_point[1], xy_point[2]);
            CoordinateSystem::from_three_points(o, x, xy)
        }
        CoordinateSystemKind::Identity => Ok(CoordinateSystem::identity()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_from_two_points() {
        let axis = ReferenceAxis::from_two_points(
            Pt3::new(0.0, 0.0, 0.0),
            Pt3::new(0.0, 0.0, 10.0),
        )
        .unwrap();
        assert!((axis.direction - Vec3::new(0.0, 0.0, 1.0)).norm() < 1e-9);
        assert!((axis.origin - Pt3::new(0.0, 0.0, 0.0)).norm() < 1e-9);
    }

    #[test]
    fn axis_from_two_points_coincident_fails() {
        let result = ReferenceAxis::from_two_points(
            Pt3::new(1.0, 2.0, 3.0),
            Pt3::new(1.0, 2.0, 3.0),
        );
        assert!(result.is_err());
    }

    #[test]
    fn axis_normalizes_direction() {
        let axis = ReferenceAxis::from_two_points(
            Pt3::new(0.0, 0.0, 0.0),
            Pt3::new(0.0, 0.0, 50.0),
        )
        .unwrap();
        assert!(
            (axis.direction.norm() - 1.0).abs() < 1e-9,
            "Direction should be unit length"
        );
    }

    #[test]
    fn coordinate_system_from_three_points() {
        let cs = CoordinateSystem::from_three_points(
            Pt3::new(0.0, 0.0, 0.0),
            Pt3::new(1.0, 0.0, 0.0),
            Pt3::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        assert!((cs.x_axis - Vec3::new(1.0, 0.0, 0.0)).norm() < 1e-9);
        assert!((cs.y_axis - Vec3::new(0.0, 1.0, 0.0)).norm() < 1e-9);
        assert!((cs.z_axis - Vec3::new(0.0, 0.0, 1.0)).norm() < 1e-9);
    }

    #[test]
    fn coordinate_system_orthogonality() {
        let cs = CoordinateSystem::from_three_points(
            Pt3::new(1.0, 2.0, 3.0),
            Pt3::new(4.0, 2.0, 3.0),
            Pt3::new(1.0, 5.0, 6.0),
        )
        .unwrap();
        assert!(cs.is_orthonormal(1e-9), "Axes must be orthonormal");
    }

    #[test]
    fn coordinate_system_collinear_fails() {
        let result = CoordinateSystem::from_three_points(
            Pt3::new(0.0, 0.0, 0.0),
            Pt3::new(1.0, 0.0, 0.0),
            Pt3::new(2.0, 0.0, 0.0),
        );
        assert!(result.is_err());
    }

    #[test]
    fn reference_point_creation() {
        let pt = ReferencePoint {
            position: Pt3::new(3.0, 4.0, 5.0),
        };
        assert!((pt.position.x - 3.0).abs() < 1e-9);
        assert!((pt.position.y - 4.0).abs() < 1e-9);
        assert!((pt.position.z - 5.0).abs() < 1e-9);
    }

    #[test]
    fn identity_coordinate_system() {
        let cs = CoordinateSystem::identity();
        assert!((cs.origin - Pt3::new(0.0, 0.0, 0.0)).norm() < 1e-9);
        assert!((cs.x_axis - Vec3::new(1.0, 0.0, 0.0)).norm() < 1e-9);
        assert!((cs.y_axis - Vec3::new(0.0, 1.0, 0.0)).norm() < 1e-9);
        assert!((cs.z_axis - Vec3::new(0.0, 0.0, 1.0)).norm() < 1e-9);
        assert!(cs.is_orthonormal(1e-12));
    }

    #[test]
    fn axis_from_edge() {
        let brep = crate::topology::builders::build_box_brep(10.0, 5.0, 3.0).unwrap();
        let axis = ReferenceAxis::from_edge(&brep, 0).unwrap();
        // The axis should be unit-length and well-defined
        assert!(
            (axis.direction.norm() - 1.0).abs() < 1e-9,
            "Edge axis direction should be normalized"
        );
    }

    #[test]
    fn compute_reference_axis_explicit() {
        let axis = compute_reference_axis(
            &ReferenceAxisKind::Explicit {
                origin: [1.0, 2.0, 3.0],
                direction: [0.0, 0.0, 5.0],
            },
            None,
        )
        .unwrap();
        assert!((axis.direction - Vec3::new(0.0, 0.0, 1.0)).norm() < 1e-9);
        assert!((axis.origin.x - 1.0).abs() < 1e-9);
    }

    #[test]
    fn compute_reference_point_works() {
        let pt = compute_reference_point(&ReferencePointParams {
            position: [7.0, 8.0, 9.0],
        });
        assert!((pt.position.x - 7.0).abs() < 1e-9);
        assert!((pt.position.y - 8.0).abs() < 1e-9);
        assert!((pt.position.z - 9.0).abs() < 1e-9);
    }

    #[test]
    fn compute_coordinate_system_identity() {
        let cs = compute_coordinate_system(&CoordinateSystemKind::Identity).unwrap();
        assert!(cs.is_orthonormal(1e-12));
        assert!((cs.origin - Pt3::new(0.0, 0.0, 0.0)).norm() < 1e-9);
    }
}
