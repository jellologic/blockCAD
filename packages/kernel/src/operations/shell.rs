use std::collections::HashMap;

use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};
use crate::topology::body::Body;
use crate::topology::brep::BRep;

use super::traits::Operation;

/// Direction in which to apply shell offset.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ShellDirection {
    /// Shell inward (existing behavior, default). The outer surface stays put,
    /// inner surface is offset inward by `thickness`.
    Inward,
    /// Shell outward. The inner surface stays at the original position,
    /// outer surface is offset outward by `thickness`.
    Outward,
    /// Shell both directions. Outer surface moves out by `thickness/2`,
    /// inner surface moves in by `thickness/2`.
    Both,
}

impl Default for ShellDirection {
    fn default() -> Self {
        ShellDirection::Inward
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShellParams {
    /// Face indices to remove (creating openings)
    pub faces_to_remove: Vec<u32>,
    /// Wall thickness
    pub thickness: f64,
    /// Direction mode for the shell offset
    #[serde(default)]
    pub direction: ShellDirection,
}

#[derive(Debug)]
pub struct ShellOp;

impl Operation for ShellOp {
    type Params = ShellParams;

    fn execute(&self, params: &Self::Params, input: &BRep) -> KernelResult<BRep> {
        shell_solid(input, params)
    }

    fn name(&self) -> &'static str {
        "Shell"
    }
}

fn quantize(p: &Pt3) -> (i64, i64, i64) {
    let scale = 1e8;
    (
        (p.x * scale).round() as i64,
        (p.y * scale).round() as i64,
        (p.z * scale).round() as i64,
    )
}

/// Solve a 3x3 linear system Ax = b using Cramer's rule.
fn solve_3x3(a: [[f64; 3]; 3], b: [f64; 3]) -> Option<Pt3> {
    let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
    if det.abs() < 1e-12 {
        return None;
    }
    let inv_det = 1.0 / det;
    let x = (b[0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (b[1] * a[2][2] - a[1][2] * b[2])
        + a[0][2] * (b[1] * a[2][1] - a[1][1] * b[2]))
        * inv_det;
    let y = (a[0][0] * (b[1] * a[2][2] - a[1][2] * b[2])
        - b[0] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * b[2] - b[1] * a[2][0]))
        * inv_det;
    let z = (a[0][0] * (a[1][1] * b[2] - b[1] * a[2][1])
        - a[0][1] * (a[1][0] * b[2] - b[1] * a[2][0])
        + b[0] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]))
        * inv_det;
    Some(Pt3::new(x, y, z))
}

/// Compute a vertex position offset along face normals by signed distance `t`.
/// Positive `t` offsets inward (against normal), negative offsets outward (along normal).
/// This is the core helper used for both inner and outer offset surfaces.
fn compute_offset_vertex(base: &Pt3, normals: &[Vec3], t: f64) -> Pt3 {
    match normals.len() {
        0 => *base,
        1 => {
            let n = normals[0];
            Pt3::new(
                base.x - n.x * t,
                base.y - n.y * t,
                base.z - n.z * t,
            )
        }
        2 => {
            // Average of two normals
            let avg = (normals[0] + normals[1]).normalize();
            let cos_half = normals[0].dot(&avg).max(0.1);
            let offset = t / cos_half;
            Pt3::new(
                base.x - avg.x * offset,
                base.y - avg.y * offset,
                base.z - avg.z * offset,
            )
        }
        _ => {
            // 3+ normals: try 3-plane intersection
            let n0 = normals[0];
            let n1 = normals[1];
            let n2 = normals[2];
            let pv = Vec3::new(base.x, base.y, base.z);
            let d0 = n0.dot(&pv) - t;
            let d1 = n1.dot(&pv) - t;
            let d2 = n2.dot(&pv) - t;
            let a = [
                [n0.x, n0.y, n0.z],
                [n1.x, n1.y, n1.z],
                [n2.x, n2.y, n2.z],
            ];
            if let Some(pt) = solve_3x3(a, [d0, d1, d2]) {
                pt
            } else {
                // Fallback to average normal
                let mut avg = Vec3::new(0.0, 0.0, 0.0);
                for n in normals {
                    avg = avg + *n;
                }
                let avg = avg.normalize();
                let cos_half = normals[0].dot(&avg).max(0.1);
                let offset = t / cos_half;
                Pt3::new(
                    base.x - avg.x * offset,
                    base.y - avg.y * offset,
                    base.z - avg.z * offset,
                )
            }
        }
    }
}

/// Shell a solid by removing selected faces and hollowing out with wall thickness.
///
/// Algorithm (3-plane intersection approach):
/// 1. Build vertex→face-normals map from all kept faces
/// 2. Compute inner vertex positions via plane intersection
/// 3. Build inner faces (reversed winding) from inner positions
/// 4. Build rim faces at removed face boundaries
pub fn shell_solid(brep: &BRep, params: &ShellParams) -> KernelResult<BRep> {
    if matches!(brep.body, Body::Empty) {
        return Err(KernelError::Operation {
            op: "shell".into(),
            detail: "Cannot shell: no existing geometry".into(),
        });
    }

    if params.thickness <= 0.0 {
        return Err(KernelError::InvalidParameter {
            param: "thickness".into(),
            value: format!("Thickness must be positive, got {}", params.thickness),
        });
    }

    let face_polygons = extract_face_polygons(brep)?;
    if face_polygons.is_empty() {
        return Err(KernelError::Operation {
            op: "shell".into(),
            detail: "No faces found in BRep".into(),
        });
    }

    let remove_set: std::collections::HashSet<u32> = params.faces_to_remove.iter().copied().collect();
    let num_faces = face_polygons.len();

    // Validate face indices
    for &idx in &params.faces_to_remove {
        if idx as usize >= num_faces {
            return Err(KernelError::InvalidParameter {
                param: "faces_to_remove".into(),
                value: format!("Face index {} out of range (0..{})", idx, num_faces),
            });
        }
    }

    // Separate kept and removed faces
    let mut kept_indices: Vec<usize> = Vec::new();
    let mut removed_indices: Vec<usize> = Vec::new();
    for i in 0..num_faces {
        if remove_set.contains(&(i as u32)) {
            removed_indices.push(i);
        } else {
            kept_indices.push(i);
        }
    }

    // Build vertex→face-normals map from ALL faces (not just kept)
    let mut vertex_normals: HashMap<(i64, i64, i64), Vec<Vec3>> = HashMap::new();
    for i in 0..num_faces {
        let (ref points, ref normal) = face_polygons[i];
        for p in points {
            let key = quantize(p);
            let entry = vertex_normals.entry(key).or_insert_with(Vec::new);
            // Only add if not a near-duplicate normal
            let dominated = entry.iter().any(|n| {
                (n.x - normal.x).abs() < 1e-8
                    && (n.y - normal.y).abs() < 1e-8
                    && (n.z - normal.z).abs() < 1e-8
            });
            if !dominated {
                entry.push(*normal);
            }
        }
    }

    // Determine offset amounts based on direction mode.
    // `outer_t` is the signed offset for the outer shell surface (negative = outward along normal).
    // `inner_t` is the signed offset for the inner shell surface (positive = inward against normal).
    let (outer_t, inner_t) = match params.direction {
        ShellDirection::Inward => (0.0, params.thickness),
        ShellDirection::Outward => (-params.thickness, 0.0),
        ShellDirection::Both => (-params.thickness / 2.0, params.thickness / 2.0),
    };

    // Compute outer and inner positions for each unique vertex
    let mut outer_pos_map: HashMap<(i64, i64, i64), Pt3> = HashMap::new();
    let mut inner_pos_map: HashMap<(i64, i64, i64), Pt3> = HashMap::new();
    for (key, normals) in &vertex_normals {
        // Reconstruct approximate original position from key
        let orig = Pt3::new(
            key.0 as f64 / 1e8,
            key.1 as f64 / 1e8,
            key.2 as f64 / 1e8,
        );
        let outer = compute_offset_vertex(&orig, normals, outer_t);
        let inner = compute_offset_vertex(&orig, normals, inner_t);
        outer_pos_map.insert(*key, outer);
        inner_pos_map.insert(*key, inner);
    }

    let mut result_faces: Vec<(Vec<Pt3>, Vec3)> = Vec::new();

    // For each kept face: add outer + inner (reversed winding)
    for &ki in &kept_indices {
        let (ref points, ref normal) = face_polygons[ki];

        // Outer face — offset outward (identity for Inward mode)
        let outer_points: Vec<Pt3> = points
            .iter()
            .map(|p| {
                let key = quantize(p);
                outer_pos_map.get(&key).copied().unwrap_or_else(|| {
                    Pt3::new(
                        p.x - normal.x * outer_t,
                        p.y - normal.y * outer_t,
                        p.z - normal.z * outer_t,
                    )
                })
            })
            .collect();
        result_faces.push((outer_points, *normal));

        // Inner face — offset inward, reverse winding
        let inner_points: Vec<Pt3> = points
            .iter()
            .map(|p| {
                let key = quantize(p);
                inner_pos_map.get(&key).copied().unwrap_or_else(|| {
                    Pt3::new(
                        p.x - normal.x * inner_t,
                        p.y - normal.y * inner_t,
                        p.z - normal.z * inner_t,
                    )
                })
            })
            .collect();

        let inner_normal = Vec3::new(-normal.x, -normal.y, -normal.z);
        let mut reversed = inner_points;
        reversed.reverse();
        result_faces.push((reversed, inner_normal));
    }

    // Build rim faces at removed-face boundaries
    // For each removed face edge, find the matching kept face edge and connect outer→inner
    let tol2 = 1e-12;

    // Pre-compute outer and inner points for each kept face
    let kept_outer_pts: Vec<Vec<Pt3>> = kept_indices
        .iter()
        .map(|&ki| {
            let (ref points, ref normal) = face_polygons[ki];
            points
                .iter()
                .map(|p| {
                    let key = quantize(p);
                    outer_pos_map.get(&key).copied().unwrap_or_else(|| {
                        Pt3::new(
                            p.x - normal.x * outer_t,
                            p.y - normal.y * outer_t,
                            p.z - normal.z * outer_t,
                        )
                    })
                })
                .collect()
        })
        .collect();

    let kept_inner_pts: Vec<Vec<Pt3>> = kept_indices
        .iter()
        .map(|&ki| {
            let (ref points, ref normal) = face_polygons[ki];
            points
                .iter()
                .map(|p| {
                    let key = quantize(p);
                    inner_pos_map.get(&key).copied().unwrap_or_else(|| {
                        Pt3::new(
                            p.x - normal.x * inner_t,
                            p.y - normal.y * inner_t,
                            p.z - normal.z * inner_t,
                        )
                    })
                })
                .collect()
        })
        .collect();

    for &ri in &removed_indices {
        let (ref removed_points, _) = face_polygons[ri];
        let rn = removed_points.len();

        for edge_idx in 0..rn {
            let r_start = removed_points[edge_idx];
            let r_end = removed_points[(edge_idx + 1) % rn];

            // Find the kept face that shares this edge (match against original positions)
            let mut found_rim: Option<(Pt3, Pt3, Pt3, Pt3)> = None;

            for (idx, &ki) in kept_indices.iter().enumerate() {
                let (ref orig_pts, _) = face_polygons[ki];
                let ref outer_pts = kept_outer_pts[idx];
                let ref inner_pts = kept_inner_pts[idx];
                let kn = orig_pts.len();

                for k in 0..kn {
                    let o_start = orig_pts[k];
                    let o_end = orig_pts[(k + 1) % kn];

                    let match_same = dist2(r_start, o_start) < tol2 && dist2(r_end, o_end) < tol2;
                    let match_rev = dist2(r_start, o_end) < tol2 && dist2(r_end, o_start) < tol2;

                    if match_same || match_rev {
                        if match_same {
                            found_rim = Some((
                                outer_pts[k],
                                outer_pts[(k + 1) % kn],
                                inner_pts[k],
                                inner_pts[(k + 1) % kn],
                            ));
                        } else {
                            found_rim = Some((
                                outer_pts[(k + 1) % kn],
                                outer_pts[k],
                                inner_pts[(k + 1) % kn],
                                inner_pts[k],
                            ));
                        }
                        break;
                    }
                }
                if found_rim.is_some() {
                    break;
                }
            }

            if let Some((o_start, o_end, i_start, i_end)) = found_rim {
                let rim_points = vec![o_start, o_end, i_end, i_start];
                let edge1 = o_end - o_start;
                let edge2 = i_start - o_start;
                let rim_normal = edge1.cross(&edge2).normalize();
                result_faces.push((rim_points, rim_normal));
            }
        }
    }

    rebuild_brep_from_faces(&result_faces)
}

fn dist2(a: Pt3, b: Pt3) -> f64 {
    let d = a - b;
    d.x * d.x + d.y * d.y + d.z * d.z
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;
    use crate::tessellation::{tessellate_brep, TessellationParams};

    #[test]
    fn shell_box_one_face_removed() {
        let brep = build_box_brep(10.0, 5.0, 3.0).unwrap();
        let params = ShellParams {
            faces_to_remove: vec![1],
            thickness: 0.5,
            direction: ShellDirection::Inward,
        };
        let result = shell_solid(&brep, &params).unwrap();
        assert_eq!(
            result.faces.len(),
            14,
            "Shell with 1 face removed should have 14 faces, got {}",
            result.faces.len()
        );
        assert!(matches!(result.body, Body::Solid(_)));
        // Verify watertightness via tessellation
        let mesh = tessellate_brep(&result, &TessellationParams::default()).unwrap();
        mesh.validate().unwrap();
    }

    #[test]
    fn shell_box_no_faces_removed() {
        let brep = build_box_brep(10.0, 5.0, 3.0).unwrap();
        let params = ShellParams {
            faces_to_remove: vec![],
            thickness: 0.5,
            direction: ShellDirection::Inward,
        };
        let result = shell_solid(&brep, &params).unwrap();
        assert_eq!(
            result.faces.len(),
            12,
            "Shell with no faces removed should have 12 faces, got {}",
            result.faces.len()
        );
        assert!(matches!(result.body, Body::Solid(_)));
        let mesh = tessellate_brep(&result, &TessellationParams::default()).unwrap();
        mesh.validate().unwrap();
    }

    #[test]
    fn shell_empty_brep_rejected() {
        let brep = BRep::new();
        let params = ShellParams {
            faces_to_remove: vec![],
            thickness: 1.0,
            direction: ShellDirection::Inward,
        };
        let result = shell_solid(&brep, &params);
        assert!(result.is_err());
    }

    #[test]
    fn shell_invalid_face_index_rejected() {
        let brep = build_box_brep(10.0, 5.0, 3.0).unwrap();
        let params = ShellParams {
            faces_to_remove: vec![99],
            thickness: 1.0,
            direction: ShellDirection::Inward,
        };
        let result = shell_solid(&brep, &params);
        assert!(result.is_err());
    }

    #[test]
    fn shell_zero_thickness_rejected() {
        let brep = build_box_brep(10.0, 5.0, 3.0).unwrap();
        let params = ShellParams {
            faces_to_remove: vec![],
            thickness: 0.0,
            direction: ShellDirection::Inward,
        };
        let result = shell_solid(&brep, &params);
        assert!(result.is_err());
    }

    #[test]
    fn shell_default_direction_is_inward() {
        // Verify serde default works for backward compatibility
        let json = r#"{"faces_to_remove":[1],"thickness":0.5}"#;
        let params: ShellParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.direction, ShellDirection::Inward);
    }

    /// Helper: compute mesh volume via signed tetrahedron method.
    /// TriMesh stores flat f32 positions [x0,y0,z0,x1,...] and flat u32 indices [i0,i1,i2,...].
    fn mesh_volume(mesh: &crate::tessellation::mesh::TriMesh) -> f64 {
        let mut vol: f64 = 0.0;
        let pos = &mesh.positions;
        let idx = &mesh.indices;
        let tri_count = idx.len() / 3;
        for t in 0..tri_count {
            let ia = idx[t * 3] as usize;
            let ib = idx[t * 3 + 1] as usize;
            let ic = idx[t * 3 + 2] as usize;
            let ax = pos[ia * 3] as f64;
            let ay = pos[ia * 3 + 1] as f64;
            let az = pos[ia * 3 + 2] as f64;
            let bx = pos[ib * 3] as f64;
            let by = pos[ib * 3 + 1] as f64;
            let bz = pos[ib * 3 + 2] as f64;
            let cx = pos[ic * 3] as f64;
            let cy = pos[ic * 3 + 1] as f64;
            let cz = pos[ic * 3 + 2] as f64;
            vol += (ax * (by * cz - bz * cy)
                + ay * (bz * cx - bx * cz)
                + az * (bx * cy - by * cx))
                / 6.0;
        }
        vol.abs()
    }

    #[test]
    fn shell_inward_box_volume_smaller_than_original() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let original_mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        let original_vol = mesh_volume(&original_mesh);

        let params = ShellParams {
            faces_to_remove: vec![1],
            thickness: 1.0,
            direction: ShellDirection::Inward,
        };
        let result = shell_solid(&brep, &params).unwrap();
        let mesh = tessellate_brep(&result, &TessellationParams::default()).unwrap();
        mesh.validate().unwrap();
        let shell_vol = mesh_volume(&mesh);

        assert!(
            shell_vol < original_vol,
            "Inward shell volume ({}) should be less than original ({})",
            shell_vol,
            original_vol
        );
    }

    #[test]
    fn shell_outward_box_one_face_removed() {
        let brep = build_box_brep(10.0, 5.0, 3.0).unwrap();
        let params = ShellParams {
            faces_to_remove: vec![1],
            thickness: 0.5,
            direction: ShellDirection::Outward,
        };
        let result = shell_solid(&brep, &params).unwrap();
        assert_eq!(
            result.faces.len(),
            14,
            "Outward shell with 1 face removed should have 14 faces, got {}",
            result.faces.len()
        );
        assert!(matches!(result.body, Body::Solid(_)));
        let mesh = tessellate_brep(&result, &TessellationParams::default()).unwrap();
        mesh.validate().unwrap();
    }

    #[test]
    fn shell_outward_volume_larger_than_inward() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();

        let inward_params = ShellParams {
            faces_to_remove: vec![1],
            thickness: 1.0,
            direction: ShellDirection::Inward,
        };
        let inward_result = shell_solid(&brep, &inward_params).unwrap();
        let inward_mesh = tessellate_brep(&inward_result, &TessellationParams::default()).unwrap();
        inward_mesh.validate().unwrap();
        let inward_vol = mesh_volume(&inward_mesh);

        let outward_params = ShellParams {
            faces_to_remove: vec![1],
            thickness: 1.0,
            direction: ShellDirection::Outward,
        };
        let outward_result = shell_solid(&brep, &outward_params).unwrap();
        let outward_mesh = tessellate_brep(&outward_result, &TessellationParams::default()).unwrap();
        outward_mesh.validate().unwrap();
        let outward_vol = mesh_volume(&outward_mesh);

        assert!(
            outward_vol > inward_vol,
            "Outward shell volume ({}) should be greater than inward shell volume ({})",
            outward_vol,
            inward_vol
        );
    }

    #[test]
    fn shell_both_box_one_face_removed() {
        let brep = build_box_brep(10.0, 5.0, 3.0).unwrap();
        let params = ShellParams {
            faces_to_remove: vec![1],
            thickness: 1.0,
            direction: ShellDirection::Both,
        };
        let result = shell_solid(&brep, &params).unwrap();
        assert_eq!(
            result.faces.len(),
            14,
            "Both shell with 1 face removed should have 14 faces, got {}",
            result.faces.len()
        );
        assert!(matches!(result.body, Body::Solid(_)));
        let mesh = tessellate_brep(&result, &TessellationParams::default()).unwrap();
        mesh.validate().unwrap();
    }

    #[test]
    fn shell_both_volume_between_inward_and_outward() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();

        let inward_params = ShellParams {
            faces_to_remove: vec![1],
            thickness: 2.0,
            direction: ShellDirection::Inward,
        };
        let inward_result = shell_solid(&brep, &inward_params).unwrap();
        let inward_mesh = tessellate_brep(&inward_result, &TessellationParams::default()).unwrap();
        let inward_vol = mesh_volume(&inward_mesh);

        let outward_params = ShellParams {
            faces_to_remove: vec![1],
            thickness: 2.0,
            direction: ShellDirection::Outward,
        };
        let outward_result = shell_solid(&brep, &outward_params).unwrap();
        let outward_mesh = tessellate_brep(&outward_result, &TessellationParams::default()).unwrap();
        let outward_vol = mesh_volume(&outward_mesh);

        let both_params = ShellParams {
            faces_to_remove: vec![1],
            thickness: 2.0,
            direction: ShellDirection::Both,
        };
        let both_result = shell_solid(&brep, &both_params).unwrap();
        let both_mesh = tessellate_brep(&both_result, &TessellationParams::default()).unwrap();
        let both_vol = mesh_volume(&both_mesh);

        assert!(
            inward_vol < both_vol && both_vol < outward_vol,
            "Expected inward ({}) < both ({}) < outward ({})",
            inward_vol,
            both_vol,
            outward_vol
        );
    }

    #[test]
    fn shell_outward_no_faces_removed() {
        let brep = build_box_brep(10.0, 5.0, 3.0).unwrap();
        let params = ShellParams {
            faces_to_remove: vec![],
            thickness: 0.5,
            direction: ShellDirection::Outward,
        };
        let result = shell_solid(&brep, &params).unwrap();
        assert_eq!(result.faces.len(), 12);
        assert!(matches!(result.body, Body::Solid(_)));
        let mesh = tessellate_brep(&result, &TessellationParams::default()).unwrap();
        mesh.validate().unwrap();
    }

    #[test]
    fn shell_both_no_faces_removed() {
        let brep = build_box_brep(10.0, 5.0, 3.0).unwrap();
        let params = ShellParams {
            faces_to_remove: vec![],
            thickness: 1.0,
            direction: ShellDirection::Both,
        };
        let result = shell_solid(&brep, &params).unwrap();
        assert_eq!(result.faces.len(), 12);
        assert!(matches!(result.body, Body::Solid(_)));
        let mesh = tessellate_brep(&result, &TessellationParams::default()).unwrap();
        mesh.validate().unwrap();
    }
}
