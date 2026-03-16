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

#[test]
fn sketch_serialization_roundtrip_all_constraints() {
    // Create a sketch with every constraint type, serialize to JSON, deserialize, verify
    let mut sketch = Sketch::new(Plane::xy(0.0));

    let p0 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
    let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(10.0, 0.0) });
    let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(10.0, 5.0) });
    let p3 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 5.0) });
    let p4 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(5.0, 2.5) }); // midpoint

    let line1 = sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
    let line2 = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
    let line3 = sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });

    let center = sketch.add_entity(SketchEntity::Point { position: Pt2::new(5.0, 5.0) });
    let circle1 = sketch.add_entity(SketchEntity::Circle { center, radius: 3.0 });
    let circle2 = sketch.add_entity(SketchEntity::Circle { center, radius: 3.0 });

    // Add one of each constraint type
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![line1]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![line2]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 10.0 }, vec![p0, p1]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Parallel, vec![line1, line3]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Perpendicular, vec![line1, line2]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Equal, vec![line1, line3]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Coincident, vec![p0, p3]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Midpoint, vec![p0, p1, p4]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Radius { value: 3.0 }, vec![circle1]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Coradial, vec![circle1, circle2]));
    sketch.add_constraint(Constraint::new(ConstraintKind::PointOnCurve, vec![p4, line1]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Tangent, vec![line1, circle1]));
    sketch.add_constraint(Constraint::new(
        ConstraintKind::Angle { value: std::f64::consts::FRAC_PI_2, supplementary: false },
        vec![line1, line2],
    ));

    // Serialize
    let json = serde_json::to_string(&sketch).expect("Sketch should serialize");

    // Deserialize
    let restored: Sketch = serde_json::from_str(&json).expect("Sketch should deserialize");

    assert_eq!(restored.entity_count(), sketch.entity_count());
    assert_eq!(restored.constraint_count(), sketch.constraint_count());
    assert_eq!(restored.constraint_count(), 14); // all 14 constraints
}

#[test]
fn sketch_with_blocks_serializes() {
    use blockcad_kernel::sketch::block::{SketchBlock, SketchBlockInstance};

    let mut sketch = Sketch::new(Plane::xy(0.0));
    let p0 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
    let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(5.0, 0.0) });

    sketch.add_block(SketchBlock {
        id: "b-1".into(),
        name: "TestBlock".into(),
        insertion_point: Pt2::new(0.0, 0.0),
        entity_indices: vec![p0, p1],
    });
    sketch.add_block_instance(SketchBlockInstance {
        id: "bi-1".into(),
        block_id: "b-1".into(),
        position: Pt2::new(10.0, 0.0),
        scale: 2.0,
        rotation: 0.5,
    });

    let json = serde_json::to_string(&sketch).expect("Sketch with blocks should serialize");
    let restored: Sketch = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(restored.block_definitions.len(), 1);
    assert_eq!(restored.block_instances.len(), 1);
    assert_eq!(restored.block_definitions[0].name, "TestBlock");
    assert!((restored.block_instances[0].scale - 2.0).abs() < 1e-9);
}
