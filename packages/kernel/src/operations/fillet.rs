use crate::error::{KernelError, KernelResult};
use crate::geometry::surface::plane::Plane;
use crate::geometry::{Pt3, Vec3};
use crate::topology::adjacency::find_shared_edges;
use crate::topology::body::Body;
use crate::topology::builders::make_planar_face;
use crate::topology::edge::Orientation;
use crate::topology::shell::Shell;
use crate::topology::solid::Solid;
use crate::topology::vertex::VertexId;
use crate::topology::BRep;

use super::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FilletParams {
    pub edge_indices: Vec<u32>,
    pub radius: f64,
}

/// Number of flat segments used to approximate the fillet arc.
const FILLET_SEGMENTS: usize = 6;

#[derive(Debug)]
pub struct FilletOp;

impl Operation for FilletOp {
    type Params = FilletParams;

    fn execute(&self, params: &Self::Params, input: &BRep) -> KernelResult<BRep> {
        fillet_edges(input, params)
    }

    fn name(&self) -> &'static str {
        "Fillet"
    }
}

pub fn fillet_edges(brep: &BRep, params: &FilletParams) -> KernelResult<BRep> {
    if brep.faces.is_empty() {
        return Err(KernelError::Operation {
            op: "fillet".into(),
            detail: "Cannot fillet: no existing geometry".into(),
        });
    }

    let r = params.radius;
    if r <= 0.0 {
        return Err(KernelError::InvalidParameter {
            param: "radius".into(),
            value: format!("Fillet radius must be positive: {}", r),
        });
    }

    let shared_edges = find_shared_edges(brep, 1e-9);

    // Validate edge indices
    for &idx in &params.edge_indices {
        if (idx as usize) >= shared_edges.len() {
            return Err(KernelError::InvalidParameter {
                param: "edge_indices".into(),
                value: format!(
                    "Edge index {} out of range (max {})",
                    idx,
                    shared_edges.len().saturating_sub(1)
                ),
            });
        }
    }

    // Build a map of vertex modifications: VertexId -> new Pt3
    let mut vertex_mods: std::collections::HashMap<VertexId, Pt3> =
        std::collections::HashMap::new();

    // Collect fillet strip quads: each selected edge produces FILLET_SEGMENTS quads
    struct FilletQuad {
        points: [Pt3; 4],
    }
    let mut fillet_quads: Vec<FilletQuad> = Vec::new();

    for &edge_idx in &params.edge_indices {
        let se = &shared_edges[edge_idx as usize];

        let edge_dir = (se.end - se.start).normalize();

        // Get face normals
        let face_a = brep.faces.get(se.face_a)?;
        let face_b = brep.faces.get(se.face_b)?;

        let surf_a = face_a
            .surface_index
            .ok_or_else(|| KernelError::Topology("Face A has no surface".into()))?;
        let surf_b = face_b
            .surface_index
            .ok_or_else(|| KernelError::Topology("Face B has no surface".into()))?;

        let normal_a = brep.surfaces[surf_a].normal_at(0.0, 0.0)?;
        let normal_b = brep.surfaces[surf_b].normal_at(0.0, 0.0)?;

        // Compute offset directions perpendicular to edge, in each face's plane
        let mut offset_a = normal_a.cross(&edge_dir).normalize();
        let mut offset_b = normal_b.cross(&edge_dir).normalize();

        // Offset should point INTO the face (away from the shared edge)
        if offset_a.dot(&normal_b) < 0.0 {
            offset_a = -offset_a;
        }
        if offset_b.dot(&normal_a) < 0.0 {
            offset_b = -offset_b;
        }

        // Compute the dihedral half-angle between the two faces
        let cos_angle = normal_a.dot(&normal_b);
        // half_angle of the dihedral angle supplement (angle between the faces' inward directions)
        let half_angle = ((1.0 - cos_angle).max(0.0))
            .sqrt()
            .atan2(((1.0 + cos_angle).max(0.0)).sqrt());

        // Trim distance on each face: r * tan(half_angle)
        // For 90-degree edges (cos_angle = 0), half_angle = pi/4, tan = 1, so trim = r
        let trim = if half_angle.abs() < 1e-12 {
            r // degenerate: treat as 90 degrees
        } else {
            r * half_angle.tan()
        };

        // Trim points on face A and B
        let ta_start = se.start + offset_a * trim;
        let ta_end = se.end + offset_a * trim;
        let tb_start = se.start + offset_b * trim;
        let tb_end = se.end + offset_b * trim;

        // Record vertex modifications (trim face edges back)
        vertex_mods.insert(se.vertex_a_start, ta_start);
        vertex_mods.insert(se.vertex_a_end, ta_end);
        vertex_mods.insert(se.vertex_b_start, tb_start);
        vertex_mods.insert(se.vertex_b_end, tb_end);

        // Generate arc points at the start and end of the edge
        // The arc lies in the plane perpendicular to the edge direction,
        // going from offset_a to offset_b direction at radius r from a center point.
        //
        // Arc center is at: edge_point + center_offset
        // where center_offset is along the bisector of offset_a and offset_b
        // at distance r / cos(half_angle) from the edge
        let bisector = (offset_a + offset_b).normalize();
        let center_dist = if half_angle.cos().abs() < 1e-12 {
            r
        } else {
            r / half_angle.cos()
        };

        // For each segment, compute the arc point by interpolating the angle
        // from offset_a to offset_b
        // The total sweep angle of the fillet arc = pi - dihedral_angle = 2 * half_angle
        let sweep_angle = 2.0 * half_angle;

        // At each point along the edge (start and end), compute N+1 arc points
        let arc_points_at = |edge_pt: Pt3| -> Vec<Pt3> {
            let center = edge_pt + bisector * center_dist;
            let mut pts = Vec::with_capacity(FILLET_SEGMENTS + 1);
            for seg in 0..=FILLET_SEGMENTS {
                let t = seg as f64 / FILLET_SEGMENTS as f64;
                let angle = -half_angle + t * sweep_angle;
                // Point on the arc: center + r * (cos(angle) * (-bisector) + sin(angle) * tangent)
                // We need a tangent vector perpendicular to bisector in the arc plane
                // The arc plane is perpendicular to edge_dir
                // offset_a rotated towards offset_b
                let pt = center - bisector * (r * angle.cos())
                    + (offset_a.cross(&bisector).normalize()) * (r * angle.sin());
                // Actually let's use a cleaner parameterization:
                // The start of the arc is at ta (on face A side) and the end is at tb (on face B side)
                // So start direction from center = offset_a * trim - bisector * center_dist (normalized to r)
                pts.push(pt);
            }
            pts
        };

        let arc_start = arc_points_at(se.start);
        let arc_end = arc_points_at(se.end);

        // Create quad faces for each segment of the fillet strip
        for seg in 0..FILLET_SEGMENTS {
            fillet_quads.push(FilletQuad {
                points: [
                    arc_start[seg],
                    arc_end[seg],
                    arc_end[seg + 1],
                    arc_start[seg + 1],
                ],
            });
        }
    }

    // Reconstruct the BRep
    let mut result = BRep::new();

    // Rebuild existing faces with modified vertices
    for (_face_id, face) in brep.faces.iter() {
        let loop_id = face
            .outer_loop
            .ok_or_else(|| KernelError::Topology("Face has no outer loop".into()))?;
        let loop_ = brep.loops.get(loop_id)?;

        let surf_idx = face
            .surface_index
            .ok_or_else(|| KernelError::Topology("Face has no surface".into()))?;
        let normal = brep.surfaces[surf_idx].normal_at(0.0, 0.0)?;

        let mut points: Vec<Pt3> = Vec::new();
        for &coedge_id in &loop_.coedges {
            let coedge = brep.coedges.get(coedge_id)?;
            let edge = brep.edges.get(coedge.edge)?;
            let start_vid = match coedge.orientation {
                Orientation::Forward => edge.start,
                Orientation::Reversed => edge.end,
            };
            let vertex = brep.vertices.get(start_vid)?;
            let pos = if let Some(&new_pos) = vertex_mods.get(&start_vid) {
                new_pos
            } else {
                vertex.point
            };
            points.push(pos);
        }

        let origin = brep.surfaces[surf_idx].point_at(0.0, 0.0)?;
        let u_axis = if points.len() >= 2 {
            (points[1] - points[0]).normalize()
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let v_axis = normal.cross(&u_axis).normalize();

        let plane = Plane {
            origin,
            normal,
            u_axis,
            v_axis,
        };

        make_planar_face(&mut result, &points, plane)?;
    }

    // Add fillet strip faces
    for fq in &fillet_quads {
        let edge1 = (fq.points[1] - fq.points[0]).normalize();
        let edge2 = (fq.points[3] - fq.points[0]).normalize();
        let normal = edge1.cross(&edge2).normalize();
        let plane = Plane {
            origin: fq.points[0],
            normal,
            u_axis: edge1,
            v_axis: edge2,
        };
        make_planar_face(&mut result, &fq.points, plane)?;
    }

    // Rebuild shell and solid
    let face_ids: Vec<_> = result.faces.iter().map(|(id, _)| id).collect();
    let shell_id = result.shells.insert(Shell::new(face_ids, true));
    let solid_id = result.solids.insert(Solid::new(vec![shell_id]));
    result.body = Body::Solid(solid_id);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;

    #[test]
    fn fillet_single_edge_of_box() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = FilletParams {
            edge_indices: vec![0],
            radius: 1.0,
        };
        let result = fillet_edges(&brep, &params).unwrap();
        // 6 original faces + FILLET_SEGMENTS fillet faces
        let expected_min = 6 + FILLET_SEGMENTS;
        assert_eq!(
            result.faces.len(),
            expected_min,
            "Filleted box should have {} faces, got {}",
            expected_min,
            result.faces.len()
        );
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn fillet_empty_brep_rejected() {
        let brep = BRep::new();
        let params = FilletParams {
            edge_indices: vec![0],
            radius: 1.0,
        };
        assert!(fillet_edges(&brep, &params).is_err());
    }

    #[test]
    fn fillet_invalid_edge_rejected() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = FilletParams {
            edge_indices: vec![999],
            radius: 1.0,
        };
        assert!(fillet_edges(&brep, &params).is_err());
    }

    #[test]
    fn fillet_produces_more_faces_than_original() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = FilletParams {
            edge_indices: vec![0],
            radius: 1.0,
        };
        let result = fillet_edges(&brep, &params).unwrap();
        assert!(
            result.faces.len() > 6,
            "Filleted box should have more than 6 faces, got {}",
            result.faces.len()
        );
    }
}
