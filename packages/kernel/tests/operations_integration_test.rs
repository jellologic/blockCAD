//! Integration tests for all operations: Sketch -> Feature -> Evaluate -> Tessellate
//!
//! Each test builds a feature tree, evaluates it, tessellates the result,
//! and validates the mesh output.

use blockcad_kernel::feature_tree::evaluator::evaluate;
use blockcad_kernel::feature_tree::{Feature, FeatureKind, FeatureParams, FeatureTree};
use blockcad_kernel::geometry::surface::plane::Plane;
use blockcad_kernel::geometry::{Pt2, Pt3, Vec3};
use blockcad_kernel::operations::chamfer::ChamferParams;
use blockcad_kernel::operations::extrude::ExtrudeParams;
use blockcad_kernel::operations::fillet::FilletParams;
use blockcad_kernel::operations::pattern::circular::CircularPatternParams;
use blockcad_kernel::operations::pattern::linear::LinearPatternParams;
use blockcad_kernel::operations::pattern::mirror::MirrorParams;
use blockcad_kernel::operations::revolve::RevolveParams;
use blockcad_kernel::operations::shell::ShellParams;
use blockcad_kernel::sketch::constraint::{Constraint, ConstraintKind};
use blockcad_kernel::sketch::entity::SketchEntity;
use blockcad_kernel::sketch::Sketch;
use blockcad_kernel::tessellation::{tessellate_brep, TessellationParams};
use blockcad_kernel::topology::body::Body;

fn make_rectangle_sketch() -> Sketch {
    let mut sketch = Sketch::new(Plane::xy(0.0));
    let p0 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
    let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(8.0, 0.5) });
    let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(8.0, 4.0) });
    let p3 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.5, 4.0) });
    let bottom = sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
    let right = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
    let top = sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
    let left = sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![bottom]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![top]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![right]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![left]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 10.0 }, vec![p0, p1]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 5.0 }, vec![p1, p2]));
    sketch
}

fn build_sketch_extrude_tree(depth: f64) -> FeatureTree {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_rectangle_sketch());
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), depth))));
    tree
}

// --- EXTRUDE ---

#[test]
fn e2e_extrude_produces_valid_mesh() {
    let mut tree = build_sketch_extrude_tree(7.0);
    let brep = evaluate(&mut tree).unwrap();
    assert_eq!(brep.faces.len(), 6);
    assert!(matches!(brep.body, Body::Solid(_)));
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    mesh.validate().unwrap();
    assert!(mesh.triangle_count() > 0);
}

// --- CHAMFER ---

#[test]
fn e2e_extrude_then_chamfer() {
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new("ch1".into(), "Chamfer".into(), FeatureKind::Chamfer,
        FeatureParams::Chamfer(ChamferParams { edge_indices: vec![0], distance: 1.0, distance2: None })));
    let brep = evaluate(&mut tree).unwrap();
    assert_eq!(brep.faces.len(), 7); // 6 + 1 chamfer face
    assert!(matches!(brep.body, Body::Solid(_)));
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    mesh.validate().unwrap();
}

// --- FILLET ---

#[test]
fn e2e_extrude_then_fillet() {
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new("f1".into(), "Fillet".into(), FeatureKind::Fillet,
        FeatureParams::Fillet(FilletParams { edge_indices: vec![0], radius: 1.0 })));
    let brep = evaluate(&mut tree).unwrap();
    assert!(brep.faces.len() > 6); // 6 + N fillet segments
    assert!(matches!(brep.body, Body::Solid(_)));
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    mesh.validate().unwrap();
}

// --- LINEAR PATTERN ---

#[test]
fn e2e_extrude_then_linear_pattern() {
    let mut tree = build_sketch_extrude_tree(5.0);
    tree.push(Feature::new("lp1".into(), "Linear Pattern".into(), FeatureKind::LinearPattern,
        FeatureParams::LinearPattern(LinearPatternParams {
            direction: Vec3::new(1.0, 0.0, 0.0),
            spacing: 15.0,
            count: 3,
            direction2: None,
            spacing2: None,
            count2: None,
        })));
    let brep = evaluate(&mut tree).unwrap();
    assert_eq!(brep.faces.len(), 18); // 6 * 3
    assert!(matches!(brep.body, Body::Solid(_)));
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    mesh.validate().unwrap();
}

// --- CIRCULAR PATTERN ---

#[test]
fn e2e_extrude_then_circular_pattern() {
    let mut tree = build_sketch_extrude_tree(5.0);
    tree.push(Feature::new("cp1".into(), "Circular Pattern".into(), FeatureKind::CircularPattern,
        FeatureParams::CircularPattern(CircularPatternParams {
            axis_origin: Pt3::new(0.0, 0.0, 0.0),
            axis_direction: Vec3::new(0.0, 0.0, 1.0),
            count: 4,
            total_angle: 2.0 * std::f64::consts::PI,
        })));
    let brep = evaluate(&mut tree).unwrap();
    assert_eq!(brep.faces.len(), 24); // 6 * 4
    assert!(matches!(brep.body, Body::Solid(_)));
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    mesh.validate().unwrap();
}

// --- MIRROR ---

#[test]
fn e2e_extrude_then_mirror() {
    let mut tree = build_sketch_extrude_tree(5.0);
    tree.push(Feature::new("m1".into(), "Mirror".into(), FeatureKind::Mirror,
        FeatureParams::Mirror(MirrorParams {
            plane_origin: Pt3::new(15.0, 0.0, 0.0),
            plane_normal: Vec3::new(1.0, 0.0, 0.0),
        })));
    let brep = evaluate(&mut tree).unwrap();
    assert_eq!(brep.faces.len(), 12); // 6 * 2
    assert!(matches!(brep.body, Body::Solid(_)));
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    mesh.validate().unwrap();
}

// --- REVOLVE ---

#[test]
fn e2e_revolve_produces_valid_mesh() {
    let mut tree = FeatureTree::new();
    // Create a sketch for revolve (profile offset from Z axis)
    let mut sketch = Sketch::new(Plane {
        origin: Pt3::new(2.0, 0.0, 0.0),
        normal: Vec3::new(0.0, -1.0, 0.0),
        u_axis: Vec3::new(1.0, 0.0, 0.0),
        v_axis: Vec3::new(0.0, 0.0, 1.0),
    });
    let p0 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
    let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(2.0, 0.0) });
    let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(2.0, 2.0) });
    let p3 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 2.0) });
    sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
    sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
    sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
    sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 2.0 }, vec![p0, p1]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 2.0 }, vec![p1, p2]));

    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, sketch);
    tree.push(Feature::new("r1".into(), "Revolve".into(), FeatureKind::Revolve,
        FeatureParams::Revolve(RevolveParams::full(Pt3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)))));

    let brep = evaluate(&mut tree).unwrap();
    assert!(brep.faces.len() > 10); // Full revolve should have many faces
    assert!(matches!(brep.body, Body::Solid(_)));
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    mesh.validate().unwrap();
    assert!(mesh.triangle_count() > 0);
}

// --- CUT EXTRUDE ---

#[test]
fn e2e_extrude_then_cut_extrude() {
    let mut tree = build_sketch_extrude_tree(7.0);

    // Add a second sketch for the cut (smaller rect inside)
    let mut cut_sketch = Sketch::new(Plane::xy(0.0));
    let p0 = cut_sketch.add_entity(SketchEntity::Point { position: Pt2::new(3.0, 1.0) });
    let p1 = cut_sketch.add_entity(SketchEntity::Point { position: Pt2::new(7.0, 1.0) });
    let p2 = cut_sketch.add_entity(SketchEntity::Point { position: Pt2::new(7.0, 4.0) });
    let p3 = cut_sketch.add_entity(SketchEntity::Point { position: Pt2::new(3.0, 4.0) });
    cut_sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
    cut_sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
    cut_sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
    cut_sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });
    cut_sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
    cut_sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 4.0 }, vec![p0, p1]));
    cut_sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 3.0 }, vec![p1, p2]));

    tree.push(Feature::new("s2".into(), "Cut Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(2, cut_sketch);
    tree.push(Feature::new("ce1".into(), "Cut Extrude".into(), FeatureKind::CutExtrude,
        FeatureParams::CutExtrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 7.0))));

    let brep = evaluate(&mut tree).unwrap();
    assert!(brep.faces.len() > 6); // More faces due to cut
    assert!(matches!(brep.body, Body::Solid(_)));
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    mesh.validate().unwrap();
}

// --- SHELL ---

#[test]
fn e2e_extrude_then_shell() {
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new("sh1".into(), "Shell".into(), FeatureKind::Shell,
        FeatureParams::Shell(ShellParams { faces_to_remove: vec![1], thickness: 1.0 })));
    let brep = evaluate(&mut tree).unwrap();
    // 5 outer + 5 inner + 4 rim = 14
    assert_eq!(brep.faces.len(), 14, "Shell should produce 14 faces, got {}", brep.faces.len());
    assert!(matches!(brep.body, Body::Solid(_)));
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    mesh.validate().unwrap();
    assert!(mesh.triangle_count() > 0);
}
