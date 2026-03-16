use blockcad_kernel::geometry::Pt2;
use blockcad_kernel::geometry::surface::plane::Plane;
use blockcad_kernel::sketch::constraint::{Constraint, ConstraintKind};
use blockcad_kernel::sketch::entity::SketchEntity;
use blockcad_kernel::sketch::Sketch;

#[test]
fn create_sketch_with_line() {
    let mut sketch = Sketch::new(Plane::xy(0.0));
    let p1 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(0.0, 0.0),
    });
    let p2 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(10.0, 0.0),
    });
    let _line = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });

    assert_eq!(sketch.entity_count(), 3);
    assert_eq!(sketch.constraint_count(), 0);
}

#[test]
fn add_constraints_to_sketch() {
    let mut sketch = Sketch::new(Plane::xy(0.0));
    let p1 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(0.0, 0.0),
    });
    let p2 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(5.0, 5.0),
    });
    let p3 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(10.0, 0.0),
    });

    // Fix first point
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p1]));
    // Distance between p1 and p2
    sketch.add_constraint(Constraint::new(
        ConstraintKind::Distance { value: 7.07 },
        vec![p1, p2],
    ));
    // Horizontal constraint on p2-p3
    let _l = sketch.add_entity(SketchEntity::Line {
        start: p2,
        end: p3,
    });

    assert_eq!(sketch.constraint_count(), 2);
}

#[test]
fn sketch_circle_entity() {
    let mut sketch = Sketch::new(Plane::xy(0.0));
    let center = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(5.0, 5.0),
    });
    let _circle = sketch.add_entity(SketchEntity::Circle {
        center,
        radius: 3.0,
    });
    assert_eq!(sketch.entity_count(), 2);
}

#[test]
fn sketch_with_circle_profile_extraction() {
    use blockcad_kernel::sketch::profile::extract_profile;
    use blockcad_kernel::sketch::solver_bridge::build_constraint_graph;
    use blockcad_kernel::solver::newton_raphson::{solve, SolverConfig};

    let mut sketch = Sketch::new(Plane::xy(0.0));
    let center = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(0.0, 0.0),
    });
    sketch.add_entity(SketchEntity::Circle { center, radius: 5.0 });
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![center]));

    let (mut graph, var_map) = build_constraint_graph(&sketch).unwrap();
    let result = solve(&mut graph, &SolverConfig::default()).unwrap();
    assert!(result.converged);

    let profile = extract_profile(&sketch, &var_map, &graph).unwrap();
    assert_eq!(profile.points.len(), 32); // CIRCLE_SEGMENTS

    // All points should be at radius 5 from origin on XY plane
    for pt in &profile.points {
        let dist = (pt.x * pt.x + pt.y * pt.y).sqrt();
        assert!((dist - 5.0).abs() < 1e-6, "Point should be on circle, dist={}", dist);
        assert!(pt.z.abs() < 1e-6, "Should be on z=0 plane");
    }
}

#[test]
fn sketch_with_arc_and_line_profile_extraction() {
    use blockcad_kernel::sketch::profile::extract_profile;
    use blockcad_kernel::sketch::solver_bridge::build_constraint_graph;
    use blockcad_kernel::solver::newton_raphson::{solve, SolverConfig};

    // D-shape: vertical line on left + semicircular arc on right
    let mut sketch = Sketch::new(Plane::xy(0.0));

    let center = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(0.0, 0.0),
    });
    let p_top = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(0.0, 5.0),
    });
    let p_bottom = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(0.0, -5.0),
    });

    // Line from top to bottom (left side)
    sketch.add_entity(SketchEntity::Line { start: p_top, end: p_bottom });
    // Arc from bottom to top (right side, semicircle)
    sketch.add_entity(SketchEntity::Arc { center, start: p_bottom, end: p_top });

    // Fix all points
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![center]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p_top]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p_bottom]));

    let (mut graph, var_map) = build_constraint_graph(&sketch).unwrap();
    let result = solve(&mut graph, &SolverConfig::default()).unwrap();
    assert!(result.converged);

    let profile = extract_profile(&sketch, &var_map, &graph).unwrap();
    // Line contributes 1 start point, arc contributes 1 start + several intermediate
    assert!(profile.points.len() > 2, "D-shape should have arc samples, got {}", profile.points.len());
}
