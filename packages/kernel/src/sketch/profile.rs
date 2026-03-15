use crate::error::{KernelError, KernelResult};
use crate::geometry::surface::Surface;
use crate::operations::extrude::ExtrudeProfile;
use crate::solver::graph::ConstraintGraph;

use super::entity::SketchEntity;
use super::sketch::Sketch;
use super::variable_map::VariableMap;

/// Extract a closed profile from a solved sketch, suitable for extrusion.
///
/// Walks the sketch's line entities to find a closed loop, reads solved
/// positions from the constraint graph, and projects them onto the sketch's
/// 3D plane.
pub fn extract_profile(
    sketch: &Sketch,
    var_map: &VariableMap,
    graph: &ConstraintGraph,
) -> KernelResult<ExtrudeProfile> {
    // Collect all line entities with their start/end point IDs
    let mut lines = Vec::new();
    for (_id, entity) in sketch.entities.iter() {
        if let SketchEntity::Line { start, end } = entity {
            lines.push((*start, *end));
        }
    }

    if lines.is_empty() {
        return Err(KernelError::Operation {
            op: "extract_profile".into(),
            detail: "No line entities in sketch".into(),
        });
    }

    // Find a closed loop by chaining lines: start from first line, follow end→start connections
    let mut loop_points = vec![lines[0].0]; // start with first line's start point
    let mut current_end = lines[0].1;
    let mut used = vec![false; lines.len()];
    used[0] = true;

    let first_start = lines[0].0;

    for _ in 0..lines.len() {
        if current_end == first_start {
            break; // Loop closed
        }

        loop_points.push(current_end);

        // Find an unused line whose start matches current_end
        let mut found = false;
        for (i, &(s, e)) in lines.iter().enumerate() {
            if used[i] {
                continue;
            }
            if s == current_end {
                used[i] = true;
                current_end = e;
                found = true;
                break;
            }
            // Also try reversed direction
            if e == current_end {
                used[i] = true;
                current_end = s;
                found = true;
                break;
            }
        }

        if !found {
            return Err(KernelError::Operation {
                op: "extract_profile".into(),
                detail: "Could not find a closed loop of line entities".into(),
            });
        }
    }

    if current_end != first_start {
        return Err(KernelError::Operation {
            op: "extract_profile".into(),
            detail: "Line entities do not form a closed loop".into(),
        });
    }

    // Read solved 2D positions and project to 3D via the sketch plane
    let mut points_3d = Vec::with_capacity(loop_points.len());
    for &pt_id in &loop_points {
        let (x_var, y_var) = var_map.point_vars(pt_id).ok_or_else(|| {
            KernelError::Internal(format!("No variable mapping for point {:?}", pt_id))
        })?;
        let x = graph.variables.value(x_var);
        let y = graph.variables.value(y_var);
        let pt_3d = sketch.plane.point_at(x, y)?;
        points_3d.push(pt_3d);
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
}
