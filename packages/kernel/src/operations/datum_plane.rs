//! Datum plane (reference geometry) — creates construction planes for sketch placement.

use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::geometry::surface::plane::Plane;
use crate::topology::BRep;
use crate::topology::edge::Orientation;

/// How the datum plane is defined.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatumPlaneKind {
    /// Parallel to base plane, offset by distance along normal.
    Offset { distance: f64 },
    /// Rotated from base plane around an axis by an angle (radians).
    Angle { axis: [f64; 3], angle: f64 },
    /// Through 3 arbitrary points.
    ThreePoint { p1: [f64; 3], p2: [f64; 3], p3: [f64; 3] },
    /// On an existing BRep face (uses face centroid + surface normal).
    FacePlane { face_index: usize },
}

/// Parameters for a datum plane feature.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatumPlaneParams {
    pub kind: DatumPlaneKind,
    /// Base plane index in the datum_planes registry (for offset/angle).
    /// None = use standard XY plane as base.
    #[serde(default)]
    pub base_plane_index: Option<usize>,
}

/// Compute a datum plane from its definition.
pub fn compute_datum_plane(
    kind: &DatumPlaneKind,
    base: Option<&Plane>,
    brep: Option<&BRep>,
) -> KernelResult<Plane> {
    match kind {
        DatumPlaneKind::Offset { distance } => {
            let base = base.ok_or_else(|| KernelError::InvalidParameter {
                param: "base_plane".into(),
                value: "Offset plane requires a base plane".into(),
            })?;
            Ok(Plane {
                origin: base.origin + base.normal * *distance,
                normal: base.normal,
                u_axis: base.u_axis,
                v_axis: base.v_axis,
            })
        }

        DatumPlaneKind::Angle { axis, angle } => {
            let base = base.ok_or_else(|| KernelError::InvalidParameter {
                param: "base_plane".into(),
                value: "Angle plane requires a base plane".into(),
            })?;
            let axis_vec = Vec3::new(axis[0], axis[1], axis[2]).normalize();
            let rot = crate::geometry::transform::rotation_axis_angle(&axis_vec, *angle);
            let new_normal = crate::geometry::transform::transform_normal(&rot, &base.normal);
            let new_u = crate::geometry::transform::transform_normal(&rot, &base.u_axis);
            let new_v = new_normal.cross(&new_u).normalize();
            Ok(Plane {
                origin: base.origin,
                normal: new_normal,
                u_axis: new_u,
                v_axis: new_v,
            })
        }

        DatumPlaneKind::ThreePoint { p1, p2, p3 } => {
            let pt1 = Pt3::new(p1[0], p1[1], p1[2]);
            let pt2 = Pt3::new(p2[0], p2[1], p2[2]);
            let pt3 = Pt3::new(p3[0], p3[1], p3[2]);

            let e1 = pt2 - pt1;
            let e2 = pt3 - pt1;
            let normal = e1.cross(&e2);
            let len = normal.norm();
            if len < 1e-12 {
                return Err(KernelError::InvalidParameter {
                    param: "points".into(),
                    value: "Three points are collinear — cannot define a plane".into(),
                });
            }
            let normal = normal / len;
            let u_axis = e1.normalize();
            let v_axis = normal.cross(&u_axis).normalize();

            Ok(Plane { origin: pt1, normal, u_axis, v_axis })
        }

        DatumPlaneKind::FacePlane { face_index } => {
            let brep = brep.ok_or_else(|| KernelError::InvalidParameter {
                param: "brep".into(),
                value: "Face plane requires existing geometry".into(),
            })?;

            let (_, face) = brep.faces.iter().nth(*face_index).ok_or_else(|| {
                KernelError::NotFound(format!("Face index {}", face_index))
            })?;

            let surf_idx = face.surface_index.ok_or_else(|| {
                KernelError::Topology("Face has no surface".into())
            })?;
            let normal = brep.surfaces[surf_idx].normal_at(0.0, 0.0)?;

            // Compute face centroid from loop vertices
            let loop_id = face.outer_loop.ok_or_else(|| {
                KernelError::Topology("Face has no outer loop".into())
            })?;
            let loop_ = brep.loops.get(loop_id)?;

            let mut sum = Vec3::new(0.0, 0.0, 0.0);
            let mut count = 0;
            for &coedge_id in &loop_.coedges {
                let coedge = brep.coedges.get(coedge_id)?;
                let edge = brep.edges.get(coedge.edge)?;
                let start_vid = match coedge.orientation {
                    Orientation::Forward => edge.start,
                    Orientation::Reversed => edge.end,
                };
                let vertex = brep.vertices.get(start_vid)?;
                sum += Vec3::new(vertex.point.x, vertex.point.y, vertex.point.z);
                count += 1;
            }

            let origin = if count > 0 {
                Pt3::new(sum.x / count as f64, sum.y / count as f64, sum.z / count as f64)
            } else {
                Pt3::origin()
            };

            let u_axis = if count > 0 {
                let first_coedge = brep.coedges.get(loop_.coedges[0])?;
                let first_edge = brep.edges.get(first_coedge.edge)?;
                let (start, end) = match first_coedge.orientation {
                    Orientation::Forward => (first_edge.start, first_edge.end),
                    Orientation::Reversed => (first_edge.end, first_edge.start),
                };
                let p0 = brep.vertices.get(start)?.point;
                let p1 = brep.vertices.get(end)?.point;
                (p1 - p0).normalize()
            } else {
                Vec3::new(1.0, 0.0, 0.0)
            };
            let v_axis = normal.cross(&u_axis).normalize();

            Ok(Plane { origin, normal, u_axis, v_axis })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn front_plane() -> Plane {
        Plane {
            origin: Pt3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            u_axis: Vec3::new(1.0, 0.0, 0.0),
            v_axis: Vec3::new(0.0, 1.0, 0.0),
        }
    }

    #[test]
    fn offset_plane_moves_origin() {
        let base = front_plane();
        let result = compute_datum_plane(
            &DatumPlaneKind::Offset { distance: 10.0 },
            Some(&base), None,
        ).unwrap();
        assert!((result.origin.z - 10.0).abs() < 1e-9);
        assert!((result.normal - base.normal).norm() < 1e-9, "Normal should be unchanged");
    }

    #[test]
    fn offset_negative_distance() {
        let base = front_plane();
        let result = compute_datum_plane(
            &DatumPlaneKind::Offset { distance: -5.0 },
            Some(&base), None,
        ).unwrap();
        assert!((result.origin.z - (-5.0)).abs() < 1e-9);
    }

    #[test]
    fn angle_plane_rotates_normal() {
        let base = front_plane(); // normal = +Z
        let result = compute_datum_plane(
            &DatumPlaneKind::Angle {
                axis: [1.0, 0.0, 0.0], // rotate around X
                angle: std::f64::consts::FRAC_PI_2, // 90 degrees
            },
            Some(&base), None,
        ).unwrap();
        // After 90° rotation around X: Z normal → Y normal
        assert!((result.normal.y - 1.0).abs() < 0.01 || (result.normal.y - (-1.0)).abs() < 0.01,
            "Normal Y should be ~±1 after 90° rotation, got {:?}", result.normal);
    }

    #[test]
    fn three_point_plane() {
        let result = compute_datum_plane(
            &DatumPlaneKind::ThreePoint {
                p1: [0.0, 0.0, 0.0],
                p2: [10.0, 0.0, 0.0],
                p3: [0.0, 10.0, 0.0],
            },
            None, None,
        ).unwrap();
        // Points are in XY plane → normal should be Z
        assert!((result.normal.z.abs() - 1.0).abs() < 1e-9);
        assert!((result.origin.x).abs() < 1e-9);
    }

    #[test]
    fn three_collinear_points_fails() {
        let result = compute_datum_plane(
            &DatumPlaneKind::ThreePoint {
                p1: [0.0, 0.0, 0.0],
                p2: [5.0, 0.0, 0.0],
                p3: [10.0, 0.0, 0.0],
            },
            None, None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn face_plane_from_box() {
        let brep = crate::topology::builders::build_box_brep(10.0, 5.0, 3.0).unwrap();
        let result = compute_datum_plane(
            &DatumPlaneKind::FacePlane { face_index: 1 }, // top face
            None, Some(&brep),
        ).unwrap();
        // Top face of box has normal +Z, centroid at (5, 2.5, 3)
        assert!((result.normal.z - 1.0).abs() < 0.1, "Top face normal should be +Z");
        assert!((result.origin.z - 3.0).abs() < 0.1, "Top face origin Z should be ~3");
    }
}
