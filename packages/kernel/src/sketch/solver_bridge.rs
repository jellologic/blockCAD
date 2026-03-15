use crate::error::{KernelError, KernelResult};
use crate::solver::equations::{
    CoincidentEquation, DistanceEquation, FixedEquation, PerpendicularEquation,
};
use crate::solver::graph::ConstraintGraph;
use crate::solver::variable::Variable;

use super::constraint::ConstraintKind;
use super::entity::SketchEntity;
use super::sketch::Sketch;
use super::variable_map::VariableMap;

/// Bridge between the Sketch data model and the constraint solver.
/// Converts sketch entities and constraints into solver variables and equations.
///
/// Returns the constraint graph (ready for `solve()`) and a variable map
/// for reading solved values back into entity positions.
pub fn build_constraint_graph(sketch: &Sketch) -> KernelResult<(ConstraintGraph, VariableMap)> {
    let mut graph = ConstraintGraph::new();
    let mut var_map = VariableMap::new();

    // Phase 1: Allocate solver variables for each entity
    for (entity_id, entity) in sketch.entities.iter() {
        match entity {
            SketchEntity::Point { position } => {
                let x = graph.variables.add(Variable::new(position.x));
                let y = graph.variables.add(Variable::new(position.y));
                var_map.insert(entity_id, vec![x, y]);
            }
            SketchEntity::Line { .. } => {
                // Lines reference existing point entities — no new variables
                var_map.insert(entity_id, vec![]);
            }
            SketchEntity::Arc { .. } => {
                // Arc references center/start/end points — no new variables
                // (radius could be derived, or made a variable later)
                var_map.insert(entity_id, vec![]);
            }
            SketchEntity::Circle { center: _, radius } => {
                // Center point already has its own variables.
                // Add a variable for the radius.
                let r = graph.variables.add(Variable::new(*radius));
                var_map.insert(entity_id, vec![r]);
            }
            SketchEntity::Spline { .. } => {
                // Spline control points are separate Point entities
                var_map.insert(entity_id, vec![]);
            }
        }
    }

    // Phase 2: Map constraints to equations
    for (_constraint_id, constraint) in sketch.constraints.iter() {
        if constraint.driven {
            // Driven (reference) dimensions don't add equations
            continue;
        }

        match &constraint.kind {
            ConstraintKind::Fixed => {
                // Fix a point at its current position
                let entity_id = constraint.entities[0];
                let (x, y) = var_map.point_vars(entity_id).ok_or_else(|| {
                    KernelError::Internal("Fixed constraint on non-point entity".into())
                })?;
                let entity = sketch.entities.get(entity_id)?;
                if let SketchEntity::Point { position } = entity {
                    graph.add_equation(Box::new(FixedEquation::new(x, position.x)));
                    graph.add_equation(Box::new(FixedEquation::new(y, position.y)));
                    // Mark variables as fixed for the solver
                    graph.variables.get_mut(x).unwrap().fixed = true;
                    graph.variables.get_mut(y).unwrap().fixed = true;
                }
            }

            ConstraintKind::Coincident => {
                // Two points must be at the same location
                let (x1, y1) = var_map.point_vars(constraint.entities[0]).ok_or_else(|| {
                    KernelError::Internal("Coincident constraint on non-point entity".into())
                })?;
                let (x2, y2) = var_map.point_vars(constraint.entities[1]).ok_or_else(|| {
                    KernelError::Internal("Coincident constraint on non-point entity".into())
                })?;
                graph.add_equation(Box::new(CoincidentEquation::new(x1, x2)));
                graph.add_equation(Box::new(CoincidentEquation::new(y1, y2)));
            }

            ConstraintKind::Horizontal => {
                // Line's start and end points have the same y-coordinate
                let line_id = constraint.entities[0];
                let line = sketch.entities.get(line_id)?;
                if let SketchEntity::Line { start, end } = line {
                    let (_, y1) = var_map.point_vars(*start).ok_or_else(|| {
                        KernelError::Internal("Horizontal: start not a point".into())
                    })?;
                    let (_, y2) = var_map.point_vars(*end).ok_or_else(|| {
                        KernelError::Internal("Horizontal: end not a point".into())
                    })?;
                    graph.add_equation(Box::new(CoincidentEquation::new(y1, y2)));
                }
            }

            ConstraintKind::Vertical => {
                // Line's start and end points have the same x-coordinate
                let line_id = constraint.entities[0];
                let line = sketch.entities.get(line_id)?;
                if let SketchEntity::Line { start, end } = line {
                    let (x1, _) = var_map.point_vars(*start).ok_or_else(|| {
                        KernelError::Internal("Vertical: start not a point".into())
                    })?;
                    let (x2, _) = var_map.point_vars(*end).ok_or_else(|| {
                        KernelError::Internal("Vertical: end not a point".into())
                    })?;
                    graph.add_equation(Box::new(CoincidentEquation::new(x1, x2)));
                }
            }

            ConstraintKind::Distance { value } => {
                // Distance between two points
                let (x1, y1) = var_map.point_vars(constraint.entities[0]).ok_or_else(|| {
                    KernelError::Internal("Distance constraint on non-point entity".into())
                })?;
                let (x2, y2) = var_map.point_vars(constraint.entities[1]).ok_or_else(|| {
                    KernelError::Internal("Distance constraint on non-point entity".into())
                })?;
                graph.add_equation(Box::new(DistanceEquation::new(x1, y1, x2, y2, *value)));
            }

            ConstraintKind::Perpendicular => {
                // Two lines must be perpendicular (dot product of directions = 0)
                let line1 = sketch.entities.get(constraint.entities[0])?;
                let line2 = sketch.entities.get(constraint.entities[1])?;
                if let (
                    SketchEntity::Line {
                        start: s1,
                        end: e1,
                    },
                    SketchEntity::Line {
                        start: s2,
                        end: e2,
                    },
                ) = (line1, line2)
                {
                    let (x1, y1) = var_map.point_vars(*s1).ok_or_else(|| {
                        KernelError::Internal("Perpendicular: line1 start not a point".into())
                    })?;
                    let (x2, y2) = var_map.point_vars(*e1).ok_or_else(|| {
                        KernelError::Internal("Perpendicular: line1 end not a point".into())
                    })?;
                    let (x3, y3) = var_map.point_vars(*s2).ok_or_else(|| {
                        KernelError::Internal("Perpendicular: line2 start not a point".into())
                    })?;
                    let (x4, y4) = var_map.point_vars(*e2).ok_or_else(|| {
                        KernelError::Internal("Perpendicular: line2 end not a point".into())
                    })?;
                    graph.add_equation(Box::new(PerpendicularEquation::new(
                        x1, y1, x2, y2, x3, y3, x4, y4,
                    )));
                }
            }

            // Constraint types not yet mapped to equations — skip for now
            ConstraintKind::Collinear
            | ConstraintKind::Parallel
            | ConstraintKind::Tangent
            | ConstraintKind::Symmetric { .. }
            | ConstraintKind::Midpoint
            | ConstraintKind::Angle { .. }
            | ConstraintKind::Radius { .. }
            | ConstraintKind::Diameter { .. }
            | ConstraintKind::Equal => {
                // TODO: Implement these constraint equation mappings
            }
        }
    }

    Ok((graph, var_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;
    use crate::geometry::Pt2;
    use crate::sketch::constraint::Constraint;
    use crate::sketch::entity::SketchEntity;
    use crate::sketch::sketch::Sketch;
    use crate::solver::newton_raphson::{solve, SolverConfig};

    // --- Step 2 tests: Entity-to-variable mapping ---

    #[test]
    fn test_point_creates_two_variables() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(3.0, 4.0),
        });

        let (graph, _var_map) = build_constraint_graph(&sketch).unwrap();
        assert_eq!(graph.variables.len(), 2, "Point should create 2 variables (x, y)");
        assert_eq!(graph.free_variable_count(), 2, "Both should be free");
    }

    #[test]
    fn test_two_points_create_four_variables() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 5.0),
        });

        let (graph, _var_map) = build_constraint_graph(&sketch).unwrap();
        assert_eq!(graph.variables.len(), 4);
    }

    #[test]
    fn test_line_reuses_point_vars() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 0.0),
        });
        sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });

        let (graph, _var_map) = build_constraint_graph(&sketch).unwrap();
        // Only the 2 points create variables (4 total), line adds 0
        assert_eq!(graph.variables.len(), 4);
    }

    #[test]
    fn test_circle_creates_center_plus_radius() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let center = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 5.0),
        });
        sketch.add_entity(SketchEntity::Circle {
            center,
            radius: 3.0,
        });

        let (graph, var_map) = build_constraint_graph(&sketch).unwrap();
        // Point: 2 vars + Circle: 1 var (radius) = 3 total
        assert_eq!(graph.variables.len(), 3);
        // Center point should have x, y vars
        assert!(var_map.point_vars(center).is_some());
    }

    // --- Step 3 tests: Constraint-to-equation mapping ---

    #[test]
    fn test_fixed_point_produces_two_equations() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(3.0, 4.0),
        });
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p]));

        let (graph, _) = build_constraint_graph(&sketch).unwrap();
        assert_eq!(graph.equation_count(), 2, "Fixed point → 2 equations (x, y)");
        assert_eq!(graph.free_variable_count(), 0, "Both vars should be fixed");
    }

    #[test]
    fn test_coincident_produces_two_equations() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 1.0),
        });
        sketch.add_constraint(Constraint::new(ConstraintKind::Coincident, vec![p1, p2]));

        let (graph, _) = build_constraint_graph(&sketch).unwrap();
        assert_eq!(graph.equation_count(), 2, "Coincident → 2 equations (x, y)");
    }

    #[test]
    fn test_horizontal_constraint_on_line() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 0.5),
        });
        let line = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
        sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![line]));

        let (graph, _) = build_constraint_graph(&sketch).unwrap();
        assert_eq!(graph.equation_count(), 1, "Horizontal → 1 equation (y1 == y2)");
    }

    #[test]
    fn test_vertical_constraint_on_line() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.5, 5.0),
        });
        let line = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
        sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![line]));

        let (graph, _) = build_constraint_graph(&sketch).unwrap();
        assert_eq!(graph.equation_count(), 1, "Vertical → 1 equation (x1 == x2)");
    }

    #[test]
    fn test_distance_constraint() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(3.0, 4.0),
        });
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Distance { value: 5.0 },
            vec![p1, p2],
        ));

        let (graph, _) = build_constraint_graph(&sketch).unwrap();
        assert_eq!(graph.equation_count(), 1);
    }

    #[test]
    fn test_driven_constraint_skipped() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 0.0),
        });
        sketch.add_constraint(
            Constraint::new(ConstraintKind::Distance { value: 5.0 }, vec![p1, p2]).driven(),
        );

        let (graph, _) = build_constraint_graph(&sketch).unwrap();
        assert_eq!(graph.equation_count(), 0, "Driven constraints should be skipped");
    }

    #[test]
    fn test_rectangle_solves_through_bridge() {
        // Build a fully constrained rectangle sketch:
        // 4 points, 4 lines, p0 fixed at origin,
        // horizontal bottom/top, vertical left/right,
        // distance bottom=10, distance right=5
        let mut sketch = Sketch::new(Plane::xy(0.0));

        // Points with approximate initial positions
        let p0 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(8.0, 0.5),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(8.0, 4.0),
        });
        let p3 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.5, 4.0),
        });

        // Lines forming a loop
        let bottom = sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
        let right = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
        let top = sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
        let left = sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });

        // Fix origin
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
        // Horizontal bottom & top
        sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![bottom]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![top]));
        // Vertical left & right
        sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![right]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![left]));
        // Width = 10, Height = 5
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Distance { value: 10.0 },
            vec![p0, p1],
        ));
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Distance { value: 5.0 },
            vec![p1, p2],
        ));

        let (mut graph, var_map) = build_constraint_graph(&sketch).unwrap();

        // Solve
        let result = solve(&mut graph, &SolverConfig::default()).unwrap();
        assert!(result.converged, "Solver should converge for rectangle");

        // Read solved positions
        let (x0, y0) = var_map.point_vars(p0).unwrap();
        let (x1, y1) = var_map.point_vars(p1).unwrap();
        let (x2, y2) = var_map.point_vars(p2).unwrap();
        let (x3, y3) = var_map.point_vars(p3).unwrap();

        assert!((graph.variables.value(x0) - 0.0).abs() < 1e-6);
        assert!((graph.variables.value(y0) - 0.0).abs() < 1e-6);
        assert!((graph.variables.value(x1) - 10.0).abs() < 1e-6);
        assert!((graph.variables.value(y1) - 0.0).abs() < 1e-6);
        assert!((graph.variables.value(x2) - 10.0).abs() < 1e-6);
        assert!((graph.variables.value(y2) - 5.0).abs() < 1e-6);
        assert!((graph.variables.value(x3) - 0.0).abs() < 1e-6);
        assert!((graph.variables.value(y3) - 5.0).abs() < 1e-6);
    }
}
