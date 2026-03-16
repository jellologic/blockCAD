use crate::error::{KernelError, KernelResult};
use crate::geometry::surface::plane::Plane;
use crate::geometry::{Pt3, Vec3};
use crate::topology::adjacency::find_shared_edges;
use crate::topology::body::Body;
use crate::topology::builders::make_planar_face;
use crate::topology::edge::Orientation;
use crate::topology::face::FaceId;
use crate::topology::shell::Shell;
use crate::topology::solid::Solid;
use crate::topology::vertex::VertexId;
use crate::topology::BRep;

use super::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChamferParams {
    pub edge_indices: Vec<u32>,
    pub distance: f64,
    /// Optional second distance for asymmetric chamfer
    pub distance2: Option<f64>,
}

#[derive(Debug)]
pub struct ChamferOp;

impl Operation for ChamferOp {
    type Params = ChamferParams;

    fn execute(&self, params: &Self::Params, input: &BRep) -> KernelResult<BRep> {
        chamfer_edges(input, params)
    }

    fn name(&self) -> &'static str {
        "Chamfer"
    }
}

/// At each endpoint of a chamfered edge, third-party faces need their corner
/// replaced with the two chamfer points to close the gap.
struct EndpointSplit {
    original_pos: Pt3,
    /// Points to insert: from face-A trim point (ca) to face-B trim point (cb).
    /// For chamfer this is [ca, cb]; for fillet it would be [ta, arc..., tb].
    split_points: Vec<Pt3>,
    face_a: FaceId,
    face_b: FaceId,
}

pub fn chamfer_edges(brep: &BRep, params: &ChamferParams) -> KernelResult<BRep> {
    if brep.faces.is_empty() {
        return Err(KernelError::Operation {
            op: "chamfer".into(),
            detail: "Cannot chamfer: no existing geometry".into(),
        });
    }

    let d1 = params.distance;
    let d2 = params.distance2.unwrap_or(d1);

    if d1 <= 0.0 || d2 <= 0.0 {
        return Err(KernelError::InvalidParameter {
            param: "distance".into(),
            value: format!("Chamfer distances must be positive: d1={}, d2={}", d1, d2),
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

    // Endpoint splits for third-party faces
    let mut endpoint_splits: Vec<EndpointSplit> = Vec::new();

    // Collect chamfer face data
    struct ChamferFace {
        points: [Pt3; 4],
    }
    let mut chamfer_faces: Vec<ChamferFace> = Vec::new();

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

        // Offset should point INTO the solid (away from the shared edge, toward the interior).
        if offset_a.dot(&normal_b) > 0.0 {
            offset_a = -offset_a;
        }
        if offset_b.dot(&normal_a) > 0.0 {
            offset_b = -offset_b;
        }

        // Compute chamfer corner points
        let ca_start = se.start + offset_a * d1;
        let ca_end = se.end + offset_a * d1;
        let cb_start = se.start + offset_b * d2;
        let cb_end = se.end + offset_b * d2;

        // Record vertex modifications
        vertex_mods.insert(se.vertex_a_start, ca_start);
        vertex_mods.insert(se.vertex_a_end, ca_end);
        vertex_mods.insert(se.vertex_b_start, cb_start);
        vertex_mods.insert(se.vertex_b_end, cb_end);

        // Record endpoint splits for third-party faces
        endpoint_splits.push(EndpointSplit {
            original_pos: se.start,
            split_points: vec![ca_start, cb_start],
            face_a: se.face_a,
            face_b: se.face_b,
        });
        endpoint_splits.push(EndpointSplit {
            original_pos: se.end,
            split_points: vec![ca_end, cb_end],
            face_a: se.face_a,
            face_b: se.face_b,
        });

        // Record chamfer face with correct outward-pointing normal.
        // The offsets point INTO the solid, so the chamfer face normal should
        // point AWAY from offset directions (toward the removed corner).
        let outward_dir = -(offset_a + offset_b).normalize();
        let trial_e1 = (ca_end - ca_start).normalize();
        let trial_e2 = (cb_start - ca_start).normalize();
        let trial_normal = trial_e1.cross(&trial_e2);
        if trial_normal.dot(&outward_dir) >= 0.0 {
            chamfer_faces.push(ChamferFace {
                points: [ca_start, ca_end, cb_end, cb_start],
            });
        } else {
            chamfer_faces.push(ChamferFace {
                points: [cb_start, cb_end, ca_end, ca_start],
            });
        }
    }

    // Reconstruct the BRep
    let mut result = BRep::new();

    let tol2 = 1e-9 * 1e-9;

    // Rebuild existing faces with modified vertices AND endpoint splits
    for (face_id, face) in brep.faces.iter() {
        let loop_id = face
            .outer_loop
            .ok_or_else(|| KernelError::Topology("Face has no outer loop".into()))?;
        let loop_ = brep.loops.get(loop_id)?;

        let surf_idx = face
            .surface_index
            .ok_or_else(|| KernelError::Topology("Face has no surface".into()))?;
        let normal = brep.surfaces[surf_idx].normal_at(0.0, 0.0)?;

        // First pass: collect all original positions and their vertex IDs
        let mut orig_positions: Vec<Pt3> = Vec::new();
        let mut orig_vids: Vec<VertexId> = Vec::new();
        for &coedge_id in &loop_.coedges {
            let coedge = brep.coedges.get(coedge_id)?;
            let edge = brep.edges.get(coedge.edge)?;
            let start_vid = match coedge.orientation {
                Orientation::Forward => edge.start,
                Orientation::Reversed => edge.end,
            };
            let vertex = brep.vertices.get(start_vid)?;
            orig_positions.push(vertex.point);
            orig_vids.push(start_vid);
        }

        // Second pass: build output polygon, handling vertex_mods and splits
        let n = orig_positions.len();
        let mut points: Vec<Pt3> = Vec::new();
        for i in 0..n {
            let vid = orig_vids[i];
            let pos = orig_positions[i];

            if let Some(&new_pos) = vertex_mods.get(&vid) {
                points.push(new_pos);
            } else {
                // Check if this vertex position matches an endpoint that needs splitting
                let mut split_done = false;
                for ep in &endpoint_splits {
                    if face_id == ep.face_a || face_id == ep.face_b {
                        continue;
                    }
                    let d = pos - ep.original_pos;
                    if d.x * d.x + d.y * d.y + d.z * d.z < tol2 {
                        // Determine correct winding order using adjacent vertices.
                        // The previous vertex in the polygon tells us which direction
                        // we're coming from, and the next vertex tells us where we go.
                        // The split point closest to the previous vertex should come first.
                        let prev_pos = orig_positions[(i + n - 1) % n];
                        let prev_actual = if let Some(&mp) = vertex_mods.get(&orig_vids[(i + n - 1) % n]) {
                            mp
                        } else {
                            prev_pos
                        };

                        let first = ep.split_points.first().unwrap();
                        let last = ep.split_points.last().unwrap();

                        let dist_first_to_prev = {
                            let d = *first - prev_actual;
                            d.x * d.x + d.y * d.y + d.z * d.z
                        };
                        let dist_last_to_prev = {
                            let d = *last - prev_actual;
                            d.x * d.x + d.y * d.y + d.z * d.z
                        };

                        if dist_first_to_prev <= dist_last_to_prev {
                            // first (ca) is closer to prev -> forward order
                            for pt in &ep.split_points {
                                points.push(*pt);
                            }
                        } else {
                            // last (cb) is closer to prev -> reverse order
                            for pt in ep.split_points.iter().rev() {
                                points.push(*pt);
                            }
                        }
                        split_done = true;
                        break;
                    }
                }
                if !split_done {
                    points.push(pos);
                }
            }
        }

        // Get origin from original surface
        let origin = brep.surfaces[surf_idx].point_at(0.0, 0.0)?;

        // Recompute u_axis from the first edge direction
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

    // Add chamfer faces
    for cf in &chamfer_faces {
        let edge1 = (cf.points[1] - cf.points[0]).normalize();
        let edge2 = (cf.points[3] - cf.points[0]).normalize();
        let normal = edge1.cross(&edge2).normalize();
        let plane = Plane {
            origin: cf.points[0],
            normal,
            u_axis: edge1,
            v_axis: edge2,
        };
        make_planar_face(&mut result, &cf.points, plane)?;
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
    fn chamfer_single_edge_of_box() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = ChamferParams {
            edge_indices: vec![0],
            distance: 1.0,
            distance2: None,
        };
        let result = chamfer_edges(&brep, &params).unwrap();
        // 6 original faces + 1 chamfer face = 7
        assert_eq!(
            result.faces.len(),
            7,
            "Chamfered box should have 7 faces, got {}",
            result.faces.len()
        );
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn chamfer_empty_brep_rejected() {
        let brep = BRep::new();
        let params = ChamferParams {
            edge_indices: vec![0],
            distance: 1.0,
            distance2: None,
        };
        assert!(chamfer_edges(&brep, &params).is_err());
    }

    #[test]
    fn chamfer_invalid_edge_rejected() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = ChamferParams {
            edge_indices: vec![999],
            distance: 1.0,
            distance2: None,
        };
        assert!(chamfer_edges(&brep, &params).is_err());
    }

    #[test]
    fn chamfer_tessellates_without_error() {
        use crate::tessellation::{tessellate_brep, TessellationParams};

        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = ChamferParams {
            edge_indices: vec![0],
            distance: 1.0,
            distance2: None,
        };
        let result = chamfer_edges(&brep, &params).unwrap();
        let mesh = tessellate_brep(&result, &TessellationParams::default()).unwrap();
        mesh.validate().unwrap();
        assert!(
            mesh.triangle_count() > 0,
            "Chamfer mesh should have triangles"
        );
    }

    #[test]
    fn chamfer_multiple_edges() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = ChamferParams {
            edge_indices: vec![0, 1, 2],
            distance: 1.0,
            distance2: None,
        };
        let result = chamfer_edges(&brep, &params).unwrap();
        // 6 original + 3 chamfer faces = 9
        assert_eq!(result.faces.len(), 9);
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn chamfer_asymmetric_distances() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = ChamferParams {
            edge_indices: vec![0],
            distance: 1.0,
            distance2: Some(2.0),
        };
        let result = chamfer_edges(&brep, &params).unwrap();
        assert_eq!(result.faces.len(), 7);
        assert!(matches!(result.body, Body::Solid(_)));
    }

}
