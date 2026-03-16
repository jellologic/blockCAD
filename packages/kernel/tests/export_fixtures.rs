//! Generate STL + JSON fixture files for cross-validation with external CAD tools.
//!
//! These tests export known geometries as STL binary files and their computed
//! mass properties as JSON, which Python/FreeCAD tests then independently validate.
//!
//! Run with: cargo test --test export_fixtures

use blockcad_kernel::feature_tree::evaluator::evaluate;
use blockcad_kernel::feature_tree::{Feature, FeatureKind, FeatureParams, FeatureTree};
use blockcad_kernel::geometry::surface::plane::Plane;
use blockcad_kernel::geometry::{Pt2, Vec3};
use blockcad_kernel::operations::chamfer::ChamferParams;
use blockcad_kernel::operations::extrude::ExtrudeParams;
use blockcad_kernel::operations::fillet::FilletParams;
use blockcad_kernel::sketch::constraint::{Constraint, ConstraintKind};
use blockcad_kernel::sketch::entity::SketchEntity;
use blockcad_kernel::sketch::Sketch;
use blockcad_kernel::tessellation::{compute_mass_properties, tessellate_brep, TessellationParams};
use blockcad_kernel::export::stl::export_stl_binary;

use std::path::Path;

/// Output directory for fixture files (relative to packages/kernel/)
const FIXTURE_DIR: &str = "../../tests/cross-validation/fixtures";

fn ensure_fixture_dir() {
    std::fs::create_dir_all(FIXTURE_DIR).ok();
}

fn write_fixture(name: &str, stl: &[u8], props: &blockcad_kernel::tessellation::MassProperties) {
    ensure_fixture_dir();
    let stl_path = format!("{}/{}.stl", FIXTURE_DIR, name);
    let json_path = format!("{}/{}.json", FIXTURE_DIR, name);
    std::fs::write(&stl_path, stl).unwrap();
    std::fs::write(&json_path, serde_json::to_string_pretty(props).unwrap()).unwrap();
    println!("  Wrote {} ({} bytes) + {}", stl_path, stl.len(), json_path);
}

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

fn make_circle_sketch() -> Sketch {
    let mut sketch = Sketch::new(Plane::xy(0.0));
    let center = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
    sketch.add_entity(SketchEntity::Circle { center, radius: 5.0 });
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![center]));
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

#[test]
fn export_box_fixture() {
    let mut tree = build_sketch_extrude_tree(7.0);
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("box_10x5x7", &stl, &props);

    // Sanity check
    assert!((props.volume - 350.0).abs() < 1.0);
}

#[test]
fn export_fillet_fixture() {
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new("f1".into(), "Fillet".into(), FeatureKind::Fillet,
        FeatureParams::Fillet(FilletParams { edge_indices: vec![0], radius: 1.0 })));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("box_fillet_r1", &stl, &props);

    assert!(props.volume < 350.0, "Fillet should reduce volume");
}

#[test]
fn export_chamfer_fixture() {
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new("c1".into(), "Chamfer".into(), FeatureKind::Chamfer,
        FeatureParams::Chamfer(ChamferParams { edge_indices: vec![0], distance: 1.0, distance2: None })));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("box_chamfer_d1", &stl, &props);

    assert!(props.volume < 350.0, "Chamfer should reduce volume");
}

#[test]
fn export_cylinder_fixture() {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_circle_sketch());
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 10.0))));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("cylinder_r5_h10", &stl, &props);

    let expected = std::f64::consts::PI * 25.0 * 10.0;
    assert!((props.volume - expected).abs() < 30.0, "Cylinder volume should be ~{:.0}", expected);
}
