use crate::error::{KernelError, KernelResult};
use crate::geometry::surface::plane::Plane;
use crate::geometry::{Pt3, Vec3};
use crate::topology::builders::{add_inner_loop_to_face, make_planar_face};
use crate::topology::body::Body;
use crate::topology::face::FaceId;
use crate::topology::shell::Shell;
use crate::topology::solid::Solid;
use crate::topology::BRep;

use super::extrude::{compute_centroid, EndCondition, ExtrudeParams, ExtrudeProfile, THROUGH_ALL_DEPTH};

pub type CutExtrudeParams = ExtrudeParams;

/// Cut-extrude: subtract a prismatic shape from an existing solid.
///
/// This is a "native cut" operation that directly constructs the result
/// by modifying the stock's coplanar faces (adding inner loops) and
/// adding cut side walls. Handles through-holes and blind pockets.
pub fn cut_extrude(
    mut stock: BRep,
    profile: &ExtrudeProfile,
    params: &CutExtrudeParams,
) -> KernelResult<BRep> {
    if stock.faces.is_empty() {
        return Err(KernelError::Operation {
            op: "cut_extrude".into(),
            detail: "Cannot cut: no existing geometry".into(),
        });
    }

    let n = profile.points.len();
    if n < 3 {
        return Err(KernelError::InvalidParameter {
            param: "profile".into(),
            value: format!("Need at least 3 points, got {}", n),
        });
    }

    let dir_norm = params.direction.normalize();

    // Step 1: Resolve effective depths (same logic as extrude_profile)
    let depth = match params.end_condition {
        EndCondition::Blind => params.depth,
        EndCondition::ThroughAll => THROUGH_ALL_DEPTH,
        EndCondition::UpToNext => params.up_to_next_depth.unwrap_or(params.depth),
        EndCondition::UpToSurface => params.up_to_next_depth.unwrap_or(params.depth),
        EndCondition::OffsetFromSurface => {
            params.up_to_next_depth.unwrap_or(params.depth) + params.surface_offset
        }
        EndCondition::UpToVertex => {
            if let Some(pos) = params.target_vertex_position {
                let v = Pt3::new(pos[0], pos[1], pos[2]);
                let centroid = compute_centroid(&profile.points, n);
                let dir = params.direction.normalize();
                let diff = v - centroid;
                let t = Vec3::new(diff.x, diff.y, diff.z).dot(&dir);
                t.max(0.001)
            } else {
                params.depth
            }
        }
    };
    let depth2_effective = match params.end_condition2 {
        EndCondition::Blind => params.depth2,
        EndCondition::ThroughAll => THROUGH_ALL_DEPTH,
        EndCondition::UpToNext => params.up_to_next_depth.unwrap_or(params.depth2),
        EndCondition::UpToSurface => params.up_to_next_depth.unwrap_or(params.depth2),
        EndCondition::OffsetFromSurface => {
            params.up_to_next_depth.unwrap_or(params.depth2) + params.surface_offset
        }
        EndCondition::UpToVertex => {
            if let Some(pos) = params.target_vertex_position {
                let v = Pt3::new(pos[0], pos[1], pos[2]);
                let centroid = compute_centroid(&profile.points, n);
                let dir = params.direction.normalize();
                let diff = v - centroid;
                let t = Vec3::new(diff.x, diff.y, diff.z).dot(&dir);
                t.max(0.001)
            } else {
                params.depth2
            }
        }
    };

    if depth <= 0.0 {
        return Err(KernelError::InvalidParameter {
            param: "depth".into(),
            value: depth.to_string(),
        });
    }

    // Step 2: Compute base offset (from_offset)
    let base_offset = dir_norm * params.from_offset;

    // Step 3: Compute bottom and top offsets (symmetric / direction2 / simple)
    let (bottom_offset, top_offset) = if params.symmetric {
        let half = depth / 2.0;
        (base_offset - dir_norm * half, base_offset + dir_norm * half)
    } else if params.direction2_enabled {
        (base_offset - dir_norm * depth2_effective.max(0.0), base_offset + dir_norm * depth)
    } else {
        (base_offset, base_offset + dir_norm * depth)
    };

    // Step 4: Compute cut bottom and top points with draft angle support

    // Un-tapered bottom points
    let cut_bottom_pts_untapered: Vec<Pt3> = profile
        .points
        .iter()
        .map(|p| p + bottom_offset)
        .collect();

    // Apply draft_angle2 taper to bottom points when direction2 is enabled
    let cut_bottom_pts: Vec<Pt3> =
        if params.direction2_enabled && !params.symmetric && params.draft_angle2.abs() > 1e-12 {
            let centroid = compute_centroid(&cut_bottom_pts_untapered, n);
            let draft_offset = depth2_effective.max(0.0) * params.draft_angle2.tan();
            cut_bottom_pts_untapered
                .iter()
                .map(|p| {
                    let to_centroid = Vec3::new(
                        centroid.x - p.x,
                        centroid.y - p.y,
                        centroid.z - p.z,
                    );
                    let dist = to_centroid.norm();
                    if dist > 1e-12 {
                        p + to_centroid.normalize() * draft_offset
                    } else {
                        *p
                    }
                })
                .collect()
        } else {
            cut_bottom_pts_untapered.clone()
        };

    // Compute un-tapered top reference points
    let cut_top_pts_base: Vec<Pt3> = profile
        .points
        .iter()
        .map(|p| p + top_offset)
        .collect();

    // Apply draft_angle taper to top points
    let cut_top_pts: Vec<Pt3> = if params.draft_angle.abs() > 1e-12 {
        // Use bottom profile position as reference for taper direction
        let profile_base: Vec<Pt3> = profile
            .points
            .iter()
            .map(|p| p + bottom_offset)
            .collect();
        let centroid = compute_centroid(&profile_base, n);

        let draft_offset = depth * params.draft_angle.tan();

        profile_base
            .iter()
            .map(|p| {
                let to_centroid = Vec3::new(
                    centroid.x - p.x,
                    centroid.y - p.y,
                    centroid.z - p.z,
                );
                let dist = to_centroid.norm();
                let inward = if dist > 1e-12 {
                    to_centroid.normalize() * draft_offset
                } else {
                    Vec3::new(0.0, 0.0, 0.0)
                };
                // Top vertex = bottom vertex + extrusion vector + inward taper
                p + top_offset - bottom_offset + inward
            })
            .collect()
    } else {
        cut_top_pts_base
    };

    // Find stock faces that are coplanar with the cut entry/exit planes
    let bottom_plane_origin = profile.plane.origin + bottom_offset;
    let top_plane_origin = profile.plane.origin + top_offset;

    let bottom_face_ids = faces_on_plane(&stock, bottom_plane_origin, profile.plane.normal, 1e-6);
    let top_face_ids = faces_on_plane(&stock, top_plane_origin, profile.plane.normal, 1e-6);

    if params.flip_side_to_cut {
        // Flip side to cut: remove coplanar faces and replace with cut profile faces.
        // This effectively keeps only the cut profile area, removing the exterior.
        for &face_id in bottom_face_ids.iter() {
            let _ = stock.faces.remove(face_id);
        }
        for &face_id in top_face_ids.iter() {
            let _ = stock.faces.remove(face_id);
        }

        // Add new faces using cut profile as the boundary
        let bottom_reversed: Vec<Pt3> = cut_bottom_pts.iter().rev().copied().collect();
        let bottom_plane = Plane {
            origin: profile.plane.origin + bottom_offset,
            normal: -profile.plane.normal,
            u_axis: profile.plane.u_axis,
            v_axis: profile.plane.v_axis,
        };
        let top_plane = Plane {
            origin: profile.plane.origin + top_offset,
            normal: profile.plane.normal,
            u_axis: profile.plane.u_axis,
            v_axis: profile.plane.v_axis,
        };
        make_planar_face(&mut stock, &bottom_reversed, bottom_plane)?;
        make_planar_face(&mut stock, &cut_top_pts, top_plane)?;

        // Side walls with NORMAL winding (outward, not reversed like regular cut)
        for i in 0..n {
            let j = (i + 1) % n;
            let p0 = cut_bottom_pts[i];
            let p1 = cut_bottom_pts[j];
            let p2 = cut_top_pts[j];
            let p3 = cut_top_pts[i];

            let edge_dir = (p1 - p0).normalize();
            let up_dir = (p3 - p0).normalize();
            let side_normal = edge_dir.cross(&up_dir).normalize();

            let side_plane = Plane {
                origin: p0,
                normal: side_normal,
                u_axis: edge_dir,
                v_axis: up_dir,
            };
            make_planar_face(&mut stock, &[p0, p1, p2, p3], side_plane)?;
        }
    } else {
        // Normal cut: add inner loops to coplanar faces + reversed side walls
        // Entry face (bottom) needs reversed winding for the inner loop
        let cut_bottom_pts_reversed: Vec<Pt3> = cut_bottom_pts.iter().rev().copied().collect();
        for &face_id in bottom_face_ids.iter() {
            add_inner_loop_to_face(&mut stock, face_id, &cut_bottom_pts_reversed)?;
        }
        for &face_id in top_face_ids.iter() {
            add_inner_loop_to_face(&mut stock, face_id, &cut_top_pts)?;
        }

        // For blind pockets cut from a stock face: add a cap at the far end.
        // This applies when:
        // - The entry plane IS coplanar with a stock face (inner loop added)
        // - The far end plane is NOT coplanar with any stock face (inside the solid)
        // This creates the "floor" of the pocket.
        if !bottom_face_ids.is_empty() && top_face_ids.is_empty() {
            // Pocket floor cap at the far end of the cut.
            // The pocket void is between entry (bottom) and this cap (top).
            // Outward normal from solid points BACK toward entry (opposite to cut direction).
            let top_reversed: Vec<Pt3> = cut_top_pts.iter().rev().copied().collect();
            let top_plane = Plane {
                origin: profile.plane.origin + top_offset,
                normal: -dir_norm,
                u_axis: profile.plane.u_axis,
                v_axis: profile.plane.v_axis,
            };
            make_planar_face(&mut stock, &top_reversed, top_plane)?;
        }
        // Mirror case: far end is on stock face but entry is not
        if bottom_face_ids.is_empty() && !top_face_ids.is_empty() {
            let bottom_plane = Plane {
                origin: profile.plane.origin + bottom_offset,
                normal: dir_norm,
                u_axis: profile.plane.u_axis,
                v_axis: profile.plane.v_axis,
            };
            make_planar_face(&mut stock, &cut_bottom_pts, bottom_plane)?;
        }

        // Add cut side walls (normals pointing INTO the cut — reversed from normal extrude)
        for i in 0..n {
            let j = (i + 1) % n;
            // Reversed winding: swap order so normals point inward
            let p0 = cut_bottom_pts[j];
            let p1 = cut_bottom_pts[i];
            let p2 = cut_top_pts[i];
            let p3 = cut_top_pts[j];

            let edge_dir = (p1 - p0).normalize();
            let up_dir = (p3 - p0).normalize();
            let side_normal = edge_dir.cross(&up_dir).normalize();

            let side_plane = Plane {
                origin: p0,
                normal: side_normal,
                u_axis: edge_dir,
                v_axis: up_dir,
            };
            make_planar_face(&mut stock, &[p0, p1, p2, p3], side_plane)?;
        }
    }

    // Rebuild shell and solid with all faces
    // Remove old shells/solids and create new ones
    let old_shell_ids: Vec<_> = stock.shells.iter().map(|(id, _)| id).collect();
    for id in old_shell_ids {
        let _ = stock.shells.remove(id);
    }
    let old_solid_ids: Vec<_> = stock.solids.iter().map(|(id, _)| id).collect();
    for id in old_solid_ids {
        let _ = stock.solids.remove(id);
    }

    let face_ids: Vec<_> = stock.faces.iter().map(|(id, _)| id).collect();
    let shell_id = stock.shells.insert(Shell::new(face_ids, true));
    let solid_id = stock.solids.insert(Solid::new(vec![shell_id]));
    stock.body = Body::Solid(solid_id);

    Ok(stock)
}

/// Find faces whose surface plane is coplanar with the given plane.
fn faces_on_plane(brep: &BRep, plane_origin: Pt3, plane_normal: Vec3, tol: f64) -> Vec<FaceId> {
    let mut result = Vec::new();
    for (face_id, face) in brep.faces.iter() {
        if let Some(surf_idx) = face.surface_index {
            if let Ok(normal) = brep.surfaces[surf_idx].normal_at(0.0, 0.0) {
                // Normals parallel or anti-parallel?
                if normal.dot(&plane_normal).abs() > 1.0 - tol {
                    // Is the face origin on the target plane?
                    if let Ok(face_origin) = brep.surfaces[surf_idx].point_at(0.0, 0.0) {
                        let dist = plane_normal.dot(&(face_origin - plane_origin)).abs();
                        if dist < tol {
                            result.push(face_id);
                        }
                    }
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::extrude::{extrude_profile, ExtrudeParams, ExtrudeProfile};

    fn make_stock() -> BRep {
        // 10x5 rectangle extruded by 3 along Z
        let profile = ExtrudeProfile {
            points: vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(10.0, 0.0, 0.0),
                Pt3::new(10.0, 5.0, 0.0),
                Pt3::new(0.0, 5.0, 0.0),
            ],
            plane: Plane::xy(0.0),
        };
        extrude_profile(&profile, &ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 3.0)).unwrap()
    }

    fn small_rect_profile() -> ExtrudeProfile {
        // 2x2 centered in the stock
        ExtrudeProfile {
            points: vec![
                Pt3::new(4.0, 1.5, 0.0),
                Pt3::new(6.0, 1.5, 0.0),
                Pt3::new(6.0, 3.5, 0.0),
                Pt3::new(4.0, 3.5, 0.0),
            ],
            plane: Plane::xy(0.0),
        }
    }

    #[test]
    fn cut_through_adds_side_walls() {
        let stock = make_stock();
        let profile = small_rect_profile();
        let params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 3.0);
        let result = cut_extrude(stock, &profile, &params).unwrap();
        // Stock had 6 faces. Cut adds 4 side walls. Top and bottom get inner loops (same face count).
        // So: 6 original + 4 cut sides = 10 faces
        assert_eq!(result.faces.len(), 10);
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn cut_empty_stock_fails() {
        let stock = BRep::new();
        let profile = small_rect_profile();
        let params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 3.0);
        assert!(cut_extrude(stock, &profile, &params).is_err());
    }

    #[test]
    fn cut_adds_inner_loops() {
        let stock = make_stock();
        let profile = small_rect_profile();
        let params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 3.0);
        let result = cut_extrude(stock, &profile, &params).unwrap();
        // Check that some faces have inner loops
        let faces_with_holes: Vec<_> = result
            .faces
            .iter()
            .filter(|(_, f)| !f.inner_loops.is_empty())
            .collect();
        assert!(
            faces_with_holes.len() >= 1,
            "Should have at least one face with a hole"
        );
    }

    #[test]
    fn cut_zero_depth_fails() {
        let stock = make_stock();
        let profile = small_rect_profile();
        let params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 0.0);
        assert!(cut_extrude(stock, &profile, &params).is_err());
    }

    #[test]
    fn cut_symmetric_creates_centered_cut() {
        let stock = make_stock(); // 10x5x3 box, z from 0 to 3
        // Profile on z=1.5 (mid-plane of the stock)
        let profile = ExtrudeProfile {
            points: vec![
                Pt3::new(4.0, 1.5, 1.5),
                Pt3::new(6.0, 1.5, 1.5),
                Pt3::new(6.0, 3.5, 1.5),
                Pt3::new(4.0, 3.5, 1.5),
            ],
            plane: Plane {
                origin: Pt3::new(5.0, 2.5, 1.5),
                normal: Vec3::new(0.0, 0.0, 1.0),
                u_axis: Vec3::new(1.0, 0.0, 0.0),
                v_axis: Vec3::new(0.0, 1.0, 0.0),
            },
        };
        let mut params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 2.0);
        params.symmetric = true;
        let result = cut_extrude(stock, &profile, &params).unwrap();
        // Stock had 6 faces + 4 cut side walls = 10 faces
        assert_eq!(result.faces.len(), 10);
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn cut_with_draft_creates_tapered_walls() {
        let stock = make_stock();
        let profile = small_rect_profile();
        let mut params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 3.0);
        params.draft_angle = 5.0_f64.to_radians();
        let result = cut_extrude(stock, &profile, &params).unwrap();
        // Same face count: 6 stock + 4 side walls = 10
        assert_eq!(result.faces.len(), 10);
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn cut_through_all_works() {
        let stock = make_stock();
        let profile = small_rect_profile();
        let mut params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 3.0);
        params.end_condition = EndCondition::ThroughAll;
        let result = cut_extrude(stock, &profile, &params).unwrap();
        assert!(result.faces.len() >= 10);
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn cut_direction2_bidirectional() {
        let stock = make_stock();
        // Profile at the middle of the stock (z=1.5) so the cut can go both ways
        let profile = ExtrudeProfile {
            points: vec![
                Pt3::new(4.0, 1.5, 1.5),
                Pt3::new(6.0, 1.5, 1.5),
                Pt3::new(6.0, 3.5, 1.5),
                Pt3::new(4.0, 3.5, 1.5),
            ],
            plane: Plane {
                origin: Pt3::new(5.0, 2.5, 1.5),
                normal: Vec3::new(0.0, 0.0, 1.0),
                u_axis: Vec3::new(1.0, 0.0, 0.0),
                v_axis: Vec3::new(0.0, 1.0, 0.0),
            },
        };
        let mut params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 1.5);
        params.direction2_enabled = true;
        params.depth2 = 1.5;
        let result = cut_extrude(stock, &profile, &params).unwrap();
        // Should have at least 10 faces (6 stock + 4 cut sides)
        assert!(result.faces.len() >= 10);
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn cut_direction2_with_draft() {
        let stock = make_stock();
        let profile = ExtrudeProfile {
            points: vec![
                Pt3::new(4.0, 1.5, 1.5),
                Pt3::new(6.0, 1.5, 1.5),
                Pt3::new(6.0, 3.5, 1.5),
                Pt3::new(4.0, 3.5, 1.5),
            ],
            plane: Plane {
                origin: Pt3::new(5.0, 2.5, 1.5),
                normal: Vec3::new(0.0, 0.0, 1.0),
                u_axis: Vec3::new(1.0, 0.0, 0.0),
                v_axis: Vec3::new(0.0, 1.0, 0.0),
            },
        };
        let mut params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 1.5);
        params.direction2_enabled = true;
        params.depth2 = 1.5;
        params.draft_angle = 5.0_f64.to_radians();
        params.draft_angle2 = 3.0_f64.to_radians();
        let result = cut_extrude(stock, &profile, &params).unwrap();
        assert!(result.faces.len() >= 10);
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn cut_flip_side_removes_exterior() {
        let stock = make_stock();
        let profile = small_rect_profile();
        let mut params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 3.0);
        params.flip_side_to_cut = true;
        let result = cut_extrude(stock, &profile, &params).unwrap();
        // Original stock has 6 faces. Flip side removes 2 coplanar faces (-2),
        // adds 2 new (+2), adds 4 side walls (+4) = 10
        assert!(
            result.faces.len() >= 8,
            "Flip side should create valid geometry, got {} faces",
            result.faces.len()
        );
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn cut_symmetric_ignores_direction2() {
        let stock = make_stock();
        let profile = ExtrudeProfile {
            points: vec![
                Pt3::new(4.0, 1.5, 1.5),
                Pt3::new(6.0, 1.5, 1.5),
                Pt3::new(6.0, 3.5, 1.5),
                Pt3::new(4.0, 3.5, 1.5),
            ],
            plane: Plane {
                origin: Pt3::new(5.0, 2.5, 1.5),
                normal: Vec3::new(0.0, 0.0, 1.0),
                u_axis: Vec3::new(1.0, 0.0, 0.0),
                v_axis: Vec3::new(0.0, 1.0, 0.0),
            },
        };
        let mut params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 2.0);
        params.symmetric = true;
        params.direction2_enabled = true; // should be ignored
        params.depth2 = 100.0;           // should be ignored
        let result = cut_extrude(stock, &profile, &params).unwrap();
        assert_eq!(result.faces.len(), 10);
    }
}
