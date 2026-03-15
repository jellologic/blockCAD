/// End-to-end integration tests for the parametric modeling pipeline.
///
/// Tests the full flow: Sketch → Solve → Extract Profile → Extrude → Tessellate
/// through the feature tree evaluator.
use blockcad_kernel::feature_tree::evaluator::evaluate;
use blockcad_kernel::feature_tree::tree::FeatureTree;
use blockcad_kernel::feature_tree::{Feature, FeatureKind, FeatureParams};
use blockcad_kernel::geometry::surface::plane::Plane;
use blockcad_kernel::geometry::{Pt2, Pt3, Vec3};
use blockcad_kernel::operations::extrude::ExtrudeParams;
use blockcad_kernel::operations::revolve::RevolveParams;
use std::f64::consts::PI;
use blockcad_kernel::sketch::constraint::{Constraint, ConstraintKind};
use blockcad_kernel::sketch::entity::SketchEntity;
use blockcad_kernel::sketch::Sketch;
use blockcad_kernel::tessellation::{tessellate_brep, TessellationParams};
use blockcad_kernel::topology::body::Body;

fn make_constrained_rectangle(w: f64, h: f64) -> Sketch {
    let mut sketch = Sketch::new(Plane::xy(0.0));

    let p0 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(0.0, 0.0),
    });
    let p1 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(w * 0.9, 0.2),
    });
    let p2 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(w * 0.9, h * 0.9),
    });
    let p3 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(0.2, h * 0.9),
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

fn build_sketch_extrude_tree(w: f64, h: f64, depth: f64) -> FeatureTree {
    let mut tree = FeatureTree::new();

    tree.push(Feature::new(
        "sketch-1".into(),
        "Base Sketch".into(),
        FeatureKind::Sketch,
        FeatureParams::Placeholder,
    ));
    tree.sketches.insert(0, make_constrained_rectangle(w, h));

    tree.push(Feature::new(
        "extrude-1".into(),
        "Extrude Base".into(),
        FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams {
            direction: Vec3::new(0.0, 0.0, 1.0),
            depth,
            symmetric: false,
            draft_angle: 0.0,
        }),
    ));

    tree
}

#[test]
fn test_sketch_extrude_full_pipeline() {
    let mut tree = build_sketch_extrude_tree(10.0, 5.0, 7.0);
    let brep = evaluate(&mut tree).unwrap();

    // BRep validation
    assert_eq!(brep.faces.len(), 6, "Extruded rectangle = 6 faces");
    assert!(matches!(brep.body, Body::Solid(_)));

    // Tessellation
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    assert_eq!(mesh.vertex_count(), 24, "6 faces × 4 vertices");
    assert_eq!(mesh.triangle_count(), 12, "6 faces × 2 triangles");
    mesh.validate().unwrap();
}

#[test]
fn test_different_dimensions() {
    // 20×10 rectangle extruded 15 deep
    let mut tree = build_sketch_extrude_tree(20.0, 10.0, 15.0);
    let brep = evaluate(&mut tree).unwrap();
    assert_eq!(brep.faces.len(), 6);

    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    mesh.validate().unwrap();
}

#[test]
fn test_rollback_and_forward() {
    let mut tree = build_sketch_extrude_tree(10.0, 5.0, 7.0);

    // Full evaluation
    let brep = evaluate(&mut tree).unwrap();
    assert_eq!(brep.faces.len(), 6);

    // Roll back to just sketch
    tree.rollback_to(1).unwrap();
    let brep = evaluate(&mut tree).unwrap();
    assert!(matches!(brep.body, Body::Empty), "Sketch only = no solid");

    // Roll back to nothing
    tree.rollback_to(0).unwrap();
    let brep = evaluate(&mut tree).unwrap();
    assert!(matches!(brep.body, Body::Empty));

    // Roll forward to full model
    tree.roll_forward();
    let brep = evaluate(&mut tree).unwrap();
    assert_eq!(brep.faces.len(), 6, "Model restored after roll forward");

    // Tessellate the restored model
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    mesh.validate().unwrap();
}

#[test]
fn test_suppress_and_unsuppress() {
    let mut tree = build_sketch_extrude_tree(10.0, 5.0, 7.0);

    // Suppress extrude
    tree.suppress(1).unwrap();
    let brep = evaluate(&mut tree).unwrap();
    assert!(matches!(brep.body, Body::Empty));

    // Unsuppress
    tree.unsuppress(1).unwrap();
    let brep = evaluate(&mut tree).unwrap();
    assert_eq!(brep.faces.len(), 6);
    assert!(matches!(brep.body, Body::Solid(_)));
}

#[test]
fn test_suppress_sketch_breaks_extrude() {
    let mut tree = build_sketch_extrude_tree(10.0, 5.0, 7.0);

    // Suppress the sketch — extrude should fail (no profile available)
    tree.suppress(0).unwrap();
    let result = evaluate(&mut tree);
    assert!(result.is_err(), "Extrude without sketch profile should fail");
}

#[test]
fn test_triangle_extrusion() {
    let mut tree = FeatureTree::new();

    // Triangle sketch
    let mut sketch = Sketch::new(Plane::xy(0.0));
    let p0 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(0.0, 0.0),
    });
    let p1 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(6.0, 0.0),
    });
    let p2 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(3.0, 4.0),
    });
    sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
    sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
    sketch.add_entity(SketchEntity::Line { start: p2, end: p0 });
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p1]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p2]));

    tree.push(Feature::new(
        "sketch-1".into(),
        "Triangle Sketch".into(),
        FeatureKind::Sketch,
        FeatureParams::Placeholder,
    ));
    tree.sketches.insert(0, sketch);

    tree.push(Feature::new(
        "extrude-1".into(),
        "Extrude Triangle".into(),
        FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams {
            direction: Vec3::new(0.0, 0.0, 1.0),
            depth: 3.0,
            symmetric: false,
            draft_angle: 0.0,
        }),
    ));

    let brep = evaluate(&mut tree).unwrap();
    // Triangle extrusion: 2 caps + 3 sides = 5 faces
    assert_eq!(brep.faces.len(), 5);

    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    // 2 triangle caps (1 tri each) + 3 quad sides (2 tris each) = 8 triangles
    assert_eq!(mesh.triangle_count(), 8);
    mesh.validate().unwrap();
}

#[test]
fn test_sketch_revolve_pipeline() {
    let mut tree = FeatureTree::new();

    // Rectangle sketch in XZ plane, offset from Z axis
    let mut sketch = Sketch::new(Plane {
        origin: Pt3::new(2.0, 0.0, 0.0),
        normal: Vec3::new(0.0, -1.0, 0.0),
        u_axis: Vec3::new(1.0, 0.0, 0.0),
        v_axis: Vec3::new(0.0, 0.0, 1.0),
    });
    let p0 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(0.0, 0.0), // maps to (2, 0, 0)
    });
    let p1 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(2.0, 0.0), // maps to (4, 0, 0)
    });
    let p2 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(2.0, 2.0), // maps to (4, 0, 2)
    });
    let p3 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(0.0, 2.0), // maps to (2, 0, 2)
    });
    sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
    sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
    sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
    sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p1]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p2]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p3]));

    tree.push(Feature::new(
        "sketch-1".into(),
        "Revolve Sketch".into(),
        FeatureKind::Sketch,
        FeatureParams::Placeholder,
    ));
    tree.sketches.insert(0, sketch);

    tree.push(Feature::new(
        "revolve-1".into(),
        "Revolve Full".into(),
        FeatureKind::Revolve,
        FeatureParams::Revolve(RevolveParams {
            axis_origin: Pt3::origin(),
            axis_direction: Vec3::new(0.0, 0.0, 1.0),
            angle: 2.0 * PI,
        }),
    ));

    let brep = evaluate(&mut tree).unwrap();
    // Full revolution of 4-edge profile with 36 segments = 144 faces
    assert_eq!(brep.faces.len(), 144);
    assert!(matches!(brep.body, Body::Solid(_)));

    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    mesh.validate().unwrap();
    assert_eq!(mesh.triangle_count(), 288);
}
