use crate::error::{KernelError, KernelResult};
use crate::geometry::surface::plane::Plane;
use crate::geometry::{Pt3, Vec3};
use crate::topology::builders::{add_inner_loop_to_face, make_planar_face};
use crate::topology::body::Body;
use crate::topology::shell::Shell;
use crate::topology::solid::Solid;
use crate::topology::BRep;

use super::traits::Operation;

/// From condition: where extrusion starts.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FromCondition {
    #[default]
    SketchPlane,
    Offset,
    Surface,
    Vertex,
}

/// End condition for extrusion depth.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EndCondition {
    #[default]
    Blind,
    ThroughAll,
    UpToNext,
    UpToSurface,
    OffsetFromSurface,
    UpToVertex,
}

/// Depth used when end condition is ThroughAll (effectively infinite).
pub const THROUGH_ALL_DEPTH: f64 = 1e6;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtrudeParams {
    /// Direction of extrusion (unit vector)
    pub direction: Vec3,
    /// Depth of extrusion
    pub depth: f64,
    /// Whether to extrude symmetrically in both directions
    pub symmetric: bool,
    /// Draft angle in radians (for tapered extrusions)
    pub draft_angle: f64,
    /// End condition for the primary extrusion direction
    #[serde(default)]
    pub end_condition: EndCondition,
    /// Whether a second direction (reverse) extrusion is enabled
    #[serde(default)]
    pub direction2_enabled: bool,
    /// Depth of extrusion in the reverse direction
    #[serde(default)]
    pub depth2: f64,
    /// Draft angle in radians for the reverse direction
    #[serde(default)]
    pub draft_angle2: f64,
    /// End condition for the reverse direction
    #[serde(default)]
    pub end_condition2: EndCondition,
    /// Offset from the sketch plane before extruding
    #[serde(default)]
    pub from_offset: f64,
    /// Pre-computed depth for UpToNext end condition (set by the evaluator)
    #[serde(default)]
    pub up_to_next_depth: Option<f64>,
    /// Whether thin feature (shell) extrusion is enabled
    #[serde(default)]
    pub thin_feature: bool,
    /// Wall thickness for thin feature extrusion
    #[serde(default)]
    pub thin_wall_thickness: f64,
    /// Target face index for UpToSurface / OffsetFromSurface end conditions
    #[serde(default)]
    pub target_face_index: Option<usize>,
    /// Offset distance from the target surface (for OffsetFromSurface)
    #[serde(default)]
    pub surface_offset: f64,
    /// Target vertex position for UpToVertex end condition
    #[serde(default)]
    pub target_vertex_position: Option<[f64; 3]>,
    /// When true, flip the side to cut (cut exterior instead of interior)
    #[serde(default)]
    pub flip_side_to_cut: bool,
    /// When true, cap the inner ends of a thin feature extrusion
    #[serde(default)]
    pub cap_ends: bool,
    /// From condition: where the extrusion starts
    #[serde(default)]
    pub from_condition: FromCondition,
    /// Face index for From: Surface mode (evaluator resolves to from_offset)
    #[serde(default)]
    pub from_face_index: Option<usize>,
    /// Vertex position for From: Vertex mode (evaluator resolves to from_offset)
    #[serde(default)]
    pub from_vertex_position: Option<[f64; 3]>,
    /// Index of the contour to extrude when multiple closed loops exist
    #[serde(default)]
    pub contour_index: Option<usize>,
}

impl ExtrudeParams {
    /// Create a simple blind extrusion with no draft, no symmetry, no offset.
    pub fn blind(direction: Vec3, depth: f64) -> Self {
        ExtrudeParams {
            direction,
            depth,
            symmetric: false,
            draft_angle: 0.0,
            end_condition: EndCondition::Blind,
            direction2_enabled: false,
            depth2: 0.0,
            draft_angle2: 0.0,
            end_condition2: EndCondition::Blind,
            from_offset: 0.0,
            up_to_next_depth: None,
            thin_feature: false,
            thin_wall_thickness: 0.0,
            target_face_index: None,
            surface_offset: 0.0,
            target_vertex_position: None,
            flip_side_to_cut: false,
            cap_ends: false,
            from_condition: FromCondition::SketchPlane,
            from_face_index: None,
            from_vertex_position: None,
            contour_index: None,
        }
    }
}

/// Input profile for extrusion: an ordered list of 3D points forming a closed loop,
/// plus the plane they lie on.
#[derive(Debug, Clone)]
pub struct ExtrudeProfile {
    pub points: Vec<Pt3>,
    pub plane: Plane,
}

#[derive(Debug)]
pub struct ExtrudeOp;

impl Operation for ExtrudeOp {
    type Params = ExtrudeParams;

    fn execute(&self, params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        if params.depth <= 0.0 {
            return Err(KernelError::InvalidParameter {
                param: "depth".into(),
                value: params.depth.to_string(),
            });
        }
        // For now, delegate to extrude_profile for standalone use.
        // Full integration with input BRep (reading sketch profiles from it) comes later.
        Err(KernelError::Operation {
            op: "extrude".into(),
            detail: "Use extrude_profile() for standalone extrusion".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Extrude"
    }
}

/// Compute the centroid of a set of 3D points.
pub(crate) fn compute_centroid(pts: &[Pt3], n: usize) -> Pt3 {
    let sum = pts.iter().fold(Vec3::new(0.0, 0.0, 0.0), |acc, p| {
        acc + Vec3::new(p.x, p.y, p.z)
    });
    Pt3::new(sum.x / n as f64, sum.y / n as f64, sum.z / n as f64)
}

/// Offset a polygon inward by projecting to the plane's 2D coordinate system,
/// offsetting each edge inward, and recomputing vertex positions.
fn offset_polygon_inward(points: &[Pt3], plane: &Plane, distance: f64) -> KernelResult<Vec<Pt3>> {
    let n = points.len();
    if n < 3 {
        return Err(KernelError::InvalidParameter {
            param: "profile".into(),
            value: format!("Need at least 3 points, got {}", n),
        });
    }

    // Project to 2D
    let pts_2d: Vec<(f64, f64)> = points
        .iter()
        .map(|p| {
            let v = *p - plane.origin;
            (v.dot(&plane.u_axis), v.dot(&plane.v_axis))
        })
        .collect();

    // Compute signed area to determine winding direction
    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += pts_2d[i].0 * pts_2d[j].1 - pts_2d[j].0 * pts_2d[i].1;
    }
    let sign = if area > 0.0 { 1.0 } else { -1.0 }; // CCW = positive

    // For each edge, compute the inward normal and offset line
    let mut offset_lines: Vec<((f64, f64), (f64, f64))> = Vec::new(); // (point_on_line, direction)
    for i in 0..n {
        let j = (i + 1) % n;
        let dx = pts_2d[j].0 - pts_2d[i].0;
        let dy = pts_2d[j].1 - pts_2d[i].1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-12 {
            continue;
        } // skip degenerate edges
        // Inward normal: perpendicular to edge, pointing inward
        // For CCW polygon, inward normal of edge (dx,dy) is (dy, -dx) normalized
        let nx = sign * dy / len;
        let ny = sign * (-dx) / len;
        // Offset point on the line
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
        // Line intersection: p1 + t*d1 = p2 + s*d2
        let cross = d1.0 * d2.1 - d1.1 * d2.0;
        if cross.abs() < 1e-12 {
            // Parallel lines, use midpoint
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
        .map(|(u, v)| plane.origin + plane.u_axis * *u + plane.v_axis * *v)
        .collect())
}

/// Extrude a closed planar profile along a direction to create a solid BRep.
///
/// This is the core extrusion algorithm for the vertical slice.
/// Handles only linear edges (polygonal profiles).
///
/// Supports symmetric (mid-plane) extrusion and draft angle (taper).
/// - Symmetric: extrudes half the depth in each direction from the sketch plane.
/// - Draft angle (radians): positive = taper inward (top face smaller than bottom).
pub fn extrude_profile(profile: &ExtrudeProfile, params: &ExtrudeParams) -> KernelResult<BRep> {
    let direction = params.direction;

    // Step 1: Resolve effective depths (through_all override)
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
                let centroid = compute_centroid(&profile.points, profile.points.len());
                let dir = direction.normalize();
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
                let centroid = compute_centroid(&profile.points, profile.points.len());
                let dir = direction.normalize();
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
    let n = profile.points.len();
    if n < 3 {
        return Err(KernelError::InvalidParameter {
            param: "profile".into(),
            value: format!("Need at least 3 points, got {}", n),
        });
    }

    let dir_norm = direction.normalize();

    // Step 2: Compute base offset from from_offset
    let base_offset = match params.from_condition {
        FromCondition::SketchPlane => Vec3::new(0.0, 0.0, 0.0),
        FromCondition::Offset | FromCondition::Surface | FromCondition::Vertex => {
            dir_norm * params.from_offset
        }
    };

    // Step 3: Compute bottom and top offsets (factoring in symmetric OR direction2)
    let (bottom_offset, top_offset) = if params.symmetric {
        let half = depth / 2.0;
        (dir_norm * (-half), dir_norm * half)
    } else if params.direction2_enabled {
        // Direction 2: extrude in reverse for bottom, forward for top
        (-dir_norm * depth2_effective, dir_norm * depth)
    } else {
        (Vec3::new(0.0, 0.0, 0.0), dir_norm * depth)
    };

    // The extrusion height from bottom to top
    let extrude_height = depth;

    // Step 4: Build bottom_pts and top_pts with draft angles

    // Compute un-tapered bottom points (profile + bottom_offset + base_offset)
    let bottom_pts_untapered: Vec<Pt3> = profile
        .points
        .iter()
        .map(|p| p + bottom_offset + base_offset)
        .collect();

    // Apply draft_angle2 taper to bottom vertices when direction2 is enabled
    let bottom_pts: Vec<Pt3> =
        if params.direction2_enabled && !params.symmetric && params.draft_angle2.abs() > 1e-12 {
            let centroid = compute_centroid(&bottom_pts_untapered, n);
            let draft_offset = depth2_effective * params.draft_angle2.tan();
            bottom_pts_untapered
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
            bottom_pts_untapered.clone()
        };

    // Compute un-tapered top reference points (profile + top_offset + base_offset)
    let top_pts_base: Vec<Pt3> = profile
        .points
        .iter()
        .map(|p| p + top_offset + base_offset)
        .collect();

    // Apply draft_angle taper to top vertices
    let top_pts: Vec<Pt3> = if params.draft_angle.abs() > 1e-12 {
        // Centroid from the un-tapered bottom points at the profile plane level
        // (used as reference for inward taper direction)
        let profile_base: Vec<Pt3> = profile
            .points
            .iter()
            .map(|p| p + bottom_offset + base_offset)
            .collect();
        let centroid = compute_centroid(&profile_base, n);

        let draft_offset = extrude_height * params.draft_angle.tan();

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
        top_pts_base
    };

    let mut brep = BRep::new();

    // Bottom face: reversed winding for outward normal pointing down
    let bottom_face_points: Vec<Pt3> = bottom_pts.iter().rev().copied().collect();
    let bottom_normal = -profile.plane.normal;
    let bottom_plane = Plane {
        origin: profile.plane.origin + bottom_offset + base_offset,
        normal: bottom_normal,
        u_axis: profile.plane.u_axis,
        v_axis: profile.plane.v_axis,
    };
    let bottom_face_id = make_planar_face(&mut brep, &bottom_face_points, bottom_plane.clone())?;

    // Top face: use the (possibly tapered) top points
    let top_plane = Plane {
        origin: profile.plane.origin + top_offset + base_offset,
        normal: profile.plane.normal,
        u_axis: profile.plane.u_axis,
        v_axis: profile.plane.v_axis,
    };
    let top_face_id = make_planar_face(&mut brep, &top_pts, top_plane.clone())?;

    // Side faces: one quad per profile edge
    for i in 0..n {
        let j = (i + 1) % n;
        let p0 = bottom_pts[i];
        let p1 = bottom_pts[j];
        let p2 = top_pts[j];
        let p3 = top_pts[i];

        // Compute outward normal for this side face
        let edge_dir = (p1 - p0).normalize();
        let up_dir = (p3 - p0).normalize();
        let side_normal = edge_dir.cross(&up_dir).normalize();

        let side_plane = Plane {
            origin: p0,
            normal: side_normal,
            u_axis: edge_dir,
            v_axis: up_dir,
        };
        make_planar_face(&mut brep, &[p0, p1, p2, p3], side_plane)?;
    }

    // Thin feature: create inner walls to form a shell (hollow extrusion)
    if params.thin_feature && params.thin_wall_thickness > 0.0 {
        // Compute inner profile by offsetting the polygon inward
        let inner_bottom_pts =
            offset_polygon_inward(&bottom_pts, &bottom_plane, params.thin_wall_thickness)?;
        let inner_top_pts =
            offset_polygon_inward(&top_pts, &top_plane, params.thin_wall_thickness)?;

        let inner_n = inner_bottom_pts.len();

        // Add inner side walls (reversed winding for inward-facing normals)
        for i in 0..inner_n {
            let j = (i + 1) % inner_n;
            // Reversed: swap i,j so normals point inward
            let p0 = inner_bottom_pts[j];
            let p1 = inner_bottom_pts[i];
            let p2 = inner_top_pts[i];
            let p3 = inner_top_pts[j];

            let edge_dir = (p1 - p0).normalize();
            let up_dir = (p3 - p0).normalize();
            let side_normal = edge_dir.cross(&up_dir).normalize();

            let side_plane = Plane {
                origin: p0,
                normal: side_normal,
                u_axis: edge_dir,
                v_axis: up_dir,
            };
            make_planar_face(&mut brep, &[p0, p1, p2, p3], side_plane)?;
        }

        // Add inner loops to bottom and top faces to make them annular
        add_inner_loop_to_face(&mut brep, bottom_face_id, &inner_bottom_pts)?;
        add_inner_loop_to_face(&mut brep, top_face_id, &inner_top_pts)?;

        // Cap ends: add solid inner bottom and top caps to close the thin shell
        if params.cap_ends {
            let inner_bottom_plane = Plane {
                origin: bottom_plane.origin,
                normal: bottom_plane.normal,
                u_axis: bottom_plane.u_axis,
                v_axis: bottom_plane.v_axis,
            };
            let inner_top_plane = Plane {
                origin: top_plane.origin,
                normal: top_plane.normal,
                u_axis: top_plane.u_axis,
                v_axis: top_plane.v_axis,
            };
            // Add solid inner bottom cap (reversed winding for outward normal)
            let inner_bottom_reversed: Vec<Pt3> =
                inner_bottom_pts.iter().rev().copied().collect();
            make_planar_face(&mut brep, &inner_bottom_reversed, inner_bottom_plane)?;
            // Add solid inner top cap
            make_planar_face(&mut brep, &inner_top_pts, inner_top_plane)?;
        }
    }

    // Collect all faces into a shell and solid
    let face_ids: Vec<_> = brep.faces.iter().map(|(id, _)| id).collect();
    let shell_id = brep.shells.insert(Shell::new(face_ids, true));
    let solid_id = brep.solids.insert(Solid::new(vec![shell_id]));
    brep.body = Body::Solid(solid_id);

    Ok(brep)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;

    fn square_profile() -> ExtrudeProfile {
        ExtrudeProfile {
            points: vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(10.0, 0.0, 0.0),
                Pt3::new(10.0, 5.0, 0.0),
                Pt3::new(0.0, 5.0, 0.0),
            ],
            plane: Plane::xy(0.0),
        }
    }

    fn default_params(depth: f64) -> ExtrudeParams {
        ExtrudeParams {
            direction: Vec3::new(0.0, 0.0, 1.0),
            depth,
            symmetric: false,
            draft_angle: 0.0,
            end_condition: EndCondition::Blind,
            direction2_enabled: false,
            depth2: 0.0,
            draft_angle2: 0.0,
            end_condition2: EndCondition::Blind,
            from_offset: 0.0,
            up_to_next_depth: None,
            thin_feature: false,
            thin_wall_thickness: 0.0,
            target_face_index: None,
            surface_offset: 0.0,
            target_vertex_position: None,
            flip_side_to_cut: false,
            cap_ends: false,
            from_condition: FromCondition::SketchPlane,
            from_face_index: None,
            from_vertex_position: None,
            contour_index: None,
        }
    }

    #[test]
    fn extrude_square_creates_six_faces() {
        let profile = square_profile();
        let brep = extrude_profile(&profile, &default_params(3.0)).unwrap();
        assert_eq!(brep.faces.len(), 6, "Box should have 6 faces");
    }

    #[test]
    fn extrude_square_has_solid_body() {
        let profile = square_profile();
        let brep = extrude_profile(&profile, &default_params(3.0)).unwrap();
        assert!(matches!(brep.body, Body::Solid(_)));
    }

    #[test]
    fn extrude_faces_have_four_edges_each() {
        let profile = square_profile();
        let brep = extrude_profile(&profile, &default_params(3.0)).unwrap();
        for (_id, face) in brep.faces.iter() {
            let loop_id = face.outer_loop.unwrap();
            let loop_ = brep.loops.get(loop_id).unwrap();
            assert_eq!(loop_.len(), 4);
        }
    }

    #[test]
    fn extrude_zero_depth_rejected() {
        let profile = square_profile();
        assert!(extrude_profile(&profile, &default_params(0.0)).is_err());
    }

    #[test]
    fn extrude_negative_depth_rejected() {
        let profile = square_profile();
        assert!(extrude_profile(&profile, &default_params(-1.0)).is_err());
    }

    #[test]
    fn extrude_triangle() {
        let profile = ExtrudeProfile {
            points: vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(5.0, 0.0, 0.0),
                Pt3::new(2.5, 4.0, 0.0),
            ],
            plane: Plane::xy(0.0),
        };
        let brep = extrude_profile(&profile, &default_params(2.0)).unwrap();
        // Triangle extrusion: 2 caps + 3 sides = 5 faces
        assert_eq!(brep.faces.len(), 5);
    }

    #[test]
    fn extrude_symmetric_creates_six_faces() {
        let profile = square_profile();
        let params = ExtrudeParams {
            symmetric: true,
            ..default_params(6.0)
        };
        let brep = extrude_profile(&profile, &params).unwrap();
        assert_eq!(brep.faces.len(), 6, "Symmetric extrude of square should have 6 faces");
    }

    #[test]
    fn extrude_symmetric_vertices_centered() {
        let profile = square_profile();
        let depth = 6.0;
        let params = ExtrudeParams {
            symmetric: true,
            ..default_params(depth)
        };
        let brep = extrude_profile(&profile, &params).unwrap();

        // Collect all vertex Z values
        let mut z_values: Vec<f64> = brep
            .vertices
            .iter()
            .map(|(_, v)| v.point.z)
            .collect();
        z_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        z_values.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

        assert_eq!(z_values.len(), 2, "Should have exactly 2 distinct Z levels");
        assert!((z_values[0] - (-depth / 2.0)).abs() < 1e-9, "Bottom should be at -depth/2");
        assert!((z_values[1] - (depth / 2.0)).abs() < 1e-9, "Top should be at +depth/2");
    }

    #[test]
    fn extrude_with_draft_creates_six_faces() {
        let profile = square_profile();
        let params = ExtrudeParams {
            draft_angle: 5.0_f64.to_radians(),
            ..default_params(3.0)
        };
        let brep = extrude_profile(&profile, &params).unwrap();
        assert_eq!(brep.faces.len(), 6, "Draft extrude of square should have 6 faces");
    }

    #[test]
    fn extrude_symmetric_with_draft() {
        let profile = square_profile();
        let params = ExtrudeParams {
            symmetric: true,
            draft_angle: 5.0_f64.to_radians(),
            ..default_params(6.0)
        };
        let brep = extrude_profile(&profile, &params).unwrap();
        assert_eq!(brep.faces.len(), 6, "Symmetric+draft extrude should have 6 faces");
    }

    #[test]
    fn extrude_through_all_creates_six_faces() {
        let profile = square_profile();
        let params = ExtrudeParams {
            end_condition: EndCondition::ThroughAll,
            ..default_params(10.0) // depth ignored for through_all
        };
        let brep = extrude_profile(&profile, &params).unwrap();
        assert_eq!(brep.faces.len(), 6);
    }

    #[test]
    fn extrude_params_default_end_condition_is_blind() {
        let json = r#"{"direction":[0,0,1],"depth":5.0,"symmetric":false,"draft_angle":0.0}"#;
        let params: ExtrudeParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.end_condition, EndCondition::Blind);
        assert!(!params.direction2_enabled);
        assert_eq!(params.from_offset, 0.0);
    }

    #[test]
    fn extrude_direction2_asymmetric() {
        let profile = square_profile();
        let params = ExtrudeParams {
            direction2_enabled: true,
            depth2: 7.0,
            ..default_params(3.0)
        };
        let brep = extrude_profile(&profile, &params).unwrap();
        assert_eq!(brep.faces.len(), 6);
        // Verify Z range: bottom at -7, top at 3
        let z_values: Vec<f64> = brep.vertices.iter().map(|(_, v)| v.point.z).collect();
        let z_min = z_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let z_max = z_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!((z_min - (-7.0)).abs() < 1e-9);
        assert!((z_max - 3.0).abs() < 1e-9);
    }

    #[test]
    fn extrude_direction2_with_draft() {
        let profile = square_profile();
        let params = ExtrudeParams {
            direction2_enabled: true,
            depth2: 5.0,
            draft_angle: 5.0_f64.to_radians(),
            draft_angle2: 3.0_f64.to_radians(),
            ..default_params(5.0)
        };
        let brep = extrude_profile(&profile, &params).unwrap();
        assert_eq!(brep.faces.len(), 6);
    }

    #[test]
    fn extrude_direction2_ignored_when_symmetric() {
        let profile = square_profile();
        let params = ExtrudeParams {
            symmetric: true,
            direction2_enabled: true,
            depth2: 100.0,
            ..default_params(6.0)
        };
        let brep = extrude_profile(&profile, &params).unwrap();
        // Should be symmetric, not direction2
        let z_values: Vec<f64> = brep.vertices.iter().map(|(_, v)| v.point.z).collect();
        let z_min = z_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let z_max = z_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!((z_min - (-3.0)).abs() < 1e-9);
        assert!((z_max - 3.0).abs() < 1e-9);
    }

    #[test]
    fn extrude_from_offset_shifts_all_vertices() {
        let profile = square_profile();
        let offset = 5.0;
        let depth = 3.0;
        let params = ExtrudeParams {
            from_offset: offset,
            from_condition: FromCondition::Offset,
            ..default_params(depth)
        };
        let brep = extrude_profile(&profile, &params).unwrap();
        let z_values: Vec<f64> = brep.vertices.iter().map(|(_, v)| v.point.z).collect();
        let z_min = z_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let z_max = z_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!((z_min - offset).abs() < 1e-9);
        assert!((z_max - (offset + depth)).abs() < 1e-9);
    }

    #[test]
    fn extrude_up_to_next_uses_precomputed_depth() {
        let profile = square_profile();
        let params = ExtrudeParams {
            end_condition: EndCondition::UpToNext,
            up_to_next_depth: Some(7.5),
            ..default_params(10.0)
        };
        let brep = extrude_profile(&profile, &params).unwrap();
        assert_eq!(brep.faces.len(), 6);
        let z_values: Vec<f64> = brep.vertices.iter().map(|(_, v)| v.point.z).collect();
        let z_max = z_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!((z_max - 7.5).abs() < 1e-9);
    }

    #[test]
    fn extrude_up_to_next_fallback_to_depth() {
        let profile = square_profile();
        let params = ExtrudeParams {
            end_condition: EndCondition::UpToNext,
            up_to_next_depth: None, // no precomputed depth
            ..default_params(5.0)
        };
        let brep = extrude_profile(&profile, &params).unwrap();
        let z_values: Vec<f64> = brep.vertices.iter().map(|(_, v)| v.point.z).collect();
        let z_max = z_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!((z_max - 5.0).abs() < 1e-9);
    }

    #[test]
    fn extrude_thin_feature_creates_more_faces() {
        let profile = square_profile();
        let params = ExtrudeParams {
            thin_feature: true,
            thin_wall_thickness: 1.0,
            ..default_params(3.0)
        };
        let brep = extrude_profile(&profile, &params).unwrap();
        // 4 outer sides + 4 inner sides + 2 annular caps = 10 faces
        assert!(
            brep.faces.len() >= 10,
            "Thin feature should have at least 10 faces, got {}",
            brep.faces.len()
        );
    }

    #[test]
    fn extrude_thin_feature_disabled_normal_solid() {
        let profile = square_profile();
        let params = ExtrudeParams {
            thin_feature: false,
            thin_wall_thickness: 1.0,
            ..default_params(3.0)
        };
        let brep = extrude_profile(&profile, &params).unwrap();
        assert_eq!(brep.faces.len(), 6); // Normal solid
    }

    #[test]
    fn extrude_up_to_surface_uses_precomputed_depth() {
        let profile = square_profile();
        let mut params = default_params(10.0);
        params.end_condition = EndCondition::UpToSurface;
        params.up_to_next_depth = Some(5.0);
        let brep = extrude_profile(&profile, &params).unwrap();
        let z_max = brep
            .vertices
            .iter()
            .map(|(_, v)| v.point.z)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((z_max - 5.0).abs() < 1e-9);
    }

    #[test]
    fn extrude_offset_from_surface_adds_offset() {
        let profile = square_profile();
        let mut params = default_params(10.0);
        params.end_condition = EndCondition::OffsetFromSurface;
        params.up_to_next_depth = Some(5.0);
        params.surface_offset = 2.0;
        let brep = extrude_profile(&profile, &params).unwrap();
        let z_max = brep
            .vertices
            .iter()
            .map(|(_, v)| v.point.z)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((z_max - 7.0).abs() < 1e-9);
    }

    #[test]
    fn extrude_up_to_vertex_computes_depth() {
        let profile = square_profile();
        let mut params = default_params(10.0);
        params.end_condition = EndCondition::UpToVertex;
        params.target_vertex_position = Some([5.0, 2.5, 8.0]); // vertex at z=8
        let brep = extrude_profile(&profile, &params).unwrap();
        let z_max = brep
            .vertices
            .iter()
            .map(|(_, v)| v.point.z)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((z_max - 8.0).abs() < 1e-9);
    }

    #[test]
    fn extrude_cap_ends_adds_inner_caps() {
        let profile = square_profile();
        let mut params = default_params(3.0);
        params.thin_feature = true;
        params.thin_wall_thickness = 1.0;
        params.cap_ends = true;
        let brep = extrude_profile(&profile, &params).unwrap();
        // Without cap_ends: 2 annular caps + 4 outer sides + 4 inner sides = 10
        // With cap_ends: + 2 inner caps = 12
        assert!(
            brep.faces.len() >= 12,
            "Cap ends should add 2 inner cap faces, got {}",
            brep.faces.len()
        );
    }

    #[test]
    fn extrude_from_surface_uses_precomputed_offset() {
        let profile = square_profile();
        let mut params = default_params(3.0);
        params.from_condition = FromCondition::Surface;
        params.from_offset = 5.0; // evaluator would set this
        let brep = extrude_profile(&profile, &params).unwrap();
        let z_min = brep.vertices.iter().map(|(_, v)| v.point.z).fold(f64::INFINITY, f64::min);
        let z_max = brep.vertices.iter().map(|(_, v)| v.point.z).fold(f64::NEG_INFINITY, f64::max);
        assert!((z_min - 5.0).abs() < 1e-9);
        assert!((z_max - 8.0).abs() < 1e-9);
    }

    #[test]
    fn extrude_from_vertex_uses_precomputed_offset() {
        let profile = square_profile();
        let mut params = default_params(3.0);
        params.from_condition = FromCondition::Vertex;
        params.from_offset = 2.0;
        let brep = extrude_profile(&profile, &params).unwrap();
        let z_min = brep.vertices.iter().map(|(_, v)| v.point.z).fold(f64::INFINITY, f64::min);
        assert!((z_min - 2.0).abs() < 1e-9);
    }

    #[test]
    fn extrude_from_sketch_plane_no_offset() {
        let profile = square_profile();
        let mut params = default_params(3.0);
        params.from_condition = FromCondition::SketchPlane;
        params.from_offset = 99.0; // should be ignored
        let brep = extrude_profile(&profile, &params).unwrap();
        let z_min = brep.vertices.iter().map(|(_, v)| v.point.z).fold(f64::INFINITY, f64::min);
        assert!((z_min - 0.0).abs() < 1e-9); // starts at sketch plane, not offset
    }
}
