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
pub struct FilletParams {
    pub edge_indices: Vec<u32>,
    pub radius: f64,
}

/// A control point specifying the fillet radius at a particular position along an edge.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RadiusPoint {
    /// Parameter along the edge, 0.0 = start, 1.0 = end.
    pub parameter: f64,
    /// Fillet radius at this parameter.
    pub radius: f64,
}

/// Parameters for a variable-radius fillet where the radius varies along each edge.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VariableFilletParams {
    /// Indices of edges to fillet (into the shared-edge list).
    pub edge_indices: Vec<u32>,
    /// At least 2 radius control points (start and end). Must be sorted by parameter.
    pub radius_points: Vec<RadiusPoint>,
    /// If true, use cubic (Catmull-Rom) interpolation; otherwise linear.
    pub smooth_transition: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FaceFilletParams {
    pub face_indices: Vec<usize>,
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

#[derive(Debug)]
pub struct VariableFilletOp;

impl Operation for VariableFilletOp {
    type Params = VariableFilletParams;

    fn execute(&self, params: &Self::Params, input: &BRep) -> KernelResult<BRep> {
        variable_fillet_edges(input, params)
    }

    fn name(&self) -> &'static str {
        "VariableFillet"
    }
}

#[derive(Debug)]
pub struct FaceFilletOp;

impl Operation for FaceFilletOp {
    type Params = FaceFilletParams;

    fn execute(&self, params: &Self::Params, input: &BRep) -> KernelResult<BRep> {
        face_fillet(input, params)
    }

    fn name(&self) -> &'static str {
        "FaceFillet"
    }
}

/// Apply fillet to all edges bounding the selected face(s).
pub fn face_fillet(brep: &BRep, params: &FaceFilletParams) -> KernelResult<BRep> {
    if brep.faces.is_empty() {
        return Err(KernelError::Operation {
            op: "face_fillet".into(),
            detail: "Cannot face fillet: no existing geometry".into(),
        });
    }

    if params.face_indices.is_empty() {
        return Err(KernelError::InvalidParameter {
            param: "face_indices".into(),
            value: "No face indices provided".into(),
        });
    }

    let face_ids: Vec<FaceId> = brep.faces.iter().map(|(id, _)| id).collect();
    for &fi in &params.face_indices {
        if fi >= face_ids.len() {
            return Err(KernelError::InvalidParameter {
                param: "face_indices".into(),
                value: format!(
                    "Face index {} out of range (max {})",
                    fi,
                    face_ids.len().saturating_sub(1)
                ),
            });
        }
    }

    let selected_face_ids: std::collections::HashSet<FaceId> = params
        .face_indices
        .iter()
        .map(|&fi| face_ids[fi])
        .collect();

    let shared_edges = find_shared_edges(brep, 1e-9);

    let mut edge_index_set = std::collections::BTreeSet::new();
    for (idx, se) in shared_edges.iter().enumerate() {
        if selected_face_ids.contains(&se.face_a) || selected_face_ids.contains(&se.face_b) {
            edge_index_set.insert(idx as u32);
        }
    }

    if edge_index_set.is_empty() {
        return Err(KernelError::Operation {
            op: "face_fillet".into(),
            detail: "No edges found for the selected face(s)".into(),
        });
    }

    let edge_params = FilletParams {
        edge_indices: edge_index_set.into_iter().collect(),
        radius: params.radius,
    };

    fillet_edges(brep, &edge_params)
}

/// Interpolate the fillet radius at parameter `t` given sorted control points.
/// Uses linear interpolation when `smooth` is false, cubic (Catmull-Rom) when true.
fn interpolate_radius(points: &[RadiusPoint], t: f64, smooth: bool) -> f64 {
    debug_assert!(points.len() >= 2);
    let t = t.clamp(0.0, 1.0);

    // Find the segment containing t
    let n = points.len();
    let mut idx = 0;
    for i in 0..n - 1 {
        if t >= points[i].parameter && t <= points[i + 1].parameter {
            idx = i;
            break;
        }
        if i == n - 2 {
            idx = i; // clamp to last segment
        }
    }

    let p0 = &points[idx];
    let p1 = &points[idx + 1];
    let seg_len = p1.parameter - p0.parameter;
    if seg_len < 1e-15 {
        return p0.radius;
    }
    let local_t = (t - p0.parameter) / seg_len;

    if !smooth || n < 3 {
        // Linear interpolation
        p0.radius + (p1.radius - p0.radius) * local_t
    } else {
        // Catmull-Rom spline interpolation
        let r_prev = if idx > 0 {
            points[idx - 1].radius
        } else {
            // Extrapolate: mirror slope
            2.0 * p0.radius - p1.radius
        };
        let r_next = if idx + 2 < n {
            points[idx + 2].radius
        } else {
            2.0 * p1.radius - p0.radius
        };

        let t2 = local_t * local_t;
        let t3 = t2 * local_t;

        let r = 0.5
            * ((2.0 * p0.radius)
                + (-r_prev + p1.radius) * local_t
                + (2.0 * r_prev - 5.0 * p0.radius + 4.0 * p1.radius - r_next) * t2
                + (-r_prev + 3.0 * p0.radius - 3.0 * p1.radius + r_next) * t3);

        // Clamp to positive to prevent negative radii from overshoot
        r.max(1e-6)
    }
}

/// Number of stations along the edge for variable fillet sampling.
const VARIABLE_FILLET_STATIONS: usize = 8;

pub fn variable_fillet_edges(brep: &BRep, params: &VariableFilletParams) -> KernelResult<BRep> {
    if brep.faces.is_empty() {
        return Err(KernelError::Operation {
            op: "variable_fillet".into(),
            detail: "Cannot fillet: no existing geometry".into(),
        });
    }

    // Validate radius points
    if params.radius_points.len() < 2 {
        return Err(KernelError::InvalidParameter {
            param: "radius_points".into(),
            value: "Variable fillet requires at least 2 radius control points".into(),
        });
    }
    for rp in &params.radius_points {
        if rp.radius <= 0.0 {
            return Err(KernelError::InvalidParameter {
                param: "radius".into(),
                value: format!("Fillet radius must be positive: {}", rp.radius),
            });
        }
        if rp.parameter < 0.0 || rp.parameter > 1.0 {
            return Err(KernelError::InvalidParameter {
                param: "parameter".into(),
                value: format!(
                    "Radius point parameter must be in [0, 1]: {}",
                    rp.parameter
                ),
            });
        }
    }
    // Ensure sorted
    for i in 1..params.radius_points.len() {
        if params.radius_points[i].parameter < params.radius_points[i - 1].parameter {
            return Err(KernelError::InvalidParameter {
                param: "radius_points".into(),
                value: "Radius control points must be sorted by parameter".into(),
            });
        }
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

    let mut vertex_mods: std::collections::HashMap<VertexId, Pt3> =
        std::collections::HashMap::new();
    let mut endpoint_splits: Vec<EndpointSplit> = Vec::new();

    struct FilletQuad {
        points: [Pt3; 4],
    }
    let mut fillet_quads: Vec<FilletQuad> = Vec::new();

    for &edge_idx in &params.edge_indices {
        let se = &shared_edges[edge_idx as usize];

        let edge_vec = se.end - se.start;
        if edge_vec.norm() < 1e-6 {
            continue;
        }
        let edge_dir = edge_vec.normalize();

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

        let mut offset_a = normal_a.cross(&edge_dir).normalize();
        let mut offset_b = normal_b.cross(&edge_dir).normalize();

        if offset_a.dot(&normal_b) > 0.0 {
            offset_a = -offset_a;
        }
        if offset_b.dot(&normal_a) > 0.0 {
            offset_b = -offset_b;
        }

        let cos_angle = normal_a.dot(&normal_b);
        let half_angle = ((1.0 - cos_angle).max(0.0))
            .sqrt()
            .atan2(((1.0 + cos_angle).max(0.0)).sqrt());

        if half_angle.abs() < 0.025 {
            continue;
        }

        let bisector = (offset_a + offset_b).normalize();

        // Sample stations along the edge, computing arc cross-sections at each.
        // Use VARIABLE_FILLET_STATIONS interior stations + 2 endpoints.
        let total_stations = VARIABLE_FILLET_STATIONS + 2;
        let mut station_arcs: Vec<Vec<Pt3>> = Vec::with_capacity(total_stations);

        for station in 0..total_stations {
            let t = station as f64 / (total_stations - 1) as f64;
            let edge_pt = Pt3::new(
                se.start.x + edge_vec.x * t,
                se.start.y + edge_vec.y * t,
                se.start.z + edge_vec.z * t,
            );

            let r = interpolate_radius(&params.radius_points, t, params.smooth_transition);

            let trim = if half_angle.abs() < 1e-12 {
                r
            } else {
                r * half_angle.tan()
            };

            let ta = edge_pt + offset_a * trim;
            let tb = edge_pt + offset_b * trim;

            let center_dist = if half_angle.cos().abs() < 1e-12 {
                r
            } else {
                r / half_angle.cos()
            };
            let center = edge_pt + bisector * center_dist;
            let start_dir = (ta - center).normalize();
            let arc_tangent = edge_dir.cross(&start_dir).normalize();
            let sweep_angle = 2.0 * half_angle;

            let mut arc_pts = Vec::with_capacity(FILLET_SEGMENTS + 1);
            for seg in 0..=FILLET_SEGMENTS {
                if seg == 0 {
                    arc_pts.push(ta);
                } else if seg == FILLET_SEGMENTS {
                    arc_pts.push(tb);
                } else {
                    let s = seg as f64 / FILLET_SEGMENTS as f64;
                    let angle = s * sweep_angle;
                    arc_pts.push(
                        center + start_dir * (r * angle.cos()) + arc_tangent * (r * angle.sin()),
                    );
                }
            }
            station_arcs.push(arc_pts);
        }

        // Record vertex mods using the start/end station arcs
        let arc_start = &station_arcs[0];
        let arc_end = &station_arcs[total_stations - 1];

        vertex_mods.insert(se.vertex_a_start, arc_start[0]);
        vertex_mods.insert(se.vertex_a_end, arc_end[0]);
        vertex_mods.insert(se.vertex_b_start, arc_start[FILLET_SEGMENTS]);
        vertex_mods.insert(se.vertex_b_end, arc_end[FILLET_SEGMENTS]);

        // Endpoint splits
        endpoint_splits.push(EndpointSplit {
            original_pos: se.start,
            arc_points: arc_start.clone(),
            face_a: se.face_a,
            face_b: se.face_b,
        });
        endpoint_splits.push(EndpointSplit {
            original_pos: se.end,
            arc_points: arc_end.clone(),
            face_a: se.face_a,
            face_b: se.face_b,
        });

        // Build fillet strip quads between consecutive stations
        let outward_dir = -(offset_a + offset_b).normalize();
        for st in 0..total_stations - 1 {
            let curr = &station_arcs[st];
            let next = &station_arcs[st + 1];
            for seg in 0..FILLET_SEGMENTS {
                let p0 = curr[seg];
                let p1 = next[seg];
                let p2 = next[seg + 1];
                let p3 = curr[seg + 1];

                let trial_e1 = (p1 - p0).normalize();
                let trial_e2 = (p3 - p0).normalize();
                let trial_normal = trial_e1.cross(&trial_e2);

                if trial_normal.dot(&outward_dir) >= 0.0 {
                    fillet_quads.push(FilletQuad { points: [p0, p1, p2, p3] });
                } else {
                    fillet_quads.push(FilletQuad { points: [p3, p2, p1, p0] });
                }
            }
        }
    }

    // Position-based lookup for revolution seam duplicates
    let pos_tol2 = 1e-9 * 1e-9;
    let vertex_mod_positions: Vec<(Pt3, Pt3)> = vertex_mods
        .iter()
        .map(|(vid, &new_pos)| {
            let old_pos = brep.vertices.get(*vid).map(|v| v.point).unwrap_or(new_pos);
            (old_pos, new_pos)
        })
        .collect();

    // Reconstruct the BRep
    let mut result = BRep::new();
    let tol2 = 1e-9 * 1e-9;

    // Rebuild existing faces with modified vertices and endpoint splits
    for (face_id, face) in brep.faces.iter() {
        let loop_id = face
            .outer_loop
            .ok_or_else(|| KernelError::Topology("Face has no outer loop".into()))?;
        let loop_ = brep.loops.get(loop_id)?;

        let surf_idx = face
            .surface_index
            .ok_or_else(|| KernelError::Topology("Face has no surface".into()))?;
        let normal = brep.surfaces[surf_idx].normal_at(0.0, 0.0)?;

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

        let num = orig_positions.len();
        let mut points: Vec<Pt3> = Vec::new();
        for i in 0..num {
            let vid = orig_vids[i];
            let pos = orig_positions[i];

            if let Some(&new_pos) = vertex_mods.get(&vid) {
                points.push(new_pos);
                continue;
            }

            let mut split_done = false;
            for ep in &endpoint_splits {
                if face_id == ep.face_a || face_id == ep.face_b {
                    continue;
                }
                let d = pos - ep.original_pos;
                if d.x * d.x + d.y * d.y + d.z * d.z < tol2 {
                    let prev_pos = orig_positions[(i + num - 1) % num];
                    let prev_vid = orig_vids[(i + num - 1) % num];
                    let prev_actual = vertex_mods
                        .get(&prev_vid)
                        .copied()
                        .or_else(|| {
                            vertex_mod_positions.iter().find_map(|(old_p, new_p)| {
                                let d = prev_pos - *old_p;
                                if d.x * d.x + d.y * d.y + d.z * d.z < pos_tol2 {
                                    Some(*new_p)
                                } else {
                                    None
                                }
                            })
                        })
                        .unwrap_or(prev_pos);

                    let first = ep.arc_points.first().unwrap();
                    let last = ep.arc_points.last().unwrap();

                    let dist_first = {
                        let d = *first - prev_actual;
                        d.x * d.x + d.y * d.y + d.z * d.z
                    };
                    let dist_last = {
                        let d = *last - prev_actual;
                        d.x * d.x + d.y * d.y + d.z * d.z
                    };

                    if dist_first <= dist_last {
                        for pt in &ep.arc_points {
                            points.push(*pt);
                        }
                    } else {
                        for pt in ep.arc_points.iter().rev() {
                            points.push(*pt);
                        }
                    }
                    split_done = true;
                    break;
                }
            }
            if split_done {
                continue;
            }

            if let Some(new_pos) = vertex_mod_positions.iter().find_map(|(old_p, new_p)| {
                let d = pos - *old_p;
                if d.x * d.x + d.y * d.y + d.z * d.z < pos_tol2 {
                    Some(*new_p)
                } else {
                    None
                }
            }) {
                points.push(new_pos);
                continue;
            }

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

        let _ = make_planar_face(&mut result, &points, plane);
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
        let _ = make_planar_face(&mut result, &fq.points, plane);
    }

    // Rebuild shell and solid
    let face_ids: Vec<_> = result.faces.iter().map(|(id, _)| id).collect();
    let shell_id = result.shells.insert(Shell::new(face_ids, true));
    let solid_id = result.solids.insert(Solid::new(vec![shell_id]));
    result.body = Body::Solid(solid_id);

    Ok(result)
}

/// At each endpoint of a filleted edge, third-party faces need their corner
/// replaced with the full arc point sequence so that the end face shares
/// edges with the fillet strip quads.
struct EndpointSplit {
    /// The original vertex position at the endpoint
    original_pos: Pt3,
    /// All arc points at this endpoint, from ta (face A trim) to tb (face B trim).
    /// This is FILLET_SEGMENTS+1 points, matching the fillet strip boundary.
    arc_points: Vec<Pt3>,
    /// Face A id (should NOT get the split, it uses vertex_mods)
    face_a: FaceId,
    /// Face B id (should NOT get the split, it uses vertex_mods)
    face_b: FaceId,
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

    // Map VertexId -> new position for vertices on the two faces adjacent to each filleted edge.
    let mut vertex_mods: std::collections::HashMap<VertexId, Pt3> =
        std::collections::HashMap::new();

    // Endpoint splits for third-party faces (with full arc point sequences)
    let mut endpoint_splits: Vec<EndpointSplit> = Vec::new();

    // Collect fillet strip quads
    struct FilletQuad {
        points: [Pt3; 4],
    }
    let mut fillet_quads: Vec<FilletQuad> = Vec::new();

    for &edge_idx in &params.edge_indices {
        let se = &shared_edges[edge_idx as usize];

        // Skip degenerate (seam) edges where start and end coincide.
        let edge_vec = se.end - se.start;
        if edge_vec.norm() < 1e-6 {
            continue;
        }
        let edge_dir = edge_vec.normalize();

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

        // Compute the dihedral half-angle between the two faces
        let cos_angle = normal_a.dot(&normal_b);
        let half_angle = ((1.0 - cos_angle).max(0.0))
            .sqrt()
            .atan2(((1.0 + cos_angle).max(0.0)).sqrt());

        // Skip nearly-coplanar edges (dihedral angle < ~3 degrees).
        // These are typically revolution seam edges or tessellation artifacts
        // where there is no real corner to fillet.
        if half_angle.abs() < 0.025 {
            continue;
        }

        // Trim distance on each face
        let trim = if half_angle.abs() < 1e-12 {
            r
        } else {
            r * half_angle.tan()
        };

        // Trim points on face A and B
        let ta_start = se.start + offset_a * trim;
        let ta_end = se.end + offset_a * trim;
        let tb_start = se.start + offset_b * trim;
        let tb_end = se.end + offset_b * trim;

        // Record vertex modifications for the two adjacent faces.
        vertex_mods.insert(se.vertex_a_start, ta_start);
        vertex_mods.insert(se.vertex_a_end, ta_end);
        vertex_mods.insert(se.vertex_b_start, tb_start);
        vertex_mods.insert(se.vertex_b_end, tb_end);

        // Generate arc
        let bisector = (offset_a + offset_b).normalize();
        let center_dist = if half_angle.cos().abs() < 1e-12 {
            r
        } else {
            r / half_angle.cos()
        };

        let sweep_angle = 2.0 * half_angle;

        let arc_points_at = |edge_pt: Pt3, trim_a: Pt3, trim_b: Pt3| -> Vec<Pt3> {
            let center = edge_pt + bisector * center_dist;
            let start_dir = (trim_a - center).normalize();
            let arc_tangent = edge_dir.cross(&start_dir).normalize();

            let mut pts = Vec::with_capacity(FILLET_SEGMENTS + 1);
            for seg in 0..=FILLET_SEGMENTS {
                if seg == 0 {
                    pts.push(trim_a);
                } else if seg == FILLET_SEGMENTS {
                    pts.push(trim_b);
                } else {
                    let t = seg as f64 / FILLET_SEGMENTS as f64;
                    let angle = t * sweep_angle;
                    let pt =
                        center + start_dir * (r * angle.cos()) + arc_tangent * (r * angle.sin());
                    pts.push(pt);
                }
            }
            pts
        };

        let arc_start = arc_points_at(se.start, ta_start, tb_start);
        let arc_end = arc_points_at(se.end, ta_end, tb_end);

        // Record endpoint splits with full arc point sequences
        endpoint_splits.push(EndpointSplit {
            original_pos: se.start,
            arc_points: arc_start.clone(),
            face_a: se.face_a,
            face_b: se.face_b,
        });
        endpoint_splits.push(EndpointSplit {
            original_pos: se.end,
            arc_points: arc_end.clone(),
            face_a: se.face_a,
            face_b: se.face_b,
        });

        // Create quad faces for each segment of the fillet strip.
        // The outward direction for fillet strip faces points AWAY from the solid
        // interior (away from the offset/bisector direction).
        let outward_dir = -(offset_a + offset_b).normalize();
        for seg in 0..FILLET_SEGMENTS {
            let p0 = arc_start[seg];
            let p1 = arc_end[seg];
            let p2 = arc_end[seg + 1];
            let p3 = arc_start[seg + 1];

            let trial_e1 = (p1 - p0).normalize();
            let trial_e2 = (p3 - p0).normalize();
            let trial_normal = trial_e1.cross(&trial_e2);

            if trial_normal.dot(&outward_dir) >= 0.0 {
                fillet_quads.push(FilletQuad {
                    points: [p0, p1, p2, p3],
                });
            } else {
                fillet_quads.push(FilletQuad {
                    points: [p3, p2, p1, p0],
                });
            }
        }
    }

    // Build a position-based lookup for vertex modifications so that vertices
    // at the same geometric position (e.g. revolution seam duplicates) also get
    // the correct trim applied, even if they have different VertexIds.
    let pos_tol2 = 1e-9 * 1e-9;
    let vertex_mod_positions: Vec<(Pt3, Pt3)> = vertex_mods
        .iter()
        .map(|(vid, &new_pos)| {
            let old_pos = brep.vertices.get(*vid).map(|v| v.point).unwrap_or(new_pos);
            (old_pos, new_pos)
        })
        .collect();

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
        let num = orig_positions.len();
        let mut points: Vec<Pt3> = Vec::new();
        for i in 0..num {
            let vid = orig_vids[i];
            let pos = orig_positions[i];

            // 1. Check vertex_mods by exact VertexId (faces A/B of the filleted edge)
            if let Some(&new_pos) = vertex_mods.get(&vid) {
                points.push(new_pos);
                continue;
            }

            // 2. Check endpoint splits by position (third-party faces at edge endpoints)
            let mut split_done = false;
            {
            for ep in &endpoint_splits {
                if face_id == ep.face_a || face_id == ep.face_b {
                    continue;
                }
                let d = pos - ep.original_pos;
                if d.x * d.x + d.y * d.y + d.z * d.z < tol2 {
                    let prev_pos = orig_positions[(i + num - 1) % num];
                    let prev_vid = orig_vids[(i + num - 1) % num];
                    let prev_actual = vertex_mods.get(&prev_vid).copied().or_else(|| {
                        vertex_mod_positions.iter().find_map(|(old_p, new_p)| {
                            let d = prev_pos - *old_p;
                            if d.x * d.x + d.y * d.y + d.z * d.z < pos_tol2 {
                                Some(*new_p)
                            } else {
                                None
                            }
                        })
                    }).unwrap_or(prev_pos);

                    let first = ep.arc_points.first().unwrap();
                    let last = ep.arc_points.last().unwrap();

                    let dist_first_to_prev = {
                        let d = *first - prev_actual;
                        d.x * d.x + d.y * d.y + d.z * d.z
                    };
                    let dist_last_to_prev = {
                        let d = *last - prev_actual;
                        d.x * d.x + d.y * d.y + d.z * d.z
                    };

                    if dist_first_to_prev <= dist_last_to_prev {
                        for pt in &ep.arc_points {
                            points.push(*pt);
                        }
                    } else {
                        for pt in ep.arc_points.iter().rev() {
                            points.push(*pt);
                        }
                    }
                    split_done = true;
                    break;
                }
            }
            } // end if !has_pos_mod
            if split_done {
                continue;
            }

            // 3. Position-based vertex_mods fallback for revolution seam duplicates:
            //    vertices that share a geometric position with a modified vertex but
            //    have a different VertexId (e.g. at the wrap-around seam of a full revolve).
            if let Some(new_pos) = vertex_mod_positions.iter().find_map(|(old_p, new_p)| {
                let d = pos - *old_p;
                if d.x * d.x + d.y * d.y + d.z * d.z < pos_tol2 {
                    Some(*new_p)
                } else {
                    None
                }
            }) {
                points.push(new_pos);
                continue;
            }

            // 4. No modification needed — keep original position
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

        let _ = make_planar_face(&mut result, &points, plane);
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
        let _ = make_planar_face(&mut result, &fq.points, plane);
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

    #[test]
    fn fillet_tessellates_without_error() {
        use crate::tessellation::{tessellate_brep, TessellationParams};

        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = FilletParams {
            edge_indices: vec![0],
            radius: 1.0,
        };
        let result = fillet_edges(&brep, &params).unwrap();
        let mesh = tessellate_brep(&result, &TessellationParams::default()).unwrap();
        mesh.validate().unwrap();
        assert!(mesh.triangle_count() > 0, "Fillet mesh should have triangles");
        let box_mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        assert!(
            mesh.triangle_count() > box_mesh.triangle_count(),
            "Fillet mesh ({}) should have more triangles than box ({})",
            mesh.triangle_count(),
            box_mesh.triangle_count()
        );
    }

    #[test]
    fn fillet_multiple_edges() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = FilletParams {
            edge_indices: vec![0, 1, 2],
            radius: 1.0,
        };
        let result = fillet_edges(&brep, &params).unwrap();
        // 6 original + 3 * FILLET_SEGMENTS fillet faces
        assert_eq!(result.faces.len(), 6 + 3 * FILLET_SEGMENTS,);
        assert!(matches!(result.body, Body::Solid(_)));
    }

    // ── Variable fillet tests ──────────────────────────────────────────

    #[test]
    fn variable_fillet_linear_radius_on_box() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = VariableFilletParams {
            edge_indices: vec![0],
            radius_points: vec![
                RadiusPoint { parameter: 0.0, radius: 1.0 },
                RadiusPoint { parameter: 1.0, radius: 2.0 },
            ],
            smooth_transition: false,
        };
        let result = variable_fillet_edges(&brep, &params).unwrap();
        // Should produce more faces than the original 6
        assert!(
            result.faces.len() > 6,
            "Variable fillet should add faces, got {}",
            result.faces.len()
        );
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn variable_fillet_uniform_matches_constant() {
        // Uniform radius variable fillet should produce the same face count
        // as the constant fillet (though with more fillet strip quads due to
        // finer station sampling).
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = VariableFilletParams {
            edge_indices: vec![0],
            radius_points: vec![
                RadiusPoint { parameter: 0.0, radius: 1.0 },
                RadiusPoint { parameter: 1.0, radius: 1.0 },
            ],
            smooth_transition: false,
        };
        let result = variable_fillet_edges(&brep, &params).unwrap();
        assert!(result.faces.len() > 6);
        assert!(matches!(result.body, Body::Solid(_)));

        // The constant fillet for comparison
        let const_params = FilletParams {
            edge_indices: vec![0],
            radius: 1.0,
        };
        let const_result = fillet_edges(&brep, &const_params).unwrap();
        // Variable fillet has more stations so more fillet strip faces
        assert!(
            result.faces.len() >= const_result.faces.len(),
            "Variable fillet ({} faces) should have at least as many faces as constant ({} faces)",
            result.faces.len(),
            const_result.faces.len()
        );
    }

    #[test]
    fn variable_fillet_three_control_points() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = VariableFilletParams {
            edge_indices: vec![0],
            radius_points: vec![
                RadiusPoint { parameter: 0.0, radius: 0.5 },
                RadiusPoint { parameter: 0.5, radius: 2.0 },
                RadiusPoint { parameter: 1.0, radius: 0.5 },
            ],
            smooth_transition: true,
        };
        let result = variable_fillet_edges(&brep, &params).unwrap();
        assert!(result.faces.len() > 6);
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn variable_fillet_on_box_edge_produces_expected_faces() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = VariableFilletParams {
            edge_indices: vec![0],
            radius_points: vec![
                RadiusPoint { parameter: 0.0, radius: 1.0 },
                RadiusPoint { parameter: 1.0, radius: 2.0 },
            ],
            smooth_transition: false,
        };
        let result = variable_fillet_edges(&brep, &params).unwrap();
        // 6 original faces + (VARIABLE_FILLET_STATIONS+1) * FILLET_SEGMENTS fillet quads
        let expected_fillet_faces = (VARIABLE_FILLET_STATIONS + 1) * FILLET_SEGMENTS;
        let expected_total = 6 + expected_fillet_faces;
        assert_eq!(
            result.faces.len(),
            expected_total,
            "Variable fillet should have {} faces (6 original + {} fillet), got {}",
            expected_total,
            expected_fillet_faces,
            result.faces.len()
        );
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn variable_fillet_empty_brep_rejected() {
        let brep = BRep::new();
        let params = VariableFilletParams {
            edge_indices: vec![0],
            radius_points: vec![
                RadiusPoint { parameter: 0.0, radius: 1.0 },
                RadiusPoint { parameter: 1.0, radius: 2.0 },
            ],
            smooth_transition: false,
        };
        assert!(variable_fillet_edges(&brep, &params).is_err());
    }

    #[test]
    fn variable_fillet_invalid_edge_rejected() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = VariableFilletParams {
            edge_indices: vec![999],
            radius_points: vec![
                RadiusPoint { parameter: 0.0, radius: 1.0 },
                RadiusPoint { parameter: 1.0, radius: 2.0 },
            ],
            smooth_transition: false,
        };
        assert!(variable_fillet_edges(&brep, &params).is_err());
    }

    #[test]
    fn variable_fillet_too_few_control_points_rejected() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = VariableFilletParams {
            edge_indices: vec![0],
            radius_points: vec![RadiusPoint { parameter: 0.0, radius: 1.0 }],
            smooth_transition: false,
        };
        assert!(variable_fillet_edges(&brep, &params).is_err());
    }

    #[test]
    fn variable_fillet_negative_radius_rejected() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = VariableFilletParams {
            edge_indices: vec![0],
            radius_points: vec![
                RadiusPoint { parameter: 0.0, radius: -1.0 },
                RadiusPoint { parameter: 1.0, radius: 2.0 },
            ],
            smooth_transition: false,
        };
        assert!(variable_fillet_edges(&brep, &params).is_err());
    }

    #[test]
    fn variable_fillet_out_of_range_parameter_rejected() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = VariableFilletParams {
            edge_indices: vec![0],
            radius_points: vec![
                RadiusPoint { parameter: -0.5, radius: 1.0 },
                RadiusPoint { parameter: 1.0, radius: 2.0 },
            ],
            smooth_transition: false,
        };
        assert!(variable_fillet_edges(&brep, &params).is_err());
    }

    #[test]
    fn interpolate_radius_linear() {
        let pts = vec![
            RadiusPoint { parameter: 0.0, radius: 1.0 },
            RadiusPoint { parameter: 1.0, radius: 3.0 },
        ];
        assert!((interpolate_radius(&pts, 0.0, false) - 1.0).abs() < 1e-10);
        assert!((interpolate_radius(&pts, 0.5, false) - 2.0).abs() < 1e-10);
        assert!((interpolate_radius(&pts, 1.0, false) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn interpolate_radius_cubic_smooth() {
        let pts = vec![
            RadiusPoint { parameter: 0.0, radius: 1.0 },
            RadiusPoint { parameter: 0.5, radius: 3.0 },
            RadiusPoint { parameter: 1.0, radius: 1.0 },
        ];
        // At control points, should be close to specified values
        assert!((interpolate_radius(&pts, 0.0, true) - 1.0).abs() < 1e-10);
        assert!((interpolate_radius(&pts, 1.0, true) - 1.0).abs() < 1e-10);
        // Midpoint should be close to 3.0 (cubic passes through control points)
        assert!((interpolate_radius(&pts, 0.5, true) - 3.0).abs() < 1e-10);
    }
}
