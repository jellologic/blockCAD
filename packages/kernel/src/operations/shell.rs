use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};
use crate::topology::body::Body;
use crate::topology::brep::BRep;

use super::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShellParams {
    /// Face indices to remove (creating openings)
    pub faces_to_remove: Vec<u32>,
    /// Wall thickness
    pub thickness: f64,
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

/// Offset a face polygon inward by `distance` using the face normal to define the plane.
fn offset_face_inward(points: &[Pt3], normal: &Vec3, distance: f64) -> KernelResult<Vec<Pt3>> {
    let n = points.len();
    if n < 3 {
        return Err(KernelError::InvalidParameter {
            param: "face".into(),
            value: format!("Need at least 3 points, got {}", n),
        });
    }

    // Build a local 2D coordinate system on the face plane
    let u_axis = (points[1] - points[0]).normalize();
    let v_axis = normal.cross(&u_axis).normalize();
    let origin = points[0];

    // Project to 2D
    let pts_2d: Vec<(f64, f64)> = points
        .iter()
        .map(|p| {
            let v = *p - origin;
            (v.dot(&u_axis), v.dot(&v_axis))
        })
        .collect();

    // Compute signed area to determine winding direction
    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += pts_2d[i].0 * pts_2d[j].1 - pts_2d[j].0 * pts_2d[i].1;
    }
    let sign = if area > 0.0 { 1.0 } else { -1.0 };

    // For each edge, compute offset line
    let mut offset_lines: Vec<((f64, f64), (f64, f64))> = Vec::new();
    for i in 0..n {
        let j = (i + 1) % n;
        let dx = pts_2d[j].0 - pts_2d[i].0;
        let dy = pts_2d[j].1 - pts_2d[i].1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-12 {
            continue;
        }
        // Inward normal perpendicular to edge
        let nx = sign * dy / len;
        let ny = sign * (-dx) / len;
        let ox = pts_2d[i].0 + nx * distance;
        let oy = pts_2d[i].1 + ny * distance;
        offset_lines.push(((ox, oy), (dx, dy)));
    }

    // Compute new vertices as intersections of adjacent offset lines
    let m = offset_lines.len();
    let mut new_pts_2d: Vec<(f64, f64)> = Vec::new();
    for i in 0..m {
        let j = (i + 1) % m;
        let (p1, d1) = offset_lines[i];
        let (p2, d2) = offset_lines[j];
        let cross = d1.0 * d2.1 - d1.1 * d2.0;
        if cross.abs() < 1e-12 {
            new_pts_2d.push(((p1.0 + p2.0) / 2.0, (p1.1 + p2.1) / 2.0));
        } else {
            let dx = p2.0 - p1.0;
            let dy = p2.1 - p1.1;
            let t = (dx * d2.1 - dy * d2.0) / cross;
            new_pts_2d.push((p1.0 + t * d1.0, p1.1 + t * d1.1));
        }
    }

    // Project back to 3D
    Ok(new_pts_2d
        .iter()
        .map(|(u, v)| origin + u_axis * *u + v_axis * *v)
        .collect())
}

/// Shell a solid by removing selected faces and hollowing out with wall thickness.
///
/// Algorithm:
/// 1. Extract all face polygons from input BRep
/// 2. Split into kept and removed faces by index
/// 3. For each kept face: create outer face (original) + inner face (offset inward, reversed normal)
/// 4. For each edge shared between a kept face and a removed face: create rim connecting outer→inner
/// 5. Rebuild BRep from all faces
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

    let mut result_faces: Vec<(Vec<Pt3>, Vec3)> = Vec::new();

    // For each kept face: add outer (original) + inner (offset inward, reversed normal)
    let mut inner_polygons: Vec<(usize, Vec<Pt3>)> = Vec::new(); // (original_index, inner_points)
    for &ki in &kept_indices {
        let (ref points, ref normal) = face_polygons[ki];

        // Outer face — keep as-is
        result_faces.push((points.clone(), *normal));

        // Inner face — offset inward by thickness, reverse normal
        let inner_points = offset_face_inward(points, normal, params.thickness)?;
        let inner_normal = Vec3::new(-normal.x, -normal.y, -normal.z);
        // Reverse winding for inner face (normals point inward)
        let mut reversed = inner_points.clone();
        reversed.reverse();
        result_faces.push((reversed, inner_normal));

        inner_polygons.push((ki, inner_points));
    }

    // Build rim faces at removed-face boundaries.
    // For each removed face, find its edges. Each edge that is shared with a kept face
    // needs a rim face connecting the outer edge to the inner edge.
    //
    // We use a simpler approach: for each removed face, iterate its edges.
    // Each edge of the removed face is shared with exactly one kept face.
    // Find that kept face's outer and inner edge positions to build the rim quad.
    let tol = 1e-6;
    let tol2 = tol * tol;

    for &ri in &removed_indices {
        let (ref removed_points, _) = face_polygons[ri];
        let rn = removed_points.len();

        for edge_idx in 0..rn {
            let r_start = removed_points[edge_idx];
            let r_end = removed_points[(edge_idx + 1) % rn];

            // Find the kept face that shares this edge
            let mut found_inner: Option<(Pt3, Pt3)> = None;

            for &(ki, ref inner_pts) in &inner_polygons {
                let (ref outer_pts, _) = face_polygons[ki];
                let kn = outer_pts.len();

                for k in 0..kn {
                    let o_start = outer_pts[k];
                    let o_end = outer_pts[(k + 1) % kn];

                    // Check if edge matches (same or reversed direction)
                    let match_same = dist2(r_start, o_start) < tol2 && dist2(r_end, o_end) < tol2;
                    let match_rev = dist2(r_start, o_end) < tol2 && dist2(r_end, o_start) < tol2;

                    if match_same || match_rev {
                        // Found matching kept face edge — get corresponding inner edge
                        let i_start = inner_pts[k];
                        let i_end = inner_pts[(k + 1) % kn];
                        if match_same {
                            found_inner = Some((i_start, i_end));
                        } else {
                            found_inner = Some((i_end, i_start));
                        }
                        break;
                    }
                }
                if found_inner.is_some() {
                    break;
                }
            }

            if let Some((i_start, i_end)) = found_inner {
                // Build rim quad: outer_start → outer_end → inner_end → inner_start
                // Normal points outward (same direction as the removed face was)
                let rim_points = vec![r_start, r_end, i_end, i_start];
                let edge1 = r_end - r_start;
                let edge2 = i_start - r_start;
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

    #[test]
    fn shell_box_one_face_removed() {
        let brep = build_box_brep(10.0, 5.0, 3.0).unwrap();
        let params = ShellParams {
            faces_to_remove: vec![1], // Remove top face (index 1 in build_box_brep)
            thickness: 0.5,
        };
        let result = shell_solid(&brep, &params).unwrap();
        // 5 outer + 5 inner + 4 rim = 14 faces
        assert_eq!(
            result.faces.len(),
            14,
            "Shell with 1 face removed should have 14 faces, got {}",
            result.faces.len()
        );
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn shell_box_no_faces_removed() {
        let brep = build_box_brep(10.0, 5.0, 3.0).unwrap();
        let params = ShellParams {
            faces_to_remove: vec![],
            thickness: 0.5,
        };
        let result = shell_solid(&brep, &params).unwrap();
        // 6 outer + 6 inner + 0 rim = 12 faces
        assert_eq!(
            result.faces.len(),
            12,
            "Shell with no faces removed should have 12 faces, got {}",
            result.faces.len()
        );
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn shell_empty_brep_rejected() {
        let brep = BRep::new();
        let params = ShellParams {
            faces_to_remove: vec![],
            thickness: 1.0,
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
        };
        let result = shell_solid(&brep, &params);
        assert!(result.is_err());
    }
}
