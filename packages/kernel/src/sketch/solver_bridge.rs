use crate::error::{KernelError, KernelResult};
use crate::solver::equations::{
    AngleEquation, CoincidentEquation, CollinearEquation, DistanceEquation, EqualLengthEquation,
    FixedEquation, MidpointEquation, ParallelEquation, PerpendicularEquation,
    PointOnCircleEquation, PointOnLineEquation, RadiusEquation,
    SymmetricMidpointEquation, SymmetricPerpendicularEquation, TangentLineCircleEquation,
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
            SketchEntity::Ellipse { radius_x, radius_y, .. } => {
                // Center is a separate Point entity.
                // Add variables for the two radii.
                let rx = graph.variables.add(Variable::new(*radius_x));
                let ry = graph.variables.add(Variable::new(*radius_y));
                var_map.insert(entity_id, vec![rx, ry]);
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

            ConstraintKind::Parallel => {
                // Two lines must be parallel (cross product of directions = 0)
                let line1 = sketch.entities.get(constraint.entities[0])?;
                let line2 = sketch.entities.get(constraint.entities[1])?;
                if let (
                    SketchEntity::Line { start: s1, end: e1 },
                    SketchEntity::Line { start: s2, end: e2 },
                ) = (line1, line2)
                {
                    let (x1, y1) = var_map.point_vars(*s1).ok_or_else(|| {
                        KernelError::Internal("Parallel: line1 start not a point".into())
                    })?;
                    let (x2, y2) = var_map.point_vars(*e1).ok_or_else(|| {
                        KernelError::Internal("Parallel: line1 end not a point".into())
                    })?;
                    let (x3, y3) = var_map.point_vars(*s2).ok_or_else(|| {
                        KernelError::Internal("Parallel: line2 start not a point".into())
                    })?;
                    let (x4, y4) = var_map.point_vars(*e2).ok_or_else(|| {
                        KernelError::Internal("Parallel: line2 end not a point".into())
                    })?;
                    graph.add_equation(Box::new(ParallelEquation::new(
                        x1, y1, x2, y2, x3, y3, x4, y4,
                    )));
                }
            }

            ConstraintKind::Collinear => {
                // Two lines are collinear: all 4 points on same line.
                // We enforce: line2.start on line1, line2.end on line1
                let line1 = sketch.entities.get(constraint.entities[0])?;
                let line2 = sketch.entities.get(constraint.entities[1])?;
                if let (
                    SketchEntity::Line { start: s1, end: e1 },
                    SketchEntity::Line { start: s2, end: e2 },
                ) = (line1, line2)
                {
                    let (ax, ay) = var_map.point_vars(*s1).ok_or_else(|| {
                        KernelError::Internal("Collinear: line1 start not a point".into())
                    })?;
                    let (bx, by) = var_map.point_vars(*e1).ok_or_else(|| {
                        KernelError::Internal("Collinear: line1 end not a point".into())
                    })?;
                    let (cx, cy) = var_map.point_vars(*s2).ok_or_else(|| {
                        KernelError::Internal("Collinear: line2 start not a point".into())
                    })?;
                    let (dx, dy) = var_map.point_vars(*e2).ok_or_else(|| {
                        KernelError::Internal("Collinear: line2 end not a point".into())
                    })?;
                    graph.add_equation(Box::new(CollinearEquation::new(ax, ay, bx, by, cx, cy)));
                    graph.add_equation(Box::new(CollinearEquation::new(ax, ay, bx, by, dx, dy)));
                }
            }

            ConstraintKind::Angle { value, .. } => {
                // Angle between two lines
                let line1 = sketch.entities.get(constraint.entities[0])?;
                let line2 = sketch.entities.get(constraint.entities[1])?;
                if let (
                    SketchEntity::Line { start: s1, end: e1 },
                    SketchEntity::Line { start: s2, end: e2 },
                ) = (line1, line2)
                {
                    let (x1, y1) = var_map.point_vars(*s1).ok_or_else(|| {
                        KernelError::Internal("Angle: line1 start not a point".into())
                    })?;
                    let (x2, y2) = var_map.point_vars(*e1).ok_or_else(|| {
                        KernelError::Internal("Angle: line1 end not a point".into())
                    })?;
                    let (x3, y3) = var_map.point_vars(*s2).ok_or_else(|| {
                        KernelError::Internal("Angle: line2 start not a point".into())
                    })?;
                    let (x4, y4) = var_map.point_vars(*e2).ok_or_else(|| {
                        KernelError::Internal("Angle: line2 end not a point".into())
                    })?;
                    graph.add_equation(Box::new(AngleEquation::new(
                        x1, y1, x2, y2, x3, y3, x4, y4, *value,
                    )));
                }
            }

            ConstraintKind::Midpoint => {
                // Point C is midpoint of points A and B
                let (ax, ay) = var_map.point_vars(constraint.entities[0]).ok_or_else(|| {
                    KernelError::Internal("Midpoint: entity 0 not a point".into())
                })?;
                let (bx, by) = var_map.point_vars(constraint.entities[1]).ok_or_else(|| {
                    KernelError::Internal("Midpoint: entity 1 not a point".into())
                })?;
                let (cx, cy) = var_map.point_vars(constraint.entities[2]).ok_or_else(|| {
                    KernelError::Internal("Midpoint: entity 2 not a point".into())
                })?;
                graph.add_equation(Box::new(MidpointEquation::new(ax, bx, cx)));
                graph.add_equation(Box::new(MidpointEquation::new(ay, by, cy)));
            }

            ConstraintKind::Symmetric { axis } => {
                // Two points symmetric about an axis line
                let (p1x, p1y) = var_map.point_vars(constraint.entities[0]).ok_or_else(|| {
                    KernelError::Internal("Symmetric: entity 0 not a point".into())
                })?;
                let (p2x, p2y) = var_map.point_vars(constraint.entities[1]).ok_or_else(|| {
                    KernelError::Internal("Symmetric: entity 1 not a point".into())
                })?;
                let axis_line = sketch.entities.get(*axis)?;
                if let SketchEntity::Line { start, end } = axis_line {
                    let (ax, ay) = var_map.point_vars(*start).ok_or_else(|| {
                        KernelError::Internal("Symmetric: axis start not a point".into())
                    })?;
                    let (bx, by) = var_map.point_vars(*end).ok_or_else(|| {
                        KernelError::Internal("Symmetric: axis end not a point".into())
                    })?;
                    graph.add_equation(Box::new(SymmetricMidpointEquation::new(
                        p1x, p1y, p2x, p2y, ax, ay, bx, by,
                    )));
                    graph.add_equation(Box::new(SymmetricPerpendicularEquation::new(
                        p1x, p1y, p2x, p2y, ax, ay, bx, by,
                    )));
                }
            }

            ConstraintKind::Radius { value } => {
                // Circle radius equals value
                let circle_id = constraint.entities[0];
                if let Some(r_var) = var_map.circle_radius_var(circle_id) {
                    graph.add_equation(Box::new(RadiusEquation::new(r_var, *value)));
                }
            }

            ConstraintKind::Diameter { value } => {
                // Circle diameter equals value → radius = value/2
                let circle_id = constraint.entities[0];
                if let Some(r_var) = var_map.circle_radius_var(circle_id) {
                    graph.add_equation(Box::new(RadiusEquation::new(r_var, *value / 2.0)));
                }
            }

            ConstraintKind::Equal => {
                // Two lines have equal length (or two circles have equal radius)
                let e1 = sketch.entities.get(constraint.entities[0])?;
                let e2 = sketch.entities.get(constraint.entities[1])?;
                match (e1, e2) {
                    (
                        SketchEntity::Line { start: s1, end: e1 },
                        SketchEntity::Line { start: s2, end: e2 },
                    ) => {
                        let (x1, y1) = var_map.point_vars(*s1).ok_or_else(|| {
                            KernelError::Internal("Equal: line1 start not a point".into())
                        })?;
                        let (x2, y2) = var_map.point_vars(*e1).ok_or_else(|| {
                            KernelError::Internal("Equal: line1 end not a point".into())
                        })?;
                        let (x3, y3) = var_map.point_vars(*s2).ok_or_else(|| {
                            KernelError::Internal("Equal: line2 start not a point".into())
                        })?;
                        let (x4, y4) = var_map.point_vars(*e2).ok_or_else(|| {
                            KernelError::Internal("Equal: line2 end not a point".into())
                        })?;
                        graph.add_equation(Box::new(EqualLengthEquation::new(
                            x1, y1, x2, y2, x3, y3, x4, y4,
                        )));
                    }
                    (
                        SketchEntity::Circle { .. },
                        SketchEntity::Circle { .. },
                    ) => {
                        if let (Some(r1), Some(r2)) = (
                            var_map.circle_radius_var(constraint.entities[0]),
                            var_map.circle_radius_var(constraint.entities[1]),
                        ) {
                            graph.add_equation(Box::new(CoincidentEquation::new(r1, r2)));
                        }
                    }
                    _ => {} // Unsupported entity combination
                }
            }

            ConstraintKind::Coradial => {
                // Two circles/arcs share same center and same radius.
                // entities[0] and entities[1] are circles.
                let e1 = sketch.entities.get(constraint.entities[0])?;
                let e2 = sketch.entities.get(constraint.entities[1])?;
                if let (
                    SketchEntity::Circle { center: c1, .. },
                    SketchEntity::Circle { center: c2, .. },
                ) = (e1, e2)
                {
                    // Same center (coincident)
                    let (cx1, cy1) = var_map.point_vars(*c1).ok_or_else(|| {
                        KernelError::Internal("Coradial: circle1 center not a point".into())
                    })?;
                    let (cx2, cy2) = var_map.point_vars(*c2).ok_or_else(|| {
                        KernelError::Internal("Coradial: circle2 center not a point".into())
                    })?;
                    graph.add_equation(Box::new(CoincidentEquation::new(cx1, cx2)));
                    graph.add_equation(Box::new(CoincidentEquation::new(cy1, cy2)));

                    // Same radius
                    if let (Some(r1), Some(r2)) = (
                        var_map.circle_radius_var(constraint.entities[0]),
                        var_map.circle_radius_var(constraint.entities[1]),
                    ) {
                        graph.add_equation(Box::new(CoincidentEquation::new(r1, r2)));
                    }
                }
            }

            ConstraintKind::PointOnCurve => {
                // Point lies on a curve (line or circle).
                // entities[0] = point, entities[1] = curve (line or circle)
                let point_id = constraint.entities[0];
                let curve_id = constraint.entities[1];
                let (px, py) = var_map.point_vars(point_id).ok_or_else(|| {
                    KernelError::Internal("PointOnCurve: entity 0 not a point".into())
                })?;
                let curve = sketch.entities.get(curve_id)?;
                match curve {
                    SketchEntity::Line { start, end } => {
                        let (ax, ay) = var_map.point_vars(*start).ok_or_else(|| {
                            KernelError::Internal("PointOnCurve: line start not a point".into())
                        })?;
                        let (bx, by) = var_map.point_vars(*end).ok_or_else(|| {
                            KernelError::Internal("PointOnCurve: line end not a point".into())
                        })?;
                        graph.add_equation(Box::new(PointOnLineEquation::new(
                            ax, ay, bx, by, px, py,
                        )));
                    }
                    SketchEntity::Circle { center, .. } => {
                        let (cx, cy) = var_map.point_vars(*center).ok_or_else(|| {
                            KernelError::Internal("PointOnCurve: circle center not a point".into())
                        })?;
                        let r = var_map.circle_radius_var(curve_id).ok_or_else(|| {
                            KernelError::Internal("PointOnCurve: circle has no radius var".into())
                        })?;
                        graph.add_equation(Box::new(PointOnCircleEquation::new(
                            px, py, cx, cy, r,
                        )));
                    }
                    _ => {} // Unsupported curve type
                }
            }

            ConstraintKind::Tangent => {
                // Tangent between a line and a circle, or two circles
                let e1 = sketch.entities.get(constraint.entities[0])?;
                let e2 = sketch.entities.get(constraint.entities[1])?;
                match (e1, e2) {
                    // Line-Circle tangent
                    (
                        SketchEntity::Line { start, end },
                        SketchEntity::Circle { center, .. },
                    ) => {
                        let (ax, ay) = var_map.point_vars(*start).ok_or_else(|| {
                            KernelError::Internal("Tangent: line start not a point".into())
                        })?;
                        let (bx, by) = var_map.point_vars(*end).ok_or_else(|| {
                            KernelError::Internal("Tangent: line end not a point".into())
                        })?;
                        let (cx, cy) = var_map.point_vars(*center).ok_or_else(|| {
                            KernelError::Internal("Tangent: circle center not a point".into())
                        })?;
                        let r = var_map.circle_radius_var(constraint.entities[1]).ok_or_else(|| {
                            KernelError::Internal("Tangent: circle has no radius var".into())
                        })?;
                        graph.add_equation(Box::new(TangentLineCircleEquation::new(
                            ax, ay, bx, by, cx, cy, r,
                        )));
                    }
                    // Circle-Line tangent (reversed order)
                    (
                        SketchEntity::Circle { center, .. },
                        SketchEntity::Line { start, end },
                    ) => {
                        let (ax, ay) = var_map.point_vars(*start).ok_or_else(|| {
                            KernelError::Internal("Tangent: line start not a point".into())
                        })?;
                        let (bx, by) = var_map.point_vars(*end).ok_or_else(|| {
                            KernelError::Internal("Tangent: line end not a point".into())
                        })?;
                        let (cx, cy) = var_map.point_vars(*center).ok_or_else(|| {
                            KernelError::Internal("Tangent: circle center not a point".into())
                        })?;
                        let r = var_map.circle_radius_var(constraint.entities[0]).ok_or_else(|| {
                            KernelError::Internal("Tangent: circle has no radius var".into())
                        })?;
                        graph.add_equation(Box::new(TangentLineCircleEquation::new(
                            ax, ay, bx, by, cx, cy, r,
                        )));
                    }
                    // Circle-Circle tangent (external): dist(centers) = r1 + r2
                    (
                        SketchEntity::Circle { center: c1, .. },
                        SketchEntity::Circle { center: c2, .. },
                    ) => {
                        let (cx1, cy1) = var_map.point_vars(*c1).ok_or_else(|| {
                            KernelError::Internal("Tangent: circle1 center not a point".into())
                        })?;
                        let (cx2, cy2) = var_map.point_vars(*c2).ok_or_else(|| {
                            KernelError::Internal("Tangent: circle2 center not a point".into())
                        })?;
                        // For external tangent, distance = r1 + r2
                        // We need the current radii to compute the target distance
                        if let (
                            SketchEntity::Circle { radius: r1, .. },
                            SketchEntity::Circle { radius: r2, .. },
                        ) = (
                            sketch.entities.get(constraint.entities[0])?,
                            sketch.entities.get(constraint.entities[1])?,
                        ) {
                            graph.add_equation(Box::new(DistanceEquation::new(
                                cx1, cy1, cx2, cy2, r1 + r2,
                            )));
                        }
                    }
                    _ => {} // Unsupported tangent combination
                }
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

    #[test]
    fn test_coradial_constraint() {
        // Two circles with different centers and radii.
        // Coradial should force same center and same radius.
        let mut sketch = Sketch::new(Plane::xy(0.0));

        let c1_center = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 2.0),
        });
        let circle1 = sketch.add_entity(SketchEntity::Circle {
            center: c1_center,
            radius: 3.0,
        });

        let c2_center = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.5, 2.5),
        });
        let circle2 = sketch.add_entity(SketchEntity::Circle {
            center: c2_center,
            radius: 4.0,
        });

        // Fix circle1's center so the solver has something to anchor to
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![c1_center]));
        // Fix circle1's radius
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Radius { value: 3.0 },
            vec![circle1],
        ));
        // Coradial: circles must share center and radius
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Coradial,
            vec![circle1, circle2],
        ));

        let (graph, _) = build_constraint_graph(&sketch).unwrap();
        // Coradial produces 3 equations: 2 coincident (cx, cy) + 1 equal radius
        // Plus Fixed produces 2, Radius produces 1 = total 6
        assert_eq!(graph.equation_count(), 6);

        // Now solve and verify
        let (mut graph, var_map) = build_constraint_graph(&sketch).unwrap();
        let result = solve(&mut graph, &SolverConfig::default()).unwrap();
        assert!(result.converged, "Solver should converge for coradial");

        let (cx1, cy1) = var_map.point_vars(c1_center).unwrap();
        let (cx2, cy2) = var_map.point_vars(c2_center).unwrap();
        let r1 = var_map.circle_radius_var(circle1).unwrap();
        let r2 = var_map.circle_radius_var(circle2).unwrap();

        // Centers should match
        assert!((graph.variables.value(cx1) - graph.variables.value(cx2)).abs() < 1e-6);
        assert!((graph.variables.value(cy1) - graph.variables.value(cy2)).abs() < 1e-6);
        // Radii should match
        assert!((graph.variables.value(r1) - graph.variables.value(r2)).abs() < 1e-6);
        // And both radii should be 3.0
        assert!((graph.variables.value(r1) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_duplicate_constraint_builds_graph() {
        // Adding the same constraint twice should not panic
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
        let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(10.0, 0.5) });
        let line = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });

        // Add horizontal constraint TWICE (over-constrained but shouldn't panic)
        sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![line]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![line]));

        let result = build_constraint_graph(&sketch);
        assert!(result.is_ok(), "Duplicate constraints should not cause build_constraint_graph to fail");
        let (graph, _) = result.unwrap();
        // Should have 2 equations (one per horizontal constraint)
        assert_eq!(graph.equation_count(), 2);
    }

    #[test]
    fn test_point_on_line_constraint() {
        // A line and a separate point. PointOnCurve should place the point on the line.
        let mut sketch = Sketch::new(Plane::xy(0.0));

        let p_start = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p_end = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(10.0, 0.0),
        });
        let line = sketch.add_entity(SketchEntity::Line {
            start: p_start,
            end: p_end,
        });

        // A point off the line (x=5, y=3)
        let p_free = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 3.0),
        });

        // Fix line endpoints
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p_start]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p_end]));
        // Fix the free point's x so the system is fully determined (1 DOF for y, 1 equation)
        // We use a distance constraint from p_start to p_free of 5.0 along x, combined with
        // the PointOnCurve. Instead, just also fix its x by adding a distance-from-start constraint
        // Actually, easiest: constrain x by fixing it at x=5 via a second point + coincident on x.
        // Simpler: add a distance constraint between p_start and p_free = 5.0 (after solving on-line, dist will be 5).
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Distance { value: 5.0 },
            vec![p_start, p_free],
        ));
        // Point on line constraint
        sketch.add_constraint(Constraint::new(
            ConstraintKind::PointOnCurve,
            vec![p_free, line],
        ));

        let (mut graph, var_map) = build_constraint_graph(&sketch).unwrap();
        let result = solve(&mut graph, &SolverConfig::default()).unwrap();
        assert!(result.converged, "Solver should converge for point-on-line");

        let (_px, py) = var_map.point_vars(p_free).unwrap();
        let solved_y = graph.variables.value(py);
        // Line is y=0 from (0,0) to (10,0), so point should be at y=0
        assert!(
            solved_y.abs() < 1e-6,
            "Point should lie on the horizontal line, y={}", solved_y
        );
    }

    #[test]
    fn test_point_on_circle_constraint() {
        // A circle and a separate point. PointOnCurve should place the point on the circle.
        // We fix center, radius, and the point's x-coordinate so the system is fully determined.
        let mut sketch = Sketch::new(Plane::xy(0.0));

        let center = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let circle = sketch.add_entity(SketchEntity::Circle {
            center,
            radius: 5.0,
        });

        // A point near the circle (x=3, y=4 is exactly on a radius-5 circle; start slightly off)
        let p_free = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(3.0, 4.5),
        });

        // Use a helper fixed point at x=3 and constrain p_free's x via Vertical alignment
        let p_anchor = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(3.0, 0.0),
        });
        let vline = sketch.add_entity(SketchEntity::Line {
            start: p_anchor,
            end: p_free,
        });

        // Fix center, radius, and anchor
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![center]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p_anchor]));
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Radius { value: 5.0 },
            vec![circle],
        ));
        // Vertical constraint on the helper line fixes p_free's x to 3.0
        sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![vline]));
        // Point on circle
        sketch.add_constraint(Constraint::new(
            ConstraintKind::PointOnCurve,
            vec![p_free, circle],
        ));

        let (mut graph, var_map) = build_constraint_graph(&sketch).unwrap();
        let result = solve(&mut graph, &SolverConfig::default()).unwrap();
        assert!(result.converged, "Solver should converge for point-on-circle");

        let (px, py) = var_map.point_vars(p_free).unwrap();
        let (cx, cy) = var_map.point_vars(center).unwrap();
        let sx = graph.variables.value(px) - graph.variables.value(cx);
        let sy = graph.variables.value(py) - graph.variables.value(cy);
        let dist = (sx * sx + sy * sy).sqrt();
        assert!(
            (dist - 5.0).abs() < 1e-6,
            "Point should be at distance 5 from center, got {}", dist
        );
    }

    #[test]
    fn test_tangent_line_circle_constraint() {
        // A horizontal line and a circle above it.
        // Tangent should make the perpendicular distance from center to line equal the radius.
        // Center x is fixed so the system is fully determined (1 DOF for center y, 1 equation).
        let mut sketch = Sketch::new(Plane::xy(0.0));

        // Horizontal line at y=0
        let l_start = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(-10.0, 0.0),
        });
        let l_end = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(10.0, 0.0),
        });
        let line = sketch.add_entity(SketchEntity::Line {
            start: l_start,
            end: l_end,
        });

        // Circle above the line — center starts at (0, 4), should move to (0, 3)
        let center = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 4.0),
        });
        let circle = sketch.add_entity(SketchEntity::Circle {
            center,
            radius: 3.0,
        });

        // Helper: fix center x by using a vertical line from a fixed anchor
        let anchor = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let vline = sketch.add_entity(SketchEntity::Line {
            start: anchor,
            end: center,
        });

        // Fix the line endpoints and anchor
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![l_start]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![l_end]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![anchor]));
        // Fix center's x via vertical constraint
        sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![vline]));
        // Fix the radius
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Radius { value: 3.0 },
            vec![circle],
        ));
        // Tangent constraint
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Tangent,
            vec![line, circle],
        ));

        let (mut graph, var_map) = build_constraint_graph(&sketch).unwrap();
        let result = solve(&mut graph, &SolverConfig::default()).unwrap();
        assert!(result.converged, "Solver should converge for tangent line-circle");

        // Read solved center position
        let (_cx_var, cy_var) = var_map.point_vars(center).unwrap();
        let cy = graph.variables.value(cy_var);
        let r_var = var_map.circle_radius_var(circle).unwrap();
        let r = graph.variables.value(r_var);

        // The line is at y=0 (horizontal). Perpendicular distance from center to line = |cy|.
        // Tangent means |cy| == r.
        assert!(
            (cy.abs() - r).abs() < 1e-6,
            "Perpendicular distance from center to line should equal radius: |cy|={}, r={}", cy.abs(), r
        );
    }
}
