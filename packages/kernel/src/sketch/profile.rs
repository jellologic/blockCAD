use crate::error::{KernelError, KernelResult};
use crate::geometry::surface::Surface;
use crate::operations::extrude::ExtrudeProfile;
use crate::solver::graph::ConstraintGraph;

use super::entity::{SketchEntity, SketchEntityId};
use super::sketch::Sketch;
use super::variable_map::VariableMap;

/// Number of segments used to approximate an arc in the profile polyline.
const ARC_SEGMENTS: usize = 8;
/// Number of segments used to approximate a full circle in the profile polyline.
const CIRCLE_SEGMENTS: usize = 32;

/// A profile edge: either a straight line or an arc between two endpoints.
#[derive(Debug, Clone)]
enum EdgeKind {
    Line,
    Arc { center: SketchEntityId },
}

#[derive(Debug, Clone)]
struct ProfileEdge {
    start: SketchEntityId,
    end: SketchEntityId,
    kind: EdgeKind,
}

/// Read 2D position of a point from the solved constraint graph.
fn read_point_2d(
    pt_id: SketchEntityId,
    var_map: &VariableMap,
    graph: &ConstraintGraph,
) -> KernelResult<(f64, f64)> {
    let (x_var, y_var) = var_map.point_vars(pt_id).ok_or_else(|| {
        KernelError::Internal(format!("No variable mapping for point {:?}", pt_id))
    })?;
    Ok((graph.variables.value(x_var), graph.variables.value(y_var)))
}

/// Extract a closed profile from a solved sketch, suitable for extrusion.
///
/// Walks the sketch's line and arc entities to find a closed loop, reads solved
/// positions from the constraint graph, and projects them onto the sketch's
/// 3D plane. Arcs are sampled into polyline segments.
pub fn extract_profile(
    sketch: &Sketch,
    var_map: &VariableMap,
    graph: &ConstraintGraph,
) -> KernelResult<ExtrudeProfile> {
    // Collect all non-construction edges (lines and arcs)
    let mut edges: Vec<ProfileEdge> = Vec::new();

    for (id, entity) in sketch.entities.iter() {
        if sketch.is_construction(id.index() as usize) {
            continue;
        }
        match entity {
            SketchEntity::Line { start, end } => {
                edges.push(ProfileEdge {
                    start: *start,
                    end: *end,
                    kind: EdgeKind::Line,
                });
            }
            SketchEntity::Arc { center, start, end } => {
                edges.push(ProfileEdge {
                    start: *start,
                    end: *end,
                    kind: EdgeKind::Arc { center: *center },
                });
            }
            SketchEntity::Circle { center, radius } => {
                // A full circle is a single-edge closed loop.
                // We handle it specially: generate a complete polyline profile.
                if *radius > 0.0 {
                    // If there are no other edges, this circle IS the profile.
                    // We'll check for this after collecting all edges.
                    // For now, mark it; we'll handle standalone circles below.
                }
                // Circles don't have start/end points in the edge-chaining sense,
                // so we handle them as a special case after edge collection.
            }
            _ => {} // Points, splines, ellipses don't form profile edges (yet)
        }
    }

    // Special case: standalone circle profile (no lines or arcs)
    if edges.is_empty() {
        // Look for a non-construction circle
        for (id, entity) in sketch.entities.iter() {
            if sketch.is_construction(id.index() as usize) {
                continue;
            }
            if let SketchEntity::Circle { center, radius } = entity {
                if *radius > 0.0 {
                    let (cx, cy) = read_point_2d(*center, var_map, graph)?;
                    // Check if there's a radius variable (solved value)
                    let r = if let Some(r_var) = var_map.circle_radius_var(id) {
                        graph.variables.value(r_var)
                    } else {
                        *radius
                    };
                    // Sample a full circle
                    let mut points_3d = Vec::with_capacity(CIRCLE_SEGMENTS);
                    for i in 0..CIRCLE_SEGMENTS {
                        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (CIRCLE_SEGMENTS as f64);
                        let x = cx + r * angle.cos();
                        let y = cy + r * angle.sin();
                        points_3d.push(sketch.plane.point_at(x, y)?);
                    }
                    return Ok(ExtrudeProfile {
                        points: points_3d,
                        plane: sketch.plane.clone(),
                    });
                }
            }
        }
        return Err(KernelError::Operation {
            op: "extract_profile".into(),
            detail: "No line, arc, or circle entities in sketch".into(),
        });
    }

    // Chain edges into a closed loop
    let mut loop_edges: Vec<(ProfileEdge, bool)> = Vec::new(); // (edge, reversed)
    let mut used = vec![false; edges.len()];

    // Start with the first edge
    used[0] = true;
    loop_edges.push((edges[0].clone(), false));
    let first_start = edges[0].start;
    let mut current_end = edges[0].end;

    for _ in 0..edges.len() {
        if current_end == first_start {
            break; // Loop closed
        }

        let mut found = false;
        for (i, edge) in edges.iter().enumerate() {
            if used[i] {
                continue;
            }
            if edge.start == current_end {
                used[i] = true;
                loop_edges.push((edge.clone(), false));
                current_end = edge.end;
                found = true;
                break;
            }
            if edge.end == current_end {
                used[i] = true;
                loop_edges.push((edge.clone(), true)); // reversed
                current_end = edge.start;
                found = true;
                break;
            }
        }

        if !found {
            return Err(KernelError::Operation {
                op: "extract_profile".into(),
                detail: "Could not find a closed loop of edges".into(),
            });
        }
    }

    if current_end != first_start {
        return Err(KernelError::Operation {
            op: "extract_profile".into(),
            detail: "Edges do not form a closed loop".into(),
        });
    }

    // Emit 3D points from the loop
    let mut points_3d = Vec::new();

    for (edge, reversed) in &loop_edges {
        let (edge_start, edge_end) = if *reversed {
            (edge.end, edge.start)
        } else {
            (edge.start, edge.end)
        };

        match &edge.kind {
            EdgeKind::Line => {
                // Emit the start point of this line
                let (x, y) = read_point_2d(edge_start, var_map, graph)?;
                points_3d.push(sketch.plane.point_at(x, y)?);
            }
            EdgeKind::Arc { center } => {
                // Emit the start point, then sample intermediate arc points
                let (sx, sy) = read_point_2d(edge_start, var_map, graph)?;
                let (ex, ey) = read_point_2d(edge_end, var_map, graph)?;
                let (cx, cy) = read_point_2d(*center, var_map, graph)?;

                // Emit start point
                points_3d.push(sketch.plane.point_at(sx, sy)?);

                // Compute angles
                let radius = ((sx - cx).powi(2) + (sy - cy).powi(2)).sqrt();
                let mut start_angle = (sy - cy).atan2(sx - cx);
                let mut end_angle = (ey - cy).atan2(ex - cx);

                // Ensure we go the right way around the arc
                // If reversed, we need to swap the direction
                if *reversed {
                    std::mem::swap(&mut start_angle, &mut end_angle);
                }

                // Normalize: ensure we traverse the shorter arc (< 2π)
                // by making end_angle > start_angle
                while end_angle <= start_angle {
                    end_angle += 2.0 * std::f64::consts::PI;
                }
                // If the arc is more than a full circle, take the shorter path
                if end_angle - start_angle > 2.0 * std::f64::consts::PI {
                    end_angle -= 2.0 * std::f64::consts::PI;
                }

                // Sample intermediate points (exclude start, exclude end — end is the next edge's start)
                for i in 1..ARC_SEGMENTS {
                    let t = i as f64 / ARC_SEGMENTS as f64;
                    let angle = start_angle + t * (end_angle - start_angle);
                    let x = cx + radius * angle.cos();
                    let y = cy + radius * angle.sin();
                    points_3d.push(sketch.plane.point_at(x, y)?);
                }
            }
        }
    }

    Ok(ExtrudeProfile {
        points: points_3d,
        plane: sketch.plane.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;
    use crate::geometry::Pt2;
    use crate::sketch::constraint::{Constraint, ConstraintKind};
    use crate::sketch::solver_bridge::build_constraint_graph;
    use crate::solver::newton_raphson::{solve, SolverConfig};

    fn make_rectangle_sketch(plane: Plane, w: f64, h: f64) -> Sketch {
        let mut sketch = Sketch::new(plane);

        let p0 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(w * 0.8, 0.1),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(w * 0.8, h * 0.8),
        });
        let p3 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.1, h * 0.8),
        });

        let bottom = sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
        let right = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
        let top = sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
        let left = sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });

        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![bottom]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![top]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![right]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![left]));
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Distance { value: w },
            vec![p0, p1],
        ));
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Distance { value: h },
            vec![p1, p2],
        ));

        sketch
    }

    fn solve_sketch(sketch: &Sketch) -> (ConstraintGraph, VariableMap) {
        let (mut graph, var_map) = build_constraint_graph(sketch).unwrap();
        let result = solve(&mut graph, &SolverConfig::default()).unwrap();
        assert!(result.converged);
        (graph, var_map)
    }

    #[test]
    fn test_extract_rectangle_profile() {
        let sketch = make_rectangle_sketch(Plane::xy(0.0), 10.0, 5.0);
        let (graph, var_map) = solve_sketch(&sketch);
        let profile = extract_profile(&sketch, &var_map, &graph).unwrap();

        assert_eq!(profile.points.len(), 4, "Rectangle should have 4 points");

        // Verify the solved positions form a 10x5 rectangle at z=0
        let pts = &profile.points;
        assert!((pts[0].x - 0.0).abs() < 1e-6);
        assert!((pts[0].y - 0.0).abs() < 1e-6);
        assert!((pts[0].z - 0.0).abs() < 1e-6);

        assert!((pts[1].x - 10.0).abs() < 1e-6);
        assert!((pts[1].y - 0.0).abs() < 1e-6);

        assert!((pts[2].x - 10.0).abs() < 1e-6);
        assert!((pts[2].y - 5.0).abs() < 1e-6);

        assert!((pts[3].x - 0.0).abs() < 1e-6);
        assert!((pts[3].y - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_extract_triangle_profile() {
        let mut sketch = Sketch::new(Plane::xy(0.0));

        let p0 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(2.5, 4.0),
        });

        sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
        sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
        sketch.add_entity(SketchEntity::Line { start: p2, end: p0 });

        // Fix all points (fully constrained triangle)
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p1]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p2]));

        let (graph, var_map) = solve_sketch(&sketch);
        let profile = extract_profile(&sketch, &var_map, &graph).unwrap();

        assert_eq!(profile.points.len(), 3, "Triangle should have 3 points");
    }

    #[test]
    fn test_profile_on_yz_plane() {
        use crate::geometry::Vec3;

        let yz_plane = Plane {
            origin: crate::geometry::Pt3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(1.0, 0.0, 0.0), // normal along X
            u_axis: Vec3::new(0.0, 1.0, 0.0),  // u along Y
            v_axis: Vec3::new(0.0, 0.0, 1.0),  // v along Z
        };
        let sketch = make_rectangle_sketch(yz_plane, 10.0, 5.0);
        let (graph, var_map) = solve_sketch(&sketch);
        let profile = extract_profile(&sketch, &var_map, &graph).unwrap();

        assert_eq!(profile.points.len(), 4);
        // On YZ plane: x should be 0, y maps to u, z maps to v
        let pts = &profile.points;
        assert!((pts[0].x - 0.0).abs() < 1e-6, "Should be on YZ plane");
        assert!((pts[1].y - 10.0).abs() < 1e-6, "Width=10 along Y axis");
        assert!((pts[2].z - 5.0).abs() < 1e-6, "Height=5 along Z axis");
    }

    #[test]
    fn test_extract_profile_with_arc() {
        // Create a "D-shape": line on the left, arc on the right
        // Two points connected by a line and an arc
        let mut sketch = Sketch::new(Plane::xy(0.0));

        // Arc center at origin, start at (0, -5), end at (0, 5)
        let center = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p_start = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, -5.0),
        });
        let p_end = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 5.0),
        });

        // Line from p_end back to p_start (left side, vertical)
        sketch.add_entity(SketchEntity::Line { start: p_end, end: p_start });
        // Arc from p_start to p_end around center (right side, semicircle)
        sketch.add_entity(SketchEntity::Arc { center, start: p_start, end: p_end });

        // Fix all points
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![center]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p_start]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p_end]));

        let (graph, var_map) = solve_sketch(&sketch);
        let profile = extract_profile(&sketch, &var_map, &graph).unwrap();

        // Line contributes 1 point (start), arc contributes 1 start + 7 intermediate = 8
        // Total: 1 (line start) + 1 (arc start) + 7 (arc intermediate) = 9
        assert!(profile.points.len() > 2, "Profile with arc should have sampled points, got {}", profile.points.len());
        assert!(profile.points.len() <= 2 + ARC_SEGMENTS, "Too many points");
    }

    #[test]
    fn test_extract_circle_profile() {
        // Single circle centered at origin, radius 5
        let mut sketch = Sketch::new(Plane::xy(0.0));

        let center = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        sketch.add_entity(SketchEntity::Circle { center, radius: 5.0 });

        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![center]));

        let (graph, var_map) = solve_sketch(&sketch);
        let profile = extract_profile(&sketch, &var_map, &graph).unwrap();

        assert_eq!(profile.points.len(), CIRCLE_SEGMENTS, "Circle profile should have {} points", CIRCLE_SEGMENTS);

        // All points should be at distance 5 from origin
        for pt in &profile.points {
            let dist = (pt.x * pt.x + pt.y * pt.y).sqrt();
            assert!((dist - 5.0).abs() < 1e-6, "Point should be on circle, dist={}", dist);
        }
    }

    #[test]
    fn test_extract_profile_mixed_lines_and_arcs() {
        // Rounded rectangle: 4 lines + 4 corner arcs
        // This tests chaining heterogeneous edge types.
        let mut sketch = Sketch::new(Plane::xy(0.0));

        let r = 1.0; // corner radius
        let w = 10.0;
        let h = 6.0;

        // Corner arc centers (inside the rectangle corners)
        let c_bl = sketch.add_entity(SketchEntity::Point { position: Pt2::new(r, r) });
        let c_br = sketch.add_entity(SketchEntity::Point { position: Pt2::new(w - r, r) });
        let c_tr = sketch.add_entity(SketchEntity::Point { position: Pt2::new(w - r, h - r) });
        let c_tl = sketch.add_entity(SketchEntity::Point { position: Pt2::new(r, h - r) });

        // Points where arcs meet lines (8 tangent points)
        // Bottom-left arc: from (0, r) to (r, 0)
        let p_bl_top = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, r) });
        let p_bl_right = sketch.add_entity(SketchEntity::Point { position: Pt2::new(r, 0.0) });
        // Bottom-right arc: from (w-r, 0) to (w, r)
        let p_br_left = sketch.add_entity(SketchEntity::Point { position: Pt2::new(w - r, 0.0) });
        let p_br_top = sketch.add_entity(SketchEntity::Point { position: Pt2::new(w, r) });
        // Top-right arc: from (w, h-r) to (w-r, h)
        let p_tr_bottom = sketch.add_entity(SketchEntity::Point { position: Pt2::new(w, h - r) });
        let p_tr_left = sketch.add_entity(SketchEntity::Point { position: Pt2::new(w - r, h) });
        // Top-left arc: from (r, h) to (0, h-r)
        let p_tl_right = sketch.add_entity(SketchEntity::Point { position: Pt2::new(r, h) });
        let p_tl_bottom = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, h - r) });

        // 4 lines connecting arc endpoints
        sketch.add_entity(SketchEntity::Line { start: p_bl_right, end: p_br_left }); // bottom
        sketch.add_entity(SketchEntity::Line { start: p_br_top, end: p_tr_bottom }); // right
        sketch.add_entity(SketchEntity::Line { start: p_tr_left, end: p_tl_right }); // top
        sketch.add_entity(SketchEntity::Line { start: p_tl_bottom, end: p_bl_top }); // left

        // 4 corner arcs
        sketch.add_entity(SketchEntity::Arc { center: c_bl, start: p_bl_top, end: p_bl_right });
        sketch.add_entity(SketchEntity::Arc { center: c_br, start: p_br_left, end: p_br_top });
        sketch.add_entity(SketchEntity::Arc { center: c_tr, start: p_tr_bottom, end: p_tr_left });
        sketch.add_entity(SketchEntity::Arc { center: c_tl, start: p_tl_right, end: p_tl_bottom });

        // Fix all points so the sketch is fully constrained
        for pt in [c_bl, c_br, c_tr, c_tl, p_bl_top, p_bl_right, p_br_left, p_br_top,
                    p_tr_bottom, p_tr_left, p_tl_right, p_tl_bottom] {
            sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![pt]));
        }

        let (graph, var_map) = solve_sketch(&sketch);
        let profile = extract_profile(&sketch, &var_map, &graph);
        assert!(profile.is_ok(), "Mixed lines+arcs profile should succeed");
        let profile = profile.unwrap();
        // 4 lines contribute 4 start points, 4 arcs contribute 4 starts + 4*(ARC_SEGMENTS-1) intermediates
        // Total = 4 + 4 + 4*(8-1) = 36
        assert!(
            profile.points.len() > 4,
            "Rounded rectangle should have more than 4 points (has arcs), got {}",
            profile.points.len()
        );
    }

    #[test]
    fn test_profile_empty_sketch_errors() {
        // Sketch with only points, no lines/arcs/circles -> should return error
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p0 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 1.0),
        });

        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p1]));

        let (graph, var_map) = solve_sketch(&sketch);
        let result = extract_profile(&sketch, &var_map, &graph);
        assert!(result.is_err(), "Profile from points-only sketch should error");
    }

    #[test]
    fn test_profile_open_loop_errors() {
        // Two lines that don't form a closed loop -> should return error
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p0 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 5.0),
        });

        // Two lines: p0->p1 and p1->p2, but no closing edge back to p0
        sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
        sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });

        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p1]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p2]));

        let (graph, var_map) = solve_sketch(&sketch);
        let result = extract_profile(&sketch, &var_map, &graph);
        assert!(result.is_err(), "Open loop should return error");
    }
}
